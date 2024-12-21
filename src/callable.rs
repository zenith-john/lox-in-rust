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

#[derive(Clone)]
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

    pub fn bind(self, instance: Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let new_env = self.closure.clone();
        new_env
            .borrow_mut()
            .define("this".to_string(), instance.clone());
        return Self::new(self.name, self.params, self.body, new_env, self.table);
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

#[derive(Clone)]
pub struct LoxClass {
    name: Token,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: Token, methods: HashMap<String, LoxFunction>) -> LoxClass {
        return LoxClass {
            name: name,
            methods: methods,
        };
    }

    pub fn find_method(&self, method: String) -> Option<LoxFunction> {
        return self.methods.get(&method).cloned();
    }
}

impl Callable for LoxClass {
    fn call(&self, _arguments: &mut LinkedList<Rc<dyn Any>>) -> Option<Rc<dyn Any>> {
        return Some(Rc::new(RefCell::new(LoxInstance::new(self.clone()))));
    }
    fn arity(&self) -> usize {
        return 0;
    }
}

#[derive(Clone)]
pub struct LoxInstance {
    pub klass: LoxClass,
    pub fields: HashMap<String, Rc<dyn Any>>,
}

impl LoxInstance {
    pub fn new(klass: LoxClass) -> LoxInstance {
        return LoxInstance {
            klass: klass,
            fields: HashMap::new(),
        };
    }

    // pub fn get(&mut self, name: Token) -> Option<Rc<dyn Any>> {
    //     let st = name.lexeme.unwrap().as_ref().downcast_ref::<String>()?.to_string();
    //     if self.fields.contains_key(&st) {
    //         return self.fields.get(&st).cloned();
    //     }

    //     eprintln!("Undefined property.");
    //     return None;
    // }

    pub fn set(&mut self, name: Token, value: Rc<dyn Any>) -> Option<Rc<dyn Any>> {
        let st = name
            .lexeme
            .unwrap()
            .as_ref()
            .downcast_ref::<String>()?
            .to_string();
        return self.fields.insert(st, value.clone());
    }
}
