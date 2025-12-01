//! Statement type checking.

use crate::ast::*;

use super::environment::TypeEnv;
use super::types::{TcType, VarInfo};

impl TypeEnv {
    /// Type check a block of statements and return the block's type.
    pub fn check_block(&mut self, stmts: &[Stmt], expected_ret: &TcType) -> TcType {
        let mut ret_ty = TcType::Null;

        // Predeclare local function variables in this block to support mutual recursion
        // and allow references within the same scope before their textual definition.
        for stmt in stmts {
            if let Stmt::VarDecl {
                mutable,
                name,
                r#type,
                value,
                ..
            } = stmt
                && let Expr::Function {
                    arguments,
                    return_type,
                    ..
                } = value
            {
                // Determine function type from signature
                let mut param_types = Vec::new();
                for arg in arguments {
                    param_types.push(Self::type_from_ast(&arg.r#type));
                }
                let ret_ty_annot = if let Some(rt) = return_type {
                    Self::type_from_ast(rt)
                } else {
                    TcType::Unknown
                };
                let func_ty = TcType::Function {
                    params: param_types,
                    ret: Box::new(ret_ty_annot),
                };
                self.declare(
                    name.clone(),
                    VarInfo {
                        ty: func_ty,
                        mutable: *mutable,
                        annotated: r#type.is_some(),
                    },
                );
            }
        }

        let len = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            let is_last = i == len - 1;
            match stmt {
                Stmt::Return { value: expr, span } => {
                    ret_ty = self.check_expr(expr);
                    if !ret_ty.is_compatible(expected_ret) && *expected_ret != TcType::Unknown {
                        self.error(
                            format!("Return type mismatch: expected {expected_ret}, got {ret_ty}"),
                            *span,
                        );
                    }
                }
                _ => {
                    self.check_stmt(stmt);
                    // For the last statement, compute implicit return type from control flow
                    if is_last {
                        ret_ty = self.compute_implicit_return_type(stmt, expected_ret);
                    }
                }
            }
        }

        ret_ty
    }

    /// Compute the implicit return type of a statement when it's the last statement in a block.
    /// This handles control flow statements (if, match) that may implicitly return values.
    fn compute_implicit_return_type(&mut self, stmt: &Stmt, expected_ret: &TcType) -> TcType {
        match stmt {
            Stmt::If {
                then_block,
                elif_blocks,
                else_block,
                span,
                ..
            } => {
                // For an if statement to be a valid implicit return, we need an else block
                // (otherwise not all paths return a value)
                if else_block.is_none() {
                    return TcType::Null;
                }

                // Compute return type of each branch
                let then_ret = self.compute_block_return_type(then_block, expected_ret);

                let mut unified = then_ret;

                for (_, block) in elif_blocks {
                    let elif_ret = self.compute_block_return_type(block, expected_ret);
                    unified = self.unify_return_types(unified, elif_ret);
                }

                if let Some(block) = else_block {
                    let else_ret = self.compute_block_return_type(block, expected_ret);
                    unified = self.unify_return_types(unified, else_ret);
                }

                // Check if the unified type matches expected
                if !unified.is_compatible(expected_ret) && *expected_ret != TcType::Unknown {
                    self.error(
                        format!("Return type mismatch: expected {expected_ret}, got {unified}"),
                        *span,
                    );
                }

                unified
            }
            Stmt::Match { arms, span, .. } => {
                if arms.is_empty() {
                    return TcType::Null;
                }

                let mut unified: Option<TcType> = None;

                for (_, body) in arms {
                    let arm_ret = self.compute_block_return_type(body, expected_ret);
                    unified = Some(match unified {
                        None => arm_ret,
                        Some(current) => self.unify_return_types(current, arm_ret),
                    });
                }

                let result = unified.unwrap_or(TcType::Null);

                // Check if the unified type matches expected
                if !result.is_compatible(expected_ret) && *expected_ret != TcType::Unknown {
                    self.error(
                        format!("Return type mismatch: expected {expected_ret}, got {result}"),
                        *span,
                    );
                }

                result
            }
            _ => TcType::Null,
        }
    }

    /// Compute the return type of a block by looking at its last statement.
    fn compute_block_return_type(&mut self, stmts: &[Stmt], expected_ret: &TcType) -> TcType {
        if let Some(last) = stmts.last() {
            match last {
                Stmt::Return { value: expr, .. } => self.check_expr(expr),
                _ => self.compute_implicit_return_type(last, expected_ret),
            }
        } else {
            TcType::Null
        }
    }

    /// Unify two return types, preferring the more specific type.
    fn unify_return_types(&self, a: TcType, b: TcType) -> TcType {
        if a.is_compatible(&b) {
            a
        } else if b.is_compatible(&a) {
            b
        } else {
            TcType::Unknown
        }
    }

    /// Type check a single statement.
    pub fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Match { expr, arms, .. } => {
                // Check the match expression
                let expr_ty = self.check_expr(expr);

                // Check for unreachable patterns
                self.check_unreachable_patterns(arms);

                // Check exhaustiveness
                self.check_match_exhaustiveness(arms, Some(&expr_ty), stmt.span());

                // For each arm, check the pattern and body
                for (pattern, body) in arms {
                    self.push_scope();
                    // Bind pattern variables with the matched expression's type
                    self.check_pattern(pattern, &expr_ty, false, true); // match bindings are immutable

                    // Check the body statements
                    self.in_match_arm_depth += 1;
                    for stmt in body {
                        self.check_stmt(stmt);
                    }
                    self.in_match_arm_depth -= 1;
                    self.pop_scope();
                }
            }
            Stmt::VarDecl {
                mutable,
                name,
                r#type,
                value,
                span,
            } => {
                // For function values, we already pre-declared them in typecheck_program
                // Just check the function body here
                let value_ty = match value {
                    Expr::Function { .. } => {
                        // Function was already declared, just check its body
                        self.check_expr(value)
                    }
                    _ => {
                        // Non-function: check value and declare normally
                        let val_ty = self.check_expr(value);

                        let declared_ty = if let Some(ty) = r#type {
                            let t = Self::type_from_ast(ty);
                            if !val_ty.is_compatible(&t) {
                                self.error(
                                    format!("Variable {name}: declared type {t}, got {val_ty}"),
                                    *span,
                                );
                            }
                            t
                        } else {
                            val_ty.clone()
                        };

                        self.declare(
                            name.clone(),
                            VarInfo {
                                ty: declared_ty,
                                mutable: *mutable,
                                annotated: r#type.is_some(),
                            },
                        );

                        val_ty
                    }
                };

                // Verify declared type matches if annotated (for functions)
                if matches!(value, Expr::Function { .. })
                    && let Some(ty) = r#type
                {
                    let declared = Self::type_from_ast(ty);
                    if !value_ty.is_compatible(&declared) {
                        self.error(
                            format!("Variable {name}: declared type {declared}, got {value_ty}"),
                            *span,
                        );
                    }
                }
            }

