//! Pattern definitions for the Luma AST

use serde::{Deserialize, Serialize};

use super::Span;

/// Pattern for destructuring and pattern matching
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Identifier pattern - binds to a variable
    Ident {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Wildcard pattern - matches anything, doesn't bind
    Wildcard {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// List destructuring pattern
    ListPattern {
        elements: Vec<Pattern>,
        rest: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Table destructuring pattern
    TablePattern {
        fields: Vec<TablePatternField>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Literal pattern - matches a specific value
    Literal {
        value: Literal,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
}

impl Pattern {
    /// Get the span of this pattern, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Pattern::Ident { span, .. } => *span,
            Pattern::Wildcard { span, .. } => *span,
            Pattern::ListPattern { span, .. } => *span,
            Pattern::TablePattern { span, .. } => *span,
            Pattern::Literal { span, .. } => *span,
        }
    }
}

/// Field in a table destructuring pattern
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TablePatternField {
    pub key: String,
    pub binding: Option<String>, // None means key is also the binding name
}

/// Literal value in patterns
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}
