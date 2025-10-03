use crate::error::ScanError;
use crate::scanner::keywords;
use crate::token::TokenType;

struct NewToken {
    pub ttype: TokenType,
    pub start: usize,
    pub length: i32,
    pub line: i32,
}

struct Scanner {
    source: Vec<char>,
    length: usize,
    pos: usize,
    line: i32,
}

impl Scanner {
    fn init_scanner(src: &str) -> Scanner {
        Scanner {
            source: src.chars().collect(),
            length: src.len(),
            pos: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        return self.pos == self.length;
    }

    fn scan_token(&mut self) -> Result<NewToken, ScanError> {
        self.skip_whitespace();
        let start = self.pos;
        if self.is_at_end() {
            return Ok(self.make_token(TokenType::Eof, start));
        }
        let c = self.advance();
        match c {
            '(' => return Ok(self.make_token(TokenType::LeftParen, start)),
            ')' => return Ok(self.make_token(TokenType::RightParen, start)),
            '{' => return Ok(self.make_token(TokenType::LeftBrace, start)),
            '}' => return Ok(self.make_token(TokenType::RightBrace, start)),
            ';' => return Ok(self.make_token(TokenType::Semicolon, start)),
            ',' => return Ok(self.make_token(TokenType::Comma, start)),
            '.' => return Ok(self.make_token(TokenType::Dot, start)),
            '-' => return Ok(self.make_token(TokenType::Minus, start)),
            '+' => return Ok(self.make_token(TokenType::Plus, start)),
            '/' => return Ok(self.make_token(TokenType::Slash, start)),
            '*' => return Ok(self.make_token(TokenType::Star, start)),
            '!' => {
                let ttype = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                return Ok(self.make_token(ttype, start));
            }
            '=' => {
                let ttype = if self.is_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                return Ok(self.make_token(ttype, start));
            }
            '<' => {
                let ttype = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                return Ok(self.make_token(ttype, start));
            }
            '>' => {
                let ttype = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                return Ok(self.make_token(ttype, start));
            }
            '"' => {
                while self.peek() != '"' && !self.is_at_end() {
                    if self.peek() != '\n' {
                        self.line += 1;
                    }
                    self.pos += 1
                }
                if self.is_at_end() {
                    return Err(ScanError::new(
                        self.line,
                        "Unterminated string.".to_string(),
                    ));
                }
                self.pos += 1;
                return Ok(self.make_token(TokenType::String, start));
            }
            '0'..'9' => {
                while is_digit(self.peek()) {
                    self.advance();
                }
                if self.peek() == '.' && is_digit(self.source[self.pos + 1]) {
                    self.advance();
                    while is_digit(self.peek()) {
                        self.advance();
                    }
                }
                return Ok(self.make_token(TokenType::Number, start));
            }
            'a'..'z' | 'A'..'Z' | '_' => {
                while is_alpha_numeric(self.peek()) {
                    self.advance();
                }
                let text: String = self.source[start..self.pos].iter().collect();
                let ttype: TokenType = match keywords.get(&text) {
                    Some(i) => i.clone(),
                    None => TokenType::Identifier,
                };

                return Ok(self.make_token(ttype, start));
            }
            _ => {}
        }
        return Err(ScanError::new(self.line, "Unknown Error".to_string()));
    }

    fn peek(&self) -> char {
        return self.source[self.pos];
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => self.pos += 1,
                '\n' => {
                    self.line += 1;
                    self.pos += 1;
                }
                '/' => {
                    if self.source[self.pos + 1] == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.pos += 1;
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn advance(&mut self) -> char {
        self.pos += 1;
        self.source[self.pos - 1]
    }

    fn is_match(&mut self, expect: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expect {
            return false;
        }
        self.pos += 1;
        return true;
    }

    fn make_token(&self, ttype: TokenType, start: usize) -> NewToken {
        return NewToken {
            ttype: ttype,
            start: start,
            length: (self.pos - start) as i32,
            line: self.line,
        };
    }
}

fn compile(src: &str) -> Result<(), ScanError> {
    let mut scanner = Scanner::init_scanner(src);
    let mut line: i32 = -1;
    loop {
        match scanner.scan_token() {
            Ok(token) => {
                if token.line != line {
                    line = token.line;
                    print!("{:3} ", line);
                } else {
                    print!("  | ");
                }
                println!(
                    "{:?} {} {}",
                    token.ttype,
                    token.length,
                    &src[token.start..(token.start + token.length as usize)]
                );
                if token.ttype == TokenType::Eof {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Scan Error: {}", e)
            }
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let _ = compile("var x = 1;\nvar y = 2;\nwhile(x <= 3)\n {\n x = x + y;\n}\n");
    }
}