            Stmt::DestructuringVarDecl {
                mutable,
                pattern,
                value,
                ..
            } => {
                let value_ty = self.check_expr(value);
                self.check_pattern(pattern, &value_ty, *mutable, false);
            }

            Stmt::Assignment {
                target,
                op: _,
                value,
                span,
            } => {
                let target_ty = self.check_assignment_target(target);
                let value_ty = self.check_expr(value);

                if !value_ty.is_compatible(&target_ty) {
                    self.error(
                        format!("Assignment type mismatch: target {target_ty}, value {value_ty}"),
                        *span,
                    );
                }
            }

            Stmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                self.expect_type(condition, &TcType::Boolean, "If condition");

                self.push_scope();
                for stmt in then_block {
                    self.check_stmt(stmt);
                }
                self.pop_scope();

                for (cond, block) in elif_blocks {
                    self.expect_type(cond, &TcType::Boolean, "Elif condition");
                    self.push_scope();
                    for stmt in block {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }

                if let Some(block) = else_block {
                    self.push_scope();
                    for stmt in block {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }
            }

            Stmt::While {
                condition, body, ..
            } => {
                self.expect_type(condition, &TcType::Boolean, "While condition");
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }

            Stmt::DoWhile {
                body, condition, ..
            } => {
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                self.expect_type(condition, &TcType::Boolean, "Do-while condition");
            }

            Stmt::For {
                pattern,
                iterator,
                body,
                span,
            } => {
                let iter_ty = self.check_expr(iterator);

                self.push_scope();
                match &iter_ty {
                    TcType::List(elem_ty) => {
                        self.check_pattern(pattern, elem_ty, true, false);
                    }
                    TcType::Table | TcType::TableWithFields(_) => {
                        // Iteration over tables yields [key, value] pairs
                        let pair_elem = TcType::List(Box::new(TcType::Unknown));
                        self.check_pattern(pattern, &pair_elem, true, false);
                    }
                    TcType::Unknown | TcType::Any => {
                        self.check_pattern(pattern, &TcType::Unknown, true, false);
                    }
                    _ => {
                        self.error(
                            format!("For loop requires List or Table iterator, got {iter_ty}"),
                            *span,
                        );
                        self.check_pattern(pattern, &TcType::Unknown, true, false);
                    }
                }

                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }

            Stmt::Break { level: _, .. } | Stmt::Continue { level: _, .. } => {
                // TODO: Could check if we're inside a loop
            }

            Stmt::Return { value: expr, .. } => {
                self.check_expr(expr);
            }

            Stmt::ExprStmt { expr, .. } => {
                self.check_expr(expr);
            }
        }
    }

    /// Type check an assignment target and return its type.
    pub fn check_assignment_target(&mut self, target: &Expr) -> TcType {
        match target {
            Expr::Identifier { name, span } => {
                if let Some(info) = self.lookup(name) {
                    let ty = info.ty.clone();
                    let mutable = info.mutable;
                    if !mutable {
                        self.error(
                            format!("Cannot assign to immutable variable: {name}"),
                            *span,
                        );
                    }
                    ty
                } else {
                    self.error(format!("Undefined variable: {name}"), *span);
                    TcType::Unknown
                }
            }
            Expr::MemberAccess {
                object,
                member: _,
                span,
            } => {
                let obj_ty = self.check_expr(object);
                match obj_ty {
                    TcType::Table | TcType::TableWithFields(_) => TcType::Unknown,
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(
                            format!("Member assignment requires a table, got {obj_ty}"),
                            *span,
                        );
                        TcType::Unknown
                    }
                }
            }
            Expr::Index {
                object,
                index,
                span,
            } => {
                let obj_ty = self.check_expr(object);
                let idx_ty = self.check_expr(index);

                match obj_ty {
                    TcType::List(elem_ty) => {
                        if !idx_ty.is_compatible(&TcType::Number) {
                            self.error(format!("List index requires Number, got {idx_ty}"), *span);
                        }
                        (*elem_ty).clone()
                    }
                    TcType::Table => {
                        if !idx_ty.is_compatible(&TcType::String) {
                            self.error(format!("Table index requires String, got {idx_ty}"), *span);
                        }
                        TcType::Unknown
                    }
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(
                            format!("Index assignment requires List or Table, got {obj_ty}"),
                            *span,
                        );
                        TcType::Unknown
                    }
                }
            }
            _ => {
                self.error("Invalid assignment target".to_string(), target.span());
                TcType::Unknown
            }
        }
    }
}
