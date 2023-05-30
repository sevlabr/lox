use bvm::chunk::{Chunk, OpCode};
use bvm::debug::disassemble_chunk;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.write_value(1.2);
    chunk.write_instruction(OpCode::OpConstant, 123);
    chunk.write_raw_instruction(constant as u8, 123);

    chunk.write_instruction(OpCode::OpReturn, 123);

    disassemble_chunk(&chunk, "test chunk");
}
