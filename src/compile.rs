use crate::chunk::*;
use crate::scanner::keywords;
use crate::token::{BasicType, TokenType};
use crate::vm::VM;
use crate::{DEBUG, USIZE};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
pub enum Prec {
    // Precedence
    None = 0,
    Assignment = 1,
    Or = 2,
    And = 3,
    Equality = 4,
    Comparison = 5,
    Term = 6,
    Factor = 7,
    Unary = 8,
    Call = 9,
    Primary = 10,
}

impl Prec {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn next(self) -> Self {
        FromPrimitive::from_u8(self.as_u8() + 1).unwrap_or(Self::Primary)
    }
}

impl PartialOrd for Prec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Prec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

#[derive(Debug)]
pub struct ParseError {
    line: i32,
    token: String,
    reason: String,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[line {}] at {}: {}", self.line, self.token, self.reason)
    }
}
impl std::error::Error for ParseError {}

#[derive(Clone, Copy)]
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
        self.pos == self.length
    }

    fn scan_token(&mut self) -> Result<NewToken, ParseError> {
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
                    return Err(ParseError {
                        line: self.line,
                        token: "end".to_string(),
                        reason: "Unterminated string.".to_string(),
                    });
                }
                self.pos += 1;
                return Ok(self.make_token(TokenType::String, start));
            }
            '0'..='9' => {
                while !self.is_at_end() && is_digit(self.peek()) {
                    self.advance();
                }
                if !self.is_at_end() && self.peek() == '.' && is_digit(self.source[self.pos + 1]) {
                    self.advance();
                    while !self.is_at_end() && is_digit(self.peek()) {
                        self.advance();
                    }
                }
                return Ok(self.make_token(TokenType::Number, start));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                while !self.is_at_end() && is_alpha_numeric(self.peek()) {
                    self.advance();
                }
                let text: String = self.source[start..self.pos].iter().collect();
                let ttype: TokenType = match keywords.get(&text) {
                    Some(i) => *i,
                    None => TokenType::Identifier,
                };

                return Ok(self.make_token(ttype, start));
            }
            _ => {}
        }
        Err(ParseError {
            line: self.line,
            token: self.source[start..self.pos].iter().collect(),
            reason: "Unknown Error".to_string(),
        })
    }

    fn peek(&self) -> char {
        self.source[self.pos]
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
        true
    }

    fn make_token(&self, ttype: TokenType, start: usize) -> NewToken {
        NewToken {
            ttype,
            start,
            length: (self.pos - start) as i32,
            line: self.line,
        }
    }

    fn get_string(&self, start: usize, length: i32) -> String {
        self.source[start..start + length as usize].iter().collect()
    }
}

pub fn compile(src: &str) {
    let scanner = Scanner::init_scanner(src);
    let mut parser = Parser {
        previous: NewToken {
            ttype: TokenType::Eof,
            start: 0,
            length: 0,
            line: -1,
        },
        current: NewToken {
            ttype: TokenType::Eof,
            start: 0,
            length: 0,
            line: -1,
        },
        had_error: false,
        scanner: Box::new(scanner),
        chunk: Box::new(Chunk::new()),
        scope: Scope::init(),
    };
    if !parser.parse() {
        parser.run();
    }
}

struct Parser {
    current: NewToken,
    previous: NewToken,
    had_error: bool,
    scanner: Box<Scanner>,
    chunk: Box<Chunk>,
    scope: Scope,
}

impl Parser {
    fn advance(&mut self) -> Result<(), ParseError> {
        self.previous = self.current;
        let token = self.scanner.scan_token()?;
        self.current = token;
        Ok(())
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write_chunk(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OP_RETURN);
    }

    fn number(&mut self) -> Result<(), ParseError> {
        let value = self.get_string(&self.previous).parse::<f64>().unwrap();
        self.emit_constant(BasicType::Number(value))
    }

    fn emit_constant(&mut self, val: Value) -> Result<(), ParseError> {
        let pos = self.make_constant(val)?;
        self.emit_bytes(OP_CONSTANT, pos);
        Ok(())
    }

