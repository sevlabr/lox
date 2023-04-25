use crate::evaluator::{Evaluator, Function, Instance, Object, RuntimeError};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: String,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Self {
        Class { name, methods }
    }

    pub fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        if let Some(init) = initializer {
            return init.arity();
        }
        0
    }

    pub fn call(&self, evaluator: &mut Evaluator, arguments: Vec<Object>) -> Result<Object, RuntimeError> {
        let instance = Instance::new(self.clone());
        let initializer = self.find_method("init");
        if let Some(init) = initializer {
            (init.bind(&instance)?).call(evaluator, arguments)?;
        }

        Ok(Object::Instance(instance))
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        self.methods.get(name).cloned()
    }

    fn stringify(&self) -> String {
        self.name.clone()
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stringify())
    }
}
