//! Collection (arrays/lists) tests

mod common;

use common::assert_program_output;
use luma::vm::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_empty_array() {
    let source = "[]";
    let expected = Value::List(Rc::new(RefCell::new(vec![])));
    assert_program_output(source, expected);
}

#[test]
fn test_array_with_numbers() {
    let source = "[1, 2, 3]";
    let expected = Value::List(Rc::new(RefCell::new(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ])));
    assert_program_output(source, expected);
}

#[test]
fn test_array_indexing() {
    let source = r#"
        let arr = [10, 20, 30]
        arr[0]
    "#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_array_indexing_middle() {
    let source = r#"
        let arr = [10, 20, 30]
        arr[1]
    "#;
    assert_program_output(source, Value::Number(20.0));
}

#[test]
fn test_array_indexing_last() {
    let source = r#"
        let arr = [10, 20, 30]
        arr[2]
    "#;
    assert_program_output(source, Value::Number(30.0));
}

#[test]
fn test_array_indexing_with_variable() {
    let source = r#"
        let arr = [10, 20, 30, 40]
        let i = 2
        arr[i]
    "#;
    assert_program_output(source, Value::Number(30.0));
}

#[test]
fn test_array_in_variable() {
    let source = r#"
        var arr = [1, 2, 3]
        arr[0] = 10
        arr[0]
    "#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_nested_arrays() {
    let source = r#"
        let arr = [[1, 2], [3, 4]]
        arr[0][1]
    "#;
    assert_program_output(source, Value::Number(2.0));
}

#[test]
fn test_array_iteration() {
    let source = r#"
        var sum = 0
        let arr = [1, 2, 3, 4, 5]
        for x in arr do
            sum = sum + x
        end
        sum
    "#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_array_read() {
    let source = r#"
[1, 2, 3][0]
"#;
    assert_program_output(source, Value::Number(1.0));
}

#[test]
fn test_array_write() {
    let source = r#"
var arr = [1, 2, 3]
arr[1] = 10
arr[1]
"#;
    assert_program_output(source, Value::Number(10.0));
}
