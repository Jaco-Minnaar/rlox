use crate::{
    interpreter::{interpreter::Interpreter, InterpreterErrorKind},
    lexer, parser, LoxError, LoxErrorType,
};

pub struct Runner {
    interpreter: Interpreter,
}

impl Runner {
    pub fn new() -> Runner {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, code: &str) -> Result<(), LoxError> {
        // let tokens = lexer::tokenize(content);
        //
        // for token in tokens {
        //     println!("{:?}", token);
        // }

        let tokens = lexer::tokenize(code);

        let stmts = parser::parse(tokens);
        // let printed_ast = ast::pretty_print(&expr);
        // println!("{}", printed_ast);

        for stmt in stmts {
            if let Err(e) = self.interpreter.execute(&stmt) {
                match e {
                    InterpreterErrorKind::General(s) => {
                        eprintln!("Interpreter Error: {}", s);
                        return Err(LoxError {
                            error_type: LoxErrorType::RuntimeError,
                            line: 0,
                        });
                    }
                    _ => (),
                }
            };
        }

        Ok(())
    }
}
