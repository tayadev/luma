//! Type checking error types and result types.

use crate::ast::Span;

/// A type error with message and optional source location.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Option<Span>,
}

/// Result type for type checking operations.
pub type TypecheckResult<T> = Result<T, Vec<TypeError>>;
