//! Control flow tests (if/else, while, for, break, continue)

mod common;

use common::assert_program_output;
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

#[test]
fn test_break_dowhile() {
    let source = r#"
var sum = 0
var i = 0
do
  i = i + 1
  if i == 4 do
    break
  end
  sum = sum + i
while i < 10 end
sum
"#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_break_for() {
    let source = r#"
var sum = 0
for i in [1, 2, 3, 4, 5] do
  if i == 3 do
    break
  end
  sum = sum + i
end
sum
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_break_nested() {
    let source = r#"
var sum = 0
var i = 0
while i < 5 do
  i = i + 1
  var j = 0
  while j < 10 do
    j = j + 1
    sum = sum + 1
    if sum == 7 do
      break 2
    end
  end
end
sum
"#;
    assert_program_output(source, Value::Number(7.0));
}

#[test]
fn test_break_simple() {
    let source = r#"
var sum = 0
var i = 0
while i < 10 do
  if i == 3 do
    break
  end
  sum = sum + i
  i = i + 1
end
sum
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_continue_for() {
    let source = r#"
var sum = 0
for i in [1, 2, 3, 4, 5] do
  if i == 3 || i == 4 do
    continue
  end
  sum = sum + i
end
sum
"#;
    assert_program_output(source, Value::Number(8.0));
}

#[test]
fn test_continue_nested() {
    let source = r#"
var sum = 0
var i = 0
while i < 3 do
  i = i + 1
  var j = 0
  while j < 5 do
    j = j + 1
    if j == 2 do
      continue 2
    end
    sum = sum + j
  end
end
sum
"#;
    assert_program_output(source, Value::Number(13.0));
}

#[test]
fn test_continue_simple() {
    let source = r#"
var sum = 0
var i = 0
while i < 10 do
  i = i + 1
  if i == 3 || i == 5 do
    continue
  end
  sum = sum + i
end
sum
"#;
    assert_program_output(source, Value::Number(47.0));
}

#[test]
fn test_do_while_sum() {
    let source = r#"
var sum = 0
var i = 1
do
  sum = sum + i
  i = i + 1
while i <= 5 end
sum
"#;
    assert_program_output(source, Value::Number(15.0));
}

#[test]
fn test_for_destructure_indexed() {
    let source = r#"
let arr = [4, 5, 6]
var total = 0
for [item, i] in indexed(arr) do
  total = total + i
end
total
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_for_simple() {
    let source = r#"
for x in [1, 2] do
  x
end
"#;
    assert_program_output(source, Value::Null);
}

#[test]
fn test_for_sum() {
    let source = r#"
var sum = 0
for x in [1, 2, 3, 4] do
  sum = sum + x
end
sum
"#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_if_local() {
    let source = r#"
if true do
  var y = 41
  y = y + 1
  y
end
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_if_then_else() {
    let source = r#"
var x = 1
if true do
  x = x + 1
end
x
"#;
    assert_program_output(source, Value::Number(2.0));
}

#[test]
fn test_indexed_sum() {
    let source = r#"
let arr = [4, 5, 6]
var total = 0
for pair in indexed(arr) do
  total = total + pair[1]
end
total
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_iter_table_sum() {
    let source = r#"
let t = { a = 1, b = 2, c = 3 }
var sum = 0
for [k, v] in t do
  sum = sum + v
end
sum
"#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_range_sum() {
    let source = r#"
var sum = 0
for x in range(0, 5) do
  sum = sum + x
end
sum
"#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_while_sum() {
    let source = r#"
var sum = 0
var i = 0
while i < 3 do
  sum = sum + i
  i = i + 1
end
sum
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_conditional_assignments() {
    let source = r#"
        let x = 10
        let y = if x > 5 do 100 else do 1 end
        y
    "#;
    assert_program_output(source, Value::Number(100.0));
}

#[test]
fn test_match_simple() {
    let source = r#"
-- Simple match statement test
let obj = { ok = 42 }

match obj do
  ok do
    obj.ok
  end
  err do
    obj.err
  end
  _ do
    -1
  end
end
"#;
    assert_program_output(source, Value::Number(42.0));
}
