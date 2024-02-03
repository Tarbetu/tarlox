pub mod expression;
pub mod minor_parse_error;

use crate::LoxError;
use crate::LoxResult;
use crate::Token;
use crate::TokenType;
use astro_float::BigFloat;
use async_recursion::async_recursion;
pub use expression::Expression;
use expression::LoxLiteral;
use minor_parse_error::MinorParserError;

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    #[async_recursion(?Send)]
    pub async fn expression(&mut self) -> LoxResult<Expression> {
        self.equality().await
    }

    // These methods can be handled via macros

    async fn equality(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.comparison().await?;

        while self.is_match(&[BangEqual, EqualEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.comparison().await?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    async fn comparison(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.term().await?;

        while self.is_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.term().await?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    async fn term(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.factor().await?;

        while self.is_match(&[Minus, Plus]) {
            let operator = self.previous().try_into()?;
            let right = self.factor().await?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    async fn factor(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.unary().await?;

        while self.is_match(&[Slash, Star]) {
            let operator = self.previous().try_into()?;
            let right = self.unary().await?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    async fn unary(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        if self.is_match(&[Bang, Minus]) {
            let operator = self.previous().try_into()?;
            let right = self.primary().await?;

            return Ok(Expression::Unary(operator, right.into()));
        }

        self.primary().await
    }

    async fn primary(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        if self.is_match(&[False]) {
            return Ok(Expression::Literal(LoxLiteral::Bool(false)));
        }
        if self.is_match(&[True]) {
            return Ok(Expression::Literal(LoxLiteral::Bool(true)));
        }
        if self.is_match(&[Nil]) {
            return Ok(Expression::Literal(LoxLiteral::Nil));
        }
        if self.is_match(&[Number(BigFloat::new(0).into())]) {
            let num = match self.previous().kind.clone() {
                Number(x) => x,
                _ => {
                    return Err(LoxError::InternalParsingError(
                        "Error while parsing Number".into(),
                    ))
                }
            };
            return Ok(Expression::Literal(LoxLiteral::Number(num)));
        }
        if self.is_match(&[LoxString(String::new().into())]) {
            let str = match self.previous().kind.clone() {
                LoxString(s) => s,
                _ => {
                    return Err(LoxError::InternalParsingError(
                        "Error while parsing String".into(),
                    ))
                }
            };
            return Ok(Expression::Literal(LoxLiteral::LoxString(str)));
        }
        if self.is_match(&[LeftParen]) {
            let expr = self.expression().await?;

            if let Err(err) = self.consume(RightParen).await {
                return Err(err.into_lox_error(self.previous().line, None, None));
            }

            return Ok(Expression::Grouping(Box::new(expr)));
        }

        Err(LoxError::ExceptedExpression(if self.current == 0 {
            0
        } else {
            self.previous().line
        }))
    }

    async fn consume(&mut self, token_type: TokenType) -> Result<(), MinorParserError> {
        if self.check(&token_type) {
            self.current += 1;
            return Ok(());
        };

        Err(MinorParserError::Unmatched(token_type))
    }

    async fn synchronize(&mut self) {
        use TokenType::*;

        self.advance();

        while let Some(val) = self.peek() {
            if self.previous().kind == Semicolon {
                return;
            };

            match val.kind {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn is_match(&mut self, token_types: &[TokenType]) -> bool {
        token_types
            .iter()
            .any(|x| self.check(x))
            .then(|| {
                self.advance();
            })
            .is_some()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        use std::mem::discriminant;

        self.peek()
            .is_some_and(|res| discriminant(&res.kind) == discriminant(token_type))
    }

    fn advance(&mut self) -> &Token {
        if self.peek().is_some() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> Option<&Token> {
        match self.tokens.get(self.current) {
            Some(x) if x.kind == TokenType::EOF => None,
            None => None,
            Some(x) => Some(x),
        }
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::expression::Operator;
    use std::rc::Rc;

    fn create_expression(source: &str) -> LoxResult<Expression> {
        use crate::Scanner;
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            Parser::new(&Scanner::new(source).scan_tokens().await.unwrap())
                .expression()
                .await
        })
    }

    fn create_number(value: i32) -> Box<Expression> {
        Box::new(Expression::Literal(LoxLiteral::Number(Rc::new(
            value.into(),
        ))))
    }

    #[test]
    fn test_minus_unary_expression() {
        assert_eq!(
            create_expression("-4").unwrap(),
            Expression::Unary(Operator::Minus, create_number(4))
        )
    }

    #[test]
    fn test_plus_unary_expression() {
        assert!(create_expression("+4").is_err())
    }

    #[test]
    fn test_not_expression() {
        assert!(create_expression("Tarbetu is best!").is_err())
    }
}
