use crate::evaluator::native::Clock;
use crate::evaluator::Object;
use crate::evaluator::RuntimeError;
use crate::lexer::token::Token;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Environment {
        let mut values = HashMap::new();
        // globals
        values.insert("clock".to_string(), Object::Time(Clock));

        // Very dirty "unique" id generation for each environment
        // let id = rand::random::<i128>().to_string();
        // values.insert(id, Object::None);

        Environment { enclosing, values }
    }

    pub fn values(&self) -> HashMap<String, Object> {
        self.values.clone()
    }

    pub fn enclosing(&self) -> Option<Box<Environment>> {
        self.enclosing.clone()
    }

    fn _ancestor_clone(&self, distance: usize) -> Environment {
        let mut env = self.clone();
        let msg = format!("Can't get clone of ancestor ({})!", distance);
        for _ in 0..distance {
            env = *env.enclosing().expect(&msg);
        }
        env
    }

    pub fn set_values(&mut self, values: HashMap<String, Object>) {
        self.values = values;
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

    pub fn _assign_at(&mut self, distance: usize, name: Token, value: Object) {
        let mut env = Some(self);
        let msg = format!("Can't get mutable reference to ancestor ({})!", distance);
        for _ in 0..distance {
            let enclosing_option = &mut env.expect(&msg).enclosing;
            let enclosing_box = enclosing_option.as_mut().expect(&msg);
            let environment = enclosing_box.as_mut();
            env = Some(environment);
        }
        env.expect(&msg)
            .values
            .insert(name.get_lexeme().to_string(), value);
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn _get_at(&self, distance: usize, name: &str) -> Result<Object, RuntimeError> {
        let env = self._ancestor_clone(distance);
        if let Some(obj) = env.values().get(name) {
            return Ok(obj.clone());
        }
        let msg = format!("Did not find '{name}' in ancestor environment!");
        unreachable!("{msg}");
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

    pub fn _ref_global(&self) -> &Environment {
        let mut env = self;
        while let Some(enclosing) = &env.enclosing {
            env = enclosing;
        }
        env
    }

    pub fn _ref_mut_global_values(&mut self) -> &mut HashMap<String, Object> {
        let mut env = self;
        let mut enclosing = &mut env.enclosing;
        while let Some(box_encl) = enclosing {
            env = box_encl.as_mut();
            enclosing = &mut env.enclosing;
        }
        &mut env.values
    }

    pub fn _ref_mut_obj(&mut self, obj: Object) -> &mut Object {
        let mut env = self;
        let mut enclosing = &mut env.enclosing;
        loop {
            for item in env.values.values_mut() {
                if *item == obj {
                    return item;
                }
            }

            if let Some(box_encl) = enclosing {
                env = box_encl.as_mut();
                enclosing = &mut env.enclosing;
            } else {
                break;
            }
        }
        unreachable!("Can't find given object!");
    }
}

pub fn _print_envs(mut env: Environment) {
    let mut i = 0;
    loop {
        println!(
            "{} Begin environment {} {}\n",
            "-".repeat(10),
            i,
            "-".repeat(10)
        );
        println!("{:#?}", env);
        println!(
            "\n{} End environment {} {}\n",
            "-".repeat(10),
            i,
            "-".repeat(10)
        );

        i += 1;

        if let Some(boxed_env) = env.enclosing {
            env = *boxed_env;
        } else {
            break;
        }
    }
}
