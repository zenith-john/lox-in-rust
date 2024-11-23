extern crate lazy_static;
use std::env;
use std::process;
use std::fs::File;
use std::io;
use std::io::{Write, BufReader, BufRead, Error};

mod scanner;
mod token;
mod expr;
use crate::scanner::scan_tokens;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        process::exit(0x0040);
    }
    else if args.len() == 2 {
        let _ = run_file(&args[1]);
    }
    else {
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
    let tokens = scan_tokens(&source, &mut line);
    if tokens.len() == 1 {
        error(line_number, String::from("Invalid input"), had_error);
    }
    for token in tokens {
        println!("{:?}", token);
    }
}

pub fn error(line: i32, message: String, had_error: &mut bool) {
    eprintln!("[Line {}] Error: {}", line, message);
    *had_error = true
}
