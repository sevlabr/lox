pub mod environment;
pub mod function;
pub mod native;

use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::lexer::token::{Literal, Token, TokenType};
use crate::Visitor;
use environment::Environment;
use function::Function;
use native::Clock;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct RuntimeError {
    token: Token,
    message: String,
    value: Object,
    is_return: bool,
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
            value: Object::None,
            is_return: false,
        }
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }

    pub fn get_token(&self) -> Token {
        self.token.clone()
    }

    fn make_return(&mut self, value: Object) {
        self.value = value;
        self.is_return = true;
    }

    pub fn is_return(&self) -> bool {
        self.is_return
    }

    pub fn get_value(&self) -> Object {
        self.value.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Bool(bool),
    Fun(Function),
    Time(Clock),
    Number(f64),
    String(String),
    None,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Bool(b) => write!(f, "{b}"),
            Object::Fun(fun) => write!(f, "Function: {}.", fun),
            Object::Time(c) => write!(f, "Function: {}.", c),
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

impl Object {
    fn is_callable(&self, tok: &Token) -> Result<(), RuntimeError> {
        match self {
            Object::Fun(_) => Ok(()),
            Object::Time(_) => Ok(()),
            _ => Err(RuntimeError::new(
                tok,
                "Can only call functions and classes.",
            )),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Object::Fun(f) => f.arity(),
            Object::Time(c) => c.arity(),
            _ => panic!("arity() is defined only for Function and clock()."),
        }
    }

    fn call(
        &mut self,
        evaluator: &mut Evaluator,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        match self {
            Object::Fun(f) => f.call(evaluator, arguments),
            Object::Time(c) => c.call(evaluator, arguments),
            _ => panic!("call() is defined only for Function and clock()."),
        }
    }
}

#[derive(Clone)]
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
            Expr::Logical(left, op, right) => {
                let l = self.evaluate(left)?;

                if *op.get_type() == TokenType::Or {
                    if self.is_truthy(&l) {
                        return Ok(l);
                    }
                } else if !self.is_truthy(&l) {
                    return Ok(l);
                }

                Ok(self.evaluate(right)?)
            }
            Expr::Grouping(exp) => self.evaluate(exp),
            Expr::Unary(op, right) => {
                let r = self.evaluate(right)?;

                match op.get_type() {
                    TokenType::Minus => Ok(Object::Number(-self.cast_num(op, r)?)),
                    TokenType::Bang => Ok(Object::Bool(!self.is_truthy(&r))),
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

            Expr::Call(callee_expr, tok, args) => {
                let mut callee = self.evaluate(callee_expr)?;

                let mut arguments: Vec<Object> = Vec::new();
                for arg in args {
                    arguments.push(self.evaluate(arg)?);
                }

                callee.is_callable(tok)?;

                let arg_len = arguments.len();
                let arity = callee.arity();
                if arg_len != arity {
                    let mut message = String::new();
                    message.push_str("Expected ");
                    message.push_str(&arity.to_string());
                    message.push_str(" arguments but got ");
                    message.push_str(&arg_len.to_string());
                    message.push('.');
                    return Err(RuntimeError::new(tok, &message));
                }

                Ok(callee.call(self, arguments)?)
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
            fun @ Stmt::Function(name, _, _) => {
                let function = Function::new(name, fun.clone(), self.environment.clone())?;
                self.environment
                    .define(name.get_lexeme().to_string(), Object::Fun(function));
                Ok(())
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let cond = self.evaluate(condition)?;
                if self.is_truthy(&cond) {
                    self.execute(then_branch)?;
                    return Ok(());
                }
                if let Some(s) = else_branch {
                    self.execute(s)?;
                }
                Ok(())
            }
            Stmt::Print(exp) => {
                let value = self.evaluate(exp)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Return(keyword, value) => {
                let val = match value {
                    Expr::LiteralExpr(Literal::None) => Object::None,
                    _ => self.evaluate(value)?,
                };
                let mut return_val = RuntimeError::new(keyword, "Return value, not an error.");
                return_val.make_return(val);
                Err(return_val)
            }
            Stmt::Block(statements) => {
                self.execute_block(
                    statements,
                    // TODO: cloning is inefficient, change to ref
                    Environment::new(Some(Box::new(self.environment.clone()))),
                )?;
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
            Stmt::While(condition, body) => {
                let mut cond = self.evaluate(condition)?;
                while self.is_truthy(&cond) {
                    self.execute(body)?;
                    cond = self.evaluate(condition)?;
                }
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

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        env: Environment,
    ) -> Result<(), RuntimeError> {
        // TODO: cloning is inefficient, change to ref
        // let previous = self.environment.clone();
        self.environment = env;
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => continue,
                Err(err) => {
                    let previous = self.environment.enclosing();
                    self.environment = Environment::from_inner(previous);
                    return Err(err);
                }
            }
        }

        let previous = self.environment.enclosing();
        self.environment = Environment::from_inner(previous);
        Ok(())
    }

    fn execute_block_fun(
        &mut self,
        statements: &Vec<Stmt>,
        env: Environment,
        depth: usize,
    ) -> (Result<(), RuntimeError>, Environment) {
        // TODO: cloning is inefficient, change to ref
        // let previous = self.environment.clone();
        self.environment = env;
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => continue,
                Err(err) => {
                    let mut previous = self.environment.enclosing();
                    if depth >= 1 {
                        for _ in 1..depth {
                            previous = previous.unwrap().enclosing();
                        }
                    }
                    let closure = self.environment.clone();
                    self.environment = Environment::from_inner(previous);
                    return (Err(err), closure);
                }
            }
        }

        let mut previous = self.environment.enclosing();
        if depth >= 1 {
            for _ in 1..depth {
                previous = previous.unwrap().enclosing();
            }
        }
        let closure = self.environment.clone();
        self.environment = Environment::from_inner(previous);
        (Ok(()), closure)
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

    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::None => false,
            Object::Bool(b) => *b,
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
