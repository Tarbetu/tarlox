pub mod expression;

use crate::LoxResult;
use crate::Token;
use crate::TokenType;
use astro_float::BigFloat;
use expression::Expression;
use expression::LoxLiteral;
use expression::Operator;

#[derive(Debug, Copy, Clone)]
struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    fn expression(self) -> LoxResult<'a, Expression> {
        self.equality()
    }

    // These methods can be handled via macros

    fn equality(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        let mut expr = self.comparison()?;

        while self.is_match(&[BangEqual, EqualEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.comparison()?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn comparison(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        let mut expr = self.term()?;

        while self.is_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().try_into()?;
            let right = self.term()?;
            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn term(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        let mut expr = self.factor()?;

        while self.is_match(&[Minus, Plus]) {
            let operator = self.previous().try_into()?;
            let right = self.factor()?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn factor(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        let mut expr = self.factor()?;

        while self.is_match(&[Slash, Star]) {
            let operator = self.previous().try_into()?;
            let right = self.unary()?;

            expr = Expression::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn unary(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        if self.is_match(&[Bang, Minus]) {
            let operator = self.previous().try_into()?;
            let right = self.unary()?;

            return Ok(Expression::Unary(operator, right.into()));
        }

        self.primary()
    }

    fn primary(mut self) -> LoxResult<'a, Expression> {
        use TokenType::*;

        if self.is_match(&[False]) {
            return Ok(Expression::Literal(LoxLiteral::Bool(false)));
        }
        if self.is_match(&[True]) {
            return Ok(Expression::Literal(LoxLiteral::Bool(true)));
        }
        if self.is_match(&[Number(BigFloat::new(0))]) {
            unimplemented!();
        }

        if self.is_match(&[LoxString(String::new())]) {
            unimplemented!();
        }

        unimplemented!()
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
        self.tokens.get(self.current).unwrap()
    }
}
