use crate::chunk::OpCode;
use crate::compiler::Parser;
use crate::debug::{disassemble_chunk, disassemble_instruction};
use crate::object::{Closure, Native, Obj, Upvalue};
use crate::scanner::print_tokens;
use crate::value::Value;
use crate::Config;
use std::collections::{HashMap, LinkedList};
use std::{cell::RefCell, rc::Rc};

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * 256;

pub struct CallFrame {
    closure: Rc<RefCell<Closure>>,
    ip: usize,
    slots: usize,
}

impl Default for CallFrame {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Closure::default())), 0, 0)
    }
}

impl CallFrame {
    fn new(closure: Rc<RefCell<Closure>>, ip: usize, slots: usize) -> Self {
        Self { closure, ip, slots }
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

    open_upvalues: Option<Rc<RefCell<Upvalue>>>,

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
            open_upvalues: None,
            objects: LinkedList::new(),
            globals: HashMap::new(),
        }
    }
}

impl VM {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frames: [CallFrame; FRAMES_MAX],
        frame_count: isize,
        config: Config,
        stack: Vec<Value>,
        stack_top: usize,
        open_upvalues: Option<Rc<RefCell<Upvalue>>>,
        objects: LinkedList<*mut Obj>,
        globals: HashMap<String, Value>,
    ) -> Self {
        VM {
            frames,
            frame_count,
            config,
            stack,
            stack_top,
            open_upvalues,
            objects,
            globals,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn init(&mut self) {
        self.reset_stack();

        self.define_native("clock");
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.frame_count = 0;
        self.open_upvalues = None;
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
                    self.push(Value::Obj(Obj::Fun((*function).borrow().clone())));
                    let closure = Closure::new(&function);
                    let closure_ref = Rc::new(RefCell::new(closure.clone()));
                    self.pop();
                    self.push(Value::Obj(Obj::Closure(closure)));

                    // self.call(function.clone() // .borrow().clone(), 0);
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    self.frame_count += 1;
                    frame.closure = closure_ref;
                    frame.ip = 0;
                    frame.slots = 0;

                    if self.config.bytecode {
                        let chunk = (*frame.closure).borrow().chunk();
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
                let chunk = (*frame.closure).borrow().chunk();
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
                OpCode::GetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    let upvalue = frame.closure.borrow().upvalue(slot);
                    let upvalue = upvalue.borrow();
                    let value = if upvalue.is_closed() {
                        upvalue.closed_value()
                    } else {
                        let location = upvalue.location();
                        self.stack[location].clone()
                    };
                    self.push(value);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let frame = self
                        .frames
                        .get_mut(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    let upvalue = frame.closure.borrow().upvalue(slot);
                    let mut upvalue = upvalue.borrow_mut();
                    let value = self.peek(0);
                    if upvalue.is_closed() {
                        upvalue.set_closed_value(Box::new(value));
                    } else {
                        let location = upvalue.location();
                        self.stack[location] = value;
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
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    if !self.call_value(self.peek(arg_count), arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Closure => {
                    let function = self.read_constant();
                    if function.is_obj_type("Function") {
                        let function = unsafe { function.as_obj().as_fun() };
                        let function = Rc::new(RefCell::new(function));
                        let mut closure = Closure::new(&function);
                        self.push(Value::Obj(Obj::Closure(closure.clone())));

                        for i in 0..closure.upvalue_count() as usize {
                            let is_local = self.read_byte();
                            let index = self.read_byte() as usize;
                            let frame = self
                                .frames
                                .get_mut(self.frame_count as usize - 1)
                                .expect("Instruction pointer is out of vm.frames bounds.");
                            match is_local {
                                1 => {
                                    let frame_slots = frame.slots;
                                    let upvalue = self.capture_upvalue(frame_slots + index);
                                    closure.set_upvalue(i, upvalue);
                                }
                                0 => {
                                    let upvalue = frame.closure.borrow().upvalue(index);
                                    closure.set_upvalue(i, upvalue)
                                }
                                _ => unreachable!("`is_local` can be either 0 or 1."),
                            }
                        }

                        self.pop();
                        self.push(Value::Obj(Obj::Closure(closure)));
                    }
                }
                OpCode::CloseUpvalue => {
                    let location = self.stack_top - 1;
                    self.close_upvalues(location);
                    self.pop();
                }
                OpCode::Return => {
                    let result = self.pop();
                    let frame = self
                        .frames
                        .get(self.frame_count as usize - 1)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    self.close_upvalues(frame.slots);
                    self.frame_count -= 1;
                    if self.frame_count == 0 {
                        self.pop();
                        return Ok(InterpretResult::Ok);
                    }

                    let frame = self
                        .frames
                        .get(self.frame_count as usize)
                        .expect("Instruction pointer is out of vm.frames bounds.");
                    self.stack_top = frame.slots;
                    self.push(result);
                }

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
        let chunk = (*frame.closure).borrow().chunk();
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
        let chunk = (*frame.closure).borrow().chunk();
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
        let chunk = (*frame.closure).borrow().chunk();
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

    fn call(&mut self, closure: Closure, arg_count: usize) -> bool {
        let function = closure.function();
        let function = function.borrow();
        if arg_count != function.arity() as usize {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.",
                function.arity(),
                arg_count
            ));
            return false;
        }

        if self.frame_count as usize == FRAMES_MAX {
            self.runtime_error("Stack overflow.".to_string());
            return false;
        }

        let frame = self
            .frames
            .get_mut(self.frame_count as usize)
            .expect("Instruction pointer is out of vm.frames bounds.");
        self.frame_count += 1;
        frame.closure = Rc::new(RefCell::new(closure));
        frame.ip = 0;
        frame.slots = self.stack_top - arg_count - 1;
        true
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if callee.is_obj() {
            match callee {
                Value::Obj(Obj::Closure(callee)) => {
                    return self.call(callee, arg_count);
                }
                Value::Obj(Obj::BuiltIn(native)) => {
                    let result = native.call(arg_count, self.stack_top - arg_count);
                    self.stack_top -= arg_count + 1;
                    self.push(result);
                    return true;
                }
                // Non-callable object type.
                _ => (),
            }
        }
        self.runtime_error("Can only call functions and classes.".to_string());
        false
    }

    fn capture_upvalue(&mut self, local: usize) -> Rc<RefCell<Upvalue>> {
        let mut prev_upvalue = None;
        let mut upvalue = self.open_upvalues.as_ref().map(Rc::clone);
        while upvalue.is_some() && upvalue.as_ref().unwrap().borrow().location() > local {
            prev_upvalue = Some(Rc::clone(upvalue.as_ref().unwrap()));
            upvalue = upvalue.unwrap().borrow().next();
        }

        if let Some(ref upval) = upvalue {
            if upval.borrow().location() == local {
                return Rc::clone(upval);
            }
        }

        let mut created_upvalue = Upvalue::new(local);
        created_upvalue.set_next(upvalue);
        let created_upvalue = Rc::new(RefCell::new(created_upvalue));

        if prev_upvalue.is_none() {
            self.open_upvalues = Some(Rc::clone(&created_upvalue));
        } else {
            prev_upvalue
                .as_ref()
                .unwrap()
                .borrow_mut()
                .set_next(Some(Rc::clone(&created_upvalue)));
        }

        created_upvalue
    }

    fn close_upvalues(&mut self, last: usize) {
        while self.open_upvalues.is_some()
            && self.open_upvalues.as_ref().unwrap().borrow().location() >= last
        {
            let upvalue = Rc::clone(self.open_upvalues.as_ref().unwrap());
            let location = upvalue.borrow().location();
            let value = Box::new(self.stack[location].clone());
            upvalue.borrow_mut().set_closed_value(value);
            upvalue.borrow_mut().set_closed();
            self.open_upvalues = upvalue.borrow().next();
        }
    }

    fn define_native(&mut self, name: &str) {
        let name = name.to_string();
        self.push(Value::Obj(Obj::Str(name.clone())));
        self.push(Value::Obj(Obj::BuiltIn(Native::new(name))));
        let name = unsafe { self.stack[0].as_obj().as_string() };
        let native_fun = self.stack[1].clone();
        self.globals.insert(name, native_fun);
        self.pop();
        self.pop();
    }

    fn runtime_error(&mut self, message: String) {
        eprintln!("\nRuntimeError: {message}");
        for frame in self.frames.iter().take(self.frame_count as usize).rev() {
            let closure = frame.closure.borrow();
            let function = closure.function();
            let function = function.borrow();
            let instruction = frame.ip - 1;
            let chunk = function.chunk();
            let chunk = chunk.borrow();
            let line = chunk.lines[instruction];
            eprint!("[line {}] in ", line);
            if function.name().is_empty() {
                eprintln!("script");
            } else {
                eprintln!("{}()", function.name());
            }
        }

        if self.config.debug {
            let frame = self
                .frames
                .get(self.frame_count as usize - 1)
                .expect("Instruction pointer is out of vm.frames bounds.");
            let chunk = (*frame.closure).borrow().chunk();
            let chunk = (*chunk).borrow();

            let function = (*frame.closure).borrow().function();
            let name = if function.borrow().name().is_empty() {
                "<script>".to_string()
            } else {
                function.borrow().name()
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
    #[allow(clippy::unused_unit)]
    fn _free_obj(&mut self, index: usize) {
        let loc = self.objects.remove(index);
        if !loc.is_null() {
            unsafe {
                match *loc {
                    Obj::BuiltIn(_) => (),
                    Obj::Closure(_) => {
                        // free Vec with upvalues.
                        ()
                    }
                    Obj::Fun(ref mut fun) => {
                        fun.free();
                    }
                    Obj::Str(ref mut s) => {
                        s.clear() // This does not do the job.
                                  // s.zeroize();
                    }
                    // Do smth reasonable here.
                    Obj::Upval(ref _value) => {
                        ()
                        // let value = value.location();
                        // let value = value.borrow();
                        // match value {
                        //     _ => (),
                        // }
                    }
                }
            }
        } else {
            panic!("Detected attempt to dereference a null-pointer.");
        }
    }
}
