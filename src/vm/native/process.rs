//! Process-related native functions: os detection, exit, environment access.
//!
//! This module provides the `process` global object which contains:
//! - `process.os` - The current operating system ('windows', 'linux', or 'macos')
//! - `process.exit(code)` - Terminates the program with the given exit code

use crate::vm::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Native function: process.exit(code: Number) -> Never
/// Terminates the program with the given exit code.
pub fn native_process_exit(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "process.exit() expects 1 argument, got {}",
            args.len()
        ));
    }

    let code = match &args[0] {
        Value::Number(n) => *n as i32,
        _ => return Err("process.exit() argument must be a number".to_string()),
    };

    std::process::exit(code);
}

/// Returns the current operating system as a string.
/// Returns 'windows', 'linux', or 'macos'.
fn get_os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "unknown"
    }
}

/// Create the `process` module table with process information and functions.
pub fn create_process_module() -> Value {
    let mut process_table: HashMap<String, Value> = HashMap::new();

    // Add the OS name as a string property
    process_table.insert("os".to_string(), Value::String(get_os_name().to_string()));

    // Add the exit function
    process_table.insert(
        "exit".to_string(),
        Value::NativeFunction {
            name: "process.exit".to_string(),
            arity: 1,
        },
    );

    Value::Table(Rc::new(RefCell::new(process_table)))
}
