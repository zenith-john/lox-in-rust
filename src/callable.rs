use crate::interpreter::{evaluate, execute, rc_to_string};
use crate::stmt::{Environment, Stmt};
use crate::token::Token;
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

pub trait Callable {
    fn call(&self, arguments: &mut LinkedList<Rc<dyn Any>>) -> Option<Rc<dyn Any>>;
    fn arity(&self) -> usize;
}

pub struct LoxFunction {
    name: Token,
    params: LinkedList<Token>,
    body: LinkedList<Box<Stmt>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: LinkedList<Token>,
        body: LinkedList<Box<Stmt>>,
        env: Rc<RefCell<Environment>>,
    ) -> LoxFunction {
        LoxFunction {
            name: name,
            params: params,
            body: body,
            closure: env,
        }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        return self.params.len();
    }
    fn call(&self, arguments: &mut LinkedList<Rc<dyn Any>>) -> Option<Rc<dyn Any>> {
        if self.arity() != arguments.len() {
            eprintln!("Wrong argument number.");
            return None;
        }
        let env = Rc::new(RefCell::new(Environment::from(self.closure.clone())));
        for param in self.params.clone() {
            env.borrow_mut()
                .define(rc_to_string(param.lexeme?), arguments.pop_front()?);
        }
        for stmt in self.body.clone() {
            match *stmt {
                Stmt::Return { keyword: _, value } => match value {
                    None => return Some(Rc::new(true)),
                    Some(expr) => return evaluate(*expr, env.clone()),
                },
                _ => match execute(stmt, env.clone()) {
                    Ok(()) => {}
                    Err(_e) => return None,
                },
            }
        }
        return Some(Rc::new(true));
    }
}
