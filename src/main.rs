mod errors;
mod executor;
mod scanner;
mod standard;
mod syntax;

pub use crate::errors::LoxError;
pub use crate::errors::LoxResult;
pub use crate::scanner::{Token, TokenType};
use executor::Executor;
use scanner::Scanner;
use std::env;
use std::process;
use std::{num::NonZeroUsize, thread::available_parallelism};
use syntax::Parser;

use lazy_static::lazy_static;
use rayon::ThreadPoolBuilder;
use tokio::fs;

// pub const NUMBER_PREC: u32 = rug::float::prec_max();
pub const NUMBER_PREC: u32 = 2046;

lazy_static! {
    static ref WORKERS: rayon::ThreadPool = ThreadPoolBuilder::new()
        .num_threads(
            available_parallelism()
                .unwrap_or(NonZeroUsize::new(1).unwrap())
                .into(),
        )
        .build()
        .unwrap();
}

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
                let mut exe = Executor::new_with_env(&WORKERS, standard::globals());
                if let Err(e) = run(&source_code, &mut exe) {
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
    let mut exe = Executor::new_with_env(&WORKERS, standard::globals());

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("Tarbetu's Lox>> ");

        match readline {
            Ok(input) => {
                if input.is_empty() {
                    break;
                };

                if let Err(e) = run(&input, &mut exe) {
                    println!("{e}\n");
                };
            }
            Err(_) => break,
        }
    }
}

fn run(code: &str, exe: &mut Executor) -> LoxResult<()> {
    let expr = {
        let tokens = Scanner::new(code).scan_tokens()?;
        Parser::new(&tokens).parse()?
    };

    exe.execute(&expr)?;

    Ok(())
}
