use crate::scanner::Scanner;
use crate::token::TokenType;

pub fn compile(source: String) {
    let mut scanner = Scanner::new(source);
    let mut line: isize = -1;
    loop {
        let (token, message) = match scanner.scan_token() {
            Ok(token) => (token, scanner.lexeme(token.start, token.length)),
            Err(err) => (err.token(), format!("{err}")),
        };

        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        let token_kind = format!("{}", token.kind);
        println!("{:>12} '{}'", token_kind, message);

        if token.kind == TokenType::EoF {
            break;
        }
    }
}
