use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_stdin_default_run() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"10 + 20").expect("Failed to write to stdin");
    // Close stdin by dropping the mutable reference
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "30");
}

#[test]
fn test_stdin_ast_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("ast")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"1 + 2").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Program"));
    assert!(stdout.contains("Binary"));
    assert!(stdout.contains("Add"));
}

#[test]
fn test_stdin_check_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("check")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"1 + 2").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "Typecheck: OK");
}

#[test]
fn test_stdin_bytecode_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("bytecode")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"1 + 2").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Chunk"));
    assert!(stdout.contains("instructions"));
    assert!(stdout.contains("Add"));
}

#[test]
fn test_stdin_multiline_program() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_luma"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn luma command");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"let x = 10\nlet y = 20\nx + y").expect("Failed to write to stdin");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait for command");
    
    assert!(output.status.success(), "Command failed with stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "30");
}
