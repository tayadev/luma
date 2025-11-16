use serde::{Serialize, Deserialize};

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
    GetLocal(usize),
    SetLocal(usize),
    MakeFunction(usize), // const index of Function chunk
    Call(usize),         // arity (number of arguments)
    Return,              // return top of stack
    Halt,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub local_count: u16,
    pub name: String,
}
