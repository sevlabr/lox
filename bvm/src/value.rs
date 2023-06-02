use std::fmt;

#[derive(Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nil,
    Num(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{}", val),
            Value::Nil => write!(f, "nil"),
            Value::Num(val) => write!(f, "{}", val),
        }
    }
}

impl Value {
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_num(&self) -> bool {
        matches!(self, Value::Num(_))
    }

    /// Extract inner `bool` value.
    /// 
    /// # Safety
    /// 
    /// Fails if `Value::is_bool()` returns `false`.
    /// Use `Value::is_bool()` before applying this function.
    pub unsafe fn as_bool(&self) -> f64 {
        match self {
            Value::Num(val) => *val,
            _ => panic!("Expected f64 value."),
        }
    }

    /// Extract inner `f64` value.
    /// 
    /// # Safety
    /// 
    /// Fails if `Value::is_num()` returns `false`.
    /// Use `Value::is_num()` before applying this function.
    pub unsafe fn as_num(&self) -> f64 {
        match self {
            Value::Num(val) => *val,
            _ => panic!("Expected f64 value."),
        }
    }
}
