pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<f64>,
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
            constants: Vec::new(),
        }
    }

    pub fn write_instruction(&mut self, code: OpCode, line: usize) {
        self.code.push(code as u8);
        self.lines.push(line);
    }

    pub fn write_raw_instruction(&mut self, code: u8, line: usize) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn write_value(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    OpConstant,
    OpReturn,
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::OpConstant),
            1 => Ok(OpCode::OpReturn),
            _ => Err("Failed to convert from u8: unknown OpCode."),
        }
    }
}
