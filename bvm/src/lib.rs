use std::{fs, io::Write, process};
use vm::{InterpretResult, VM};

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod scanner;
pub mod token;
pub mod vm;

const DEBUG_TRACE_EXECUTION: bool = true;

pub fn repl(mut vm: VM) {
    loop {
        print!("> ");

        std::io::stdout()
            .flush()
            .expect("Error during buffer flushing.");

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => {
                if line.is_empty() {
                    println!();
                    break;
                } else {
                    vm.interpret(line);
                }
            }
            Err(e) => {
                eprintln!("Error during reading prompt: {e}");
                break;
            }
        }
    }
}

pub fn run_file(mut vm: VM, path: String) {
    match fs::read_to_string(path) {
        Ok(contents) => match vm.interpret(contents) {
            InterpretResult::CompileError => process::exit(65),
            InterpretResult::RuntimeError => process::exit(70),
            _ => (),
        },
        Err(e) => {
            eprintln!("Error during reading file: {e}");
            process::exit(74);
        }
    }
}
