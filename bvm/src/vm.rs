use crate::chunk::OpCode;
use crate::compiler::Parser;
use crate::debug::{disassemble_chunk, disassemble_instruction};
use crate::object::{Function, Obj};
use crate::scanner::print_tokens;
use crate::value::Value;
use crate::Config;
use std::collections::{HashMap, LinkedList};
use std::{cell::RefCell, rc::Rc};

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * 256;

pub struct CallFrame {
    function: Rc<RefCell<Function>>,
    ip: usize,
    slots: usize,
}

impl Default for CallFrame {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Function::default())), 0, 0)
    }
}

impl CallFrame {
    fn new(function: Rc<RefCell<Function>>, ip: usize, slots: usize) -> Self {
        Self {
            function,
            ip,
            slots,
        }
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    frames: [CallFrame; FRAMES_MAX],
    frame_count: isize,

    config: Config,

    stack: Vec<Value>,
    stack_top: usize,

    // maybe use Rc::try_unwrap
    objects: LinkedList<*mut Obj>,

    globals: HashMap<String, Value>,
}

impl Default for VM {
    fn default() -> Self {
        VM {
            frames: [(); FRAMES_MAX].map(|_| CallFrame::default()),
            frame_count: 0,
            config: Config::default(),
            stack: vec![Value::Nil; STACK_MAX],
            stack_top: 0,
            objects: LinkedList::new(),
            globals: HashMap::new(),
        }
    }
}

impl VM {
    pub fn new(
        frames: [CallFrame; FRAMES_MAX],
        frame_count: isize,
        config: Config,
        stack: Vec<Value>,
        stack_top: usize,
        objects: LinkedList<*mut Obj>,
        globals: HashMap<String, Value>,
    ) -> Self {
        VM {
            frames,
            frame_count,
            config,
            stack,
            stack_top,
            objects,
            globals,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn init(&mut self) {
        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    // Maybe add check of a value type here to add it to `self.objects`,
    // so that GC can delete it later.
    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].clone()
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        if self.config.scanner {
            print_tokens(source);
            InterpretResult::Ok
        } else {
            let mut parser = Parser::new(self.config);
            match parser.compile(source) {
                Ok(function) => {
                    self.push(Value::Obj(Obj::Fun(function.borrow().clone())));
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    self.frame_count += 1;
                    frame.function = function;
                    frame.ip = 0;
                    frame.slots = 0;

                    if self.config.bytecode {
                        let chunk = (*frame.function).borrow().chunk();
                        let chunk = (*chunk).borrow();
                        disassemble_chunk(&chunk, "code");
                        InterpretResult::Ok
                    } else {
                        match self.run() {
                            Ok(result) => result,
                            Err(result) => result,
                        }
                    }
                }
                Err(_err) => {
                    // eprintln!("{_err}");
                    InterpretResult::CompileError
                }
            }
        }
    }

    fn run(&mut self) -> Result<InterpretResult, InterpretResult> {
        loop {
            if self.config.trace {
                print!("          ");
                for (i, val) in self.stack.iter().enumerate() {
                    if i < self.stack_top {
                        print!("[ ");
                        print!("{}", val);
                        print!(" ]");
                    }
                }
                println!();

                let frame = self
                    .frames
                    .get(self.frame_count as usize - 1)
                    .expect("Instruction pointer is out of vm.frames bounds.");
                let chunk = (*frame.function).borrow().chunk();
                let chunk = (*chunk).borrow();
                disassemble_instruction(&chunk, frame.ip);
            }

            let raw_instruction = self.read_byte();
            let instruction = OpCode::try_from(raw_instruction).unwrap();

            match instruction {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    let val = self.stack[frame.slots + slot as usize].clone();
                    self.push(val);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    let value = self.peek(0);
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    self.stack[frame.slots + slot as usize] = value;
                }
                OpCode::GetGlobal => {
                    // Safe to not check if it is a string,
                    // because compiler never emits an instruction
                    // that refers to a non-string constant.
                    let name = unsafe { self.read_constant().as_obj().as_string() };
                    let value = self.globals.get(&name);
                    match value {
                        Some(val) => self.push(val.clone()),
                        None => {
                            self.runtime_error(format!("Undefined variable '{}'.", name));
                            return Err(InterpretResult::RuntimeError);
                        }
                    }
                }
                OpCode::DefineGlobal => {
                    // See comment for GetGlobal.
                    let name = unsafe { self.read_constant().as_obj().as_string() };
                    self.globals.insert(name, self.peek(0));
                    self.pop();
                }
                OpCode::SetGlobal => {
                    // See comment for GetGlobal.
                    let name = unsafe { self.read_constant().as_obj().as_string() };
                    if self.globals.insert(name.clone(), self.peek(0)).is_none() {
                        self.globals.remove(&name);
                        self.runtime_error(format!("Undefined variable '{}'.", name));
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a.equal(b)));
                }
                OpCode::Greater => self.binary_op(">")?,
                OpCode::Less => self.binary_op("<")?,
                OpCode::Add => self.binary_op("+")?,
                OpCode::Subtract => self.binary_op("-")?,
                OpCode::Multiply => self.binary_op("*")?,
                OpCode::Divide => self.binary_op("/")?,
                OpCode::Not => {
                    let new_val = self.pop().is_falsey();
                    self.push(Value::Bool(new_val))
                }
                OpCode::Negate => {
                    if !self.peek(0).is_num() {
                        self.runtime_error("Operand must be a number.".to_string());
                        return Err(InterpretResult::RuntimeError);
                    }
                    let val = self.pop();
                    let val_f64 = unsafe { val.as_num() };
                    self.push(Value::Num(-val_f64));
                }
                OpCode::Print => {
                    println!("{}", self.pop());
                }
                OpCode::Jump => {
                    let offset: u16 = self.read_short();
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    frame.ip += offset as usize;
                }
                OpCode::JumpIfFalse => {
                    let offset: u16 = self.read_short();
                    if self.peek(0).is_falsey() {
                        let frame = self
                            .frames
                            .get_mut(self.frame_count as usize - 1)
                            .expect("Instruction pointer is out of vm.frames bounds.");
                        frame.ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset: u16 = self.read_short();
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    frame.ip -= offset as usize;
                }
                OpCode::Return => {
                    return Ok(InterpretResult::Ok);
                }

                #[allow(unreachable_patterns)]
                _ => {
                    eprintln!("Unknown opcode {:?}", instruction);
                    return Err(InterpretResult::RuntimeError);
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self
            .frames
            .get_mut(self.frame_count as usize - 1)
            .expect("Instruction pointer is out of vm.frames bounds.");
        let ip = &mut frame.ip;
        let chunk = (*frame.function).borrow().chunk();
        let chunk = (*chunk).borrow();
        let raw_instruction = chunk
            .code
            .get(*ip)
            .expect("Instruction pointer is out of chunk.code bounds.");
        *ip += 1;
        *raw_instruction
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        let frame = self
            .frames
            .get(self.frame_count as usize - 1)
            .expect("Instruction pointer is out of vm.frames bounds.");
        let chunk = (*frame.function).borrow().chunk();
        let chunk = (*chunk).borrow();
        chunk
            .constants
            .get(index)
            .expect("Index of a constant value is out of bounds.")
            .clone()
    }

    fn read_short(&mut self) -> u16 {
        let frame = self
            .frames
            .get_mut(self.frame_count as usize - 1)
            .expect("Instruction pointer is out of vm.frames bounds.");
        let ip = &mut frame.ip;
        *ip += 2;
        let chunk = (*frame.function).borrow().chunk();
        let chunk = (*chunk).borrow();
        let offset = chunk.code[*ip - 2] as u16;
        (offset << 8) | (chunk.code[*ip - 1] as u16)
    }

    fn binary_op(&mut self, op: &str) -> Result<(), InterpretResult> {
        if op == "+" {
            return self.binary_plus();
        }
        if !self.peek(0).is_num() || !self.peek(1).is_num() {
            self.runtime_error("Operands must be numbers.".to_string());
            return Err(InterpretResult::RuntimeError);
        }
        let b = unsafe { self.pop().as_num() };
        let a = unsafe { self.pop().as_num() };
        match op {
            ">" => self.push(Value::Bool(a > b)),
            "<" => self.push(Value::Bool(a < b)),
            "-" => self.push(Value::Num(a - b)),
            "*" => self.push(Value::Num(a * b)),
            "/" => self.push(Value::Num(a / b)),
            _ => {
                panic!("Unknown binary operation: {}", op);
            }
        }
        Ok(())
    }

    fn binary_plus(&mut self) -> Result<(), InterpretResult> {
        if self.peek(0).is_obj_type("String") && self.peek(1).is_obj_type("String") {
            // Concatenation
            let b = unsafe { self.pop().as_obj().as_string() };
            let a = unsafe { self.pop().as_obj().as_string() };
            let res = self.allocate_obj(Obj::Str(a + &b));
            self.push(Value::Obj(res));
        } else if self.peek(0).is_num() && self.peek(1).is_num() {
            let b = unsafe { self.pop().as_num() };
            let a = unsafe { self.pop().as_num() };
            self.push(Value::Num(a + b));
        } else {
            self.runtime_error("Operands must be two numbers or two strings.".to_string());
            return Err(InterpretResult::RuntimeError);
        }
        Ok(())
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack_top - 1 - distance].clone()
    }

    fn runtime_error(&mut self, message: String) {
        let frame = self
            .frames
            .get(self.frame_count as usize - 1)
            .expect("Instruction pointer is out of vm.frames bounds.");
        let chunk = (*frame.function).borrow().chunk();
        let chunk = (*chunk).borrow();

        eprintln!("{message}");
        let instruction = frame.ip - 1;
        let line = chunk.lines[instruction];
        eprintln!("[line {}] in script", line);

        if self.config.debug {
            let name = if frame.function.borrow().name().is_empty() {
                "<script>".to_string()
            } else {
                frame.function.borrow().name()
            };
            println!();
            disassemble_chunk(&chunk, &name)
        }

        self.reset_stack();
    }

    // Start tracking created object to be able to deallocate it with GC later.
    fn allocate_obj(&mut self, mut obj: Obj) -> Obj {
        self.objects.push_front(&mut obj as *mut _);
        obj
    }

    // Example function for deallocation. May change later.
    // Supposed to be used for GC.
    // This variant is less safe than the other.
    fn _deallocate_obj(&mut self, index: usize) {
        use std::alloc::{dealloc, Layout};

        let loc = self.objects.remove(index) as *mut u8;
        if !loc.is_null() {
            unsafe { dealloc(loc, Layout::new::<Obj>()) }
        } else {
            panic!("Detected attempt to dereference a null-pointer.");
        }
    }

    // Example function for deallocation. May change later.
    // Supposed to be used for GC.
    // This variant is more safe than the other.
    // (Also check Zeroize crate if needed).
    fn _free_obj(&mut self, index: usize) {
        let loc = self.objects.remove(index);
        if !loc.is_null() {
            unsafe {
                match *loc {
                    Obj::Fun(ref mut fun) => {
                        fun.free();
                    }
                    Obj::Str(ref mut s) => {
                        s.clear() // This does not do the job.
                                  // s.zeroize();
                    }
                }
            }
        } else {
            panic!("Detected attempt to dereference a null-pointer.");
        }
    }
}
