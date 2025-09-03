use crate::error::ParseError;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{BasicType, Token, TokenType};
use std::collections::LinkedList;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn get_count() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn parser(tokens: &mut LinkedList<Token>) -> Result<LinkedList<Box<Stmt>>, ParseError> {
    let mut statements: LinkedList<Box<Stmt>> = LinkedList::new();
    let mut has_fail: bool = false;
    while !match_head(tokens, &[TokenType::Eof]) {
        match declaration(tokens) {
            Ok(stmt) => statements.push_back(stmt),
            Err(e) => {
                has_fail = true;
                println!("{}", e);
                synchronize(tokens);
            }
        }
    }
    if has_fail {
        Err(ParseError::new(0, "Interpretation stopped.".to_string()))
    } else {
        Ok(statements)
    }
}

fn match_head(tokens: &LinkedList<Token>, slice: &[TokenType]) -> bool {
    let head = &tokens.front().unwrap().ttype;
    for t in slice.iter() {
        if *head == *t {
            return true;
        }
    }
    false
}

fn declaration(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    if match_head(tokens, &[TokenType::Class]) {
        return class_declaration(tokens);
    }
    if match_head(tokens, &[TokenType::Fun]) {
        return function_declaration(tokens);
    }
    if match_head(tokens, &[TokenType::Var]) {
        var_declaration(tokens)
    } else {
        statement(tokens)
    }
}

fn class_declaration(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    let mut superclass: Option<Box<Expr>> = None;
    if !match_head(tokens, &[TokenType::Identifier]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Invalid token for class name".to_string(),
        ));
    }
    let name = tokens.pop_front().expect("Must be an identifier.");
    if match_head(tokens, &[TokenType::Less]) {
        tokens.pop_front();
        if match_head(tokens, &[TokenType::Identifier]) {
            superclass = Some(Box::new(Expr::Variable {
                name: tokens.pop_front().expect("Must be an identifier."),
                id: get_count(),
            }));
        } else {
            return Err(ParseError::new(
                tokens.front().unwrap().line,
                "Invalid superclass name".to_string(),
            ));
        }
    }
    if !match_head(tokens, &[TokenType::LeftBrace]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect '{{' before class body".to_string(),
        ));
    }
    tokens.pop_front();
    let mut methods: LinkedList<Box<Stmt>> = LinkedList::new();
    while !match_head(tokens, &[TokenType::RightBrace]) {
        methods.push_back(function(tokens)?);
    }
    if !match_head(tokens, &[TokenType::RightBrace]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect '}}' before class body".to_string(),
        ));
    }
    tokens.pop_front();
    Ok(Box::new(Stmt::Class {
        name,
        superclass,
        methods,
    }))
}

fn function_declaration(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    function(tokens)
}

fn function(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    if !match_head(tokens, &[TokenType::Identifier]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Invalid token for function name.".to_string(),
        ));
    }
    let nm = tokens.pop_front().expect("Must be an identifier");

    if !match_head(tokens, &[TokenType::LeftParen]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect ( for function arguments.".to_string(),
        ));
    }
    tokens.pop_front();

    let mut ps: LinkedList<Token> = LinkedList::new();
    if !match_head(tokens, &[TokenType::RightParen]) {
        loop {
            if ps.len() >= 255 {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Arguments of function exceed 255.".to_string(),
                ));
            }
            if !match_head(tokens, &[TokenType::Identifier]) {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Invalid name for arguments.".to_string(),
                ));
            } else {
                ps.push_back(tokens.pop_front().expect("Must be an identifier."));
            }
            if !match_head(tokens, &[TokenType::RightParen, TokenType::Comma]) {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Invalid function definition".to_string(),
                ));
            } else if match_head(tokens, &[TokenType::RightParen]) {
                break;
            } else {
                tokens.pop_front();
            }
        }
    }
    tokens.pop_front();

    if !match_head(tokens, &[TokenType::LeftBrace]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect '{{' for function body".to_string(),
        ));
    }
    let b: LinkedList<Box<Stmt>> = block(tokens)?;
    Ok(Box::new(Stmt::Function {
        name: nm,
        params: ps,
        body: b,
    }))
}

