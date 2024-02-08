mod errors;
mod executor;
mod scanner;
mod syntax;

pub use crate::errors::LoxError;
pub use crate::errors::LoxResult;
pub use crate::scanner::{Token, TokenType};
use scanner::Scanner;
use std::env;
use std::process;
use syntax::Parser;
use tokio::fs;

// pub const NUMBER_PREC: u32 = rug::float::prec_max();
pub const NUMBER_PREC: u32 = 2046;

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
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("Tarbetu's Lox>> ");

        match readline {
            Ok(input) => {
                if input.is_empty() {
                    break;
                };

                if let Err(e) = run(&input).await {
                    println!("{e}\n");
                };
            }
            Err(_) => break,
        }
    }
}

async fn run(code: &str) -> LoxResult<()> {
    let expr = {
        let tokens = Scanner::new(code).scan_tokens()?;
        Parser::new(&tokens).parse()?
    };

    executor::interpret(expr).await?;

    Ok(())
}
