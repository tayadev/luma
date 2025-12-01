//! Shared CLI utilities for reading input and formatting errors

use luma_core::ast;
use luma_core::diagnostics;
use luma_core::typecheck;
use std::fs;
use std::io::{self, Read};

/// Read source code from a file or stdin.
/// If `file` is "-", reads from stdin. Otherwise reads from the specified file.
pub fn read_source(file: &str) -> io::Result<String> {
    if file == "-" {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    } else {
        fs::read_to_string(file)
    }
}

/// Format and print parse errors to stderr
pub fn format_parse_errors(errors: &[diagnostics::Diagnostic], source: &str) {
    for error in errors {
        eprintln!("{}", error.format(source));
    }
}

/// Format and print typecheck errors to stderr
pub fn format_typecheck_errors(errors: &[typecheck::TypeError], file: &str, source: &str) {
    eprintln!("Typecheck failed:");
    for e in errors {
        let diag = diagnostics::Diagnostic::error(
            diagnostics::DiagnosticKind::Type,
            e.message.clone(),
            e.span.unwrap_or_else(|| ast::Span::new(0, 0)),
            file.to_string(),
        );
        eprintln!("{}", diag.format(source));
    }
}
