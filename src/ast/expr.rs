use std::fmt::Display;

use crate::{
    interpreter::callable::LoxCallable,
    lexer::{Token, TokenKind},
};

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    EqEq,
    Ne,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinOp::*;
        let result = match self {
            Plus => "+",
            Minus => "-",
            Multiply => "*",
            Divide => "/",
            Eq => "=",
            Gt => ">",
            Lt => "<",
            Ge => ">=",
            Le => "<=",
            EqEq => "==",
            Ne => "!=",
        };

        write!(f, "{}", result)
    }
}

impl TryFrom<TokenKind> for BinOp {
    type Error = &'static str;
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        let op = match value {
            TokenKind::Le => Self::Le,
            TokenKind::Lt => Self::Lt,
            TokenKind::Ge => Self::Ge,
            TokenKind::Gt => Self::Gt,
            TokenKind::Eq => Self::Eq,
            TokenKind::EqEq => Self::EqEq,
            TokenKind::Plus => Self::Plus,
            TokenKind::Minus => Self::Minus,
            TokenKind::Star => Self::Multiply,
            TokenKind::Slash => Self::Divide,
            _ => return Err("Unmatchable Token"),
        };

        Ok(op)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnOp {
    BinNeg,
    LogNeg,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnOp::*;
        let result = match self {
            BinNeg => "-",
            LogNeg => "!",
        };

        write!(f, "{}", result)
    }
}

impl TryFrom<TokenKind> for UnOp {
    type Error = &'static str;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        let op = match value {
            TokenKind::Bang => Self::LogNeg,
            TokenKind::Minus => Self::BinNeg,
            _ => return Err("Unmatchable token"),
        };

        Ok(op)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LogOp {
    And,
    Or,
}

impl Display for LogOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            LogOp::And => "and",
            LogOp::Or => "or",
        };

        write!(f, "{}", result)
    }
}

impl TryFrom<TokenKind> for LogOp {
    type Error = &'static str;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        let op = match value {
            TokenKind::And => Self::And,
            TokenKind::Or => Self::Or,
            _ => return Err("Unmatchable token"),
        };

        Ok(op)
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Callable(LoxCallable),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;

        let result = match self {
            String(s) => format!("\"{}\"", s.clone()),
            Number(n) => n.to_string(),
            Bool(b) => b.to_string(),
            Callable(_) => "callable".to_string(),
            Nil => "nil".to_string(),
        };

        write!(f, "{}", result)
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical(LogOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
}
