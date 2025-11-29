//! FFI (Foreign Function Interface) native functions.
//!
//! This module provides the infrastructure for calling native C code from Luma.
//! The FFI system allows Luma code to:
//! - Define C type and function signatures via `ffi.def()`
//! - Create C strings from Luma strings via `ffi.new_cstr()`
//! - Call external C functions through the returned definition object
//!
//! Note: This is a stub implementation that provides the interface for FFI
//! without actually loading dynamic libraries. Full FFI implementation would
//! require integration with libffi or similar.

use crate::vm::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::helpers::make_result_err;

/// Global counter for generating unique external handles
static NEXT_EXTERNAL_HANDLE: AtomicUsize = AtomicUsize::new(1);

/// Generate a new unique handle for an external resource
fn new_external_handle() -> usize {
    NEXT_EXTERNAL_HANDLE.fetch_add(1, Ordering::SeqCst)
}

/// Native function: ffi.def(declarations: String) -> Table
///
/// Parses C-style type and function declarations and returns a table
/// containing callable wrappers for the declared functions.
///
/// Example:
/// ```luma
/// let c_def = ffi.def("
///   typedef struct FILE FILE;
///   FILE *fopen(const char *path, const char *mode);
///   int fclose(FILE *stream);
/// ")
/// ```
///
/// Note: This is a stub implementation. Full implementation would:
/// 1. Parse the C declarations
/// 2. Load symbols from the C runtime library
/// 3. Create callable wrappers using libffi
pub fn native_ffi_def(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("ffi.def() expects 1 argument, got {}", args.len()));
    }

    let declarations = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(
                "ffi.def() argument must be a string containing C declarations".to_string(),
            );
        }
    };

    // Create a stub table that contains the FFI definition
    // In a full implementation, this would parse the declarations and
    // create callable function wrappers
    let mut ffi_table: HashMap<String, Value> = HashMap::new();

    // Store the raw declarations for debugging/introspection
    ffi_table.insert(
        "__declarations".to_string(),
        Value::String(declarations.clone()),
    );

    // Mark this as an FFI definition table
    ffi_table.insert("__ffi_def".to_string(), Value::Boolean(true));

    // Parse declarations and create stub function entries
    // This is a simplified parser - a full implementation would use a proper C parser
    for line in declarations.lines() {
        let line = line.trim();
        // Skip typedef lines and empty lines
        if line.is_empty() || line.starts_with("typedef") {
            continue;
        }

        // Try to extract function name (very simplified parsing)
        // Look for pattern: return_type name(
        if let Some(paren_pos) = line.find('(') {
            let before_paren = &line[..paren_pos];
            // Find the last word before the opening parenthesis
            // This handles cases like "int *fopen(" or "FILE *fopen("
            let func_name = before_paren
                .split(|c: char| c.is_whitespace() || c == '*')
                .filter(|s| !s.is_empty())
                .next_back()
                .unwrap_or("");

            // Validate: function name must start with a letter or underscore
            // and contain only alphanumeric characters and underscores
            let is_valid_name = !func_name.is_empty()
                && func_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                && func_name.chars().all(|c| c.is_alphanumeric() || c == '_');

            if is_valid_name {
                // Create a stub native function marker
                ffi_table.insert(
                    func_name.to_string(),
                    Value::NativeFunction {
                        name: format!("ffi.{func_name}"),
                        arity: 0, // Variable arity for FFI functions
                    },
                );
            }
        }
    }

    Ok(Value::Table(Rc::new(RefCell::new(ffi_table))))
}

