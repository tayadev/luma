use super::compile::Compiler;
use super::ir::{Constant, Instruction};

// Hidden names and global helper identifiers
pub(super) const HIDDEN_MATCH_VAL: &str = "__match_val";
pub(super) const HIDDEN_DESTRUCTURE_VAL: &str = "__destructure_val";
pub(super) const HIDDEN_ITER: &str = "__iter";
pub(super) const HIDDEN_I: &str = "__i";
pub(super) const GLOBAL_ITER_FN: &str = "iter";

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
        panic!("Compiler error: {msg}")
    }

    // Shared: emit a single match arm; returns optional jump index to patch at end
    pub(super) fn emit_match_arm(
        &mut self,
        match_val_slot: usize,
        pattern: &crate::ast::Pattern,
        body: &[crate::ast::Stmt],
        is_last: bool,
    ) -> Option<usize> {
        use crate::ast::Pattern;
        use crate::bytecode::ir::{Constant, Instruction};

        let is_wildcard = matches!(pattern, Pattern::Wildcard { .. });
        let is_tag_pattern = matches!(pattern, Pattern::Ident { name, .. } if matches!(name.as_str(), "ok" | "err" | "some" | "none"));
        let is_catch_all =
            is_wildcard || (matches!(pattern, Pattern::Ident { name: _, .. }) && !is_tag_pattern);

        if !is_catch_all {
            match pattern {
                Pattern::Ident { name: tag, .. } => {
                    self.emit_get_local(match_val_slot);
                    let tag_idx =
                        super::compile::push_const(&mut self.chunk, Constant::String(tag.clone()));
                    self.chunk.instructions.push(Instruction::GetProp(tag_idx));
                    self.push_null();
                    self.chunk.instructions.push(Instruction::Ne);
                    let jf_next_arm = self.emit_jump_if_false();
                    let arm_body = super::compile::apply_implicit_return_to_arm(body);
                    for stmt in &arm_body {
                        self.emit_stmt(stmt);
                    }
                    let arm_preserves = super::compile::does_block_leave_value(&arm_body);
                    if !arm_preserves {
                        let null_idx = super::compile::push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                    }
                    let j = self.emit_jump();
                    let next_arm_ip = self.current_ip();
                    self.patch_jump(jf_next_arm, next_arm_ip);
                    Some(j)
                }
                Pattern::Literal { value: lit, .. } => {
                    self.emit_get_local(match_val_slot);
                    match lit {
                        crate::ast::Literal::Number(n) => self.push_number(*n),
                        crate::ast::Literal::String(s) => self.push_string(s.clone()),
                        crate::ast::Literal::Boolean(b) => self.push_boolean(*b),
                        crate::ast::Literal::Null => self.push_null(),
                    }
                    self.chunk.instructions.push(Instruction::Eq);
                    let jf_next_arm = self.emit_jump_if_false();
                    let arm_body = super::compile::apply_implicit_return_to_arm(body);
                    for stmt in &arm_body {
                        self.emit_stmt(stmt);
                    }
                    let arm_preserves = super::compile::does_block_leave_value(&arm_body);
                    if !arm_preserves {
                        self.push_null();
                    }
                    let j = self.emit_jump();
                    let next_arm_ip = self.current_ip();
                    self.patch_jump(jf_next_arm, next_arm_ip);
                    Some(j)
                }
                Pattern::ListPattern { .. } | Pattern::TablePattern { .. } => {
                    self.error("Structural patterns in match statements not yet fully supported");
                }
                _ => None,
            }
        } else {
            let arm_body = super::compile::apply_implicit_return_to_arm(body);
            for stmt in &arm_body {
                self.emit_stmt(stmt);
            }
            let arm_preserves = super::compile::does_block_leave_value(&arm_body);
            if !arm_preserves {
                self.push_null();
            }
            if !is_last {
                Some(self.emit_jump())
            } else {
                None
            }
        }
    }

    // Shared: destructuring for globals
    pub(super) fn emit_destructure_global(&mut self, pattern: &crate::ast::Pattern) {
        use crate::ast::Pattern;
        use crate::bytecode::ir::{Constant, Instruction};

        match pattern {
            Pattern::ListPattern { elements, rest, .. } => {
                for (i, elem_pattern) in elements.iter().enumerate() {
                    match elem_pattern {
                        Pattern::Ident { name, .. } => {
                            self.chunk.instructions.push(Instruction::Dup);
                            self.push_number(i as f64);
                            self.chunk.instructions.push(Instruction::GetIndex);
                            let name_idx = super::compile::push_const(
                                &mut self.chunk,
                                Constant::String(name.clone()),
                            );
                            self.chunk
                                .instructions
                                .push(Instruction::SetGlobal(name_idx));
                        }
                        Pattern::Wildcard { .. } => {}
                        _ => {
                            self.error("Nested destructuring patterns not yet supported");
                        }
                    }
                }
                if let Some(rest_name) = rest {
                    let start_index = elements.len();
                    self.chunk.instructions.push(Instruction::Dup);
                    self.chunk
                        .instructions
                        .push(Instruction::SliceList(start_index));
                    let name_idx = super::compile::push_const(
                        &mut self.chunk,
                        Constant::String(rest_name.clone()),
                    );
                    self.chunk
                        .instructions
                        .push(Instruction::SetGlobal(name_idx));
                } else {
                    self.chunk.instructions.push(Instruction::Pop);
                }
            }
            Pattern::TablePattern { fields, .. } => {
                for field in fields {
                    self.chunk.instructions.push(Instruction::Dup);
                    let key_idx = super::compile::push_const(
                        &mut self.chunk,
                        Constant::String(field.key.clone()),
                    );
                    self.chunk.instructions.push(Instruction::GetProp(key_idx));
                    let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                    let name_idx = super::compile::push_const(
                        &mut self.chunk,
                        Constant::String(binding_name.clone()),
                    );
                    self.chunk
                        .instructions
                        .push(Instruction::SetGlobal(name_idx));
                }
                self.chunk.instructions.push(Instruction::Pop);
            }
            Pattern::Ident { name, .. } => {
                let name_idx =
                    super::compile::push_const(&mut self.chunk, Constant::String(name.clone()));
                self.chunk
                    .instructions
                    .push(Instruction::SetGlobal(name_idx));
            }
            Pattern::Wildcard { .. } | Pattern::Literal { .. } => {
                self.chunk.instructions.push(Instruction::Pop);
            }
        }
    }

    // Shared: destructuring for locals
    pub(super) fn emit_destructure_local(
        &mut self,
        pattern: &crate::ast::Pattern,
        value_slot: usize,
    ) {
        use crate::ast::Pattern;
        use crate::bytecode::ir::{Constant, Instruction};

        match pattern {
            Pattern::ListPattern { elements, rest, .. } => {
                for (i, elem_pattern) in elements.iter().enumerate() {
                    match elem_pattern {
                        Pattern::Ident { name, .. } => {
                            self.chunk
                                .instructions
                                .push(Instruction::GetLocal(value_slot));
                            self.push_number(i as f64);
                            self.chunk.instructions.push(Instruction::GetIndex);
                            let elem_slot = self.local_count;
                            self.scopes
                                .last_mut()
                                .unwrap()
                                .insert(name.clone(), elem_slot);
                            self.local_count += 1;
                        }
                        Pattern::Wildcard { .. } => {}
                        _ => {
                            self.error("Nested destructuring patterns not yet supported");
                        }
                    }
                }
                if let Some(rest_name) = rest {
                    let start_index = elements.len();
                    self.chunk
                        .instructions
                        .push(Instruction::GetLocal(value_slot));
                    self.chunk
                        .instructions
                        .push(Instruction::SliceList(start_index));
                    let rest_slot = self.local_count;
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(rest_name.clone(), rest_slot);
                    self.local_count += 1;
                }
            }
            Pattern::TablePattern { fields, .. } => {
                for field in fields {
                    self.chunk
                        .instructions
                        .push(Instruction::GetLocal(value_slot));
                    let key_idx = super::compile::push_const(
                        &mut self.chunk,
                        Constant::String(field.key.clone()),
                    );
                    self.chunk.instructions.push(Instruction::GetProp(key_idx));
                    let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                    let key_slot = self.local_count;
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(binding_name.clone(), key_slot);
                    self.local_count += 1;
                }
            }
            Pattern::Ident { name, .. } => {
                let slot = self.local_count;
                self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                self.local_count += 1;
            }
            Pattern::Wildcard { .. } | Pattern::Literal { .. } => {
                self.local_count += 1;
            }
        }
    }

    // Local const pool helper bridging compile core
    pub(super) fn push_const(&mut self, c: Constant) -> usize {
        self.chunk.constants.push(c);
        self.chunk.constants.len() - 1
    }
}

