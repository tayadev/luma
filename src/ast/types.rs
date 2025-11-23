//! Type definitions for the Luma AST

use serde::{Deserialize, Serialize};

use super::Span;

/// Type annotation in Luma
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Type {
    /// Simple type identifier (Number, String, Boolean, etc.)
    TypeIdent {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Generic type with type arguments (List(String), Result(Number, String))
    GenericType {
        name: String,
        type_args: Vec<Type>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Function type (fn(Number, String): Boolean)
    FunctionType {
        param_types: Vec<Type>,
        return_type: Box<Type>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    /// Dynamic type - no static type checking
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
}

impl Type {
    /// Get the span of this type, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Type::TypeIdent { span, .. } => *span,
            Type::GenericType { span, .. } => *span,
            Type::FunctionType { span, .. } => *span,
            Type::Any { span, .. } => *span,
        }
    }
}

/// Function argument with optional type and default value
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub default: Option<crate::ast::Expr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub span: Option<Span>,
}
