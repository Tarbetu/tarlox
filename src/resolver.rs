use ahash::AHashMap;

use crate::{
    executor::Executor,
    syntax::{Expression, Statement},
    LoxError::ParseError,
    LoxResult, Token, TokenType,
};

struct Resolver<'a> {
    executor: Executor<'a>,
    scopes: Vec<AHashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    fn resolve(&mut self, statements: &[&'a Statement]) -> LoxResult<()> {
        self.begin_scope()?;
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        self.end_scope()?;
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        use Statement::*;

        match statement {
            Print(..) => self.print_statement(statement),
            StmtExpression(..) => self.expression_statement(statement),
            Var(..) => self.var_statement(statement),
            AwaitVar(..) => self.var_statement(statement),
            Block(..) => self.block_statement(statement),
            If(..) => self.if_statement(statement),
            While(..) => self.while_statement(statement),
            Return(..) => self.return_statement(statement),
            Function(..) => self.function_statement(statement),
        }
    }

    fn resolve_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        use Expression::*;

        match expression {
            Binary(..) => self.binary_expression(expression),
            Unary(..) => self.unary_expression(expression),
            Grouping(..) => self.grouping_expression(expression),
            Logical(..) => self.logical_expression(expression),
            Variable(..) => self.variable_expression(expression),
            Assign(..) => self.assignment_expression(expression),
            Call(..) => self.call_expression(expression),
            Lambda(..) => self.lambda_expression(expression),
            Literal(..) => Ok(()),
        }
    }

    fn block_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::Block(body) = statement {
            self.begin_scope()?;
            for statement in body.as_ref() {
                self.resolve_statement(statement.as_ref())?;
            }
            self.end_scope()?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn var_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        use Statement::{AwaitVar, Var};

        macro_rules! define_and_exit {
            ($name:expr, $body:block) => {
                self.declare($name);
                $body
                self.define($name);
            };
        }

        match statement {
            AwaitVar(name, initializer) => {
                define_and_exit!(&name, {
                    self.resolve_expression(initializer)?;
                });
            }
            Var(name, initializer) => {
                define_and_exit!(&name, {
                    if let Some(expr) = initializer {
                        self.resolve_expression(expr.as_ref())?;
                    }
                });
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn variable_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        // Some(Expression::Variable(name))
        if let Expression::Variable(name) = expression {
            if let Some(scope) = self.scopes.last() {
                if scope.get(&name.to_string()) == Some(&false) {
                    return Err(ParseError {
                        line: Some(name.line),
                        msg: "Can't read local variable its own initializer".into(),
                    });
                }
            }

            self.resolve_local(expression, name)
        } else {
            unreachable!()
        }
    }

    fn assignment_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Assign(name, value) = expression {
            self.resolve_expression(value)?;
            self.resolve_local(expression, name)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn function_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::Function(name, ..) = statement {
            self.declare(name);
            self.define(name);

            self.resolve_function(statement)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn resolve_function(&mut self, function: &'a Statement) -> LoxResult<()> {
        if let Statement::Function(_, params, body) = function {
            self.begin_scope()?;

            for i in params {
                self.declare(i);
                self.define(i);
            }

            self.resolve_statement(body)?;
            self.end_scope()
        } else {
            unreachable!()
        }
    }

    fn expression_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::StmtExpression(expr) = statement {
            self.resolve_expression(expr)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn print_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::Print(expr) = statement {
            self.resolve_expression(expr)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn if_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::If(condition, then_branch, else_branch) = statement {
            self.resolve_expression(condition)?;
            self.resolve_statement(then_branch)?;

            if let Some(branch) = else_branch {
                self.resolve_statement(branch)?;
            }

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn return_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::Return(Some(expr)) = statement {
            self.resolve_expression(expr)
        } else {
            unreachable!()
        }
    }

    fn while_statement(&mut self, statement: &'a Statement) -> LoxResult<()> {
        if let Statement::While(condition, body) = statement {
            self.resolve_expression(condition)?;
            self.resolve_statement(body)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn binary_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Binary(left, _, right) = expression {
            self.resolve_expression(left)?;
            self.resolve_expression(right)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn call_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Call(callee, _, arguments) = expression {
            self.resolve_expression(callee)?;

            for argument in arguments {
                self.resolve_expression(argument)?;
            }

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn grouping_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Grouping(inner) = expression {
            self.resolve_expression(inner)
        } else {
            unreachable!()
        }
    }

    fn logical_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Logical(left, _, right) = expression {
            self.resolve_expression(left)?;
            self.resolve_expression(right)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn unary_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Unary(_, right) = expression {
            self.resolve_expression(right)
        } else {
            unreachable!()
        }
    }

    fn lambda_expression(&mut self, expression: &'a Expression) -> LoxResult<()> {
        if let Expression::Lambda(params, body) = expression {
            self.begin_scope()?;

            for i in params {
                self.declare(i);
                self.define(i);
            }

            self.resolve_statement(body)?;
            self.end_scope()
        } else {
            unreachable!()
        }
    }

    fn resolve_local(&mut self, expression: &'a Expression, name: &Token) -> LoxResult<()> {
        if let Some((index, _)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(&name.to_string()))
        {
            self.executor
                .resolve(expression, self.scopes.len() - 1 - index);
        }

        Ok(())
    }

    fn begin_scope(&mut self) -> LoxResult<()> {
        self.scopes.push(AHashMap::new());
        Ok(())
    }

    fn end_scope(&mut self) -> LoxResult<()> {
        self.scopes.pop();
        Ok(())
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let TokenType::Identifier(str) = &name.kind {
                scope.insert(str.to_string(), false);
            } else {
                unreachable!()
            }
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let TokenType::Identifier(str) = &name.kind {
                scope.insert(str.to_string(), true);
            } else {
                unreachable!()
            }
        }
    }
}
