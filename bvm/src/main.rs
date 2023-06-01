use bvm::vm::VM;
use bvm::{repl, run_file};
use std::env;
use std::process;

fn main() {
    let vm = VM::default();

    let mut args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(vm),
        2 => run_file(vm, args.swap_remove(1)),
        _ => {
            eprintln!("Usage: lox [path]");
            process::exit(64);
        }
    }
}