/// Native function: ffi.new_cstr(string: String) -> External
///
/// Creates a C-compatible string (null-terminated) from a Luma string.
/// Returns an External value that can be passed to C functions expecting
/// a `const char*` argument.
///
/// Example:
/// ```luma
/// let c_str = ffi.new_cstr("Hello, World!")
/// c_def.puts(c_str)
/// ```
///
/// Note: In a full implementation, this would allocate memory for the
/// C string and return a pointer to it. Memory management would need
/// to be handled carefully to avoid leaks.
pub fn native_ffi_new_cstr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "ffi.new_cstr() expects 1 argument, got {}",
            args.len()
        ));
    }

    // Validate argument is a string
    let _string_value = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ffi.new_cstr() argument must be a string".to_string()),
    };

    // In a full implementation, we would:
    // 1. Allocate memory for the null-terminated C string
    // 2. Copy the string data
    // 3. Return a pointer wrapped in External
    //
    // For now, we just create an External with a unique handle
    // that could be used to look up the string value when needed
    let handle = new_external_handle();

    // Store the string in a global registry (in a real implementation)
    // For now, just return the External value
    Ok(Value::External {
        handle,
        type_name: "cstr".to_string(),
    })
}

/// Native function: ffi.nullptr() -> External
///
/// Returns a null pointer (handle = 0) that can be used in FFI calls
/// where a null pointer is expected.
pub fn native_ffi_nullptr(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::External {
        handle: 0,
        type_name: "null".to_string(),
    })
}

/// Native function: ffi.is_null(ptr: External) -> Boolean
///
/// Checks if an External value represents a null pointer.
pub fn native_ffi_is_null(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "ffi.is_null() expects 1 argument, got {}",
            args.len()
        ));
    }

    match &args[0] {
        Value::External { handle, .. } => Ok(Value::Boolean(*handle == 0)),
        Value::Null => Ok(Value::Boolean(true)),
        _ => Err("ffi.is_null() argument must be an External or null".to_string()),
    }
}

/// Native function: ffi.call(def: Table, func_name: String, ...args) -> Result
///
/// Calls an FFI function with the given arguments.
/// This is the low-level function that actually invokes the foreign function.
///
/// Note: This is a stub that always returns an error indicating FFI is not
/// fully implemented. A real implementation would use libffi to call the function.
pub fn native_ffi_call(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("ffi.call() expects at least 2 arguments (def, func_name)".to_string());
    }

    let _def = match &args[0] {
        Value::Table(_) => &args[0],
        _ => return Err("ffi.call() first argument must be an FFI definition table".to_string()),
    };

    let func_name = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("ffi.call() second argument must be a function name string".to_string()),
    };

    // In a full implementation, this would:
    // 1. Look up the function in the def table
    // 2. Marshal the Luma arguments to C types
    // 3. Call the function using libffi
    // 4. Marshal the return value back to Luma
    //
    // For now, return an error indicating this is not implemented
    Ok(make_result_err(format!(
        "FFI call to '{}' not implemented. Full FFI requires libffi integration.",
        func_name
    )))
}

/// Create the `ffi` module table with all FFI functions
pub fn create_ffi_module() -> Value {
    let mut ffi_table: HashMap<String, Value> = HashMap::new();

    // Note: The actual native function implementations are registered
    // in the VM, so we just create the module structure here
    ffi_table.insert(
        "def".to_string(),
        Value::NativeFunction {
            name: "ffi.def".to_string(),
            arity: 1,
        },
    );

    ffi_table.insert(
        "new_cstr".to_string(),
        Value::NativeFunction {
            name: "ffi.new_cstr".to_string(),
            arity: 1,
        },
    );

    ffi_table.insert(
        "nullptr".to_string(),
        Value::NativeFunction {
            name: "ffi.nullptr".to_string(),
            arity: 0,
        },
    );

    ffi_table.insert(
        "is_null".to_string(),
        Value::NativeFunction {
            name: "ffi.is_null".to_string(),
            arity: 1,
        },
    );

    ffi_table.insert(
        "call".to_string(),
        Value::NativeFunction {
            name: "ffi.call".to_string(),
            arity: 0, // Variable arity
        },
    );

    Value::Table(Rc::new(RefCell::new(ffi_table)))
}
