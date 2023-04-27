use twi::evaluator::{environment::Environment, Evaluator};
use twi::Lox;
use std::env;
use std::process;

fn main() {
    let environment = Environment::new(None);
    let evaluator = Evaluator::new(environment);
    let mut interpreter = Lox::new(evaluator);

    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => interpreter.run_file(&args[1]),
        1 => interpreter.run_promt(),
        _ => {
            eprintln!("Usage: lox [script]");
            process::exit(64);
        }
    }
}
