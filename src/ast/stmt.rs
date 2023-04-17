use crate::ast::Expr;
use crate::lexer::token::Token;

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Expr),       // (name, initializer)
    While(Expr, Box<Stmt>), // (condition, body)
}
