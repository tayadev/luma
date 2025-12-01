//! Type checking error types and result types.

use crate::ast::Span;
use crate::diagnostics::FixIt;

/// A type error with message and optional source location.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Option<Span>,
    /// Optional human-friendly suggestions
    pub suggestions: Vec<String>,
    /// Optional machine-applicable fixes
    pub fixits: Vec<FixIt>,
}

/// Result type for type checking operations.
pub type TypecheckResult<T> = Result<T, Vec<TypeError>>;
