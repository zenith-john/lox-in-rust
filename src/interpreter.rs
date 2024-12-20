use crate::callable::{Callable, LoxFunction};
use crate::expr::Expr;
use crate::stmt::{Environment, Stmt};
use crate::token::{Token, TokenType};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

pub fn rc_to_string(rc: Rc<dyn Any>) -> String {
    if let Some(value) = rc.as_ref().downcast_ref::<f64>() {
        return format!("{value}");
    }
    if let Some(value) = rc.as_ref().downcast_ref::<String>() {
        return format!("{value}");
    }
    if let Some(value) = rc.as_ref().downcast_ref::<bool>() {
        return format!("{value}");
    } else {
        return format!("{rc:?}");
    }
}

pub fn interpret(
    stmts: LinkedList<Box<Stmt>>,
    env: Rc<RefCell<Environment>>,
) -> Result<(), &'static str> {
    for stmt in stmts {
        match execute(stmt, env.clone()) {
            Err(s) => return Err(s),
            Ok(()) => {}
        }
    }
    return Ok(());
}

pub fn execute(stmt: Box<Stmt>, env: Rc<RefCell<Environment>>) -> Result<(), &'static str> {
    match *stmt {
        Stmt::Expression { expression } => match evaluate(*expression, env.clone()) {
            None => return Err("Runtime Error"),
            _ => return Ok(()),
        },
        Stmt::Function { name, params, body } => {
            let fun = Rc::new(LoxFunction::new(name.clone(), params, body, env.clone()));
            let st = name
                .lexeme
                .unwrap()
                .as_ref()
                .downcast_ref::<String>()
                .ok_or("Not a correct identifier")?
                .clone();
            env.borrow_mut().define(st, fun);
            return Ok(());
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let is_true: bool;
            match evaluate(*condition, env.clone()) {
                None => return Err("Runtime Error"),
                Some(val) => {
                    if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                        is_true = *value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return Err("Runtime Error");
                    }
                }
            }
            if is_true {
                return execute(then_branch, env.clone());
            } else if let Some(branch) = else_branch {
                return execute(branch, env.clone());
            }
            return Ok(());
        }
        Stmt::Print { expression } => {
            if let Some(value) = evaluate(*expression, env.clone()) {
                println!("{}", rc_to_string(value));
            } else {
                return Err("Runtime Error");
            }
            return Ok(());
        }
        Stmt::Return {
            keyword: _,
            value: _,
        } => {
            return Ok(()); // Should not handle it here.
        }
        Stmt::Var { name, initializer } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                match initializer {
                    None => env.borrow_mut().define(key.clone(), Rc::new(0)),
                    Some(val) => {
                        let result = evaluate(*val, env.clone());
                        match result {
                            Some(val) => env.borrow_mut().define(key.clone(), val),
                            None => return Err("Runtime Error"),
                        }
                    }
                };
                return Ok(());
            } else {
                return Err("Invalid Variable Name");
            }
        }
        Stmt::While { condition, body } => {
            let mut is_true: bool;
            match evaluate(*condition.clone(), env.clone()) {
                None => return Err("Runtime Error"),
                Some(val) => {
                    if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                        is_true = *value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return Err("Runtime Error");
                    }
                }
            }
            while is_true {
                match execute(body.clone(), env.clone()) {
                    Err(str) => return Err(str),
                    Ok(()) => {}
                }
                match evaluate(*condition.clone(), env.clone()) {
                    None => return Err("Runtime Error"),
                    Some(val) => {
                        if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                            is_true = *value;
                        } else {
                            eprintln!("Statement in condition is not of bool type.");
                            return Err("Runtime Error");
                        }
                    }
                }
            }
            return Ok(());
        }
        Stmt::Block { statements } => {
            let new_env = Rc::new(RefCell::new(Environment::from(env.clone())));
            match interpret(statements, new_env.clone()) {
                Ok(()) => {}
                Err(s) => return Err(s),
            }
            return Ok(());
        }
    }
}

