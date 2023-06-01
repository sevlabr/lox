use crate::token::{Token, TokenType};
use std::{char, error::Error, fmt};

#[derive(Debug)]
pub struct ScanError {
    token: Token,
    message: &'static str,
}

impl Error for ScanError {}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.message, self.token)
    }
}

impl ScanError {
    fn new(message: &'static str, token: Token) -> Self {
        ScanError { token, message }
    }

    pub fn token(&self) -> Token {
        self.token
    }
}

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: isize,
}

impl Scanner {
    pub fn new(mut source: String) -> Self {
        source.push('\0');
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_end() {
            return Ok(self.make_token(TokenType::EoF));
        }

        match self.advance() {
            // Single-character tokens.
            '(' => Ok(self.make_token(TokenType::LeftParen)),
            ')' => Ok(self.make_token(TokenType::RightParen)),
            '{' => Ok(self.make_token(TokenType::LeftBrace)),
            '}' => Ok(self.make_token(TokenType::RightBrace)),
            ';' => Ok(self.make_token(TokenType::Semicolon)),
            ',' => Ok(self.make_token(TokenType::Comma)),
            '.' => Ok(self.make_token(TokenType::Dot)),
            '-' => Ok(self.make_token(TokenType::Minus)),
            '+' => Ok(self.make_token(TokenType::Plus)),
            '/' => Ok(self.make_token(TokenType::Slash)),
            '*' => Ok(self.make_token(TokenType::Star)),

            // One or two character tokens.
            '!' => {
                if self.complete('=') {
                    Ok(self.make_token(TokenType::BangEqual))
                } else {
                    Ok(self.make_token(TokenType::Bang))
                }
            }
            '=' => {
                if self.complete('=') {
                    Ok(self.make_token(TokenType::EqualEqual))
                } else {
                    Ok(self.make_token(TokenType::Equal))
                }
            }
            '<' => {
                if self.complete('=') {
                    Ok(self.make_token(TokenType::LessEqual))
                } else {
                    Ok(self.make_token(TokenType::Less))
                }
            }
            '>' => {
                if self.complete('=') {
                    Ok(self.make_token(TokenType::GreaterEqual))
                } else {
                    Ok(self.make_token(TokenType::Greater))
                }
            }

            // Strings and numbers.
            '"' => self.string(),
            '0'..='9' => Ok(self.number()),

            // Identifiers and keywords.
            'a'..='z' | 'A'..='Z' | '_' => Ok(self.identifier()),

            // Invalid tokens.
            _ => Err(self.error_token("Unexpected character.")),
        }
    }

    pub fn lexeme(&self, begin: usize, length: usize) -> String {
        self.source.chars().skip(begin).take(length).collect()
    }

    fn make_token(&self, kind: TokenType) -> Token {
        Token::new(kind, self.start, self.current - self.start, self.line)
    }

    fn error_token(&self, message: &'static str) -> ScanError {
        ScanError::new(message, self.make_token(TokenType::Error))
    }

    fn nth(&self, index: usize) -> char {
        self.source.chars().nth(index).unwrap()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.nth(self.current - 1)
    }

    fn complete(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }
        if self.nth(self.current) != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn string(&mut self) -> Result<Token, ScanError> {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            return Err(self.error_token("Unterminated string."));
        }

        // The closing quote.
        self.advance();
        Ok(self.make_token(TokenType::String))
    }

    fn is_digit(&self, c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn is_alpha(&self, c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.nth(self.start) {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.nth(self.start + 1) {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.nth(self.start + 1) {
                        'h' => self.check_keyword(2, 2, "is", TokenType::This),
                        'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(
        &self,
        start: usize,
        length: usize,
        rest: &'static str,
        kind: TokenType,
    ) -> TokenType {
        if self.current - self.start == start + length
            && self.compare(self.start + start, rest, length)
        {
            return kind;
        }
        TokenType::Identifier
    }

    fn compare(&self, start: usize, rest: &'static str, length: usize) -> bool {
        let mut same = true;
        for (i, j) in (start..(start + length)).enumerate() {
            if self.nth(j)
                != rest
                    .chars()
                    .nth(i)
                    .expect("Failed compare in a trie with static string.")
            {
                same = false;
                break;
            }
        }
        same
    }

    fn is_end(&self) -> bool {
        self.nth(self.current) == '\0'
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // A comment goes until the end of the line.
                        while self.peek() != '\n' && !self.is_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> char {
        self.nth(self.current)
    }

    fn peek_next(&self) -> char {
        if self.is_end() {
            return '\0';
        }
        self.nth(self.current + 1)
    }
}
