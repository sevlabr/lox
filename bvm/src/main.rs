use bvm::chunk::{Chunk, OpCode};
use bvm::debug::disassemble_chunk;
use bvm::vm::VM;

fn main() {
    let mut vm = VM::default();
    let mut chunk = Chunk::new();

    let mut constant = chunk.write_value(1.2);
    chunk.write_instruction(OpCode::Constant, 123);
    chunk.write_raw_instruction(constant as u8, 123);

    constant = chunk.write_value(3.4);
    chunk.write_instruction(OpCode::Constant, 123);
    chunk.write_raw_instruction(constant as u8, 123);

    chunk.write_instruction(OpCode::Add, 123);

    constant = chunk.write_value(5.6);
    chunk.write_instruction(OpCode::Constant, 123);
    chunk.write_raw_instruction(constant as u8, 123);

    chunk.write_instruction(OpCode::Divide, 123);
    chunk.write_instruction(OpCode::Negate, 123);

    chunk.write_instruction(OpCode::Return, 123);

    disassemble_chunk(&chunk, "test chunk");

    println!("\nStart execution.");
    vm.interpret(chunk);
}
