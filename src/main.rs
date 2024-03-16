mod errors;
mod executor;
mod resolver;
mod scanner;
mod standard;
mod syntax;

pub use crate::errors::LoxError;
pub use crate::errors::LoxResult;
pub use crate::scanner::{Token, TokenType};
use executor::Environment;
use executor::Executor;
use resolver::Resolver;
use scanner::Scanner;
use std::env;
use std::fs;
use std::process;
use std::sync::Arc;
use std::{num::NonZeroUsize, thread::available_parallelism};
use syntax::Parser;

use lazy_static::lazy_static;
use threadpool::ThreadPool;

// pub const NUMBER_PREC: u32 = rug::float::prec_max();
pub const NUMBER_PREC: u32 = 256;

lazy_static! {
    static ref WORKERS: ThreadPool = ThreadPool::new(
        available_parallelism()
            .unwrap_or(NonZeroUsize::new(1).unwrap())
            .into()
    );
    static ref GLOBALS: Arc<Environment> = standard::globals();
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
            let _ = &args.next();
            let path = &args.next().unwrap();
            if let Ok(source_code) = fs::read_to_string(path) {
                let exe = Executor::new(&WORKERS);
                let mut resolver = Resolver::new(&exe);

                if let Err(e) = run(&source_code, &mut resolver) {
                    println!("{e}");
                    process::exit(65)
                }
            } else {
                println!("File not found: {path}");
                process::exit(65)
            }

            process::exit(0);
        }
        Less => run_prompt().await,
    }
}

async fn run_prompt() {
    let exe = Executor::new(&WORKERS);
    let mut resolver = Resolver::new(&exe);

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("Tarbetu's Lox>> ");

        match readline {
            Ok(input) => {
                if input.is_empty() {
                    break;
                };

                if let Err(e) = run(&input, &mut resolver) {
                    println!("{e}\n");
                };
            }
            Err(_) => break,
        }
    }
}

fn run(code: &str, resolver: &mut Resolver) -> LoxResult<()> {
    let stmt = {
        let tokens = Scanner::new(code).scan_tokens()?;
        Parser::new(&tokens).parse()?
    };

    resolver.resolve(Arc::clone(&stmt))?;

    resolver.executor.execute(Arc::clone(&stmt))?;

    Ok(())
}
