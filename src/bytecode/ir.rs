use serde::{Serialize, Deserialize};

/// Describes where an upvalue is captured from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpvalueDescriptor {
    /// Captured from a local variable at the given stack slot in the enclosing function
    Local(usize),
    /// Captured from an upvalue in the enclosing function at the given upvalue index
    Upvalue(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Function(Chunk),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Const(usize), // push constant at index
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg, // Unary negation
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Not,
    Pop,
    // Pop N items but preserve the previous top-of-stack value
    PopNPreserve(usize),
    Dup,
    Jump(usize),
    JumpIfFalse(usize),
    GetGlobal(usize), // const string name index
    SetGlobal(usize), // const string name index, pops value
    BuildArray(usize), // n
    BuildTable(usize), // n pairs
    GetIndex,          // pops index and object, pushes value
    GetProp(usize),    // const string name index
    GetLen,            // pops array or table, pushes Number (length)
    SetIndex,          // pops value, index, and object
    SetProp(usize),    // const string name index, pops value and object
    GetLocal(usize),
    SetLocal(usize),
    SliceArray(usize),   // pops array, pushes sliced array from index onwards
    MakeFunction(usize), // const index of Function chunk
    Closure(usize),      // const index of Function chunk, captures upvalues from stack/upvalues
    GetUpvalue(usize),   // get upvalue at index
    SetUpvalue(usize),   // set upvalue at index
    Call(usize),         // arity (number of arguments)
    Return,              // return top of stack
    Halt,
    Import,              // pops path string, pushes module value
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub local_count: u16,
    pub name: String,
    /// Describes which upvalues this chunk needs, in order
    /// Each upvalue descriptor tells us how to capture the value when creating a closure
    pub upvalue_descriptors: Vec<UpvalueDescriptor>,
}
