use super::compile::{Compiler, does_block_leave_value};
use super::helpers::{
    GLOBAL_ITER_FN, HIDDEN_DESTRUCTURE_VAL, HIDDEN_I, HIDDEN_ITER, HIDDEN_MATCH_VAL,
};
use super::ir::{Constant, Instruction};
use crate::ast::{Expr, Stmt};

pub(super) fn emit_stmt(c: &mut Compiler, s: &Stmt) {
    match s {
        Stmt::Return { value, .. } => {
            c.emit_expr(value);
        }
        Stmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            c.push_null();
            c.emit_expr(condition);
            let jf_main = c.emit_jump_if_false();

            c.enter_scope();
            c.predeclare_function_locals(then_block);
            c.chunk.instructions.push(Instruction::Pop);
            for st in then_block {
                c.emit_stmt(st);
            }
            let then_preserve = does_block_leave_value(then_block);
            c.exit_scope_with_preserve(then_preserve);
            if !then_preserve {
                c.push_null();
            }
            let mut end_jumps: Vec<usize> = Vec::new();
            end_jumps.push(c.emit_jump());

            let mut next_start = c.current_ip();
            c.patch_jump(jf_main, next_start);

            for (elif_cond, elif_body) in elif_blocks {
                c.emit_expr(elif_cond);
                let jf_elif = c.emit_jump_if_false();
                c.enter_scope();
                c.predeclare_function_locals(elif_body);
                c.chunk.instructions.push(Instruction::Pop);
                for st in elif_body {
                    c.emit_stmt(st);
                }
                let elif_preserve = does_block_leave_value(elif_body);
                c.exit_scope_with_preserve(elif_preserve);
                if !elif_preserve {
                    c.push_null();
                }
                end_jumps.push(c.emit_jump());
                next_start = c.current_ip();
                c.patch_jump(jf_elif, next_start);
            }

            if let Some(else_body) = else_block {
                c.enter_scope();
                c.predeclare_function_locals(else_body);
                c.chunk.instructions.push(Instruction::Pop);
                for st in else_body {
                    c.emit_stmt(st);
                }
                let else_preserve = does_block_leave_value(else_body);
                c.exit_scope_with_preserve(else_preserve);
                if !else_preserve {
                    c.push_null();
                }
            }

            let end_ip = c.current_ip();
            for j in end_jumps {
                c.patch_jump(j, end_ip);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            let loop_start = c.current_ip();
            c.loop_stack.push(super::compile::LoopContext {
                break_patches: Vec::new(),
                continue_patches: Vec::new(),
                local_count: c.local_count,
                continue_target: Some(loop_start),
            });
            c.emit_expr(condition);
            let jf_end = c.emit_jump_if_false();
            c.enter_scope();
            c.predeclare_function_locals(body);
            for st in body {
                c.emit_stmt(st);
            }
            c.exit_scope_with_preserve(false);
            c.chunk.instructions.push(Instruction::Jump(loop_start));
            let end_ip = c.current_ip();
            c.patch_jump(jf_end, end_ip);
            let loop_ctx = c.loop_stack.pop().unwrap();
            for break_ip in loop_ctx.break_patches {
                c.patch_jump(break_ip, end_ip);
            }
        }
        Stmt::DoWhile {
            body, condition, ..
        } => {
            let loop_start = c.current_ip();
            c.loop_stack.push(super::compile::LoopContext {
                break_patches: Vec::new(),
                continue_patches: Vec::new(),
                local_count: c.local_count,
                continue_target: Some(loop_start),
            });
            c.enter_scope();
            c.predeclare_function_locals(body);
            for st in body {
                c.emit_stmt(st);
            }
            c.exit_scope_with_preserve(false);
            c.emit_expr(condition);
            let jf_end = c.emit_jump_if_false();
            c.chunk.instructions.push(Instruction::Jump(loop_start));
            let end_ip = c.current_ip();
            c.patch_jump(jf_end, end_ip);
            let loop_ctx = c.loop_stack.pop().unwrap();
            for break_ip in loop_ctx.break_patches {
                c.patch_jump(break_ip, end_ip);
            }
        }
        Stmt::Match { expr, arms, .. } => {
            c.enter_scope();
            c.emit_expr(expr);
            let match_val_slot = c.local_count;
            c.bind_hidden_local(HIDDEN_MATCH_VAL.to_string(), match_val_slot);
            c.local_count += 1;
            let mut end_jumps = Vec::new();
            for (i, (pattern, body)) in arms.iter().enumerate() {
                if let Some(j) =
                    c.emit_match_arm(match_val_slot, pattern, body, i == arms.len() - 1)
                {
                    end_jumps.push(j);
                }
            }
            let end_ip = c.current_ip();
            for jump_pos in end_jumps {
                c.patch_jump(jump_pos, end_ip);
            }
            c.exit_scope_with_preserve(true);
        }
        Stmt::VarDecl { name, value, .. } => {
            if c.scopes.is_empty() {
                c.emit_expr(value);
                let name_idx =
                    super::compile::push_const(&mut c.chunk, Constant::String(name.clone()));
                c.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                if let Expr::Function { arguments, .. } = value {
                    let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                    c.global_fn_params.insert(name.clone(), params);
                }
            } else {
                if let Some(&slot) = c.scopes.last().and_then(|m| m.get(name)) {
                    c.emit_expr(value);
                    c.chunk.instructions.push(Instruction::SetLocal(slot));
                } else {
                    c.emit_expr(value);
                    let slot = c.local_count;
                    c.scopes.last_mut().unwrap().insert(name.clone(), slot);
                    c.local_count += 1;
                }
                if let Expr::Function { arguments, .. } = value {
                    let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                    if let Some(scope) = c.param_scopes.last_mut() {
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
            c.emit_expr(value);
            if c.scopes.is_empty() {
                c.emit_destructure_global(pattern);
            } else {
                let value_slot = c.local_count;
                c.bind_hidden_local(HIDDEN_DESTRUCTURE_VAL.to_string(), value_slot);
                c.local_count += 1;
                c.emit_destructure_local(pattern, value_slot);
            }
        }
        Stmt::Assignment {
            target,
            op: _,
            value,
            ..
        } => match target {
            Expr::Identifier { name, .. } => {
                c.emit_expr(value);
                if let Some(slot) = c.lookup_local(name) {
                    c.chunk.instructions.push(Instruction::SetLocal(slot));
                } else if let Some(upvalue_idx) = c.resolve_upvalue(name) {
                    c.chunk
                        .instructions
                        .push(Instruction::SetUpvalue(upvalue_idx));
                } else {
                    let name_idx =
                        super::compile::push_const(&mut c.chunk, Constant::String(name.clone()));
                    c.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                }
            }
            Expr::MemberAccess { object, member, .. } => {
                c.emit_expr(object);
                c.emit_expr(value);
                let name_idx =
                    super::compile::push_const(&mut c.chunk, Constant::String(member.clone()));
                c.chunk.instructions.push(Instruction::SetProp(name_idx));
            }
            Expr::Index { object, index, .. } => {
                c.emit_expr(object);
                c.emit_expr(index);
                c.emit_expr(value);
                c.chunk.instructions.push(Instruction::SetIndex);
            }
            _ => {}
        },
        Stmt::For {
            pattern,
            iterator,
            body,
            ..
        } => {
            c.enter_scope();
            let iter_name_idx = super::compile::push_const(
                &mut c.chunk,
                Constant::String(GLOBAL_ITER_FN.to_string()),
            );
            c.chunk
                .instructions
                .push(Instruction::GetGlobal(iter_name_idx));
            c.emit_expr(iterator);
            c.chunk.instructions.push(Instruction::Call(1));
            let iter_slot = c.local_count;
            c.bind_hidden_local(HIDDEN_ITER.to_string(), iter_slot);
            c.local_count += 1;
            let zero_idx = super::compile::push_const(&mut c.chunk, Constant::Number(0.0));
            c.chunk.instructions.push(Instruction::Const(zero_idx));
            let i_slot = c.local_count;
            c.bind_hidden_local(HIDDEN_I.to_string(), i_slot);
            c.local_count += 1;

            let loop_pat = c.prepare_loop_pattern(pattern);

            let loop_start = c.current_ip();
            let loop_ctx_idx = c.loop_stack.len();
            c.loop_stack.push(super::compile::LoopContext {
                break_patches: Vec::new(),
                continue_patches: Vec::new(),
                local_count: c.local_count,
                continue_target: None,
            });
            c.chunk.instructions.push(Instruction::GetLocal(i_slot));
            c.chunk.instructions.push(Instruction::GetLocal(iter_slot));
            c.chunk.instructions.push(Instruction::GetLen);
            c.chunk.instructions.push(Instruction::Lt);
            let jf_end = c.emit_jump_if_false();
            c.assign_loop_pattern_value(&loop_pat, iter_slot, i_slot);
            c.predeclare_function_locals(body);
            for stmt in body {
                c.emit_stmt(stmt);
            }
            let continue_target = c.current_ip();
            let loop_ctx = &c.loop_stack[loop_ctx_idx];
            let continue_ips = loop_ctx.continue_patches.clone();
            for continue_ip in continue_ips {
                c.patch_jump(continue_ip, continue_target);
            }
            c.chunk.instructions.push(Instruction::GetLocal(i_slot));
            let one_idx = super::compile::push_const(&mut c.chunk, Constant::Number(1.0));
            c.chunk.instructions.push(Instruction::Const(one_idx));
            c.chunk.instructions.push(Instruction::Add);
            c.chunk.instructions.push(Instruction::SetLocal(i_slot));
            c.chunk.instructions.push(Instruction::Jump(loop_start));
            let exit_ip = c.current_ip();
            c.patch_jump(jf_end, exit_ip);
            let loop_ctx = c.loop_stack.pop().unwrap();
            for break_ip in loop_ctx.break_patches {
                c.patch_jump(break_ip, exit_ip);
            }
            c.exit_scope_with_preserve(false);
        }
        Stmt::Break {
            level: level_opt, ..
        } => {
            let level = level_opt.unwrap_or(1) as usize;
            if level > c.loop_stack.len() {
                c.error(&format!(
                    "break {} exceeds loop nesting depth of {}",
                    level,
                    c.loop_stack.len()
                ));
            }
            if level == 0 {
                c.error("break level must be at least 1");
            }
            let target_loop_idx = c.loop_stack.len() - level;
            let target_loop = &c.loop_stack[target_loop_idx];
            let locals_to_pop = c.local_count - target_loop.local_count;
            for _ in 0..locals_to_pop {
                c.chunk.instructions.push(Instruction::Pop);
            }
            let jump_ip = c.emit_jump();
            c.loop_stack[target_loop_idx].break_patches.push(jump_ip);
        }
        Stmt::Continue {
            level: level_opt, ..
        } => {
            let level = level_opt.unwrap_or(1) as usize;
            if level > c.loop_stack.len() {
                c.error(&format!(
                    "continue {} exceeds loop nesting depth of {}",
                    level,
                    c.loop_stack.len()
                ));
            }
            if level == 0 {
                c.error("continue level must be at least 1");
            }
            let target_loop_idx = c.loop_stack.len() - level;
            let target_loop = &c.loop_stack[target_loop_idx];
            let locals_to_pop = c.local_count - target_loop.local_count;
            for _ in 0..locals_to_pop {
                c.chunk.instructions.push(Instruction::Pop);
            }
            if let Some(target_ip) = target_loop.continue_target {
                c.chunk.instructions.push(Instruction::Jump(target_ip));
            } else {
                let jump_ip = c.emit_jump();
                c.loop_stack[target_loop_idx].continue_patches.push(jump_ip);
            }
        }
        Stmt::ExprStmt { expr, .. } => {
            c.emit_expr(expr);
            c.chunk.instructions.push(Instruction::Pop);
        }
    }
}
