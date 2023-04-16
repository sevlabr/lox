use crate::ast::Expr;
use crate::lexer::token::Token;

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Expr),       // (name, initializer)
    While(Expr, Box<Stmt>), // (condition, body)
}