fn var_declaration(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    if match_head(tokens, &[TokenType::Identifier]) {
        let name = tokens.pop_front().expect("Identifier Token.");
        let mut initializer: Option<Box<Expr>> = None;
        if match_head(tokens, &[TokenType::Equal]) {
            tokens.pop_front();
            match expression(tokens) {
                Ok(val) => initializer = Some(val),
                Err(e) => return Err(e),
            }
        }
        if match_head(tokens, &[TokenType::Semicolon]) {
            tokens.pop_front();
            Ok(Box::new(Stmt::Var { name, initializer }))
        } else {
            Err(ParseError::new(
                tokens.front().unwrap().line,
                "Expect ';' after expression : Declaration.".to_string(),
            ))
        }
    } else {
        Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect an identifier.".to_string(),
        ))
    }
}

fn statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    if match_head(tokens, &[TokenType::If]) {
        return if_statement(tokens);
    }
    if match_head(tokens, &[TokenType::Print]) {
        return print_statement(tokens);
    }
    if match_head(tokens, &[TokenType::Return]) {
        return return_statement(tokens);
    }
    if match_head(tokens, &[TokenType::While]) {
        return while_statement(tokens);
    }
    if match_head(tokens, &[TokenType::LeftBrace]) {
        return block_statement(tokens);
    }
    expression_statement(tokens)
}

fn block_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    block(tokens).map(|val| Box::new(Stmt::Block { statements: val }))
}

fn block(tokens: &mut LinkedList<Token>) -> Result<LinkedList<Box<Stmt>>, ParseError> {
    let mut stmts: LinkedList<Box<Stmt>> = LinkedList::new();
    tokens.pop_front();
    while !match_head(tokens, &[TokenType::RightBrace, TokenType::Eof]) {
        match declaration(tokens) {
            Ok(val) => stmts.push_back(val),
            Err(e) => return Err(e),
        }
    }
    if match_head(tokens, &[TokenType::RightBrace]) {
        tokens.pop_front();
    } else {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "No matching } for block.".to_string(),
        ));
    }
    Ok(stmts)
}

fn if_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LeftParen]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "No ( after if.".to_string(),
        ));
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr> = expression(tokens)?;
    if !match_head(tokens, &[TokenType::RightParen]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "No ) after if.".to_string(),
        ));
    } else {
        tokens.pop_front();
    }
    let then_b: Box<Stmt> = statement(tokens)?;
    let mut else_b: Option<Box<Stmt>> = None;
    if match_head(tokens, &[TokenType::Else]) {
        tokens.pop_front();
        match statement(tokens) {
            Ok(val) => else_b = Some(val),
            Err(e) => return Err(e),
        }
    }
    Ok(Box::new(Stmt::If {
        condition: cond,
        then_branch: then_b,
        else_branch: else_b,
    }))
}

fn return_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    let token = tokens.pop_front().expect("Must be keyword return.");
    let mut value: Option<Box<Expr>> = None;
    if !match_head(tokens, &[TokenType::Semicolon]) {
        value = Some(expression(tokens)?);
    }
    if !match_head(tokens, &[TokenType::Semicolon]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "Expect ';' after return.".to_string(),
        ));
    }
    tokens.pop_front();
    Ok(Box::new(Stmt::Return {
        keyword: token,
        value,
    }))
}

fn while_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LeftParen]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "No ( after while.".to_string(),
        ));
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr> = expression(tokens)?;
    if !match_head(tokens, &[TokenType::RightParen]) {
        return Err(ParseError::new(
            tokens.front().unwrap().line,
            "No ) after while.".to_string(),
        ));
    } else {
        tokens.pop_front();
    }

    let stmt: Box<Stmt> = statement(tokens)?;
    Ok(Box::new(Stmt::While {
        condition: cond,
        body: stmt,
    }))
}

fn print_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    tokens.pop_front();
    match expression(tokens) {
        Ok(value) => {
            if match_head(tokens, &[TokenType::Semicolon]) {
                tokens.pop_front();
                Ok(Box::new(Stmt::Print { expression: value }))
            } else {
                Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Expect ';' after expression.".to_string(),
                ))
            }
        }
        Err(e) => Err(e),
    }
}

