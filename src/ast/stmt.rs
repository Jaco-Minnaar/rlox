use crate::lexer::Token;

use super::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
}
