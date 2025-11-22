use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_repl_basic_expression() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"1 + 2\n").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "Expected output to contain '3', got: {}", stdout);
}

#[test]
fn test_repl_variable_persistence() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    // Define a variable and use it in subsequent expression
    stdin.write_all(b"let x = 10\nx * 2\n").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show null for let statement, then 20 for x * 2
    assert!(stdout.contains("null"), "Expected 'null' for let statement, got: {}", stdout);
    assert!(stdout.contains("20"), "Expected '20' for x * 2, got: {}", stdout);
}

#[test]
fn test_repl_multiple_variables() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    // Define multiple variables and compute with them
    stdin.write_all(b"let x = 5\nlet y = 10\nx + y\n").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("15"), "Expected '15' for x + y, got: {}", stdout);
}

#[test]
fn test_repl_function_persistence() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    // Define a function and call it
    stdin.write_all(b"let double = fn(n: Number) do n * 2 end\ndouble(21)\n")
        .expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"), "Expected '42' for double(21), got: {}", stdout);
}

#[test]
fn test_repl_parse_error_recovery() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    // Send an invalid expression followed by a valid one
    stdin.write_all(b"1 +\n5 + 5\n").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    // REPL should continue after parse errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stderr.contains("Parse error") || stdout.contains("Parse error"), 
            "Expected parse error in output");
    assert!(stdout.contains("10"), "Expected '10' for 5 + 5 after error, got: {}", stdout);
}

#[test]
fn test_repl_empty_lines() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    // Send empty lines mixed with expressions
    stdin.write_all(b"\n42\n\n").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"), "Expected '42' in output, got: {}", stdout);
}
