use std::collections::HashMap;

use crate::{ast::expr::Literal, lexer::Token};

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Literal, String> {
        match self.values.get(name.lexeme.as_str()) {
            Some(v) => Ok(v),
            None => Err(format!("Undefined variable '{}'.", name.lexeme.as_str())),
        }
    }
}
