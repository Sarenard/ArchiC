#![allow(non_snake_case)]

use std::{env, process::ExitCode};

use ArchiC::compile::bisare;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let src = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} <program.ac> [--original]", args[0]);
            return ExitCode::FAILURE;
        }
    };

    let original = args.iter().any(|a| a == "--original");

    if let Err(e) = bisare(src.into(), original) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
