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

/// A field in a table literal, with optional type annotation
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TableField {
    pub key: TableKey,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub field_type: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number {
        value: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Identifier {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    String {
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Function {
        arguments: Vec<Argument>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Boolean {
        value: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Null {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    List {
        elements: Vec<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Table {
        fields: Vec<TableField>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
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
    /// Method call with colon operator: object:method(args)
    /// Desugars to: object.method(object, args)
    MethodCall {
        object: Box<Expr>,
        method: String,
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
    Block {
        statements: Vec<Stmt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Import {
        path: Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
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
            Expr::Number { span, .. } => *span,
            Expr::Identifier { span, .. } => *span,
            Expr::String { span, .. } => *span,
            Expr::Function { span, .. } => *span,
            Expr::Boolean { span, .. } => *span,
            Expr::Null { span, .. } => *span,
            Expr::List { span, .. } => *span,
            Expr::Table { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Logical { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::MemberAccess { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Block { span, .. } => *span,
            Expr::Import { span, .. } => *span,
            Expr::Match { span, .. } => *span,
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
    Break {
        level: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Continue {
        level: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    ExprStmt {
        expr: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },

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
            Stmt::Break { span, .. } => *span,
            Stmt::Continue { span, .. } => *span,
            Stmt::ExprStmt { span, .. } => *span,
            Stmt::Match { span, .. } => *span,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