pub fn evaluate(expr: Expr, env: Rc<RefCell<Environment>>) -> Option<Rc<dyn Any>> {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => return binary_eval(*left, operator, *right, env),
        Expr::Call {
            callee,
            paren: _,
            arguments,
        } => {
            let callee_evaluated = match evaluate(*callee, env.clone()) {
                None => return None,
                Some(val) => val,
            };
            let mut args: LinkedList<Rc<dyn Any>> = LinkedList::new();
            for expr in arguments {
                match evaluate(*expr, env.clone()) {
                    None => return None,
                    Some(val) => args.push_back(val),
                }
            }
            if let Ok(val) = callee_evaluated.clone().downcast::<LoxFunction>() {
                return val.call(&mut args);
            } else {
                eprintln!(
                    "Callee {} is not a function.",
                    rc_to_string(callee_evaluated)
                );
                return None;
            }
        }
        Expr::Grouping { expression } => {
            return evaluate(*expression, env);
        }
        Expr::Literal { value } => return Some(value),
        Expr::Logical {
            left,
            operator,
            right,
        } => {
            let is_true: bool;
            match evaluate(*left, env.clone()) {
                None => return None,
                Some(val) => {
                    if let Some(value) = val.as_ref().downcast_ref::<bool>() {
                        is_true = *value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return None;
                    }
                }
            }
            if operator.ttype == TokenType::OR {
                if is_true {
                    return Some(Rc::new(is_true));
                }
            } else {
                if !is_true {
                    return Some(Rc::new(is_true));
                }
            }

            return evaluate(*right, env.clone());
        }
        Expr::Unary { operator, right } => return unitary_eval(operator, *right, env),
        Expr::Variable { name } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                return match env.borrow_mut().get(key) {
                    None => {
                        eprintln!("Undefined Variable {}.", key);
                        None
                    }
                    Some(val) => Some(val),
                };
            } else {
                eprintln!("Invalid identifier.");
                return None;
            }
        }
        Expr::Assign { name, value } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                let val: Rc<dyn Any>;
                match evaluate(*value, env.clone()) {
                    None => return None,
                    Some(x) => val = x,
                }
                return env.borrow_mut().assign(key.clone(), val);
            } else {
                eprintln!("Invalid identifier.");
                return None;
            }
        }
    }
}

fn unitary_eval(token: Token, expr: Expr, env: Rc<RefCell<Environment>>) -> Option<Rc<dyn Any>> {
    let right;
    match evaluate(expr, env.clone()) {
        Some(x) => right = x,
        None => return None,
    }

    match token.ttype {
        TokenType::MINUS => match right.downcast::<f64>() {
            Ok(x) => return Some(Rc::new(-*x)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::BANG => {
            match right.as_ref().downcast_ref::<bool>() {
                Some(x) => return Some(Rc::new(!*x)),
                _ => {}
            }

            match right.as_ref().downcast_ref::<Option<bool>>() {
                Some(x) => match *x {
                    None => return Some(Rc::new(true)),
                    Some(_x) => return Some(Rc::new(false)),
                },
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        _ => {
            eprintln!("Wrong operator.");
            return None;
        }
    }
}

fn binary_eval(
    expr1: Expr,
    token: Token,
    expr2: Expr,
    env: Rc<RefCell<Environment>>,
) -> Option<Rc<dyn Any>> {
    let left;
    let right;
    match evaluate(expr1, env.clone()) {
        Some(x) => left = x,
        None => return None,
    }
    match evaluate(expr2, env.clone()) {
        Some(x) => right = x,
        None => return None,
    }

    match token.ttype {
        TokenType::MINUS => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(*x - *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::SLASH => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => divide(*x, *y).map(|b| b as Rc<dyn Any>),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::STAR => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(*x * *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::PLUS => {
            match (
                left.as_ref().downcast_ref::<f64>(),
                right.as_ref().downcast_ref::<f64>(),
            ) {
                (Some(x), Some(y)) => return Some(Rc::new(*x + *y)),
                _ => {}
            }

            match (
                left.as_ref().downcast_ref::<String>(),
                right.as_ref().downcast_ref::<String>(),
            ) {
                (Some(x), Some(y)) => return Some(Rc::new(x.clone() + &*y)),
                _ => {}
            }
            error_type_mismatch();
            return None;
        }

        TokenType::GREATER => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(x > y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }

        TokenType::GREATER_EQUAL => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(*x >= *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }

        TokenType::LESS => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(*x < *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::LESS_EQUAL => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Rc::new(*x <= *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::BANG_EQUAL => match is_equal(left, right) {
            Some(x) => return Some(Rc::new(!*x)),
            None => return None,
        },
        TokenType::EQUAL_EQUAL => {
            return is_equal(left, right).map(|b| b as Rc<dyn Any>);
        }
        _ => {
            eprintln!("Wrong operator.");
            return None;
        }
    }
}

fn divide(numerator: f64, denominator: f64) -> Option<Rc<f64>> {
    if denominator == 0.0 {
        eprintln!("Divide by 0.");
        None
    } else {
        Some(Rc::new(numerator / denominator))
    }
}

fn is_equal(l1: Rc<dyn Any>, l2: Rc<dyn Any>) -> Option<Rc<bool>> {
    match (
        l1.as_ref().downcast_ref::<f64>(),
        l2.as_ref().downcast_ref::<f64>(),
    ) {
        (Some(x), Some(y)) => return Some(Rc::new(*x == *y)),
        _ => {}
    }

    match (
        l1.as_ref().downcast_ref::<bool>(),
        l2.as_ref().downcast_ref::<bool>(),
    ) {
        (Some(x), Some(y)) => return Some(Rc::new(*x == *y)),
        _ => {}
    }

    match (
        l1.as_ref().downcast_ref::<String>(),
        l2.as_ref().downcast_ref::<String>(),
    ) {
        (Some(x), Some(y)) => return Some(Rc::new(*x == *y)),
        _ => {}
    }

    error_type_mismatch();
    return None;
}

fn error_type_mismatch() {
    eprintln!("Type mismatch.");
}