// Shared: prepare for-loop pattern bindings (declare locals initialized to null)
// Returns a descriptor to use during loop iteration assignment.
pub(super) enum LoopPatDesc {
    Ident {
        slot: usize,
    },
    List {
        elem_slots: Vec<Option<usize>>,
        rest_slot: Option<usize>,
    },
}

impl Compiler {
    pub(super) fn prepare_loop_pattern(&mut self, pattern: &crate::ast::Pattern) -> LoopPatDesc {
        use crate::ast::Pattern;
        use crate::bytecode::ir::{Constant, Instruction};

        match pattern {
            Pattern::Ident { name: var_name, .. } => {
                let null_idx = super::compile::push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx));
                let slot = self.local_count;
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(var_name.clone(), slot);
                self.local_count += 1;
                LoopPatDesc::Ident { slot }
            }
            Pattern::ListPattern { elements, rest, .. } => {
                let mut elem_slots: Vec<Option<usize>> = Vec::with_capacity(elements.len());
                for elem in elements {
                    match elem {
                        Pattern::Ident { name, .. } => {
                            self.push_null();
                            let slot = self.local_count;
                            self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                            self.local_count += 1;
                            elem_slots.push(Some(slot));
                        }
                        Pattern::Wildcard { .. } => {
                            elem_slots.push(None);
                        }
                        _ => {
                            self.error("Nested patterns not yet supported in for destructuring");
                        }
                    }
                }
                let rest_slot = if let Some(rest_name) = rest {
                    self.push_null();
                    let slot = self.local_count;
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(rest_name.clone(), slot);
                    self.local_count += 1;
                    Some(slot)
                } else {
                    None
                };
                LoopPatDesc::List {
                    elem_slots,
                    rest_slot,
                }
            }
            _ => {
                self.error("Unsupported pattern in for loop");
            }
        }
    }

    // Shared: assign current iter value to loop pattern bindings
    pub(super) fn assign_loop_pattern_value(
        &mut self,
        desc: &LoopPatDesc,
        iter_slot: usize,
        i_slot: usize,
    ) {
        use crate::bytecode::ir::Instruction;
        match desc {
            LoopPatDesc::Ident { slot } => {
                self.chunk
                    .instructions
                    .push(Instruction::GetLocal(iter_slot));
                self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                self.chunk.instructions.push(Instruction::GetIndex);
                self.chunk.instructions.push(Instruction::SetLocal(*slot));
            }
            LoopPatDesc::List {
                elem_slots,
                rest_slot,
            } => {
                self.chunk
                    .instructions
                    .push(Instruction::GetLocal(iter_slot));
                self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                self.chunk.instructions.push(Instruction::GetIndex);
                for (idx, slot_opt) in elem_slots.iter().enumerate() {
                    if let Some(slot) = slot_opt {
                        self.chunk.instructions.push(Instruction::Dup);
                        self.push_number(idx as f64);
                        self.chunk.instructions.push(Instruction::GetIndex);
                        self.chunk.instructions.push(Instruction::SetLocal(*slot));
                    }
                }
                if let Some(rest_slot) = rest_slot {
                    let start_index = elem_slots.len();
                    self.chunk
                        .instructions
                        .push(Instruction::SliceList(start_index));
                    self.chunk
                        .instructions
                        .push(Instruction::SetLocal(*rest_slot));
                } else {
                    self.chunk.instructions.push(Instruction::Pop);
                }
            }
        }
    }
}
