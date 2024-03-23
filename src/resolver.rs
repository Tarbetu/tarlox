use ahash::AHashMap;
use std::sync::Arc;

use crate::{
    executor::Executor,
    syntax::{Expression, Statement},
    LoxError::ParseError,
    LoxResult, Token, TokenType,
};

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a> {
    pub executor: &'a Executor,
    scopes: Vec<AHashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(executor: &'a Executor) -> Self {
        let mut result = Self {
            executor,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        };

        result.begin_scope();

        result
    }

    pub fn resolve(&mut self, statements: Arc<Vec<Arc<Statement>>>) -> LoxResult<()> {
        for statement in statements.iter() {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> LoxResult<()> {
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
            Class(..) => self.class_statement(statement),
        }
    }

    fn resolve_expression(&mut self, expression: &Expression) -> LoxResult<()> {
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
            Get(..) => self.get_expression(expression),
            Set(..) => self.set_expression(expression),
            This(..) => self.this_expression(expression),
            Super(..) => self.super_expression(expression),
            Literal(..) => Ok(()),
        }
    }

    fn block_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::Block(body) = statement {
            self.begin_scope();
            for statement in body.as_ref() {
                self.resolve_statement(statement.as_ref())?;
            }
            self.end_scope();

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn var_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        use Statement::{AwaitVar, Var};

        macro_rules! define_and_exit {
            ($name:expr, $body:block) => {
                self.declare($name)?;
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

    fn variable_expression(&mut self, expression: &Expression) -> LoxResult<()> {
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

    fn assignment_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Assign(name, value) = expression {
            self.resolve_expression(value)?;
            self.resolve_local(expression, name)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn function_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::Function(name, ..) = statement {
            self.declare(name)?;
            self.define(name);

            self.resolve_function(statement, FunctionType::Function)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn resolve_function(&mut self, function: &Statement, f_type: FunctionType) -> LoxResult<()> {
        if let Statement::Function(_, params, body) = function {
            let enclosing_function = self.current_function;
            self.current_function = f_type;

            self.begin_scope();

            for i in params {
                self.declare(i)?;
                self.define(i);
            }

            self.resolve_statement(body)?;
            self.end_scope();

            self.current_function = enclosing_function;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn class_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::Class(name, superclass, methods) = statement {
            let enclosing_class = self.current_class;
            self.current_class = ClassType::Class;
            self.declare(name)?;
            self.define(name);

            if let Some(Expression::Variable(superclass_name)) =
                superclass.as_ref().map(|arc| arc.as_ref())
            {
                if name.to_string() == superclass_name.to_string() {
                    return Err(ParseError {
                        line: Some(superclass_name.line),
                        msg: "A class can't inherit from itself.".into(),
                    });
                }
            }

            if let Some(superclass) = superclass {
                self.resolve_expression(superclass)?;
                self.scopes
                    .last_mut()
                    .and_then(|scope| scope.insert(format!("{:?}", TokenType::Super), true));
            }

            self.begin_scope();
            self.scopes
                .last_mut()
                .and_then(|scope| scope.insert(format!("{:?}", TokenType::This), true));

            for method in methods {
                let mut declaration = FunctionType::Method;

                if let Statement::Function(
                    Token {
                        kind: TokenType::Identifier(name),
                        ..
                    },
                    ..,
                ) = method
                {
                    if name == "init" {
                        declaration = FunctionType::Initializer;
                    }
                }

                self.resolve_function(method, declaration)?
            }
            self.end_scope();

            self.current_class = enclosing_class;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn expression_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::StmtExpression(expr) = statement {
            self.resolve_expression(expr)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn print_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::Print(expr) = statement {
            self.resolve_expression(expr)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn if_statement(&mut self, statement: &Statement) -> LoxResult<()> {
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

    fn return_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::Return(Some(expr)) = statement {
            if let FunctionType::None = self.current_function {
                Err(ParseError {
                    line: None,
                    msg: "Can't return from top-level code.".into(),
                })
            } else if let FunctionType::Initializer = self.current_function {
                Err(ParseError {
                    line: None,
                    msg: "Can't return inside from initializer".into(),
                })
            } else {
                self.resolve_expression(expr)
            }
        } else {
            Ok(())
        }
    }

    fn this_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::This(keyword) = expression {
            if let ClassType::None = self.current_class {
                Err(ParseError {
                    line: None,
                    msg: "Can't use 'this' outside of a class.".into(),
                })
            } else {
                self.resolve_local(expression, keyword)
            }
        } else {
            unreachable!()
        }
    }

    fn super_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Super(keyword, _) = expression {
            self.resolve_local(expression, keyword)
        } else {
            unreachable!()
        }
    }

    fn while_statement(&mut self, statement: &Statement) -> LoxResult<()> {
        if let Statement::While(condition, body) = statement {
            self.resolve_expression(condition)?;
            self.resolve_statement(body)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn binary_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Binary(left, _, right) = expression {
            self.resolve_expression(left)?;
            self.resolve_expression(right)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn call_expression(&mut self, expression: &Expression) -> LoxResult<()> {
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

    fn get_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Get(object, ..) = expression {
            self.resolve_expression(object)
        } else {
            unreachable!()
        }
    }

    fn set_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Set(object, _name, value) = expression {
            self.resolve_expression(value)?;
            self.resolve_expression(object)
        } else {
            unreachable!()
        }
    }

    fn grouping_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Grouping(inner) = expression {
            self.resolve_expression(inner)
        } else {
            unreachable!()
        }
    }

    fn logical_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Logical(left, _, right) = expression {
            self.resolve_expression(left)?;
            self.resolve_expression(right)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn unary_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Unary(_, right) = expression {
            self.resolve_expression(right)
        } else {
            unreachable!()
        }
    }

    fn lambda_expression(&mut self, expression: &Expression) -> LoxResult<()> {
        if let Expression::Lambda(params, body) = expression {
            for i in params {
                self.declare(i)?;
                self.define(i);
            }

            self.resolve_statement(body)?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn resolve_local(&self, expression: &Expression, name: &Token) -> LoxResult<()> {
        if let Some((index, _)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name.to_string().as_str()))
        {
            self.executor
                .resolve(name.id, expression, self.scopes.len() - 1 - index);
        }

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(AHashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> LoxResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.to_string().as_str()) {
                Err(ParseError {
                    line: Some(name.line),
                    msg: "Already a variable with this name in this scope.".into(),
                })
            } else {
                scope.insert(name.to_string(), false);
                Ok(())
            }
        } else {
            unreachable!()
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        } else {
            unreachable!()
        }
    }
}

impl Drop for Resolver<'_> {
    fn drop(&mut self) {
        self.end_scope()
    }
}
