//! Error formatting for parser errors
//!
//! Converts Chumsky parser errors into user-friendly diagnostic messages

use crate::ast::Span;
use crate::diagnostics::{Diagnostic, DiagnosticKind};
use chumsky::error::{Rich, RichReason};

/// Convert Chumsky error reason to readable message
pub fn format_error_reason(reason: &RichReason<char>) -> String {
    match reason {
        RichReason::ExpectedFound { expected, found } => {
            let found_msg = match found {
                Some(c) => format!("'{}'", c.escape_debug()),
                None => "end of input".to_string(),
            };

            if expected.is_empty() {
                format!("unexpected {}", found_msg)
            } else {
                // Just show a simplified message instead of listing all expected tokens
                if found.is_none() {
                    "unexpected end of input".to_string()
                } else {
                    format!("unexpected {}", found_msg)
                }
            }
        }
        RichReason::Custom(msg) => msg.to_string(),
    }
}

/// Convert Chumsky parse errors to Luma diagnostics
pub fn errors_to_diagnostics(errors: Vec<Rich<char>>, filename: &str) -> Vec<Diagnostic> {
    errors
        .into_iter()
        .map(|e| {
            let span = Span::new(e.span().start, e.span().end);
            let message = format_error_reason(e.reason());
            Diagnostic::error(DiagnosticKind::Parse, message, span, filename.to_string())
        })
        .collect()
}
