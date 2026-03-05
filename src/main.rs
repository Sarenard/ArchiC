#![allow(non_snake_case)]

use std::{env, process::ExitCode};

use ArchiC::compile::bisare;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let src = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} <program.ac>", args[0]);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = bisare(src.into()) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}