    fn make_constant(&mut self, val: Value) -> Result<u8, ParseError> {
        let pos = self.chunk.add_constant(val);
        // Deal with possibly too many constants
        if pos >= 256 {
            return Err(ParseError {
                line: self.previous.line,
                token: self.get_string(&self.previous),
                reason: "Too many constants".to_string(),
            });
        }
        Ok(pos as u8)
    }

    fn declaration(&mut self) -> Result<(), ParseError> {
        if self.match_advance(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<(), ParseError> {
        let global: u8 = self.parse_variable()?;
        if self.match_advance(TokenType::Equal) {
            self.expression()?;
        } else {
            self.emit_byte(OP_NIL);
        }
        self.expect(TokenType::Semicolon)?;
        self.define_variable(global)
    }

    fn parse_variable(&mut self) -> Result<u8, ParseError> {
        self.expect(TokenType::Identifier)?;

        self.declare_variable()?;
        if self.scope.depth > 0 {
            return Ok(0);
        }

        self.identifier_constant()
    }

    fn declare_variable(&mut self) -> Result<(), ParseError> {
        if self.scope.depth == 0 {
            return Ok(());
        }

        let name = &self.previous;
        for i in (0..self.scope.locals.len()).rev() {
            if self.scope.depth != -1 && self.scope.locals[i].depth < self.scope.depth {
                break;
            }
            if self.identifier_equal(name, &self.scope.locals[i].name) {
                return Err(ParseError {
                    line: self.previous.line,
                    token: self.get_string(&self.previous),
                    reason: "A variable with same name defined in this scope".to_string(),
                });
            }
        }
        self.add_local(*name)
    }

    fn identifier_equal(&self, token1: &NewToken, token2: &NewToken) -> bool {
        self.get_string(token1) == self.get_string(token2)
    }

    fn add_local(&mut self, token: NewToken) -> Result<(), ParseError> {
        self.scope.locals.push(Local {
            name: token,
            depth: -1,
        });
        Ok(())
    }

    fn identifier_constant(&mut self) -> Result<u8, ParseError> {
        self.make_constant(BasicType::String(self.get_string(&self.previous)))
    }

    fn define_variable(&mut self, id: u8) -> Result<(), ParseError> {
        if self.scope.depth > 0 {
            self.make_initialized();
            return Ok(());
        }
        self.emit_bytes(OP_DEFINE_GLOBAL, id);
        Ok(())
    }

    fn make_initialized(&mut self) {
        let len = self.scope.locals.len();
        self.scope.locals[len - 1].depth = self.scope.depth;
    }

    fn begin_scope(&mut self) {
        self.scope.depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope.depth -= 1;

        for i in (0..self.scope.locals.len()).rev() {
            if self.scope.locals[i].depth > self.scope.depth {
                self.emit_byte(OP_POP);
                self.scope.locals.pop();
            } else {
                return;
            }
        }
    }

    fn statement(&mut self) -> Result<(), ParseError> {
        if self.match_advance(TokenType::Print) {
            self.print_statement()
        } else if self.match_advance(TokenType::If) {
            self.if_statement()
        } else if self.match_advance(TokenType::While) {
            self.while_statement()
        } else if self.match_advance(TokenType::LeftBrace) {
            self.begin_scope();
            self.block()?;
            self.end_scope();
            Ok(())
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<(), ParseError> {
        self.expect(TokenType::LeftParen)?;
        self.expression()?;
        self.expect(TokenType::RightParen)?;

        let then_jump = self.emit_jump(OP_JUMP_IF_FALSE)?;
        self.emit_byte(OP_POP);
        self.statement()?;

        if self.match_advance(TokenType::Else) {
            let else_jump = self.emit_jump(OP_JUMP)?;
            self.patch_jump(then_jump)?;
            self.emit_byte(OP_POP);
            self.statement()?;
            self.patch_jump(else_jump)?;
        } else {
            self.patch_jump(then_jump)?;
        }
        Ok(())
    }

    fn emit_jump(&mut self, op: u8) -> Result<usize, ParseError> {
        self.emit_byte(op);
        for _ in 0..USIZE {
            self.emit_byte(0xff);
        }
        Ok(self.chunk.len() - USIZE)
    }

    fn emit_loop(&mut self, start: usize) -> Result<(), ParseError> {
        self.emit_byte(OP_LOOP);
        let offset = self.chunk.len() - start + USIZE;
        for byte in offset.to_ne_bytes() {
            self.emit_byte(byte);
        }
        Ok(())
    }

    fn patch_jump(&mut self, offset: usize) -> Result<(), ParseError> {
        let jump = self.chunk.len() - offset - USIZE;
        let bytes = jump.to_ne_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            self.chunk.modify_chunk(offset + i, byte);
        }
        Ok(())
    }

    fn while_statement(&mut self) -> Result<(), ParseError> {
        let start = self.chunk.len();
        self.expect(TokenType::LeftParen)?;
        self.expression()?;
        self.expect(TokenType::RightParen)?;
        let exit_jump = self.emit_jump(OP_JUMP_IF_FALSE)?;
        self.emit_byte(OP_POP);
        self.statement()?;
        self.emit_loop(start)?;
        self.patch_jump(exit_jump)?;
        self.emit_byte(OP_POP);
        Ok(())
    }

    fn block(&mut self) -> Result<(), ParseError> {
        while (!self.is_match(TokenType::RightBrace)) && (!self.is_match(TokenType::Eof)) {
            self.declaration()?;
        }

        self.expect(TokenType::RightBrace)
    }

    fn expression_statement(&mut self) -> Result<(), ParseError> {
        self.expression()?;
        self.expect(TokenType::Semicolon)?;
        self.emit_byte(OP_POP);
        Ok(())
    }

    fn print_statement(&mut self) -> Result<(), ParseError> {
        self.expression()?;
        self.expect(TokenType::Semicolon)?;
        self.emit_byte(OP_PRINT);
        Ok(())
    }

    fn grouping(&mut self) -> Result<(), ParseError> {
        self.expression()?;
        self.expect(TokenType::RightParen)
    }

    fn unary(&mut self) -> Result<(), ParseError> {
        let op = self.previous.ttype;
        self.expression()?;
        match op {
            TokenType::Minus => {
                self.emit_byte(OP_NEGATE);
            }
            TokenType::Bang => {
                self.emit_byte(OP_NOT);
            }
            _ => {
                return Err(ParseError {
                    line: self.previous.line,
                    token: self.get_string(&self.previous),
                    reason: format!("{:?} is not an unary operator.", self.previous.ttype),
                })
            }
        }
        Ok(())
    }

    fn binary(&mut self) -> Result<(), ParseError> {
        let op = self.previous.ttype;

        let prec = get_precedence(op);
        self.parse_precedence(prec.next())?;
        match op {
            TokenType::Plus => self.emit_byte(OP_ADD),
            TokenType::Minus => self.emit_byte(OP_SUBTRACT),
            TokenType::Star => self.emit_byte(OP_MULTIPLY),
            TokenType::Slash => self.emit_byte(OP_DIVIDE),
            TokenType::BangEqual => self.emit_bytes(OP_EQUAL, OP_NOT),
            TokenType::EqualEqual => self.emit_byte(OP_EQUAL),
            TokenType::Greater => self.emit_byte(OP_GREATER),
            TokenType::GreaterEqual => self.emit_bytes(OP_LESS, OP_NOT),
            TokenType::Less => self.emit_byte(OP_LESS),
            TokenType::LessEqual => self.emit_bytes(OP_GREATER, OP_NOT),
            _ => {
                return Err(ParseError {
                    line: self.previous.line,
                    token: self.get_string(&self.previous),
                    reason: format!("{:?} is not a binary operator.", self.previous.ttype),
                })
            }
        }
        Ok(())
    }

    fn literal(&mut self) -> Result<(), ParseError> {
        match self.previous.ttype {
            TokenType::False => self.emit_byte(OP_FALSE),
            TokenType::Nil => self.emit_byte(OP_NIL),
            TokenType::True => self.emit_byte(OP_TRUE),
            _ => {
                return Err(ParseError {
                    line: self.previous.line,
                    token: self.get_string(&self.previous),
                    reason: format!("{:?} expect expression.", self.previous.ttype),
                })
            }
        }
        Ok(())
    }

    fn expression(&mut self) -> Result<(), ParseError> {
        self.parse_precedence(Prec::Assignment)
    }

    fn string(&mut self) -> Result<(), ParseError> {
        let string = self.get_string(&self.previous);
        self.emit_constant(BasicType::String(string[1..string.len() - 1].to_string()))
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), ParseError> {
        self.named_variable(can_assign)
    }

    fn resolve_local(&mut self) -> Result<Option<u8>, ParseError> {
        for i in (0..self.scope.locals.len()).rev() {
            if self.identifier_equal(&self.previous, &self.scope.locals[i].name) {
                if self.scope.locals[i].depth == -1 {
                    return Err(ParseError {
                        line: self.previous.line,
                        token: self.get_string(&self.previous),
                        reason: "Can't read local variable in its own identifier".to_string(),
                    });
                }
                return Ok(Some(i as u8));
            }
        }
        Ok(None)
    }

    fn named_variable(&mut self, can_assign: bool) -> Result<(), ParseError> {
        let arg = self.resolve_local()?;
        let get_op = if arg.is_none() {
            OP_GET_GLOBAL
        } else {
            OP_GET_LOCAL
        };
        let set_op = if arg.is_none() {
            OP_SET_GLOBAL
        } else {
            OP_SET_LOCAL
        };

        let pos = if let Some(u) = arg {
            u
        } else {
            self.identifier_constant()?
        };

        if can_assign && self.match_advance(TokenType::Equal) {
            self.expression()?;
            self.emit_bytes(set_op, pos);
        } else {
            self.emit_bytes(get_op, pos);
        }
        Ok(())
    }

    fn parse_precedence(&mut self, prec: Prec) -> Result<(), ParseError> {
        self.advance()?;
        let can_assign = prec <= Prec::Assignment;
        match self.previous.ttype {
            TokenType::LeftParen => self.grouping(),
            TokenType::Number => self.number(),
            TokenType::Minus | TokenType::Bang => self.unary(),
            TokenType::False | TokenType::True | TokenType::Nil => self.literal(),
            TokenType::String => self.string(),
            TokenType::Identifier => self.variable(can_assign),
            _ => Err(ParseError {
                line: self.previous.line,
                token: self.get_string(&self.previous),
                reason: format!("{:?} expect expression.", self.previous.ttype),
            }),
        }?;

        while prec <= get_precedence(self.current.ttype) {
            self.advance()?;
            match self.previous.ttype {
                TokenType::Minus
                | TokenType::Plus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::BangEqual
                | TokenType::EqualEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual => self.binary(),
                TokenType::And => self.and(),
                TokenType::Or => self.or(),
                _ => Ok(()),
            }?
        }
        if can_assign && self.is_match(TokenType::Equal) {
            return Err(ParseError {
                line: self.previous.line,
                token: self.get_string(&self.current),
                reason: "Invalid assignment statement.".to_string(),
            });
        }
        Ok(())
    }

    fn and(&mut self) -> Result<(), ParseError> {
        let end_jump = self.emit_jump(OP_JUMP_IF_FALSE)?;
        self.emit_byte(OP_POP);
        self.parse_precedence(Prec::And)?;
        self.patch_jump(end_jump)
    }

    fn or(&mut self) -> Result<(), ParseError> {
        let else_jump = self.emit_jump(OP_JUMP_IF_FALSE)?;
        let end_jump = self.emit_jump(OP_JUMP)?;
        self.patch_jump(else_jump)?;
        self.emit_byte(OP_POP);
        self.parse_precedence(Prec::Or)?;
        self.patch_jump(end_jump)
    }

    fn get_string(&self, token: &NewToken) -> String {
        self.scanner.get_string(token.start, token.length)
    }

    fn expect(&mut self, ttype: TokenType) -> Result<(), ParseError> {
        if self.current.ttype == ttype {
            self.advance()?;
            Ok(())
        } else {
            Err(ParseError {
                line: self.current.line,
                token: self.get_string(&self.current),
                reason: format!("Expected {:?} but get {:?}", ttype, self.current.ttype),
            })
        }
    }

    fn is_match(&mut self, ttype: TokenType) -> bool {
        self.current.ttype == ttype
    }

    fn match_advance(&mut self, ttype: TokenType) -> bool {
        if !self.is_match(ttype) {
            false
        } else {
            let _ = self.advance(); // May not the desired behavior
            true
        }
    }

    fn disassemble_chunk(&self) {
        self.chunk.disassemble_chunk();
    }

    fn synchronize(&mut self) {
        while self.current.ttype != TokenType::Eof {
            if self.previous.ttype == TokenType::Semicolon {
                return;
            }
            match self.current.ttype {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    let _ = self.advance();
                }
            }
        }
    }

    fn handle_result(&mut self, res: Result<(), ParseError>) {
        if let Err(e) = res {
            self.had_error = true;
            eprintln!("{}", e);
            let _ = self.advance();
            self.synchronize();
        }
    }

    fn parse(&mut self) -> bool {
        let res = self.advance();
        self.handle_result(res);
        while !self.match_advance(TokenType::Eof) {
            let res = self.declaration();
            self.handle_result(res);
        }
        self.emit_return();
        if DEBUG && !self.had_error {
            self.disassemble_chunk();
        }
        self.had_error
    }

    fn run(&self) {
        let mut vm = VM::init(&self.chunk);
        vm.interpret(&self.chunk);
    }
}

struct Local {
    name: NewToken,
    depth: i32,
}

struct Scope {
    pub locals: Vec<Local>,
    pub depth: i32,
}

impl Scope {
    fn init() -> Scope {
        Scope {
            locals: Vec::new(),
            depth: 0,
        }
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_alpha_numeric(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_')
}

fn get_precedence(ttype: TokenType) -> Prec {
    match ttype {
        TokenType::Minus | TokenType::Plus => Prec::Term,
        TokenType::Slash | TokenType::Star => Prec::Factor,
        TokenType::BangEqual | TokenType::EqualEqual => Prec::Equality,
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => {
            Prec::Comparison
        }
        TokenType::And => Prec::And,
        TokenType::Or => Prec::Or,
        _ => Prec::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_prec1() {
        let _ = compile("1 + 2 - 3 * 4;");
    }

    #[test]
    fn test_compile_prec2() {
        let _ = compile("1 - (2 - 3) * 4;");
    }

    #[test]
    fn test_bool() {
        let _ = compile("true;");
    }

    #[test]
    fn test_type_mismatch() {
        let _ = compile("- true;");
    }

    #[test]
    fn test_invalid_assignment() {
        let _ = compile("var a = 1;\nvar b = 2;\na * b = 3;");
    }

    #[test]
    fn test_string_concatenation() {
        let _ = compile("\"test\" + \"output\";");
    }

    #[test]
    fn test_compile() {
        let _ = compile("var x = \"test\";\nvar y = \"output\";\nprint x + y;\n");
    }

    #[test]
    fn test_local_variable() {
        let _ =
            compile("var x = 1;\n{\nvar x = 2;\nprint x;\nvar y=2;\nprint x + y;\n}\nprint x;\n");
    }

    #[test]
    fn test_while_statement() {
        let _ = compile("var x = 1;\nvar y = 5;\nwhile (x <= y)\n{\nprint x;\nx = x + 1;\n}\n");
    }

    #[test]
    fn test_if_statement() {
        let _ = compile("var x = true;\nvar y = false;\nif (x or y)\n print \"Correct\";\nelse\nprint \"Wrong\";\n");
    }

    #[test]
    fn test_if_statement2() {
        let _ = compile("var x = true;\nvar y = false;\nif (x and y)\n print \"Wrong\";\nelse\nprint \"Correct\";\n");
    }
}
