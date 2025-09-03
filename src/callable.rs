use crate::error::RuntimeError;
use crate::interpreter::{evaluate, execute};
use crate::stmt::{Environment, Stmt};
use crate::token::{BasicType, Token};
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

pub trait Callable {
    fn call(&self, arguments: &mut LinkedList<BasicType>) -> Result<BasicType, RuntimeError>;
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
    fn call(&self, arguments: &mut LinkedList<BasicType>) -> Result<BasicType, RuntimeError> {
        if self.arity() != arguments.len() {
            return Err(RuntimeError::new("Wrong argument number.".to_string()));
        }
        let env = Rc::new(RefCell::new(Environment::from(self.closure.clone())));
        for param in self.params.clone() {
            env.borrow_mut().define(
                (param.lexeme.expect("Well defined variables."))
                    .as_string()
                    .unwrap(),
                arguments
                    .pop_front()
                    .ok_or(RuntimeError::new("Invalid Argument".to_string()))?,
            );
        }
        for stmt in self.body.clone() {
            match *stmt {
                Stmt::Return { keyword: _, value } => match value {
                    None => return Ok(BasicType::Bool(true)),
                    Some(expr) => return evaluate(*expr, env.clone(), &self.table),
                },
                _ => match execute(*stmt, env.clone(), &self.table) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(RuntimeError::new(format!(
                            "Error in function {}\n{}",
                            self.name.lexeme.clone().unwrap(),
                            e
                        )));
                    }
                },
            }
        }
        Ok(BasicType::Bool(true))
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
    fn call(&self, _arguments: &mut LinkedList<BasicType>) -> Result<BasicType, RuntimeError> {
        Ok(BasicType::Instance(Rc::new(RefCell::new(
            LoxInstance::new(Rc::new(self.clone())),
        ))))
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
        let st = name.lexeme.unwrap().as_string().unwrap();
        self.fields.insert(st, value.clone())
    }
}
