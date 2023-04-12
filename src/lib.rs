pub mod ast;
pub mod lexer;

use lexer::scanner::Scanner;
use std::fs;
use std::process;
use std::io::{self, Write};

pub struct Lox {
    had_error: bool
}

impl Lox {
    pub fn new() -> Self { Lox { had_error: false } }

    pub fn run_file(&mut self, path: &str) {
        let contents = fs::read_to_string(path)
            .expect("Couldn't read the given file!");
        self.run(contents);
    
        // Indicate an error in the exit code.
        if self.had_error { process::exit(65) };
    }
    
    pub fn run_promt(&mut self) {
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
    
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => self.run(line),
                Err(e) => {
                    eprintln!("Error during reading prompt: {e}");
                    break
                },
            }
    
            self.had_error = false;
        }
    }
    
    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(self, &source);
        scanner.scan_tokens();
    
        for token in scanner.tokens() {
            println!("{token}");
        }
    
        println!("{source}");
    }

    pub fn error(&mut self, line: usize, msg: &str) {
        self.report(line, "", msg);
    }

    fn report(&mut self, line: usize, err_type: &str, msg: &str) {
        println!("[line {line}] Error{err_type}: {msg}");
        self.had_error = true;
    }
}
