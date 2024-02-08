pub mod expression;
pub mod minor_parse_error;
pub mod statement;

use crate::LoxError;
use crate::LoxResult;
use crate::Token;
use crate::TokenType;
pub use expression::Expression;
use expression::LoxLiteral;
use minor_parse_error::MinorParserError;
pub use statement::Statement;

use rug::Float;

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

macro_rules! return_if_cant_consume {
    ($self:expr, $token_type:expr) => {
        if let Err(err) = $self.consume($token_type) {
            return Err(err.into_lox_error($self.previous().line, None, None));
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> LoxResult<Vec<Statement>> {
        let mut statements: Vec<Statement> = vec![];

        while self.peek().is_some() {
            let program = self.declaration();
            match program {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.synchronize();
                    return Err(e);
                }
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> LoxResult<Statement> {
        use TokenType::Var;

        if self.is_match(&[Var]) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn var_declaration(&mut self) -> LoxResult<Statement> {
        let name_result = self
            .consume(TokenType::Identifier(String::new()))
            .map(|token| token.to_owned());

        if let Ok(name) = name_result {
            let mut initializer: Option<Expression> = None;

            if self.is_match(&[TokenType::Equal]) {
                initializer = Some(self.expression()?);
            }

            return_if_cant_consume!(self, TokenType::Semicolon);

            Ok(Statement::Var(name, initializer))
        } else {
            Err(name_result.unwrap_err().into_lox_error(0, None, None))
        }
    }

    fn statement(&mut self) -> LoxResult<Statement> {
        use TokenType::*;

        if self.is_match(&[Print]) {
            self.print_statement()
        } else if self.is_match(&[Ready]) {
            self.ready_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> LoxResult<Statement> {
        let expr = self.expression()?;

        return_if_cant_consume!(self, TokenType::Semicolon);

        Ok(Statement::Print(expr))
    }

    fn expression_statement(&mut self) -> LoxResult<Statement> {
        let expr = self.expression()?;

        return_if_cant_consume!(self, TokenType::Semicolon);

        Ok(Statement::StmtExpression(expr))
    }

    fn ready_statement(&mut self) -> LoxResult<Statement> {
        let expr = self.expression()?;

        return_if_cant_consume!(self, TokenType::Semicolon);

        Ok(Statement::Ready(expr))
    }

    fn expression(&mut self) -> LoxResult<Expression> {
        self.equality()
    }

    // These methods can be handled via macros

    fn equality(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.comparison()?;

        while self.is_match(&[BangEqual, EqualEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.comparison()?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.term()?;

        while self.is_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.term()?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn term(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.factor()?;

        while self.is_match(&[Minus, Plus]) {
            let operator = self.previous().try_into()?;
            let right = self.factor()?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn factor(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        let mut expr = self.unary()?;

        while self.is_match(&[Slash, Star]) {
            let operator = self.previous().try_into()?;
            let right = self.unary()?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> LoxResult<Expression> {
        use TokenType::*;

        if self.is_match(&[Bang, Minus]) {
            let operator = self.previous().try_into()?;
            let right = self.unary()?;

            return Ok(Expression::Unary(operator, right.into()));
        }

        self.primary()
    }

    fn primary(&mut self) -> LoxResult<Expression> {
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
        if self.is_match(&[Number(Float::new(2))]) {
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
        if self.is_match(&[LoxString(String::new())]) {
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
        if self.is_match(&[Identifier(String::new())]) {
            return Ok(Expression::Variable(self.previous().to_owned()));
        }
        if self.is_match(&[LeftParen]) {
            let expr = self.expression()?;

            if let Err(err) = self.consume(RightParen) {
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

    fn consume(&mut self, token_type: TokenType) -> Result<&Token, MinorParserError> {
        if self.check(&token_type) {
            self.current += 1;
            return Ok(self.previous());
        };

        Err(MinorParserError::Unmatched(token_type))
    }

    fn synchronize(&mut self) {
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
    use crate::{syntax::expression::Operator, NUMBER_PREC};

    fn create_expression(source: &str) -> LoxResult<Expression> {
        use crate::Scanner;
        Parser::new(&Scanner::new(source).scan_tokens().unwrap()).expression()
    }

    fn create_number(value: i32) -> Box<Expression> {
        Box::new(Expression::Literal(LoxLiteral::Number(Float::with_val(
            NUMBER_PREC,
            value,
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
    fn test_bang_unary_expression() {
        assert_eq!(
            create_expression("!4").unwrap(),
            Expression::Unary(Operator::Not, create_number(4))
        )
    }

    #[test]
    fn test_bang_bang_unary_expression() {
        assert_eq!(
            create_expression("!!4").unwrap(),
            Expression::Unary(
                Operator::Not,
                Box::new(Expression::Unary(Operator::Not, create_number(4)))
            )
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
