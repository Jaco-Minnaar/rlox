pub mod callable;
pub mod interpreter;

pub use interpreter::InterpreterErrorKind;

use crate::{ast::stmt::Stmt, environment::Environment};

use self::interpreter::Interpreter;
