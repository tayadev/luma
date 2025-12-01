use super::compile::Compiler;
use super::helpers::HIDDEN_MATCH_VAL;
use super::ir::{Constant, Instruction};
use crate::ast::{BinaryOp, CallArgument, Expr, LogicalOp, TableKey, UnaryOp};

impl Compiler {
    pub(super) fn emit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Number { value: n, .. } => {
                let idx = self.push_const(Constant::Number(*n));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::String { value: s, .. } => {
                let idx = self.push_const(Constant::String(s.clone()));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Boolean { value: b, .. } => {
                let idx = self.push_const(Constant::Boolean(*b));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Null { .. } => {
                let idx = self.push_const(Constant::Null);
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
                for field in fields {
                    match &field.key {
                        TableKey::Identifier(s) | TableKey::StringLiteral(s) => {
                            let k_idx = self.push_const(Constant::String(s.clone()));
                            self.chunk.instructions.push(Instruction::Const(k_idx));
                        }
                        TableKey::Computed(expr) => {
                            self.emit_expr(expr);
                        }
                    }
                    self.emit_expr(&field.value);
                }
                self.chunk
                    .instructions
                    .push(Instruction::BuildTable(fields.len()));
            }
            Expr::MemberAccess { object, member, .. } => {
                self.emit_expr(object);
                let name_idx = self.push_const(Constant::String(member.clone()));
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
                    let name_idx = self.push_const(Constant::String(name.clone()));
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
            } => match op {
                LogicalOp::And => {
                    self.emit_expr(left);
                    self.chunk.instructions.push(Instruction::Dup);
                    let jf = self.emit_jump_if_false();
                    self.chunk.instructions.push(Instruction::Pop);
                    self.emit_expr(right);
                    let end = self.current_ip();
                    self.patch_jump(jf, end);
                }
                LogicalOp::Or => {
                    self.emit_expr(left);
                    self.chunk.instructions.push(Instruction::Dup);
                    let jf = self.emit_jump_if_false();
                    let jend = self.emit_jump();
                    let after_jf = self.current_ip();
                    self.patch_jump(jf, after_jf);
                    self.chunk.instructions.push(Instruction::Pop);
                    self.emit_expr(right);
                    let end = self.current_ip();
                    self.patch_jump(jend, end);
                }
            },
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
                let (fn_chunk, upvalue_descriptors) = self.compile_nested_function(arguments, body);
                let idx = self.push_const(Constant::Function(fn_chunk));
                if upvalue_descriptors.is_empty() {
                    self.chunk.instructions.push(Instruction::MakeFunction(idx));
                } else {
                    self.chunk.instructions.push(Instruction::Closure(idx));
                }
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                let has_named = arguments
                    .iter()
                    .any(|a| matches!(a, CallArgument::Named { .. }));
                if !has_named {
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
                    let mut seen_named = false;
                    for arg in arguments {
                        match arg {
                            CallArgument::Named { .. } => seen_named = true,
                            CallArgument::Positional(_) if seen_named => {
                                self.error("Positional arguments cannot follow named arguments");
                            }
                            _ => {}
                        }
                    }

                    let param_names: Vec<String> = match &**callee {
                        Expr::Function {
                            arguments: fn_args, ..
                        } => fn_args.iter().map(|a| a.name.clone()).collect(),
                        Expr::Identifier { name: n, .. } => {
                            self.lookup_fn_params(n).unwrap_or_else(|| {
                                self.error(&format!(
                                    "Named arguments require statically-known callee '{n}'"
                                ));
                            })
                        }
                        _ => self.error("Named arguments require statically-known callee"),
                    };

                    let mut positional: Vec<&Expr> = Vec::new();
                    let mut named_map: std::collections::HashMap<&str, &Expr> =
                        std::collections::HashMap::new();
                    for arg in arguments {
                        match arg {
                            CallArgument::Positional(expr) => positional.push(expr),
                            CallArgument::Named { name, value } => {
                                if named_map.contains_key(name.as_str()) {
                                    self.error(&format!("Duplicate named argument '{name}'"));
                                }
                                named_map.insert(name.as_str(), value);
                            }
                        }
                    }

                    if param_names.len() < positional.len() {
                        self.error(&format!(
                            "Too many positional arguments: expected {} got {}",
                            param_names.len(),
                            positional.len()
                        ));
                    }

                    let mut final_args: Vec<&Expr> = Vec::with_capacity(param_names.len());
                    for (i, pname) in param_names.iter().enumerate() {
                        if i < positional.len() {
                            final_args.push(positional[i]);
                        } else if let Some(v) = named_map.get::<str>(pname.as_str()) {
                            final_args.push(*v);
                        } else {
                            self.error(&format!("Missing required argument '{pname}'"));
                        }
                    }

                    self.emit_expr(callee);
                    for e in final_args {
                        self.emit_expr(e);
                    }
                    self.chunk
                        .instructions
                        .push(Instruction::Call(param_names.len()));
                }
            }
            Expr::MethodCall {
                object,
                method,
                arguments,
                ..
            } => {
                // Desugar: object:method(args) to object.method(object, args)
                self.emit_expr(object);
                let method_idx = self.push_const(Constant::String(method.clone()));
                self.chunk
                    .instructions
                    .push(Instruction::GetProp(method_idx));
                self.emit_expr(object); // Insert object as first argument
                for arg in arguments {
                    match arg {
                        CallArgument::Positional(expr) => self.emit_expr(expr),
                        CallArgument::Named { value, .. } => self.emit_expr(value),
                    }
                }
                self.chunk
                    .instructions
                    .push(Instruction::Call(arguments.len() + 1));
            }
            Expr::Block {
                statements: stmts, ..
            } => {
                self.enter_scope();
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
                self.emit_expr(condition);
                let jf = self.emit_jump_if_false();
                self.enter_scope();
                self.predeclare_function_locals(then_block);
                for stmt in then_block {
                    self.emit_stmt(stmt);
                }
                self.exit_scope_with_preserve(true);

                if let Some(else_stmts) = else_block {
                    let jend = self.emit_jump();
                    let else_start = self.current_ip();
                    self.patch_jump(jf, else_start);

                    self.enter_scope();
                    self.predeclare_function_locals(else_stmts);
                    for stmt in else_stmts {
                        self.emit_stmt(stmt);
                    }
                    self.exit_scope_with_preserve(true);

                    let end = self.current_ip();
                    self.patch_jump(jend, end);
                } else {
                    let jend = self.emit_jump();
                    let else_start = self.current_ip();
                    self.patch_jump(jf, else_start);
                    let null_idx = self.push_const(Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                    let end = self.current_ip();
                    self.patch_jump(jend, end);
                }
            }
            Expr::Import { path, .. } => {
                self.emit_expr(path);
                self.chunk.instructions.push(Instruction::Import);
            }
            Expr::Match { expr, arms, .. } => {
                self.enter_scope();
                self.emit_expr(expr);
                let match_val_slot = self.local_count;
                self.bind_hidden_local(HIDDEN_MATCH_VAL.to_string(), match_val_slot);
                self.local_count += 1;

                let mut end_jumps = Vec::new();
                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let is_wildcard = matches!(pattern, crate::ast::Pattern::Wildcard { .. });
                    let is_tag_pattern = matches!(pattern, crate::ast::Pattern::Ident { name, .. } if matches!(name.as_str(), "ok" | "err" | "some" | "none"));
                    let is_catch_all = is_wildcard
                        || (matches!(pattern, crate::ast::Pattern::Ident { name: _, .. })
                            && !is_tag_pattern);

                    if !is_catch_all {
                        match pattern {
                            crate::ast::Pattern::Ident { name, .. } => {
                                self.emit_get_local(match_val_slot);
                                let name_idx = self.push_const(Constant::String(name.clone()));
                                self.chunk.instructions.push(Instruction::GetProp(name_idx));
                                self.push_null();
                                self.chunk.instructions.push(Instruction::Ne);
                                let jf_next = self.emit_jump_if_false();

                                let arm_body = super::compile::apply_implicit_return_to_arm(body);
                                for st in &arm_body {
                                    self.emit_stmt(st);
                                }
                                if !super::compile::does_block_leave_value(&arm_body) {
                                    self.push_null();
                                }
                                end_jumps.push(self.emit_jump());
                                let next_ip = self.current_ip();
                                self.patch_jump(jf_next, next_ip);
                            }
                            crate::ast::Pattern::Literal { value: lit, .. } => {
                                self.emit_get_local(match_val_slot);
                                match lit {
                                    crate::ast::Literal::Number(n) => {
                                        self.push_number(*n);
                                    }
                                    crate::ast::Literal::String(s) => {
                                        self.push_string(s.clone());
                                    }
                                    crate::ast::Literal::Boolean(b) => {
                                        self.push_boolean(*b);
                                    }
                                    crate::ast::Literal::Null => {
                                        self.push_null();
                                    }
                                }
                                self.chunk.instructions.push(Instruction::Eq);
                                let jf_next = self.emit_jump_if_false();

                                let arm_body = super::compile::apply_implicit_return_to_arm(body);
                                for st in &arm_body {
                                    self.emit_stmt(st);
                                }
                                if !super::compile::does_block_leave_value(&arm_body) {
                                    self.push_null();
                                }
                                end_jumps.push(self.emit_jump());
                                let next_ip = self.current_ip();
                                self.patch_jump(jf_next, next_ip);
                            }
                            _ => self.error("structural patterns for match expr not yet supported"),
                        }
                    } else {
                        let arm_body = super::compile::apply_implicit_return_to_arm(body);
                        for st in &arm_body {
                            self.emit_stmt(st);
                        }
                        if !super::compile::does_block_leave_value(&arm_body) {
                            self.push_null();
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
            }
        }
    }
}
