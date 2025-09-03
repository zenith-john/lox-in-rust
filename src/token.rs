use crate::callable::{LoxClass, LoxFunction, LoxInstance};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: Option<BasicType>,
    pub line: i32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.lexeme {
            Some(x) => write!(f, "{}", x),
            None => write!(f, "{:?}", self.ttype),
        }
    }
}
#[derive(Clone)]
pub enum BasicType {
    None,
    String(String),
    Number(f64),
    Bool(bool),
    Function(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl BasicType {
    pub fn as_string(&self) -> Option<String> {
        if let BasicType::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let BasicType::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let BasicType::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_class(&self) -> Option<Rc<LoxClass>> {
        if let BasicType::Class(c) = self {
            Some(c.clone())
        } else {
            None
        }
    }
    pub fn as_instance(&self) -> Option<Rc<RefCell<LoxInstance>>> {
        if let BasicType::Instance(i) = self {
            Some(i.clone())
        } else {
            None
        }
    }
}

impl fmt::Display for BasicType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BasicType::String(s) => write!(f, "{}", s),
            BasicType::Number(n) => write!(f, "{}", n),
            BasicType::Bool(b) => write!(f, "{}", b),
            BasicType::Function(l) => write!(f, "{}", l.name.lexeme.clone().unwrap()),
            BasicType::Class(c) => write!(f, "{}", c.name.lexeme.clone().unwrap()),
            BasicType::Instance(_) => write!(f, ""),
            BasicType::None => write!(f, "Nil"),
        }
    }
}

impl PartialEq for BasicType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BasicType::String(s1), BasicType::String(s2)) => s1 == s2,
            (BasicType::Bool(b1), BasicType::Bool(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl fmt::Debug for BasicType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
