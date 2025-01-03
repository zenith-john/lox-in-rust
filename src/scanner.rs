use crate::error;
use crate::token::{Token, TokenType};
use lazy_static::lazy_static;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

lazy_static! {
    static ref keywords: HashMap<String, TokenType> = HashMap::from([
        ("and".to_string(), TokenType::AND),
        ("class".to_string(), TokenType::CLASS),
        ("else".to_string(), TokenType::ELSE),
        ("false".to_string(), TokenType::FALSE),
        ("for".to_string(), TokenType::FOR),
        ("fun".to_string(), TokenType::FUN),
        ("if".to_string(), TokenType::IF),
        ("nil".to_string(), TokenType::NIL),
        ("or".to_string(), TokenType::OR),
        ("print".to_string(), TokenType::PRINT),
        ("return".to_string(), TokenType::RETURN),
        ("super".to_string(), TokenType::SUPER),
        ("this".to_string(), TokenType::THIS),
        ("true".to_string(), TokenType::TRUE),
        ("var".to_string(), TokenType::VAR),
        ("while".to_string(), TokenType::WHILE),
    ]);
}

pub fn scan_tokens(string: &String, line: &mut i32) -> Option<LinkedList<Token>> {
    let mut start: usize;
    let mut current: usize = 0;
    let mut tokens: LinkedList<Token> = LinkedList::new();
    let mut token: Option<Token>;
    while current < string.len() {
        start = current;
        (token, current) = scan_token(&string, start, line);
        match token {
            Some(tok) => tokens.push_back(tok),
            None => {
                if current == 0 {
                    return None;
                }
            }
        }
    }
    tokens.push_back(Token {
        ttype: TokenType::EOF,
        lexeme: None,
        line: *line,
    });
    return Some(tokens);
}

fn scan_token(string: &String, pos: usize, line: &mut i32) -> (Option<Token>, usize) {
    let c: char = string.chars().nth(pos).expect("End of string.");
    let mut end: usize = pos;
    let mut token: Option<Token> = None;
    match c {
        '(' => {
            token = Some(Token {
                ttype: TokenType::LEFT_PAREN,
                lexeme: None,
                line: *line,
            })
        }
        ')' => {
            token = Some(Token {
                ttype: TokenType::RIGHT_PAREN,
                lexeme: None,
                line: *line,
            })
        }
        '{' => {
            token = Some(Token {
                ttype: TokenType::LEFT_BRACE,
                lexeme: None,
                line: *line,
            })
        }
        '}' => {
            token = Some(Token {
                ttype: TokenType::RIGHT_BRACE,
                lexeme: None,
                line: *line,
            })
        }
        ',' => {
            token = Some(Token {
                ttype: TokenType::COMMA,
                lexeme: None,
                line: *line,
            })
        }
        '.' => {
            token = Some(Token {
                ttype: TokenType::DOT,
                lexeme: None,
                line: *line,
            })
        }
        '-' => {
            token = Some(Token {
                ttype: TokenType::MINUS,
                lexeme: None,
                line: *line,
            })
        }
        '+' => {
            token = Some(Token {
                ttype: TokenType::PLUS,
                lexeme: None,
                line: *line,
            })
        }
        ';' => {
            token = Some(Token {
                ttype: TokenType::SEMICOLON,
                lexeme: None,
                line: *line,
            })
        }
        '*' => {
            token = Some(Token {
                ttype: TokenType::STAR,
                lexeme: None,
                line: *line,
            })
        }
        '!' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                token = Some(Token {
                    ttype: TokenType::BANG_EQUAL,
                    lexeme: None,
                    line: *line,
                });
                end = pos + 1
            } else {
                token = Some(Token {
                    ttype: TokenType::BANG,
                    lexeme: None,
                    line: *line,
                });
            }
        }
        '=' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                token = Some(Token {
                    ttype: TokenType::EQUAL_EQUAL,
                    lexeme: None,
                    line: *line,
                });
                end = pos + 1
            } else {
                token = Some(Token {
                    ttype: TokenType::EQUAL,
                    lexeme: None,
                    line: *line,
                });
            }
        }
        '<' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                token = Some(Token {
                    ttype: TokenType::LESS_EQUAL,
                    lexeme: None,
                    line: *line,
                });
                end = pos + 1
            } else {
                token = Some(Token {
                    ttype: TokenType::LESS,
                    lexeme: None,
                    line: *line,
                });
            }
        }
        '>' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                token = Some(Token {
                    ttype: TokenType::GREATER_EQUAL,
                    lexeme: None,
                    line: *line,
                });
                end = pos + 1
            } else {
                token = Some(Token {
                    ttype: TokenType::GREATER,
                    lexeme: None,
                    line: *line,
                });
            }
        }
        '/' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '/'
            {
                end = string.len();
            } else {
                token = Some(Token {
                    ttype: TokenType::SLASH,
                    lexeme: None,
                    line: *line,
                })
            }
        }
        '"' => {
            end = pos + 1;
            while end < string.len() && string.chars().nth(end).expect("End of string") != '"' {
                end = end + 1
            }
            if end == string.len() {
                error(*line, "Unterminated string.".to_string());
            } else {
                token = Some(Token {
                    ttype: TokenType::STRING,
                    lexeme: Some(Rc::new(string[pos + 1..end].to_string())),
                    line: *line,
                })
            }
        }
        ' ' | '\t' | '\r' => {}
        '\n' => *line = *line + 1,
        '0'..='9' => {
            end = pos;
            while end + 1 < string.len()
                && is_digit(string.chars().nth(end + 1).expect("End of string"))
            {
                end = end + 1;
            }
            if end + 2 < string.len()
                && string.chars().nth(end + 1).expect("End of string") == '.'
                && is_digit(string.chars().nth(end + 2).expect("End of string"))
            {
                end = end + 2;
                while end + 1 < string.len()
                    && is_digit(string.chars().nth(end + 1).expect("End of string"))
                {
                    end = end + 1;
                }
            }
            token = Some(Token {
                ttype: TokenType::NUMBER,
                lexeme: Some(Rc::new(string[pos..end + 1].parse::<f64>().unwrap())),
                line: *line,
            })
        }
        'a'..='z' | 'A'..='Z' => {
            end = pos;
            while end + 1 < string.len()
                && is_alpha_numeric(string.chars().nth(end + 1).expect("End of string"))
            {
                end = end + 1;
            }
            let text = string[pos..end + 1].to_string();
            let ttype: TokenType = match keywords.get(&text) {
                Some(i) => i.clone(),
                None => TokenType::IDENTIFIER,
            };
            token = Some(Token {
                ttype: ttype,
                lexeme: Some(Rc::new(text)),
                line: *line,
            });
        }
        _ => {
            error(*line, "Unexpected character".to_string());
            return (None, 0);
        }
    }
    return (token, end + 1);
}

fn is_digit(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

fn is_alpha_numeric(c: char) -> bool {
    match c {
        '0'..='9' | 'a'..='z' | 'A'..='Z' => true,
        _ => false,
    }
}
