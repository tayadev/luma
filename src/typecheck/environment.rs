//! Type environment for scope and variable management.

use std::collections::HashMap;

use crate::ast::{Expr, Span, Type};

use super::errors::TypeError;
use super::types::{TcType, VarInfo};

/// Type environment that tracks variable scopes and accumulates errors.
pub struct TypeEnv {
    pub scopes: Vec<HashMap<String, VarInfo>>,
    pub errors: Vec<TypeError>,
    /// Track match arm context to relax certain checks inside arms.
    pub in_match_arm_depth: usize,
}

impl TypeEnv {
    /// Create a new type environment with built-in functions registered.
    pub fn new() -> Self {
        let mut env = TypeEnv {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
            in_match_arm_depth: 0,
        };

        // Register built-in functions
        env.declare(
            "cast".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Any, TcType::Any],
                    ret: Box::new(TcType::Any),
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "isInstanceOf".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Any, TcType::Any],
                    ret: Box::new(TcType::Boolean),
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "into".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Any, TcType::Any],
                    ret: Box::new(TcType::Any),
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "typeof".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Any],
                    ret: Box::new(TcType::String),
                },
                mutable: false,
                annotated: true,
            },
        );

        // print is variadic - we use Any to accept any number of arguments
        // The actual arity check is skipped for print in the VM
        env.declare(
            "print".to_string(),
            VarInfo {
                ty: TcType::Any, // Variadic function - any type
                mutable: false,
                annotated: true,
            },
        );

        // Register I/O native functions
        env.declare(
            "write".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Number, TcType::Any],
                    ret: Box::new(TcType::Table), // Returns Result
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "read_file".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::String],
                    ret: Box::new(TcType::Table), // Returns Result
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "write_file".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::String, TcType::Any],
                    ret: Box::new(TcType::Table), // Returns Result
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "file_exists".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::String],
                    ret: Box::new(TcType::Boolean),
                },
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "panic".to_string(),
            VarInfo {
                ty: TcType::Function {
                    params: vec![TcType::Any],
                    ret: Box::new(TcType::Any), // Never returns, but use Any
                },
                mutable: false,
                annotated: true,
            },
        );

        // Register file descriptor constants
        env.declare(
            "STDOUT".to_string(),
            VarInfo {
                ty: TcType::Number,
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "STDERR".to_string(),
            VarInfo {
                ty: TcType::Number,
                mutable: false,
                annotated: true,
            },
        );

        // Register prelude types/tables (from prelude.luma)
        // These are tables containing methods/constructors
        env.declare(
            "Result".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "Option".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "File".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "List".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        env.declare(
            "String".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        // Prelude helpers registered as built-ins (MVP: treat as Any to allow flexible arity)
        env.declare(
            "range".to_string(),
            VarInfo {
                ty: TcType::Any,
                mutable: false,
                annotated: true,
            },
        );
        env.declare(
            "indexed".to_string(),
            VarInfo {
                ty: TcType::Any,
                mutable: false,
                annotated: true,
            },
        );

        // Register FFI module
        env.declare(
            "ffi".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        // Register process module
        env.declare(
            "process".to_string(),
            VarInfo {
                ty: TcType::Table,
                mutable: false,
                annotated: true,
            },
        );

        // Register External type marker
        env.declare(
            "External".to_string(),
            VarInfo {
                ty: TcType::Any, // External values can be any type
                mutable: false,
                annotated: true,
            },
        );

        env
    }

    /// Push a new scope onto the scope stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope from the scope stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declare a variable in the current scope.
    pub fn declare(&mut self, name: String, info: VarInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    /// Look up a variable by name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Record a type error.
    pub fn error(&mut self, message: String, span: Option<Span>) {
        self.errors.push(TypeError { message, span });
    }

    /// Check if an expression has the expected type, reporting an error if not.
    pub fn expect_type(&mut self, expr: &Expr, expected: &TcType, context: &str) -> TcType {
        let ty = self.check_expr(expr);
        if !ty.is_compatible(expected) {
            self.error(
                format!("{context}: expected {expected}, got {ty}"),
                expr.span(),
            );
        }
        ty
    }

    /// Check if a type has an operator method (e.g., __neg, __mod, __lt).
    pub fn has_operator_method(ty: &TcType, method_name: &str) -> bool {
        match ty {
            // Any always allowed (by design)
            TcType::Any => true,
            // Unknown types are permissively allowed (we don't have enough info to reject)
            // This matches the gradual typing philosophy and is_compatible behavior
            TcType::Unknown => true,
            TcType::Table => true, // dynamic table may provide method at runtime
            TcType::TableWithFields(fields) => fields.contains(&method_name.to_string()),
            _ => false,
        }
    }

    /// Convert an AST type to a TcType.
    pub fn type_from_ast(ty: &Type) -> TcType {
        match ty {
            Type::TypeIdent { name, .. } => match name.as_str() {
                "Number" => TcType::Number,
                "String" => TcType::String,
                "Boolean" => TcType::Boolean,
                "Null" => TcType::Null,
                "Table" => TcType::Table,
                "Any" => TcType::Any,
                _ => TcType::Unknown, // Unknown type name
            },
            Type::Any { .. } => TcType::Any,
            Type::GenericType {
                name, type_args, ..
            } => {
                match name.as_str() {
                    // Concrete generics support (MVP): List<T>
                    "List" => {
                        if let Some(first) = type_args.first() {
                            TcType::List(Box::new(Self::type_from_ast(first)))
                        } else {
                            // List without argument defaults to Unknown element
                            TcType::List(Box::new(TcType::Unknown))
                        }
                    }
                    // Unknown generic types fall back to Unknown (until structural typing improves)
                    _ => TcType::Unknown,
                }
            }
            Type::FunctionType {
                param_types,
                return_type,
                ..
            } => {
                let params = param_types
                    .iter()
                    .map(Self::type_from_ast)
                    .collect::<Vec<_>>();
                let ret = Box::new(Self::type_from_ast(return_type));
                TcType::Function { params, ret }
            }
        }
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}
