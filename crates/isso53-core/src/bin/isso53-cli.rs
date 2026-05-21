//! ISSO 53 command-line interface.
//!
//! Usage:
//!     isso53-cli <input.json> [output.json]
//!     cat input.json | isso53-cli -
//!
//! Reads a Project JSON, runs the calculation, and writes ProjectResult JSON.

use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let input = match args.get(1).map(String::as_str) {
        None | Some("-") => {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("error reading stdin: {e}");
                return ExitCode::from(2);
            }
            buf
        }
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading {path}: {e}");
                return ExitCode::from(2);
            }
        },
    };

    let output = match isso53_core::calculate_from_json(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("calculation error: {e}");
            return ExitCode::from(1);
        }
    };

    match args.get(2) {
        Some(path) => {
            if let Err(e) = std::fs::write(path, output) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::from(2);
            }
        }
        None => {
            if let Err(e) = io::stdout().write_all(output.as_bytes()) {
                eprintln!("error writing stdout: {e}");
                return ExitCode::from(2);
            }
        }
    }

    ExitCode::SUCCESS
}
