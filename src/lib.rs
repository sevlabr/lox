pub mod ast;
pub mod evaluator;
pub mod lexer;
pub mod parser;

use ast::AstPrinter;
use evaluator::{Evaluator, RuntimeError};
use lexer::scanner::Scanner;
use lexer::token::{Token, TokenType};
use parser::Parser;
use std::fs;
use std::io::{self, Write};
use std::process;

pub trait Visitor<T1, T2> {
    fn visit_expr(&mut self, e: &ast::expr::Expr) -> T1;
    fn visit_stmt(&mut self, s: &ast::stmt::Stmt) -> T2;
}

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,

    evaluator: Evaluator,
}

impl Lox {
    pub fn new(evaluator: Evaluator) -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
            evaluator,
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let contents = fs::read_to_string(path).expect("Couldn't read the given file!");
        self.run(contents);

        // Indicate an error in the exit code.
        if self.had_error {
            process::exit(65)
        }
        if self.had_runtime_error {
            process::exit(70)
        }
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
                    break;
                }
            }

            self.had_error = false;
        }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(self, &source);
        scanner.scan_tokens();

        let tokens = scanner.tokens().clone();
        let mut parser = Parser::new(self, tokens);
        let statements = parser.parse();

        // Stop if there was a syntax error.
        if self.had_error {
            return;
        }

        for statement in statements {
            match statement {
                Some(s) => match self.evaluator.execute(&s) {
                    Ok(_) => (),
                    Err(err) => {
                        self.runtime_error(err);
                        println!("Failed expression evaluation!");
                    }
                },
                None => println!("Found None instead of Stmt while evaluation!"),
            }
        }
    }

    fn _run_ast_print(&mut self, source: String) {
        let mut scanner = Scanner::new(self, &source);
        scanner.scan_tokens();

        let tokens = scanner.tokens().clone();
        let mut parser = Parser::new(self, tokens);
        let statements = parser.parse();

        // Stop if there was a syntax error.
        if self.had_error {
            return;
        }

        let mut printer = AstPrinter;
        for statement in statements {
            match statement {
                Some(s) => {
                    println!("{}", printer.visit_stmt(&s))
                }
                None => println!("Failed while printing AST. (None Stmt)."),
            }
        }
    }

    fn _run_lex_print(&mut self, source: String) {
        let mut scanner = Scanner::new(self, &source);
        scanner.scan_tokens();

        for token in scanner.tokens() {
            println!("{token}");
        }

        println!("{source}");
    }

    // TODO: blend lex_error and error together
    pub fn lex_error(&mut self, line: usize, msg: &str) {
        self.report(line, "", msg);
    }

    pub fn runtime_error(&mut self, err: RuntimeError) {
        let mut err_msg = err.get_message();
        let line = err.get_token().get_line();
        err_msg.push_str("\n[line ");
        err_msg.push_str(&line.to_string());
        err_msg.push(']');

        println!("{err_msg}");

        self.had_runtime_error = true;
    }

    pub fn error(&mut self, token: &Token, msg: &str) {
        if *token.get_type() == TokenType::Eof {
            self.report(token.get_line(), " at end", msg)
        } else {
            let err_type = format!(" at '{}'", token.get_lexeme());
            self.report(token.get_line(), &err_type, msg)
        }
    }

    fn report(&mut self, line: usize, err_type: &str, msg: &str) {
        println!("[line {line}] Error{err_type}: {msg}");
        self.had_error = true;
    }
}

#[cfg(test)]
mod test_parser_basic {
    use crate::evaluator::environment::Environment;
    use crate::{AstPrinter, Evaluator, Lox, Parser, Scanner, Visitor};

    fn run(source: &str) -> String {
        let environment = Environment::new();
        let evaluator = Evaluator::new(environment);
        let mut interpreter = Lox::new(evaluator);

        let mut scanner = Scanner::new(&mut interpreter, &source);
        scanner.scan_tokens();

        let tokens = scanner.tokens().clone();
        let mut parser = Parser::new(&mut interpreter, tokens);
        let statements = parser.parse();

        let mut printer = AstPrinter;
        for statement in statements {
            match statement {
                Some(s) => return format!("{}", printer.visit_stmt(&s)),
                None => panic!("Failed while printing AST. (None Stmt)."),
            }
        }

        panic!("Failed while printing AST.");
    }

    #[test]
    fn test_basic() {
        let source = "print 1 - (2 + 3) / 7 != \"name\";";
        assert_eq!(run(source), "(print (!= (- 1 (/ (group (+ 2 3)) 7)) name))",);

        let source = "print -1 + 3 * 4 - 6 / 3.0 * 9 * (10 * 11) >= \"a\" + \"c\" * (-20);";
        assert_eq!(
            run(source),
            "(print (>= (- (+ (- 1) (* 3 4)) (* (* (/ 6 3) 9) (group (* 10 11)))) (+ a (* c (group (- 20))))))",
        );
    }

    // #[test]
    // fn test_weird() {
    //     let source = "print 1 +- 2;";
    //     assert_eq!(run(source), "(+ 1 (- 2))",);

    //     let source = "print 896 - 1);";
    //     assert_eq!(run(source), "(- 896 1)",);
    // }

    // #[test]
    // #[should_panic]
    // fn test_error() {
    //     let source = "23.123 - + 2";

    //     let evaluator = Evaluator;
    //     let mut interpreter = Lox::new(evaluator);

    //     let mut scanner = Scanner::new(&mut interpreter, &source);
    //     scanner.scan_tokens();

    //     let tokens = scanner.tokens().clone();
    //     let mut parser = Parser::new(&mut interpreter, tokens);
    //     let expression = parser.parse();

    //     expression.unwrap();
    // }
}
