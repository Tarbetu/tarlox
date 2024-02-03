mod token;
mod token_type;

pub use token::Token;
pub use token_type::TokenType;

use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

use crate::{LoxError, LoxResult};

// Our scanner is cool, but it can be improved
// For example, we don't need any string allocations
// Some methods are repeatible
// We may prefer to include whole source to our struct instead of our "chars"
// Anyway, Peakable is so cool.
pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: LoxResult<Vec<Token>>,
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

    pub async fn scan_tokens(mut self) -> LoxResult<Vec<Token>> {
        while self.chars.peek().is_some() {
            self.scan_token().await;

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

    async fn scan_token(&mut self) {
        use TokenType::*;

        if let Some(next_char) = self.chars.next() {
            match next_char {
                '(' => {
                    self.add_token(LeftParen).await;
                }
                ')' => {
                    self.add_token(RightParen).await;
                }
                '{' => {
                    self.add_token(LeftBrace).await;
                }
                '}' => {
                    self.add_token(RightBrace).await;
                }
                ',' => {
                    self.add_token(Comma).await;
                }
                '.' => {
                    self.add_token(Dot).await;
                }
                '-' => {
                    self.add_token(Minus).await;
                }
                '+' => {
                    self.add_token(Plus).await;
                }
                ';' => {
                    self.add_token(Semicolon).await;
                }
                '*' => {
                    self.add_token(Star).await;
                }
                '!' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(BangEqual).await;
                    } else {
                        self.add_token(Bang).await;
                    }
                }
                '=' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(EqualEqual).await;
                    } else {
                        self.add_token(Equal).await;
                    }
                }
                '<' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(LessEqual).await;
                    } else {
                        self.add_token(Less).await;
                    }
                }
                '>' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(GreaterEqual).await;
                    } else {
                        self.add_token(Greater).await;
                    }
                }
                '/' => {
                    if self.chars.next_if_eq(&'/').is_some() {
                        while !(self.chars.next() == Some('\n') || self.chars.peek().is_none()) {}
                    } else {
                        self.add_token(Slash).await;
                    }
                }
                ' ' | '\r' | '\t' => (),
                '\n' => self.line += 1,
                '"' => self.string().await,
                num if num.is_ascii_digit() => self.number(num).await,
                alpha if alpha.is_ascii_alphabetic() => self.identifier(alpha).await,
                unexcepted_char => {
                    self.tokens = Err(LoxError::UnexceptedCharacter {
                        line: self.line,
                        character: unexcepted_char,
                    })
                }
            }
        }
    }

    async fn add_token(&mut self, kind: TokenType) {
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

    async fn string(&mut self) {
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

        self.add_token(TokenType::LoxString(Rc::new(string))).await
    }

    async fn number(&mut self, first_digit: char) {
        let mut string = String::from(first_digit);

        loop {
            match self.chars.peek() {
                Some(x) if x.is_ascii_digit() => string.push(self.chars.next().unwrap()),
                Some('.') => {
                    if let Some('.') = self.chars.next() {
                        if self.chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                            string.push('.');
                            string.push(self.chars.next().unwrap())
                        } else {
                            self.add_token(TokenType::Dot).await;
                            break;
                        }
                    }
                }
                None | Some(_) => break,
            }
        }

        self.add_token(TokenType::Number(Rc::new(string.parse().unwrap())))
            .await
    }

    async fn identifier(&mut self, first_digit: char) {
        let mut string = String::from(first_digit);

        loop {
            match self.chars.peek() {
                Some(char) if char.is_ascii_alphanumeric() || *char == '_' => {
                    string.push(self.chars.next().unwrap())
                }
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
                _ => Identifier(Rc::new(string)),
            }
        };

        self.add_token(token_type).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astro_float::BigFloat;
    use TokenType::*;

    fn get_tokens(source: &'static str) -> LoxResult<Vec<Token>> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { Scanner::new(source).scan_tokens().await })
    }

    fn convert_tokens_into_token_types(tokens: Vec<Token>) -> Vec<TokenType> {
        tokens
            .into_iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>()
    }

    fn test_scanner(source: &'static str, mut excepted_tokens: Vec<TokenType>) {
        let tokens = get_tokens(source);

        excepted_tokens.push(EOF);

        assert_eq!(
            excepted_tokens,
            convert_tokens_into_token_types(tokens.unwrap())
        )
    }

    #[test]
    fn test_sum() {
        test_scanner(
            "2 + 2",
            vec![
                Number(Rc::new(BigFloat::from(2.0))),
                Plus,
                Number(Rc::new(BigFloat::from(2.0))),
            ],
        )
    }

    #[test]
    fn test_float_sum() {
        test_scanner(
            "2.5 + 2.5",
            vec![
                Number(Rc::new(BigFloat::from(2.5))),
                Plus,
                Number(Rc::new(BigFloat::from(2.5))),
            ],
        )
    }

    #[test]
    fn test_grouping() {
        test_scanner("()", vec![LeftParen, RightParen])
    }

    #[test]
    fn test_grouping_with_sum() {
        test_scanner(
            "(2 + 2)",
            vec![
                LeftParen,
                Number(Rc::new(BigFloat::from(2.0))),
                Plus,
                Number(Rc::new(BigFloat::from(2.0))),
                RightParen,
            ],
        )
    }

    #[test]
    fn test_grouping_float_sum() {
        test_scanner(
            "(2.5 + 2.5)",
            vec![
                LeftParen,
                Number(Rc::new(BigFloat::from(2.5))),
                Plus,
                Number(Rc::new(BigFloat::from(2.5))),
                RightParen,
            ],
        )
    }

    #[test]
    fn test_identifier() {
        test_scanner(
            "tarbetu_is_best",
            vec![Identifier(Rc::new(String::from("tarbetu_is_best")))],
        )
    }

    #[should_panic]
    #[test]
    fn wrong_identifier_with_unicode() {
        test_scanner(
            "tarbetü",
            vec![Identifier(Rc::new(String::from("tarbetü")))],
        )
    }

    #[test]
    fn test_number_with_identifier() {
        test_scanner(
            "222a",
            vec![
                Number(Rc::new(BigFloat::from(222.0))),
                Identifier(Rc::new(String::from("a"))),
            ],
        )
    }

    #[test]
    fn test_string() {
        test_scanner(
            r#""This is a cool string""#,
            vec![LoxString(Rc::new(String::from("This is a cool string")))],
        )
    }

    #[test]
    fn test_grouped_string_with_postfix() {
        test_scanner(
            r#"("This is a cool string"s)"#,
            vec![
                LeftParen,
                LoxString(Rc::new(String::from("This is a cool string"))),
                Identifier(Rc::new(String::from('s'))),
                RightParen,
            ],
        )
    }

    #[test]
    fn test_identifier_with_string() {
        test_scanner(
            r#"test"Best String!""#,
            vec![
                Identifier(Rc::new(String::from("test"))),
                LoxString(Rc::new(String::from("Best String!"))),
            ],
        )
    }

    #[test]
    fn test_for_keyword() {
        test_scanner("for", vec![For])
    }
}
