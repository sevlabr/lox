use std::fmt;

#[derive(Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nil,
    Num(f64),
}

#[derive(PartialEq, Eq)]
enum ValueType {
    Bool,
    Nil,
    Num,
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

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && unsafe { !self.as_bool() })
    }

    pub fn equal(&self, other: Self) -> bool {
        if self.kind() != other.kind() {
            return false;
        }
        match self {
            Value::Bool(a) => *a == unsafe { other.as_bool() },
            Value::Nil => true,
            Value::Num(a) => *a == unsafe { other.as_num() },
        }
    }

    fn kind(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Nil => ValueType::Nil,
            Value::Num(_) => ValueType::Num,
        }
    }

    /// Extract inner `bool` value.
    ///
    /// # Safety
    ///
    /// Fails if `Value::is_bool()` returns `false`.
    /// Use `Value::is_bool()` before applying this function.
    pub unsafe fn as_bool(&self) -> bool {
        match self {
            Value::Bool(val) => *val,
            _ => panic!("Expected bool value."),
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
