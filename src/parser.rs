use crate::expr::Expr;
use crate::token::{Token, TokenType};
use std::collections::LinkedList;

pub fn parser(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if let Some(expr) = expression(tokens) {
        if !match_head(tokens, &[TokenType::EOF]) {
            eprintln!("Remaining symbols.");
            return None;
        } else {
            return Some(expr);
        }
    } else {
        return None;
    }
}

fn match_head(tokens: &LinkedList<Token>, slice: &[TokenType]) -> bool {
    let head = &tokens.front().unwrap().ttype;
    // if *head == TokenType::EOF {
    //     return false;
    // }
    for t in slice.iter() {
        if *head == *t {
            return true;
        }
    }
    return false;
}
fn expression(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = comparison(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
        let operator = tokens.pop_front().unwrap();
        let right = comparison(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator: operator,
                    right: x,
                })
            }
            None => return None,
        }
    }

    return Some(expr);
}

fn comparison(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = term(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(
        tokens,
        &[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ],
    ) {
        let operator = tokens.pop_front().unwrap();
        let right = term(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator: operator,
                    right: x,
                })
            }
            None => return None,
        }
    }
    return Some(expr);
}

fn term(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = factor(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::PLUS, TokenType::MINUS]) {
        let operator = tokens.pop_front().unwrap();
        let right = factor(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator: operator,
                    right: x,
                })
            }
            None => return None,
        }
    }

    return Some(expr);
}

fn factor(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = unary(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::SLASH, TokenType::STAR]) {
        let operator = tokens.pop_front().unwrap();
        let right = unary(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator: operator,
                    right: x,
                })
            }
            None => return None,
        }
    }
    return Some(expr);
}

fn unary(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if match_head(tokens, &[TokenType::BANG, TokenType::MINUS]) {
        let operator = tokens.pop_front().unwrap();
        let right = unary(tokens);
        match right {
            Some(x) => {
                return Some(Box::new(Expr::Unary {
                    operator: operator,
                    right: x,
                }))
            }
            None => return None,
        };
    }
    return primary(tokens);
}

fn primary(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if match_head(tokens, &[TokenType::FALSE]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Box::new(false),
        }));
    }
    if match_head(tokens, &[TokenType::TRUE]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Box::new(true),
        }));
    }
    if match_head(tokens, &[TokenType::NIL]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Box::new(Option::<bool>::None),
        }));
    }
    if match_head(tokens, &[TokenType::NUMBER, TokenType::STRING]) {
        let token = tokens.pop_front().unwrap();
        return Some(Box::new(Expr::Literal {
            value: token.lexeme.unwrap(),
        }));
    }
    if match_head(tokens, &[TokenType::LEFT_PAREN]) {
        tokens.pop_front();
        let opt = expression(tokens);
        let expr: Box<Expr>;
        match opt {
            Some(x) => expr = x,
            None => return None,
        }
        if !match_head(tokens, &[TokenType::RIGHT_PAREN]) {
            error(
                tokens.front().unwrap(),
                "Expect ')' after expression.".to_string(),
            );
            return None;
        }
        tokens.pop_front();
        return Some(Box::new(Expr::Grouping { expression: expr }));
    }
    error(tokens.front().unwrap(), "No matching.".to_string());
    return None;
}

fn error(token: &Token, message: String) {
    if token.ttype == TokenType::EOF {
        println!("Line {} unsolved at end. {}", token.line, message);
    } else {
        println!("{} at '{:?}' {}", token.line, token, message);
    }
}

fn synchronize(tokens: &mut LinkedList<Token>) {
    while match_head(tokens, &[TokenType::EOF]) {
        match tokens.front().unwrap().ttype {
            TokenType::SEMICOLON => return,
            TokenType::CLASS => return,
            TokenType::FUN => return,
            TokenType::VAR => return,
            TokenType::FOR => return,
            TokenType::IF => return,
            TokenType::WHILE => return,
            TokenType::PRINT => return,
            TokenType::RETURN => return,
            _ => {
                tokens.pop_front();
                return;
            }
        }
    }
}
