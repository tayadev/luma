//! `run` subcommand handler

use crate::utils::read_source;
use std::process;

/// Execute a Luma script file
pub fn handle_run(file: &str) {
    let source = match read_source(file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{file}': {err}");
            process::exit(1);
        }
    };

    match luma_stdlib::run_program(source.clone(), file.to_string()) {
        Ok(_val) => {}
        Err(e) => {
            eprintln!("{}", e.format_with_source(&source));
            process::exit(1);
        }
    }
}
