use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub default: Option<Expr>,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CallArgument {
    Positional(Expr),
    Named { name: String, value: Expr },
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
    },
    Boolean(bool),
    Null,
    Array(Vec<Expr>),
    Table(Vec<(String, Expr)>),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<CallArgument>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    Block(Vec<Stmt>),
    Import {
        path: Box<Expr>,
    },
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
    ArrayPattern {
        elements: Vec<Pattern>,
        rest: Option<String>,
    },
    TablePattern(Vec<String>),
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
    },
    Return(Expr),
    DestructuringVarDecl {
        mutable: bool,
        pattern: Pattern,
        value: Expr,
    },
    Assignment {
        target: Expr,  // Can be Identifier, MemberAccess, or Index
        op: AssignOp,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        elif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    For {
        pattern: Pattern,
        iterator: Expr,
        body: Vec<Stmt>,
    },
    Break(Option<u32>),  // Optional level (default: 1)
    Continue(Option<u32>),  // Optional level (default: 1)
    ExprStmt(Expr),  // Expression statement (e.g., function calls)

    /// Pattern matching: match expr do ... end
    Match {
        expr: Expr,
        arms: Vec<(Pattern, Vec<Stmt>)>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
