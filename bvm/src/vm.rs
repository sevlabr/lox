use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_instruction;
use crate::DEBUG_TRACE_EXECUTION;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: [f64; STACK_MAX],
    stack_top: usize,
}

impl Default for VM {
    fn default() -> Self {
        VM {
            chunk: Chunk::default(),
            ip: 0,
            stack: [0f64; STACK_MAX],
            stack_top: 0,
        }
    }
}

impl VM {
    pub fn new(chunk: Chunk, ip: usize, stack: [f64; STACK_MAX], stack_top: usize) -> Self {
        VM {
            chunk,
            ip,
            stack,
            stack_top,
        }
    }

    pub fn init(&mut self) {
        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn push(&mut self, value: f64) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> f64 {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    pub fn set_chunk(&mut self, chunk: Chunk) {
        self.chunk = chunk;
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.set_chunk(chunk);
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for (i, val) in self.stack.iter().enumerate() {
                    if i < self.stack_top {
                        print!("[ ");
                        print!("{}", val);
                        print!(" ]");
                    }
                }
                println!();

                disassemble_instruction(&self.chunk, self.ip);
            }

            let raw_instruction = self.read_byte();
            let instruction = OpCode::try_from(raw_instruction).unwrap();

            match instruction {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::Add => self.binary_op("+"),
                OpCode::Subtract => self.binary_op("-"),
                OpCode::Multiply => self.binary_op("*"),
                OpCode::Divide => self.binary_op("/"),
                OpCode::Negate => {
                    let val = self.pop();
                    self.push(-val);
                }
                OpCode::Return => {
                    println!("{}", self.pop());
                    return InterpretResult::Ok;
                }

                #[allow(unreachable_patterns)]
                _ => {
                    eprintln!("Unknown opcode {:?}", instruction);
                    return InterpretResult::RuntimeError;
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let raw_instruction = self
            .chunk
            .code
            .get(self.ip)
            .expect("instruction pointer is out of chunk.code bounds.");
        self.ip += 1;
        *raw_instruction
    }

    fn read_constant(&mut self) -> f64 {
        let index = self.read_byte() as usize;
        *self
            .chunk
            .constants
            .get(index)
            .expect("Index of a constant value is out of bounds.")
    }

    fn binary_op(&mut self, op: &str) {
        let b = self.pop();
        let a = self.pop();
        match op {
            "+" => self.push(a + b),
            "-" => self.push(a - b),
            "*" => self.push(a * b),
            "/" => self.push(a / b),
            _ => {
                panic!("Unknown binary operation: {}", op);
            }
        }
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
