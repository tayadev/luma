//! Integration tests for control flow through the full pipeline

use crate::integration::assert_program_output;
use luma::vm::value::Value;

#[test]
fn test_if_true_branch() {
    let source = r#"
        if true do
            42
        else do
            0
        end
    "#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_if_false_branch() {
    let source = r#"
        if false do
            0
        else do
            42
        end
    "#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_if_condition_evaluated() {
    let source = r#"
        let x = 10
        if x > 5 do
            "greater"
        else do
            "less"
        end
    "#;
    assert_program_output(source, Value::String("greater".to_string()));
}

#[test]
fn test_if_elif_else() {
    let source = r#"
        let x = 10
        if x < 5 do
            "small"
        else if x < 15 do
            "medium"
        else do
            "large"
        end
    "#;
    assert_program_output(source, Value::String("medium".to_string()));
}

#[test]
fn test_multiple_elif_branches() {
    let source = r#"
        let score = 85
        if score >= 90 do
            "A"
        else if score >= 80 do
            "B"
        else if score >= 70 do
            "C"
        else do
            "F"
        end
    "#;
    assert_program_output(source, Value::String("B".to_string()));
}

#[test]
fn test_nested_if() {
    let source = r#"
        let x = 5
        let y = 10
        if x > 0 do
            if y > 0 do
                x + y
            else do
                x - y
            end
        else do
            0
        end
    "#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_while_loop_basic() {
    let source = r#"
        var i = 0
        while i < 5 do
            i = i + 1
        end
        i
    "#;
    assert_program_output(source, Value::Number(5.0));
}

#[test]
fn test_while_loop_with_accumulation() {
    let source = r#"
        var sum = 0
        var i = 1
        while i <= 5 do
            sum = sum + i
            i = i + 1
        end
        sum
    "#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_for_loop_basic() {
    let source = r#"
        var sum = 0
        for i in [1, 2, 3, 4] do
            sum = sum + i
        end
        sum
    "#;
    assert_program_output(source, Value::Number(10.0)); // 1+2+3+4
}

#[test]
fn test_for_loop_array() {
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
fn test_break_in_while() {
    let source = r#"
        var i = 0
        while true do
            i = i + 1
            if i == 5 do
                break
            end
        end
        i
    "#;
    assert_program_output(source, Value::Number(5.0));
}

#[test]
fn test_break_in_for_loop() {
    let source = r#"
        var count = 0
        for i in [1, 2, 3, 4, 5, 6, 7, 8, 9] do
            if i == 5 do
                break
            end
            count = i
        end
        count
    "#;
    assert_program_output(source, Value::Number(4.0));
}

#[test]
fn test_continue_in_while() {
    let source = r#"
        var sum = 0
        var i = 0
        while i < 10 do
            i = i + 1
            if i % 2 == 0 do
                continue
            end
            sum = sum + i
        end
        sum
    "#;
    assert_program_output(source, Value::Number(25.0)); // 1+3+5+7+9
}

#[test]
fn test_continue_in_for_loop() {
    let source = r#"
        var sum = 0
        for i in [1, 2, 3, 4, 5] do
            if i == 3 do
                continue
            end
            sum = sum + i
        end
        sum
    "#;
    assert_program_output(source, Value::Number(12.0)); // 1+2+4+5
}
