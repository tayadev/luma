//! Bytecode compiler for Luma.
//!
//! This module implements the compilation of Luma's AST into bytecode instructions.
//! The compiler performs two passes:
//!
//! 1. **Pre-declaration pass**: Scans top-level function declarations and registers them
//!    in the global scope. This enables mutual recursion between functions.
//!
//! 2. **Compilation pass**: Traverses the AST and emits bytecode instructions. This includes:
//!    - Expression compilation with proper operator precedence
//!    - Statement compilation with control flow handling
//!    - Closure capture via upvalue descriptors
//!    - Pattern matching and destructuring
//!    - Loop compilation with break/continue support
//!
//! The compiler uses a stack-based virtual machine model where most operations
//! work on values at the top of the stack.

use super::ir::{Chunk, Constant, Instruction, UpvalueDescriptor};
use crate::ast::{Argument, Expr, Program, Stmt};
use std::collections::HashMap;

pub fn compile_program(program: &Program) -> Chunk {
    let mut c = Compiler::new("<program>");

    // Pre-scan: record top-level function parameter names for named-arg reordering
    let mut global_fn_params: HashMap<String, Vec<String>> = HashMap::new();
    for stmt in &program.statements {
        if let Stmt::VarDecl { name, value, .. } = stmt
            && let Expr::Function { arguments, .. } = value
        {
            let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
            global_fn_params.insert(name.clone(), params);
        }
    }
    c.global_fn_params = global_fn_params;

    // First pass: Pre-register all top-level let/var declarations with null placeholders
    for stmt in &program.statements {
        if let Stmt::VarDecl { name, .. } = stmt {
            let null_idx = push_const(&mut c.chunk, Constant::Null);
            c.chunk.instructions.push(Instruction::Const(null_idx));
            let name_idx = push_const(&mut c.chunk, Constant::String(name.clone()));
            c.chunk.instructions.push(Instruction::SetGlobal(name_idx));
        }
    }

    // Second pass: compile and initialize all statements
    for stmt in &program.statements {
        c.emit_stmt(stmt);
    }
    c.chunk.instructions.push(Instruction::Halt);
    c.chunk.clone()
}

pub(super) fn push_const(chunk: &mut Chunk, c: Constant) -> usize {
    chunk.constants.push(c);
    chunk.constants.len() - 1
}

pub(super) struct LoopContext {
    pub(super) break_patches: Vec<usize>,
    pub(super) continue_patches: Vec<usize>,
    pub(super) local_count: usize,
    pub(super) continue_target: Option<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct UpvalueInfo {
    pub(super) descriptor: UpvalueDescriptor,
    pub(super) _name: String,
}

pub(super) struct Compiler {
    pub(super) chunk: Chunk,
    pub(super) scopes: Vec<HashMap<String, usize>>,
    pub(super) local_count: usize,
    pub(super) loop_stack: Vec<LoopContext>,
    pub(super) upvalues: Vec<UpvalueInfo>,
    pub(super) parent: Option<Box<Compiler>>,
    pub(super) param_scopes: Vec<HashMap<String, Vec<String>>>,
    pub(super) global_fn_params: HashMap<String, Vec<String>>,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self {
            chunk: Chunk {
                name: name.to_string(),
                ..Default::default()
            },
            scopes: Vec::new(),
            local_count: 0,
            loop_stack: Vec::new(),
            upvalues: Vec::new(),
            parent: None,
            param_scopes: Vec::new(),
            global_fn_params: HashMap::new(),
        }
    }
    fn new_with_parent(name: &str, parent: Compiler) -> Self {
        Self {
            chunk: Chunk {
                name: name.to_string(),
                ..Default::default()
            },
            scopes: Vec::new(),
            local_count: 0,
            loop_stack: Vec::new(),
            upvalues: Vec::new(),
            parent: Some(Box::new(parent)),
            param_scopes: Vec::new(),
            global_fn_params: HashMap::new(),
        }
    }
    pub(super) fn emit_stmt(&mut self, s: &Stmt) {
        super::emit_stmt::emit_stmt(self, s);
    }

    // emit_expr moved to emit_expr.rs

