use crate::token::Token;
use std::any::Any;
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Box<dyn Any>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

fn any_to_string(value: &dyn Any) -> String {
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
            Expr::Grouping { expression } => write!(f, "({})", expression),
            Expr::Literal { value } => write!(f, "{}", any_to_string(value.deref())), // Don't know why but it works.
            Expr::Unary { operator, right } => write!(f, "({:?} {})", operator, right),
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
