//! Debug subcommands: `ast` and `bytecode`

use crate::utils::{format_parse_errors, read_source};
use luma_core::bytecode;
use luma_core::parser;
use std::process;

/// Print the parsed AST for debugging
pub fn handle_ast(file: &str) {
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

    println!("{ast:#?}");
}

/// Print the compiled bytecode for debugging
pub fn handle_bytecode(file: &str) {
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

    let chunk = bytecode::compile::compile_program(&ast);
    println!("{chunk:#?}");
}
