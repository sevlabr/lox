use crate::evaluator::{environment::Environment, Evaluator, Object, RuntimeError};
use crate::{ast::stmt::Stmt, Token};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
}

impl Function {
    pub fn new(tok: &Token, declaration: Stmt) -> Result<Function, RuntimeError> {
        match declaration {
            Stmt::Function(name, parameters, body) => Ok(Function {
                name,
                parameters,
                body,
            }),
            _ => Err(RuntimeError::new(
                tok,
                "Can create Function object only from Stmt::Function.",
            )),
        }
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    pub fn call(
        &self,
        evaluator: &mut Evaluator,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let mut env = Environment::new(Some(Box::new(evaluator.environment.clone())));

        for i in 0..self.parameters.len() {
            env.define(
                self.parameters.get(i).unwrap().get_lexeme().to_string(),
                arguments.get(i).unwrap().clone(),
            )
        }

        match evaluator.execute_block(&self.body, env) {
            Ok(_) => Ok(Object::None),
            Err(err) => {
                if err.is_return() {
                    return Ok(err.get_value());
                }
                Err(err)
            }
        }
    }

    fn stringify(&self) -> String {
        let mut pretty_str = String::new();
        pretty_str.push_str("<fun ");
        pretty_str.push_str(self.name.get_lexeme());
        pretty_str.push('>');

        pretty_str
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}
