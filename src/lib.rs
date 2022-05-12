pub mod ast;
pub mod lexer;

use std::{fs, io};

use rustyline::{error::ReadlineError, Editor};

const HISTORY_PATH: &'static str = ".dev-data/history";

#[derive(Debug)]
enum LoxErrorType {
    LexingError,
    ParsingError,
}

#[derive(Debug)]
struct LoxError {
    error_type: LoxErrorType,
    line: usize,
}

pub fn run_file(path: String) -> io::Result<()> {
    let file_contents = fs::read_to_string(path)?;

    run(file_contents.as_str()).unwrap();

    Ok(())
}

pub fn run_prompt() -> io::Result<()> {
    let mut rl = Editor::<()>::new();
    rl.load_history(&HISTORY_PATH).unwrap_or_default();

    loop {
        let readline = rl.readline(">> ");

        let line = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
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

        if let Err(e) = run(line.as_str()) {
            eprintln!("Error: {:?}", e);
            continue;
        }
    }
    rl.save_history(HISTORY_PATH).unwrap();

    Ok(())
}

fn run(content: &str) -> Result<(), LoxError> {
    let tokens = lexer::tokenize(content);

    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
