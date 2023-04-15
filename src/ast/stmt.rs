use crate::ast::Expr;
use crate::lexer::token::Token;

pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var(Token, Expr), // (name, initializer)
}
