use super::ir::{Constant, Instruction};
use std::collections::HashMap;

// Shared string constants to avoid magic strings
pub(super) const HIDDEN_MATCH_VAL: &str = "__match_val";
pub(super) const HIDDEN_DESTRUCTURE_VAL: &str = "__destructure_val";
pub(super) const HIDDEN_ITER: &str = "__iter";
pub(super) const HIDDEN_I: &str = "__i";
pub(super) const GLOBAL_ITER_FN: &str = "iter";

// Forward-declare Compiler so we can extend it
use super::compile::Compiler;

impl Compiler {
    // Stack/const helpers
    pub(super) fn push_null(&mut self) {
        let idx = self.push_const(Constant::Null);
        self.chunk.instructions.push(Instruction::Const(idx));
    }
    pub(super) fn push_number(&mut self, n: f64) {
        let idx = self.push_const(Constant::Number(n));
        self.chunk.instructions.push(Instruction::Const(idx));
    }
    pub(super) fn push_string(&mut self, s: String) {
        let idx = self.push_const(Constant::String(s));
        self.chunk.instructions.push(Instruction::Const(idx));
    }
    pub(super) fn push_boolean(&mut self, b: bool) {
        let idx = self.push_const(Constant::Boolean(b));
        self.chunk.instructions.push(Instruction::Const(idx));
    }
    pub(super) fn emit_get_local(&mut self, slot: usize) {
        self.chunk.instructions.push(Instruction::GetLocal(slot));
    }

    // Hidden binding helper
    pub(super) fn bind_hidden_local(&mut self, name: String, slot: usize) {
        self.scopes.last_mut().unwrap().insert(name, slot);
    }

    // Error helper
    pub(super) fn error(&self, msg: &str) -> ! {
        panic!("Compiler error: {}", msg)
    }

    // Local const pool helper to keep consistency
    pub(super) fn push_const(&mut self, c: Constant) -> usize {
        self.chunk.constants.push(c);
        self.chunk.constants.len() - 1
    }
}
