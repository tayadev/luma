//! Full pipeline integration tests
//!
//! These tests verify end-to-end functionality from source code through to runtime execution.
//! Tests are organized by feature in the `integration/` subdirectory:
//!
//! - **expressions**: Basic literals, operators, variables
//! - **functions**: Definitions, calls, recursion, closures
//! - **control_flow**: if/else, loops, break/continue
//! - **collections**: Arrays, maps, iteration
//! - **errors**: Error handling, type checking, runtime errors
//! - **programs**: Complete program examples (algorithms, data structures)

// Declare integration as a module directory
mod integration {
    pub mod collections;
    pub mod control_flow;
    pub mod errors;
    pub mod expressions;
    pub mod functions;
    pub mod programs;

    use luma::pipeline::Pipeline;
    use luma::vm::value::Value;

    /// Helper to run a Luma program and get its result
    pub fn run_program(source: &str) -> Result<Value, String> {
        let pipeline = Pipeline::new(source.to_string(), "test.luma".to_string());
        pipeline.run_all().map_err(|e| e.format_display())
    }

    /// Helper to assert program output equals expected value
    pub fn assert_program_output(source: &str, expected: Value) {
        match run_program(source) {
            Ok(value) => {
                assert_eq!(
                    value, expected,
                    "Program output mismatch.\nSource: {}\nExpected: {}\nGot: {}",
                    source, expected, value
                );
            }
            Err(e) => {
                panic!(
                    "Program execution failed.\nSource: {}\nError: {}",
                    source, e
                );
            }
        }
    }

    /// Helper to assert program fails with expected error type
    pub fn assert_program_fails(source: &str, expected_error_substring: &str) {
        match run_program(source) {
            Ok(value) => {
                panic!(
                    "Program should have failed but returned: {}\nSource: {}",
                    value, source
                );
            }
            Err(e) => {
                assert!(
                    e.contains(expected_error_substring),
                    "Error should contain '{}' but got: {}",
                    expected_error_substring,
                    e
                );
            }
        }
    }
}
