use crate::expr::Expr;
use crate::token::Token;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

pub enum Stmt {
    Expression {
        expression: Box<Expr>,
    },
    Print {
        expression: Box<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Block {
        statements: LinkedList<Box<Stmt>>,
    },
}

pub struct Environment {
    values: HashMap<String, Box<dyn Any>>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from(env: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(env.clone()),
        }
    }

    pub fn define(&mut self, key: String, value: Box<dyn Any>) -> Option<Box<dyn Any>> {
        return self.values.insert(key, value);
    }

    pub fn assign(&mut self, key: String, value: Box<dyn Any>) -> Option<Box<dyn Any>> {
        if self.values.contains_key(&key) {
            return self.values.insert(key, value);
        } else {
            match &self.enclosing {
                None => return None,
                Some(env) => return env.borrow_mut().assign(key, value),
            }
        }
    }

    pub fn get(&self, key: &String) -> Option<Box<dyn Any>> {
        match self.values.get(key) {
            None => match &self.enclosing {
                None => return None,
                Some(env) => return env.borrow_mut().get(key),
            },
            Some(val) => {
                if let Some(value) = val.as_ref().downcast_ref::<f64>() {
                    return Some(Box::new(value.clone()));
                }
                if let Some(value) = val.as_ref().downcast_ref::<String>() {
                    return Some(Box::new(value.clone()));
                }
                if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                    return Some(Box::new(value.clone()));
                }
                return None;
            }
        }
    }
}
