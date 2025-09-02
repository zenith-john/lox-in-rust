use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType, BasicType};
use std::collections::LinkedList;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn get_count() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn parser(tokens: &mut LinkedList<Token>) -> Option<LinkedList<Box<Stmt>>> {
    let mut statements: LinkedList<Box<Stmt>> = LinkedList::new();
    let mut has_fail: bool = false;
    while !match_head(tokens, &[TokenType::Eof]) {
        if let Some(stmt) = declaration(tokens) {
            statements.push_back(stmt);
        } else {
            has_fail = true;
            synchronize(tokens);
        }
    }
    if has_fail {
        None
    } else {
        Some(statements)
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

fn declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
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

fn class_declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    let mut superclass: Option<Box<Expr>> = None;
    if !match_head(tokens, &[TokenType::Identifier]) {
        eprintln!("Invalid Token for class name.");
        return None;
    }
    let name = tokens.pop_front()?;
    if match_head(tokens, &[TokenType::Less]) {
        tokens.pop_front();
        if match_head(tokens, &[TokenType::Identifier]) {
            superclass = Some(Box::new(Expr::Variable {
                name: tokens.pop_front()?,
                id: get_count(),
            }));
        } else {
            eprintln!("Invalid superclass.");
        }
    }
    if !match_head(tokens, &[TokenType::LeftBrace]) {
        eprintln!("Expect '{{' before class body.");
        return None;
    }
    tokens.pop_front();
    let mut methods: LinkedList<Box<Stmt>> = LinkedList::new();
    while !match_head(tokens, &[TokenType::RightBrace]) {
        methods.push_back(function(tokens)?);
    }
    if !match_head(tokens, &[TokenType::RightBrace]) {
        eprintln!("Expect '}}' after class body.");
        return None;
    }
    tokens.pop_front();
    Some(Box::new(Stmt::Class {
        name,
        superclass,
        methods,
    }))
}

fn function_declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    function(tokens)
}

fn function(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    if !match_head(tokens, &[TokenType::Identifier]) {
        eprintln!("Invalid Token for function name.");
        return None;
    }
    let nm = tokens.pop_front()?;

    if !match_head(tokens, &[TokenType::LeftParen]) {
        eprintln!("Expect ( but not found.");
        return None;
    }
    tokens.pop_front();

    let mut ps: LinkedList<Token> = LinkedList::new();
    if !match_head(tokens, &[TokenType::RightParen]) {
        loop {
            if ps.len() >= 255 {
                eprintln!("Too many arguments.");
            }
            if !match_head(tokens, &[TokenType::Identifier]) {
                eprintln!("Invalid argument.");
            } else {
                ps.push_back(tokens.pop_front()?);
            }
            if !match_head(tokens, &[TokenType::RightParen, TokenType::Comma]) {
                eprintln!("Invalid function definition.");
                return None;
            } else if match_head(tokens, &[TokenType::RightParen]) {
                break;
            } else {
                tokens.pop_front();
            }
        }
    }
    tokens.pop_front();

    if !match_head(tokens, &[TokenType::LeftBrace]) {
        eprintln!("Not correct function body.");
    }
    let b: LinkedList<Box<Stmt>> = block(tokens)?;
    Some(Box::new(Stmt::Function {
        name: nm,
        params: ps,
        body: b,
    }))
}

fn var_declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if match_head(tokens, &[TokenType::Identifier]) {
        let name = tokens.pop_front().expect("Identifier Token.");
        let mut initializer: Option<Box<Expr>> = None;
        if match_head(tokens, &[TokenType::Equal]) {
            tokens.pop_front();
            if let Some(val) = expression(tokens) {
                initializer = Some(val);
            } else {
                return None;
            }
        }
        if match_head(tokens, &[TokenType::Semicolon]) {
            tokens.pop_front();
            Some(Box::new(Stmt::Var { name, initializer }))
        } else {
            eprintln!("Expect ';' after expression : Declaration.");
            None
        }
    } else {
        eprintln!("Expect an identifier.");
        None
    }
}

fn statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
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

fn block_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    block(tokens).map(|val| Box::new(Stmt::Block { statements: val }))
}

