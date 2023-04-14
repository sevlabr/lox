pub mod environment;

use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::lexer::token::{Literal, Token, TokenType};
use crate::Visitor;
use environment::Environment;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct RuntimeError {
    token: Token,
    message: String,
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RuntimeError for Token: <{}>. Message: {}",
            self.token, self.message
        )
    }
}

impl RuntimeError {
    fn new(t: &Token, msg: &str) -> RuntimeError {
        RuntimeError {
            token: t.clone(),
            message: msg.to_string(),
        }
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }

    pub fn get_token(&self) -> Token {
        self.token.clone()
    }
}

// TODO: probably Literal is enough.
#[derive(Clone, PartialEq)]
pub enum Object {
    Bool(bool),
    Number(f64),
    String(String),
    None,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Bool(b) => write!(f, "{b}"),
            Object::Number(n) => {
                if n.fract() == 0.0 {
                    return write!(f, "{}", (*n as i64));
                }
                write!(f, "{}", (*n))
            }
            Object::String(s) => write!(f, "{s}"),
            Object::None => write!(f, "nil"),
        }
    }
}

pub struct Evaluator {
    environment: Environment,
}

impl Visitor<Result<Object, RuntimeError>, Result<(), RuntimeError>> for Evaluator {
    fn visit_expr(&mut self, e: &Expr) -> Result<Object, RuntimeError> {
        match e {
            Expr::LiteralExpr(l) => match l {
                Literal::Bool(b) => Ok(Object::Bool(*b)),
                Literal::Number(n) => Ok(Object::Number(*n)),
                Literal::String(s) => Ok(Object::String(s.clone())),
                Literal::None => Ok(Object::None),
            },
            Expr::Grouping(exp) => self.evaluate(exp),
            Expr::Unary(op, right) => {
                let r = self.evaluate(right)?;

                match op.get_type() {
                    TokenType::Minus => Ok(Object::Number(-self.cast_num(op, r)?)),
                    TokenType::Bang => Ok(Object::Bool(!self.is_truthy(r))),
                    _ => Ok(Object::None),
                }
            }

            Expr::Variable(name) => self.environment.get(name.clone()),

            Expr::Binary(left, op, right) => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;

                match op.get_type() {
                    TokenType::Greater => {
                        Ok(Object::Bool(self.cast_num(op, l)? > self.cast_num(op, r)?))
                    }
                    TokenType::GreaterEqual => {
                        Ok(Object::Bool(self.cast_num(op, l)? >= self.cast_num(op, r)?))
                    }
                    TokenType::Less => {
                        Ok(Object::Bool(self.cast_num(op, l)? < self.cast_num(op, r)?))
                    }
                    TokenType::LessEqual => {
                        Ok(Object::Bool(self.cast_num(op, l)? <= self.cast_num(op, r)?))
                    }

                    TokenType::BangEqual => Ok(Object::Bool(!self.is_equal(l, r))),
                    TokenType::EqualEqual => Ok(Object::Bool(self.is_equal(l, r))),

                    TokenType::Minus => Ok(Object::Number(
                        self.cast_num(op, l)? - self.cast_num(op, r)?,
                    )),
                    TokenType::Slash => Ok(Object::Number(
                        self.cast_num(op, l)? / self.cast_num(op, r)?,
                    )),
                    TokenType::Star => Ok(Object::Number(
                        self.cast_num(op, l)? * self.cast_num(op, r)?,
                    )),

                    TokenType::Plus => {
                        if self.is_num(&l) && self.is_num(&r) {
                            return Ok(Object::Number(
                                self.cast_num(op, l)? + self.cast_num(op, r)?,
                            ));
                        }

                        if self.is_str(&l) && self.is_str(&r) {
                            let mut concatenated_str = self.cast_str(op, l)?;
                            concatenated_str.push_str(&self.cast_str(op, r)?);
                            return Ok(Object::String(concatenated_str));
                        }

                        Err(RuntimeError::new(
                            op,
                            "Operands must be two numbers or two strings.",
                        ))
                    }

                    _ => Ok(Object::None),
                }
            }

            Expr::Assign(name, value) => {
                let val = self.evaluate(value)?;
                self.environment.assign(name, val.clone())?;
                Ok(val)
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<(), RuntimeError> {
        match s {
            Stmt::Expression(exp) => {
                self.evaluate(exp)?;
                Ok(())
            }
            Stmt::Print(exp) => {
                let value = self.evaluate(exp)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                let mut value = Object::None;
                if *initializer != Expr::LiteralExpr(Literal::None) {
                    value = self.evaluate(initializer)?;
                }

                self.environment
                    .define(name.get_lexeme().to_string(), value);
                Ok(())
            }
        }
    }
}

impl Evaluator {
    pub fn new(environment: Environment) -> Evaluator {
        Evaluator { environment }
    }

    pub fn evaluate(&mut self, exp: &Expr) -> Result<Object, RuntimeError> {
        self.visit_expr(exp)
    }

    pub fn execute(&mut self, s: &Stmt) -> Result<(), RuntimeError> {
        self.visit_stmt(s)
    }

    fn cast_num(&self, op: &Token, obj: Object) -> Result<f64, RuntimeError> {
        match obj {
            Object::Number(n) => Ok(n),
            _ => Err(RuntimeError::new(op, "Operand must be a number.")),
        }
    }

    fn cast_str(&self, op: &Token, obj: Object) -> Result<String, RuntimeError> {
        match obj {
            Object::String(s) => Ok(s),
            _ => Err(RuntimeError::new(op, "Operand must be a string.")),
        }
    }

    fn is_num(&self, obj: &Object) -> bool {
        matches!(obj, Object::Number(_))
    }

    fn is_str(&self, obj: &Object) -> bool {
        matches!(obj, Object::String(_))
    }

    fn is_none(&self, obj: &Object) -> bool {
        matches!(obj, Object::None)
    }

    fn is_truthy(&self, obj: Object) -> bool {
        match obj {
            Object::None => false,
            Object::Bool(b) => b,
            _ => true,
        }
    }

    fn is_equal(&self, l: Object, r: Object) -> bool {
        if self.is_none(&l) && self.is_none(&r) {
            return true;
        }
        if self.is_none(&l) {
            return false;
        }

        l == r
    }
}
