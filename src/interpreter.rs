use crate::callable::{Callable, LoxClass, LoxFunction};
use crate::error::RuntimeError;
use crate::expr::Expr;
use crate::stmt::{Environment, Stmt};
use crate::token::{BasicType, Token, TokenType};
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

pub fn interpret(
    stmts: LinkedList<Box<Stmt>>,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<(), RuntimeError> {
    for stmt in stmts {
        execute(*stmt, env.clone(), table)?
    }
    Ok(())
}

pub fn execute(
    stmt: Stmt,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<(), RuntimeError> {
    match stmt {
        Stmt::Block { statements } => {
            let new_env = Rc::new(RefCell::new(Environment::from(env.clone())));
            match interpret(statements, new_env.clone(), table) {
                Ok(()) => {}
                Err(s) => return Err(s),
            }
            Ok(())
        }
        Stmt::Class {
            name,
            superclass,
            methods,
        } => {
            let mut sp: Option<Rc<LoxClass>> = None;
            let mut local_env = env.clone();
            if let Some(expr) = superclass {
                if let Some(val) = evaluate(*expr.clone(), env.clone(), table)
                    .expect("Non empty")
                    .as_class()
                {
                    sp = Some(val.clone());
                    local_env = Rc::new(RefCell::new(Environment::from(env.clone())));
                    local_env
                        .borrow_mut()
                        .define("super".to_string(), BasicType::Class(val));
                } else {
                    return Err(RuntimeError::new(
                        expr.line_number(),
                        format!("{} is not a class name.", expr),
                    ));
                }
            }
            local_env = Rc::new(RefCell::new(Environment::from(local_env.clone())));

            let mut kmethods: HashMap<String, LoxFunction> = HashMap::new();
            for method in methods {
                if let Stmt::Function {
                    name: new_name,
                    params,
                    body,
                } = *method
                {
                    let st = new_name
                        .lexeme
                        .clone()
                        .unwrap()
                        .as_string()
                        .expect("Must be a identifier.")
                        .clone();
                    kmethods.insert(
                        st,
                        LoxFunction::new(new_name, params, body, local_env.clone(), table.clone()),
                    );
                }
            }
            let klass = BasicType::Class(Rc::new(LoxClass::new(name.clone(), sp, kmethods)));
            let st = name
                .lexeme
                .unwrap()
                .as_string()
                .expect("Must be a identifier.")
                .clone();
            env.borrow_mut().define(st, klass);
            Ok(())
        }
        Stmt::Expression { expression } => match evaluate(*expression, env.clone(), table) {
            Err(e) => Err(e),
            _ => Ok(()),
        },
        Stmt::Function { name, params, body } => {
            let fun = Rc::new(LoxFunction::new(
                name.clone(),
                params,
                body,
                env.clone(),
                table.clone(),
            ));
            let st = name
                .lexeme
                .unwrap()
                .as_string()
                .expect("Must be a identifier.")
                .clone();
            env.borrow_mut().define(st, BasicType::Function(fun));
            Ok(())
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let is_true: bool;
            let line_number = condition.line_number();
            match evaluate(*condition, env.clone(), table) {
                Err(e) => return Err(e),
                Ok(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        return Err(RuntimeError::new(
                            line_number,
                            "Statement in condition is not of bool type.".to_string(),
                        ));
                    }
                }
            }
            if is_true {
                return execute(*then_branch, env.clone(), table);
            } else if let Some(branch) = else_branch {
                return execute(*branch, env.clone(), table);
            }
            Ok(())
        }
        Stmt::Print { expression } => match evaluate(*expression, env.clone(), table) {
            Ok(value) => {
                println!("{}", value);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Stmt::Return { keyword: _, value } => match value {
            None => Err(RuntimeError::ReturnValue(BasicType::None)),
            Some(expr) => match evaluate(*expr, env.clone(), table) {
                Ok(val) => Err(RuntimeError::ReturnValue(val)),
                Err(e) => Err(e),
            },
        },
        Stmt::Var { name, initializer } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                if env.borrow().is_defined(key.to_string()) {
                    return Err(RuntimeError::new(
                        name.line,
                        format!("Multiple definition of some variable {}.", key),
                    ));
                }
                match initializer {
                    None => env.borrow_mut().define(key.clone(), BasicType::None),
                    Some(val) => {
                        let result = evaluate(*val, env.clone(), table);
                        match result {
                            Ok(val) => env.borrow_mut().define(key.clone(), val),
                            Err(e) => return Err(e),
                        }
                    }
                };
                Ok(())
            } else {
                Err(RuntimeError::new(
                    name.line,
                    "Invalid Variable Name".to_string(),
                ))
            }
        }
        Stmt::While { condition, body } => {
            let mut is_true: bool;
            match evaluate(*condition.clone(), env.clone(), table) {
                Err(e) => return Err(e),
                Ok(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        return Err(RuntimeError::new(
                            condition.line_number(),
                            "Statement in condition is not of bool type.".to_string(),
                        ));
                    }
                }
            }
            while is_true {
                execute(*body.clone(), env.clone(), table)?;
                match evaluate(*condition.clone(), env.clone(), table) {
                    Err(e) => return Err(e),
                    Ok(val) => {
                        if let Some(value) = val.as_bool() {
                            is_true = value;
                        } else {
                            return Err(RuntimeError::new(
                                condition.line_number(),
                                "Statement in condition is not of bool type.".to_string(),
                            ));
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

pub fn evaluate(
    expr: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<BasicType, RuntimeError> {
    let line_number = expr.line_number();
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => binary_eval(*left, operator, *right, env, table),
        Expr::Call {
            callee,
            paren: _,
            arguments,
        } => {
            let callee_evaluated = evaluate(*callee, env.clone(), table)?;
            let mut args: LinkedList<BasicType> = LinkedList::new();
            for expr in arguments {
                match evaluate(*expr, env.clone(), table) {
                    Err(e) => return Err(e),
                    Ok(val) => args.push_back(val),
                }
            }
            if let BasicType::Function(val) = callee_evaluated {
                val.call(&mut args, line_number)
            } else if let BasicType::Class(val) = callee_evaluated {
                val.call(&mut args, line_number)
            } else {
                Err(RuntimeError::new(
                    line_number,
                    format!("Callee {} is not a function.", callee_evaluated),
                ))
            }
        }
        Expr::Get { object, name } => {
            let ob = evaluate(*object, env, table)?;
            if let BasicType::Instance(val) = ob.clone() {
                let st = name.lexeme.unwrap().as_string().unwrap();
                if val.borrow_mut().fields.contains_key(&st) {
                    return Ok(val
                        .borrow_mut()
                        .fields
                        .get(&st)
                        .cloned()
                        .expect("Key is found."));
                }
                let mut klass = val.borrow_mut().clone().klass.clone();
                loop {
                    if let Some(method) = klass.find_method(st.clone()) {
                        return Ok(BasicType::Function(Rc::new(method.bind(val))));
                    }
                    match klass.superclass() {
                        None => {
                            return Err(RuntimeError::new(
                                line_number,
                                "Undefined property.".to_string(),
                            ));
                        }
                        Some(val) => klass = val,
                    }
                }
            } else {
                Err(RuntimeError::new(
                    line_number,
                    "Invalid property call.".to_string(),
                ))
            }
        }
        Expr::Grouping { expression } => evaluate(*expression, env, table),
        Expr::Literal { value } => Ok(value),
        Expr::Logical {
            left,
            operator,
            right,
        } => {
            let is_true: bool;
            match evaluate(*left, env.clone(), table) {
                Err(e) => return Err(e),
                Ok(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        return Err(RuntimeError::new(
                            line_number,
                            "Statement in condition is not of bool type.".to_string(),
                        ));
                    }
                }
            }
            if operator.ttype == TokenType::Or {
                if is_true {
                    return Ok(BasicType::Bool(is_true));
                }
            } else if !is_true {
                return Ok(BasicType::Bool(is_true));
            }
            evaluate(*right, env.clone(), table)
        }
        Expr::Set {
            object,
            name,
            value,
        } => {
            let ob = evaluate(*object, env.clone(), table)?;
            if let BasicType::Instance(val) = ob.clone() {
                let v = evaluate(*value, env.clone(), table)?;
                val.borrow_mut().set(name, v.clone());
                Ok(v)
            } else {
                Err(RuntimeError::new(
                    line_number,
                    "Invalid property call.".to_string(),
                ))
            }
        }
        Expr::Super {
            keyword: _,
            method,
            id,
        } => {
            let depth = table.get(&id).expect("ID automatically generated.");
            let superclass = match env.borrow_mut().get(&"super".to_string(), *depth) {
                None => {
                    return Err(RuntimeError::new(
                        line_number,
                        "Don't know what \"super\" referred to.".to_string(),
                    ));
                }
                Some(val) => val.as_class().expect("Lox Class"),
            };
            let object = match env.borrow_mut().get(&"this".to_string(), *depth - 1) {
                None => {
                    return Err(RuntimeError::new(
                        line_number,
                        "Don't know what \"this\" referred to.".to_string(),
                    ));
                }
                Some(val) => val.as_instance().expect("Lox Instance"),
            };
            let st = method.lexeme.unwrap().as_string().unwrap();
            let mut klass = superclass.clone();
            loop {
                if let Some(method) = klass.find_method(st.clone()) {
                    return Ok(BasicType::Function(Rc::new(method.bind(object))));
                }
                match klass.superclass() {
                    None => {
                        return Err(RuntimeError::new(
                            line_number,
                            "Undefined property.".to_string(),
                        ));
                    }
                    Some(val) => klass = val,
                }
            }
        }
        Expr::This { keyword: _, id } => {
            let depth = table.get(&id).expect("ID automatically generated.");
            match env.borrow_mut().get(&"this".to_string(), *depth) {
                None => Err(RuntimeError::new(
                    line_number,
                    "Don't know what \"this\" referred to.".to_string(),
                )),
                Some(val) => Ok(val),
            }
        }
        Expr::Unary { operator, right } => unitary_eval(operator, *right, env, table),
        Expr::Variable { name, id } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                let depth = table.get(&id).expect("ID automatically generated.");
                return match env.borrow_mut().get(&key, *depth) {
                    None => Err(RuntimeError::new(
                        line_number,
                        format!("Undefined Variable {}.", key),
                    )),
                    Some(val) => Ok(val),
                };
            } else {
                Err(RuntimeError::new(
                    line_number,
                    "Invalid identifier.".to_string(),
                ))
            }
        }
        Expr::Assign { name, value, id } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                let depth = table.get(&id).expect("ID automatically generated.");
                let val: BasicType = evaluate(*value, env.clone(), table)?;
                return Ok(env
                    .borrow_mut()
                    .assign(key.clone(), val, *depth)
                    .expect("Always initialized."));
            } else {
                Err(RuntimeError::new(
                    line_number,
                    "Invalid identifier.".to_string(),
                ))
            }
        }
    }
}

fn unitary_eval(
    token: Token,
    expr: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<BasicType, RuntimeError> {
    let line_number = expr.line_number();
    let right = evaluate(expr, env.clone(), table)?;

    match token.ttype {
        TokenType::Minus => match right.as_number() {
            Some(x) => Ok(BasicType::Number(-x)),
            _ => Err(RuntimeError::new(line_number, "Type mismatch.".to_string())),
        },
        TokenType::Bang => {
            if let Some(x) = right.as_bool() {
                Ok(BasicType::Bool(!x))
            } else {
                Err(RuntimeError::new(line_number, "Type mismatch.".to_string()))
            }
        }
        _ => Err(RuntimeError::new(
            line_number,
            "Unknown operator.".to_string(),
        )),
    }
}

fn binary_eval(
    expr1: Expr,
    token: Token,
    expr2: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<BasicType, RuntimeError> {
    let left = evaluate(expr1, env.clone(), table)?;
    let right = evaluate(expr2, env.clone(), table)?;

    match token.ttype {
        TokenType::Minus => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Number(x - y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },
        TokenType::Slash => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => {
                if y == 0.0 {
                    Err(RuntimeError::new(token.line, "Divide by 0.".to_string()))
                } else {
                    Ok(BasicType::Number(x / y))
                }
            }
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },
        TokenType::Star => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Number(x * y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },
        TokenType::Plus => {
            if let (Some(x), Some(y)) = (left.as_number(), right.as_number()) {
                return Ok(BasicType::Number(x + y));
            }

            if let (Some(x), Some(y)) = (left.as_string(), right.as_string()) {
                return Ok(BasicType::String(x.clone() + &*y));
            }
            Err(RuntimeError::new(token.line, "Type mismatch.".to_string()))
        }

        TokenType::Greater => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Bool(x > y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },

        TokenType::GreaterEqual => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Bool(x >= y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },

        TokenType::Less => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Bool(x < y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },
        TokenType::LessEqual => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Ok(BasicType::Bool(x <= y)),
            _ => Err(RuntimeError::new(token.line, "Type mismatch.".to_string())),
        },
        TokenType::BangEqual => Ok(BasicType::Bool(!(left == right))),
        TokenType::EqualEqual => Ok(BasicType::Bool(left == right)),
        _ => Err(RuntimeError::new(
            token.line,
            "Unknown operator.".to_string(),
        )),
    }
}
