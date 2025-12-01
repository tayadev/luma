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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn parse_and_typecheck(input: &str) -> TypecheckResult<()> {
        let program = parse(input, "test.luma").expect("Parse failed");
        typecheck_program(&program)
    }

    // Basic type inference tests
    #[test]
    fn test_simple_number_declaration() {
        let result = parse_and_typecheck("let x = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_string_declaration() {
        let result = parse_and_typecheck("let s = \"hello\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_boolean_declaration() {
        let result = parse_and_typecheck("let b = true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_null_declaration() {
        let result = parse_and_typecheck("let n = null");
        assert!(result.is_ok());
    }

    // Type annotation tests
    #[test]
    fn test_correct_type_annotation() {
        let result = parse_and_typecheck("let x: Number = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_incorrect_type_annotation() {
        let result = parse_and_typecheck("let x: String = 42");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .message
                .contains("declared type String, got Number")
        );
    }

    #[test]
    fn test_list_type_annotation() {
        let result = parse_and_typecheck("let nums = [1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_heterogeneous_list_error() {
        let result = parse_and_typecheck("let mixed = [1, \"two\", 3]");
        assert!(result.is_err());
    }

    // Undefined variable tests
    #[test]
    fn test_undefined_variable() {
        let result = parse_and_typecheck("let x = y");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Undefined variable: y"));
    }

    #[test]
    fn test_use_before_define() {
        let result = parse_and_typecheck("let x = y\nlet y = 42");
        assert!(result.is_err());
    }

    // Function type checking tests
    #[test]
    fn test_simple_function_declaration() {
        let result = parse_and_typecheck("let f = fn(x: Number): Number do return x + 1 end");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_return_type_mismatch() {
        let result = parse_and_typecheck("let f = fn(x: Number): String do return x + 1 end");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Return type mismatch"));
    }

    #[test]
    fn test_function_call_arg_count() {
        let result =
            parse_and_typecheck("let f = fn(x: Number): Number do return x end\nlet y = f(1, 2)");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("expected 1 arguments, got 2"));
    }

    #[test]
    fn test_function_call_arg_type() {
        let result =
            parse_and_typecheck("let f = fn(x: Number): Number do return x end\nlet y = f(\"hi\")");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("expected Number, got String"));
    }

    #[test]
    fn test_mutual_recursion() {
        let code = r#"
            let even = fn(n: Number): Boolean do
                if n == 0 do
                    return true
                else do
                    return odd(n - 1)
                end
            end
            
            let odd = fn(n: Number): Boolean do
                if n == 0 do
                    return false
                else do
                    return even(n - 1)
                end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    // Mutability tests
    #[test]
    fn test_immutable_assignment_error() {
        let result = parse_and_typecheck("let x = 42\nx = 43");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors[0]
                .message
                .contains("Cannot assign to immutable variable")
        );
    }

    #[test]
    fn test_mutable_assignment() {
        let result = parse_and_typecheck("var x = 42\nx = 43");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutable_assignment_type_mismatch() {
        let result = parse_and_typecheck("var x = 42\nx = \"hello\"");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Assignment type mismatch"));
    }

    // Arithmetic operator tests
    #[test]
    fn test_number_arithmetic() {
        let result = parse_and_typecheck("let x = 1 + 2 * 3 - 4 / 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_concatenation() {
        let result = parse_and_typecheck("let s = \"hello\" + \" world\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_string_arithmetic() {
        let result = parse_and_typecheck("let x = \"hello\" * 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mixed_arithmetic() {
        let result = parse_and_typecheck("let x = 42 + \"hello\"");
        assert!(result.is_err());
    }

    // Comparison operator tests
    #[test]
    fn test_number_comparison() {
        let result = parse_and_typecheck("let b = 1 < 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_equality_comparison() {
        let result = parse_and_typecheck("let b = 42 == 42");
        assert!(result.is_ok());
    }

    // Logical operator tests
    #[test]
    fn test_logical_operators() {
        let result = parse_and_typecheck("let b = true && false || true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_operator_type_error() {
        let result = parse_and_typecheck("let b = 42 && true");
        assert!(result.is_err());
    }

    // Unary operator tests
    #[test]
    fn test_unary_negation() {
        let result = parse_and_typecheck("let x = -42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_not() {
        let result = parse_and_typecheck("let b = !true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_unary_negation() {
        let result = parse_and_typecheck("let x = -\"hello\"");
        assert!(result.is_err());
    }

    // Collection tests
    #[test]
    fn test_list_indexing() {
        let result = parse_and_typecheck("let x = [1, 2, 3][0]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_index_type_error() {
        let result = parse_and_typecheck("let x = [1, 2, 3][\"hello\"]");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("List index requires Number"));
    }

    #[test]
    fn test_table_creation() {
        let result = parse_and_typecheck("let t = { x = 1, y = 2 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_member_access() {
        let result = parse_and_typecheck("let t = { x = 1 }\nlet v = t.x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_unknown_field() {
        let result = parse_and_typecheck("let t = { x = 1 }\nlet v = t.y");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Unknown field 'y'"));
    }

    #[test]
    fn test_table_indexing() {
        let result = parse_and_typecheck("let t = { x = 1 }\nlet v = t[\"x\"]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_index_type_error() {
        let result = parse_and_typecheck("let t = { x = 1 }\nlet v = t[42]");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Table index requires String"));
    }

    // Control flow tests
    #[test]
    fn test_if_statement() {
        let result = parse_and_typecheck("if true do let x = 42 end");
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_condition_type_error() {
        let result = parse_and_typecheck("if 42 do let x = 1 end");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("expected Boolean"));
    }

    #[test]
    fn test_if_else_statement() {
        let result = parse_and_typecheck("if true do let x = 1 else do let y = 2 end");
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_loop() {
        let result = parse_and_typecheck("while true do let x = 1 end");
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_condition_type_error() {
        let result = parse_and_typecheck("while 42 do let x = 1 end");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("expected Boolean"));
    }

    #[test]
    fn test_for_loop_list() {
        let result = parse_and_typecheck("for x in [1, 2, 3] do let y = x end");
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_invalid_iterator() {
        let result = parse_and_typecheck("for x in 42 do let y = x end");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors[0]
                .message
                .contains("For loop requires List or Table iterator")
        );
    }

    // Scoping tests
    #[test]
    fn test_block_scoping() {
        let result = parse_and_typecheck("do let x = 42 end\nlet y = x");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("Undefined variable: x"));
    }

    #[test]
    fn test_nested_scopes() {
        let result = parse_and_typecheck("let x = 1\ndo let x = 2\nlet y = x end");
        assert!(result.is_ok());
    }

    // Pattern matching tests (if supported)
    #[test]
    fn test_simple_destructuring() {
        let result = parse_and_typecheck("let [x, y] = [1, 2]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_destructuring() {
        let result = parse_and_typecheck("let { x, y } = { x = 1, y = 2 }");
        assert!(result.is_ok());
    }

    // Match statement tests
    #[test]
    fn test_match_with_literals() {
        let code = r#"
            match 5 do
                0 do let x = "zero" end
                1 do let x = "one" end
                _ do let x = "other" end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_with_list_patterns() {
        let code = r#"
            match [1, 2, 3] do
                [first] do let y = first end
                [first, second] do let z = first + second end
                [first, second, third] do let w = first + second + third end
                _ do let x = 0 end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_with_table_patterns() {
        let code = r#"
            match { x = 1, y = 2 } do
                { x } do let a = x end
                _ do let b = 0 end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_non_exhaustive() {
        let code = r#"
            match 5 do
                0 do let x = "zero" end
                1 do let x = "one" end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].message.contains("not exhaustive"));
    }

    #[test]
    fn test_match_unreachable_pattern() {
        let code = r#"
            match 5 do
                _ do let x = "catch-all" end
                1 do let y = "unreachable" end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors[0].message.contains("unreachable") || errors[0].message.contains("Unreachable")
        );
    }

    #[test]
    fn test_match_with_rest_pattern() {
        let code = r#"
            match [1, 2, 3, 4] do
                [first, ...rest] do let x = first end
                _ do let y = 0 end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let result = parse_and_typecheck("let [first, ...rest] = [1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_return_type_consistency() {
        let code = r#"
            let f = fn(x: Number): Number do
                match x do
                    0 do return 0 end
                    1 do return 1 end
                    _ do return x * 2 end
                end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_match() {
        let code = r#"
            match 42 do
                0 do 
                    match 10 do
                        5 do let x = 5 end
                        _ do let x = 10 end
                    end
                end
                _ do let sum = 0 end
            end
        "#;
        let result = parse_and_typecheck(code);
        assert!(result.is_ok());
    }
}
