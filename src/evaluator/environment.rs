use crate::evaluator::native::Clock;
use crate::evaluator::Object;
use crate::evaluator::RuntimeError;
use crate::lexer::token::Token;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Environment {
        let mut values = HashMap::new();
        // globals
        values.insert("clock".to_string(), Object::Time(Clock));

        Environment { enclosing, values }
    }

    pub fn enclosing(&self) -> Option<Box<Environment>> {
        self.enclosing.clone()
    }

    pub fn from_inner(environment: Option<Box<Environment>>) -> Environment {
        match environment {
            Some(env) => Environment {
                enclosing: env.enclosing(),
                values: env.values,
            },
            None => Environment::new(None),
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        if self.values.contains_key(name.get_lexeme()) {
            self.values.insert(name.get_lexeme().to_string(), value);
            return Ok(());
        }

        if let Some(env) = &mut self.enclosing {
            env.assign(name, value)?;
            return Ok(());
        }

        let mut msg = String::new();
        msg.push_str("Undefined variable '");
        msg.push_str(name.get_lexeme());
        msg.push_str("'.");
        Err(RuntimeError::new(name, &msg))
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<Object, RuntimeError> {
        if self.values.contains_key(name.get_lexeme()) {
            return Ok(self.values.get(name.get_lexeme()).cloned().unwrap());
        }

        if let Some(env) = &self.enclosing {
            return env.get(name);
        }

        let mut msg = String::new();
        msg.push_str("Undefined variable '");
        msg.push_str(name.get_lexeme());
        msg.push_str("'.");
        Err(RuntimeError::new(&name, &msg))
    }
}