    pub(super) fn emit_jump_if_false(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk
            .instructions
            .push(Instruction::JumpIfFalse(usize::MAX));
        pos
    }
    pub(super) fn emit_jump(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::Jump(usize::MAX));
        pos
    }
    pub(super) fn patch_jump(&mut self, at: usize, target: usize) {
        match self.chunk.instructions.get_mut(at) {
            Some(Instruction::JumpIfFalse(addr)) => *addr = target,
            Some(Instruction::Jump(addr)) => *addr = target,
            _ => {}
        }
    }
    pub(super) fn current_ip(&self) -> usize {
        self.chunk.instructions.len()
    }

    // helpers are implemented in helpers.rs as impl Compiler

    pub(super) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.param_scopes.push(HashMap::new());
    }

    pub(super) fn exit_scope_with_preserve(&mut self, preserve_top: bool) {
        if let Some(scope) = self.scopes.pop() {
            let to_pop = scope.len();
            if to_pop > 0 {
                if preserve_top {
                    self.chunk
                        .instructions
                        .push(Instruction::PopNPreserve(to_pop));
                } else {
                    for _ in 0..to_pop {
                        self.chunk.instructions.push(Instruction::Pop);
                    }
                }
            }
            self.local_count = self.local_count.saturating_sub(to_pop);
        }
        self.param_scopes.pop();
    }

    pub(super) fn lookup_local(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    /// Lookup parameter names for a function variable by identifier name.
    /// Searches current and outer param scopes, then global map, then parent's chain.
    pub(super) fn lookup_fn_params(&self, name: &str) -> Option<Vec<String>> {
        for scope in self.param_scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        if let Some(v) = self.global_fn_params.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.lookup_fn_params(name);
        }
        None
    }

    /// Resolve an upvalue by searching parent scopes.
    /// Returns the upvalue index if found, None otherwise.
    pub(super) fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        // If there's no parent, we can't have upvalues
        let parent = self.parent.as_mut()?;

        // First check if the variable is a local in the parent
        if let Some(slot) = parent.lookup_local(name) {
            // It's a local in the parent - capture it
            let descriptor = UpvalueDescriptor::Local(slot);
            return Some(self.add_upvalue(descriptor, name.to_string()));
        }

        // Otherwise, try to resolve it as an upvalue in the parent
        if let Some(parent_upvalue_idx) = parent.resolve_upvalue(name) {
            // It's an upvalue in the parent - capture that upvalue
            let descriptor = UpvalueDescriptor::Upvalue(parent_upvalue_idx);
            return Some(self.add_upvalue(descriptor, name.to_string()));
        }

        None
    }

    /// Add an upvalue to this function's upvalue list.
    /// Returns the index of the upvalue.
    /// If the upvalue already exists, returns its existing index.
    fn add_upvalue(&mut self, descriptor: UpvalueDescriptor, name: String) -> usize {
        // Check if we already have this upvalue
        for (i, uv) in self.upvalues.iter().enumerate() {
            match (&uv.descriptor, &descriptor) {
                (UpvalueDescriptor::Local(a), UpvalueDescriptor::Local(b)) if a == b => return i,
                (UpvalueDescriptor::Upvalue(a), UpvalueDescriptor::Upvalue(b)) if a == b => {
                    return i;
                }
                _ => {}
            }
        }

        // Add new upvalue
        let idx = self.upvalues.len();
        self.upvalues.push(UpvalueInfo {
            descriptor,
            _name: name,
        });
        idx
    }

    /// Compile a nested function with access to parent scope for closures.
    /// This is done by temporarily moving self to become the parent of a new compiler.
    pub(super) fn compile_nested_function(
        &mut self,
        arguments: &[Argument],
        body: &[Stmt],
    ) -> (Chunk, Vec<UpvalueDescriptor>) {
        // We need to create a new compiler with self as parent, but we can't move self
        // Instead, we'll use std::mem::replace to temporarily swap self with a dummy
        let parent = std::mem::replace(self, Compiler::new("__temp__"));

        // Create nested compiler with parent
        let mut nested = Compiler::new_with_parent("<function>", parent);
        let arity = arguments.len();

        // Enter scope for function parameters
        nested.enter_scope();
        // Parameters become locals in order
        for arg in arguments {
            let slot = nested.local_count;
            nested
                .scopes
                .last_mut()
                .unwrap()
                .insert(arg.name.clone(), slot);
            nested.local_count += 1;
        }
        // Predeclare local function names within function body for mutual recursion
        nested.predeclare_function_locals(body);

        // Compile body
        for stmt in body {
            nested.emit_stmt(stmt);
        }

        // Exit scope
        nested.exit_scope_with_preserve(does_block_leave_value(body));
        nested.chunk.instructions.push(Instruction::Return);
        nested.chunk.local_count = arity as u16;

        // Extract upvalue descriptors and chunk
        let upvalue_descriptors: Vec<UpvalueDescriptor> = nested
            .upvalues
            .iter()
            .map(|uv| uv.descriptor.clone())
            .collect();
        nested.chunk.upvalue_descriptors = upvalue_descriptors.clone();
        let chunk = nested.chunk.clone();

        // Restore self from parent
        let parent = nested.parent.take().unwrap();
        *self = *parent;

        (chunk, upvalue_descriptors)
    }

    /// Predeclare local function names in the current scope so mutually-recursive
    /// functions can reference each other as locals (not fall back to globals).
    /// Also records parameter name lists for named-arg reordering.
    pub(super) fn predeclare_function_locals(&mut self, stmts: &[Stmt]) {
        if self.scopes.is_empty() {
            return;
        }
        for stmt in stmts {
            if let Stmt::VarDecl { name, value, .. } = stmt
                && let Expr::Function { arguments, .. } = value
            {
                // Skip if already declared in this scope (avoid duplicates)
                let already = self.scopes.last().and_then(|m| m.get(name)).is_some();
                if !already {
                    // Initialize slot with Null to occupy stack/local
                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                    let slot = self.local_count;
                    self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                    self.local_count += 1;
                }
                // Record param names for named-arg reordering in this scope
                if let Some(scope) = self.param_scopes.last_mut() {
                    let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                    scope.insert(name.clone(), params);
                }
            }
        }
    }
}

/// Applies implicit return to a match arm body by converting trailing ExprStmt to Return
pub(super) fn apply_implicit_return_to_arm(body: &[Stmt]) -> Vec<Stmt> {
    let mut arm_body = body.to_vec();
    if let Some(last) = arm_body.pop() {
        match last {
            Stmt::ExprStmt { expr: e, .. } => arm_body.push(Stmt::Return {
                value: e,
                span: None,
            }),
            other => arm_body.push(other),
        }
    }
    arm_body
}

pub(super) fn does_block_leave_value(block: &[Stmt]) -> bool {
    match block.last() {
        Some(Stmt::Return { .. }) => true,
        Some(Stmt::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        }) => {
            // An if-statement leaves a value if all branches leave a value
            let then_leaves = does_block_leave_value(then_block);
            let elif_leave = elif_blocks.iter().all(|(_, b)| does_block_leave_value(b));
            let else_leaves = else_block
                .as_ref()
                .is_some_and(|b| does_block_leave_value(b));
            then_leaves && elif_leave && else_leaves
        }
        _ => false,
    }
}
