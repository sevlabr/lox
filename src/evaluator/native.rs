use crate::{evaluator::function::Callable};
use super::{Evaluator, Object};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &Evaluator, _: Vec<Object>) -> Object {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let in_secs = since_the_epoch.as_secs();

        Object::Number(in_secs as f64)
    }

    fn stringify(&self) -> String {
        "<native fun 'clock'>".to_string()
    }
}