fn expression_statement(tokens: &mut LinkedList<Token>) -> Result<Box<Stmt>, ParseError> {
    match expression(tokens) {
        Ok(value) => {
            if match_head(tokens, &[TokenType::Semicolon]) {
                tokens.pop_front();
                Ok(Box::new(Stmt::Expression { expression: value }))
            } else {
                Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Expect ';' after expression : Expression.".to_string(),
                ))
            }
        }
        Err(e) => Err(e),
    }
}

fn expression(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    assignment(tokens)
}

fn assignment(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let expr: Box<Expr> = or(tokens)?;
    if match_head(tokens, &[TokenType::Equal]) {
        tokens.pop_front();
        match *expr {
            Expr::Variable { name, id: _ } => {
                let val: Box<Expr> = assignment(tokens)?;
                return Ok(Box::new(Expr::Assign {
                    name,
                    value: val,
                    id: get_count(),
                }));
            }
            Expr::Get { object, name } => {
                let val = assignment(tokens)?;
                return Ok(Box::new(Expr::Set {
                    object,
                    name,
                    value: val,
                }));
            }
            _ => {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Assign to something not assignable.".to_string(),
                ))
            }
        }
    }
    Ok(expr)
}

fn or(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = and(tokens)?;
    while match_head(tokens, &[TokenType::Or]) {
        let op = tokens.pop_front().expect("Must be or.");
        let rexpr: Box<Expr> = and(tokens)?;
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op,
            right: rexpr,
        })
    }
    Ok(expr)
}

fn and(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = equality(tokens)?;
    while match_head(tokens, &[TokenType::And]) {
        let op = tokens.pop_front().expect("Must be and.");
        let rexpr: Box<Expr> = equality(tokens)?;
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op,
            right: rexpr,
        })
    }
    Ok(expr)
}

fn equality(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = comparison(tokens)?;
    while match_head(tokens, &[TokenType::BangEqual, TokenType::EqualEqual]) {
        let operator = tokens.pop_front().unwrap();
        match comparison(tokens) {
            Ok(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            Err(e) => return Err(e),
        }
    }

    Ok(expr)
}

fn comparison(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = term(tokens)?;
    while match_head(
        tokens,
        &[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ],
    ) {
        let operator = tokens.pop_front().unwrap();
        match term(tokens) {
            Ok(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            Err(e) => return Err(e),
        }
    }
    Ok(expr)
}

fn term(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = factor(tokens)?;
    while match_head(tokens, &[TokenType::Plus, TokenType::Minus]) {
        let operator = tokens.pop_front().unwrap();
        match factor(tokens) {
            Ok(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            Err(e) => return Err(e),
        }
    }

    Ok(expr)
}

fn factor(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = unary(tokens)?;
    while match_head(tokens, &[TokenType::Slash, TokenType::Star]) {
        let operator = tokens.pop_front().unwrap();
        match unary(tokens) {
            Ok(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            Err(e) => return Err(e),
        }
    }
    Ok(expr)
}

fn unary(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    if match_head(tokens, &[TokenType::Bang, TokenType::Minus]) {
        let operator = tokens.pop_front().unwrap();
        match unary(tokens) {
            Ok(x) => return Ok(Box::new(Expr::Unary { operator, right: x })),
            Err(e) => return Err(e),
        };
    }
    call(tokens)
}

fn call(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    let mut expr: Box<Expr> = primary(tokens)?;
    loop {
        if match_head(tokens, &[TokenType::LeftParen]) {
            expr = finish_call(tokens, expr)?;
        } else if match_head(tokens, &[TokenType::Dot]) {
            tokens.pop_front();
            if !match_head(tokens, &[TokenType::Identifier]) {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Invalid class method.".to_string(),
                ));
            }
            let name = tokens.pop_front().expect("Must be identifier");
            expr = Box::new(Expr::Get { object: expr, name });
        } else {
            break;
        }
    }
    Ok(expr)
}

fn finish_call(tokens: &mut LinkedList<Token>, expr: Box<Expr>) -> Result<Box<Expr>, ParseError> {
    tokens.pop_front();
    let mut args = LinkedList::<Box<Expr>>::new();
    if !match_head(tokens, &[TokenType::RightParen]) {
        loop {
            match expression(tokens) {
                Ok(val) => args.push_back(val),
                Err(e) => return Err(e),
            }
            if args.len() >= 255 {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Function can't have more than 255 arguments.".to_string(),
                ));
            }
            if !match_head(tokens, &[TokenType::RightParen, TokenType::Comma]) {
                return Err(ParseError::new(
                    tokens.front().unwrap().line,
                    "Invalid expression call.".to_string(),
                ));
            } else if match_head(tokens, &[TokenType::RightParen]) {
                break;
            }
            tokens.pop_front();
        }
    }
    let p = tokens.pop_front().expect("Must be right paren.");
    Ok(Box::new(Expr::Call {
        callee: expr,
        paren: p,
        arguments: args,
    }))
}

