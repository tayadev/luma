use crate::ast::{Program, Stmt, Expr, BinaryOp, UnaryOp, LogicalOp, Argument, Pattern, CallArgument};
use super::ir::{Chunk, Instruction, Constant};
use std::collections::HashMap;

pub fn compile_program(program: &Program) -> Chunk {
    let mut c = Compiler::new("<program>");
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

struct Compiler {
    chunk: Chunk,
    scopes: Vec<HashMap<String, usize>>, // name -> slot index
    local_count: usize,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self { chunk: Chunk { name: name.to_string(), ..Default::default() }, scopes: Vec::new(), local_count: 0 }
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
                    if let Instruction::Jump(addr) = instr { if *addr == usize::MAX { *addr = end_ip; } }
                }
            }
            Stmt::While { condition, body } => {
                let loop_start = self.current_ip();
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
            }
            Stmt::DoWhile { body, condition } => {
                let loop_start = self.current_ip();
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
                    return;
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
                
                // Exit the entire loop scope (pop __iter, __i, and loop_var)
                self.exit_scope_with_preserve(false);
                
                // For loops don't leave a value on the stack
                let null_idx2 = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(null_idx2));
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
                        // 0 - operand
                        let zero = push_const(&mut self.chunk, Constant::Number(0.0));
                        self.chunk.instructions.push(Instruction::Const(zero));
                        self.emit_expr(operand);
                        self.chunk.instructions.push(Instruction::Sub);
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
            // Other expressions not yet supported
            _ => {
                let idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(idx));
            }
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
            let else_leaves = else_block.as_ref().map_or(false, |b| block_leaves_value(b));
            then_leaves && elif_leave && else_leaves
        }
        _ => false,
    }
}
