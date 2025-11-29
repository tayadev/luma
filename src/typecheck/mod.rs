//! Static type checker for Luma.
//!
//! This module implements a gradual type system that provides static type checking
//! while allowing dynamic typing where needed.
//!
//! ## Type System Features
//!
//! - **Gradual typing**: Mix of static and dynamic types with `Any` and `Unknown`
//! - **Type inference**: Infers types from expressions and declarations
//! - **Pattern matching**: Type-safe destructuring with exhaustiveness checking
//! - **Operator overloading**: Validates operator methods on custom types
//! - **Function types**: First-class function types with parameter and return type checking
//!
//! ## Type Checking Process
//!
//! 1. **Pre-declaration**: Top-level functions are registered to enable mutual recursion
//! 2. **Type checking**: Traverse statements and expressions, building type environment
//! 3. **Pattern validation**: Check patterns for exhaustiveness and type compatibility
//! 4. **Error collection**: Accumulate all type errors for batch reporting
//!
//! The type checker is designed to be permissive - it allows `Any` and `Unknown` types
//! where exact types cannot be determined, falling back to runtime checking.

mod environment;
mod errors;
mod expressions;
mod patterns;
mod statements;
mod types;

use crate::ast::{Expr, Program, Stmt};

pub use errors::{TypeError, TypecheckResult};
pub use types::TcType;

use environment::TypeEnv;
use types::VarInfo;

/// Type check a program and return any errors found.
pub fn typecheck_program(program: &Program) -> TypecheckResult<()> {
    let mut env = TypeEnv::new();

    // First pass: Pre-declare all top-level let/var with function values
    // This enables mutual recursion between functions
    for stmt in &program.statements {
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
            // Compute function type from signature
            let mut param_types = Vec::new();
            for arg in arguments {
                param_types.push(TypeEnv::type_from_ast(&arg.r#type));
            }
            let ret_ty = if let Some(rt) = return_type {
                TypeEnv::type_from_ast(rt)
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
