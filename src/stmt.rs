use crate::expr::Expr;
use crate::token::Token;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Stmt {
    Block {
        statements: LinkedList<Box<Stmt>>,
    },
    Class {
        name: Token,
        methods: LinkedList<Box<Stmt>>,
    },
    Expression {
        expression: Box<Expr>,
    },
    Function {
        name: Token,
        params: LinkedList<Token>,
        body: LinkedList<Box<Stmt>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expression: Box<Expr>,
    },
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
}

pub struct Environment {
    values: HashMap<String, Rc<dyn Any>>,
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

    pub fn define(&mut self, key: String, value: Rc<dyn Any>) -> Option<Rc<dyn Any>> {
        return self.values.insert(key, value);
    }

    pub fn is_defined(&self, key: String) -> bool {
        return self.values.contains_key(&key);
    }

    pub fn assign(&mut self, key: String, value: Rc<dyn Any>, depth: i32) -> Option<Rc<dyn Any>> {
        if depth == 0 {
            return self.values.insert(key, value);
        } else {
            return (*self.enclosing.clone()?)
                .borrow_mut()
                .assign(key, value, depth - 1);
        }
    }

    pub fn get(&self, key: &String, depth: i32) -> Option<Rc<dyn Any>> {
        if depth == 0 {
            match self.values.get(key) {
                None => return None,
                Some(val) => {
                    if let Some(value) = val.as_ref().downcast_ref::<f64>() {
                        return Some(Rc::new(value.clone()));
                    }
                    if let Some(value) = val.as_ref().downcast_ref::<String>() {
                        return Some(Rc::new(value.clone()));
                    }
                    if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                        return Some(Rc::new(value.clone()));
                    }
                    return Some(val.clone());
                }
            }
        } else {
            return (*self.enclosing.clone()?).borrow().get(key, depth - 1);
        }
    }
}
