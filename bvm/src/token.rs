use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: isize,
}

impl Token {
    pub fn new(kind: TokenType, start: usize, length: usize, line: isize) -> Self {
        Token {
            kind,
            start,
            length,
            line,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new(TokenType::Error, 0, 0, -1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    EoF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
