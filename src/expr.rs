use crate::token::Token;
use std::any::Any;
use std::collections::LinkedList;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug)]
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
        value: Rc<dyn Any>,
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

pub fn any_to_string(value: &dyn Any) -> String {
    if let Some(val) = value.downcast_ref::<f64>() {
        format!("{}", val)
    } else if let Some(val) = value.downcast_ref::<String>() {
        format!("{}", val)
    } else {
        format!("{value:?}")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({:?} {} {})", operator, left, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => write!(f, "{} {:?} {:?}", callee, paren.lexeme, arguments),
            Expr::Get { object, name } => write!(f, "{}.{:?}", object, name.lexeme),
            Expr::Grouping { expression } => write!(f, "({})", expression),
            Expr::Literal { value } => write!(f, "{}", any_to_string(value.deref())), // Don't know why but it works.
            Expr::Logical {
                left,
                operator,
                right,
            } => write!(f, "({:?} {} {})", operator, left, right),
            Expr::Set {
                object,
                name,
                value,
            } => write!(f, "{}.{:?} = {}", object, name, value),
            Expr::This { keyword: _, id } => write!(f, "this {}", id),
            Expr::Unary { operator, right } => write!(f, "({:?} {})", operator, right),
            Expr::Variable { name, id } => write!(f, "{:?} {}", name.lexeme, id),
            Expr::Assign { name, value, id } => write!(f, "({:?} {} = {})", name.lexeme, value, id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_to_string() {
        let x = Box::new(12.1);
        assert!(String::from("12.1").eq(&any_to_string(Box::<_>::leak(x))));
    }

    #[test]
    fn print_expr() {
        let test_expr: Expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Box::new(12.1),
            }),
            operator: Token {
                ttype: TokenType::BANG_EQUAL,
                lexeme: None,
                line: 0,
            },
            right: Box::new(Expr::Unary {
                operator: Token {
                    ttype: TokenType::MINUS,
                    lexeme: None,
                    line: 0,
                },
                right: Box::new(Expr::Grouping {
                    expression: Box::new(Expr::Literal {
                        value: Box::new("12.1".to_string()),
                    }),
                }),
            }),
        };
        println!("{}", test_expr);
    }
}
