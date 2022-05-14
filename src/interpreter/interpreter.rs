use crate::{
    ast::{
        expr::{BinOp, Expr, ExprKind, Literal, UnOp},
        stmt::Stmt,
    },
    environment::Environment,
};

#[derive(Debug)]
pub enum InterpreterErrorKind {
    General(String),
}

pub struct Interpreter<'a> {
    environment: &'a mut Environment,
}

impl<'a> Interpreter<'a> {
    pub fn new(environment: &'a mut Environment) -> Self {
        Self { environment }
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

                self.environment.define(name.lexeme.clone(), value);
            }
        };

        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, InterpreterErrorKind> {
        let lit = match &expr.kind {
            ExprKind::Literal(l) => l.clone(),
            ExprKind::Grouping(expr) => self.evaluate(&expr)?,
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
            ExprKind::Variable(name) => match self.environment.get(name) {
                Ok(val) => val.clone().to_owned(),
                Err(e) => return Err(InterpreterErrorKind::General(format!("{}", e))),
            },
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
