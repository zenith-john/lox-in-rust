use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;
use std::collections::{HashMap, LinkedList};

pub fn resolve(
    statements: LinkedList<Box<Stmt>>,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) {
    for stmt in statements {
        resolve_stmt(stmt, scopes, table);
    }
}
fn resolve_stmt(
    stmt: Box<Stmt>,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) {
    match *stmt {
        Stmt::Expression { expression } => {
            resolve_expr(expression, scopes, table);
            return;
        }
        Stmt::Function { name, params, body } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                declare(key.to_string(), scopes);
                define(key.to_string(), scopes);
                resolve_function(params, body, scopes, table);
                return;
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            resolve_expr(condition, scopes, table);
            resolve_stmt(then_branch, scopes, table);
            if let Some(stmt) = else_branch {
                resolve_stmt(stmt, scopes, table);
                return;
            }
        }
        Stmt::Print { expression } => {
            resolve_expr(expression, scopes, table);
            return;
        }
        Stmt::Return { keyword: _, value } => {
            if let Some(expr) = value {
                resolve_expr(expr, scopes, table);
            }
            return;
        }
        Stmt::Var { name, initializer } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                declare(key.to_string(), scopes);
                if let Some(expr) = initializer {
                    resolve_expr(expr, scopes, table);
                }
                define(key.to_string(), scopes);
            } else {
                eprintln!("Invalid identifier.");
            }
            return;
        }
        Stmt::While { condition, body } => {
            resolve_expr(condition, scopes, table);
            resolve_stmt(body, scopes, table);
            return;
        }
        Stmt::Block { statements } => {
            begin_scope(scopes);
            for stmt in statements {
                resolve_stmt(stmt, scopes, table);
            }
            end_scope(scopes);
            return;
        }
    }
}

fn resolve_expr(
    expr: Box<Expr>,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) {
    match *expr.clone() {
        Expr::Binary {
            left,
            operator: _,
            right,
        } => {
            resolve_expr(left, scopes, table);
            resolve_expr(right, scopes, table);
            return;
        }
        Expr::Call {
            callee,
            paren: _,
            arguments,
        } => {
            resolve_expr(callee, scopes, table);
            for arg in arguments {
                resolve_expr(arg, scopes, table);
            }
            return;
        }
        Expr::Grouping { expression } => {
            resolve_expr(expression, scopes, table);
            return;
        }
        Expr::Literal { .. } => {
            return;
        }
        Expr::Logical {
            left,
            operator: _,
            right,
        } => {
            resolve_expr(left, scopes, table);
            resolve_expr(right, scopes, table);
            return;
        }
        Expr::Unary { operator: _, right } => {
            resolve_expr(right, scopes, table);
            return;
        }
        Expr::Variable { name, id } => {
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                if !scopes.is_empty()
                    && scopes.front_mut().expect("Non empty").get(key) == Some(&false)
                {
                    eprintln!("Can't read local variable in its own initializer.")
                }
                resolve_local(id, key, scopes, table);
            }
            return;
        }
        Expr::Assign { name, value, id } => {
            resolve_expr(value, scopes, table);
            if let Some(key) = name.lexeme.unwrap().as_ref().downcast_ref::<String>() {
                resolve_local(id, key, scopes, table);
            }
            return;
        }
    }
}

fn begin_scope(scopes: &mut LinkedList<HashMap<String, bool>>) {
    scopes.push_front(HashMap::<String, bool>::new());
}

fn end_scope(scopes: &mut LinkedList<HashMap<String, bool>>) {
    scopes.pop_front();
}

fn declare(var: String, scopes: &mut LinkedList<HashMap<String, bool>>) {
    if scopes.is_empty() {
        return;
    }
    if let Some(scope) = scopes.front_mut() {
        scope.insert(var, false);
    }
}

fn define(var: String, scopes: &mut LinkedList<HashMap<String, bool>>) {
    if scopes.is_empty() {
        return;
    }
    if let Some(scope) = scopes.front_mut() {
        scope.insert(var, true);
    }
}

fn resolve_local(
    id: u64,
    var: &String,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) {
    let mut i: i32 = 0;
    for scope in scopes {
        if scope.contains_key(var) {
            table.insert(id, i);
            return;
        }
        i = i + 1;
    }
}

fn resolve_function(
    params: LinkedList<Token>,
    body: LinkedList<Box<Stmt>>,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) {
    begin_scope(scopes);
    for token in params {
        if let Some(key) = token.lexeme.unwrap().as_ref().downcast_ref::<String>() {
            declare(key.to_string(), scopes);
            define(key.to_string(), scopes);
        }
    }
    for stmt in body {
        resolve_stmt(stmt, scopes, table);
    }
    end_scope(scopes);
}
