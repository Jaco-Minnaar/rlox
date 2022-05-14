mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;

use std::{fs, io};

use rustyline::{error::ReadlineError, Editor};

use crate::environment::Environment;
use crate::interpreter::InterpreterErrorKind;

const HISTORY_PATH: &'static str = ".dev-data/history";

#[derive(Debug)]
enum LoxErrorType {
    LexingError,
    ParsingError,
    RuntimeError,
}

#[derive(Debug)]
struct LoxError {
    error_type: LoxErrorType,
    line: usize,
}

pub fn run_file(path: String) -> io::Result<()> {
    let file_contents = fs::read_to_string(path)?;
    let mut env = Environment::new();

    run(file_contents.as_str(), &mut env).unwrap();

    Ok(())
}

pub fn run_prompt() -> io::Result<()> {
    let mut rl = Editor::<()>::new();
    rl.load_history(&HISTORY_PATH).unwrap_or_default();
    let mut environment = Environment::new();

    loop {
        let readline = rl.readline(">> ");

        let line = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        rl.add_history_entry(line.as_str());

        if let Err(_e) = run(line.as_str(), &mut environment) {
            continue;
        }
    }
    rl.save_history(HISTORY_PATH).unwrap();

    Ok(())
}

fn run(content: &str, env: &mut Environment) -> Result<(), LoxError> {
    // let tokens = lexer::tokenize(content);
    //
    // for token in tokens {
    //     println!("{:?}", token);
    // }

    let tokens = lexer::tokenize(content);

    let stmts = parser::parse(tokens);
    // let printed_ast = ast::pretty_print(&expr);
    // println!("{}", printed_ast);

    if let Err(e) = interpreter::interpret(&stmts, env) {
        match e {
            InterpreterErrorKind::General(s) => {
                eprintln!("Interpreter Error: {}", s);
                return Err(LoxError {
                    error_type: LoxErrorType::RuntimeError,
                    line: 0,
                });
            }
        }
    };

    Ok(())
}
