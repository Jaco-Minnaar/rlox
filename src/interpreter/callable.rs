use std::{cell::RefCell, fmt::Display, rc::Rc, time::SystemTime};

use crate::{
    ast::{expr::Literal, stmt::Stmt},
    environment::Environment,
    lexer::Token,
};

use dyn_clone::DynClone;

use super::{interpreter::Interpreter, InterpreterErrorKind};

pub trait Callable: ToString + Display + DynClone + std::fmt::Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Literal],
    ) -> Result<Literal, InterpreterErrorKind>;
    fn arity(&self) -> usize;
}

dyn_clone::clone_trait_object!(Callable);

impl<T: Callable> TryFrom<Literal> for (T,) {
    type Error = String;
    fn try_from(value: Literal) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub enum LoxCallable {
    Function(LoxFunction),
    Other(Box<dyn Callable>),
}

impl Callable for LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Literal],
    ) -> Result<Literal, InterpreterErrorKind> {
        match self {
            LoxCallable::Function(fun) => fun.call(interpreter, args),
            LoxCallable::Other(fun) => fun.call(interpreter, args),
        }
    }

    fn arity(&self) -> usize {
        match self {
            LoxCallable::Function(fun) => fun.arity(),
            LoxCallable::Other(fun) => fun.arity(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(fun) => fun.fmt(f),
            Self::Other(fun) => fun.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoxFunction {
    name: String,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: String,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
        }
    }
}

impl Callable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Literal],
    ) -> Result<Literal, InterpreterErrorKind> {
        let closure = Rc::clone(&self.closure);
        let mut environment = Environment::with_enclosing(closure);

        for i in 0..self.params.len() {
            environment.define(self.params[i].lexeme.clone(), args[i].clone());
        }

        match interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment))) {
            Err(InterpreterErrorKind::Return(value)) => Ok(if let Some(value) = value {
                value
            } else {
                Literal::Nil
            }),
            Err(e) => Err(e),
            _ => Ok(Literal::Nil),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: &[Literal],
    ) -> Result<Literal, InterpreterErrorKind> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(Literal::Number(time as f64))
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
