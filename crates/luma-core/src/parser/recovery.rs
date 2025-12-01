//! Error recovery strategies for the Luma parser.
//!
//! This module provides recovery strategies to allow the parser to continue
//! past errors and collect multiple errors in a single pass.
//!
//! ## Recovery Strategy
//!
//! When a statement fails to parse, the parser uses error recovery to:
//! 1. Skip to the next line (newline character)
//! 2. Resume parsing from the next statement
//! 3. Continue accumulating errors for the rest of the file
//!
//! ## Recoverable Scenarios
//!
//! The following error types can be recovered from:
//! - Invalid characters at the start of a statement (e.g., `@`, `#`)
//! - Malformed expressions within a statement
//! - Missing tokens after partial statement parsing
//!
//! ## Non-Recoverable Scenarios
//!
//! Some errors cannot be recovered from or may produce misleading secondary errors:
//! - Unbalanced delimiters (parentheses, brackets, braces)
//! - Errors inside block structures (if/while/for/function bodies)
//! - Missing `end` keywords for blocks
//!
//! For these cases, the parser will still report the primary error, but recovery
//! may skip valid code or produce confusing secondary errors.
//!
//! ## Example
//!
//! ```text
//! let a = @invalid     -- Error: unexpected '@'
//! let b = 42           -- Successfully parsed after recovery
//! let c = #invalid     -- Error: unexpected '#'  
//! let d = 100          -- Successfully parsed after recovery
//! ```
//!
//! This input produces 2 errors instead of 1, allowing developers to see
//! multiple issues in a single parse pass.

use crate::ast::{Expr, Span, Stmt};
use chumsky::prelude::*;

/// Creates a recovery parser for statements.
///
/// When a statement fails to parse, this will skip to the next line
/// and return a placeholder statement.
///
/// # Returns
///
/// Returns a `Stmt::ExprStmt` containing a `Null` expression as a placeholder.
/// This placeholder statement occupies the position of the failed statement
/// in the AST, allowing parsing to continue with subsequent statements.
///
/// # Recovery Behavior
///
/// The recovery parser:
/// 1. Consumes at least one character (to make progress)
/// 2. Continues consuming until it reaches a newline or end of input
/// 3. Optionally consumes the newline to position at the start of the next line
/// 4. Returns a placeholder statement with the error span
pub fn statement_recovery<'a>() -> impl Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone
{
    // Skip all characters on the current line (up to newline or EOF)
    // then optionally consume the newline to position at the start of the next line
    none_of("\n")
        .repeated()
        .at_least(1) // Must consume at least something to make progress
        .then_ignore(just("\n").or_not())
        .to_slice()
        .try_map(|_skipped: &str, span| {
            // Return a placeholder statement for the skipped error content
            Ok(Stmt::ExprStmt {
                expr: Expr::Null {
                    span: Some(Span::from_chumsky(span)),
                },
                span: Some(Span::from_chumsky(span)),
            })
        })
}
