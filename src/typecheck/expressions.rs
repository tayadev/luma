//! Expression type checking.

use crate::ast::*;

use super::environment::TypeEnv;
use super::types::TcType;

impl TypeEnv {
    /// Type check an expression and return its type.
    pub fn check_expr(&mut self, expr: &Expr) -> TcType {
        match expr {
            Expr::Number { value: _, .. } => TcType::Number,
            Expr::String { value: _, .. } => TcType::String,
            Expr::Boolean { value: _, .. } => TcType::Boolean,
            Expr::Null { .. } => TcType::Null,

            Expr::Identifier { name, span, .. } => {
                if let Some(info) = self.lookup(name) {
                    info.ty.clone()
                } else {
                    self.error(format!("Undefined variable: {}", name), *span);
                    TcType::Unknown
                }
            }

            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    TcType::List(Box::new(TcType::Unknown))
                } else {
                    let first_ty = self.check_expr(&elements[0]);
                    for elem in &elements[1..] {
                        let ty = self.check_expr(elem);
                        if !ty.is_compatible(&first_ty) {
                            self.error(
                                format!(
                                    "List elements have inconsistent types: {} vs {}",
                                    first_ty, ty
                                ),
                                elem.span(),
                            );
                        }
                    }
                    TcType::List(Box::new(first_ty))
                }
            }

            Expr::Table {
                fields: entries, ..
            } => {
                for (_, value) in entries {
                    self.check_expr(value);
                }
                // Collect identifier and string literal keys for structural presence
                let mut fields = Vec::new();
                for (k, _) in entries {
                    match k {
                        TableKey::Identifier(s) | TableKey::StringLiteral(s) => {
                            fields.push(s.clone())
                        }
                        TableKey::Computed(_) => {}
                    }
                }
                // Deduplicate while preserving order
                let mut seen = std::collections::HashSet::new();
                fields.retain(|f| seen.insert(f.clone()));
                TcType::TableWithFields(fields)
            }

            Expr::Binary {
                left,
                op,
                right,
                span,
            } => self.check_binary_expr(left, op, right, *span),

            Expr::Unary { op, operand, span } => self.check_unary_expr(op, operand, *span),

            Expr::Logical {
                left, op: _, right, ..
            } => {
                self.expect_type(left, &TcType::Boolean, "Logical op left operand");
                self.expect_type(right, &TcType::Boolean, "Logical op right operand");
                TcType::Boolean
            }

            Expr::Call {
                callee,
                arguments,
                span,
            } => self.check_call_expr(callee, arguments, *span),

            Expr::MemberAccess {
                object,
                member,
                span,
            } => self.check_member_access(object, member, *span),

            Expr::Index {
                object,
                index,
                span,
            } => self.check_index_expr(object, index, *span),

            Expr::Function {
                arguments,
                return_type,
                body,
                span,
            } => self.check_function_expr(arguments, return_type.as_ref(), body, *span),

            Expr::Block {
                statements: stmts, ..
            } => {
                self.push_scope();
                let ret_ty = self.check_block(stmts, &TcType::Unknown);
                self.pop_scope();
                ret_ty
            }

            Expr::If {
                condition,
                then_block,
                else_block,
                span,
            } => self.check_if_expr(condition, then_block, else_block.as_deref(), *span),

            Expr::Import { path, span } => {
                // Check that path is a string expression
                let path_ty = self.check_expr(path);
                if !path_ty.is_compatible(&TcType::String) && path_ty != TcType::Unknown {
                    self.error(
                        format!("Import path should be a String, got {}", path_ty),
                        *span,
                    );
                }
                // Import returns the module's exported value
                // For now, we type it as Unknown (proper typing would require module analysis)
                TcType::Unknown
            }

