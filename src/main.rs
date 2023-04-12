use lox::Lox;
use std::env;
use std::process;

fn main() {
    let mut interpreter = Lox::new();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => interpreter.run_file(&args[1]),
        1 => interpreter.run_promt(),
        _ => {
            println!("Usage: twilox [script]");
            process::exit(64);
        },
    }
}
