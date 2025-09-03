use crate::callable::{Callable, LoxClass, LoxFunction};
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
) -> Result<(), &'static str> {
    for stmt in stmts {
        execute(*stmt, env.clone(), table)?
    }
    Ok(())
}

pub fn execute(
    stmt: Stmt,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Result<(), &'static str> {
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
                    .clone()
                    .expect("Non empty")
                    .as_class()
                {
                    sp = Some(val.clone());
                    local_env = Rc::new(RefCell::new(Environment::from(env.clone())));
                    local_env
                        .borrow_mut()
                        .define("super".to_string(), BasicType::Class(val));
                } else {
                    eprintln!("{} is not a class name.", expr);
                    return Err("Runtime Error");
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
                        .ok_or("Not a correct identifier")?
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
                .ok_or("Not a correct identifier")?
                .clone();
            env.borrow_mut().define(st, klass);
            Ok(())
        }
        Stmt::Expression { expression } => match evaluate(*expression, env.clone(), table) {
            None => Err("Runtime Error"),
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
                .ok_or("Not a correct identifier")?
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
            match evaluate(*condition, env.clone(), table) {
                None => return Err("Runtime Error"),
                Some(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return Err("Runtime Error");
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
        Stmt::Print { expression } => {
            if let Some(value) = evaluate(*expression, env.clone(), table) {
                println!("{}", value);
            } else {
                return Err("Runtime Error");
            }
            Ok(())
        }
        Stmt::Return {
            keyword: _,
            value: _,
        } => {
            Err("Invalid return expression.") // Should not handle it here.
        }
        Stmt::Var { name, initializer } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                if env.borrow().is_defined(key.to_string()) {
                    eprintln!("Multiple definition of some variable {}.", key);
                    return Err("Runtime Error");
                }
                match initializer {
                    None => env.borrow_mut().define(key.clone(), BasicType::Number(0.0)),
                    Some(val) => {
                        let result = evaluate(*val, env.clone(), table);
                        match result {
                            Some(val) => env.borrow_mut().define(key.clone(), val),
                            None => return Err("Runtime Error"),
                        }
                    }
                };
                Ok(())
            } else {
                Err("Invalid Variable Name")
            }
        }
        Stmt::While { condition, body } => {
            let mut is_true: bool;
            match evaluate(*condition.clone(), env.clone(), table) {
                None => return Err("Runtime Error"),
                Some(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return Err("Runtime Error");
                    }
                }
            }
            while is_true {
                execute(*body.clone(), env.clone(), table)?;
                match evaluate(*condition.clone(), env.clone(), table) {
                    None => return Err("Runtime Error"),
                    Some(val) => {
                        if let Some(value) = val.as_bool() {
                            is_true = value;
                        } else {
                            eprintln!("Statement in condition is not of bool type.");
                            return Err("Runtime Error");
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
) -> Option<BasicType> {
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
                    None => return None,
                    Some(val) => args.push_back(val),
                }
            }
            if let BasicType::Function(val) = callee_evaluated {
                val.call(&mut args)
            } else if let BasicType::Class(val) = callee_evaluated {
                val.call(&mut args)
            } else {
                eprintln!("Callee {} is not a function.", callee_evaluated);
                None
            }
        }
        Expr::Get { object, name } => {
            let ob = evaluate(*object, env, table)?;
            if let BasicType::Instance(val) = ob.clone() {
                let st = name.lexeme.unwrap().as_string().unwrap();
                if val.borrow_mut().fields.contains_key(&st) {
                    return val.borrow_mut().fields.get(&st).cloned();
                }
                let mut klass = val.borrow_mut().clone().klass.clone();
                loop {
                    if let Some(method) = klass.find_method(st.clone()) {
                        return Some(BasicType::Function(Rc::new(method.bind(val))));
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
                None
            }
        }
        Expr::Grouping { expression } => evaluate(*expression, env, table),
        Expr::Literal { value } => Some(value),
        Expr::Logical {
            left,
            operator,
            right,
        } => {
            let is_true: bool;
            match evaluate(*left, env.clone(), table) {
                None => return None,
                Some(val) => {
                    if let Some(value) = val.as_bool() {
                        is_true = value;
                    } else {
                        eprintln!("Statement in condition is not of bool type.");
                        return None;
                    }
                }
            }
            if operator.ttype == TokenType::Or {
                if is_true {
                    return Some(BasicType::Bool(is_true));
                }
            } else if !is_true {
                return Some(BasicType::Bool(is_true));
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
                Some(v)
            } else {
                eprintln!("Invalid property call.");
                None
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
                Some(val) => val.as_class().expect("Lox Class"),
            };
            let object = match env.borrow_mut().get(&"this".to_string(), *depth - 1) {
                None => {
                    eprintln!(
                        "Line {}: Don't know what \"this\" refered to.",
                        keyword.line
                    );
                    return None;
                }
                Some(val) => val.as_instance().expect("Lox Instance"),
            };
            let st = method.lexeme.unwrap().as_string().unwrap();
            let mut klass = superclass.clone();
            loop {
                if let Some(method) = klass.find_method(st.clone()) {
                    return Some(BasicType::Function(Rc::new(method.bind(object))));
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
                    None
                }
                Some(val) => Some(val),
            }
        }
        Expr::Unary { operator, right } => unitary_eval(operator, *right, env, table),
        Expr::Variable { name, id } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                let depth = table.get(&id)?;
                return match env.borrow_mut().get(&key, *depth) {
                    None => {
                        eprintln!("Undefined Variable {}.", key);
                        return None;
                    }
                    Some(val) => Some(val),
                };
            } else {
                eprintln!("Invalid identifier.");
                None
            }
        }
        Expr::Assign { name, value, id } => {
            if let Some(key) = name.lexeme.unwrap().as_string() {
                let depth = table.get(&id)?;
                let val: BasicType = evaluate(*value, env.clone(), table)?;
                return env.borrow_mut().assign(key.clone(), val, *depth);
            } else {
                eprintln!("Invalid identifier.");
                None
            }
        }
    }
}

fn unitary_eval(
    token: Token,
    expr: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Option<BasicType> {
    let right = evaluate(expr, env.clone(), table)?;

    match token.ttype {
        TokenType::Minus => match right.as_number() {
            Some(x) => Some(BasicType::Number(-x)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::Bang => {
            if let Some(x) = right.as_bool() {
                Some(BasicType::Bool(!x))
            } else {
                error_type_mismatch();
                None
            }
        }
        _ => {
            eprintln!("Wrong operator.");
            None
        }
    }
}

fn binary_eval(
    expr1: Expr,
    token: Token,
    expr2: Expr,
    env: Rc<RefCell<Environment>>,
    table: &HashMap<u64, i32>,
) -> Option<BasicType> {
    let left = evaluate(expr1, env.clone(), table)?;
    let right = evaluate(expr2, env.clone(), table)?;

    match token.ttype {
        TokenType::Minus => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Number(x - y)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::Slash => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => divide(x, y).map(BasicType::Number),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::Star => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Number(x * y)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::Plus => {
            if let (Some(x), Some(y)) = (left.as_number(), right.as_number()) {
                return Some(BasicType::Number(x + y));
            }

            if let (Some(x), Some(y)) = (left.as_string(), right.as_string()) {
                return Some(BasicType::String(x.clone() + &*y));
            }
            error_type_mismatch();
            None
        }

        TokenType::Greater => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Bool(x > y)),
            _ => {
                error_type_mismatch();
                None
            }
        },

        TokenType::GreaterEqual => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Bool(x >= y)),
            _ => {
                error_type_mismatch();
                None
            }
        },

        TokenType::Less => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Bool(x < y)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::LessEqual => match (left.as_number(), right.as_number()) {
            (Some(x), Some(y)) => Some(BasicType::Bool(x <= y)),
            _ => {
                error_type_mismatch();
                None
            }
        },
        TokenType::BangEqual => Some(BasicType::Bool(!(left == right))),
        TokenType::EqualEqual => Some(BasicType::Bool(left == right)),
        _ => {
            eprintln!("Wrong operator.");
            None
        }
    }
}

fn divide(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 {
        eprintln!("Divide by 0.");
        None
    } else {
        Some(numerator / denominator)
    }
}

fn error_type_mismatch() {
    eprintln!("Type mismatch.");
}
