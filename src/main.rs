extern crate lazy_static;
use std::any::Any;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error};
use std::process;

mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;
use crate::interpreter::evaluate;
use crate::parser::parser;
use crate::scanner::scan_tokens;

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
    let mut had_error = false;
    let mut l: i32 = 1;
    for line in buffered.lines() {
        run(line?, l, &mut had_error);
        if had_error {
            process::exit(0x0041)
        }
        l = l + 1;
    }
    return Ok(());
}

fn run_prompt() -> Result<(), Error> {
    let lines = io::stdin().lines();
    let mut had_error = false;
    let mut l: i32 = 1;
    for line in lines {
        run(line.unwrap(), l, &mut had_error);
        if had_error {
            had_error = false;
        }
        l = l + 1;
    }
    return Ok(());
}

fn run(source: String, line_number: i32, had_error: &mut bool) {
    let mut line: i32 = line_number;
    let mut tokens = scan_tokens(&source, &mut line);
    if tokens.len() == 1 {
        error(line_number, String::from("Invalid input"), had_error);
    }
    let expr = parser(&mut tokens);
    match expr {
        Some(x) => {
            match evaluate(*x) {
                Some(x) => println!("{}", result_to_string(x)),
                None => eprintln!("Line {}: Runtime error", line),
            };
        }
        None => eprintln!("Line {}: Semantics error", line),
    }
}

pub fn error(line: i32, message: String, had_error: &mut bool) {
    eprintln!("[Line {}] Error: {}", line, message);
    *had_error = true
}

fn result_to_string(result: Box<dyn Any>) -> String {
    if let Some(value) = result.as_ref().downcast_ref::<f64>() {
        return format!("{value}");
    }
    if let Some(value) = result.as_ref().downcast_ref::<String>() {
        return format!("{value}");
    }
    if let Some(value) = result.as_ref().downcast_ref::<bool>() {
        return format!("{value}");
    } else {
        return format!("{result:?}");
    }
}
