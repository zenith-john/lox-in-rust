use crate::interpreter::{evaluate, execute, rc_to_string};
use crate::stmt::{Environment, Stmt};
use crate::token::Token;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
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
    table: HashMap<u64, i32>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: LinkedList<Token>,
        body: LinkedList<Box<Stmt>>,
        env: Rc<RefCell<Environment>>,
        table: HashMap<u64, i32>,
    ) -> LoxFunction {
        LoxFunction {
            name: name,
            params: params,
            body: body,
            closure: env,
            table: table,
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
                Stmt::Return { keyword, value } => match value {
                    None => return Some(Rc::new(true)),
                    Some(expr) => match evaluate(*expr, env.clone(), &self.table) {
                        None => {
                            eprintln!("Error in line: {}", keyword.line);
                            return None;
                        }
                        Some(val) => return Some(val),
                    },
                },
                _ => match execute(stmt, env.clone(), &self.table) {
                    Ok(()) => {}
                    Err(_e) => {
                        eprintln!(
                            "Error in function {}",
                            rc_to_string(self.name.lexeme.clone()?)
                        );
                        return None;
                    }
                },
            }
        }
        return Some(Rc::new(true));
    }
}
