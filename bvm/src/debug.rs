use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        let instruction = chunk
            .code
            .get(offset)
            .expect("Instruction index out of bounds (in chunk.code).");
        offset = disassemble_instruction(OpCode::try_from(*instruction).unwrap(), chunk, offset);
    }
}

fn disassemble_instruction(instruction: OpCode, chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    match instruction {
        OpCode::OpReturn => simple_instruction("OP_RETURN", offset),
        OpCode::OpConstant => constant_instruction("OP_CONSTANT", chunk, offset),

        #[allow(unreachable_patterns)]
        _ => {
            println!("Unknown opcode {:?}", instruction);
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
