use crate::bytecode::ir::Chunk;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// Type for native function pointers
pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

/// An upvalue is a reference to a variable captured by a closure.
/// It uses RefCell to allow mutation of the captured value.
#[derive(Debug, Clone)]
pub struct Upvalue {
    pub value: Rc<RefCell<Value>>,
}

impl Upvalue {
    pub fn new(value: Value) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    #[serde(skip)]
    List(Rc<RefCell<Vec<Value>>>),
    #[serde(skip)]
    Table(Rc<RefCell<HashMap<String, Value>>>),
    Function {
        chunk: Chunk,
        arity: usize,
    },
    /// Closure is like a function but also captures upvalues from its enclosing scope
    #[serde(skip)]
    Closure {
        chunk: Chunk,
        arity: usize,
        upvalues: Vec<Upvalue>,
    },
    /// NativeFunction stores only metadata (name, arity).
    /// The actual function pointer is stored in VM's native_functions HashMap.
    /// This design allows Values to be serializable while keeping function pointers
    /// in the VM runtime.
    #[serde(skip)]
    NativeFunction {
        name: String,
        arity: usize,
    },
    /// Type represents a type definition (used for cast and isInstanceOf).
    /// It's essentially a table that describes the structure of a type,
    /// including field definitions and optional __parent for inheritance.
    #[serde(skip)]
    Type(Rc<RefCell<HashMap<String, Value>>>),
    /// External represents an opaque pointer to external/native data.
    /// Used for FFI to hold references to C data structures.
    /// The usize is an opaque handle that the FFI system uses to track the resource.
    #[serde(skip)]
    External {
        /// Opaque handle identifying the external resource
        handle: usize,
        /// Type name for debugging/display purposes
        type_name: String,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => {
                // Compare by reference first, then by value
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }
            (Value::Table(a), Value::Table(b)) => {
                // Compare by reference first, then by value
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }
            (Value::Function { arity: a1, .. }, Value::Function { arity: a2, .. }) => a1 == a2,
            (Value::Closure { arity: a1, .. }, Value::Closure { arity: a2, .. }) => a1 == a2,
            (
                Value::NativeFunction {
                    name: n1,
                    arity: a1,
                },
                Value::NativeFunction {
                    name: n2,
                    arity: a2,
                },
            ) => n1 == n2 && a1 == a2,
            (Value::Type(a), Value::Type(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Value::External { handle: h1, .. }, Value::External { handle: h2, .. }) => h1 == h2,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // Format numbers nicely - remove .0 for whole numbers
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::String(s) => write!(f, "{s}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Null => write!(f, "null"),
            Value::List(arr) => {
                let borrowed = arr.borrow();
                write!(f, "[")?;
                for (i, val) in borrowed.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
            Value::Table(table) => {
                let borrowed = table.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (key, val) in borrowed.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{key}: {val}")?;
                }
                write!(f, "}}")
            }
            Value::Function { arity, .. } => write!(f, "<function/{arity}>"),
            Value::Closure { arity, .. } => write!(f, "<closure/{arity}>"),
            Value::NativeFunction { name, arity } => {
                write!(f, "<native function {name}/{arity}>",)
            }
            Value::Type(_) => write!(f, "<type>"),
            Value::External { handle, type_name } => {
                write!(f, "<external {type_name} id:{handle}>")
            }
        }
    }
}
