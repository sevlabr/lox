use phf::phf_map;
use std::fmt;
use std::hash::{Hash, Hasher};

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and"    => TokenType::And,
    "class"  => TokenType::Class,
    "else"   => TokenType::Else,
    "false"  => TokenType::False,
    "for"    => TokenType::For,
    "fun"    => TokenType::Fun,
    "if"     => TokenType::If,
    "nil"    => TokenType::Nil,
    "or"     => TokenType::Or,
    "print"  => TokenType::Print,
    "return" => TokenType::Return,
    "super"  => TokenType::Super,
    "this"   => TokenType::This,
    "true"   => TokenType::True,
    "var"    => TokenType::Var,
    "while"  => TokenType::While,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

    Eof,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Token {
    tok_type: TokenType,
    lexeme: String,
    literal: Literal,
    line: usize,
}

impl Token {
    pub fn new(tok_type: TokenType, lexeme: &str, literal: Literal, line: usize) -> Self {
        Token {
            tok_type,
            lexeme: lexeme.to_string(),
            literal,
            line,
        }
    }

    pub fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn get_type(&self) -> &TokenType {
        &self.tok_type
    }

    pub fn get_literal(&self) -> &Literal {
        &self.literal
    }

    pub fn get_line(&self) -> usize {
        self.line
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:?} {} {:?}",
            self.line, self.tok_type, self.lexeme, self.literal
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Literal {
    Bool(bool),
    Number(Num),
    String(String),
    None,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::None => write!(f, "nil"),
            Literal::String(s) => write!(f, "{s}"),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::Bool(b) => write!(f, "{b}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Num {
    n: f64,
}

impl Num {
    pub fn new(n: f64) -> Num {
        Num { n }
    }

    pub fn get(&self) -> f64 {
        self.n
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.n)
    }
}

const E15: f64 = 1_000_000_000_000_000.0;

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        let self_n = (self.n * E15).round() as i64;
        let other_n = (other.n * E15).round() as i64;
        self_n == other_n
    }
}

impl Eq for Num {}

impl Hash for Num {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_n = (self.n * E15).round() as i64;
        self_n.hash(state);
    }
}
