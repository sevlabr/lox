use crate::ast::Expr;
use crate::lexer::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Expr),       // (name, initializer)
    While(Expr, Box<Stmt>), // (condition, body)
}
