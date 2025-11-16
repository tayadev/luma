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
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Identifier(String),
    String(String),
    Concat(Vec<Expr>),
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
    Block(Vec<Stmt>),
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
pub enum Pattern {
    Ident(String),
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
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