fn block(tokens: &mut LinkedList<Token>) -> Option<LinkedList<Box<Stmt>>> {
    let mut stmts: LinkedList<Box<Stmt>> = LinkedList::new();
    tokens.pop_front();
    while !match_head(tokens, &[TokenType::RightBrace, TokenType::Eof]) {
        match declaration(tokens) {
            Some(val) => stmts.push_back(val),
            None => return None,
        }
    }
    if match_head(tokens, &[TokenType::RightBrace]) {
        tokens.pop_front();
    } else {
        eprintln!("No matching '}}'.");
        return None;
    }
    Some(stmts)
}

fn if_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LeftParen]) {
        eprintln!("No ( after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr> = expression(tokens)?;
    if !match_head(tokens, &[TokenType::RightParen]) {
        eprintln!("No ) after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let then_b: Box<Stmt> = statement(tokens)?;
    let mut else_b: Option<Box<Stmt>> = None;
    if match_head(tokens, &[TokenType::Else]) {
        tokens.pop_front();
        match statement(tokens) {
            Some(val) => else_b = Some(val),
            None => return None,
        }
    }
    Some(Box::new(Stmt::If {
        condition: cond,
        then_branch: then_b,
        else_branch: else_b,
    }))
}

fn return_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    let token = tokens.pop_front()?;
    let mut value: Option<Box<Expr>> = None;
    if !match_head(tokens, &[TokenType::Semicolon]) {
        value = expression(tokens);
    }
    if !match_head(tokens, &[TokenType::Semicolon]) {
        eprintln!("Expect ';' after return.");
        return None;
    }
    tokens.pop_front();
    Some(Box::new(Stmt::Return {
        keyword: token,
        value,
    }))
}

fn while_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LeftParen]) {
        eprintln!("No ( after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr> = expression(tokens)?;
    if !match_head(tokens, &[TokenType::RightParen]) {
        eprintln!("No ) after if.");
        return None;
    } else {
        tokens.pop_front();
    }

    let stmt: Box<Stmt> = statement(tokens)?;
    Some(Box::new(Stmt::While {
        condition: cond,
        body: stmt,
    }))
}

fn print_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if let Some(value) = expression(tokens) {
        if match_head(tokens, &[TokenType::Semicolon]) {
            tokens.pop_front();
            Some(Box::new(Stmt::Print { expression: value }))
        } else {
            eprintln!("Expect ';' after expression : Print.");
            None
        }
    } else {
        None
    }
}

fn expression_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    if let Some(value) = expression(tokens) {
        if match_head(tokens, &[TokenType::Semicolon]) {
            tokens.pop_front();
            Some(Box::new(Stmt::Expression { expression: value }))
        } else {
            eprintln!("Expect ';' after expression : Expression.");
            None
        }
    } else {
        None
    }
}

fn expression(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    assignment(tokens)
}

fn assignment(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let expr: Box<Expr> = or(tokens)?;
    if match_head(tokens, &[TokenType::Equal]) {
        tokens.pop_front();
        match *expr {
            Expr::Variable { name, id: _ } => {
                let val: Box<Expr> = assignment(tokens)?;
                return Some(Box::new(Expr::Assign {
                    name,
                    value: val,
                    id: get_count(),
                }));
            }
            Expr::Get { object, name } => {
                let val = assignment(tokens)?;
                return Some(Box::new(Expr::Set {
                    object,
                    name,
                    value: val,
                }));
            }
            _ => return None,
        }
    }
    Some(expr)
}

fn or(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let mut expr: Box<Expr>;
    match and(tokens) {
        Some(x) => expr = x,
        None => return None,
    }
    while match_head(tokens, &[TokenType::Or]) {
        let op = tokens.pop_front();
        let rexpr: Box<Expr> = and(tokens)?;
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op?,
            right: rexpr,
        })
    }
    Some(expr)
}

fn and(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let mut expr: Box<Expr>;
    match equality(tokens) {
        Some(x) => expr = x,
        None => return None,
    }
    while match_head(tokens, &[TokenType::And]) {
        let op = tokens.pop_front();
        let rexpr: Box<Expr> = equality(tokens)?;
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op?,
            right: rexpr,
        })
    }
    Some(expr)
}

fn equality(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = comparison(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::BangEqual, TokenType::EqualEqual]) {
        let operator = tokens.pop_front().unwrap();
        let right = comparison(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            None => return None,
        }
    }

    Some(expr)
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
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ],
    ) {
        let operator = tokens.pop_front().unwrap();
        let right = term(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            None => return None,
        }
    }
    Some(expr)
}

fn term(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = factor(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::Plus, TokenType::Minus]) {
        let operator = tokens.pop_front().unwrap();
        let right = factor(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            None => return None,
        }
    }

    Some(expr)
}

