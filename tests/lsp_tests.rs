//! Tests for the Language Server Protocol implementation

use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Test that the LSP server can start and respond to a basic request
#[test]
fn test_lsp_server_starts() {
    // Build the project first
    let build_status = Command::new("cargo")
        .args(["build", "--bin", "luma"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to build");
    assert!(build_status.success(), "Build failed");

    // Start the LSP server
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "luma", "--", "lsp"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start LSP server");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    // Send an initialize request
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#;
    let content_length = init_request.len();
    let message = format!("Content-Length: {content_length}\r\n\r\n{init_request}");
    stdin
        .write_all(message.as_bytes())
        .expect("Failed to write to stdin");

    // Give the server time to process
    thread::sleep(Duration::from_millis(100));

    // Send initialized notification
    let initialized = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    let content_length = initialized.len();
    let message = format!("Content-Length: {content_length}\r\n\r\n{initialized}");
    stdin
        .write_all(message.as_bytes())
        .expect("Failed to write initialized");

    // Give the server time to process
    thread::sleep(Duration::from_millis(100));

    // Send shutdown and exit
    let shutdown_request = r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#;
    let content_length = shutdown_request.len();
    let message = format!("Content-Length: {content_length}\r\n\r\n{shutdown_request}");
    stdin
        .write_all(message.as_bytes())
        .expect("Failed to write shutdown");

    // Give the server time to process
    thread::sleep(Duration::from_millis(100));

    let exit_notification = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    let content_length = exit_notification.len();
    let message = format!("Content-Length: {content_length}\r\n\r\n{exit_notification}");
    stdin
        .write_all(message.as_bytes())
        .expect("Failed to write exit");

    // Wait for the process to exit (with timeout)
    let output = child.wait_with_output().expect("Failed to wait for child");

    // The LSP server should respond
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that we got a response (should contain Content-Length header)
    assert!(
        stdout.contains("Content-Length:"),
        "LSP server should respond with Content-Length header. Got: {stdout}"
    );
}
