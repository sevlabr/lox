use crate::ast::expr::Expr;
use crate::lexer::token::{Literal, Token, TokenType};
use crate::Visitor;
use crate::Lox;
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
        write!(f, "RuntimeError for Token: <{}>. Message: {}", self.token, self.message)
    }
}

impl RuntimeError {
    fn new(t: &Token, msg: &str) -> RuntimeError {
        RuntimeError { token: t.clone(), message: msg.to_string() }
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }

    pub fn get_token(&self) -> Token {
        self.token.clone()
    }
}

// TODO: probably Literal is enough.
#[derive(PartialEq)]
pub enum Object {
    Bool(bool),
    Number(f64),
    String(String),
    None,
}

impl Object {
    pub fn to_string(&self) -> String {
        match self {
            Object::Bool(b) => b.to_string(),
            Object::Number(n) => {
                if n.fract() == 0.0 {
                    return (*n as i64).to_string();
                }
                return n.to_string();
            },
            Object::String(s) => s.clone(),
            Object::None => "nil".to_string()
        }
    }
}

pub struct Evaluator<'a> {
    interpreter: &'a mut Lox,
}

impl Visitor<Result<Object, RuntimeError>> for Evaluator<'_> {
    fn visit_expr(&self, e: &Expr) -> Result<Object, RuntimeError> {
        match e {
            Expr::LiteralExpr(l) => {
                match l {
                    Literal::Bool(b) => Ok(Object::Bool(*b)),
                    Literal::Number(n) => Ok(Object::Number(*n)),
                    Literal::String(s) => Ok(Object::String(s.clone())),
                    Literal::None => Ok(Object::None),
                }
            },
            Expr::Grouping(exp) => self.evaluate(exp),
            Expr::Unary(op, right) => {
                let r = self.evaluate(right)?;

                match op.get_type() {
                    TokenType::Minus => Ok(Object::Number(-self.cast_num(op, r)?)),
                    TokenType::Bang => Ok(Object::Bool(!self.is_truthy(r))),
                    _ => Ok(Object::None)
                }
            },
            Expr::Binary(left, op, right) => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;

                match op.get_type() {
                    TokenType::Greater => Ok(Object::Bool(self.cast_num(op, l)? > self.cast_num(op, r)?)),
                    TokenType::GreaterEqual => Ok(Object::Bool(self.cast_num(op, l)? >= self.cast_num(op, r)?)),
                    TokenType::Less => Ok(Object::Bool(self.cast_num(op, l)? < self.cast_num(op, r)?)),
                    TokenType::LessEqual => Ok(Object::Bool(self.cast_num(op, l)? <= self.cast_num(op, r)?)),

                    TokenType::BangEqual => Ok(Object::Bool(!self.is_equal(l, r))),
                    TokenType::EqualEqual => Ok(Object::Bool(self.is_equal(l, r))),

                    TokenType::Minus => Ok(Object::Number(self.cast_num(op, l)? - self.cast_num(op, r)?)),
                    TokenType::Slash => Ok(Object::Number(self.cast_num(op, l)? / self.cast_num(op, r)?)),
                    TokenType::Star => Ok(Object::Number(self.cast_num(op, l)? * self.cast_num(op, r)?)),

                    TokenType::Plus => {
                        if self.is_num(&l) && self.is_num(&r) {
                            return Ok(Object::Number(self.cast_num(op, l)? + self.cast_num(op, r)?));
                        }

                        if self.is_str(&l) && self.is_str(&r) {
                            let mut concatenated_str = self.cast_str(op, l)?;
                            concatenated_str.push_str(&self.cast_str(op, r)?);
                            return Ok(Object::String(concatenated_str));
                        }

                        return Err(RuntimeError::new(op, "Operands must be two numbers or two strings."));
                    }

                    _ => Ok(Object::None),
                }
            }
        }
    }
}

impl Evaluator<'_> {
    pub fn new(interpreter: &mut Lox) -> Evaluator {
        Evaluator { interpreter }
    }

    pub fn interpret(&mut self, exp: Expr) -> Option<Object> {
        let value = self.evaluate(&Box::new(exp));

        match value {
            Ok(v) => Some(v),
            Err(err) => {
                self.interpreter.runtime_error(err);
                return None;
            }
        }
    }

    fn evaluate(&self, exp: &Box<Expr>) -> Result<Object, RuntimeError> {
        self.visit_expr(&*exp)
    }

    fn cast_num(&self, op: &Token, obj: Object) -> Result<f64, RuntimeError> {
        match obj {
            Object::Number(n) => Ok(n),
            _ => Err(RuntimeError::new(op, "Operand must be a number."))
        }
    }

    fn cast_str(&self, op: &Token, obj: Object) -> Result<String, RuntimeError> {
        match obj {
            Object::String(s) => Ok(s),
            _ => Err(RuntimeError::new(op, "Operand must be a string."))
        }
    }

    fn is_num(&self, obj: &Object) -> bool {
        match obj {
            Object::Number(_) => true,
            _ => false
        }
    }

    fn is_str(&self, obj: &Object) -> bool {
        match obj {
            Object::String(_) => true,
            _ => false
        }
    }

    fn is_none(&self, obj: &Object) -> bool {
        match obj {
            Object::None => true,
            _ => false
        }
    }

    fn is_truthy(&self, obj: Object) -> bool {
        match obj {
            Object::None => false,
            Object::Bool(b) => b,
            _ => true
        }
    }

    fn is_equal(&self, l: Object, r: Object) -> bool {
        if self.is_none(&l) && self.is_none(&r) {
            return true;
        }
        if self.is_none(&l) {
            return false;
        }

        return l == r;
    }
}
