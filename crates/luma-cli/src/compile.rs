//! `compile` subcommand handler

use crate::utils::{format_parse_errors, format_typecheck_errors, read_source};
use luma_core::bytecode;
use luma_core::parser;
use luma_core::typecheck;
use std::fs;
use std::process;

/// Compile a Luma script to bytecode
pub fn handle_compile(file: &str, output: Option<&str>) {
    let source = match read_source(file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{file}': {err}");
            process::exit(1);
        }
    };

    let ast = match parser::parse(&source, file) {
        Ok(ast) => ast,
        Err(errors) => {
            format_parse_errors(&errors, &source);
            process::exit(1);
        }
    };

    if let Err(errs) = typecheck::typecheck_program(&ast) {
        format_typecheck_errors(&errs, file, &source);
        process::exit(1);
    }

    let chunk = bytecode::compile::compile_program(&ast);

    // Determine output filename
    let output_file = match output {
        Some(o) => o.to_string(),
        None => {
            if file == "-" {
                eprintln!("Error: Cannot compile from stdin without --output flag");
                process::exit(1);
            }
            // Replace extension with .lumac
            let path = std::path::Path::new(file);
            path.with_extension("lumac").to_string_lossy().to_string()
        }
    };

    // Serialize the bytecode chunk
    let serialized = match ron::ser::to_string_pretty(&chunk, ron::ser::PrettyConfig::default()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error serializing bytecode: {e}");
            process::exit(1);
        }
    };

    // Write to file
    if let Err(e) = fs::write(&output_file, serialized) {
        eprintln!("Error writing to '{output_file}': {e}");
        process::exit(1);
    }

    println!("Compiled '{file}' to '{output_file}'");
}
