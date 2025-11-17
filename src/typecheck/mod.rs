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
    Array(Box<TcType>),
    Table,
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
            (TcType::Array(a), TcType::Array(b)) => a.is_compatible(b),
            (TcType::Table, TcType::Table) => true,
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
}

impl TypeEnv {
    fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
        }
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

            Expr::Array(elements) => {
                if elements.is_empty() {
                    TcType::Array(Box::new(TcType::Unknown))
                } else {
                    let first_ty = self.check_expr(&elements[0]);
                    for elem in &elements[1..] {
                        let ty = self.check_expr(elem);
                        if !ty.is_compatible(&first_ty) {
                            self.error(format!(
                                "Array elements have inconsistent types: {:?} vs {:?}",
                                first_ty, ty
                            ));
                        }
                    }
                    TcType::Array(Box::new(first_ty))
                }
            }

            Expr::Table(entries) => {
                for (_, value) in entries {
                    self.check_expr(value);
                }
                TcType::Table
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
                        if !left_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Arithmetic op {:?} requires Number on left, got {:?}",
                                op, left_ty
                            ));
                        }
                        if !right_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Arithmetic op {:?} requires Number on right, got {:?}",
                                op, right_ty
                            ));
                        }
                        TcType::Number
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        // Allow any types for equality comparison
                        TcType::Boolean
                    }
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        if !left_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Comparison op {:?} requires Number on left, got {:?}",
                                op, left_ty
                            ));
                        }
                        if !right_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Comparison op {:?} requires Number on right, got {:?}",
                                op, right_ty
                            ));
                        }
                        TcType::Boolean
                    }
                }
            }

            Expr::Unary { op, operand } => match op {
                UnaryOp::Neg => {
                    self.expect_type(operand, &TcType::Number, "Unary negation");
                    TcType::Number
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

            Expr::MemberAccess { object, member: _ } => {
                let obj_ty = self.check_expr(object);
                match obj_ty {
                    TcType::Table => TcType::Unknown, // Can't know member types in MVP
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
                    TcType::Array(elem_ty) => {
                        if !idx_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Array index requires Number, got {:?}",
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
                            "Index operation requires Array or Table, got {:?}",
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

                TcType::Function {
                    params: param_types,
                    ret: Box::new(actual_ret),
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
        }
    }

    fn check_block(&mut self, stmts: &[Stmt], expected_ret: &TcType) -> TcType {
        let mut ret_ty = TcType::Null;

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
                
                // For each arm, check the pattern and body
                for (pattern, body) in arms {
                    self.push_scope();
                    // Bind pattern variables with the matched expression's type
                    self.check_pattern(pattern, &expr_ty, false); // match bindings are immutable
                    
                    // Check the body statements
                    for stmt in body {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }
            }
            Stmt::VarDecl { mutable, name, r#type, value } => {
                // For function values, we need to declare the variable first to support recursion
                let value_ty = match value {
                    Expr::Function { arguments, return_type, body: _ } => {
                        // Pre-declare the function variable to support recursion
                        let mut param_types = Vec::new();
                        for arg in arguments {
                            param_types.push(self.type_from_ast(&arg.r#type));
                        }
                        let ret_ty = if let Some(rt) = return_type {
                            self.type_from_ast(rt)
                        } else {
                            TcType::Unknown
                        };
                        
                        let func_ty = TcType::Function {
                            params: param_types,
                            ret: Box::new(ret_ty),
                        };
                        
                        // Declare the function variable before checking body
                        self.declare(
                            name.clone(),
                            VarInfo {
                                ty: func_ty.clone(),
                                mutable: *mutable,
                                annotated: r#type.is_some(),
                            },
                        );
                        
                        // Now check the function expression (which may reference itself)
                        self.check_expr(value)
                    }
                    _ => self.check_expr(value),
                };
                
                // If not a function, declare normally
                if !matches!(value, Expr::Function { .. }) {
                    let declared_ty = if let Some(ty) = r#type {
                        let t = self.type_from_ast(ty);
                        if !value_ty.is_compatible(&t) {
                            self.error(format!(
                                "Variable {}: declared type {:?}, got {:?}",
                                name, t, value_ty
                            ));
                        }
                        t
                    } else {
                        value_ty
                    };

                    self.declare(
                        name.clone(),
                        VarInfo {
                            ty: declared_ty,
                            mutable: *mutable,
                            annotated: r#type.is_some(),
                        },
                    );
                }
            }

            Stmt::DestructuringVarDecl { mutable, pattern, value } => {
                let value_ty = self.check_expr(value);
                self.check_pattern(pattern, &value_ty, *mutable);
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
                    TcType::Array(elem_ty) => {
                        self.check_pattern(pattern, elem_ty, true);
                    }
                    TcType::Unknown | TcType::Any => {
                        self.check_pattern(pattern, &TcType::Unknown, true);
                    }
                    _ => {
                        self.error(format!(
                            "For loop requires Array iterator, got {:?}",
                            iter_ty
                        ));
                        self.check_pattern(pattern, &TcType::Unknown, true);
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
                    TcType::Table => TcType::Unknown,
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
                    TcType::Array(elem_ty) => {
                        if !idx_ty.is_compatible(&TcType::Number) {
                            self.error(format!(
                                "Array index requires Number, got {:?}",
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
                            "Index assignment requires Array or Table, got {:?}",
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

    fn check_pattern(&mut self, pattern: &Pattern, ty: &TcType, mutable: bool) {
        match pattern {
            Pattern::Ident(name) => {
                self.declare(
                    name.clone(),
                    VarInfo {
                        ty: ty.clone(),
                        mutable,
                        annotated: false,
                    },
                );
            }
            Pattern::ArrayPattern { elements, rest } => {
                match ty {
                    TcType::Array(elem_ty) => {
                        for elem in elements {
                            self.check_pattern(elem, elem_ty, mutable);
                        }
                        if let Some(rest_name) = rest {
                            self.declare(
                                rest_name.clone(),
                                VarInfo {
                                    ty: TcType::Array(elem_ty.clone()),
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Unknown | TcType::Any => {
                        for elem in elements {
                            self.check_pattern(elem, &TcType::Unknown, mutable);
                        }
                        if let Some(rest_name) = rest {
                            self.declare(
                                rest_name.clone(),
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
                            "Array pattern requires Array type, got {:?}",
                            ty
                        ));
                    }
                }
            }
            Pattern::TablePattern(keys) => {
                match ty {
                    TcType::Table => {
                        for key in keys {
                            self.declare(
                                key.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Unknown | TcType::Any => {
                        for key in keys {
                            self.declare(
                                key.clone(),
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
            // For now, generic and function types are treated as Unknown
            // Full type system implementation will handle these properly
            Type::GenericType { .. } => TcType::Unknown,
            Type::FunctionType { .. } => TcType::Unknown,
        }
    }
}

pub fn typecheck_program(program: &Program) -> TypecheckResult<()> {
    let mut env = TypeEnv::new();

    for stmt in &program.statements {
        env.check_stmt(stmt);
    }

    if env.errors.is_empty() {
        Ok(())
    } else {
        Err(env.errors)
    }
}
