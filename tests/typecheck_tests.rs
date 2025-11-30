//! Tests for typecheck error accumulation.
//!
//! This module tests that the type checker:
//! 1. Accumulates all type errors rather than stopping at the first one
//! 2. Produces accurate diagnostic messages with proper source locations
//! 3. Handles various error scenarios correctly

use luma::diagnostics::{Diagnostic, DiagnosticKind};
use luma::parser;
use luma::typecheck::{TypeError, typecheck_program};

/// Helper to extract error messages from type errors
fn error_messages(errors: &[TypeError]) -> Vec<String> {
    errors.iter().map(|e| e.message.clone()).collect()
}

#[test]
fn test_typecheck_accumulates_multiple_errors() {
    let source = r#"
let x: Number = "hello"
let y: String = 42
let z = undefined_var
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should have at least 3 errors: two type mismatches and one undefined variable
    assert!(
        errors.len() >= 3,
        "Expected at least 3 type errors, got {}",
        errors.len()
    );
}

#[test]
fn test_typecheck_error_for_type_mismatch() {
    let source = "let x: Number = \"hello\"";

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    let msg = &errors[0].message;
    assert!(
        msg.contains("Number") && msg.contains("String"),
        "Error should mention both Number and String types, got: {}",
        msg
    );
}

#[test]
fn test_typecheck_error_for_undefined_variable() {
    let source = "let x = undefined_var";

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    let msg = &errors[0].message;
    assert!(
        msg.contains("Undefined") || msg.contains("undefined_var"),
        "Error should mention undefined variable, got: {}",
        msg
    );
}

#[test]
fn test_typecheck_error_for_function_arity_mismatch() {
    let source = r#"
let add = fn(a: Number, b: Number): Number do
    a + b
end
let result = add(1)
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    let messages = error_messages(&errors);
    let has_arity_error = messages.iter().any(|m| {
        (m.contains("argument") || m.contains("expected")) && (m.contains("2") || m.contains("1"))
    });
    assert!(
        has_arity_error,
        "Error should mention argument count mismatch, got: {:?}",
        messages
    );
}

#[test]
fn test_typecheck_error_for_immutable_assignment() {
    let source = r#"
let x = 1
x = 2
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    let messages = error_messages(&errors);
    let has_immutable_error = messages
        .iter()
        .any(|m| m.contains("immutable") || m.contains("Cannot assign"));
    assert!(
        has_immutable_error,
        "Error should mention immutable assignment, got: {:?}",
        messages
    );
}

#[test]
fn test_typecheck_errors_have_spans() {
    let source = "let x: Number = \"hello\"";

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    // The type error should have a span for accurate location reporting
    assert!(
        errors[0].span.is_some(),
        "Type error should have a source span"
    );
}

#[test]
fn test_typecheck_multiple_undefined_variables() {
    let source = r#"
let a = undefined1
let b = undefined2
let c = undefined3
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should report all three undefined variable errors
    assert_eq!(errors.len(), 3, "Expected 3 undefined variable errors");

    // Each error should reference a different variable
    let messages = error_messages(&errors);
    assert!(messages.iter().any(|m| m.contains("undefined1")));
    assert!(messages.iter().any(|m| m.contains("undefined2")));
    assert!(messages.iter().any(|m| m.contains("undefined3")));
}

#[test]
fn test_typecheck_valid_program_produces_no_errors() {
    let source = r#"
let x = 1
let y = 2
let z = x + y
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(
        result.is_ok(),
        "Valid program should typecheck successfully"
    );
}

#[test]
fn test_typecheck_error_in_function_body() {
    let source = r#"
let f = fn(x: Number): Number do
    let y: String = x
    y
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn test_typecheck_errors_in_multiple_functions() {
    let source = r#"
let f = fn(x: Number): Number do
    let y: String = x
    1
end
let g = fn(a: String): String do
    let b: Number = a
    "test"
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should have errors from both functions
    assert!(
        errors.len() >= 2,
        "Expected errors from both functions, got {}",
        errors.len()
    );
}

#[test]
fn test_typecheck_error_format_with_diagnostics() {
    let source = "let x: Number = \"hello\"";

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    // Create a diagnostic from the error
    let error = &errors[0];
    let diag = Diagnostic::error(
        DiagnosticKind::Type,
        error.message.clone(),
        error.span.unwrap_or_else(|| luma::ast::Span::new(0, 0)),
        "test.luma".to_string(),
    );

    let formatted = diag.format(source);

    // The formatted diagnostic should contain the error message
    assert!(
        formatted.contains("Number") && formatted.contains("String"),
        "Formatted diagnostic should show type names, got:\n{}",
        formatted
    );

    // Should contain the file location
    assert!(
        formatted.contains("test.luma:1:"),
        "Formatted diagnostic should contain file:line, got:\n{}",
        formatted
    );
}

#[test]
fn test_typecheck_implicit_return_from_if_statement() {
    // Test case from issue: if statement as last statement in function should be implicit return
    let source = r#"
let fibonacci = fn(n: Number): Number do
  if n <= 1 do
    n
  else do
    fibonacci(n - 1) + fibonacci(n - 2)
  end
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(
        result.is_ok(),
        "If statement as implicit return should typecheck successfully, got errors: {:?}",
        result.err()
    );
}

#[test]
fn test_typecheck_implicit_return_from_if_with_elif() {
    let source = r#"
let grade = fn(score: Number): String do
  if score >= 90 do
    "A"
  else if score >= 80 do
    "B"
  else if score >= 70 do
    "C"
  else do
    "F"
  end
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(
        result.is_ok(),
        "If-elif statement as implicit return should typecheck successfully, got errors: {:?}",
        result.err()
    );
}

#[test]
fn test_typecheck_if_without_else_not_implicit_return() {
    // If without else cannot be an implicit return because not all paths return
    let source = r#"
let test = fn(n: Number): Number do
  if n > 0 do
    n
  end
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(
        result.is_err(),
        "If without else should fail typecheck for Number return type"
    );
}

#[test]
fn test_typecheck_nested_if_implicit_return() {
    let source = r#"
let test = fn(n: Number): Number do
  if n > 10 do
    if n > 20 do
      20
    else do
      10
    end
  else do
    n
  end
end
"#;

    let ast = parser::parse(source, "test.luma").expect("Should parse successfully");
    let result = typecheck_program(&ast);

    assert!(
        result.is_ok(),
        "Nested if as implicit return should typecheck successfully, got errors: {:?}",
        result.err()
    );
}
