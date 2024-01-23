mod token;
mod token_type;

use token::Token;
use token_type::TokenType;

use std::iter::Peekable;
use std::str::Chars;

use crate::{LoxError, LoxResult};

// Our scanner is cool, but it can be improved
// For example, we don't need any string allocations
// Some methods are repeatible
// We may prefer to include whole source to our struct instead of our "chars"
// Anyway, Peakable is so cool.
pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: LoxResult<'a, Vec<Token>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: Ok(Vec::with_capacity(source.len())),
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> LoxResult<'a, Vec<Token>> {
        while self.chars.peek().is_some() {
            self.scan_token();

            if self.tokens.is_err() {
                break;
            }
        }

        if self.tokens.is_ok() {
            self.tokens.as_mut().unwrap().push(Token {
                kind: TokenType::EOF,
                line: self.line,
            });
        }

        self.tokens
    }

    fn scan_token(&mut self) {
        use TokenType::*;

        if let Some(next_char) = self.chars.next() {
            match next_char {
                '(' => {
                    self.add_token(LeftParen);
                }
                ')' => {
                    self.add_token(RightParen);
                }
                '{' => {
                    self.add_token(LeftBrace);
                }
                '}' => {
                    self.add_token(RightBrace);
                }
                ',' => {
                    self.add_token(Comma);
                }
                '.' => {
                    self.add_token(Dot);
                }
                '-' => {
                    self.add_token(Minus);
                }
                '+' => {
                    self.add_token(Plus);
                }
                ';' => {
                    self.add_token(Semicolon);
                }
                '*' => {
                    self.add_token(Star);
                }
                '!' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(BangEqual)
                    } else {
                        self.add_token(Bang)
                    }
                }
                '=' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(EqualEqual)
                    } else {
                        self.add_token(Equal)
                    }
                }
                '<' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(LessEqual)
                    } else {
                        self.add_token(Less)
                    }
                }
                '>' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(GreaterEqual)
                    } else {
                        self.add_token(Greater)
                    }
                }
                '/' => {
                    if self.chars.next_if_eq(&'/').is_some() {
                        while !(self.chars.next() == Some('\n') || self.chars.peek().is_none()) {}
                    } else {
                        self.add_token(Slash)
                    }
                }
                ' ' | '\r' | '\t' => (),
                '\n' => self.line += 1,
                '"' => self.string(),
                num if num.is_ascii_digit() => self.number(num),
                alpha if alpha.is_ascii_alphabetic() => self.identifier(alpha),
                unexcepted_char => {
                    self.tokens = Err(LoxError::UnexceptedCharacter {
                        line: self.line,
                        character: unexcepted_char,
                    })
                }
            }
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        if let Ok(tokens) = &mut self.tokens {
            tokens.push(Token {
                kind,
                line: self.line,
            })
        }
    }

    // Prefer macros for string, number and identifier
    // Also, we don't need any String allocation.
    // This is easy for now, but should be replaced with substrings.

    fn string(&mut self) {
        let mut string = String::new();

        loop {
            let next_char = self.chars.next();

            match next_char {
                Some('"') => break,
                Some('\n') => self.line += 1,
                Some(char) => string.push(char),
                None => {
                    self.tokens = Err(LoxError::UnterminatedString);

                    return;
                }
            }
        }

        self.add_token(TokenType::String(string))
    }

    fn number(&mut self, first_digit: char) {
        let mut string = String::from(first_digit);

        loop {
            let next_char = self.chars.next();

            match next_char {
                Some('.') if self.chars.peek().is_some_and(|&c| c.is_ascii_digit()) => {
                    string.push('.')
                }
                Some(x) if x.is_ascii_digit() => string.push(x),
                None | Some(_) => break,
            }
        }

        self.add_token(TokenType::Number(string.parse().unwrap()))
    }

    fn identifier(&mut self, first_digit: char) {
        let mut string = String::from(first_digit);

        loop {
            let next_char = self.chars.next();

            match next_char {
                Some(char) if char.is_ascii_alphanumeric() => string.push(char),
                None | Some(_) => break,
            }
        }

        let token_type = {
            use TokenType::*;
            match string.as_str() {
                "and" => And,
                "class" => Class,
                "else" => Else,
                "false" => False,
                "for" => For,
                "fun" => Fun,
                "if" => If,
                "nil" => Nil,
                "or" => Or,
                "print" => Print,
                "return" => Return,
                "super" => Super,
                "this" => This,
                "true" => True,
                "var" => Var,
                "while" => While,
                _ => Identifier(string),
            }
        };

        self.add_token(token_type)
    }
}
