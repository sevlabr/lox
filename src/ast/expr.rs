use crate::lexer::token::{Literal, Token};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    LiteralExpr(Literal),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Variable(Token),
}
