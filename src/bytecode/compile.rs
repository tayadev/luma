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
use crate::ast::{
    Argument, BinaryOp, CallArgument, Expr, LogicalOp, Pattern, Program, Stmt, TableKey, UnaryOp,
};
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
    // This allows recursive functions to reference themselves
    for stmt in &program.statements {
        if let Stmt::VarDecl { name, .. } = stmt {
            // Emit null and set the global to reserve the name
            let null_idx = push_const(&mut c.chunk, Constant::Null);
            c.chunk.instructions.push(Instruction::Const(null_idx));
            let name_idx = push_const(&mut c.chunk, Constant::String(name.clone()));
            c.chunk.instructions.push(Instruction::SetGlobal(name_idx));
        }
    }

    // Second pass: Actually compile and initialize all statements
    for stmt in &program.statements {
        c.emit_stmt(stmt);
    }
    c.chunk.instructions.push(Instruction::Halt);
    c.chunk.clone()
}

fn push_const(chunk: &mut Chunk, c: Constant) -> usize {
    chunk.constants.push(c);
    chunk.constants.len() - 1
}

struct LoopContext {
    break_patches: Vec<usize>,    // IPs of break jumps to patch when loop exits
    continue_patches: Vec<usize>, // IPs of continue jumps to patch (for for-loops)
    local_count: usize,           // Number of locals when this loop started
    continue_target: Option<usize>, // Explicit continue target (set after body for for-loops)
}

/// Tracks upvalues for a function being compiled
#[derive(Debug, Clone)]
struct UpvalueInfo {
    /// Which local or upvalue in the enclosing function this upvalue captures
    descriptor: UpvalueDescriptor,
    /// Name of the variable being captured (useful for debugging and error messages)
    _name: String,
}

struct Compiler {
    chunk: Chunk,
    scopes: Vec<HashMap<String, usize>>, // name -> slot index
    local_count: usize,
    loop_stack: Vec<LoopContext>, // Track nested loops for break/continue
    upvalues: Vec<UpvalueInfo>,   // Upvalues captured by this function
    parent: Option<Box<Compiler>>, // Parent compiler (for nested functions)
    // Track function parameter names per scope for named arguments
    param_scopes: Vec<HashMap<String, Vec<String>>>,
    // Top-level function parameter names
    global_fn_params: HashMap<String, Vec<String>>,
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

