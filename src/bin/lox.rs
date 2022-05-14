use rlox;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    let args_len = args.len();
    if args_len > 2 {
        eprintln!("Usage: rlox [script]");
        process::exit(64);
    } else if args_len == 2 {
        rlox::run_file(args[1].clone()).unwrap();
    } else {
        rlox::run_prompt().unwrap();
    }
}
