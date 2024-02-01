mod errors;
mod scanner;
mod syntax;

pub use crate::errors::LoxError;
pub use crate::errors::LoxResult;
pub use crate::scanner::{Token, TokenType};
use scanner::Scanner;
use std::io;
use std::io::Write;
use std::process;
use std::{env, fs};
use syntax::Parser;

fn main() {
    let mut args = env::args();

    use std::cmp::Ordering::*;
    match args.len().cmp(&2) {
        Greater => {
            println!("Usage: tlox [script]");
            process::exit(64);
        }
        Equal => {
            let path = &args.next().unwrap();
            if let Ok(source_code) = fs::read_to_string(path) {
                if let Err(e) = run(&source_code) {
                    println!("{e}");
                    process::exit(65)
                }
            } else {
                println!("File not found");
                process::exit(65)
            }

            process::exit(0);
        }
        Less => run_prompt(),
    }
}

fn run_prompt() {
    let mut input = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // This looks very clumsy. It's repeated inside the loop.
    print!("Tarbetu's Lox>> ");
    Write::flush(&mut stdout).expect("Can't flush stdout!");
    while stdin.read_line(&mut input).is_ok() {
        if input.is_empty() {
            break;
        };

        if let Err(e) = run(&input) {
            println!("{e}\n");
        };

        input.clear();

        print!("Tarbetu's Lox>> ");
        Write::flush(&mut stdout).expect("Can't flush stdout!");
    }
}

fn run(code: &str) -> LoxResult<()> {
    let tokens = Scanner::new(code).scan_tokens()?;
    let expr = Parser::new(&tokens).expression()?;

    println!("{expr}");

    Ok(())
}
