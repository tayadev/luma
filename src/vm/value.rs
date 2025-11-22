use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};
use crate::bytecode::ir::Chunk;

// Type for native function pointers
pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    #[serde(skip)]
    Array(Rc<RefCell<Vec<Value>>>),
    #[serde(skip)]
    Table(Rc<RefCell<HashMap<String, Value>>>),
    Function { chunk: Chunk, arity: usize },
    /// NativeFunction stores only metadata (name, arity).
    /// The actual function pointer is stored in VM's native_functions HashMap.
    /// This design allows Values to be serializable while keeping function pointers
    /// in the VM runtime.
    #[serde(skip)]
    NativeFunction { name: String, arity: usize },
    /// Type represents a type definition (used for cast and isInstanceOf).
    /// It's essentially a table that describes the structure of a type,
    /// including field definitions and optional __parent for inheritance.
    #[serde(skip)]
    Type(Rc<RefCell<HashMap<String, Value>>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => {
                // Compare by reference first, then by value
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }
            (Value::Table(a), Value::Table(b)) => {
                // Compare by reference first, then by value
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }
            (Value::Function { arity: a1, .. }, Value::Function { arity: a2, .. }) => a1 == a2,
            (Value::NativeFunction { name: n1, arity: a1 }, Value::NativeFunction { name: n2, arity: a2 }) => {
                n1 == n2 && a1 == a2
            }
            (Value::Type(a), Value::Type(b)) => {
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }
            _ => false,
        }
    }
}
