use super::{Evaluator, Object, RuntimeError};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq)]
pub struct Clock;

impl Clock {
    pub fn arity(&self) -> usize {
        0
    }

    pub fn call(&self, _: &mut Evaluator, _: Vec<Object>) -> Result<Object, RuntimeError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let in_secs = since_the_epoch.as_secs();

        Ok(Object::Number(in_secs as f64))
    }

    fn stringify(&self) -> String {
        "<native fun 'clock'>".to_string()
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}
