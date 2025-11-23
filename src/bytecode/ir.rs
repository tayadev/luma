use serde::{Serialize, Deserialize};
use crate::ast::Span;

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
    BuildList(usize), // n
    BuildTable(usize), // n pairs
    GetIndex,          // pops index and object, pushes value
    GetProp(usize),    // const string name index
    GetLen,            // pops list or table, pushes Number (length)
    SetIndex,          // pops value, index, and object
    SetProp(usize),    // const string name index, pops value and object
    GetLocal(usize),
    SetLocal(usize),
    SliceList(usize),   // pops list, pushes sliced list from index onwards
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
    /// Maps instruction index to source span (parallel to instructions)
    /// None indicates no source location available
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub spans: Vec<Option<Span>>,
}

impl Chunk {
    /// Creates a new empty chunk with the given name
    pub fn new_empty(name: String) -> Self {
        Chunk {
            instructions: vec![Instruction::Halt],
            constants: vec![],
            local_count: 0,
            name,
            upvalue_descriptors: vec![],
            spans: vec![None],  // One span for the Halt instruction
        }
    }

    /// Get the span for an instruction at a given index
    pub fn get_span(&self, ip: usize) -> Option<Span> {
        self.spans.get(ip).and_then(|&s| s)
    }

    /// Push an instruction with an optional span
    pub fn push_instruction(&mut self, instr: Instruction, span: Option<Span>) {
        self.instructions.push(instr);
        self.spans.push(span);
    }
}
