extern crate lazy_static;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error};
use std::process;
use std::rc::Rc;

mod callable;
mod expr;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
use crate::interpreter::interpret;
use crate::parser::parser;
use crate::resolver::resolve;
use crate::scanner::scan_tokens;
use crate::stmt::Environment;
use crate::token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        process::exit(0x0040);
    } else if args.len() == 2 {
        let _ = run_file(&args[1]);
    } else {
        let _ = run_prompt();
    }
}

fn run_file(path: &String) -> Result<(), Error> {
    let input = File::open(path)?;
    let buffered = BufReader::new(input);
    let mut l: i32 = 1;
    let env: Rc<RefCell<Environment>> = Rc::new(RefCell::new(Environment::new()));
    let mut tokens: LinkedList<Token> = LinkedList::new();
    for line in buffered.lines() {
        tokens.pop_back();
        match scan_tokens(&line?, &mut l) {
            None => {
                eprintln!("Scanning Error");
            }
            Some(mut val) => {
                tokens.append(&mut val);
            }
        }
        l = l + 1;
    }
    let result = parser(&mut tokens);
    match result {
        Some(stmts) => {
            let mut table: HashMap<u64, i32> = HashMap::new();
            let mut scopes: LinkedList<HashMap<String, bool>> = LinkedList::new();
            scopes.push_front(HashMap::<String, bool>::new());
            resolve(stmts.clone(), &mut scopes, &mut table);
            match interpret(stmts, env, &table) {
                Ok(_) => return Ok(()),
                Err(_e) => {
                    panic!("Runtime Error.");
                }
            }
        }
        None => {
            panic!("Parsing Error.");
        }
    }
}

fn run_prompt() -> Result<(), Error> {
    let lines = io::stdin().lines();
    let mut l: i32 = 1;
    let env: Rc<RefCell<Environment>> = Rc::new(RefCell::new(Environment::new()));
    let mut table: HashMap<u64, i32> = HashMap::new();
    let mut scopes: LinkedList<HashMap<String, bool>> = LinkedList::new();
    scopes.push_front(HashMap::<String, bool>::new());
    for line in lines {
        match run(line.unwrap(), l, env.clone(), &mut scopes, &mut table) {
            Err(_) => eprintln!("Error in evaluation"),
            Ok(_) => {}
        }
        l = l + 1;
    }
    return Ok(());
}

fn run(
    source: String,
    line_number: i32,
    env: Rc<RefCell<Environment>>,
    scopes: &mut LinkedList<HashMap<String, bool>>,
    table: &mut HashMap<u64, i32>,
) -> Result<(), ()> {
    let mut line: i32 = line_number;
    let mut tokens: LinkedList<Token>;
    match scan_tokens(&source, &mut line) {
        None => {
            eprintln!("Scanning Error.");
            return Err(());
        }
        Some(val) => tokens = val,
    }
    let result = parser(&mut tokens);
    match result {
        Some(stmts) => {
            resolve(stmts.clone(), scopes, table);
            match interpret(stmts, env, &table) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("Line {}: {}", line_number, e);
                    return Err(());
                }
            }
        }
        None => {
            eprintln!("Line {}: Parser error", line_number);
            return Err(());
        }
    }
}

pub fn error(line: i32, message: String) {
    eprintln!("[Line {}] Error: {}", line, message);
}
