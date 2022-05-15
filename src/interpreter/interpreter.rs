use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{
        expr::{BinOp, Expr, ExprKind, Literal, LogOp, UnOp},
        stmt::Stmt,
    },
    environment::Environment,
};

use super::callable::{Callable, Clock, LoxCallable, LoxFunction};

#[derive(Debug)]
pub enum InterpreterErrorKind {
    General(String),
    Return(Option<Literal>),
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();

        let clock = Clock {};

        globals.define(
            "clock".into(),
            Literal::Callable(LoxCallable::Other(Box::new(clock))),
        );

        let globals = Rc::new(RefCell::new(globals));
        let environment = Rc::clone(&globals);

        Self {
            environment,
            globals,
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterErrorKind> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr)?;
                println!("{}", value);
            }
            Stmt::Expression(expr) => {
                self.evaluate(&expr)?;
            }
            Stmt::Var(name, initializer) => {
                let value = if let Some(initializer) = initializer {
                    self.evaluate(initializer)?
                } else {
                    Literal::Nil
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
            }
            Stmt::Block(stmts) => {
                let environment = Rc::clone(&self.environment);
                self.execute_block(
                    &stmts,
                    Rc::new(RefCell::new(Environment::with_enclosing(environment))),
                )?;
            }
            Stmt::If(condition, then_stmt, else_stmt) => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(&then_stmt)?;
                } else {
                    if let Some(stmt) = else_stmt.as_ref() {
                        self.execute(&stmt)?;
                    }
                }
            }
            Stmt::While(condition, body) => {
                while is_truthy(&self.evaluate(condition)?) {
                    self.execute(body)?;
                }
            }
            Stmt::Function(name, params, body) => {
                let func = LoxFunction::new(
                    name.lexeme.clone(),
                    params.to_vec(),
                    body.to_vec(),
                    Rc::clone(&self.environment),
                );

                self.environment.borrow_mut().define(
                    name.lexeme.clone(),
                    Literal::Callable(LoxCallable::Function(func)),
                );
            }
            Stmt::Return(_, value) => {
                let value = match value {
                    Some(value) => Some(self.evaluate(value)?),
                    _ => None,
                };

                return Err(InterpreterErrorKind::Return(value));
            }
        };

        Ok(())
    }

    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), InterpreterErrorKind> {
        let previous = self.environment.clone();

        self.environment = environment;

        let mut err = None;
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                err.replace(e);
                break;
            }
        }

        self.environment = previous;

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, InterpreterErrorKind> {
        let lit = match &expr.kind {
            ExprKind::Literal(l) => l.clone(),
            ExprKind::Grouping(expr) => self.evaluate(&expr)?,
            ExprKind::Variable(name) => match self.environment.borrow_mut().get(name) {
                Ok(val) => val,
                Err(e) => return Err(InterpreterErrorKind::General(format!("{}", e))),
            },
            ExprKind::Assign(name, expr) => {
                let value = self.evaluate(&expr)?;
                if let Err(e) = self
                    .environment
                    .borrow_mut()
                    .assign(name.lexeme.as_str(), value.clone())
                {
                    return Err(InterpreterErrorKind::General(e));
                }

                value
            }
            ExprKind::Call(callee, arguments) => {
                let callee_v = if let Literal::Callable(callable) = self.evaluate(callee)? {
                    callable
                } else {
                    return Err(InterpreterErrorKind::General(
                        "Can only call functions and classes".into(),
                    ));
                };

                if arguments.len() != callee_v.arity() {
                    return Err(InterpreterErrorKind::General(format!(
                        "Expected {} arguments but got {}.",
                        callee_v.arity(),
                        arguments.len()
                    )));
                }

                let mut arguments_v = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    arguments_v.push(self.evaluate(argument)?);
                }

                callee_v.call(self, &arguments_v)?
            }
            ExprKind::Unary(op, expr) => {
                let right = self.evaluate(&expr)?;

                match op {
                    UnOp::BinNeg => {
                        if let Literal::Number(n) = right {
                            Literal::Number(-n)
                        } else {
                            return Err(InterpreterErrorKind::General(
                                "Operand must be a number".into(),
                            ));
                        }
                    }
                    UnOp::LogNeg => Literal::Bool(is_truthy(&right)),
                }
            }
            ExprKind::Logical(op, lhs, rhs) => {
                let left = self.evaluate(&lhs)?;

                match op {
                    LogOp::And if !is_truthy(&left) => left,
                    LogOp::Or if is_truthy(&left) => left,
                    _ => self.evaluate(rhs)?,
                }
            }
            ExprKind::Binary(op, lhs, rhs) => {
                let left = self.evaluate(&lhs)?;
                let right = self.evaluate(&rhs)?;

                match op {
                    BinOp::Minus => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Number(n1 - n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Plus => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Number(n1 + n2),
                        (Literal::String(s1), Literal::String(s2)) => {
                            Literal::String(format!("{}{}", s1, s2))
                        }
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be two numbers or two strings".into(),
                            ))
                        }
                    },
                    BinOp::Multiply => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Number(n1 * n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Divide => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Number(n1 / n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Gt => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Bool(n1 > n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Ge => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Bool(n1 >= n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Lt => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Bool(n1 < n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::Le => match (left, right) {
                        (Literal::Number(n1), Literal::Number(n2)) => Literal::Bool(n1 <= n2),
                        _ => {
                            return Err(InterpreterErrorKind::General(
                                "Operands must be numbers".into(),
                            ))
                        }
                    },
                    BinOp::EqEq => Literal::Bool(is_equal(&left, &right)),
                    BinOp::Ne => Literal::Bool(!is_equal(&left, &right)),
                    _ => unimplemented!(),
                }
            }
        };

        Ok(lit)
    }
}

fn is_truthy(val: &Literal) -> bool {
    match val {
        Literal::Nil => false,
        Literal::Bool(b) => *b,
        _ => true,
    }
}

fn is_equal(lhs: &Literal, rhs: &Literal) -> bool {
    match (lhs, rhs) {
        (Literal::Nil, Literal::Nil) => true,
        (Literal::Number(n1), Literal::Number(n2)) => n1 == n2,
        (Literal::String(s1), Literal::String(s2)) => s1 == s2,
        (Literal::Bool(b1), Literal::Bool(b2)) => b1 == b2,
        _ => false,
    }
}
