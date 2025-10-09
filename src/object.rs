use crate::chunk::Chunk;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum LoxType {
    None,
    String(String),
    Number(f64),
    Bool(bool),
    Function(Rc<Function>),
    Closure(Closure),
    Class(Rc<Class>),
    Instance(Rc<RefCell<Instance>>),
}

impl LoxType {
    pub fn as_string(&self) -> Option<String> {
        if let LoxType::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let LoxType::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<Rc<Function>> {
        if let LoxType::Function(func) = self {
            Some(func.clone())
        } else {
            None
        }
    }
}

impl fmt::Display for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxType::String(s) => write!(f, "{}", s),
            LoxType::Number(n) => write!(f, "{}", n),
            LoxType::Bool(b) => write!(f, "{}", b),
            LoxType::Function(fun) => write!(f, "{}", fun.name),
            LoxType::Closure(c) => write!(f, "{}", c.function.name),
            LoxType::Class(k) => write!(f, "{}", k.name),
            LoxType::Instance(i) => write!(f, "Instance of {}", i.borrow().klass.name),
            LoxType::None => write!(f, "Nil"),
        }
    }
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxType::String(s1), LoxType::String(s2)) => s1 == s2,
            (LoxType::Bool(b1), LoxType::Bool(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl fmt::Debug for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone)]
pub struct Function {
    pub arity: u8,
    pub upvalue: u8,
    pub chunk: Box<Chunk>,
    pub name: String,
}

#[derive(Clone)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

impl Closure {
    pub fn new(func: Rc<Function>) -> Closure {
        Closure {
            function: func.clone(),
            upvalues: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Class {
    pub name: String,
}

#[derive(Clone)]
pub struct Instance {
    pub klass: Rc<Class>,
    pub fields: HashMap<String, LoxType>,
}

#[derive(Clone)]
pub enum Upvalue {
    Stack(usize),
    Out(LoxType),
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Instance {
        Instance {
            klass,
            fields: HashMap::new(),
        }
    }
}
