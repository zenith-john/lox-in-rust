use crate::chunk::Chunk;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum LoxType {
    None,
    String(String),
    Number(f64),
    Bool(bool),
    Function(Rc<Function>),
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
}

impl fmt::Display for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxType::String(s) => write!(f, "{}", s),
            LoxType::Number(n) => write!(f, "{}", n),
            LoxType::Bool(b) => write!(f, "{}", b),
            LoxType::Function(l) => write!(f, "{}", l.name),
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
    pub chunk: Box<Chunk>,
    pub name: String,
}
