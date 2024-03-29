use crate::chunk::Chunk;
use crate::value::Value;
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone, PartialEq, Eq)]
pub enum Obj {
    BuiltIn(Native),
    Closure(Closure),
    Fun(Function),
    Str(String),
    Upval(Upvalue),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::BuiltIn(native) => write!(f, "{}", native),
            Obj::Closure(closure) => {
                // debug version:
                // write!(f, "[ Closure: {} ]", closure.function.borrow())
                write!(f, "{}", closure.function.borrow())
            }
            Obj::Fun(fun) => write!(f, "{}", fun),
            Obj::Str(s) => write!(f, "{}", s),
            Obj::Upval(_value) => {
                // debug version (probably outdated):
                // write!(f, "{}", value.location.borrow())
                write!(f, "upvalue")
            }
        }
    }
}

impl Obj {
    pub fn is_builtin(&self) -> bool {
        matches!(self, Obj::BuiltIn(_))
    }

    pub fn is_closure(&self) -> bool {
        matches!(self, Obj::Closure(_))
    }

    pub fn is_fun(&self) -> bool {
        matches!(self, Obj::Fun(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Obj::Str(_))
    }

    pub fn is_obj_type(&self, kind: &'static str) -> bool {
        match kind {
            "BuiltIn" => self.is_builtin(),
            "Closure" => self.is_closure(),
            "Function" => self.is_fun(),
            "String" => self.is_string(),
            _ => panic!("Invalid Obj type specified: {}.", kind),
        }
    }

    /// Extract inner `Native`. This function returns cloned object,
    /// not the original one.
    ///
    /// # Safety
    ///
    /// Fails if `Obj::is_builtin()` returns `false`.
    /// Use `Obj::is_builtin()` before applying this function.
    pub unsafe fn as_builtin(&self) -> Native {
        match self {
            Obj::BuiltIn(native) => native.clone(),
            _ => panic!("Expected Native object."),
        }
    }

    /// Extract inner `Closure`. This function returns cloned object,
    /// not the original one.
    ///
    /// # Safety
    ///
    /// Fails if `Obj::is_closure()` returns `false`.
    /// Use `Obj::is_closure()` before applying this function.
    pub unsafe fn as_closure(&self) -> Closure {
        match self {
            Obj::Closure(closure) => closure.clone(),
            _ => panic!("Expected Closure object."),
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
    upvalue_count: isize,
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
            upvalue_count: 0,
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

    pub fn upvalue_count(&self) -> isize {
        self.upvalue_count
    }

    pub fn change_upvalue_count(&mut self, upvalue_count: isize) {
        self.upvalue_count = upvalue_count;
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

#[derive(Clone)]
pub struct Closure {
    function: Rc<RefCell<Function>>,
    upvalues: Vec<Rc<RefCell<Upvalue>>>,
    upvalue_count: isize,
}

impl Default for Closure {
    fn default() -> Self {
        Self::create(&Rc::new(RefCell::new(Function::default())), Vec::new(), 0)
    }
}

impl Closure {
    pub fn create(
        function: &Rc<RefCell<Function>>,
        upvalues: Vec<Rc<RefCell<Upvalue>>>,
        upvalue_count: isize,
    ) -> Self {
        Closure {
            function: function.clone(),
            upvalues,
            upvalue_count,
        }
    }

    pub fn new(function: &Rc<RefCell<Function>>) -> Self {
        let upvalue_count = function.borrow().upvalue_count() as usize;
        let upvalues = {
            // Note: each element points to the same `Rc` instance
            let data = Rc::new(RefCell::new(Upvalue::default()));
            vec![data; upvalue_count]
        };
        Closure {
            function: function.clone(),
            upvalues,
            upvalue_count: upvalue_count as isize,
        }
    }

    pub fn function(&self) -> Rc<RefCell<Function>> {
        Rc::clone(&self.function)
    }

    pub fn chunk(&self) -> Rc<RefCell<Chunk>> {
        self.function.borrow().chunk()
    }

    pub fn upvalue_count(&self) -> isize {
        self.upvalue_count
    }

    pub fn set_upvalue(&mut self, index: usize, value: Rc<RefCell<Upvalue>>) {
        self.upvalues[index] = value;
    }

    pub fn upvalue(&self, index: usize) -> Rc<RefCell<Upvalue>> {
        Rc::clone(&self.upvalues[index])
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        *self.function.borrow() == *other.function.borrow()
    }
}

impl Eq for Closure {}

#[derive(Clone, PartialEq, Eq)]
pub struct Native {
    name: String,
}

impl Native {
    pub fn new(name: String) -> Self {
        Native { name }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn call(&self, _arg_count: usize, _args: usize) -> Value {
        match self.name.as_str() {
            "clock" => Value::Num(crate::native::clock()),
            _ => panic!(
                "Call of unknown Native function with name: '{}'.",
                self.name
            ),
        }
    }
}

impl fmt::Display for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fun {}>", &self.name)
    }
}

#[derive(Clone)]
pub struct Upvalue {
    location: usize,
    is_closed: bool,
    closed: Box<Value>,
    next: Option<Rc<RefCell<Upvalue>>>,
}

impl Default for Upvalue {
    fn default() -> Self {
        Self::create(0, false, Box::new(Value::Nil), None)
    }
}

impl Upvalue {
    pub fn create(
        location: usize,
        is_closed: bool,
        closed: Box<Value>,
        next: Option<Rc<RefCell<Self>>>,
    ) -> Self {
        Self {
            location,
            is_closed,
            closed,
            next,
        }
    }

    pub fn new(location: usize) -> Self {
        Self {
            location,
            is_closed: false,
            closed: Box::new(Value::Nil),
            next: None,
        }
    }

    pub fn location(&self) -> usize {
        self.location
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn next(&self) -> Option<Rc<RefCell<Upvalue>>> {
        if self.next.is_some() {
            Some(Rc::clone(self.next.as_ref().unwrap()))
        } else {
            // panic!("Next Upvalue expected to be non-null.");
            None
        }
    }

    pub fn set_next(&mut self, next: Option<Rc<RefCell<Upvalue>>>) {
        self.next = next;
    }

    pub fn set_closed(&mut self) {
        self.is_closed = true;
    }

    pub fn set_closed_value(&mut self, closed: Box<Value>) {
        self.closed = closed;
    }

    pub fn closed_value(&self) -> Value {
        *self.closed.clone()
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        if !self.is_closed && !other.is_closed {
            self.location == other.location
        } else if self.is_closed && other.is_closed {
            match *self.closed {
                Value::Bool(l) => match *other.closed {
                    Value::Bool(r) => l == r,
                    _ => false,
                },
                Value::Nil => matches!(*other.closed, Value::Nil),
                Value::Obj(ref l) => match *other.closed {
                    Value::Obj(ref r) => *l == *r,
                    _ => false,
                },
                Value::Num(l) => match *other.closed {
                    Value::Num(r) => l == r,
                    _ => false,
                },
            }
        } else {
            false
        }
    }
}

impl Eq for Upvalue {}

// #[derive(Clone)]
// pub struct Upvalue {
//     location: Rc<RefCell<Value>>,
// }

// impl Default for Upvalue {
//     fn default() -> Self {
//         Self::new(Value::Nil)
//     }
// }

// impl Upvalue {
//     pub fn new(location: Value) -> Self {
//         Self { location: Rc::new(RefCell::new(location)) }
//     }

//     pub fn location(&self) -> Rc<RefCell<Value>> {
//         Rc::clone(&self.location)
//     }
// }

// impl PartialEq for Upvalue {
//     fn eq(&self, other: &Self) -> bool {
//         *self.location.borrow() == *other.location.borrow()
//     }
// }

// impl Eq for Upvalue {}
