use std::fmt::Display;

pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinOp::*;
        let result = match self {
            Plus => "+",
            Minus => "-",
            Multiply => "*",
            Divide => "/",
        };

        write!(f, "{}", result)
    }
}

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

pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;

        let result = match self {
            String(s) => s.clone(),
            Number(n) => n.to_string(),
            Bool(b) => b.to_string(),
            Nil => "nil".to_string(),
        };

        write!(f, "{}", result)
    }
}

pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(UnOp, Box<Expr>),
}

pub struct Expr {
    pub kind: ExprKind,
}
