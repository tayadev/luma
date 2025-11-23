use luma::bytecode::ir::{Chunk, Instruction, Constant};
use luma::ast::Span;

#[test]
fn test_diagnostic_with_span() {
    // Create a simple chunk with span information
    let mut chunk = Chunk::default();
    chunk.name = "test".to_string();
    
    // Add a constant and instruction with a span
    chunk.constants.push(Constant::Number(1.0));
    chunk.push_instruction(Instruction::Const(0), Some(Span::new(0, 10)));
    chunk.push_instruction(Instruction::Halt, None);
    
    // Verify spans are stored correctly
    assert_eq!(chunk.get_span(0), Some(Span::new(0, 10)));
    assert_eq!(chunk.get_span(1), None);
}

#[test]
fn test_error_formatting() {
    let source = "let x = 10\nlet y = 20\n";
    let error = luma::vm::VmError::with_location(
        "Test error message".to_string(),
        Some(Span::new(11, 21)),  // Second line
        Some("test.luma".to_string()),
    );
    
    let formatted = error.format(Some(source));
    println!("Formatted error:\n{}", formatted);
    assert!(formatted.contains("test.luma"));
    assert!(formatted.contains("2:"));  // Line 2 in format
    assert!(formatted.contains("Test error message"));
}
