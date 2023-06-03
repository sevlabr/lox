use crate::object::Obj;
use std::fmt;

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Num(f64),
    Obj(Obj),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{}", val),
            Value::Nil => write!(f, "nil"),
            Value::Num(val) => write!(f, "{}", val),
            Value::Obj(obj) => write!(f, "{}", obj),
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

    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && unsafe { !self.as_bool() })
    }

    pub fn is_obj_type(&self, kind: &'static str) -> bool {
        self.is_obj() && unsafe { self.as_obj().is_obj_type(kind) }
    }

    pub fn equal(&self, other: Self) -> bool {
        if !self.eq_type(&other) {
            return false;
        }
        match self {
            Value::Bool(a) => *a == unsafe { other.as_bool() },
            Value::Nil => true,
            Value::Num(a) => *a == unsafe { other.as_num() },
            Value::Obj(obj) => *obj == unsafe { other.as_obj() },
        }
    }

    fn eq_type(&self, other: &Self) -> bool {
        match self {
            Value::Bool(_) => matches!(other, Value::Bool(_)),
            Value::Nil => matches!(other, Value::Nil),
            Value::Num(_) => matches!(other, Value::Num(_)),
            Value::Obj(_) => matches!(other, Value::Obj(_)),
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

    /// Extract inner `Obj`. This function returns cloned value,
    /// not the original one.
    ///
    /// # Safety
    ///
    /// Fails if `Value::is_obj()` returns `false`.
    /// Use `Value::is_obj()` before applying this function.
    pub unsafe fn as_obj(&self) -> Obj {
        match self {
            Value::Obj(obj) => obj.clone(),
            _ => panic!("Expected Obj value."),
        }
    }
}
