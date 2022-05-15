use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::expr::Literal, lexer::Token};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value).unwrap();
            Ok(())
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow_mut().assign(name, value)
            } else {
                Err(format!("Undefined variable '{}'.", name))
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal, String> {
        match self.values.get(name.lexeme.as_str()) {
            Some(v) => Ok(v.clone().to_owned()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(name),
                None => Err(format!("Undefined variable '{}'.", name.lexeme.as_str())),
            },
        }
    }
}
