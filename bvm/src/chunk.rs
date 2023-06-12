use crate::value::Value;

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<isize>,
    pub constants: Vec<Value>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::with_capacity(u8::MAX.into()),
        }
    }

    pub fn write_instruction(&mut self, code: OpCode, line: isize) {
        self.code.push(code as u8);
        self.lines.push(line);
    }

    pub fn write_raw_instruction(&mut self, code: u8, line: isize) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn write_value(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    // Example function for deallocation. May change later.
    // Supposed to be used for GC.
    // (Also check Zeroize crate if needed since this implementation
    // actually doesn't free anything).
    pub fn free(&mut self) {
        self.code.clear();
        self.lines.clear();
        self.constants.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetUpvalue,
    SetUpvalue,
    GetProperty,
    SetProperty,
    GetSuper,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Invoke,
    SuperInvoke,
    Closure,
    CloseUpvalue,
    Return,
    Class,
    Inherit,
    Method,
}

/// `num_enum` crate is better solution here.
impl TryFrom<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Constant),
            1 => Ok(OpCode::Nil),
            2 => Ok(OpCode::True),
            3 => Ok(OpCode::False),
            4 => Ok(OpCode::Pop),
            5 => Ok(OpCode::GetLocal),
            6 => Ok(OpCode::SetLocal),
            7 => Ok(OpCode::GetGlobal),
            8 => Ok(OpCode::DefineGlobal),
            9 => Ok(OpCode::SetGlobal),
            10 => Ok(OpCode::GetUpvalue),
            11 => Ok(OpCode::SetUpvalue),
            12 => Ok(OpCode::GetProperty),
            13 => Ok(OpCode::SetProperty),
            14 => Ok(OpCode::GetSuper),
            15 => Ok(OpCode::Equal),
            16 => Ok(OpCode::Greater),
            17 => Ok(OpCode::Less),
            18 => Ok(OpCode::Add),
            19 => Ok(OpCode::Subtract),
            20 => Ok(OpCode::Multiply),
            21 => Ok(OpCode::Divide),
            22 => Ok(OpCode::Not),
            23 => Ok(OpCode::Negate),
            24 => Ok(OpCode::Print),
            25 => Ok(OpCode::Jump),
            26 => Ok(OpCode::JumpIfFalse),
            27 => Ok(OpCode::Loop),
            28 => Ok(OpCode::Call),
            29 => Ok(OpCode::Invoke),
            30 => Ok(OpCode::SuperInvoke),
            31 => Ok(OpCode::Closure),
            32 => Ok(OpCode::CloseUpvalue),
            33 => Ok(OpCode::Return),
            34 => Ok(OpCode::Class),
            35 => Ok(OpCode::Inherit),
            36 => Ok(OpCode::Method),
            _ => {
                eprintln!("Code value: {}.", value);
                Err("Failed to convert from u8: unknown OpCode.")
            }
        }
    }
}
