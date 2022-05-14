use crate::{ast::stmt::Stmt, lexer::Token};

use self::parser::Parser;

mod parser;

enum ParsingError {
    GeneralError(String),
}

pub fn parse(tokens: impl Iterator<Item = Token>) -> Vec<Stmt> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
