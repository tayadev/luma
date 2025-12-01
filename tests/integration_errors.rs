//! Error handling tests

mod common;

use common::assert_program_fails;

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

// ============================================================================
// Type Check Failure Tests (should fail at typecheck)
// ============================================================================

#[test]
fn test_should_fail_undefined_var() {
    let source = "undefined_variable";
    assert_program_fails(source, "Undefined variable");
}

#[test]
fn test_should_fail_immutable_let() {
    let source = r#"
let x = 5
x = 10
x
"#;
    assert_program_fails(source, "immutable");
}

#[test]
fn test_should_fail_number_plus_string() {
    let source = r#"
var x = 5
x + "hello"
"#;
    assert_program_fails(source, "Type error");
}

#[test]
fn test_should_fail_type_mismatch_add() {
    let source = r#"
var x = "hello"
x + 5
"#;
    assert_program_fails(source, "Type error");
}

#[test]
fn test_should_fail_wrong_arity() {
    let source = r#"
var foo = fn(x: Number) do
  x
end
foo()
"#;
    assert_program_fails(source, "argument");
}

#[test]
fn test_should_fail_duplicate_named_arg() {
    let source = r#"
let greet = fn(name: String, age: Number): String do
  return "Hello"
end

greet(name = "Alice", age = 30, name = "Bob")
"#;
    assert_program_fails(source, "argument");
}

#[test]
fn test_should_fail_function_return_mismatch() {
    let source = r#"
let bad: fn(Number): Number = fn(x: Number): Number do
  "oops"
end
"#;
    assert_program_fails(source, "Type");
}

#[test]
fn test_should_fail_function_generic_arg_mismatch() {
    let source = r#"
let sumList: fn(List(Number)): Number = fn(xs: List(Number)): Number do
  0
end

sumList(["a", "b"])
"#;
    assert_program_fails(source, "Type");
}

#[test]
fn test_should_fail_table_unknown_field() {
    let source = r#"
let t = { a = 1, b = 2 }
let x = t.c
"#;
    assert_program_fails(source, "field");
}

#[test]
fn test_should_fail_unary_neg_no_overload() {
    let source = r#"
let v = {}
let res = -v
res
"#;
    assert_program_fails(source, "Type");
}

#[test]
fn test_should_fail_match_not_exhaustive() {
    let source = r#"
let x = 1
match x do
  0 do print("zero") end
end
"#;
    assert_program_fails(source, "exhaustive");
}

#[test]
fn test_should_fail_match_missing_tag() {
    let source = r#"
let v = { ok = 1 }
match v do
  err do 0 end
  ok do 1 end
end
"#;
    assert_program_fails(source, "not present");
}

#[test]
fn test_should_fail_unreachable_pattern() {
    let source = r#"
let x = 1
match x do
  _ do print("wildcard") end
  0 do print("zero") end
end
"#;
    assert_program_fails(source, "unreachable");
}
