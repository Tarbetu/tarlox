mod token;
mod token_type;

pub use token::Token;
pub use token_type::TokenType;

use std::iter::Peekable;
use std::str::Chars;

use rug::Float;

use crate::{LoxError, LoxResult, NUMBER_PREC};

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

    pub fn scan_tokens(mut self) -> LoxResult<Vec<Token>> {
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
                        self.add_token(BangEqual);
                    } else {
                        self.add_token(Bang);
                    }
                }
                '=' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(EqualEqual);
                    } else {
                        self.add_token(Equal);
                    }
                }
                '<' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(LessEqual);
                    } else {
                        self.add_token(Less);
                    }
                }
                '>' => {
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.add_token(GreaterEqual);
                    } else {
                        self.add_token(Greater);
                    }
                }
                '/' => {
                    if self.chars.next_if_eq(&'/').is_some() {
                        while !(self.chars.next() == Some('\n') || self.chars.peek().is_none()) {}
                    } else {
                        self.add_token(Slash);
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

        self.add_token(TokenType::LoxString(string))
    }

    fn number(&mut self, first_digit: char) {
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
                            self.add_token(TokenType::Dot);
                            break;
                        }
                    }
                }
                None | Some(_) => break,
            }
        }

        self.add_token(TokenType::Number(Float::with_val(
            NUMBER_PREC,
            Float::parse(string).unwrap(),
        )))
    }

    fn identifier(&mut self, first_digit: char) {
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
                "is_ready" => IsReady,
                _ => Identifier(string),
            }
        };

        self.add_token(token_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rug::Float;
    use TokenType::*;

    fn get_tokens(source: &'static str) -> LoxResult<Vec<Token>> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { Scanner::new(source).scan_tokens() })
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
                Number(Float::with_val(NUMBER_PREC, 2.0)),
                Plus,
                Number(Float::with_val(NUMBER_PREC, 2.0)),
            ],
        )
    }

    #[test]
    fn test_float_sum() {
        test_scanner(
            "2.5 + 2.5",
            vec![
                Number(Float::with_val(NUMBER_PREC, 2.5)),
                Plus,
                Number(Float::with_val(NUMBER_PREC, 2.5)),
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
                Number(Float::with_val(NUMBER_PREC, 2.0)),
                Plus,
                Number(Float::with_val(NUMBER_PREC, 2.0)),
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
                Number(Float::with_val(NUMBER_PREC, 2.5)),
                Plus,
                Number(Float::with_val(NUMBER_PREC, 2.5)),
                RightParen,
            ],
        )
    }

    #[test]
    fn test_identifier() {
        test_scanner(
            "tarbetu_is_best",
            vec![Identifier(String::from("tarbetu_is_best"))],
        )
    }

    #[should_panic]
    #[test]
    fn wrong_identifier_with_unicode() {
        test_scanner("tarbetü", vec![Identifier(String::from("tarbetü"))])
    }

    #[test]
    fn test_number_with_identifier() {
        test_scanner(
            "222a",
            vec![
                Number(Float::with_val(NUMBER_PREC, 222.0)),
                Identifier(String::from("a")),
            ],
        )
    }

    #[test]
    fn test_string() {
        test_scanner(
            r#""This is a cool string""#,
            vec![LoxString(String::from("This is a cool string"))],
        )
    }

    #[test]
    fn test_grouped_string_with_postfix() {
        test_scanner(
            r#"("This is a cool string"s)"#,
            vec![
                LeftParen,
                LoxString(String::from("This is a cool string")),
                Identifier(String::from('s')),
                RightParen,
            ],
        )
    }

    #[test]
    fn test_identifier_with_string() {
        test_scanner(
            r#"test"Best String!""#,
            vec![
                Identifier(String::from("test")),
                LoxString(String::from("Best String!")),
            ],
        )
    }

    #[test]
    fn test_bang() {
        test_scanner("!bang", vec![Bang, Identifier(String::from("bang"))])
    }

    #[test]
    fn test_bang_bang() {
        test_scanner("!!bang", vec![Bang, Bang, Identifier(String::from("bang"))])
    }

    #[test]
    fn test_for_keyword() {
        test_scanner("for", vec![For])
    }
}
