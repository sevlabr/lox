use std::fmt;
use phf::phf_map;

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
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

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Eof
}

pub struct Token {
    tok_type: TokenType,
    lexeme: String,
    literal: Literal,
    line: usize,
}

#[derive(Debug)]
pub enum Literal {
    None,
    String(String),
    Number(f64),
}

impl Token {
    pub fn new(tok_type: TokenType, lexeme: String, literal: Literal, line: usize) -> Token {
        Token { tok_type, lexeme, literal, line }
    }

    pub fn to_string(&self) -> String {
        format!("{} {:?} {} {:?}", self.line, self.tok_type, self.lexeme, self.literal)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?} {} {:?}", self.line, self.tok_type, self.lexeme, self.literal)
    }
}
