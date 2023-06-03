use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Obj {
    Str(String),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::Str(s) => write!(f, "{}", s),
        }
    }
}

impl Obj {
    pub fn is_string(&self) -> bool {
        matches!(self, Obj::Str(_))
    }

    pub fn is_obj_type(&self, kind: &'static str) -> bool {
        match kind {
            "String" => self.is_string(),
            _ => panic!("Invalid Obj type specified: {}", kind),
        }
    }

    /// Extract inner `String`. This function returns cloned value,
    /// not the original one.
    ///
    /// # Safety
    ///
    /// Fails if `Obj::is_string()` returns `false`.
    /// Use `Obj::is_string()` before applying this function.
    pub unsafe fn as_string(&self) -> String {
        match self {
            Obj::Str(s) => s.clone(),
            // _ => panic!("Expected Str value."),
        }
    }
}
