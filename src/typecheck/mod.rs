use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

pub type TypecheckResult<T> = Result<T, Vec<TypeError>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TcType {
    Any,
    Unknown,
    Number,
    String,
    Boolean,
    Null,
    List(Box<TcType>),
    Table,
    // Table with a known (best-effort) set of fields; used for structural presence checks
    TableWithFields(Vec<String>),
    Function {
        params: Vec<TcType>,
        ret: Box<TcType>,
    },
}

impl TcType {
    fn is_compatible(&self, other: &TcType) -> bool {
        match (self, other) {
            (TcType::Any, _) | (_, TcType::Any) => true,
            (TcType::Unknown, _) | (_, TcType::Unknown) => true,
            (TcType::Number, TcType::Number) => true,
            (TcType::String, TcType::String) => true,
            (TcType::Boolean, TcType::Boolean) => true,
            (TcType::Null, TcType::Null) => true,
            (TcType::List(a), TcType::List(b)) => a.is_compatible(b),
            (TcType::Table, TcType::Table) => true,
            (TcType::TableWithFields(_), TcType::Table) => true,
            (TcType::Table, TcType::TableWithFields(_)) => true,
            (TcType::TableWithFields(_), TcType::TableWithFields(_)) => true,
            (TcType::Function { params: p1, ret: r1 }, TcType::Function { params: p2, ret: r2 }) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(a, b)| a.is_compatible(b))
                    && r1.is_compatible(r2)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: TcType,
    mutable: bool,
    #[allow(dead_code)]
    annotated: bool,
}

struct TypeEnv {
    scopes: Vec<HashMap<String, VarInfo>>,
    errors: Vec<TypeError>,
    // Track match arm context to relax certain checks inside arms
    in_match_arm_depth: usize,
}

impl TypeEnv {
    fn new() -> Self {
        let mut env = TypeEnv {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
            in_match_arm_depth: 0,
        };
        
        // Register built-in functions
        env.declare("cast".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Any, TcType::Any],
                ret: Box::new(TcType::Any),
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("isInstanceOf".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Any, TcType::Any],
                ret: Box::new(TcType::Boolean),
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("into".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Any, TcType::Any],
                ret: Box::new(TcType::Any),
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("typeof".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Any],
                ret: Box::new(TcType::String),
            },
            mutable: false,
            annotated: true,
        });
        
        // print is variadic - we use Any to accept any number of arguments
        // The actual arity check is skipped for print in the VM
        env.declare("print".to_string(), VarInfo {
            ty: TcType::Any,  // Variadic function - any type
            mutable: false,
            annotated: true,
        });
        
        // Register I/O native functions
        env.declare("write".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Number, TcType::Any],
                ret: Box::new(TcType::Table),  // Returns Result
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("read_file".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::String],
                ret: Box::new(TcType::Table),  // Returns Result
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("write_file".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::String, TcType::Any],
                ret: Box::new(TcType::Table),  // Returns Result
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("file_exists".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::String],
                ret: Box::new(TcType::Boolean),
            },
            mutable: false,
            annotated: true,
        });
        
        env.declare("panic".to_string(), VarInfo {
            ty: TcType::Function {
                params: vec![TcType::Any],
                ret: Box::new(TcType::Any),  // Never returns, but use Any
            },
            mutable: false,
            annotated: true,
        });
        
        // Register file descriptor constants
        env.declare("STDOUT".to_string(), VarInfo {
            ty: TcType::Number,
            mutable: false,
            annotated: true,
        });
        
        env.declare("STDERR".to_string(), VarInfo {
            ty: TcType::Number,
            mutable: false,
            annotated: true,
        });
        
        // Register prelude types/tables (from prelude.luma)
        // These are tables containing methods/constructors
        env.declare("Result".to_string(), VarInfo {
            ty: TcType::Table,
            mutable: false,
            annotated: true,
        });
        
        env.declare("Option".to_string(), VarInfo {
            ty: TcType::Table,
            mutable: false,
            annotated: true,
        });
        
        env.declare("File".to_string(), VarInfo {
            ty: TcType::Table,
            mutable: false,
            annotated: true,
        });
        
        env.declare("List".to_string(), VarInfo {
            ty: TcType::Table,
            mutable: false,
            annotated: true,
        });
        
        env.declare("String".to_string(), VarInfo {
            ty: TcType::Table,
            mutable: false,
            annotated: true,
        });
        
        // Prelude helpers registered as built-ins (MVP: treat as Any to allow flexible arity)
        env.declare("range".to_string(), VarInfo {
            ty: TcType::Any,
            mutable: false,
            annotated: true,
        });
        env.declare("indexed".to_string(), VarInfo {
            ty: TcType::Any,
            mutable: false,
            annotated: true,
        });
        
        env
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: String, info: VarInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn error(&mut self, message: String) {
        self.errors.push(TypeError { message });
    }

    fn expect_type(&mut self, expr: &Expr, expected: &TcType, context: &str) -> TcType {
        let ty = self.check_expr(expr);
        if !ty.is_compatible(expected) {
            self.error(format!(
                "{}: expected {:?}, got {:?}",
                context, expected, ty
            ));
        }
        ty
    }

    /// Check if a type has an operator method (e.g., __neg, __mod, __lt)
    fn has_operator_method(ty: &TcType, method_name: &str) -> bool {
        match ty {
            // Any always allowed (by design)
            TcType::Any => true,
            // Unknown types are permissively allowed (we don't have enough info to reject)
            // This matches the gradual typing philosophy and is_compatible behavior
            TcType::Unknown => true,
            TcType::Table => true,  // dynamic table may provide method at runtime
            TcType::TableWithFields(fields) => fields.iter().any(|f| f == method_name),
            _ => false,
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> TcType {
        match expr {
            Expr::Number(_) => TcType::Number,
            Expr::String(_) => TcType::String,
            Expr::Boolean(_) => TcType::Boolean,
            Expr::Null => TcType::Null,

            Expr::Identifier(name) => {
                if let Some(info) = self.lookup(name) {
                    info.ty.clone()
                } else {
                    self.error(format!("Undefined variable: {}", name));
                    TcType::Unknown
                }
            }

            Expr::List(elements) => {
                if elements.is_empty() {
                    TcType::List(Box::new(TcType::Unknown))
                } else {
                    let first_ty = self.check_expr(&elements[0]);
                    for elem in &elements[1..] {
                        let ty = self.check_expr(elem);
                        if !ty.is_compatible(&first_ty) {
                            self.error(format!(
                                "List elements have inconsistent types: {:?} vs {:?}",
                                first_ty, ty
                            ));
                        }
                    }
                    TcType::List(Box::new(first_ty))
                }
            }

            Expr::Table(entries) => {
                for (_, value) in entries {
                    self.check_expr(value);
                }
                // Collect identifier and string literal keys for structural presence
                let mut fields = Vec::new();
                for (k, _) in entries {
                    match k {
                        TableKey::Identifier(s) | TableKey::StringLiteral(s) => fields.push(s.clone()),
                        TableKey::Computed(_) => {}
                    }
                }
                // Deduplicate while preserving order
                let mut seen = std::collections::HashSet::new();
                fields.retain(|f| seen.insert(f.clone()));
                TcType::TableWithFields(fields)
            }

            Expr::Binary { left, op, right } => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);

                match op {
                    BinaryOp::Add => {
                        // Allow String + String → String OR Number + Number → Number
                        if left_ty.is_compatible(&TcType::String) && right_ty.is_compatible(&TcType::String) {
                            TcType::String
                        } else if left_ty.is_compatible(&TcType::Number) && right_ty.is_compatible(&TcType::Number) {
                            TcType::Number
                        } else {
                            self.error(format!(
                                "ADD requires (Number, Number) or (String, String), got ({:?}, {:?})",
                                left_ty, right_ty
                            ));
                            TcType::Unknown
                        }
                    }
                    BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        // Check if both operands are Numbers (default case)
                        if left_ty.is_compatible(&TcType::Number) && right_ty.is_compatible(&TcType::Number) {
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
                                TcType::Unknown  // Return type depends on implementation
                            } else {
                                self.error(format!(
                                    "Arithmetic op {:?} requires Number operands or type with {} method, got ({:?}, {:?})",
                                    op, method_name, left_ty, right_ty
                                ));
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
                        if left_ty.is_compatible(&TcType::Number) && right_ty.is_compatible(&TcType::Number) {
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
                                TcType::Boolean  // Comparison methods should return Boolean
                            } else {
                                self.error(format!(
                                    "Comparison op {:?} requires Number operands or type with {} method, got ({:?}, {:?})",
                                    op, method_name, left_ty, right_ty
                                ));
                                TcType::Boolean
                            }
                        }
                    }
                }
            }

            Expr::Unary { op, operand } => match op {
                UnaryOp::Neg => {
                    let ty = self.check_expr(operand);
                    if ty.is_compatible(&TcType::Number) {
                        TcType::Number
                    } else if Self::has_operator_method(&ty, "__neg") {
                        TcType::Unknown  // Return type depends on implementation
                    } else {
                        self.error(format!(
                            "Unary negation requires Number or type with __neg method, got {:?}",
                            ty
                        ));
                        TcType::Unknown
                    }
                }
                UnaryOp::Not => {
                    self.expect_type(operand, &TcType::Boolean, "Logical not");
                    TcType::Boolean
                }
            },

            Expr::Logical { left, op: _, right } => {
                self.expect_type(left, &TcType::Boolean, "Logical op left operand");
                self.expect_type(right, &TcType::Boolean, "Logical op right operand");
                TcType::Boolean
            }

            Expr::Call { callee, arguments } => {
                let callee_ty = self.check_expr(callee);
                match callee_ty {
                    TcType::Function { params, ret } => {
                        if arguments.len() != params.len() {
                            self.error(format!(
                                "Function call: expected {} arguments, got {}",
                                params.len(),
                                arguments.len()
                            ));
                        } else {
                            for (i, (arg, param_ty)) in arguments.iter().zip(params.iter()).enumerate() {
                                // Extract the expression from the CallArgument
                                let arg_expr = match arg {
                                    CallArgument::Positional(expr) => expr,
                                    CallArgument::Named { value, .. } => value,
                                };
                                let arg_ty = self.check_expr(arg_expr);
                                if !arg_ty.is_compatible(param_ty) {
                                    self.error(format!(
                                        "Function call: argument {} expected {:?}, got {:?}",
                                        i, param_ty, arg_ty
                                    ));
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
                        self.error(format!(
                            "Call expression requires a function, got {:?}",
                            callee_ty
                        ));
                        TcType::Unknown
                    }
                }
            }

            Expr::MemberAccess { object, member } => {
                let obj_ty = self.check_expr(object);
                match obj_ty {
                    TcType::Table => TcType::Unknown, // dynamic tables allowed
                    TcType::TableWithFields(ref fields) => {
                        if !fields.contains(member) {
                            if self.in_match_arm_depth == 0 {
                                self.error(format!("Unknown field '{}' on table", member));
                            }
                        }
                        TcType::Unknown
                    }
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(format!(
                            "Member access requires a table, got {:?}",
                            obj_ty
                        ));
                        TcType::Unknown
                    }
                }
            }

            Expr::Index { object, index } => {
                let obj_ty = self.check_expr(object);
                let idx_ty = self.check_expr(index);

                match obj_ty {
                    TcType::List(elem_ty) => {
                        if !idx_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "List index requires Number, got {:?}",
                                idx_ty
                            ));
                        }
                        (*elem_ty).clone()
                    }
                    TcType::Table | TcType::TableWithFields(_) => {
                        if !idx_ty.is_compatible(&TcType::String) {
                            self.error(format!(
                                "Table index requires String, got {:?}",
                                idx_ty
                            ));
                        }
                        TcType::Unknown
                    }
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(format!(
                            "Index operation requires List or Table, got {:?}",
                            obj_ty
                        ));
                        TcType::Unknown
                    }
                }
            }

            Expr::Function { arguments, return_type, body } => {
                self.push_scope();

                let mut param_types = Vec::new();
                for arg in arguments {
                    let param_ty = self.type_from_ast(&arg.r#type);
                    param_types.push(param_ty.clone());
                    self.declare(
                        arg.name.clone(),
                        VarInfo {
                            ty: param_ty,
                            mutable: true, // Function params are mutable in MVP
                            annotated: true,
                        },
                    );
                }

                let expected_ret = if let Some(ret_type) = return_type {
                    self.type_from_ast(ret_type)
                } else {
                    TcType::Unknown
                };

                let actual_ret = self.check_block(body, &expected_ret);

                if !actual_ret.is_compatible(&expected_ret) && expected_ret != TcType::Unknown {
                    self.error(format!(
                        "Function return type mismatch: declared {:?}, got {:?}",
                        expected_ret, actual_ret
                    ));
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

            Expr::Block(stmts) => {
                self.push_scope();
                let ret_ty = self.check_block(stmts, &TcType::Unknown);
                self.pop_scope();
                ret_ty
            }

            Expr::If { condition, then_block, else_block } => {
                // Check condition
                let cond_ty = self.check_expr(condition);
                if !cond_ty.is_compatible(&TcType::Boolean) && cond_ty != TcType::Unknown {
                    self.error(format!(
                        "If condition should be Boolean, got {:?}",
                        cond_ty
                    ));
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
                        self.error(format!(
                            "If branches have incompatible types: {:?} vs {:?}",
                            then_ty, else_ty
                        ));
                        TcType::Unknown
                    }
                } else {
                    // No else branch: could be null
                    then_ty
                }
            }

            Expr::Import { path } => {
                // Check that path is a string expression
                let path_ty = self.check_expr(path);
                if !path_ty.is_compatible(&TcType::String) && path_ty != TcType::Unknown {
                    self.error(format!(
                        "Import path should be a String, got {:?}",
                        path_ty
                    ));
                }
                // Import returns the module's exported value
                // For now, we type it as Unknown (proper typing would require module analysis)
                TcType::Unknown
            }
            Expr::Match { expr, arms } => {
                // Type of the matched expression
                let matched_ty = self.check_expr(expr);
                
                // Check exhaustiveness
                self.check_match_exhaustiveness(arms, Some(&matched_ty));
                
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
                            self.error(format!("Match arms have incompatible types: {:?} vs {:?}", current, arm_ret));
                            unified_ret = Some(TcType::Unknown);
                        }
                    } else {
                        unified_ret = Some(arm_ret);
                    }
                }
                unified_ret.unwrap_or(TcType::Null)
            }
        }
    }

    fn check_block(&mut self, stmts: &[Stmt], expected_ret: &TcType) -> TcType {
        let mut ret_ty = TcType::Null;

        // Predeclare local function variables in this block to support mutual recursion
        // and allow references within the same scope before their textual definition.
        for stmt in stmts {
            if let Stmt::VarDecl { mutable, name, r#type, value } = stmt {
                if let Expr::Function { arguments, return_type, .. } = value {
                    // Determine function type from signature
                    let mut param_types = Vec::new();
                    for arg in arguments {
                        param_types.push(self.type_from_ast(&arg.r#type));
                    }
                    let ret_ty_annot = if let Some(rt) = return_type {
                        self.type_from_ast(rt)
                    } else {
                        TcType::Unknown
                    };
                    let func_ty = TcType::Function { params: param_types, ret: Box::new(ret_ty_annot) };
                    self.declare(
                        name.clone(),
                        VarInfo { ty: func_ty, mutable: *mutable, annotated: r#type.is_some() },
                    );
                }
            }
        }

        for stmt in stmts {
            match stmt {
                Stmt::Return(expr) => {
                    ret_ty = self.check_expr(expr);
                    if !ret_ty.is_compatible(expected_ret) && *expected_ret != TcType::Unknown {
                        self.error(format!(
                            "Return type mismatch: expected {:?}, got {:?}",
                            expected_ret, ret_ty
                        ));
                    }
                }
                _ => self.check_stmt(stmt),
            }
        }

        ret_ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Match { expr, arms } => {
                // Check the match expression
                let expr_ty = self.check_expr(expr);
                
                // Check exhaustiveness
                self.check_match_exhaustiveness(arms, Some(&expr_ty));
                
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
            Stmt::VarDecl { mutable, name, r#type, value } => {
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
                            let t = self.type_from_ast(ty);
                            if !val_ty.is_compatible(&t) {
                                self.error(format!(
                                    "Variable {}: declared type {:?}, got {:?}",
                                    name, t, val_ty
                                ));
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
                if matches!(value, Expr::Function { .. }) {
                    if let Some(ty) = r#type {
                        let declared = self.type_from_ast(ty);
                        if !value_ty.is_compatible(&declared) {
                            self.error(format!(
                                "Variable {}: declared type {:?}, got {:?}",
                                name, declared, value_ty
                            ));
                        }
                    }
                }
            }

            Stmt::DestructuringVarDecl { mutable, pattern, value } => {
                let value_ty = self.check_expr(value);
                self.check_pattern(pattern, &value_ty, *mutable, false);
            }

            Stmt::Assignment { target, op: _, value } => {
                let target_ty = self.check_assignment_target(target);
                let value_ty = self.check_expr(value);

                if !value_ty.is_compatible(&target_ty) {
                    self.error(format!(
                        "Assignment type mismatch: target {:?}, value {:?}",
                        target_ty, value_ty
                    ));
                }
            }

            Stmt::If { condition, then_block, elif_blocks, else_block } => {
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

            Stmt::While { condition, body } => {
                self.expect_type(condition, &TcType::Boolean, "While condition");
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }

            Stmt::DoWhile { body, condition } => {
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                self.expect_type(condition, &TcType::Boolean, "Do-while condition");
            }

            Stmt::For { pattern, iterator, body } => {
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
                        self.error(format!(
                            "For loop requires List or Table iterator, got {:?}",
                            iter_ty
                        ));
                        self.check_pattern(pattern, &TcType::Unknown, true, false);
                    }
                }

                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }

            Stmt::Break(_) | Stmt::Continue(_) => {
                // TODO: Could check if we're inside a loop
            }

            Stmt::Return(expr) => {
                self.check_expr(expr);
            }

            Stmt::ExprStmt(expr) => {
                self.check_expr(expr);
            }
        }
    }

    fn check_assignment_target(&mut self, target: &Expr) -> TcType {
        match target {
            Expr::Identifier(name) => {
                if let Some(info) = self.lookup(name) {
                    let ty = info.ty.clone();
                    let mutable = info.mutable;
                    if !mutable {
                        self.error(format!(
                            "Cannot assign to immutable variable: {}",
                            name
                        ));
                    }
                    ty
                } else {
                    self.error(format!("Undefined variable: {}", name));
                    TcType::Unknown
                }
            }
            Expr::MemberAccess { object, member: _ } => {
                let obj_ty = self.check_expr(object);
                match obj_ty {
                    TcType::Table | TcType::TableWithFields(_) => TcType::Unknown,
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(format!(
                            "Member assignment requires a table, got {:?}",
                            obj_ty
                        ));
                        TcType::Unknown
                    }
                }
            }
            Expr::Index { object, index } => {
                let obj_ty = self.check_expr(object);
                let idx_ty = self.check_expr(index);

                match obj_ty {
                    TcType::List(elem_ty) => {
                        if !idx_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "List index requires Number, got {:?}",
                                idx_ty
                            ));
                        }
                        (*elem_ty).clone()
                    }
                    TcType::Table => {
                        if !idx_ty.is_compatible(&TcType::String) {
                            self.error(format!(
                                "Table index requires String, got {:?}",
                                idx_ty
                            ));
                        }
                        TcType::Unknown
                    }
                    TcType::Unknown | TcType::Any => TcType::Unknown,
                    _ => {
                        self.error(format!(
                            "Index assignment requires List or Table, got {:?}",
                            obj_ty
                        ));
                        TcType::Unknown
                    }
                }
            }
            _ => {
                self.error("Invalid assignment target".to_string());
                TcType::Unknown
            }
        }
    }

    fn check_pattern(&mut self, pattern: &Pattern, ty: &TcType, mutable: bool, in_match: bool) {
        match pattern {
            Pattern::Ident(name) => {
                if in_match && matches!(name.as_str(), "ok" | "err" | "some" | "none") {
                    // Tag pattern in match: don't bind a variable
                } else {
                    self.declare(
                        name.clone(),
                        VarInfo {
                            ty: ty.clone(),
                            mutable,
                            annotated: false,
                        },
                    );
                }
            }
            Pattern::ListPattern { elements, rest } => {
                match ty {
                    TcType::List(elem_ty) => {
                        for elem in elements {
                            self.check_pattern(elem, elem_ty, mutable, in_match);
                        }
                        if let Some(rest_name) = rest {
                            self.declare(
                                rest_name.clone(),
                                VarInfo {
                                    ty: TcType::List(elem_ty.clone()),
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Unknown | TcType::Any => {
                        for elem in elements {
                            self.check_pattern(elem, &TcType::Unknown, mutable, in_match);
                        }
                        if let Some(rest_name) = rest {
                            self.declare(
                                rest_name.clone(),
                                VarInfo {
                                    ty: TcType::List(Box::new(TcType::Unknown)),
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    _ => {
                        self.error(format!(
                            "List pattern requires List type, got {:?}",
                            ty
                        ));
                    }
                }
            }
            Pattern::TablePattern { fields } => {
                match ty {
                    TcType::TableWithFields(present) => {
                        // Validate required fields exist by name
                        for f in fields {
                            if !present.contains(&f.key) {
                                self.error(format!(
                                    "Table pattern requires field '{}' not present on value",
                                    f.key
                                ));
                            }
                        }
                        // Bind variables with Unknown type (no per-field typing yet)
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Table => {
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Unknown | TcType::Any => {
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    _ => {
                        self.error(format!(
                            "Table pattern requires Table type, got {:?}",
                            ty
                        ));
                    }
                }
            }
            Pattern::Wildcard => {
                // Wildcard pattern doesn't bind any variables, just accepts any type
            }
            Pattern::Literal(_) => {
                // Literal patterns don't bind variables, just match values
            }
        }
    }

    fn type_from_ast(&self, ty: &Type) -> TcType {
        match ty {
            Type::TypeIdent(name) => match name.as_str() {
                "Number" => TcType::Number,
                "String" => TcType::String,
                "Boolean" => TcType::Boolean,
                "Null" => TcType::Null,
                "Table" => TcType::Table,
                "Any" => TcType::Any,
                _ => TcType::Unknown, // Unknown type name
            },
            Type::Any => TcType::Any,
            Type::GenericType { name, type_args } => {
                match name.as_str() {
                    // Concrete generics support (MVP): List<T>
                    "List" => {
                        if let Some(first) = type_args.get(0) {
                            TcType::List(Box::new(self.type_from_ast(first)))
                        } else {
                            // List without argument defaults to Unknown element
                            TcType::List(Box::new(TcType::Unknown))
                        }
                    }
                    // Unknown generic types fall back to Unknown (until structural typing improves)
                    _ => TcType::Unknown,
                }
            }
            Type::FunctionType { param_types, return_type } => {
                let params = param_types.iter().map(|t| self.type_from_ast(t)).collect::<Vec<_>>();
                let ret = Box::new(self.type_from_ast(return_type));
                TcType::Function { params, ret }
            }
        }
    }

    /// Check if a match expression is exhaustive
    /// A match is exhaustive if:
    /// 1. It has a wildcard pattern (_), OR
    /// 2. It covers all known variants (like ok/err for Result, some/none for Option), OR
    /// 3. It covers all literal values (not practical, so we require wildcard for literals)
    fn check_match_exhaustiveness(&mut self, arms: &[(Pattern, Vec<Stmt>)], matched_ty: Option<&TcType>) {
        use std::collections::HashSet;
        
        let mut has_wildcard = false;
        let mut has_literal = false;
        let mut tags = HashSet::new();
        
        for (pattern, _) in arms {
            match pattern {
                Pattern::Wildcard => {
                    has_wildcard = true;
                }
                Pattern::Ident(name) => {
                    // Identifier pattern in match can be:
                    // 1. A catch-all binding (acts like wildcard)
                    // 2. A tag pattern for Result/Option (ok/err/some/none)
                    // We check if it's a known tag; otherwise treat as catch-all
                    if matches!(name.as_str(), "ok" | "err" | "some" | "none") {
                        tags.insert(name.as_str());
                    } else {
                        // Unknown identifier - treat as catch-all binding
                        has_wildcard = true;
                    }
                }
                Pattern::Literal(_) => {
                    has_literal = true;
                }
                Pattern::ListPattern { .. } | Pattern::TablePattern { .. } => {
                    // Structural patterns are specific, not catch-all
                }
            }
        }
        
        // If we used tag patterns and the matched type has known fields, check presence first
        if !tags.is_empty() && !has_wildcard {
            if let Some(ty) = matched_ty {
                if let TcType::TableWithFields(fields) = ty {
                    for &tag in &tags {
                        if !fields.contains(&tag.to_string()) {
                            self.error(format!(
                                "Match tag '{}' not present on matched table type",
                                tag
                            ));
                        }
                    }
                }
            }
        }

        // If we have a wildcard or catch-all identifier, we're exhaustive
        if has_wildcard {
            return;
        }
        
        // Check if we have all known tag variants
        let has_result_tags = tags.contains("ok") && tags.contains("err");
        let has_option_tags = tags.contains("some") && tags.contains("none");
        
        if has_result_tags || has_option_tags {
            // Exhaustive for Result or Option types
            return;
        }

        // If we have literal patterns without wildcard, not exhaustive
        if has_literal {
            self.error("Match expression is not exhaustive: literal patterns require a wildcard (_) or catch-all case".to_string());
            return;
        }
        
        // If we have tags but not all variants, not exhaustive
        if !tags.is_empty() {
            self.error(format!(
                "Match expression is not exhaustive: found tags {:?} but missing wildcard or all variants (e.g., ok/err or some/none)",
                tags
            ));
            return;
        }
        
        // Otherwise, we need a wildcard
        self.error("Match expression is not exhaustive: add a wildcard (_) pattern or cover all cases".to_string());
    }
}

pub fn typecheck_program(program: &Program) -> TypecheckResult<()> {
    let mut env = TypeEnv::new();

    // First pass: Pre-declare all top-level let/var with function values
    // This enables mutual recursion between functions
    for stmt in &program.statements {
        if let Stmt::VarDecl { mutable, name, r#type, value } = stmt {
            if let Expr::Function { arguments, return_type, .. } = value {
                // Compute function type from signature
                let mut param_types = Vec::new();
                for arg in arguments {
                    param_types.push(env.type_from_ast(&arg.r#type));
                }
                let ret_ty = if let Some(rt) = return_type {
                    env.type_from_ast(rt)
                } else {
                    TcType::Unknown
                };
                
                let func_ty = TcType::Function {
                    params: param_types,
                    ret: Box::new(ret_ty),
                };
                
                // Pre-declare the function variable
                env.declare(
                    name.clone(),
                    VarInfo {
                        ty: func_ty,
                        mutable: *mutable,
                        annotated: r#type.is_some(),
                    },
                );
            }
        }
    }

    // Second pass: Check all statements (function bodies can now reference each other)
    for stmt in &program.statements {
        env.check_stmt(stmt);
    }

    if env.errors.is_empty() {
        Ok(())
    } else {
        Err(env.errors)
    }
}
