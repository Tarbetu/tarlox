pub mod expression;
pub mod statement;

use crate::LoxError;
use crate::LoxResult;
use crate::Token;
use crate::TokenType;
pub use expression::Expression;
use expression::LoxLiteral;
pub use statement::Statement;

use rug::Float;

use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> LoxResult<Arc<Vec<Arc<Statement>>>> {
        let mut statements = vec![];

        while self.peek().is_some() {
            let program = self.declaration();
            match program {
                Ok(stmt) => statements.push(Arc::new(stmt)),
                Err(e) => {
                    self.synchronize();
                    return Err(e);
                }
            }
        }

        Ok(Arc::new(statements))
    }

    fn declaration(&mut self) -> LoxResult<Statement> {
        use TokenType::{AwaitVar, Fun, Var};

        if self.is_match(&[Fun]) {
            self.function("function")
        } else if self.is_match(&[Var]) {
            self.var_declaration(Var)
        } else if self.is_match(&[AwaitVar]) {
            self.var_declaration(AwaitVar)
        } else {
            self.statement()
        }
    }

    // Convert 'kind' parameter to enum if needed
    fn function(&mut self, kind: &str) -> LoxResult<Statement> {
        use TokenType::{Comma, Identifier, LeftBrace, LeftParen, RightParen};

        let name = self
            .consume(
                Identifier(String::new()),
                Some(format!("Except {kind} name")),
            )?
            .to_owned();

        self.consume(LeftParen, Some(format!("Except '(' after {kind} name")))?;

        let parameters = {
            let mut result = vec![];

            if !self.check(&RightParen) {
                result.push(
                    self.consume(
                        Identifier(String::new()),
                        Some("Except parameter name.".into()),
                    )?
                    .to_owned(),
                );

                while self.is_match(&[Comma]) {
                    if result.len() >= 255 {
                        return Err(LoxError::RuntimeError {
                            line: Some(self.previous().line),
                            msg: "Can't have more than 255 parameters.".into(),
                        });
                    }

                    result.push(
                        self.consume(
                            Identifier(String::new()),
                            Some("Except parameter name.".into()),
                        )
                        .map(|i| i.to_owned())?,
                    )
                }
            }

            self.consume(RightParen, Some("Except ')' after parameters.".into()))?;

            result
        };

        self.consume(LeftBrace, Some(format!("Expect '{{' before {kind} body.")))?;

        let body = self.block_statement()?;

        Ok(Statement::Function(name, parameters, body.into()))
    }

    fn var_declaration(&mut self, token_type: TokenType) -> LoxResult<Statement> {
        use TokenType::Semicolon;

        let name_result = self
            .consume(TokenType::Identifier(String::new()), None)
            .map(|token| token.to_owned());

        if let Ok(name) = name_result {
            let mut initializer: Option<Expression> = None;

            if self.is_match(&[TokenType::Equal]) {
                initializer = Some(self.expression()?);
            }

            self.consume(Semicolon, None)?;

            use TokenType::{AwaitVar, Var};
            match token_type {
                Var => Ok(Statement::Var(name, initializer.map(Arc::new))),
                AwaitVar => match initializer {
                    Some(init) => Ok(Statement::AwaitVar(name, init)),
                    None => Err(LoxError::ParseError {
                        line: Some(name.line),
                        msg: "Await expects initializer. Use 'Var' syntax without initializer."
                            .into(),
                    }),
                },
                _ => unreachable!(),
            }
        } else {
            Err(name_result.unwrap_err())
        }
    }

    fn statement(&mut self) -> LoxResult<Statement> {
        use TokenType::*;

        if self.is_match(&[For]) {
            self.for_statement()
        } else if self.is_match(&[If]) {
            self.if_statement()
        } else if self.is_match(&[Print]) {
            self.print_statement()
        } else if self.is_match(&[Return]) {
            self.return_statement()
        } else if self.is_match(&[While]) {
            self.while_statement()
        } else if self.is_match(&[LeftBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> LoxResult<Statement> {
        use TokenType::*;

        self.consume(LeftParen, None)?;

        let initializer = {
            if self.is_match(&[Semicolon]) {
                None
            } else if self.is_match(&[Var, AwaitVar]) {
                Some(self.var_declaration(AwaitVar)?)
            } else {
                Some(self.expression_statement()?)
            }
        };

        let condition = {
            if !self.check(&Semicolon) {
                Some(self.expression()?)
            } else {
                None
            }
        };

        self.consume(Semicolon, None)?;

        let increment = {
            if !self.check(&RightParen) {
                Some(self.expression()?)
            } else {
                None
            }
        };

        self.consume(RightParen, None)?;

        let mut body = self.statement()?;

        if let Some(expr) = increment {
            body = Statement::Block(Arc::new(vec![
                Arc::new(body),
                Arc::new(Statement::StmtExpression(expr)),
            ]));
        }

        if let Some(expr) = condition {
            body = Statement::While(expr, body.into())
        }

        if let Some(stmt) = initializer {
            body = Statement::Block(Arc::new(vec![Arc::new(stmt), Arc::new(body)]))
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> LoxResult<Statement> {
        use TokenType::*;

        self.consume(LeftParen, None)?;

        let condition = self.expression()?;

        self.consume(RightParen, None)?;

        let then_branch = self.statement()?;

        let else_branch = {
            if self.is_match(&[Else]) {
                Some(self.statement()?)
            } else {
                None
            }
        };

        Ok(Statement::If(
            condition,
            then_branch.into(),
            else_branch.map(|i| i.into()),
        ))
    }

    fn print_statement(&mut self) -> LoxResult<Statement> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, None)?;

        Ok(Statement::Print(expr))
    }

    fn return_statement(&mut self) -> LoxResult<Statement> {
        use TokenType::Semicolon;

        let value = {
            if !self.check(&Semicolon) {
                Some(self.expression()?)
            } else {
                None
            }
        };

        self.consume(Semicolon, Some("Need ';' after return value".into()))?;

        Ok(Statement::Return(value))
    }

    fn while_statement(&mut self) -> LoxResult<Statement> {
        use TokenType::{LeftParen, RightParen};

        self.consume(LeftParen, None)?;
        let condition = self.expression()?;
        self.consume(RightParen, None)?;
        let body = self.statement()?;

        Ok(Statement::While(condition, body.into()))
    }

    fn block_statement(&mut self) -> LoxResult<Statement> {
        use TokenType::RightBrace;

        let mut statements = vec![];

        while !self.check(&RightBrace) && self.peek().is_some() {
            statements.push(Arc::new(self.declaration()?));
        }

        self.consume(RightBrace, None)?;

        Ok(Statement::Block(Arc::new(statements)))
    }

    fn expression_statement(&mut self) -> LoxResult<Statement> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, None)?;

        Ok(Statement::StmtExpression(expr))
    }

    fn expression(&mut self) -> LoxResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> LoxResult<Expression> {
        use TokenType::Equal;

        let expr = self.or()?;

        if self.is_match(&[Equal]) {
            let value = self.assignment()?;

            if let Expression::Variable(name) = expr {
                return Ok(Expression::Assign(name, value.into()));
            } else {
                return Err(LoxError::ParseError {
                    line: Some(self.previous().line),
                    msg: "Invalid assignment target.".to_string(),
                });
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> LoxResult<Expression> {
        use TokenType::Or;

        let mut expr = self.and()?;

        while self.is_match(&[Or]) {
            let operator = self.previous().try_into()?;

            let right = self.and()?;
            expr = Expression::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn and(&mut self) -> LoxResult<Expression> {
        use TokenType::And;

        let mut expr = self.equality()?;

        while self.is_match(&[And]) {
            let operator = self.previous().try_into()?;

            let right = self.equality()?;
            expr = Expression::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

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

        if self.is_match(&[Bang, Minus, IsReady]) {
            let operator = self.previous().try_into()?;
            let right = self.unary()?;

            return Ok(Expression::Unary(operator, right.into()));
        }

        self.call()
    }

    fn call(&mut self) -> LoxResult<Expression> {
        let mut expr = self.primary()?;

        loop {
            if self.is_match(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> LoxResult<Expression> {
        use TokenType::{Comma, RightParen};

        let mut arguments = vec![];

        if !self.check(&RightParen) {
            arguments.push(self.expression()?);

            while self.is_match(&[Comma]) {
                if arguments.len() >= 255 {
                    return Err(LoxError::RuntimeError {
                        line: Some(self.previous().line),
                        msg: "Can't have more than 255 arguments.".to_string(),
                    });
                }
                arguments.push(self.expression()?)
            }
        }

        let paren = self.consume(RightParen, None)?.to_owned();

        Ok(Expression::Call(callee.into(), paren, arguments))
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
                _ => return Err(LoxError::InternalError("Error while parsing Number".into())),
            };
            return Ok(Expression::Literal(LoxLiteral::Number(num)));
        }
        if self.is_match(&[LoxString(String::new())]) {
            let str = match self.previous().kind.clone() {
                LoxString(s) => s,
                _ => return Err(LoxError::InternalError("Error while parsing String".into())),
            };
            return Ok(Expression::Literal(LoxLiteral::LoxString(str)));
        }
        if self.is_match(&[Identifier(String::new())]) {
            return Ok(Expression::Variable(self.previous().to_owned()));
        }
        if self.is_match(&[LeftParen]) {
            let expr = self.expression()?;

            self.consume(RightParen, None)?;

            return Ok(Expression::Grouping(Box::new(expr)));
        }

        Err(LoxError::ExceptedExpression(if self.current == 0 {
            0
        } else {
            self.previous().line
        }))
    }

    fn consume(&mut self, token_type: TokenType, msg: Option<String>) -> LoxResult<&Token> {
        if self.check(&token_type) {
            self.current += 1;
            return Ok(self.previous());
        };

        Err(LoxError::ParseError {
            line: Some(self.previous().line),
            msg: msg.unwrap_or(format!("{:?} not found", token_type)),
        })
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

    fn create_statement(source: &str) -> LoxResult<Statement> {
        use crate::Scanner;
        Parser::new(&Scanner::new(source).scan_tokens().unwrap()).declaration()
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

    // #[test]
    // fn test_not_expression() {
    //     assert!(create_expression("Tarbetu is best!").is_err())
    // }

    #[test]
    fn test_block_statement() {
        assert_eq!(
            create_statement("{}").unwrap(),
            Statement::Block(Arc::new(vec![]))
        )
    }

    #[test]
    fn test_call() {
        use TokenType::{Identifier, RightParen};

        assert_eq!(
            create_expression("try_call()").unwrap(),
            Expression::Call(
                Expression::Variable(Token {
                    line: 1,
                    kind: Identifier("try_call".into())
                })
                .into(),
                Token {
                    line: 1,
                    kind: RightParen
                },
                vec![]
            )
        )
    }

    #[test]
    fn test_function_declaration() {
        use TokenType::Identifier;

        assert_eq!(
            create_statement("fun this_is_function() {}").unwrap(),
            Statement::Function(
                Token {
                    line: 1,
                    kind: Identifier("this_is_function".to_string())
                },
                vec![],
                Statement::Block(Arc::new(vec![])).into()
            )
        )
    }
}
