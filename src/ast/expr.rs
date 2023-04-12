use crate::lexer::token::{Literal, Token};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    LiteralExpr(Literal),
    Unary(Token, Box<Expr>),
}
