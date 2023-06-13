use crate::chunk::Chunk;
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone, PartialEq, Eq)]
pub enum Obj {
    Fun(Function),
    Str(String),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::Fun(fun) => write!(f, "{}", fun),
            Obj::Str(s) => write!(f, "{}", s),
        }
    }
}

impl Obj {
    pub fn is_fun(&self) -> bool {
        matches!(self, Obj::Fun(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Obj::Str(_))
    }

    pub fn is_obj_type(&self, kind: &'static str) -> bool {
        match kind {
            "Function" => self.is_fun(),
            "String" => self.is_string(),
            _ => panic!("Invalid Obj type specified: {}", kind),
        }
    }

    /// Extract inner `Function`. This function returns cloned object,
    /// not the original one.
    ///
    /// # Safety
    ///
    /// Fails if `Obj::is_fun()` returns `false`.
    /// Use `Obj::is_fun()` before applying this function.
    pub unsafe fn as_fun(&self) -> Function {
        match self {
            Obj::Fun(fun) => fun.clone(),
            _ => panic!("Expected Function object."),
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
            _ => panic!("Expected Str value."),
        }
    }
}

#[derive(Clone)]
pub struct Function {
    arity: isize,
    chunk: Rc<RefCell<Chunk>>,
    name: String,
}

impl Default for Function {
    fn default() -> Self {
        Self::new()
    }
}

impl Function {
    pub fn new() -> Self {
        Self {
            arity: 0,
            chunk: Rc::new(RefCell::new(Chunk::default())),
            name: String::new(),
        }
    }

    pub fn arity(&self) -> isize {
        self.arity
    }

    pub fn change_arity(&mut self, arity: isize) {
        self.arity = arity;
    }

    pub fn chunk(&self) -> Rc<RefCell<Chunk>> {
        Rc::clone(&self.chunk)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    // Example function for deallocation. May change later.
    // Supposed to be used for GC.
    // (Also check Zeroize crate if needed since this implementation
    // actually doesn't free anything).
    pub fn free(&mut self) {
        self.name.clear();
        self.chunk().borrow_mut().free();
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            return write!(f, "<script>");
        }
        write!(f, "<fun {}>", &self.name)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        // Maybe check Chunks too.
        self.arity == other.arity && self.name == other.name
    }
}

impl Eq for Function {}