            Expr::Match { expr, arms, span } => self.check_match_expr(expr, arms, *span),
        }
    }

    fn check_binary_expr(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
        span: Option<Span>,
    ) -> TcType {
        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);

        match op {
            BinaryOp::Add => {
                // Allow String + String → String OR Number + Number → Number
                if left_ty.is_compatible(&TcType::String) && right_ty.is_compatible(&TcType::String)
                {
                    TcType::String
                } else if left_ty.is_compatible(&TcType::Number)
                    && right_ty.is_compatible(&TcType::Number)
                {
                    TcType::Number
                } else {
                    self.error(
                        format!(
                            "ADD requires (Number, Number) or (String, String), got ({}, {})",
                            left_ty, right_ty
                        ),
                        span,
                    );
                    TcType::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Check if both operands are Numbers (default case)
                if left_ty.is_compatible(&TcType::Number) && right_ty.is_compatible(&TcType::Number)
                {
                    TcType::Number
                } else {
                    // Check for operator method fallback
                    // The method receives both operands; we can't validate right operand type
                    // without full method signature info (table fields don't have type info).
                    // Runtime will validate when the method executes.
                    let method_name = match op {
                        BinaryOp::Sub => "__sub",
                        BinaryOp::Mul => "__mul",
                        BinaryOp::Div => "__div",
                        BinaryOp::Mod => "__mod",
                        _ => unreachable!(),
                    };

                    if Self::has_operator_method(&left_ty, method_name) {
                        TcType::Unknown // Return type depends on implementation
                    } else {
                        self.error(
                            format!(
                                "Arithmetic op {:?} requires Number operands or type with {} method, got ({}, {})",
                                op, method_name, left_ty, right_ty
                            ),
                            span,
                        );
                        TcType::Unknown
                    }
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Allow any types for equality comparison
                TcType::Boolean
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Check if both operands are Numbers (default case)
                if left_ty.is_compatible(&TcType::Number) && right_ty.is_compatible(&TcType::Number)
                {
                    TcType::Boolean
                } else {
                    // Check for operator method fallback
                    // The method receives both operands; we can't validate right operand type
                    // without full method signature info (table fields don't have type info).
                    // Runtime will validate when the method executes.
                    let method_name = match op {
                        BinaryOp::Lt => "__lt",
                        BinaryOp::Le => "__le",
                        BinaryOp::Gt => "__gt",
                        BinaryOp::Ge => "__ge",
                        _ => unreachable!(),
                    };

                    if Self::has_operator_method(&left_ty, method_name) {
                        TcType::Boolean // Comparison methods should return Boolean
                    } else {
                        self.error(
                            format!(
                                "Comparison op {:?} requires Number operands or type with {} method, got ({}, {})",
                                op, method_name, left_ty, right_ty
                            ),
                            span,
                        );
                        TcType::Boolean
                    }
                }
            }
        }
    }

    fn check_unary_expr(&mut self, op: &UnaryOp, operand: &Expr, span: Option<Span>) -> TcType {
        match op {
            UnaryOp::Neg => {
                let ty = self.check_expr(operand);
                if ty.is_compatible(&TcType::Number) {
                    TcType::Number
                } else if Self::has_operator_method(&ty, "__neg") {
                    TcType::Unknown // Return type depends on implementation
                } else {
                    self.error(
                        format!(
                            "Unary negation requires Number or type with __neg method, got {}",
                            ty
                        ),
                        span,
                    );
                    TcType::Unknown
                }
            }
            UnaryOp::Not => {
                self.expect_type(operand, &TcType::Boolean, "Logical not");
                TcType::Boolean
            }
        }
    }

    fn check_call_expr(
        &mut self,
        callee: &Expr,
        arguments: &[CallArgument],
        span: Option<Span>,
    ) -> TcType {
        let callee_ty = self.check_expr(callee);
        match callee_ty {
            TcType::Function { params, ret } => {
                if arguments.len() != params.len() {
                    self.error(
                        format!(
                            "Function call: expected {} arguments, got {}",
                            params.len(),
                            arguments.len()
                        ),
                        span,
                    );
                } else {
                    for (i, (arg, param_ty)) in arguments.iter().zip(params.iter()).enumerate() {
                        // Extract the expression from the CallArgument
                        let arg_expr = match arg {
                            CallArgument::Positional(expr) => expr,
                            CallArgument::Named { value, .. } => value,
                        };
                        let arg_ty = self.check_expr(arg_expr);
                        if !arg_ty.is_compatible(param_ty) {
                            self.error(
                                format!(
                                    "Function call: argument {} expected {}, got {}",
                                    i, param_ty, arg_ty
                                ),
                                arg_expr.span(),
                            );
                        }
                    }
                }
                (*ret).clone()
            }
            TcType::Unknown | TcType::Any => {
                // Check arguments but return Unknown
                for arg in arguments {
                    let arg_expr = match arg {
                        CallArgument::Positional(expr) => expr,
                        CallArgument::Named { value, .. } => value,
                    };
                    self.check_expr(arg_expr);
                }
                TcType::Unknown
            }
            _ => {
                self.error(
                    format!("Call expression requires a function, got {}", callee_ty),
                    span,
                );
                TcType::Unknown
            }
        }
    }

    fn check_member_access(&mut self, object: &Expr, member: &str, span: Option<Span>) -> TcType {
        let obj_ty = self.check_expr(object);
        match obj_ty {
            TcType::Table => TcType::Unknown, // dynamic tables allowed
            TcType::TableWithFields(ref fields) => {
                if !fields.contains(&member.to_string()) && self.in_match_arm_depth == 0 {
                    self.error(format!("Unknown field '{}' on table", member), span);
                }
                TcType::Unknown
            }
            TcType::Unknown | TcType::Any => TcType::Unknown,
            _ => {
                self.error(
                    format!("Member access requires a table, got {}", obj_ty),
                    span,
                );
                TcType::Unknown
            }
        }
    }

    fn check_index_expr(&mut self, object: &Expr, index: &Expr, span: Option<Span>) -> TcType {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);

        match obj_ty {
            TcType::List(elem_ty) => {
                if !idx_ty.is_compatible(&TcType::Number) {
                    self.error(format!("List index requires Number, got {}", idx_ty), span);
                }
                (*elem_ty).clone()
            }
            TcType::Table | TcType::TableWithFields(_) => {
                if !idx_ty.is_compatible(&TcType::String) {
                    self.error(format!("Table index requires String, got {}", idx_ty), span);
                }
                TcType::Unknown
            }
            TcType::Unknown | TcType::Any => TcType::Unknown,
            _ => {
                self.error(
                    format!("Index operation requires List or Table, got {}", obj_ty),
                    span,
                );
                TcType::Unknown
            }
        }
    }

    fn check_function_expr(
        &mut self,
        arguments: &[Argument],
        return_type: Option<&Type>,
        body: &[Stmt],
        span: Option<Span>,
    ) -> TcType {
        self.push_scope();

        let mut param_types = Vec::new();
        for arg in arguments {
            let param_ty = Self::type_from_ast(&arg.r#type);
            param_types.push(param_ty.clone());
            self.declare(
                arg.name.clone(),
                super::types::VarInfo {
                    ty: param_ty,
                    mutable: true, // Function params are mutable in MVP
                    annotated: true,
                },
            );
        }

        let expected_ret = if let Some(ret_type) = return_type {
            Self::type_from_ast(ret_type)
        } else {
            TcType::Unknown
        };

        let actual_ret = self.check_block(body, &expected_ret);

        if !actual_ret.is_compatible(&expected_ret) && expected_ret != TcType::Unknown {
            self.error(
                format!(
                    "Function return type mismatch: declared {}, got {}",
                    expected_ret, actual_ret
                ),
                span,
            );
        }

        self.pop_scope();

        // Use declared return type if provided to propagate function type outward
        let ret_ty = if !matches!(expected_ret, TcType::Unknown) {
            expected_ret
        } else {
            actual_ret
        };

        TcType::Function {
            params: param_types,
            ret: Box::new(ret_ty),
        }
    }

    fn check_if_expr(
        &mut self,
        condition: &Expr,
        then_block: &[Stmt],
        else_block: Option<&[Stmt]>,
        span: Option<Span>,
    ) -> TcType {
        // Check condition
        let cond_ty = self.check_expr(condition);
        if !cond_ty.is_compatible(&TcType::Boolean) && cond_ty != TcType::Unknown {
            self.error(
                format!("If condition should be Boolean, got {}", cond_ty),
                span,
            );
        }

        // Check then block
        self.push_scope();
        let then_ty = self.check_block(then_block, &TcType::Unknown);
        self.pop_scope();

        // Check else block if present
        if let Some(else_stmts) = else_block {
            self.push_scope();
            let else_ty = self.check_block(else_stmts, &TcType::Unknown);
            self.pop_scope();

            // Type is the common type of both branches
            if then_ty.is_compatible(&else_ty) {
                then_ty
            } else if else_ty.is_compatible(&then_ty) {
                else_ty
            } else {
                self.error(
                    format!(
                        "If branches have incompatible types: {} vs {}",
                        then_ty, else_ty
                    ),
                    span,
                );
                TcType::Unknown
            }
        } else {
            // No else branch: could be null
            then_ty
        }
    }

    fn check_match_expr(
        &mut self,
        expr: &Expr,
        arms: &[(Pattern, Vec<Stmt>)],
        span: Option<Span>,
    ) -> TcType {
        // Type of the matched expression
        let matched_ty = self.check_expr(expr);

        // Check for unreachable patterns
        self.check_unreachable_patterns(arms);

        // Check exhaustiveness
        self.check_match_exhaustiveness(arms, Some(&matched_ty), expr.span());

        let mut unified_ret: Option<TcType> = None;
        for (pattern, body) in arms {
            self.push_scope();
            // Bind pattern variables assuming matched expression type
            self.check_pattern(pattern, &matched_ty, false, true);
            // Determine arm return type similar to check_block
            self.in_match_arm_depth += 1;
            let arm_ret = self.check_block(body, &TcType::Unknown);
            self.in_match_arm_depth -= 1;
            self.pop_scope();
            if let Some(current) = &unified_ret {
                if current.is_compatible(&arm_ret) {
                    // keep current
                } else if arm_ret.is_compatible(current) {
                    unified_ret = Some(arm_ret);
                } else {
                    self.error(
                        format!(
                            "Match arms have incompatible types: {} vs {}",
                            current, arm_ret
                        ),
                        span,
                    );
                    unified_ret = Some(TcType::Unknown);
                }
            } else {
                unified_ret = Some(arm_ret);
            }
        }
        unified_ret.unwrap_or(TcType::Null)
    }
}
