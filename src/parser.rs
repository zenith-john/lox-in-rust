use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use std::collections::LinkedList;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn get_count() -> u64 {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    return id;
}

pub fn parser(tokens: &mut LinkedList<Token>) -> Option<LinkedList<Box<Stmt>>> {
    let mut statements: LinkedList<Box<Stmt>> = LinkedList::new();
    let mut has_fail: bool = false;
    while !match_head(tokens, &[TokenType::EOF]) {
        if let Some(stmt) = declaration(tokens) {
            statements.push_back(stmt);
        } else {
            has_fail = true;
            synchronize(tokens);
        }
    }
    if has_fail {
        return None;
    } else {
        return Some(statements);
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

fn declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    if match_head(tokens, &[TokenType::FUN]) {
        return function(tokens);
    }
    if match_head(tokens, &[TokenType::VAR]) {
        return var_declaration(tokens);
    } else {
        return statement(tokens);
    }
}

fn function(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::IDENTIFIER]) {
        eprintln!("Invalide Token for function name.");
        return None;
    }
    let nm = tokens.pop_front()?;

    if !match_head(tokens, &[TokenType::LEFT_PAREN]) {
        eprintln!("Expect ( but not found.");
        return None;
    }
    tokens.pop_front();

    let mut ps: LinkedList<Token> = LinkedList::new();
    if !match_head(tokens, &[TokenType::RIGHT_PAREN]) {
        loop {
            if ps.len() >= 255 {
                eprintln!("Too many arguments.");
            }
            if !match_head(tokens, &[TokenType::IDENTIFIER]) {
                eprintln!("Invalid argument.");
            } else {
                ps.push_back(tokens.pop_front()?);
            }
            if !match_head(tokens, &[TokenType::RIGHT_PAREN, TokenType::COMMA]) {
                eprintln!("Invalid function definition.");
                return None;
            } else if match_head(tokens, &[TokenType::RIGHT_PAREN]) {
                break;
            } else {
                tokens.pop_front();
            }
        }
    }
    tokens.pop_front();

    if !match_head(tokens, &[TokenType::LEFT_BRACE]) {
        eprintln!("Not correct function body.");
    }
    let b: LinkedList<Box<Stmt>> = match block(tokens) {
        None => return None,
        Some(val) => val,
    };
    return Some(Box::new(Stmt::Function {
        name: nm,
        params: ps,
        body: b,
    }));
}

fn var_declaration(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if match_head(tokens, &[TokenType::IDENTIFIER]) {
        let name = tokens.pop_front().expect("Identifier Token.");
        let mut initializer: Option<Box<Expr>> = None;
        if match_head(tokens, &[TokenType::EQUAL]) {
            tokens.pop_front();
            if let Some(val) = expression(tokens) {
                initializer = Some(val);
            } else {
                return None;
            }
        }
        if match_head(tokens, &[TokenType::SEMICOLON]) {
            tokens.pop_front();
            return Some(Box::new(Stmt::Var {
                name: name,
                initializer: initializer,
            }));
        } else {
            eprintln!("Expect ';' after expression : Declaration.");
            return None;
        }
    } else {
        eprintln!("Expect an identifier.");
        return None;
    }
}

fn statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    if match_head(tokens, &[TokenType::IF]) {
        return if_statement(tokens);
    }
    if match_head(tokens, &[TokenType::PRINT]) {
        return print_statement(tokens);
    }
    if match_head(tokens, &[TokenType::RETURN]) {
        return return_statement(tokens);
    }
    if match_head(tokens, &[TokenType::WHILE]) {
        return while_statement(tokens);
    }
    if match_head(tokens, &[TokenType::LEFT_BRACE]) {
        return block_statement(tokens);
    }
    return expression_statement(tokens);
}

fn block_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    match block(tokens) {
        Some(val) => return Some(Box::new(Stmt::Block { statements: val })),
        None => return None,
    }
}

fn block(tokens: &mut LinkedList<Token>) -> Option<LinkedList<Box<Stmt>>> {
    let mut stmts: LinkedList<Box<Stmt>> = LinkedList::new();
    tokens.pop_front();
    while !match_head(tokens, &[TokenType::RIGHT_BRACE, TokenType::EOF]) {
        match declaration(tokens) {
            Some(val) => stmts.push_back(val),
            None => return None,
        }
    }
    if match_head(tokens, &[TokenType::RIGHT_BRACE]) {
        tokens.pop_front();
    } else {
        eprintln!("No matching '}}'.");
        return None;
    }
    return Some(stmts);
}

fn if_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LEFT_PAREN]) {
        eprintln!("No ( after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr>;
    match expression(tokens) {
        Some(val) => cond = val,
        None => return None,
    }
    if !match_head(tokens, &[TokenType::RIGHT_PAREN]) {
        eprintln!("No ) after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let then_b: Box<Stmt>;
    match statement(tokens) {
        Some(val) => then_b = val,
        None => return None,
    }
    let mut else_b: Option<Box<Stmt>> = None;
    if match_head(tokens, &[TokenType::ELSE]) {
        tokens.pop_front();
        match statement(tokens) {
            Some(val) => else_b = Some(val),
            None => return None,
        }
    }
    return Some(Box::new(Stmt::If {
        condition: cond,
        then_branch: then_b,
        else_branch: else_b,
    }));
}

fn return_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    let token = tokens.pop_front()?;
    let mut value: Option<Box<Expr>> = None;
    if !match_head(tokens, &[TokenType::SEMICOLON]) {
        value = expression(tokens);
    }
    if !match_head(tokens, &[TokenType::SEMICOLON]) {
        eprintln!("Expect ';' after return.");
        return None;
    }
    tokens.pop_front();
    return Some(Box::new(Stmt::Return {
        keyword: token,
        value: value,
    }));
}

fn while_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if !match_head(tokens, &[TokenType::LEFT_PAREN]) {
        eprintln!("No ( after if.");
        return None;
    } else {
        tokens.pop_front();
    }
    let cond: Box<Expr>;
    match expression(tokens) {
        Some(val) => cond = val,
        None => return None,
    }
    if !match_head(tokens, &[TokenType::RIGHT_PAREN]) {
        eprintln!("No ) after if.");
        return None;
    } else {
        tokens.pop_front();
    }

    let stmt: Box<Stmt>;
    match statement(tokens) {
        None => return None,
        Some(val) => stmt = val,
    }
    return Some(Box::new(Stmt::While {
        condition: cond,
        body: stmt,
    }));
}

fn print_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    tokens.pop_front();
    if let Some(value) = expression(tokens) {
        if match_head(tokens, &[TokenType::SEMICOLON]) {
            tokens.pop_front();
            return Some(Box::new(Stmt::Print { expression: value }));
        } else {
            eprintln!("Expect ';' after expression : Print.");
            return None;
        }
    } else {
        return None;
    }
}

