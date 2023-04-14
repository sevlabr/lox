use crate::lexer::token::{Literal, Token};

#[derive(Clone, PartialEq)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    LiteralExpr(Literal),
    Unary(Token, Box<Expr>),
    Variable(Token),
}
