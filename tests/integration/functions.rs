//! Integration tests for functions through the full pipeline

use crate::integration::assert_program_output;
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
