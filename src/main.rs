mod errors;
mod executor;
mod scanner;
mod syntax;

pub use crate::errors::LoxError;
pub use crate::errors::LoxResult;
pub use crate::scanner::{Token, TokenType};
use executor::Interpreter;
use scanner::Scanner;
use std::env;
use std::io;
use std::io::Write;
use std::process;
use syntax::Parser;
use tokio::fs;

#[tokio::main]
async fn main() {
    let mut args = env::args();

    use std::cmp::Ordering::*;
    match args.len().cmp(&2) {
        Greater => {
            println!("Usage: tlox [script]");
            process::exit(64);
        }
        Equal => {
            let path = &args.next().unwrap();
            if let Ok(source_code) = fs::read_to_string(path).await {
                if let Err(e) = run(&source_code).await {
                    println!("{e}");
                    process::exit(65)
                }
            } else {
                println!("File not found");
                process::exit(65)
            }

            process::exit(0);
        }
        Less => run_prompt().await,
    }
}

async fn run_prompt() {
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

        if let Err(e) = run(&input).await {
            println!("{e}\n");
        };

        input.clear();

        print!("Tarbetu's Lox>> ");
        Write::flush(&mut stdout).expect("Can't flush stdout!");
    }
}

async fn run(code: &str) -> LoxResult<()> {
    let expr = {
        let tokens = Scanner::new(code).scan_tokens().await?;
        Parser::new(&tokens).expression().await?
    };

    Interpreter::interpret(expr).await;

    Ok(())
}
