use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::bytecode::ir::Chunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Table(HashMap<String, Value>),
    Function { chunk: Chunk, arity: usize },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => a == b,
            (Value::Function { arity: a1, .. }, Value::Function { arity: a2, .. }) => a1 == a2,
            _ => false,
        }
    }
}
