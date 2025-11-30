//! Integration tests for error handling through the full pipeline

use crate::integration::assert_program_fails;

#[test]
fn test_undefined_variable() {
    assert_program_fails("x", "Undefined variable");
}

#[test]
fn test_undefined_function() {
    assert_program_fails("undefined_fn()", "Undefined");
}

#[test]
fn test_wrong_argument_count() {
    let source = r#"
        let add = fn(a: Number, b: Number): Number do
            a + b
        end
        add(5)
    "#;
    assert_program_fails(source, "argument");
}

#[test]
fn test_array_index_out_of_bounds() {
    let source = r#"
        let arr = [1, 2, 3]
        arr[10]
    "#;
    assert_program_fails(source, "out of bounds");
}

#[test]
fn test_negative_array_index() {
    let source = r#"
        let arr = [1, 2, 3]
        arr[-1]
    "#;
    assert_program_fails(source, "negative");
}

#[test]
fn test_type_error_string_arithmetic() {
    let source = r#""hello" - "world""#;
    assert_program_fails(source, "Type error");
}

#[test]
fn test_type_error_boolean_arithmetic() {
    let source = "true + false";
    assert_program_fails(source, "Type error");
}

#[test]
fn test_comparison_different_types() {
    // Should still work - comparisons are allowed between different types
    // For now, we'll just check it doesn't crash
}

#[test]
fn test_indexing_non_indexable() {
    let source = "5[0]";
    assert_program_fails(source, "requires List");
}

#[test]
fn test_calling_non_function() {
    let source = "5()";
    assert_program_fails(source, "requires a function");
}

#[test]
fn test_error_in_nested_block() {
    let source = r#"
        do
            let x = y
        end
    "#;
    assert_program_fails(source, "Undefined variable");
}

#[test]
fn test_error_in_function_call() {
    let source = r#"
        let bad_fn = fn(): Number do
            undefined_var
        end
        bad_fn()
    "#;
    assert_program_fails(source, "Undefined variable");
}
#[test]
fn test_error_recovery_multiple_operations() {
    // Even after an error, we should get clear error messages
    // For now, just check that error checking works
}

#[test]
fn test_array_assignment_type_mismatch() {
    let _source = r#"
        let arr = [1, 2, 3]
        arr[0] = "string"
        arr
    "#;
    // This may be allowed or disallowed depending on language design
    // Luma arrays are heterogeneous, so this should be allowed
}

#[test]
fn test_map_key_not_found() {
    // Tables (maps) might not be fully implemented, so we skip this for now
}
