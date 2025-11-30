//! Integration tests for expressions through the full pipeline

use crate::integration::assert_program_output;
use luma::vm::value::Value;

#[test]
fn test_simple_number_literal() {
    assert_program_output("42", Value::Number(42.0));
}

#[test]
fn test_simple_string_literal() {
    assert_program_output(r#""hello""#, Value::String("hello".to_string()));
}

#[test]
fn test_boolean_true() {
    assert_program_output("true", Value::Boolean(true));
}

#[test]
fn test_boolean_false() {
    assert_program_output("false", Value::Boolean(false));
}

#[test]
fn test_null_literal() {
    assert_program_output("null", Value::Null);
}

#[test]
fn test_arithmetic_addition() {
    assert_program_output("5 + 3", Value::Number(8.0));
}

#[test]
fn test_arithmetic_subtraction() {
    assert_program_output("5 - 3", Value::Number(2.0));
}

#[test]
fn test_arithmetic_multiplication() {
    assert_program_output("5 * 3", Value::Number(15.0));
}

#[test]
fn test_arithmetic_division() {
    assert_program_output("6 / 2", Value::Number(3.0));
}

#[test]
fn test_arithmetic_modulo() {
    assert_program_output("7 % 3", Value::Number(1.0));
}

#[test]
fn test_comparison_equal() {
    assert_program_output("5 == 5", Value::Boolean(true));
}

#[test]
fn test_comparison_not_equal() {
    assert_program_output("5 != 3", Value::Boolean(true));
}

#[test]
fn test_comparison_less_than() {
    assert_program_output("3 < 5", Value::Boolean(true));
}

#[test]
fn test_comparison_less_equal() {
    assert_program_output("5 <= 5", Value::Boolean(true));
}

#[test]
fn test_comparison_greater_than() {
    assert_program_output("5 > 3", Value::Boolean(true));
}

#[test]
fn test_comparison_greater_equal() {
    assert_program_output("5 >= 5", Value::Boolean(true));
}

#[test]
fn test_string_concatenation() {
    let source = r#""hello" + " " + "world""#;
    assert_program_output(source, Value::String("hello world".to_string()));
}

#[test]
fn test_unary_negation() {
    assert_program_output("-5", Value::Number(-5.0));
}

#[test]
fn test_unary_not() {
    assert_program_output("!true", Value::Boolean(false));
}

#[test]
fn test_logical_and_true() {
    assert_program_output("true && true", Value::Boolean(true));
}

#[test]
fn test_logical_and_false() {
    assert_program_output("true && false", Value::Boolean(false));
}

#[test]
fn test_logical_or_true() {
    assert_program_output("true || false", Value::Boolean(true));
}

#[test]
fn test_logical_or_false() {
    assert_program_output("false || false", Value::Boolean(false));
}

#[test]
fn test_operator_precedence() {
    assert_program_output("2 + 3 * 4", Value::Number(14.0));
}

#[test]
fn test_operator_precedence_with_parens() {
    assert_program_output("(2 + 3) * 4", Value::Number(20.0));
}

#[test]
fn test_variable_assignment_and_use() {
    let source = r#"
        let x = 42
        x
    "#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_mutable_variable_reassignment() {
    let source = r#"
        var x = 10
        x = 20
        x
    "#;
    assert_program_output(source, Value::Number(20.0));
}

#[test]
fn test_multiple_variables() {
    let source = r#"
        let x = 5
        let y = 10
        x + y
    "#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_shadowing_variables() {
    let source = r#"
        let x = 5
        let x = 10
        x
    "#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_nested_blocks() {
    let source = r#"
        let x = 5
        do
            let y = 10
            x + y
        end
    "#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_expression_in_block() {
    let source = r#"
        do
            1 + 1
        end
    "#;
    assert_program_output(source, Value::Number(2.0));
}