fn primary(tokens: &mut LinkedList<Token>) -> Result<Box<Expr>, ParseError> {
    if match_head(tokens, &[TokenType::False]) {
        tokens.pop_front();
        return Ok(Box::new(Expr::Literal {
            value: BasicType::Bool(false),
        }));
    }
    if match_head(tokens, &[TokenType::True]) {
        tokens.pop_front();
        return Ok(Box::new(Expr::Literal {
            value: BasicType::Bool(true),
        }));
    }
    if match_head(tokens, &[TokenType::Nil]) {
        tokens.pop_front();
        return Ok(Box::new(Expr::Literal {
            value: BasicType::None,
        }));
    }
    if match_head(tokens, &[TokenType::Number, TokenType::String]) {
        let token = tokens.pop_front().expect("Must be number or string");
        return Ok(Box::new(Expr::Literal {
            value: token
                .lexeme
                .clone()
                .expect("Number or string must have conent."),
        }));
    }
    if match_head(tokens, &[TokenType::LeftParen]) {
        tokens.pop_front();
        let opt = expression(tokens);
        let expr: Box<Expr> = opt?;
        if !match_head(tokens, &[TokenType::RightParen]) {
            return Err(ParseError::new(
                tokens.front().unwrap().line,
                "Expect ')' after expression.".to_string(),
            ));
        }
        tokens.pop_front();
        return Ok(Box::new(Expr::Grouping { expression: expr }));
    }
    if match_head(tokens, &[TokenType::This]) {
        let token = tokens.pop_front().ok_or(ParseError::new(
            tokens.front().unwrap().line,
            "Invalid method or property name.".to_string(),
        ));
        return Ok(Box::new(Expr::This {
            keyword: token?,
            id: get_count(),
        }));
    }
    if match_head(tokens, &[TokenType::Super]) {
        let keyword = tokens.pop_front().ok_or(ParseError::new(
            tokens.front().unwrap().line,
            "Invalid super class name.".to_string(),
        ));
        if !match_head(tokens, &[TokenType::Dot]) {
            return Err(ParseError::new(
                tokens.front().unwrap().line,
                "Expect . after super.".to_string(),
            ));
        }
        tokens.pop_front();
        if match_head(tokens, &[TokenType::Identifier]) {
            let method = tokens.pop_front().ok_or(ParseError::new(
                tokens.front().unwrap().line,
                "Invalid method name.".to_string(),
            ));
            return Ok(Box::new(Expr::Super {
                keyword: keyword?,
                method: method?,
                id: get_count(),
            }));
        } else {
            return Err(ParseError::new(
                tokens.front().unwrap().line,
                "Not an identifier after super.".to_string(),
            ));
        }
    }
    if match_head(tokens, &[TokenType::Identifier]) {
        let token = tokens.pop_front().unwrap();
        return Ok(Box::new(Expr::Variable {
            name: token,
            id: get_count(),
        }));
    }
    Err(ParseError::new(
        tokens.front().unwrap().line,
        "Uncorrected matching.".to_string(),
    ))
}

fn synchronize(tokens: &mut LinkedList<Token>) {
    while !match_head(tokens, &[TokenType::Eof]) {
        match tokens.front().unwrap().ttype {
            TokenType::Semicolon => {
                tokens.pop_front();
                return;
            }
            TokenType::Class => return,
            TokenType::Fun => return,
            TokenType::Var => return,
            TokenType::For => return,
            TokenType::If => return,
            TokenType::While => return,
            TokenType::Print => return,
            TokenType::Return => return,
            _ => {
                tokens.pop_front();
            }
        }
    }
}
