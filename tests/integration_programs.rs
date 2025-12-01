//! Full program tests (complete algorithms and programs)

mod common;

use common::assert_program_output;
use luma::vm::value::Value;

#[test]
fn test_factorial_iterative() {
    let source = r#"
        let factorial = fn(n: Number): Number do
            var result = 1
            var i = 2
            while i <= n do
                result = result * i
                i = i + 1
            end
            result
        end
        factorial(5)
    "#;
    assert_program_output(source, Value::Number(120.0));
}

#[test]
fn test_fibonacci_program() {
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
fn test_is_prime_program() {
    let source = r#"
        let is_prime = fn(n: Number): Boolean do
            return if n < 2 do
                false
            else do
                var i = 2
                var result = true
                while i * i <= n do
                    if n % i == 0 do
                        result = false
                    end
                    i = i + 1
                end
                result
            end
        end
        is_prime(7)
    "#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_gcd_program() {
    let source = r#"
        let gcd = fn(a: Number, b: Number): Number do
            return if b == 0 do
                a
            else do
                gcd(b, a % b)
            end
        end
        gcd(48, 18)
    "#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_lcm_program() {
    let source = r#"
        let gcd = fn(a: Number, b: Number): Number do
            return if b == 0 do
                a
            else do
                gcd(b, a % b)
            end
        end
        let lcm = fn(a: Number, b: Number): Number do
            return (a * b) / gcd(a, b)
        end
        lcm(12, 18)
    "#;
    assert_program_output(source, Value::Number(36.0));
}

#[test]
fn test_collatz_sequence() {
    let source = r#"
        let collatz_length = fn(n: Number): Number do
            return if n == 1 do
                0
            else do
                return if n % 2 == 0 do
                    1 + collatz_length(n / 2)
                else do
                    1 + collatz_length(n * 3 + 1)
                end
            end
        end
        collatz_length(5)
    "#;
    assert_program_output(source, Value::Number(5.0)); // 5 -> 16 -> 8 -> 4 -> 2 -> 1
}

#[test]
fn test_string_operations() {
    let source = r#"
        let concat_three = fn(a: String, b: String, c: String): String do
            a + b + c
        end
        concat_three("Hello", " ", "World")
    "#;
    assert_program_output(source, Value::String("Hello World".to_string()));
}

#[test]
fn test_complex_logic_with_state() {
    let source = r#"
        let count_evens = fn(arr: List): Number do
            var count = 0
            for num in arr do
                if num % 2 == 0 do
                    count = count + 1
                end
            end
            count
        end
        count_evens([1, 2, 3, 4, 5, 6])
    "#;
    assert_program_output(source, Value::Number(3.0)); // 2, 4, 6
}

#[test]
fn test_nested_data_structure() {
    let source = r#"
        let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
        matrix[1][1]
    "#;
    assert_program_output(source, Value::Number(5.0));
}

#[test]
fn test_matrix_diagonal_sum() {
    let source = r#"
        let matrix = [[1, 2], [3, 4]]
        matrix[0][0] + matrix[1][1]
    "#;
    assert_program_output(source, Value::Number(5.0));
}

#[test]
fn test_dynamic_program_construction() {
    let source = r#"
        var result = 0
        for i in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] do
            if i % 2 == 1 do
                result = result + i
            end
        end
        result
    "#;
    assert_program_output(source, Value::Number(25.0)); // 1+3+5+7+9
}
