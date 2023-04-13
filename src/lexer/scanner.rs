use crate::lexer::token::{Literal, Token, TokenType, KEYWORDS};
use crate::Lox;

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

    interpreter: &'a mut Lox,
}

impl Scanner<'_> {
    pub fn new<'a>(interpreter: &'a mut Lox, source: &str) -> Scanner<'a> {
        Scanner {
            source: source.to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            interpreter,
        }
    }

    pub fn tokens(&self) -> &Vec<Token> {
        self.tokens.as_ref()
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", Literal::None, self.line));
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            '!' => {
                if self.peek('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.peek('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.peek('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.peek('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }

            '/' => {
                if self.peek('/') {
                    // A comment goes until the end of the line.
                    while self.look_ahead() != '\n' && !self.is_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }

            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            '"' => self.consume_string(),

            '0'..='9' => self.consume_number(),

            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

            _ => self.interpreter.lex_error(self.line, "Unexpected character!"),
        }
    }

    fn advance(&mut self) -> char {
        let c = self
            .source
            .chars()
            .nth(self.current)
            .expect("Failed advancing character!");
        self.current += 1;
        c
    }

    fn peek(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }
        let next_char = self
            .source
            .chars()
            .nth(self.current)
            .expect("Failed peeking next character!");
        if next_char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    // TODO: add helper fucntion to shorten str[idx] line
    fn look_ahead(&self) -> char {
        if self.is_end() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current)
            .expect("Failed looking ahead the character!")
    }

    fn look_ahead_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current + 1)
            .expect("Failed looking ahead the next character!")
    }

    fn add_token(&mut self, tok_type: TokenType) {
        self.add_token_literal(tok_type, Literal::None)
    }

    fn add_token_literal(&mut self, tok_type: TokenType, literal: Literal) {
        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        self.tokens
            .push(Token::new(tok_type, &text, literal, self.line));
    }

    fn consume_string(&mut self) {
        while self.look_ahead() != '"' && !self.is_end() {
            if self.look_ahead() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_end() {
            self.interpreter.lex_error(self.line, "Unterminated string!");
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value: String = self
            .source
            .chars()
            .skip(self.start + 1)
            .take(self.current - 2 - self.start)
            .collect();
        self.add_token_literal(TokenType::String, Literal::String(value));
    }

    fn consume_number(&mut self) {
        while self.is_digit(self.look_ahead()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.look_ahead() == '.' && self.is_digit(self.look_ahead_next()) {
            // Consume the "."
            self.advance();

            while self.is_digit(self.look_ahead()) {
                self.advance();
            }
        }

        let number: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        let number: f64 = number.trim().parse().expect("Failed parsing number!");
        self.add_token_literal(TokenType::Number, Literal::Number(number))
    }

    fn identifier(&mut self) {
        while self.is_alphanum(self.look_ahead()) {
            self.advance();
        }

        // TODO: wrap this str[beg:end] to a function
        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        let tok_type = match KEYWORDS.get(&text) {
            Some(t) => t.clone(),
            None => TokenType::Identifier,
        };
        self.add_token(tok_type);
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_digit(&self, c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn is_alphanum(&self, c: char) -> bool {
        let mut alphanum = false;
        match c {
            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => alphanum = true,
            _ => (),
        }
        alphanum
    }
}
