use crate::ast::{Program, Stmt, Expr, BinaryOp, UnaryOp, LogicalOp, Argument, Pattern, CallArgument};
use super::ir::{Chunk, Instruction, Constant};
use std::collections::HashMap;

pub fn compile_program(program: &Program) -> Chunk {
    let mut c = Compiler::new("<program>");
    
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

fn compile_function(arguments: &[Argument], body: &[Stmt]) -> Chunk {
    let mut c = Compiler::new("<function>");
    let arity = arguments.len();
    // Enter scope for function parameters
    c.enter_scope();
    // Parameters become locals in order
    for arg in arguments {
        let slot = c.local_count;
        c.scopes.last_mut().unwrap().insert(arg.name.clone(), slot);
        c.local_count += 1;
    }
    // Compile body
    for stmt in body {
        c.emit_stmt(stmt);
    }
    // Exit scope (pop parameters if needed, but they're handled by caller)
    c.exit_scope_with_preserve(block_leaves_value(body));
    c.chunk.instructions.push(Instruction::Return);
    c.chunk.local_count = arity as u16; // Store arity in the chunk
    c.chunk
}

fn push_const(chunk: &mut Chunk, c: Constant) -> usize {
    chunk.constants.push(c);
    chunk.constants.len() - 1
}

struct LoopContext {
    break_patches: Vec<usize>, // IPs of break jumps to patch when loop exits
    continue_patches: Vec<usize>, // IPs of continue jumps to patch (for for-loops)
    local_count: usize,    // Number of locals when this loop started
    continue_target: Option<usize>, // Explicit continue target (set after body for for-loops)
}

struct Compiler {
    chunk: Chunk,
    scopes: Vec<HashMap<String, usize>>, // name -> slot index
    local_count: usize,
    loop_stack: Vec<LoopContext>, // Track nested loops for break/continue
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self { 
            chunk: Chunk { name: name.to_string(), ..Default::default() }, 
            scopes: Vec::new(), 
            local_count: 0,
            loop_stack: Vec::new(),
        }
    }

    fn emit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Return(expr) => {
                self.emit_expr(expr);
            }
            Stmt::If { condition, then_block, elif_blocks, else_block } => {
                // if cond then ... elif ... else ... end
                // Always leave exactly one value on stack as the if-expression result.
                // Start with a default Null (used when no branch produces a value).
                let null_idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx));
                self.emit_expr(condition);
                let jf_main = self.emit_jump_if_false();

                // then block
                self.enter_scope();
                // Remove default Null and compute real value for this branch
                self.chunk.instructions.push(Instruction::Pop);
                for st in then_block { self.emit_stmt(st); }
                let then_preserve = block_leaves_value(then_block);
                self.exit_scope_with_preserve(then_preserve);
                if !then_preserve {
                    // branch produced no value; push Null to keep if result arity
                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                }
                let j_end = self.emit_jump();

                // else/elif chain
                let mut next_start = self.current_ip();
                self.patch_jump(jf_main, next_start);

                // elif blocks
                for (elif_cond, elif_body) in elif_blocks {
                    self.emit_expr(elif_cond);
                    let jf_elif = self.emit_jump_if_false();
                    self.enter_scope();
                    // Remove default Null and compute real value for this branch
                    self.chunk.instructions.push(Instruction::Pop);
                    for st in elif_body { self.emit_stmt(st); }
                    let elif_preserve = block_leaves_value(elif_body);
                    self.exit_scope_with_preserve(elif_preserve);
                    if !elif_preserve {
                        let null_idx = push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                    }
                    let _j_after_elif = self.emit_jump();
                    // patch jf_elif to the next block start
                    next_start = self.current_ip();
                    self.patch_jump(jf_elif, next_start);
                    // leave j_after_elif as placeholder to be patched to end later
                }

                // else block
                if let Some(else_body) = else_block {
                    self.enter_scope();
                    // Remove default Null and compute real value for this branch
                    self.chunk.instructions.push(Instruction::Pop);
                    for st in else_body { self.emit_stmt(st); }
                    let else_preserve = block_leaves_value(else_body);
                    self.exit_scope_with_preserve(else_preserve);
                    if !else_preserve {
                        let null_idx = push_const(&mut self.chunk, Constant::Null);
                        self.chunk.instructions.push(Instruction::Const(null_idx));
                    }
                }

                // Patch final end for main then and all elif end jumps
                let end_ip = self.current_ip();
                self.patch_jump(j_end, end_ip);
                // Also patch any Jump(usize::MAX) left from elif bodies to end_ip
                // Walk back and patch immediate previous Jump placeholders
                for instr in &mut self.chunk.instructions {
                    if let Instruction::Jump(addr) = instr && *addr == usize::MAX { *addr = end_ip; }
                }
            }
            Stmt::While { condition, body } => {
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
                for st in body { self.emit_stmt(st); }
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
            Stmt::DoWhile { body, condition } => {
                let loop_start = self.current_ip();
                
                // Register this loop for break/continue tracking
                self.loop_stack.push(LoopContext {
                    break_patches: Vec::new(),
                    continue_patches: Vec::new(),
                    local_count: self.local_count,
                    continue_target: Some(loop_start), // For do-while loops, continue jumps to start (body start)
                });
                
                self.enter_scope();
                for st in body { self.emit_stmt(st); }
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
            Stmt::Match { expr, arms } => {
                // For MVP: Simple pattern matching on identifier patterns
                // Match statements can produce a value (last expression in matched arm)
                
                // Evaluate the match expression once and store in a hidden local
                self.enter_scope();
                self.emit_expr(expr);
                let match_val_slot = self.local_count;
                self.scopes.last_mut().unwrap().insert("__match_val".to_string(), match_val_slot);
                self.local_count += 1;
                
                // Push default null result (in case no arms match) 
                let null_idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx));
                
                let mut end_jumps = Vec::new();
                
                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let is_wildcard = matches!(pattern, Pattern::Wildcard);
                    
                    if !is_wildcard {
                        // Check if pattern matches (for now, check if property exists)
                        match pattern {
                            Pattern::Ident(name) => {
                                // Check if the match value has this property
                                self.chunk.instructions.push(Instruction::GetLocal(match_val_slot));
                                let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(name_idx));
                                // If property exists (not null), this arm matches
                                let null_idx = push_const(&mut self.chunk, Constant::Null);
                                self.chunk.instructions.push(Instruction::Const(null_idx));
                                self.chunk.instructions.push(Instruction::Ne); // property != null
                                
                                let jf_next_arm = self.emit_jump_if_false();
                                
                                // Remove default null before executing arm
                                self.chunk.instructions.push(Instruction::Pop);
                                
                                // Execute this arm's body (no new scope needed, pattern doesn't bind)
                                // Handle implicit return: convert trailing ExprStmt to Return
                                let mut arm_body = body.to_vec();
                                if let Some(last) = arm_body.pop() {
                                    match last {
                                        Stmt::ExprStmt(e) => arm_body.push(Stmt::Return(e)),
                                        other => arm_body.push(other),
                                    }
                                }
                                
                                for stmt in &arm_body {
                                    self.emit_stmt(stmt);
                                }
                                
                                let arm_preserves = block_leaves_value(&arm_body);
                                if !arm_preserves {
                                    // Arm didn't produce value, push null
                                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                                    self.chunk.instructions.push(Instruction::Const(null_idx));
                                }
                                
                                // Jump to end after executing this arm
                                end_jumps.push(self.emit_jump());
                                
                                // Patch the jump to next arm
                                let next_arm_ip = self.current_ip();
                                self.patch_jump(jf_next_arm, next_arm_ip);
                            }
                            _ => {
                                // Complex patterns not yet supported
                                panic!("Compiler error: Complex patterns in match statements not yet supported");
                            }
                        }
                    } else {
                        // Wildcard pattern - always matches (default case)
                        // Remove default null before executing
                        self.chunk.instructions.push(Instruction::Pop);
                        
                        // Handle implicit return
                        let mut arm_body = body.to_vec();
                        if let Some(last) = arm_body.pop() {
                            match last {
                                Stmt::ExprStmt(e) => arm_body.push(Stmt::Return(e)),
                                other => arm_body.push(other),
                            }
                        }
                        
                        for stmt in &arm_body {
                            self.emit_stmt(stmt);
                        }
                        
                        let arm_preserves = block_leaves_value(&arm_body);
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
                    self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                } else {
                    // local: initializer value stays on stack as slot
                    self.emit_expr(value);
                    let slot = self.local_count;
                    self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                    self.local_count += 1;
                }
            }
            Stmt::DestructuringVarDecl { mutable: _, pattern, value } => {
                // Emit the value expression once
                self.emit_expr(value);
                
                // For now, the value is on the stack. We need to destructure it.
                if self.scopes.is_empty() {
                    // Global destructuring
                    match pattern {
                        Pattern::ArrayPattern { elements, rest } => {
                            // Array destructuring: [a, b, ...rest] = array
                            // Strategy: Get each element by index and assign to globals
                            
                            // Dup the array for each element access
                            for (i, elem_pattern) in elements.iter().enumerate() {
                                match elem_pattern {
                                    Pattern::Ident(name) => {
                                        self.chunk.instructions.push(Instruction::Dup); // dup array
                                        let idx = push_const(&mut self.chunk, Constant::Number(i as f64));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                        self.chunk.instructions.push(Instruction::GetIndex);
                                        let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                                        self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                                    }
                                    Pattern::Wildcard => {
                                        // Wildcard element: don't extract, don't bind
                                    }
                                    _ => {
                                        panic!("Nested destructuring patterns not yet supported");
                                    }
                                }
                            }
                            
                            // Handle rest pattern (e.g., ...tail)
                            if let Some(rest_name) = rest {
                                // Build array slice from elements.len() onwards
                                let start_index = elements.len();
                                self.chunk.instructions.push(Instruction::Dup); // dup array
                                self.chunk.instructions.push(Instruction::SliceArray(start_index));
                                let name_idx = push_const(&mut self.chunk, Constant::String(rest_name.clone()));
                                self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                            } else {
                                // No rest, pop the array
                                self.chunk.instructions.push(Instruction::Pop);
                            }
                        }
                        Pattern::TablePattern(keys) => {
                            // Table destructuring: {name, age} = table
                            for key in keys {
                                self.chunk.instructions.push(Instruction::Dup); // dup table
                                let key_idx = push_const(&mut self.chunk, Constant::String(key.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(key_idx));
                                let name_idx = push_const(&mut self.chunk, Constant::String(key.clone()));
                                self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                            }
                            self.chunk.instructions.push(Instruction::Pop); // pop table
                        }
                        Pattern::Ident(name) => {
                            // Simple binding, just assign
                            let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                            self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                        }
                        Pattern::Wildcard => {
                            // Wildcard doesn't bind, just pop the value
                            self.chunk.instructions.push(Instruction::Pop);
                        }
                    }
                } else {
                    // Local destructuring
                    match pattern {
                        Pattern::ArrayPattern { elements, rest } => {
                            // For locals, the value is already on stack and will become local slots
                            let value_slot = self.local_count;
                            self.scopes.last_mut().unwrap().insert("__destructure_val".to_string(), value_slot);
                            self.local_count += 1;
                            
                            // Extract each element into its own local
                            for (i, elem_pattern) in elements.iter().enumerate() {
                                match elem_pattern {
                                    Pattern::Ident(name) => {
                                        self.chunk.instructions.push(Instruction::GetLocal(value_slot));
                                        let idx = push_const(&mut self.chunk, Constant::Number(i as f64));
                                        self.chunk.instructions.push(Instruction::Const(idx));
                                        self.chunk.instructions.push(Instruction::GetIndex);
                                        
                                        let elem_slot = self.local_count;
                                        self.scopes.last_mut().unwrap().insert(name.clone(), elem_slot);
                                        self.local_count += 1;
                                    }
                                    Pattern::Wildcard => {
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
                                self.chunk.instructions.push(Instruction::GetLocal(value_slot));
                                self.chunk.instructions.push(Instruction::SliceArray(start_index));
                                let rest_slot = self.local_count;
                                self.scopes.last_mut().unwrap().insert(rest_name.clone(), rest_slot);
                                self.local_count += 1;
                            }
                        }
                        Pattern::TablePattern(keys) => {
                            let value_slot = self.local_count;
                            self.scopes.last_mut().unwrap().insert("__destructure_val".to_string(), value_slot);
                            self.local_count += 1;
                            
                            for key in keys {
                                self.chunk.instructions.push(Instruction::GetLocal(value_slot));
                                let key_idx = push_const(&mut self.chunk, Constant::String(key.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(key_idx));
                                
                                let key_slot = self.local_count;
                                self.scopes.last_mut().unwrap().insert(key.clone(), key_slot);
                                self.local_count += 1;
                            }
                        }
                        Pattern::Ident(name) => {
                            let slot = self.local_count;
                            self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                            self.local_count += 1;
                        }
                        Pattern::Wildcard => {
                            // Wildcard doesn't bind, the value stays on stack as an unused local
                            // We still need to account for it in local_count
                            self.local_count += 1;
                        }
                    }
                }
            }
            Stmt::Assignment { target, op: _, value } => {
                match target {
                    Expr::Identifier(name) => {
                        self.emit_expr(value);
                        if let Some(slot) = self.lookup_local(name) {
                            self.chunk.instructions.push(Instruction::SetLocal(slot));
                        } else {
                            let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                            self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                        }
                    }
                    Expr::MemberAccess { object, member } => {
                        // Stack layout: object, value
                        self.emit_expr(object);
                        self.emit_expr(value);
                        let name_idx = push_const(&mut self.chunk, Constant::String(member.clone()));
                        self.chunk.instructions.push(Instruction::SetProp(name_idx));
                    }
                    Expr::Index { object, index } => {
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
            Stmt::For { pattern, iterator, body } => {
                // Lower: for pattern in iterator do body end
                // To:
                //   let __iter = iterator
                //   let __i = 0
                //   let loop_var = null
                //   while __i < len(__iter) do
                //     loop_var = __iter[__i]
                //     body
                //     __i = __i + 1
                //   end
                
                // For MVP, only support simple identifier patterns
                let Pattern::Ident(var_name) = pattern else {
                    // Complex patterns not supported in for loops yet
                    panic!("Compiler error: Complex patterns in for loops are not yet supported. Use a simple identifier instead.");
                };
                
                // Create a new scope for the entire loop (including iterator and index)
                self.enter_scope();
                
                // Evaluate iterator and store in a hidden local __iter
                self.emit_expr(iterator);
                let iter_slot = self.local_count;
                self.scopes.last_mut().unwrap().insert("__iter".to_string(), iter_slot);
                self.local_count += 1;
                
                // Initialize __i = 0
                let zero_idx = push_const(&mut self.chunk, Constant::Number(0.0));
                self.chunk.instructions.push(Instruction::Const(zero_idx));
                let i_slot = self.local_count;
                self.scopes.last_mut().unwrap().insert("__i".to_string(), i_slot);
                self.local_count += 1;
                
                // Allocate slot for loop variable (initialized to null)
                let null_idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx));
                let loop_var_slot = self.local_count;
                self.scopes.last_mut().unwrap().insert(var_name.clone(), loop_var_slot);
                self.local_count += 1;
                
                // while __i < len(__iter) do
                let loop_start = self.current_ip();
                
                // Register this loop for break/continue tracking
                // Note: continue_target will be set later and continue_patches will be patched
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
                self.chunk.instructions.push(Instruction::GetLocal(iter_slot));
                self.chunk.instructions.push(Instruction::GetLen);
                
                // Compare __i < len(__iter)
                self.chunk.instructions.push(Instruction::Lt);
                
                // If false, jump to end
                let jf_end = self.emit_jump_if_false();
                
                // Update loop variable: loop_var = __iter[__i]
                self.chunk.instructions.push(Instruction::GetLocal(iter_slot));
                self.chunk.instructions.push(Instruction::GetLocal(i_slot));
                self.chunk.instructions.push(Instruction::GetIndex);
                self.chunk.instructions.push(Instruction::SetLocal(loop_var_slot));
                
                // Execute body
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
                
                // Exit the entire loop scope (pop __iter, __i, and loop_var)
                self.exit_scope_with_preserve(false);
                
                // For loops don't leave a value on the stack
                let null_idx2 = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx2));
            }
            Stmt::Break(level_opt) => {
                let level = level_opt.unwrap_or(1) as usize;
                
                // Validate that we're inside enough nested loops
                if level > self.loop_stack.len() {
                    panic!("Compiler error: break {} exceeds loop nesting depth of {}", 
                           level, self.loop_stack.len());
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
            Stmt::Continue(level_opt) => {
                let level = level_opt.unwrap_or(1) as usize;
                
                // Validate that we're inside enough nested loops
                if level > self.loop_stack.len() {
                    panic!("Compiler error: continue {} exceeds loop nesting depth of {}", 
                           level, self.loop_stack.len());
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
                    self.loop_stack[target_loop_idx].continue_patches.push(jump_ip);
                }
            }
            _ => {
                // For now, ignore non-return statements
            }
        }
    }

    fn emit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Number(n) => {
                let idx = push_const(&mut self.chunk, Constant::Number(*n));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::String(s) => {
                let idx = push_const(&mut self.chunk, Constant::String(s.clone()));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Boolean(b) => {
                let idx = push_const(&mut self.chunk, Constant::Boolean(*b));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Null => {
                let idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Array(items) => {
                for item in items { self.emit_expr(item); }
                self.chunk.instructions.push(Instruction::BuildArray(items.len()));
            }
            Expr::Table(fields) => {
                for (k, v) in fields {
                    let k_idx = push_const(&mut self.chunk, Constant::String(k.clone()));
                    self.chunk.instructions.push(Instruction::Const(k_idx));
                    self.emit_expr(v);
                }
                self.chunk.instructions.push(Instruction::BuildTable(fields.len()));
            }
            Expr::MemberAccess { object, member } => {
                self.emit_expr(object);
                let name_idx = push_const(&mut self.chunk, Constant::String(member.clone()));
                self.chunk.instructions.push(Instruction::GetProp(name_idx));
            }
            Expr::Index { object, index } => {
                self.emit_expr(object);
                self.emit_expr(index);
                self.chunk.instructions.push(Instruction::GetIndex);
            }
            Expr::Identifier(name) => {
                if let Some(slot) = self.lookup_local(name) {
                    self.chunk.instructions.push(Instruction::GetLocal(slot));
                } else {
                    let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                    self.chunk.instructions.push(Instruction::GetGlobal(name_idx));
                }
            }
            Expr::Unary { op, operand } => {
                match op {
                    UnaryOp::Neg => {
                        self.emit_expr(operand);
                        self.chunk.instructions.push(Instruction::Neg);
                    }
                    UnaryOp::Not => {
                        self.emit_expr(operand);
                        self.chunk.instructions.push(Instruction::Not);
                    }
                }
            }
            Expr::Logical { left, op, right } => {
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
            Expr::Binary { left, op, right } => {
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
            Expr::Function { arguments, body, .. } => {
                // Compile function body into a new chunk
                let fn_chunk = compile_function(arguments, body);
                let idx = push_const(&mut self.chunk, Constant::Function(fn_chunk));
                self.chunk.instructions.push(Instruction::MakeFunction(idx));
            }
            Expr::Call { callee, arguments } => {
                // Push callee
                self.emit_expr(callee);
                // Push arguments (extract expressions from CallArgument enum)
                for arg in arguments {
                    match arg {
                        CallArgument::Positional(expr) => self.emit_expr(expr),
                        CallArgument::Named { value, .. } => self.emit_expr(value),
                    }
                }
                self.chunk.instructions.push(Instruction::Call(arguments.len()));
            }
            Expr::Block(stmts) => {
                // Block is an expression that evaluates to its last statement's value
                self.enter_scope();
                for stmt in stmts {
                    self.emit_stmt(stmt);
                }
                self.exit_scope_with_preserve(true);
            }
            Expr::If { condition, then_block, else_block } => {
                // Emit condition
                self.emit_expr(condition);
                let jf = self.emit_jump_if_false();
                
                // Then block
                self.enter_scope();
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
            // Other expressions not yet supported
        }
    }

    fn emit_jump_if_false(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::JumpIfFalse(usize::MAX));
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
    fn current_ip(&self) -> usize { self.chunk.instructions.len() }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope_with_preserve(&mut self, preserve_top: bool) {
        if let Some(scope) = self.scopes.pop() {
            let to_pop = scope.len();
            if to_pop > 0 {
                if preserve_top {
                    self.chunk.instructions.push(Instruction::PopNPreserve(to_pop));
                } else {
                    for _ in 0..to_pop { self.chunk.instructions.push(Instruction::Pop); }
                }
            }
            self.local_count = self.local_count.saturating_sub(to_pop);
        }
    }

    fn lookup_local(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) { return Some(slot); }
        }
        None
    }
}

fn block_leaves_value(block: &[Stmt]) -> bool {
    match block.last() {
        Some(Stmt::Return(_)) => true,
        Some(Stmt::If { then_block, elif_blocks, else_block, .. }) => {
            // An if-statement leaves a value if all branches leave a value
            let then_leaves = block_leaves_value(then_block);
            let elif_leave = elif_blocks.iter().all(|(_, b)| block_leaves_value(b));
            let else_leaves = else_block.as_ref().is_some_and(|b| block_leaves_value(b));
            then_leaves && elif_leave && else_leaves
        }
        _ => false,
    }
}
