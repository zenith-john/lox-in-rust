use crate::callable::{Callable, LoxClass, LoxFunction, LoxInstance};
use crate::expr::Expr;
use crate::stmt::{Environment, Stmt};
use crate::token::{Token, TokenType};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
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
    table: &HashMap<u64, i32>,
) -> Result<(), &'static str> {
    for stmt in stmts {
        match execute(stmt, env.clone(), table) {
            Err(s) => return Err(s),
            Ok(()) => {}
        }
    }
    return Ok(());
}

pub fn execute(
    stmt: Box<Stmt>,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<(), &'static str> {
    match *stmt {
        Stmt::Block { statements } => {
            let new_env = Rc::new(RefCell::new(Environment::from(env.clone())));
            match interpret(statements, new_env.clone(), table) {
                Ok(()) => {}
                Err(s) => return Err(s),
            }
            return Ok(());
        }
        Stmt::Class {
            name,
            superclass,
            methods,
        } => {
            let mut sp: Option<Rc<LoxClass>> = None;
            let mut local_env = env.clone();
            if let Some(expr) = superclass {
                if let Ok(val) = evaluate(*expr.clone(), env.clone(), table)
                    .clone()
                    .expect("Non empty")
                    .downcast::<LoxClass>()
                {
                    sp = Some(val.clone());
                    local_env = Rc::new(RefCell::new(Environment::from(env.clone())));
                    local_env.borrow_mut().define("super".to_string(), val);
                } else {
                    eprintln!("{} is not a class name.", expr);
                    return Err("Runtime Error");
                }
            }
            local_env = Rc::new(RefCell::new(Environment::from(local_env.clone())));

            let mut kmethods: HashMap<String, LoxFunction> = HashMap::new();
            for method in methods {
                match *method {
                    Stmt::Function {
                        name: new_name,
                        params,
                        body,
                    } => {
                        let st = new_name
                            .lexeme
                            .clone()
                            .unwrap()
                            .as_ref()
                            .downcast_ref::<String>()
                            .ok_or("Not a correct identifier")?
                            .clone();
                        kmethods.insert(
                            st,
                            LoxFunction::new(
                                new_name,
                                params,
                                body,
                                local_env.clone(),
                                table.clone(),
                            ),
                        );
                    }
                    _ => {}
                }
            }
            let klass = Rc::new(LoxClass::new(name.clone(), sp, kmethods));
            let st = name
                .lexeme
                .unwrap()
                .as_ref()
                .downcast_ref::<String>()
                .ok_or("Not a correct identifier")?
                .clone();
            env.borrow_mut().define(st, klass);
            return Ok(());
        }
        Stmt::Expression { expression } => match evaluate(*expression, env.clone(), table) {
            None => return Err("Runtime Error"),
            _ => return Ok(()),
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
            match evaluate(*condition, env.clone(), table) {
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
                return execute(then_branch, env.clone(), table);
            } else if let Some(branch) = else_branch {
                return execute(branch, env.clone(), table);
            }
            return Ok(());
        }
        Stmt::Print { expression } => {
            if let Some(value) = evaluate(*expression, env.clone(), table) {
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
            return Err("Invalid return expression."); // Should not handle it here.
        }
        Stmt::Var { name, initializer } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                if env.borrow().is_defined(key.to_string()) {
                    eprintln!("Multiple definition of some variable {}.", key);
                    return Err("Runtime Error");
                }
                match initializer {
                    None => env.borrow_mut().define(key.clone(), Rc::new(0)),
                    Some(val) => {
                        let result = evaluate(*val, env.clone(), table);
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
            match evaluate(*condition.clone(), env.clone(), table) {
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
                match execute(body.clone(), env.clone(), table) {
                    Err(str) => return Err(str),
                    Ok(()) => {}
                }
                match evaluate(*condition.clone(), env.clone(), table) {
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
    }
}

pub fn evaluate(
    expr: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Option<Rc<dyn Any>> {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => return binary_eval(*left, operator, *right, env, table),
        Expr::Call {
            callee,
            paren: _,
            arguments,
        } => {
            let callee_evaluated = match evaluate(*callee, env.clone(), table) {
                None => return None,
                Some(val) => val,
            };
            let mut args: LinkedList<Rc<dyn Any>> = LinkedList::new();
            for expr in arguments {
                match evaluate(*expr, env.clone(), table) {
                    None => return None,
                    Some(val) => args.push_back(val),
                }
            }
            if let Ok(val) = callee_evaluated.clone().downcast::<LoxFunction>() {
                return val.call(&mut args);
            } else if let Ok(val) = callee_evaluated.clone().downcast::<LoxClass>() {
                return val.call(&mut args);
            } else {
                eprintln!(
                    "Callee {} is not a function.",
                    rc_to_string(callee_evaluated)
                );
                return None;
            }
        }
        Expr::Get { object, name } => {
            let ob = evaluate(*object, env, table)?;
            if let Ok(val) = ob.clone().downcast::<RefCell<LoxInstance>>() {
                let st = name
                    .lexeme
                    .unwrap()
                    .as_ref()
                    .downcast_ref::<String>()?
                    .to_string();
                if val.borrow_mut().fields.contains_key(&st) {
                    return val.borrow_mut().fields.get(&st).cloned();
                }
                let mut klass = val.clone().borrow_mut().klass.clone();
                loop {
                    if let Some(method) = klass.find_method(st.clone()) {
                        return Some(Rc::new(method.bind(val)));
                    }
                    match klass.superclass() {
                        None => {
                            eprintln!("Undefined property.");
                            return None;
                        }
                        Some(val) => klass = val,
                    }
                }
            } else {
                eprintln!("Invalid property call.");
                return None;
            }
        }
        Expr::Grouping { expression } => {
            return evaluate(*expression, env, table);
        }
        Expr::Literal { value } => return Some(value),
        Expr::Logical {
            left,
            operator,
            right,
        } => {
            let is_true: bool;
            match evaluate(*left, env.clone(), table) {
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

            return evaluate(*right, env.clone(), table);
        }
        Expr::Set {
            object,
            name,
            value,
        } => {
            let ob = evaluate(*object, env.clone(), table)?;
            if let Ok(val) = ob.clone().downcast::<RefCell<LoxInstance>>() {
                let v = evaluate(*value, env.clone(), table)?;
                val.borrow_mut().set(name, v.clone());
                return Some(v);
            } else {
                eprintln!("Invalid property call.");
                return None;
            }
        }
        Expr::Super {
            keyword,
            method,
            id,
        } => {
            let depth = table.get(&id)?;
            let superclass = match env.borrow_mut().get(&"super".to_string(), *depth) {
                None => {
                    eprintln!(
                        "Line {}: Don't know what \"super\" refered to.",
                        keyword.line
                    );
                    return None;
                }
                Some(val) => val.clone().downcast::<LoxClass>().expect("Lox Class"),
            };
            let object = match env.borrow_mut().get(&"this".to_string(), *depth - 1) {
                None => {
                    eprintln!(
                        "Line {}: Don't know what \"this\" refered to.",
                        keyword.line
                    );
                    return None;
                }
                Some(val) => val
                    .clone()
                    .downcast::<RefCell<LoxInstance>>()
                    .expect("Lox Instance"),
            };
            let st = method
                .lexeme
                .unwrap()
                .as_ref()
                .downcast_ref::<String>()?
                .to_string();
            let mut klass = superclass.clone();
            loop {
                if let Some(method) = klass.find_method(st.clone()) {
                    return Some(Rc::new(method.bind(object)));
                }
                match klass.superclass() {
                    None => {
                        eprintln!("Line {}: undefined property.", keyword.line);
                        return None;
                    }
                    Some(val) => klass = val,
                }
            }
        }
        Expr::This { keyword, id } => {
            let depth = table.get(&id)?;
            match env.borrow_mut().get(&"this".to_string(), *depth) {
                None => {
                    eprintln!(
                        "Line {}: Don't know what \"this\" refered to.",
                        keyword.line
                    );
                    return None;
                }
                Some(val) => return Some(val),
            }
        }
        Expr::Unary { operator, right } => return unitary_eval(operator, *right, env, table),
        Expr::Variable { name, id } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                let depth = table.get(&id)?;
                return match env.borrow_mut().get(key, *depth) {
                    None => {
                        eprintln!("Undefined Variable {}.", key);
                        return None;
                    }
                    Some(val) => Some(val),
                };
            } else {
                eprintln!("Invalid identifier.");
                return None;
            }
        }
        Expr::Assign { name, value, id } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                let val: Rc<dyn Any>;
                let depth = table.get(&id)?;
                match evaluate(*value, env.clone(), table) {
                    None => return None,
                    Some(x) => val = x,
                }
                return env.borrow_mut().assign(key.clone(), val, *depth);
            } else {
                eprintln!("Invalid identifier.");
                return None;
            }
        }
    }
}

fn unitary_eval(
    token: Token,
    expr: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Option<Rc<dyn Any>> {
    let right;
    match evaluate(expr, env.clone(), table) {
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
    table: &HashMap<u64, i32>,
) -> Option<Rc<dyn Any>> {
    let left;
    let right;
    match evaluate(expr1, env.clone(), table) {
        Some(x) => left = x,
        None => return None,
    }
    match evaluate(expr2, env.clone(), table) {
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
