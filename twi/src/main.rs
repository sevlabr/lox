use std::env;
use std::process;
use twi::evaluator::{environment::Environment, Evaluator};
use twi::Lox;

fn main() {
    let environment = Environment::new(None);
    let evaluator = Evaluator::new(environment);
    let mut interpreter = Lox::new(evaluator);

    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => match args[2].as_str() {
            "-p" => interpreter.run_ast_print(&args[1], false),
            "-v" => interpreter.run_ast_print(&args[1], true),
            _ => {
                eprintln!("Available commands: -v | -p");
                process::exit(64);
            }
        },
        2 => interpreter.run_file(&args[1]),
        1 => interpreter.run_promt(),
        _ => {
            eprintln!("Usage: lox [script]");
            process::exit(64);
        }
    }
}
