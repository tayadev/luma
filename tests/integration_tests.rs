mod common;

use luma::pipeline::Pipeline;
use luma::vm::value::Value;

/// Macro to generate individual test functions for each fixture
///
/// This expands to a separate #[test] function per fixture, making them
/// appear as independent tests in test reports and allowing parallel execution.
macro_rules! fixture_tests {
    (
        valid: [ $( $valid_name:ident ),* ],
        parse_errors: [ $( $parse_error_name:ident ),* ],
        type_errors: [ $( $type_error_name:ident ),* ]
    ) => {
        // Generate tests for valid fixtures
        $(
            #[test]
            fn $valid_name() {
                let fixture_name = stringify!($valid_name);
                let source = common::load_fixture(&format!("valid/{}", fixture_name));
                let pipeline = Pipeline::new(source, format!("{}.luma", fixture_name));
                let result = pipeline.run_all();

                assert!(
                    result.is_ok(),
                    "Valid fixture '{}' should execute successfully: {:?}",
                    fixture_name,
                    result.err()
                );
            }
        )*

        // Generate tests for parse error fixtures
        $(
            #[test]
            fn $parse_error_name() {
                let fixture_name = stringify!($parse_error_name);
                let source = common::load_fixture(&format!("invalid/parse_errors/{}", fixture_name));
                let pipeline = Pipeline::new(source, format!("{}.luma", fixture_name));
                let result = pipeline.run_all();

                assert!(
                    result.is_err(),
                    "Parse error fixture '{}' should fail to parse",
                    fixture_name
                );

                // Verify it's specifically a parse error
                if let Err(e) = result {
                    assert!(
                        matches!(e, luma::pipeline::PipelineError::Parse(_)),
                        "Fixture '{}' should produce a parse error, got: {:?}",
                        fixture_name,
                        e
                    );
                }
            }
        )*

        // Generate tests for type error fixtures
        $(
            #[test]
            fn $type_error_name() {
                let fixture_name = stringify!($type_error_name);
                let source = common::load_fixture(&format!("invalid/type_errors/{}", fixture_name));
                let pipeline = Pipeline::new(source, format!("{}.luma", fixture_name));
                let result = pipeline.run_all();

                assert!(
                    result.is_err(),
                    "Type error fixture '{}' should fail type checking",
                    fixture_name
                );

                // Verify it's specifically a type error
                if let Err(e) = result {
                    assert!(
                        matches!(e, luma::pipeline::PipelineError::Typecheck(_)),
                        "Fixture '{}' should produce a type error, got: {:?}",
                        fixture_name,
                        e
                    );
                }
            }
        )*
    };
}

// Expand all fixture tests
fixture_tests! {
    valid: [
        boolean_logic,
        closure,
        complex_expression,
        control_flow,
        early_return,
        factorial,
        fibonacci,
        higher_order,
        list_operations,
        match_patterns,
        nested_functions,
        string_operations,
        table_operations
    ],
    parse_errors: [
        invalid_operator,
        missing_brace,
        missing_end,
        missing_value,
        unclosed_list,
        unclosed_string
    ],
    type_errors: [
        call_non_function,
        function_arg_mismatch,
        heterogeneous_list,
        immutable_assignment,
        invalid_operation,
        non_boolean_condition,
        return_type_mismatch,
        type_mismatch
    ]
}

// Specific result validation tests
#[test]
fn test_fibonacci_result() {
    let source = common::load_fixture("valid/fibonacci");
    let pipeline = Pipeline::new(source, "fibonacci.luma".to_string());
    let result = pipeline.run_all();

    assert!(result.is_ok());
    let value = result.unwrap();

    // fib(10) should be 55
    match value {
        Value::Number(n) => {
            assert!(
                (n - 55.0).abs() < f64::EPSILON,
                "fib(10) should equal 55, got {}",
                n
            );
        }
        _ => panic!("Expected number result, got {:?}", value),
    }
}

#[test]
fn test_factorial_result() {
    let source = common::load_fixture("valid/factorial");
    let pipeline = Pipeline::new(source, "factorial.luma".to_string());
    let result = pipeline.run_all();

    assert!(result.is_ok());
    let value = result.unwrap();

    // factorial(5) should be 120
    match value {
        Value::Number(n) => {
            assert!(
                (n - 120.0).abs() < f64::EPSILON,
                "factorial(5) should equal 120, got {}",
                n
            );
        }
        _ => panic!("Expected number result, got {:?}", value),
    }
}

#[test]
fn test_closure_result() {
    let source = common::load_fixture("valid/closure");
    let pipeline = Pipeline::new(source, "closure.luma".to_string());
    let result = pipeline.run_all();

    assert!(result.is_ok());
    let value = result.unwrap();

    // The counter is called 3 times, so the result should be 3
    match value {
        Value::Number(n) => {
            assert!(
                (n - 3.0).abs() < f64::EPSILON,
                "Counter should equal 3, got {}",
                n
            );
        }
        _ => panic!("Expected number result, got {:?}", value),
    }
}
