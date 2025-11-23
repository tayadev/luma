//! Abstract Syntax Tree definitions for Luma
//!
//! This module defines the core AST nodes for expressions, statements, and programs.
//! Type, pattern, and span definitions are in separate submodules.

use serde::{Deserialize, Serialize};

mod patterns;
mod span;
mod types;

pub use patterns::{Literal, Pattern, TablePatternField};
pub use span::{Location, Span, Spanned};
pub use types::{Argument, Type};

/// Argument in a function call (positional or named)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CallArgument {
    Positional(Expr),
    Named { name: String, value: Expr },
}

/// Key in a table literal
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TableKey {
    Identifier(String),
    StringLiteral(String),
    Computed(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Identifier(String),
    String(String),
    Function {
        arguments: Vec<Argument>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Boolean(bool),
    Null,
    List(Vec<Expr>),
    Table(Vec<(TableKey, Expr)>),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<CallArgument>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    If {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Block(Vec<Stmt>),
    Import {
        path: Box<Expr>,
    },
    /// Match as an expression: evaluates to the value of the selected arm
    Match {
        expr: Box<Expr>,
        arms: Vec<(Pattern, Vec<Stmt>)>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
}

impl Expr {
    /// Get the span of this expression, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Expr::Function { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Logical { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MemberAccess { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            _ => None,
        }
    }
}

/// Binary operators
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg, // -x
    Not, // not x
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum LogicalOp {
    And, // and
    Or,  // or
}

/// Assignment operators
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AssignOp {
    Assign, // =
}

/// Statement types
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Stmt {
    VarDecl {
        mutable: bool,
        name: String,
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        r#type: Option<Type>,
        value: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Return {
        value: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    DestructuringVarDecl {
        mutable: bool,
        pattern: Pattern,
        value: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Assignment {
        target: Expr, // Can be Identifier, MemberAccess, or Index
        op: AssignOp,
        value: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        elif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    For {
        pattern: Pattern,
        iterator: Expr,
        body: Vec<Stmt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Break(Option<u32>),    // Optional level (default: 1)
    Continue(Option<u32>), // Optional level (default: 1)
    ExprStmt(Expr),        // Expression statement (e.g., function calls)

    /// Pattern matching: match expr do ... end
    Match {
        expr: Expr,
        arms: Vec<(Pattern, Vec<Stmt>)>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
}

impl Stmt {
    /// Get the span of this statement, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Stmt::VarDecl { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::DestructuringVarDecl { span, .. } => *span,
            Stmt::Assignment { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::DoWhile { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::Break(_) => None,
            Stmt::Continue(_) => None,
            Stmt::ExprStmt(_) => None,
            Stmt::Match { span, .. } => *span,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
