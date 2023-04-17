use super::{Evaluator, Object};
use std::fmt;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, evaluator: &Evaluator, arguments: Vec<Object>) -> Object;
    fn stringify(&self) -> String;
}

impl fmt::Display for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}
