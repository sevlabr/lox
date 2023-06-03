#![feature(linked_list_remove)]

use std::{fs, io::Write, process};
use vm::{InterpretResult, VM};

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod memory;
pub mod object;
pub mod scanner;
pub mod token;
pub mod value;
pub mod vm;

#[derive(Clone, Copy)]
pub struct Config {
    pub bytecode: bool,
    pub debug: bool,
    pub scanner: bool,
    pub trace: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            bytecode: false,
            debug: false,
            scanner: false,
            trace: false,
        }
    }
}

const WELCOME_REPL: &str =
    "Lox language (Crafting Interpreters book: https://craftinginterpreters.com/)
Interactive REPL for Bytecode Virtual Machine
Written in Rust by sevlabr";

pub fn repl(config: Config, mut vm: VM) {
    vm.set_config(config);
    println!("{}", WELCOME_REPL);
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

pub fn run_file(config: Config, mut vm: VM, path: String) {
    vm.set_config(config);
    match fs::read_to_string(path.clone()) {
        Ok(contents) => match vm.interpret(contents) {
            InterpretResult::CompileError => process::exit(65),
            InterpretResult::RuntimeError => process::exit(70),
            _ => (),
        },
        Err(e) => {
            eprintln!("Error during reading file: {e}.\nGiven [PATH]: {}", path);
            process::exit(74);
        }
    }
}
