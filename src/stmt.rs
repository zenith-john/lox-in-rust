use crate::expr::Expr;
use crate::token::{Token, BasicType};
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

#[derive(Clone)]
pub enum Stmt {
    Block {
        statements: LinkedList<Box<Stmt>>,
    },
    Class {
        name: Token,
        superclass: Option<Box<Expr>>,
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
    values: HashMap<String, BasicType>,
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

    pub fn define(&mut self, key: String, value: BasicType) -> Option<BasicType> {
        self.values.insert(key, value)
    }

    pub fn is_defined(&self, key: String) -> bool {
        self.values.contains_key(&key)
    }

    pub fn assign(&mut self, key: String, value: BasicType, depth: i32) -> Option<BasicType> {
        if depth == 0 {
            self.values.insert(key, value)
        } else {
            (*self.enclosing.clone()?)
                .borrow_mut()
                .assign(key, value, depth - 1)
        }
    }

    pub fn get(&self, key: &String, depth: i32) -> Option<BasicType> {
        if depth == 0 {
            self.values.get(key).cloned()
        }
        else {
            return (*self.enclosing.clone()?).borrow().get(key, depth - 1);
        }
    }
}
