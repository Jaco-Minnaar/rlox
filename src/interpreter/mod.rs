mod interpreter;

pub use interpreter::InterpreterErrorKind;

use crate::{ast::stmt::Stmt, environment::Environment};

use self::interpreter::Interpreter;

pub fn interpret(
    stmts: &[Stmt],
    environment: &mut Environment,
) -> Result<(), InterpreterErrorKind> {
    let mut interpreter = Interpreter::new(environment);

    for stmt in stmts {
        interpreter.execute(stmt)?;
    }

    Ok(())
}
