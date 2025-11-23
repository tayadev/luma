use serde::{Deserialize, Serialize};

/// Represents a location in the source code as byte offsets
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct Span {
    /// Byte offset of the start of the span (inclusive)
    pub start: usize,
    /// Byte offset of the end of the span (exclusive)
    pub end: usize,
}

/// A value with an associated source span
/// Serializes transparently to the inner value for backward compatibility
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

// Serialize as the inner value only (skip span for now to avoid breaking fixtures)
impl<T: Serialize> Serialize for Spanned<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

// Deserialize as the inner value with a dummy span
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Spanned<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Spanned {
            value,
            span: Span::new(0, 0), // Dummy span during deserialization
        })
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Create a span from a Chumsky SimpleSpan
    pub fn from_chumsky(span: chumsky::span::SimpleSpan) -> Self {
        Span {
            start: span.start,
            end: span.end,
        }
    }

    /// Calculate line and column from source text
    pub fn location(&self, source: &str) -> Location {
        let mut line = 1;
        let mut col = 1;
        
        for (byte_idx, ch) in source.char_indices() {
            if byte_idx >= self.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        Location { line, col, offset: self.start }
    }

    /// Get the source text for this span
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end.min(source.len())]
    }
}

/// Represents a specific location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub default: Option<Expr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub span: Option<Span>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Type {
    TypeIdent(String),
    GenericType {
        name: String,
        type_args: Vec<Type>,
    },
    FunctionType {
        param_types: Vec<Type>,
        return_type: Box<Type>,
    },
    Any,
}

impl Type {
    /// Get the span of this type, if available
    pub fn span(&self) -> Option<Span> {
        None // Types don't have spans yet, can be added later if needed
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CallArgument {
    Positional(Expr),
    Named { name: String, value: Expr },
}

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
    Neg,  // -x
    Not,  // not x
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum LogicalOp {
    And,  // and
    Or,   // or
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AssignOp {
    Assign, // =
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Ident(String),
    Wildcard,
    ListPattern {
        elements: Vec<Pattern>,
        rest: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    TablePattern {
        fields: Vec<TablePatternField>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        span: Option<Span>,
    },
    Literal(Literal),
}

impl Pattern {
    /// Get the span of this pattern, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Pattern::ListPattern { span, .. } => *span,
            Pattern::TablePattern { span, .. } => *span,
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TablePatternField {
    pub key: String,
    pub binding: Option<String>,  // None means key is also the binding name
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

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
        target: Expr,  // Can be Identifier, MemberAccess, or Index
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
    Break(Option<u32>),  // Optional level (default: 1)
    Continue(Option<u32>),  // Optional level (default: 1)
    ExprStmt(Expr),  // Expression statement (e.g., function calls)

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
