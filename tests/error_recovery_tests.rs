//! Tests for parser error recovery and diagnostic messages.
//!
//! This module tests that the parser can:
//! 1. Recover from errors and continue parsing
//! 2. Accumulate multiple errors in a single pass
//! 3. Produce accurate and helpful diagnostic messages

use luma::diagnostics::Diagnostic;
use luma::parser;

/// Helper to extract error messages from diagnostics
fn error_messages(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics.iter().map(|d| d.message.clone()).collect()
}

/// Helper to extract error line numbers from diagnostics
fn error_lines(diagnostics: &[Diagnostic], source: &str) -> Vec<usize> {
    let line_index = luma::diagnostics::LineIndex::new(source);
    diagnostics
        .iter()
        .map(|d| line_index.line_col(d.span.start).0)
        .collect()
}

#[test]
fn test_multiple_parse_errors_at_statement_boundaries() {
    let source = "@error1\nlet x = 1\n#error2\nlet y = 2\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2, "Expected 2 errors, got {}", errors.len());

    // Check that both errors are reported at correct lines
    let lines = error_lines(&errors, source);
    assert_eq!(lines, vec![1, 3], "Expected errors at lines 1 and 3");
}

#[test]
fn test_multiple_parse_errors_within_statements() {
    let source = "let a = @invalid1\nlet b = 42\nlet c = #invalid2\nlet d = 100\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2, "Expected 2 errors, got {}", errors.len());

    // Check that errors are at correct lines
    let lines = error_lines(&errors, source);
    assert_eq!(lines, vec![1, 3], "Expected errors at lines 1 and 3");
}

#[test]
fn test_error_message_for_unexpected_character() {
    let source = "@invalid";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    let msg = &errors[0].message;
    assert!(
        msg.contains("unexpected") || msg.contains("@"),
        "Error message should mention unexpected character, got: {}",
        msg
    );
}

#[test]
fn test_error_message_for_keyword_as_identifier() {
    let source = "let let = 1";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    let messages = error_messages(&errors);
    let has_keyword_error = messages
        .iter()
        .any(|m| m.contains("keyword") || m.contains("let"));
    assert!(
        has_keyword_error,
        "Error message should mention keyword, got: {:?}",
        messages
    );
}

#[test]
fn test_error_message_for_incomplete_expression() {
    let source = "let x = 1 +";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn test_single_error_produces_single_diagnostic() {
    let source = "@invalid\nlet x = 1\nlet y = 2\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(
        errors.len(),
        1,
        "Should have exactly 1 error for single issue"
    );

    // Verify the error is at line 1
    let lines = error_lines(&errors, source);
    assert_eq!(lines, vec![1]);
}

#[test]
fn test_valid_program_produces_no_errors() {
    let source = "let x = 1\nlet y = 2\nlet z = x + y\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_ok(), "Valid program should parse successfully");
}

#[test]
fn test_error_recovery_continues_past_first_error() {
    // This is the key test for error recovery - parsing should continue
    // after the first error and find subsequent errors
    let source = "let a = !\nlet b = 42\nlet c = @\nlet d = 100\nlet e = #\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    // We should have multiple errors - recovery should have continued
    assert!(
        errors.len() >= 2,
        "Parser should recover and find multiple errors, found: {}",
        errors.len()
    );
}

#[test]
fn test_diagnostic_span_is_accurate() {
    let source = "let x = @error";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    let error = &errors[0];
    // The error span should point to the '@' character at offset 8
    assert_eq!(
        error.span.start, 8,
        "Error span should start at '@' (offset 8)"
    );
}

#[test]
fn test_diagnostic_filename_is_preserved() {
    let source = "@error";
    let filename = "my_custom_file.luma";

    let result = parser::parse(source, filename);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors[0].filename, filename);
}

#[test]
fn test_empty_source_produces_no_errors() {
    let result = parser::parse("", "test.luma");
    assert!(result.is_ok());
}

#[test]
fn test_whitespace_only_produces_no_errors() {
    let result = parser::parse("   \n\t\n  ", "test.luma");
    assert!(result.is_ok());
}

#[test]
fn test_comment_only_produces_no_errors() {
    let result = parser::parse("-- this is a comment\n", "test.luma");
    assert!(result.is_ok());
}

#[test]
fn test_multiline_comment_only_produces_no_errors() {
    let result = parser::parse("--[[ multiline\ncomment ]]", "test.luma");
    assert!(result.is_ok());
}

#[test]
fn test_error_in_function_body_reports_correct_location() {
    let source = "let f = fn(x: Number): Number do\n  @error\nend\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    // Error should be on line 2 (inside the function body)
    let lines = error_lines(&errors, source);
    assert!(
        lines.contains(&2),
        "Error should be on line 2 inside function body, got lines: {:?}",
        lines
    );
}

#[test]
fn test_diagnostic_format_contains_source_snippet() {
    let source = "let x = @error\nlet y = 2\n";

    let result = parser::parse(source, "test.luma");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    let formatted = errors[0].format(source);

    // The formatted diagnostic should contain the source line
    assert!(
        formatted.contains("let x = @error"),
        "Formatted diagnostic should contain source line, got:\n{}",
        formatted
    );

    // Should contain file location
    assert!(
        formatted.contains("test.luma:1:"),
        "Formatted diagnostic should contain file:line, got:\n{}",
        formatted
    );
}
