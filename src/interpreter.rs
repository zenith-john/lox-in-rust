use crate::expr::Expr;
use crate::stmt::{Environment, Stmt};
use crate::token::{Token, TokenType};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

pub fn result_to_string(result: Box<dyn Any>) -> String {
    if let Some(value) = result.as_ref().downcast_ref::<f64>() {
        return format!("{value}");
    }
    if let Some(value) = result.as_ref().downcast_ref::<String>() {
        return format!("{value}");
    }
    if let Some(value) = result.as_ref().downcast_ref::<bool>() {
        return format!("{value}");
    } else {
        return format!("{result:?}");
    }
}

pub fn interpret(
    stmts: LinkedList<Box<Stmt>>,
    env: Rc<RefCell<Environment>>,
) -> Result<(), &'static str> {
    for stmt in stmts {
        match *stmt {
            Stmt::Expression { expression } => match evaluate(*expression, env.clone()) {
                None => return Err("Runtime Error"),
                _ => {}
            },
            Stmt::Print { expression } => {
                if let Some(value) = evaluate(*expression, env.clone()) {
                    println!("{}", result_to_string(value));
                } else {
                    return Err("Runtime Error");
                }
            }
            Stmt::Var { name, initializer } => {
                if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                    match initializer {
                        None => env.borrow_mut().define(key.clone(), Box::new(0)),
                        Some(val) => {
                            let result = evaluate(*val, env.clone());
                            match result {
                                Some(val) => env.borrow_mut().define(key.clone(), val),

                                None => return Err("Runtime Error"),
                            }
                        }
                    }
                }
            }
            Stmt::Block { statements } => {
                let new_env = Rc::new(RefCell::new(Environment::from(env.clone())));
                match interpret(statements, new_env.clone()) {
                    Ok(()) => {}
                    Err(s) => return Err(s),
                }
            }
        }
    }
    return Ok(());
}

pub fn evaluate(expr: Expr, env: Rc<RefCell<Environment>>) -> Option<Box<dyn Any>> {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => return binary_eval(*left, operator, *right, env),
        Expr::Grouping { expression } => {
            return evaluate(*expression, env);
        }
        Expr::Literal { value } => return Some(value),
        Expr::Unary { operator, right } => return unitary_eval(operator, *right, env),
        Expr::Variable { name } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                return match env.borrow_mut().get(key) {
                    None => {
                        eprintln!("Undefined Variable.");
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
                let val: Box<dyn Any>;
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

fn unitary_eval(token: Token, expr: Expr, env: Rc<RefCell<Environment>>) -> Option<Box<dyn Any>> {
    let right;
    match evaluate(expr, env.clone()) {
        Some(x) => right = x,
        None => return None,
    }

    match token.ttype {
        TokenType::MINUS => match right.downcast::<f64>() {
            Ok(x) => return Some(Box::new(-*x)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::BANG => {
            match right.as_ref().downcast_ref::<bool>() {
                Some(x) => return Some(Box::new(!*x)),
                _ => {}
            }

            match right.as_ref().downcast_ref::<Option<bool>>() {
                Some(x) => match *x {
                    None => return Some(Box::new(true)),
                    Some(_x) => return Some(Box::new(false)),
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
) -> Option<Box<dyn Any>> {
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
                (Ok(x), Ok(y)) => Some(Box::new(*x - *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::SLASH => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => divide(*x, *y).map(|b| b as Box<dyn Any>),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::STAR => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Box::new(*x * *y)),
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
                (Some(x), Some(y)) => return Some(Box::new(*x + *y)),
                _ => {}
            }

            match (
                left.as_ref().downcast_ref::<String>(),
                right.as_ref().downcast_ref::<String>(),
            ) {
                (Some(x), Some(y)) => return Some(Box::new(x.clone() + &*y)),
                _ => {}
            }
            error_type_mismatch();
            return None;
        }

        TokenType::GREATER => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Box::new(x > y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }

        TokenType::GREATER_EQUAL => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Box::new(*x >= *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }

        TokenType::LESS => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Box::new(*x < *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::LESS_EQUAL => {
            return match (left.downcast::<f64>(), right.downcast::<f64>()) {
                (Ok(x), Ok(y)) => Some(Box::new(*x <= *y)),
                _ => {
                    error_type_mismatch();
                    None
                }
            }
        }
        TokenType::BANG_EQUAL => match is_equal(left, right) {
            Some(x) => return Some(Box::new(!*x)),
            None => return None,
        },
        TokenType::EQUAL_EQUAL => {
            return is_equal(left, right).map(|b| b as Box<dyn Any>);
        }
        _ => {
            eprintln!("Wrong operator.");
            return None;
        }
    }
}

fn divide(numerator: f64, denominator: f64) -> Option<Box<f64>> {
    if denominator == 0.0 {
        eprintln!("Divide by 0.");
        None
    } else {
        Some(Box::new(numerator / denominator))
    }
}

fn is_equal(l1: Box<dyn Any>, l2: Box<dyn Any>) -> Option<Box<bool>> {
    match (
        l1.as_ref().downcast_ref::<f64>(),
        l2.as_ref().downcast_ref::<f64>(),
    ) {
        (Some(x), Some(y)) => return Some(Box::new(*x == *y)),
        _ => {}
    }

    match (
        l1.as_ref().downcast_ref::<bool>(),
        l2.as_ref().downcast_ref::<bool>(),
    ) {
        (Some(x), Some(y)) => return Some(Box::new(*x == *y)),
        _ => {}
    }

    match (
        l1.as_ref().downcast_ref::<String>(),
        l2.as_ref().downcast_ref::<String>(),
    ) {
        (Some(x), Some(y)) => return Some(Box::new(*x == *y)),
        _ => {}
    }

    error_type_mismatch();
    return None;
}

fn error_type_mismatch() {
    eprintln!("Type mismatch.");
}
