//! Error formatting for parser errors
//!
//! Converts Chumsky parser errors into user-friendly diagnostic messages

use crate::ast::Span;
use crate::diagnostics::{Diagnostic, DiagnosticKind, FixIt};
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
                format!("unexpected {found_msg}")
            } else {
                // Just show a simplified message instead of listing all expected tokens
                if found.is_none() {
                    "unexpected end of input".to_string()
                } else {
                    format!("unexpected {found_msg}")
                }
            }
        }
        RichReason::Custom(msg) => msg.to_string(),
    }
}

/// Convert Chumsky parse errors to Luma diagnostics
pub fn errors_to_diagnostics(
    errors: Vec<Rich<char>>,
    filename: &str,
    source: &str,
) -> Vec<Diagnostic> {
    errors
        .into_iter()
        .map(|e| {
            let span = Span::new(e.span().start, e.span().end);
            let message = format_error_reason(e.reason());
            let mut diag = Diagnostic::error(
                DiagnosticKind::Parse,
                message.clone(),
                span,
                filename.to_string(),
            );

            augment_with_fixits(&mut diag, e.reason(), span, source);

            diag
        })
        .collect()
}

/// Expand diagnostics with helpful fix-its based on the error reason and source context
fn augment_with_fixits(diag: &mut Diagnostic, reason: &RichReason<char>, span: Span, source: &str) {
    // We branch primarily on the human-friendly message we generate elsewhere
    let message = format_error_reason(reason);
    if message == "unexpected end of input" {
        let to_insert = compute_missing_closers(source);
        if !to_insert.is_empty() {
            let label = if to_insert == "end" {
                "Insert 'end'".to_string()
            } else {
                "Insert missing closers".to_string()
            };
            diag.suggestions
                .push("Possible unclosed block or delimiter".to_string());
            diag.fixits.push(FixIt::replace(
                Span::new(span.end, span.end),
                to_insert,
                label,
            ));
        } else {
            diag.suggestions
                .push("Did you forget to close a block with 'end'?".to_string());
            diag.fixits.push(FixIt::replace(
                Span::new(span.end, span.end),
                "\nend",
                "Insert 'end'",
            ));
        }
        return;
    }

    // Parse messages like: unexpected ')'
    let prefix = "unexpected '";
    let suffix = "'";
    if message.starts_with(prefix)
        && message.ends_with(suffix)
        && message.len() > prefix.len() + suffix.len()
    {
        let inner = &message[prefix.len()..message.len() - suffix.len()];
        if let Some(ch) = inner.chars().next() {
            if ")]}".contains(ch) {
                diag.fixits
                    .push(FixIt::replace(span, "", format!("Remove '{ch}'")));
            } else if ch == ',' {
                diag.fixits.push(FixIt::replace(span, "", "Remove ','"));
            }
        }
    }
}

/// Compute a best-effort sequence of missing closing delimiters and 'end's.
/// Returns a string to insert at EOF to balance the source.
fn compute_missing_closers(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum Closer {
        Char(char),
        End,
    }

    let mut stack: Vec<Closer> = Vec::new();

    // Scan characters for brackets
    for ch in source.chars() {
        match ch {
            '(' => stack.push(Closer::Char(')')),
            '[' => stack.push(Closer::Char(']')),
            '{' => stack.push(Closer::Char('}')),
            ')' | ']' | '}' => {
                // pop matching char if present
                if let Some(pos) = stack
                    .iter()
                    .rposition(|c| matches!(c, Closer::Char(rc) if *rc == ch))
                {
                    stack.remove(pos);
                }
            }
            _ => {}
        }
    }

    // Scan words for do/end pairing
    let mut i = 0;
    let bytes = source.as_bytes();
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let word = &source[start..i];
            match word {
                "do" => stack.push(Closer::End),
                "end" => {
                    if let Some(pos) = stack.iter().rposition(|c| matches!(c, Closer::End)) {
                        stack.remove(pos);
                    }
                }
                _ => {}
            }
        } else {
            i += 1;
        }
    }

    if stack.is_empty() {
        return String::new();
    }

    // Build insertion text from remaining stack, last-in-first-out
    let mut s = String::new();
    for c in stack.into_iter().rev() {
        match c {
            Closer::Char(ch) => s.push(ch),
            Closer::End => {
                // Put on new line to avoid accidental token gluing
                if !s.ends_with('\n') && !s.is_empty() {
                    s.push('\n');
                }
                s.push_str("end");
            }
        }
    }
    s
}
