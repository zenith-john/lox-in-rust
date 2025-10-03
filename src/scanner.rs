use crate::error::ScanError;
use crate::token::{BasicType, Token, TokenType};
use lazy_static::lazy_static;
use std::collections::{HashMap, LinkedList};

lazy_static! {
    pub static ref keywords: HashMap<String, TokenType> = HashMap::from([
        ("and".to_string(), TokenType::And),
        ("class".to_string(), TokenType::Class),
        ("else".to_string(), TokenType::Else),
        ("false".to_string(), TokenType::False),
        ("for".to_string(), TokenType::For),
        ("fun".to_string(), TokenType::Fun),
        ("if".to_string(), TokenType::If),
        ("nil".to_string(), TokenType::Nil),
        ("or".to_string(), TokenType::Or),
        ("print".to_string(), TokenType::Print),
        ("return".to_string(), TokenType::Return),
        ("super".to_string(), TokenType::Super),
        ("this".to_string(), TokenType::This),
        ("true".to_string(), TokenType::True),
        ("var".to_string(), TokenType::Var),
        ("while".to_string(), TokenType::While),
    ]);
}

pub fn scan_tokens(string: &str, line: &mut i32) -> Result<LinkedList<Token>, ScanError> {
    let mut start: usize;
    let mut current: usize = 0;
    let mut tokens: LinkedList<Token> = LinkedList::new();
    while current < string.len() {
        while current < string.len()
            && is_blank(string.chars().nth(current).expect("Not at end of string"))
        {
            current += 1
        }
        start = current;
        match scan_token(string, start, line) {
            Err(e) => return Err(e),
            Ok((token, c)) => {
                tokens.push_back(token);
                current = c;
            }
        };
    }
    if tokens.is_empty() || tokens.back().expect("Not empty").ttype != TokenType::Eof {
        tokens.push_back(Token {
            ttype: TokenType::Eof,
            lexeme: None,
            line: *line,
        });
    }
    Ok(tokens)
}

fn scan_token(string: &str, pos: usize, line: &mut i32) -> Result<(Token, usize), ScanError> {
    let c: char = string.chars().nth(pos).expect("End of string.");
    let mut end: usize = pos;
    let token: Token = match c {
        '(' => Token {
            ttype: TokenType::LeftParen,
            lexeme: None,
            line: *line,
        },
        ')' => Token {
            ttype: TokenType::RightParen,
            lexeme: None,
            line: *line,
        },
        '{' => Token {
            ttype: TokenType::LeftBrace,
            lexeme: None,
            line: *line,
        },
        '}' => Token {
            ttype: TokenType::RightBrace,
            lexeme: None,
            line: *line,
        },
        ',' => Token {
            ttype: TokenType::Comma,
            lexeme: None,
            line: *line,
        },
        '.' => Token {
            ttype: TokenType::Dot,
            lexeme: None,
            line: *line,
        },
        '-' => Token {
            ttype: TokenType::Minus,
            lexeme: None,
            line: *line,
        },
        '+' => Token {
            ttype: TokenType::Plus,
            lexeme: None,
            line: *line,
        },
        ';' => Token {
            ttype: TokenType::Semicolon,
            lexeme: None,
            line: *line,
        },
        '*' => Token {
            ttype: TokenType::Star,
            lexeme: None,
            line: *line,
        },
        '!' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                end = pos + 1;
                Token {
                    ttype: TokenType::BangEqual,
                    lexeme: None,
                    line: *line,
                }
            } else {
                Token {
                    ttype: TokenType::Bang,
                    lexeme: None,
                    line: *line,
                }
            }
        }
        '=' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                end = pos + 1;
                Token {
                    ttype: TokenType::EqualEqual,
                    lexeme: None,
                    line: *line,
                }
            } else {
                Token {
                    ttype: TokenType::Equal,
                    lexeme: None,
                    line: *line,
                }
            }
        }
        '<' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                end = pos + 1;
                Token {
                    ttype: TokenType::LessEqual,
                    lexeme: None,
                    line: *line,
                }
            } else {
                Token {
                    ttype: TokenType::Less,
                    lexeme: None,
                    line: *line,
                }
            }
        }
        '>' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '='
            {
                end = pos + 1;
                Token {
                    ttype: TokenType::GreaterEqual,
                    lexeme: None,
                    line: *line,
                }
            } else {
                Token {
                    ttype: TokenType::Greater,
                    lexeme: None,
                    line: *line,
                }
            }
        }
        '/' => {
            if pos + 1 < string.len() && string.chars().nth(pos + 1).expect("End of string") == '/'
            {
                end = string.len();
                Token {
                    ttype: TokenType::Eof,
                    lexeme: None,
                    line: *line,
                }
            } else {
                Token {
                    ttype: TokenType::Slash,
                    lexeme: None,
                    line: *line,
                }
            }
        }
        '"' => {
            end = pos + 1;
            while end < string.len() && string.chars().nth(end).expect("End of string") != '"' {
                end += 1
            }
            if end == string.len() {
                return Err(ScanError::new(*line, "Unterminated string.".to_string()));
            } else {
                Token {
                    ttype: TokenType::String,
                    lexeme: Some(BasicType::String(string[pos + 1..end].to_string())),
                    line: *line,
                }
            }
        }
        '0'..='9' => {
            end = pos;
            while end + 1 < string.len()
                && is_digit(string.chars().nth(end + 1).expect("End of string"))
            {
                end += 1;
            }
            if end + 2 < string.len()
                && string.chars().nth(end + 1).expect("End of string") == '.'
                && is_digit(string.chars().nth(end + 2).expect("End of string"))
            {
                end += 2;
                while end + 1 < string.len()
                    && is_digit(string.chars().nth(end + 1).expect("End of string"))
                {
                    end += 1;
                }
            }
            Token {
                ttype: TokenType::Number,
                lexeme: Some(BasicType::Number(
                    string[pos..end + 1].parse::<f64>().unwrap(),
                )),
                line: *line,
            }
        }
        'a'..='z' | 'A'..='Z' => {
            end = pos;
            while end + 1 < string.len()
                && is_alpha_numeric(string.chars().nth(end + 1).expect("End of string"))
            {
                end += 1;
            }
            let text = string[pos..end + 1].to_string();
            let ttype: TokenType = match keywords.get(&text) {
                Some(i) => i.clone(),
                None => TokenType::Identifier,
            };
            Token {
                ttype,
                lexeme: Some(BasicType::String(text)),
                line: *line,
            }
        }
        _ => {
            return Err(ScanError::new(*line, "Unterminated string.".to_string()));
        }
    };
    Ok((token, end + 1))
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z')
}

fn is_alpha_numeric(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_')
}

fn is_blank(c: char) -> bool {
    matches!(c, '\r' | '\n' | ' ' | '\t')
}