fn factor(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = unary(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }

    while match_head(tokens, &[TokenType::Slash, TokenType::Star]) {
        let operator = tokens.pop_front().unwrap();
        let right = unary(tokens);
        match right {
            Some(x) => {
                expr = Box::new(Expr::Binary {
                    left: expr,
                    operator,
                    right: x,
                })
            }
            None => return None,
        }
    }
    Some(expr)
}

fn unary(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if match_head(tokens, &[TokenType::Bang, TokenType::Minus]) {
        let operator = tokens.pop_front().unwrap();
        let right = unary(tokens);
        match right {
            Some(x) => return Some(Box::new(Expr::Unary { operator, right: x })),
            None => return None,
        };
    }
    call(tokens)
}

fn call(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = primary(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }
    loop {
        if match_head(tokens, &[TokenType::LeftParen]) {
            expr = finish_call(tokens, expr)?;
        } else if match_head(tokens, &[TokenType::Dot]) {
            tokens.pop_front();
            if !match_head(tokens, &[TokenType::Identifier]) {
                eprintln!("Invalid class method.");
            }
            let name = tokens.pop_front()?;
            expr = Box::new(Expr::Get { object: expr, name });
        } else {
            break;
        }
    }
    Some(expr)
}

fn finish_call(tokens: &mut LinkedList<Token>, expr: Box<Expr>) -> Option<Box<Expr>> {
    tokens.pop_front();
    let mut args = LinkedList::<Box<Expr>>::new();
    if !match_head(tokens, &[TokenType::RightParen]) {
        loop {
            match expression(tokens) {
                Some(val) => args.push_back(val),
                None => return None,
            }
            if args.len() >= 255 {
                eprintln!("Can't have more than 255 arguments.");
            }
            if !match_head(tokens, &[TokenType::RightParen, TokenType::Comma]) {
                eprintln!("Invalid expression call.");
                return None;
            } else if match_head(tokens, &[TokenType::RightParen]) {
                break;
            }
            tokens.pop_front();
        }
    }
    let p = tokens.pop_front()?;
    Some(Box::new(Expr::Call {
        callee: expr,
        paren: p,
        arguments: args,
    }))
}

fn primary(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if match_head(tokens, &[TokenType::False]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: BasicType::Bool(false),
        }));
    }
    if match_head(tokens, &[TokenType::True]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: BasicType::Bool(true),
        }));
    }
    if match_head(tokens, &[TokenType::Nil]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: BasicType::None
        }));
    }
    if match_head(tokens, &[TokenType::Number, TokenType::String]) {
        let token = tokens.pop_front().unwrap();
        return Some(Box::new(Expr::Literal {
            value: token.lexeme.clone()?,
        }));
    }
    if match_head(tokens, &[TokenType::LeftParen]) {
        tokens.pop_front();
        let opt = expression(tokens);
        let expr: Box<Expr> = opt?;
        if !match_head(tokens, &[TokenType::RightParen]) {
            error(
                tokens.front().unwrap(),
                "Expect ')' after expression.".to_string(),
            );
            return None;
        }
        tokens.pop_front();
        return Some(Box::new(Expr::Grouping { expression: expr }));
    }
    if match_head(tokens, &[TokenType::This]) {
        let token = tokens.pop_front()?;
        return Some(Box::new(Expr::This {
            keyword: token,
            id: get_count(),
        }));
    }
    if match_head(tokens, &[TokenType::Super]) {
        let keyword = tokens.pop_front()?;
        if !match_head(tokens, &[TokenType::Dot]) {
            eprintln!("Expect . after super.");
            return None;
        }
        tokens.pop_front();
        if match_head(tokens, &[TokenType::Identifier]) {
            let method = tokens.pop_front()?;
            return Some(Box::new(Expr::Super {
                keyword,
                method,
                id: get_count(),
            }));
        } else {
            eprintln!("Not an identifier after super.");
            return None;
        }
    }
    if match_head(tokens, &[TokenType::Identifier]) {
        let token = tokens.pop_front().unwrap();
        return Some(Box::new(Expr::Variable {
            name: token,
            id: get_count(),
        }));
    }
    error(tokens.front().unwrap(), "No matching.".to_string());
    None
}

fn error(token: &Token, message: String) {
    if token.ttype == TokenType::Eof {
        println!("Line {} unsolved at end. {}", token.line, message);
    } else {
        println!("{} at '{}' {}", token.line, token, message);
    }
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
