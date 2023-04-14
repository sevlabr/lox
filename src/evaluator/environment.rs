use crate::evaluator::Object;
use crate::evaluator::RuntimeError;
use crate::lexer::token::Token;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        if self.values.contains_key(name.get_lexeme()) {
            self.values.insert(name.get_lexeme().to_string(), value);
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

        let mut msg = String::new();
        msg.push_str("Undefined variable '");
        msg.push_str(name.get_lexeme());
        msg.push_str("'.");
        Err(RuntimeError::new(&name, &msg))
    }
}