fn expression_statement(tokens: &mut LinkedList<Token>) -> Option<Box<Stmt>> {
    if let Some(value) = expression(tokens) {
        if match_head(tokens, &[TokenType::SEMICOLON]) {
            tokens.pop_front();
            return Some(Box::new(Stmt::Expression { expression: value }));
        } else {
            eprintln!("Expect ';' after expression : Expression.");
            return None;
        }
    } else {
        return None;
    }
}

fn expression(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    return assignment(tokens);
}

fn assignment(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let expr: Box<Expr>;
    match or(tokens) {
        Some(x) => expr = x,
        None => return None,
    }
    if match_head(tokens, &[TokenType::EQUAL]) {
        tokens.pop_front();
        match *expr {
            Expr::Variable { name, id: _ } => {
                let val: Box<Expr>;
                match assignment(tokens) {
                    Some(x) => val = x,
                    None => return None,
                }
                return Some(Box::new(Expr::Assign {
                    name: name,
                    value: val,
                    id: get_count(),
                }));
            }
            _ => return None,
        }
    }
    return Some(expr);
}

fn or(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let mut expr: Box<Expr>;
    match and(tokens) {
        Some(x) => expr = x,
        None => return None,
    }
    while match_head(tokens, &[TokenType::OR]) {
        let op = tokens.pop_front();
        let rexpr: Box<Expr>;
        match and(tokens) {
            Some(x) => rexpr = x,
            None => return None,
        }
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op?,
            right: rexpr,
        })
    }
    return Some(expr);
}

fn and(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let mut expr: Box<Expr>;
    match equality(tokens) {
        Some(x) => expr = x,
        None => return None,
    }
    while match_head(tokens, &[TokenType::AND]) {
        let op = tokens.pop_front();
        let rexpr: Box<Expr>;
        match equality(tokens) {
            Some(x) => rexpr = x,
            None => return None,
        }
        expr = Box::new(Expr::Logical {
            left: expr,
            operator: op?,
            right: rexpr,
        })
    }
    return Some(expr);
}

fn equality(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
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
    return call(tokens);
}

fn call(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    let opt = primary(tokens);
    let mut expr: Box<Expr>;

    match opt {
        Some(x) => expr = x,
        None => return None,
    }
    loop {
        if match_head(tokens, &[TokenType::LEFT_PAREN]) {
            expr = match finish_call(tokens, expr) {
                Some(val) => val,
                None => return None,
            };
        } else {
            break;
        }
    }
    return Some(expr);
}

fn finish_call(tokens: &mut LinkedList<Token>, expr: Box<Expr>) -> Option<Box<Expr>> {
    tokens.pop_front();
    let mut args = LinkedList::<Box<Expr>>::new();
    if !match_head(tokens, &[TokenType::RIGHT_PAREN]) {
        loop {
            match expression(tokens) {
                Some(val) => args.push_back(val),
                None => return None,
            }
            if args.len() >= 255 {
                eprintln!("Can't have more than 255 arguments.");
            }
            if !match_head(tokens, &[TokenType::RIGHT_PAREN, TokenType::COMMA]) {
                eprintln!("Invalid expression call.");
                return None;
            } else if match_head(tokens, &[TokenType::RIGHT_PAREN]) {
                break;
            }
            tokens.pop_front();
        }
    }
    let p = tokens.pop_front()?;
    return Some(Box::new(Expr::Call {
        callee: expr,
        paren: p,
        arguments: args,
    }));
}

fn primary(tokens: &mut LinkedList<Token>) -> Option<Box<Expr>> {
    if match_head(tokens, &[TokenType::FALSE]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Rc::new(false),
        }));
    }
    if match_head(tokens, &[TokenType::TRUE]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Rc::new(true),
        }));
    }
    if match_head(tokens, &[TokenType::NIL]) {
        tokens.pop_front();
        return Some(Box::new(Expr::Literal {
            value: Rc::new(Option::<bool>::None),
        }));
    }
    if match_head(tokens, &[TokenType::NUMBER, TokenType::STRING]) {
        let token = tokens.pop_front().unwrap();
        return Some(Box::new(Expr::Literal {
            value: token.lexeme.clone()?,
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
    if match_head(tokens, &[TokenType::IDENTIFIER]) {
        let token = tokens.pop_front().unwrap();
        return Some(Box::new(Expr::Variable {
            name: token,
            id: get_count(),
        }));
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
    while !match_head(tokens, &[TokenType::EOF]) {
        match tokens.front().unwrap().ttype {
            TokenType::SEMICOLON => {
                tokens.pop_front();
                return;
            }
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
