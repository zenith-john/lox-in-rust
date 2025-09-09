use crate::token::{BasicType, Token};
use std::collections::LinkedList;
use std::fmt;

#[derive(Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: LinkedList<Box<Expr>>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: BasicType,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
        id: u64,
    },
    This {
        keyword: Token,
        id: u64,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
        id: u64,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
        id: u64,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator, left, right),
            Expr::Call {
                callee,
                paren,
                arguments: _,
            } => write!(f, "{} {}", callee, paren.lexeme.clone().unwrap()),
            Expr::Get { object, name } => write!(f, "{}.{}", object, name.lexeme.clone().unwrap()),
            Expr::Grouping { expression } => write!(f, "({})", expression),
            Expr::Literal { value } => write!(f, "{}", value), // Don't know why but it works.
            Expr::Logical {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator, left, right),
            Expr::Set {
                object,
                name,
                value,
            } => write!(f, "{}.{} = {}", object, name, value),
            Expr::Super {
                keyword: _,
                method,
                id,
            } => write!(f, "super {} {}", method.lexeme.clone().unwrap(), id),
            Expr::This { keyword: _, id } => write!(f, "this {}", id),
            Expr::Unary { operator, right } => write!(f, "({} {})", operator, right),
            Expr::Variable { name, id } => write!(f, "{} {}", name.lexeme.clone().unwrap(), id),
            Expr::Assign { name, value, id } => {
                write!(f, "({} {} = {})", name.lexeme.clone().unwrap(), value, id)
            }
        }
    }
}

impl Expr {
    pub fn line_number(&self) -> i32 {
        match &self {
            Expr::Binary {
                left: _,
                operator,
                right: _,
            } => operator.line,
            Expr::Call {
                callee: _,
                paren,
                arguments: _,
            } => paren.line,
            Expr::Get { object: _, name } => name.line,
            Expr::Grouping { expression: _ } => -1,
            Expr::Literal { value: _ } => -1,
            Expr::Logical {
                left: _,
                operator,
                right: _,
            } => operator.line,
            Expr::Set {
                object: _,
                name,
                value: _,
            } => name.line,
            Expr::Super {
                keyword,
                method: _,
                id: _,
            } => keyword.line,
            Expr::This { keyword, id: _ } => keyword.line,
            Expr::Unary { operator, right: _ } => operator.line,
            Expr::Variable { name, id: _ } => name.line,
            Expr::Assign {
                name,
                value: _,
                id: _,
            } => name.line,
        }
    }
}
