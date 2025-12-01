//! Call frame management for function calls in the VM

use super::value::Upvalue;
use crate::bytecode::ir::Chunk;
use std::collections::HashMap;

/// Represents a call frame in the call stack
pub struct CallFrame {
    pub chunk: Chunk,
    pub ip: usize,
    pub base: usize,
    pub upvalues: Vec<Upvalue>, // Captured upvalues for this function
    // Captured locals for this frame: absolute stack index -> shared cell
    pub captured_locals: HashMap<usize, Upvalue>,
}
