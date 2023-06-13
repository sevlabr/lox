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
        OpCode::Pop => simple_instruction("OP_POP", offset),
        OpCode::GetLocal => byte_instruction("OP_GET_LOCAL", chunk, offset),
        OpCode::SetLocal => byte_instruction("OP_SET_LOCAL", chunk, offset),
        OpCode::GetGlobal => constant_instruction("OP_GET_GLOBAL", chunk, offset),
        OpCode::DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", chunk, offset),
        OpCode::SetGlobal => constant_instruction("OP_SET_GLOBAL", chunk, offset),
        OpCode::Equal => simple_instruction("OP_EQUAL", offset),
        OpCode::Greater => simple_instruction("OP_GREATER", offset),
        OpCode::Less => simple_instruction("OP_LESS", offset),
        OpCode::Add => simple_instruction("OP_ADD", offset),
        OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
        OpCode::Not => simple_instruction("OP_NOT", offset),
        OpCode::Negate => simple_instruction("OP_NEGATE", offset),
        OpCode::Print => simple_instruction("OP_PRINT", offset),
        OpCode::Jump => jump_instruction("OP_JUMP", 1, chunk, offset),
        OpCode::JumpIfFalse => jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset),
        OpCode::Loop => jump_instruction("OP_LOOP", -1, chunk, offset),
        OpCode::Call => byte_instruction("OP_CALL", chunk, offset),
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

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let mut msg = "Failed to get an index of a variable (out of bounds in chunk.code).".to_string();
    msg += " Expected local or enclosing variable or function name.";
    let slot = chunk.code.get(offset + 1).expect(&msg);
    println!("{:16} {:4}", name, slot);
    offset + 2
}

fn jump_instruction(name: &str, sign: isize, chunk: &Chunk, offset: usize) -> usize {
    let part1 = chunk
        .code
        .get(offset + 1)
        .expect("Failed to get 1st part of jump value.");
    let mut jump = (*part1 as u16) << 8;
    let part2 = chunk
        .code
        .get(offset + 2)
        .expect("Failed to get 2nd part of jump value.");
    jump |= *part2 as u16;
    let dest: isize = (offset as isize) + 3 + sign * (jump as isize);
    println!("{:16} {:4} -> {}", name, offset, dest);
    offset + 3
}
