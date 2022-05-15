mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod runner;

use std::{fs, io};

use runner::Runner;
use rustyline::{error::ReadlineError, Editor};

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
    let mut runner = Runner::new();

    runner.run(file_contents.as_str()).unwrap();

    Ok(())
}

pub fn run_prompt() -> io::Result<()> {
    let mut rl = Editor::<()>::new();
    rl.load_history(&HISTORY_PATH).unwrap_or_default();
    let mut runner = Runner::new();

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

        if let Err(_e) = runner.run(line.as_str()) {
            continue;
        }
    }
    rl.save_history(HISTORY_PATH).unwrap();

    Ok(())
}
