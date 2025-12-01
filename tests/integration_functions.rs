//! Function tests (definitions, calls, recursion, closures)

mod common;

use common::assert_program_output;
use luma::vm::value::Value;

#[test]
fn test_simple_function_definition_and_call() {
    let source = r#"
        let add = fn(a: Number, b: Number): Number do
            a + b
        end
        add(5, 3)
    "#;
    assert_program_output(source, Value::Number(8.0));
}

#[test]
fn test_function_with_no_args() {
    let source = r#"
        let get_answer = fn(): Number do
            42
        end
        get_answer()
    "#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_function_returns_string() {
    let source = r#"
        let greet = fn(name: String): String do
            "Hello, " + name
        end
        greet("World")
    "#;
    assert_program_output(source, Value::String("Hello, World".to_string()));
}

#[test]
fn test_function_returns_boolean() {
    let source = r#"
        let is_even = fn(n: Number): Boolean do
            n % 2 == 0
        end
        is_even(4)
    "#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_function_with_local_variables() {
    let source = r#"
        let compute = fn(): Number do
            let x = 10
            let y = 20
            x + y
        end
        compute()
    "#;
    assert_program_output(source, Value::Number(30.0));
}

#[test]
fn test_function_with_multiple_statements() {
    let source = r#"
        let compute_sequence = fn(): Number do
            let a = 5
            let b = 10
            let c = a + b
            c * 2
        end
        compute_sequence()
    "#;
    assert_program_output(source, Value::Number(30.0));
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        let square = fn(x: Number): Number do
            x * x
        end
        let cube = fn(x: Number): Number do
            x * square(x)
        end
        cube(3)
    "#;
    assert_program_output(source, Value::Number(27.0));
}

#[test]
fn test_function_recursion_factorial() {
    let source = r#"
        let factorial = fn(n: Number): Number do
            return if n <= 1 do
                1
            else do
                n * factorial(n - 1)
            end
        end
        factorial(5)
    "#;
    assert_program_output(source, Value::Number(120.0));
}

#[test]
fn test_function_recursion_fibonacci() {
    let source = r#"
        let fib = fn(n: Number): Number do
            return if n <= 1 do
                n
            else do
                fib(n - 1) + fib(n - 2)
            end
        end
        fib(6)
    "#;
    assert_program_output(source, Value::Number(8.0));
}

#[test]
fn test_function_early_return() {
    let source = r#"
        let find_first_even = fn(n: Number): Number do
            if n % 2 == 0 do
                return n
            end
            n + 1
        end
        find_first_even(5)
    "#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_function_shadowing_global() {
    let source = r#"
        let x = 10
        let test_shadowing = fn(): Number do
            let x = 20
            x
        end
        test_shadowing()
    "#;
    assert_program_output(source, Value::Number(20.0));
}

#[test]
fn test_function_closure_over_global() {
    let source = r#"
        let global_x = 100
        let get_global = fn(): Number do
            global_x
        end
        get_global()
    "#;
    assert_program_output(source, Value::Number(100.0));
}

#[test]
fn test_mutually_recursive_functions() {
    let source = r#"
        let is_even = fn(n: Number): Boolean do
            return if n == 0 do
                true
            else do
                is_odd(n - 1)
            end
        end
        let is_odd = fn(n: Number): Boolean do
            return if n == 0 do
                false
            else do
                is_even(n - 1)
            end
        end
        is_even(4)
    "#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_closures_shared_state() {
    let source = r#"
var makePair = fn() do
  var x = 0
  let inc = fn() do
    x = x + 1
    x
  end
  let add = fn(n: Number) do
    x = x + n
    x
  end
  { inc = inc, add = add }
end

let pair = makePair()
pair.inc()
pair.add(5)
pair.inc()
"#;
    assert_program_output(source, Value::Number(7.0));
}

#[test]
fn test_closure_simple() {
    let source = r#"
var makeCounter = fn() do
  var count = 0
  return fn() do
    count = count + 1
    count
  end
end

let counter = makeCounter()
print(counter())
print(counter())
counter()
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_factorial() {
    let source = r#"
let factorial = fn(n: Number) do
  if n <= 1 do
    1
  else do
    n * factorial(n - 1)
  end
end

factorial(5)
"#;
    assert_program_output(source, Value::Number(120.0));
}

#[test]
fn test_fibonacci() {
    let source = r#"
let fib = fn(n: Number) do
  if n <= 1 do
    n
  else do
    fib(n - 1) + fib(n - 2)
  end
end

fib(10)
"#;
    assert_program_output(source, Value::Number(55.0));
}

#[test]
fn test_fn_multiarg() {
    let source = r#"
var sum = fn(a: Number, b: Number, c: Number) do
  a + b + c
end

sum(1, 2, 3)
"#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_fn_simple() {
    let source = r#"
var add = fn(x: Number, y: Number) do
  x + y
end

add(3, 4)
"#;
    assert_program_output(source, Value::Number(7.0));
}

#[test]
fn test_module_closure_capture() {
    let source = r#"
let module_var = 42

let get_var = fn() do
  module_var
end

get_var()
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
#[ignore = "Requires external module_with_closure.luma file"]
fn test_module_closure_capture_import() {
    let source = r#"
let get_var = import("module_with_closure.luma")
get_var()
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_mutual_recursion() {
    let source = r#"
var isEven = fn(n: Number) do
  if n == 0 do
    true
  else do
    isOdd(n - 1)
  end
end

var isOdd = fn(n: Number) do
  if n == 0 do
    false
  else do
    isEven(n - 1)
  end
end

print(isEven(4))
print(isOdd(4))
print(isEven(5))
isOdd(5)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_named_args_mixed() {
    let source = r#"
let calc = fn(a: Number, b: Number, c: Number) do
  a * 100 + b * 10 + c
end
calc(1, c = 3, b = 2)
"#;
    assert_program_output(source, Value::Number(123.0));
}

#[test]
fn test_named_args_reorder() {
    let source = r#"
let add = fn(a: Number, b: Number) do
  a + b
end
add(b = 2, a = 5)
"#;
    assert_program_output(source, Value::Number(7.0));
}
