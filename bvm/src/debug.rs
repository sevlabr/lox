use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    let instruction = chunk
        .code
        .get(offset)
        .expect("Instruction index out of bounds (in chunk.code).");

    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    match OpCode::try_from(*instruction).unwrap() {
        OpCode::Constant => constant_instruction("OP_CONSTANT", chunk, offset),
        OpCode::Nil => simple_instruction("OP_NIL", offset),
        OpCode::True => simple_instruction("OP_TRUE", offset),
        OpCode::False => simple_instruction("OP_FALSE", offset),
        OpCode::Equal => simple_instruction("OP_EQUAL", offset),
        OpCode::Greater => simple_instruction("OP_GREATER", offset),
        OpCode::Less => simple_instruction("OP_Less", offset),
        OpCode::Add => simple_instruction("OP_ADD", offset),
        OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
        OpCode::Not => simple_instruction("OP_NOT", offset),
        OpCode::Negate => simple_instruction("OP_NEGATE", offset),
        OpCode::Return => simple_instruction("OP_RETURN", offset),

        #[allow(unreachable_patterns)]
        _ => {
            eprintln!("Unknown opcode {:?}", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk
        .code
        .get(offset + 1)
        .expect("Failed to get an index of a constant value (out of bounds in chunk.code).");
    let value = chunk
        .constants
        .get(*constant as usize)
        .expect("Failed to get a value of a constant (out of bounds in chunk.constants).");
    println!("{:16} {:4} '{}'", name, constant, value);
    offset + 2
}
