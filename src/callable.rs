use crate::interpreter::{evaluate, execute};
use crate::stmt::{Environment, Stmt};
use crate::token::{Token, BasicType};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::{HashMap, LinkedList};

pub trait Callable {
    fn call(&self, arguments: &mut LinkedList<BasicType>) -> Option<BasicType>;
    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct LoxFunction {
    pub name: Token,
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
            name,
            params,
            body,
            closure: env,
            table,
        }
    }

    pub fn bind(self, instance: Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let new_env = self.closure.clone();
        new_env
            .borrow_mut()
            .define("this".to_string(), BasicType::Instance(instance.clone()));
        Self::new(self.name, self.params, self.body, new_env, self.table)
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }
    fn call(&self, arguments: &mut LinkedList<BasicType>) -> Option<BasicType> {
        if self.arity() != arguments.len() {
            eprintln!("Wrong argument number.");
            return None;
        }
        let env = Rc::new(RefCell::new(Environment::from(self.closure.clone())));
        for param in self.params.clone() {
            env.borrow_mut()
                .define((param.lexeme?).as_string().unwrap(), arguments.pop_front()?);
        }
        for stmt in self.body.clone() {
            match *stmt {
                Stmt::Return { keyword, value } => match value {
                    None => return Some(BasicType::Bool(true)),
                    Some(expr) => match evaluate(*expr, env.clone(), &self.table) {
                        None => {
                            eprintln!("Error in line: {}", keyword.line);
                            return None;
                        }
                        Some(val) => return Some(val),
                    },
                },
                _ => match execute(*stmt, env.clone(), &self.table) {
                    Ok(()) => {}
                    Err(_e) => {
                        eprintln!(
                            "Error in function {}",
                            self.name.lexeme.clone().unwrap());
                        return None;
                    }
                },
            }
        }
        Some(BasicType::Bool(true))
    }
}

#[derive(Clone)]
pub struct LoxClass {
    pub name: Token,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(
        name: Token,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, LoxFunction>,
    ) -> LoxClass {
        LoxClass {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, method: String) -> Option<LoxFunction> {
        self.methods.get(&method).cloned()
    }

    pub fn superclass(&self) -> Option<Rc<LoxClass>> {
        self.superclass.clone()
    }
}

impl Callable for LoxClass {
    fn call(&self, _arguments: &mut LinkedList<BasicType>) -> Option<BasicType> {
        Some(BasicType::Instance(Rc::new(RefCell::new(LoxInstance::new(Rc::new(
            self.clone(),
        ))))))
    }
    fn arity(&self) -> usize {
        0
    }
}

#[derive(Clone)]
pub struct LoxInstance {
    pub klass: Rc<LoxClass>,
    pub fields: HashMap<String, BasicType>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> LoxInstance {
        LoxInstance {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: Token, value: BasicType) -> Option<BasicType> {
        let st = name
            .lexeme
            .unwrap()
            .as_string()
            .unwrap();
        self.fields.insert(st, value.clone())
    }
}
