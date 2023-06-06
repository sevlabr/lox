use crate::chunk::{Chunk, OpCode};
use crate::compiler::Parser;
use crate::debug::{disassemble_chunk, disassemble_instruction};
use crate::object::Obj;
use crate::scanner::print_tokens;
use crate::value::Value;
use crate::Config;
use std::collections::{HashMap, LinkedList};
use std::{cell::RefCell, rc::Rc};

const STACK_MAX: usize = 256;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    config: Config,
    chunk: Rc<RefCell<Chunk>>,
    ip: usize,
    stack: Vec<Value>,
    stack_top: usize,

    // maybe use Rc::try_unwrap
    objects: LinkedList<*mut Obj>,

    globals: HashMap<String, Value>,
}

impl Default for VM {
    fn default() -> Self {
        VM {
            config: Config::default(),
            chunk: Rc::new(RefCell::new(Chunk::default())),
            ip: 0,
            stack: vec![Value::Nil; STACK_MAX],
            stack_top: 0,
            objects: LinkedList::new(),
            globals: HashMap::new(),
        }
    }
}

impl VM {
    pub fn new(
        config: Config,
        chunk: Chunk,
        ip: usize,
        stack: Vec<Value>,
        stack_top: usize,
        objects: LinkedList<*mut Obj>,
        globals: HashMap<String, Value>,
    ) -> Self {
        VM {
            config,
            chunk: Rc::new(RefCell::new(chunk)),
            ip,
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

    pub fn set_chunk(&mut self, chunk: Rc<RefCell<Chunk>>) {
        self.chunk = chunk;
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        if self.config.scanner {
            print_tokens(source);
            InterpretResult::Ok
        } else {
            let chunk = Rc::new(RefCell::new(Chunk::new()));
            let mut parser = Parser::new(self.config);
            match parser.compile(source, Rc::clone(&chunk)) {
                Ok(_) => {
                    self.set_chunk(chunk);
                    if self.config.bytecode {
                        disassemble_chunk(&self.chunk.borrow(), "code");
                        InterpretResult::Ok
                    } else {
                        self.ip = 0;
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

                disassemble_instruction(&self.chunk.borrow(), self.ip);
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
        let chunk = self.chunk.borrow();
        let raw_instruction = chunk
            .code
            .get(self.ip)
            .expect("instruction pointer is out of chunk.code bounds.");
        self.ip += 1;
        *raw_instruction
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        self.chunk
            .borrow()
            .constants
            .get(index)
            .expect("Index of a constant value is out of bounds.")
            .clone()
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
        eprintln!("{message}");
        let instruction = self.ip - 1;
        let line = self.chunk.borrow().lines[instruction];
        eprintln!("[line {}] in script", line);
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