    fn emit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Return { value, .. } => {
                self.emit_expr(value);
            }
            Stmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                // if cond then ... elif ... else ... end
                // Always leave exactly one value on stack as the if-expression result.
                // Start with a default Null (used when no branch produces a value).
                let null_idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx));
                self.emit_expr(condition);
                let jf_main = self.emit_jump_if_false();

                // then block
                self.enter_scope();
                // Predeclare local function names for mutual recursion in then-block
                self.predeclare_function_locals(then_block);
                // Remove default Null and compute real value for this branch
                self.chunk.instructions.push(Instruction::Pop);
                for st in then_block {
                    self.emit_stmt(st);
                }
                let then_preserve = does_block_leave_value(then_block);
                self.exit_scope_with_preserve(then_preserve);
                if !then_preserve {
                    // branch produced no value; push Null to keep if result arity
                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                }
                // Track all end jumps (from then/elif branches) to patch to final end
                let mut end_jumps: Vec<usize> = Vec::new();
                end_jumps.push(self.emit_jump());

                // else/elif chain
                let mut next_start = self.current_ip();
                self.patch_jump(jf_main, next_start);

                // elif blocks
                for (elif_cond, elif_body) in elif_blocks {
                    self.emit_expr(elif_cond);
                    let jf_elif = self.emit_jump_if_false();
                    self.enter_scope();
                    // Predeclare local function names for mutual recursion in elif-body
                    self.predeclare_function_locals(elif_body);
                    // Remove default Null and compute real value for this branch
                    self.chunk.instructions.push(Instruction::Pop);
                    for st in elif_body {
                        self.emit_stmt(st);
                    }
                    let elif_preserve = does_block_leave_value(elif_body);
                    self.exit_scope_with_preserve(elif_preserve);
                    if !elif_preserve {
                        let null_idx = push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                    }
                    // Record this branch's end jump for later patching
                    end_jumps.push(self.emit_jump());
                    // patch jf_elif to the next block start
                    next_start = self.current_ip();
                    self.patch_jump(jf_elif, next_start);
                    // leave j_after_elif as placeholder to be patched to end later
                }

                // else block
                if let Some(else_body) = else_block {
                    self.enter_scope();
                    // Predeclare local function names for mutual recursion in else-body
                    self.predeclare_function_locals(else_body);
                    // Remove default Null and compute real value for this branch
                    self.chunk.instructions.push(Instruction::Pop);
                    for st in else_body {
                        self.emit_stmt(st);
                    }
                    let else_preserve = does_block_leave_value(else_body);
                    self.exit_scope_with_preserve(else_preserve);
                    if !else_preserve {
                        let null_idx = push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                    }
                }

                // Patch final end for main then and all elif end jumps
                let end_ip = self.current_ip();
                for j in end_jumps {
                    self.patch_jump(j, end_ip);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                let loop_start = self.current_ip();

                // Register this loop for break/continue tracking
                self.loop_stack.push(LoopContext {
                    break_patches: Vec::new(),
                    continue_patches: Vec::new(),
                    local_count: self.local_count,
                    continue_target: Some(loop_start), // For while loops, continue jumps to start (condition check)
                });

                self.emit_expr(condition);
                let jf_end = self.emit_jump_if_false();
                self.enter_scope();
                // Predeclare local function names for mutual recursion in while-body
                self.predeclare_function_locals(body);
                for st in body {
                    self.emit_stmt(st);
                }
                self.exit_scope_with_preserve(false);
                // jump back to loop start
                let start = loop_start;
                self.chunk.instructions.push(Instruction::Jump(start));
                let end_ip = self.current_ip();
                self.patch_jump(jf_end, end_ip);

                // Patch all break statements to jump to end_ip
                let loop_ctx = self.loop_stack.pop().unwrap();
                for break_ip in loop_ctx.break_patches {
                    self.patch_jump(break_ip, end_ip);
                }
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                let loop_start = self.current_ip();

                // Register this loop for break/continue tracking
                self.loop_stack.push(LoopContext {
                    break_patches: Vec::new(),
                    continue_patches: Vec::new(),
                    local_count: self.local_count,
                    continue_target: Some(loop_start), // For do-while loops, continue jumps to start (body start)
                });

                self.enter_scope();
                // Predeclare local function names for mutual recursion in do-while body
                self.predeclare_function_locals(body);
                for st in body {
                    self.emit_stmt(st);
                }
                self.exit_scope_with_preserve(false);
                // evaluate condition
                self.emit_expr(condition);
                // if true, jump back to loop start
                let jf_end = self.emit_jump_if_false();
                self.chunk.instructions.push(Instruction::Jump(loop_start));
                let end_ip = self.current_ip();
                self.patch_jump(jf_end, end_ip);

                // Patch all break statements to jump to end_ip
                let loop_ctx = self.loop_stack.pop().unwrap();
                for break_ip in loop_ctx.break_patches {
                    self.patch_jump(break_ip, end_ip);
                }
            }
            Stmt::Match { expr, arms, .. } => {
                // Evaluate the match expression once and store in a hidden local
                self.enter_scope();
                self.emit_expr(expr);
                let match_val_slot = self.local_count;
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("__match_val".to_string(), match_val_slot);
                self.local_count += 1;

                let mut end_jumps = Vec::new();

                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let is_wildcard = matches!(pattern, Pattern::Wildcard { .. });
                    // Ident patterns can be catch-all or tag patterns depending on the name
                    let is_tag_pattern = matches!(pattern, Pattern::Ident { name, .. } if matches!(name.as_str(), "ok" | "err" | "some" | "none"));
                    let is_catch_all = is_wildcard
                        || (matches!(pattern, Pattern::Ident { name: _, .. }) && !is_tag_pattern);

                    if !is_catch_all {
                        // Check if pattern matches
                        match pattern {
                            Pattern::Ident { name: tag, .. } => {
                                // Tag pattern: Check if the match value has this tag/property
                                self.chunk
                                    .instructions
                                    .push(Instruction::GetLocal(match_val_slot));
                                let tag_idx =
                                    push_const(&mut self.chunk, Constant::String(tag.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(tag_idx));
                                // If property exists (not null), this arm matches
                                let null_idx = push_const(&mut self.chunk, Constant::Null);
                                self.chunk.instructions.push(Instruction::Const(null_idx));
                                self.chunk.instructions.push(Instruction::Ne); // property != null

                                let jf_next_arm = self.emit_jump_if_false();

                                // Execute this arm's body
                                let arm_body = apply_implicit_return_to_arm(body);

                                for stmt in &arm_body {
                                    self.emit_stmt(stmt);
                                }

                                let arm_preserves = does_block_leave_value(&arm_body);
                                if !arm_preserves {
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                }

                                end_jumps.push(self.emit_jump());

                                let next_arm_ip = self.current_ip();
                                self.patch_jump(jf_next_arm, next_arm_ip);
                            }
                            Pattern::Literal { value: lit, .. } => {
                                // Check if the match value equals this literal
                                self.chunk
                                    .instructions
                                    .push(Instruction::GetLocal(match_val_slot));
                                match lit {
                                    crate::ast::Literal::Number(n) => {
                                        let idx = push_const(&mut self.chunk, Constant::Number(*n));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::String(s) => {
                                        let idx = push_const(
                                            &mut self.chunk,
                                            Constant::String(s.clone()),
                                        );
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::Boolean(b) => {
                                        let idx =
                                            push_const(&mut self.chunk, Constant::Boolean(*b));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::Null => {
                                        let idx = push_const(&mut self.chunk, Constant::Null);
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                }
                                self.chunk.instructions.push(Instruction::Eq);

                                let jf_next_arm = self.emit_jump_if_false();

                                let arm_body = apply_implicit_return_to_arm(body);

                                for stmt in &arm_body {
                                    self.emit_stmt(stmt);
                                }

                                let arm_preserves = does_block_leave_value(&arm_body);
                                if !arm_preserves {
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                }

                                end_jumps.push(self.emit_jump());

                                let next_arm_ip = self.current_ip();
                                self.patch_jump(jf_next_arm, next_arm_ip);
                            }
                            Pattern::ListPattern { .. } | Pattern::TablePattern { .. } => {
                                // Structural patterns not yet fully supported in match
                                panic!(
                                    "Compiler error: Structural patterns in match statements not yet fully supported"
                                );
                            }
                            _ => {}
                        }
                    } else {
                        // Wildcard or Ident pattern - always matches (catch-all case)
                        // Handle implicit return
                        let arm_body = apply_implicit_return_to_arm(body);

                        for stmt in &arm_body {
                            self.emit_stmt(stmt);
                        }

                        let arm_preserves = does_block_leave_value(&arm_body);
                        if !arm_preserves {
                            let null_idx = push_const(&mut self.chunk, Constant::Null);
                            self.chunk.instructions.push(Instruction::Const(null_idx));
                        }

                        // No need for end jump if this is the last arm
                        if i < arms.len() - 1 {
                            end_jumps.push(self.emit_jump());
                        }
                    }
                }

                // Patch all end jumps to point here
                let end_ip = self.current_ip();
                for jump_pos in end_jumps {
                    self.patch_jump(jump_pos, end_ip);
                }

                // Exit the match scope (pop __match_val but preserve result)
                self.exit_scope_with_preserve(true);
            }
            Stmt::VarDecl { name, value, .. } => {
                if self.scopes.is_empty() {
                    // global
                    self.emit_expr(value);
                    let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                    self.chunk
                        .instructions
                        .push(Instruction::SetGlobal(name_idx));
                    // Record global function parameter names if value is function
                    if let Expr::Function { arguments, .. } = value {
                        let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                        self.global_fn_params.insert(name.clone(), params);
                    }
                } else {
                    // local
                    if let Some(&slot) = self.scopes.last().and_then(|m| m.get(name)) {
                        // Predeclared local exists: just initialize it
                        self.emit_expr(value);
                        self.chunk.instructions.push(Instruction::SetLocal(slot));
                    } else {
                        // Not predeclared: allocate new local slot; initializer stays on stack
                        self.emit_expr(value);
                        let slot = self.local_count;
                        self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                        self.local_count += 1;
                    }
                    // Record local function parameter names if value is function
                    if let Expr::Function { arguments, .. } = value {
                        let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                        if let Some(scope) = self.param_scopes.last_mut() {
                            scope.insert(name.clone(), params);
                        }
                    }
                }
            }
            Stmt::DestructuringVarDecl {
                mutable: _,
                pattern,
                value,
                ..
            } => {
                // Emit the value expression once
                self.emit_expr(value);

                // For now, the value is on the stack. We need to destructure it.
                if self.scopes.is_empty() {
                    // Global destructuring
                    match pattern {
                        Pattern::ListPattern { elements, rest, .. } => {
                            // List destructuring: [a, b, ...rest] = list
                            // Strategy: Get each element by index and assign to globals

                            // Dup the list for each element access
                            for (i, elem_pattern) in elements.iter().enumerate() {
                                match elem_pattern {
                                    Pattern::Ident { name, .. } => {
                                        self.chunk.instructions.push(Instruction::Dup); // dup list
                                        let idx =
                                            push_const(&mut self.chunk, Constant::Number(i as f64));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                        self.chunk.instructions.push(Instruction::GetIndex);
                                        let name_idx = push_const(
                                            &mut self.chunk,
                                            Constant::String(name.clone()),
                                        );
                                        self.chunk
                                            .instructions
                                            .push(Instruction::SetGlobal(name_idx));
                                    }
                                    Pattern::Wildcard { .. } => {
                                        // Wildcard element: don't extract, don't bind
                                    }
                                    _ => {
                                        panic!("Nested destructuring patterns not yet supported");
                                    }
                                }
                            }

                            // Handle rest pattern (e.g., ...tail)
                            if let Some(rest_name) = rest {
                                // Build list slice from elements.len() onwards
                                let start_index = elements.len();
                                self.chunk.instructions.push(Instruction::Dup); // dup list
                                self.chunk
                                    .instructions
                                    .push(Instruction::SliceList(start_index));
                                let name_idx = push_const(
                                    &mut self.chunk,
                                    Constant::String(rest_name.clone()),
                                );
                                self.chunk
                                    .instructions
                                    .push(Instruction::SetGlobal(name_idx));
                            } else {
                                // No rest, pop the list
                                self.chunk.instructions.push(Instruction::Pop);
                            }
                        }
                        Pattern::TablePattern { fields, .. } => {
                            // Table destructuring: {name, age: userAge} = table
                            for field in fields {
                                self.chunk.instructions.push(Instruction::Dup); // dup table
                                let key_idx = push_const(
                                    &mut self.chunk,
                                    Constant::String(field.key.clone()),
                                );
                                self.chunk.instructions.push(Instruction::GetProp(key_idx));
                                let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                                let name_idx = push_const(
                                    &mut self.chunk,
                                    Constant::String(binding_name.clone()),
                                );
                                self.chunk
                                    .instructions
                                    .push(Instruction::SetGlobal(name_idx));
                            }
                            self.chunk.instructions.push(Instruction::Pop); // pop table
                        }
                        Pattern::Ident { name, .. } => {
                            // Simple binding, just assign
                            let name_idx =
                                push_const(&mut self.chunk, Constant::String(name.clone()));
                            self.chunk
                                .instructions
                                .push(Instruction::SetGlobal(name_idx));
                        }
                        Pattern::Wildcard { .. } => {
                            // Wildcard doesn't bind, just pop the value
                            self.chunk.instructions.push(Instruction::Pop);
                        }
                        Pattern::Literal { value: _, .. } => {
                            // Literal patterns don't bind variables in destructuring context
                            self.chunk.instructions.push(Instruction::Pop);
                        }
                    }
                } else {
                    // Local destructuring
                    match pattern {
                        Pattern::ListPattern { elements, rest, .. } => {
                            // For locals, the value is already on stack and will become local slots
                            let value_slot = self.local_count;
                            self.scopes
                                .last_mut()
                                .unwrap()
                                .insert("__destructure_val".to_string(), value_slot);
                            self.local_count += 1;

                            // Extract each element into its own local
                            for (i, elem_pattern) in elements.iter().enumerate() {
                                match elem_pattern {
                                    Pattern::Ident { name, .. } => {
                                        self.chunk
                                            .instructions
                                            .push(Instruction::GetLocal(value_slot));
                                        let idx =
                                            push_const(&mut self.chunk, Constant::Number(i as f64));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                        self.chunk.instructions.push(Instruction::GetIndex);

                                        let elem_slot = self.local_count;
                                        self.scopes
                                            .last_mut()
                                            .unwrap()
                                            .insert(name.clone(), elem_slot);
                                        self.local_count += 1;
                                    }
                                    Pattern::Wildcard { .. } => {
                                        // Wildcard element: don't extract, don't bind
                                        // No need to emit any instructions
                                    }
                                    _ => {
                                        // Nested patterns not yet supported
                                        panic!("Nested destructuring patterns not yet supported");
                                    }
                                }
                            }

                            // Handle rest
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
                            let value_slot = self.local_count;
                            self.scopes
                                .last_mut()
                                .unwrap()
                                .insert("__destructure_val".to_string(), value_slot);
                            self.local_count += 1;

                            for field in fields {
                                self.chunk
                                    .instructions
                                    .push(Instruction::GetLocal(value_slot));
                                let key_idx = push_const(
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
                        Pattern::Wildcard { .. } => {
                            // Wildcard doesn't bind, the value stays on stack as an unused local
                            // We still need to account for it in local_count
                            self.local_count += 1;
                        }
                        Pattern::Literal { value: _, .. } => {
                            // Literal patterns don't bind in destructuring context
                            self.local_count += 1;
                        }
                    }
                }
            }
            Stmt::Assignment {
                target,
                op: _,
                value,
                ..
            } => {
                match target {
                    Expr::Identifier { name, .. } => {
                        self.emit_expr(value);
                        if let Some(slot) = self.lookup_local(name) {
                            self.chunk.instructions.push(Instruction::SetLocal(slot));
                        } else if let Some(upvalue_idx) = self.resolve_upvalue(name) {
                            self.chunk
                                .instructions
                                .push(Instruction::SetUpvalue(upvalue_idx));
                        } else {
                            let name_idx =
                                push_const(&mut self.chunk, Constant::String(name.clone()));
                            self.chunk
                                .instructions
                                .push(Instruction::SetGlobal(name_idx));
                        }
                    }
                    Expr::MemberAccess { object, member, .. } => {
                        // Stack layout: object, value
                        self.emit_expr(object);
                        self.emit_expr(value);
                        let name_idx =
                            push_const(&mut self.chunk, Constant::String(member.clone()));
                        self.chunk.instructions.push(Instruction::SetProp(name_idx));
                    }
                    Expr::Index { object, index, .. } => {
                        // Stack layout: object, index, value
                        self.emit_expr(object);
                        self.emit_expr(index);
                        self.emit_expr(value);
                        self.chunk.instructions.push(Instruction::SetIndex);
                    }
                    _ => {
                        // Invalid assignment target - should be caught by type checker
                    }
                }
            }
            // TODO: other statements in MVP
            Stmt::For {
                pattern,
                iterator,
                body,
                ..
            } => {
                // Lower: for pattern in iterator do body end
                // To:
                //   let __iter = iterator
                //   let __i = 0
                //   [declare pattern locals]
                //   while __i < len(__iter) do
                //     [bind pattern from __iter[__i]]
                //     body
                //     __i = __i + 1
                //   end

                // Create a new scope for the entire loop (including iterator, index, and pattern locals)
                self.enter_scope();

                // Evaluate iterator via global iter() helper and store in a hidden local __iter
                let iter_name_idx =
                    push_const(&mut self.chunk, Constant::String("iter".to_string()));
                self.chunk
                    .instructions
                    .push(Instruction::GetGlobal(iter_name_idx));
                self.emit_expr(iterator);
                self.chunk.instructions.push(Instruction::Call(1));
                let iter_slot = self.local_count;
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("__iter".to_string(), iter_slot);
                self.local_count += 1;

                // Initialize __i = 0
                let zero_idx = push_const(&mut self.chunk, Constant::Number(0.0));
                self.chunk.instructions.push(Instruction::Const(zero_idx));
                let i_slot = self.local_count;
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("__i".to_string(), i_slot);
                self.local_count += 1;

                // Predeclare pattern locals if needed
                enum LoopPat {
                    Ident {
                        slot: usize,
                    },
                    List {
                        elem_slots: Vec<Option<usize>>,
                        rest_slot: Option<usize>,
                    },
                }
                let loop_pat = match pattern {
                    Pattern::Ident { name: var_name, .. } => {
                        // Allocate slot for loop variable (initialized to null)
                        let null_idx = push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                        let slot = self.local_count;
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .insert(var_name.clone(), slot);
                        self.local_count += 1;
                        LoopPat::Ident { slot }
                    }
                    Pattern::ListPattern { elements, rest, .. } => {
                        // Predeclare locals for each identifier element and optional rest
                        let mut elem_slots: Vec<Option<usize>> = Vec::with_capacity(elements.len());
                        for elem in elements {
                            match elem {
                                Pattern::Ident { name, .. } => {
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                    let slot = self.local_count;
                                    self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                                    self.local_count += 1;
                                    elem_slots.push(Some(slot));
                                }
                                Pattern::Wildcard { .. } => {
                                    elem_slots.push(None);
                                }
                                _ => {
                                    panic!(
                                        "Compiler error: Nested patterns not yet supported in for destructuring"
                                    );
                                }
                            }
                        }
                        let rest_slot = if let Some(rest_name) = rest {
                            let null_idx = push_const(&mut self.chunk, Constant::Null);
                            self.chunk.instructions.push(Instruction::Const(null_idx));
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
                        LoopPat::List {
                            elem_slots,
                            rest_slot,
                        }
                    }
                    _ => {
                        panic!("Compiler error: Unsupported pattern in for loop");
                    }
                };

                // while __i < len(__iter) do
                let loop_start = self.current_ip();

                // Register this loop for break/continue tracking
                let loop_ctx_idx = self.loop_stack.len();
                self.loop_stack.push(LoopContext {
                    break_patches: Vec::new(),
                    continue_patches: Vec::new(),
                    local_count: self.local_count,
                    continue_target: None, // Will be set after body
                });

                // Load __i
                self.chunk.instructions.push(Instruction::GetLocal(i_slot));

                // Load __iter and get its length
                self.chunk
                    .instructions
                    .push(Instruction::GetLocal(iter_slot));
                self.chunk.instructions.push(Instruction::GetLen);

                // Compare __i < len(__iter)
                self.chunk.instructions.push(Instruction::Lt);

                // If false, jump to end
                let jf_end = self.emit_jump_if_false();

                // Bind current element according to pattern
                match &loop_pat {
                    LoopPat::Ident { slot } => {
                        self.chunk
                            .instructions
                            .push(Instruction::GetLocal(iter_slot));
                        self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                        self.chunk.instructions.push(Instruction::GetIndex);
                        self.chunk.instructions.push(Instruction::SetLocal(*slot));
                    }
                    LoopPat::List {
                        elem_slots,
                        rest_slot,
                    } => {
                        // Load current element once
                        self.chunk
                            .instructions
                            .push(Instruction::GetLocal(iter_slot));
                        self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                        self.chunk.instructions.push(Instruction::GetIndex); // stack: elem

                        // For each element binding, extract by index
                        for (idx, slot_opt) in elem_slots.iter().enumerate() {
                            if let Some(slot) = slot_opt {
                                self.chunk.instructions.push(Instruction::Dup); // dup elem
                                let i_idx =
                                    push_const(&mut self.chunk, Constant::Number(idx as f64));
                                self.chunk.instructions.push(Instruction::Const(i_idx));
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
                            // No rest: pop the elem value
                            self.chunk.instructions.push(Instruction::Pop);
                        }
                    }
                }

                // Execute body
                // Predeclare local function names for mutual recursion in for-body
                self.predeclare_function_locals(body);
                for stmt in body {
                    self.emit_stmt(stmt);
                }

                // Increment section - this is where continue should jump to
                let continue_target = self.current_ip();

                // Patch all continue statements in this loop to jump here
                let loop_ctx = &self.loop_stack[loop_ctx_idx];
                let continue_ips = loop_ctx.continue_patches.clone();
                for continue_ip in continue_ips {
                    self.patch_jump(continue_ip, continue_target);
                }

                // __i = __i + 1
                self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                let one_idx = push_const(&mut self.chunk, Constant::Number(1.0));
                self.chunk.instructions.push(Instruction::Const(one_idx));
                self.chunk.instructions.push(Instruction::Add);
                self.chunk.instructions.push(Instruction::SetLocal(i_slot));

                // Jump back to loop start
                self.chunk.instructions.push(Instruction::Jump(loop_start));

                // Patch the exit jump
                let exit_ip = self.current_ip();
                self.patch_jump(jf_end, exit_ip);

                // Patch all break statements to jump to exit_ip
                let loop_ctx = self.loop_stack.pop().unwrap();
                for break_ip in loop_ctx.break_patches {
                    self.patch_jump(break_ip, exit_ip);
                }

                // Exit the entire loop scope (pop all loop locals)
                self.exit_scope_with_preserve(false);
            }
            Stmt::Break {
                level: level_opt, ..
            } => {
                let level = level_opt.unwrap_or(1) as usize;

                // Validate that we're inside enough nested loops
                if level > self.loop_stack.len() {
                    panic!(
                        "Compiler error: break {} exceeds loop nesting depth of {}",
                        level,
                        self.loop_stack.len()
                    );
                }

                if level == 0 {
                    panic!("Compiler error: break level must be at least 1");
                }

                // Calculate how many locals need to be popped
                // We need to pop all locals from current position back to the target loop's local count
                let target_loop_idx = self.loop_stack.len() - level;
                let target_loop = &self.loop_stack[target_loop_idx];
                let locals_to_pop = self.local_count - target_loop.local_count;

                // Pop the locals that are in scope between here and the target loop
                for _ in 0..locals_to_pop {
                    self.chunk.instructions.push(Instruction::Pop);
                }

                // Emit a jump that will be patched when the target loop exits
                let jump_ip = self.emit_jump();

                // Register this jump in the appropriate loop context (counting from innermost)
                self.loop_stack[target_loop_idx].break_patches.push(jump_ip);
            }
            Stmt::Continue {
                level: level_opt, ..
            } => {
                let level = level_opt.unwrap_or(1) as usize;

                // Validate that we're inside enough nested loops
                if level > self.loop_stack.len() {
                    panic!(
                        "Compiler error: continue {} exceeds loop nesting depth of {}",
                        level,
                        self.loop_stack.len()
                    );
                }

                if level == 0 {
                    panic!("Compiler error: continue level must be at least 1");
                }

                // Calculate how many locals need to be popped
                let target_loop_idx = self.loop_stack.len() - level;
                let target_loop = &self.loop_stack[target_loop_idx];
                let locals_to_pop = self.local_count - target_loop.local_count;

                // Pop the locals that are in scope between here and the target loop
                for _ in 0..locals_to_pop {
                    self.chunk.instructions.push(Instruction::Pop);
                }

                // Check if the loop has a continue_target already set
                // If not set, it means we need to patch it later (for for-loops)
                if let Some(target_ip) = target_loop.continue_target {
                    // Continue target is known, jump directly
                    self.chunk.instructions.push(Instruction::Jump(target_ip));
                } else {
                    // Continue target not set yet - emit placeholder jump
                    // This will be patched when the loop is finished compiling
                    let jump_ip = self.emit_jump();
                    self.loop_stack[target_loop_idx]
                        .continue_patches
                        .push(jump_ip);
                }
            }
            Stmt::ExprStmt { expr, .. } => {
                // Evaluate the expression and pop the result (since it's not used)
                self.emit_expr(expr);
                self.chunk.instructions.push(Instruction::Pop);
            }
        }
    }

    fn emit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Number { value: n, .. } => {
                let idx = push_const(&mut self.chunk, Constant::Number(*n));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::String { value: s, .. } => {
                let idx = push_const(&mut self.chunk, Constant::String(s.clone()));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Boolean { value: b, .. } => {
                let idx = push_const(&mut self.chunk, Constant::Boolean(*b));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Null { .. } => {
                let idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::List {
                elements: items, ..
            } => {
                for item in items {
                    self.emit_expr(item);
                }
                self.chunk
                    .instructions
                    .push(Instruction::BuildList(items.len()));
            }
            Expr::Table { fields, .. } => {
                for (key, value) in fields {
                    // Emit the key based on its type
                    match key {
                        TableKey::Identifier(s) | TableKey::StringLiteral(s) => {
                            // Both identifier and string literal keys become string constants
                            let k_idx = push_const(&mut self.chunk, Constant::String(s.clone()));
                            self.chunk.instructions.push(Instruction::Const(k_idx));
                        }
                        TableKey::Computed(expr) => {
                            // Computed keys: evaluate the expression at runtime
                            self.emit_expr(expr);
                        }
                    }
                    // Emit the value
                    self.emit_expr(value);
                }
                self.chunk
                    .instructions
                    .push(Instruction::BuildTable(fields.len()));
            }
            Expr::MemberAccess { object, member, .. } => {
                self.emit_expr(object);
                let name_idx = push_const(&mut self.chunk, Constant::String(member.clone()));
                self.chunk.instructions.push(Instruction::GetProp(name_idx));
            }
            Expr::Index { object, index, .. } => {
                self.emit_expr(object);
                self.emit_expr(index);
                self.chunk.instructions.push(Instruction::GetIndex);
            }
            Expr::Identifier { name, .. } => {
                if let Some(slot) = self.lookup_local(name) {
                    self.chunk.instructions.push(Instruction::GetLocal(slot));
                } else if let Some(upvalue_idx) = self.resolve_upvalue(name) {
                    self.chunk
                        .instructions
                        .push(Instruction::GetUpvalue(upvalue_idx));
                } else {
                    let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                    self.chunk
                        .instructions
                        .push(Instruction::GetGlobal(name_idx));
                }
            }
            Expr::Unary { op, operand, .. } => match op {
                UnaryOp::Neg => {
                    self.emit_expr(operand);
                    self.chunk.instructions.push(Instruction::Neg);
                }
                UnaryOp::Not => {
                    self.emit_expr(operand);
                    self.chunk.instructions.push(Instruction::Not);
                }
            },
            Expr::Logical {
                left, op, right, ..
            } => {
                match op {
                    LogicalOp::And => {
                        // left && right with short-circuit
                        self.emit_expr(left);
                        self.chunk.instructions.push(Instruction::Dup);
                        let jf = self.emit_jump_if_false();
                        // left truthy: discard left and eval right
                        self.chunk.instructions.push(Instruction::Pop);
                        self.emit_expr(right);
                        let end = self.current_ip();
                        self.patch_jump(jf, end);
                    }
                    LogicalOp::Or => {
                        // left || right with short-circuit using only JUMP_IF_FALSE + JUMP
                        self.emit_expr(left);
                        self.chunk.instructions.push(Instruction::Dup);
                        let jf = self.emit_jump_if_false();
                        let jend = self.emit_jump();
                        // Evaluate right when left is falsey
                        let after_jf = self.current_ip();
                        self.patch_jump(jf, after_jf);
                        self.chunk.instructions.push(Instruction::Pop);
                        self.emit_expr(right);
                        let end = self.current_ip();
                        self.patch_jump(jend, end);
                    }
                }
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                self.emit_expr(left);
                self.emit_expr(right);
                match op {
                    BinaryOp::Add => self.chunk.instructions.push(Instruction::Add),
                    BinaryOp::Sub => self.chunk.instructions.push(Instruction::Sub),
                    BinaryOp::Mul => self.chunk.instructions.push(Instruction::Mul),
                    BinaryOp::Div => self.chunk.instructions.push(Instruction::Div),
                    BinaryOp::Mod => self.chunk.instructions.push(Instruction::Mod),
                    BinaryOp::Eq => self.chunk.instructions.push(Instruction::Eq),
                    BinaryOp::Ne => self.chunk.instructions.push(Instruction::Ne),
                    BinaryOp::Lt => self.chunk.instructions.push(Instruction::Lt),
                    BinaryOp::Le => self.chunk.instructions.push(Instruction::Le),
                    BinaryOp::Gt => self.chunk.instructions.push(Instruction::Gt),
                    BinaryOp::Ge => self.chunk.instructions.push(Instruction::Ge),
                }
            }
            Expr::Function {
                arguments, body, ..
            } => {
                // Compile function as closure (may capture upvalues from enclosing scope)
                let (fn_chunk, upvalue_descriptors) = self.compile_nested_function(arguments, body);
                let idx = push_const(&mut self.chunk, Constant::Function(fn_chunk));

                // If the function captures upvalues, emit Closure instruction
                // Otherwise, use MakeFunction for simpler non-capturing functions
                if upvalue_descriptors.is_empty() {
                    self.chunk.instructions.push(Instruction::MakeFunction(idx));
                } else {
                    self.chunk.instructions.push(Instruction::Closure(idx));
                }
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                // Determine if we need named-arg reordering
                let has_named = arguments
                    .iter()
                    .any(|a| matches!(a, CallArgument::Named { .. }));
                if !has_named {
                    // Simple fast path
                    self.emit_expr(callee);
                    for arg in arguments {
                        match arg {
                            CallArgument::Positional(expr) => self.emit_expr(expr),
                            CallArgument::Named { value, .. } => self.emit_expr(value),
                        }
                    }
                    self.chunk
                        .instructions
                        .push(Instruction::Call(arguments.len()));
                } else {
                    // Enforce: positional cannot follow named
                    let mut seen_named = false;
                    for arg in arguments {
                        match arg {
                            CallArgument::Named { .. } => seen_named = true,
                            CallArgument::Positional(_) if seen_named => {
                                panic!(
                                    "Compiler error: Positional arguments cannot follow named arguments"
                                );
                            }
                            _ => {}
                        }
                    }

                    // Resolve parameter names from callee
                    let param_names: Vec<String> = match &**callee {
                        Expr::Function { arguments: fn_args, .. } => fn_args.iter().map(|a| a.name.clone()).collect(),
                        Expr::Identifier { name: n, .. } => self.lookup_fn_params(n).unwrap_or_else(|| {
                            panic!("Compiler error: Named arguments require statically-known callee '{}" , n)
                        }),
                        _ => panic!("Compiler error: Named arguments require statically-known callee"),
                    };

                    // Split provided args into positional and named map
                    let mut positional: Vec<&Expr> = Vec::new();
                    let mut named_map: HashMap<&str, &Expr> = HashMap::new();
                    for arg in arguments {
                        match arg {
                            CallArgument::Positional(expr) => positional.push(expr),
                            CallArgument::Named { name, value } => {
                                if named_map.contains_key(name.as_str()) {
                                    panic!("Compiler error: Duplicate named argument '{}'", name);
                                }
                                named_map.insert(name.as_str(), value);
                            }
                        }
                    }

                    if param_names.len() < positional.len() {
                        panic!(
                            "Compiler error: Too many positional arguments: expected {} got {}",
                            param_names.len(),
                            positional.len()
                        );
                    }

                    // Build final ordered arg list
                    let mut final_args: Vec<&Expr> = Vec::with_capacity(param_names.len());
                    for (i, pname) in param_names.iter().enumerate() {
                        if i < positional.len() {
                            final_args.push(positional[i]);
                        } else if let Some(v) = named_map.get::<str>(pname.as_str()) {
                            final_args.push(*v);
                        } else {
                            panic!("Compiler error: Missing required argument '{}'", pname);
                        }
                    }

                    // Emit callee and ordered args
                    self.emit_expr(callee);
                    for e in final_args {
                        self.emit_expr(e);
                    }
                    self.chunk
                        .instructions
                        .push(Instruction::Call(param_names.len()));
                }
            }
            Expr::Block {
                statements: stmts, ..
            } => {
                // Block is an expression that evaluates to its last statement's value
                self.enter_scope();
                // Predeclare local function names for mutual recursion in block
                self.predeclare_function_locals(stmts);
                for stmt in stmts {
                    self.emit_stmt(stmt);
                }
                self.exit_scope_with_preserve(true);
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                // Emit condition
                self.emit_expr(condition);
                let jf = self.emit_jump_if_false();

                // Then block
                self.enter_scope();
                // Predeclare local function names for mutual recursion in then-block
                self.predeclare_function_locals(then_block);
                for stmt in then_block {
                    self.emit_stmt(stmt);
                }
                self.exit_scope_with_preserve(true);

                if let Some(else_stmts) = else_block {
                    let jend = self.emit_jump();
                    let else_start = self.current_ip();
                    self.patch_jump(jf, else_start);

                    // Else block
                    self.enter_scope();
                    // Predeclare local function names for mutual recursion in else-block
                    self.predeclare_function_locals(else_stmts);
                    for stmt in else_stmts {
                        self.emit_stmt(stmt);
                    }
                    self.exit_scope_with_preserve(true);

                    let end = self.current_ip();
                    self.patch_jump(jend, end);
                } else {
                    // No else block: if condition false, push null
                    let jend = self.emit_jump();
                    let else_start = self.current_ip();
                    self.patch_jump(jf, else_start);
                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                    let end = self.current_ip();
                    self.patch_jump(jend, end);
                }
            }

            Expr::Import { path, .. } => {
                // Emit the path expression (should be a string)
                self.emit_expr(path);
                // Emit the Import instruction
                self.chunk.instructions.push(Instruction::Import);
            }
            Expr::Match { expr, arms, .. } => {
                // Expression form of match - produces a single value directly
                self.enter_scope();
                self.emit_expr(expr);
                let match_val_slot = self.local_count;
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("__match_val".to_string(), match_val_slot);
                self.local_count += 1;

                let mut end_jumps = Vec::new();
                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let is_wildcard = matches!(pattern, Pattern::Wildcard { .. });
                    // Check for tag patterns
                    let is_tag_pattern = matches!(pattern, Pattern::Ident { name, .. } if matches!(name.as_str(), "ok" | "err" | "some" | "none"));
                    let is_catch_all = is_wildcard
                        || (matches!(pattern, Pattern::Ident { name: _, .. }) && !is_tag_pattern);

                    if !is_catch_all {
                        match pattern {
                            Pattern::Ident { name, .. } => {
                                // Tag pattern matching
                                self.chunk
                                    .instructions
                                    .push(Instruction::GetLocal(match_val_slot));
                                let name_idx =
                                    push_const(&mut self.chunk, Constant::String(name.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(name_idx));
                                let null_idx = push_const(&mut self.chunk, Constant::Null);
                                self.chunk.instructions.push(Instruction::Const(null_idx));
                                self.chunk.instructions.push(Instruction::Ne);
                                let jf_next = self.emit_jump_if_false();

                                // Execute arm and produce value
                                let arm_body = apply_implicit_return_to_arm(body);
                                for st in &arm_body {
                                    self.emit_stmt(st);
                                }
                                if !does_block_leave_value(&arm_body) {
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                }
                                end_jumps.push(self.emit_jump());
                                let next_ip = self.current_ip();
                                self.patch_jump(jf_next, next_ip);
                            }
                            Pattern::Literal { value: lit, .. } => {
                                // Literal pattern matching
                                self.chunk
                                    .instructions
                                    .push(Instruction::GetLocal(match_val_slot));
                                match lit {
                                    crate::ast::Literal::Number(n) => {
                                        let idx = push_const(&mut self.chunk, Constant::Number(*n));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::String(s) => {
                                        let idx = push_const(
                                            &mut self.chunk,
                                            Constant::String(s.clone()),
                                        );
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::Boolean(b) => {
                                        let idx =
                                            push_const(&mut self.chunk, Constant::Boolean(*b));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                    crate::ast::Literal::Null => {
                                        let idx = push_const(&mut self.chunk, Constant::Null);
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                    }
                                }
                                self.chunk.instructions.push(Instruction::Eq);
                                let jf_next = self.emit_jump_if_false();

                                // Execute arm and produce value
                                let arm_body = apply_implicit_return_to_arm(body);
                                for st in &arm_body {
                                    self.emit_stmt(st);
                                }
                                if !does_block_leave_value(&arm_body) {
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                }
                                end_jumps.push(self.emit_jump());
                                let next_ip = self.current_ip();
                                self.patch_jump(jf_next, next_ip);
                            }
                            _ => panic!(
                                "Compiler error: structural patterns for match expr not yet supported"
                            ),
                        }
                    } else {
                        // Catch-all pattern
                        let arm_body = apply_implicit_return_to_arm(body);
                        for st in &arm_body {
                            self.emit_stmt(st);
                        }
                        if !does_block_leave_value(&arm_body) {
                            let null_idx = push_const(&mut self.chunk, Constant::Null);
                            self.chunk.instructions.push(Instruction::Const(null_idx));
                        }
                        if i < arms.len() - 1 {
                            end_jumps.push(self.emit_jump());
                        }
                    }
                }
                let end_ip = self.current_ip();
                for j in end_jumps {
                    self.patch_jump(j, end_ip);
                }
                self.exit_scope_with_preserve(true);
            } // Other expressions not yet supported
        }
    }

    fn emit_jump_if_false(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk
            .instructions
            .push(Instruction::JumpIfFalse(usize::MAX));
        pos
    }
    fn emit_jump(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::Jump(usize::MAX));
        pos
    }
    fn patch_jump(&mut self, at: usize, target: usize) {
        match self.chunk.instructions.get_mut(at) {
            Some(Instruction::JumpIfFalse(addr)) => *addr = target,
            Some(Instruction::Jump(addr)) => *addr = target,
            _ => {}
        }
    }
    fn current_ip(&self) -> usize {
        self.chunk.instructions.len()
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.param_scopes.push(HashMap::new());
    }

    fn exit_scope_with_preserve(&mut self, preserve_top: bool) {
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

    fn lookup_local(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    /// Lookup parameter names for a function variable by identifier name.
    /// Searches current and outer param scopes, then global map, then parent's chain.
    fn lookup_fn_params(&self, name: &str) -> Option<Vec<String>> {
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
    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
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
    fn compile_nested_function(
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
    fn predeclare_function_locals(&mut self, stmts: &[Stmt]) {
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
fn apply_implicit_return_to_arm(body: &[Stmt]) -> Vec<Stmt> {
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

fn does_block_leave_value(block: &[Stmt]) -> bool {
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
