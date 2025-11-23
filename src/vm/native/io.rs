//! I/O native functions: print, write, read_file, write_file, file_exists, panic

use super::helpers::{make_result_err, make_result_ok};
use crate::vm::value::Value;

/// Native function: print(...values) -> null
/// Prints all arguments to stdout, separated by tabs
pub fn native_print(args: &[Value]) -> Result<Value, String> {
    let mut output = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push('\t');
        }
        output.push_str(&format!("{}", arg));
    }
    println!("{}", output);
    Ok(Value::Null)
}

/// Native function: write(fd: Number, content: String) -> Result(Null, String)
/// Writes content to a file descriptor (1=stdout, 2=stderr)
pub fn native_write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write() expects 2 arguments, got {}", args.len()));
    }

    let fd = match &args[0] {
        Value::Number(n) => *n as i32,
        _ => {
            return Ok(make_result_err(
                "write() first argument must be a number (file descriptor)".to_string(),
            ));
        }
    };

    let content = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => format!("{}", other),
    };

    use std::io::Write;
    let result = match fd {
        1 => std::io::stdout().write_all(content.as_bytes()),
        2 => std::io::stderr().write_all(content.as_bytes()),
        _ => {
            return Ok(make_result_err(format!(
                "Invalid file descriptor: {}. Only 1 (stdout) and 2 (stderr) are supported",
                fd
            )));
        }
    };

    match result {
        Ok(_) => Ok(make_result_ok(Value::Null)),
        Err(e) => Ok(make_result_err(format!("I/O error: {}", e))),
    }
}

/// Native function: read_file(path: String) -> Result(String, String)
/// Reads entire file contents as a string
pub fn native_read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "read_file() expects 1 argument, got {}",
            args.len()
        ));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Ok(make_result_err(
                "read_file() argument must be a string (file path)".to_string(),
            ));
        }
    };

    match std::fs::read_to_string(path) {
        Ok(content) => Ok(make_result_ok(Value::String(content))),
        Err(e) => Ok(make_result_err(format!(
            "Failed to read file '{}': {}",
            path, e
        ))),
    }
}

/// Native function: write_file(path: String, content: String) -> Result(Null, String)
/// Writes content to a file, creating or overwriting it
pub fn native_write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "write_file() expects 2 arguments, got {}",
            args.len()
        ));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Ok(make_result_err(
                "write_file() first argument must be a string (file path)".to_string(),
            ));
        }
    };

    let content = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => format!("{}", other),
    };

    match std::fs::write(path, content) {
        Ok(_) => Ok(make_result_ok(Value::Null)),
        Err(e) => Ok(make_result_err(format!(
            "Failed to write file '{}': {}",
            path, e
        ))),
    }
}

/// Native function: file_exists(path: String) -> Boolean
/// Checks if a file or directory exists at the given path
pub fn native_file_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "file_exists() expects 1 argument, got {}",
            args.len()
        ));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("file_exists() argument must be a string (file path)".to_string()),
    };

    Ok(Value::Boolean(std::path::Path::new(path).exists()))
}

/// Native function: panic(message: String) -> Never
/// Prints error message to stderr and terminates the program
pub fn native_panic(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("panic() expects 1 argument, got {}", args.len()));
    }

    let message = match &args[0] {
        Value::String(s) => s.clone(),
        other => format!("{}", other),
    };

    eprintln!("PANIC: {}", message);
    std::process::exit(1);
}
