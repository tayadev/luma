//! `run` subcommand handler

use crate::cli::utils::read_source;
use crate::pipeline::Pipeline;
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

    let pipeline = Pipeline::new(source.clone(), file.to_string());

    match pipeline.run_all() {
        Ok(_val) => {}
        Err(e) => {
            eprintln!("{}", e.format_with_source(&source));
            process::exit(1);
        }
    }
}
