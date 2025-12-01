//! `check` subcommand handler

use crate::utils::read_source;
use luma_core::pipeline::Pipeline;
use std::process;

/// Typecheck a Luma script without executing it
pub fn handle_check(file: &str) {
    let source = match read_source(file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{file}': {err}");
            process::exit(1);
        }
    };

    let pipeline = Pipeline::new(source.clone(), file.to_string());

    match pipeline.parse() {
        Ok(ast) => match pipeline.typecheck(&ast) {
            Ok(()) => println!("Typecheck: OK"),
            Err(e) => {
                eprintln!("{}", e.format_with_source(&source));
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", e.format_with_source(&source));
            process::exit(1);
        }
    }
}
