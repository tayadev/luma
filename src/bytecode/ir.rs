#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    // Function bytecode can be added later
}

#[derive(Debug, Clone)]
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
    Dup,
    Jump(usize),
    JumpIfFalse(usize),
    GetGlobal(usize), // const string name index
    SetGlobal(usize), // const string name index, pops value
    Halt,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub local_count: u16,
    pub name: String,
}
