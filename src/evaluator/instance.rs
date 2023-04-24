use crate::evaluator::{Class, Object, RuntimeError};
use crate::Token;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    class: Class,
    fields: HashMap<String, Object>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Instance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.get_lexeme().to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        let name_str = name.get_lexeme();
        if self.fields.contains_key(name_str) {
            return Ok(self.fields.get(name_str).cloned().unwrap());
        }

        let method_option = self.class.find_method(name_str);
        if let Some(method) = method_option {
            return Ok(Object::Fun(method.bind(self)?));
        }

        let msg = format!("Undefined property '{name_str}'.");
        Err(RuntimeError::new(name, &msg))
    }

    fn stringify(&self) -> String {
        format!("{} instance", self.class)
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stringify())
    }
}
