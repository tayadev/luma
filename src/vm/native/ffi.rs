//! FFI (Foreign Function Interface) native functions.
//!
//! This module provides the infrastructure for calling native C code from Luma.
//! The FFI system allows Luma code to:
//! - Define C type and function signatures via `ffi.def()`
//! - Create C strings from Luma strings via `ffi.new_cstr()`
//! - Call external C functions through the returned definition object
//!
//! This implementation uses libffi to dynamically call C functions.
//!
//! # Safety
//!
//! FFI is inherently unsafe. This module allows calling arbitrary C functions
//! with arbitrary arguments. Incorrect usage can lead to:
//! - Memory corruption
//! - Segmentation faults
//! - Security vulnerabilities
//!
//! Users should only use FFI with trusted C code and ensure proper type matching.

use crate::vm::value::Value;
use libffi::middle::{Arg, Builder, CodePtr, Type, arg};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(windows)]
use libloading::Library;

/// Global counter for generating unique external handles
static NEXT_EXTERNAL_HANDLE: AtomicUsize = AtomicUsize::new(1);

/// Global registry for C strings created via ffi.new_cstr()
/// Maps handle -> CString (keeps the CString alive)
static CSTRING_REGISTRY: std::sync::LazyLock<Mutex<HashMap<usize, CString>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Global registry for FFI function definitions
/// Maps function name -> FfiFunctionDef
static FFI_FUNCTIONS: std::sync::LazyLock<Mutex<HashMap<String, FfiFunctionDef>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// FFI function definition
#[derive(Clone, Debug)]
struct FfiFunctionDef {
    symbol_name: String,
    return_type: CType,
    param_types: Vec<CType>,
}

/// C type representation for FFI
#[derive(Clone, Debug, PartialEq)]
enum CType {
    Void,
    Int,
    Char,
    Pointer, // Generic pointer (void*, char*, FILE*, etc.)
}

impl CType {
    fn to_libffi_type(&self) -> Type {
        match self {
            CType::Void => Type::void(),
            CType::Int => Type::c_int(),
            CType::Char => Type::i8(),
            CType::Pointer => Type::pointer(),
        }
    }
}

/// Generate a new unique handle for an external resource
fn new_external_handle() -> usize {
    NEXT_EXTERNAL_HANDLE.fetch_add(1, Ordering::SeqCst)
}

/// Parse a simple C type string into CType
fn parse_c_type(type_str: &str) -> CType {
    let type_str = type_str.trim();

    // Check for pointer types (anything containing '*')
    if type_str.contains('*') {
        return CType::Pointer;
    }

    // Check for common types
    match type_str {
        "void" => CType::Void,
        "int" => CType::Int,
        "char" => CType::Char,
        _ => CType::Pointer, // Default unknown types to pointer
    }
}

/// Parse a C function declaration into parts
/// Returns (return_type, function_name, param_types)
fn parse_function_declaration(decl: &str) -> Option<(CType, String, Vec<CType>)> {
    let decl = decl.trim().trim_end_matches(';').trim();

    // Find the opening parenthesis
    let paren_pos = decl.find('(')?;
    let close_paren = decl.rfind(')')?;

    // Extract function name and return type
    let before_paren = &decl[..paren_pos];
    let params_str = &decl[paren_pos + 1..close_paren];

    // Parse return type and function name
    // Handle cases like "FILE *fopen" or "int fclose"
    let parts: Vec<&str> = before_paren.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let func_name = parts
        .last()?
        .trim_start_matches('*')
        .trim_end_matches('*')
        .to_string();

    // Return type is everything before the function name
    let return_type_str = if parts.len() > 1 {
        let last_word_start = before_paren.rfind(&func_name)?;
        before_paren[..last_word_start].trim()
    } else {
        "void"
    };

    let return_type = parse_c_type(return_type_str);

    // Parse parameters
    let mut param_types = Vec::new();
    if !params_str.trim().is_empty() && params_str.trim() != "void" {
        for param in params_str.split(',') {
            let param = param.trim();
            if !param.is_empty() {
                param_types.push(parse_c_type(param));
            }
        }
    }

    Some((return_type, func_name, param_types))
}

/// Load a C library symbol by name
fn get_libc_symbol(name: &str) -> Option<*const c_void> {
    // On Linux/Unix, we can use dlsym with RTLD_DEFAULT to find symbols in libc
    #[cfg(unix)]
    {
        let name_cstr = CString::new(name).ok()?;
        let ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name_cstr.as_ptr()) };
        if ptr.is_null() { None } else { Some(ptr) }
    }

    // On Windows, try to load symbols from common C runtime libraries
    #[cfg(windows)]
    {
        // List of C runtime libraries to try on Windows
        let lib_names = [
            "ucrtbase.dll",
            "msvcrt.dll",
            "api-ms-win-crt-stdio-l1-1-0.dll",
        ];

        for lib_name in &lib_names {
            if let Ok(lib) = unsafe { Library::new(lib_name) } {
                if let Ok(symbol) = unsafe { lib.get::<*const c_void>(name.as_bytes()) } {
                    let ptr = *symbol;
                    // Leak the library to keep it loaded (similar to dlopen without dlclose)
                    std::mem::forget(lib);
                    return Some(ptr);
                }
            }
        }
        None
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = name;
        None
    }
}

/// Native function: ffi.def(declarations: String) -> Table
///
/// Parses C-style type and function declarations and returns a table
/// containing callable wrappers for the declared functions.
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

    let mut ffi_table: HashMap<String, Value> = HashMap::new();

    // Store the raw declarations for debugging/introspection
    ffi_table.insert(
        "__declarations".to_string(),
        Value::String(declarations.clone()),
    );

    // Mark this as an FFI definition table
    ffi_table.insert("__ffi_def".to_string(), Value::Boolean(true));

    // Parse declarations and create function entries
    for line in declarations.lines() {
        let line = line.trim();
        // Skip typedef lines and empty lines
        if line.is_empty() || line.starts_with("typedef") {
            continue;
        }

        // Try to parse as a function declaration
        if let Some((return_type, func_name, param_types)) = parse_function_declaration(line) {
            // Validate function name
            let is_valid_name = !func_name.is_empty()
                && func_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                && func_name.chars().all(|c| c.is_alphanumeric() || c == '_');

            if is_valid_name {
                // Register the FFI function definition
                let ffi_key = format!("ffi.{func_name}");
                {
                    let mut ffi_funcs = FFI_FUNCTIONS.lock().unwrap();
                    ffi_funcs.insert(
                        ffi_key.clone(),
                        FfiFunctionDef {
                            symbol_name: func_name.clone(),
                            return_type,
                            param_types,
                        },
                    );
                }

                // Create a NativeFunction entry that will be dispatched to native_ffi_dispatch
                ffi_table.insert(
                    func_name.clone(),
                    Value::NativeFunction {
                        name: ffi_key,
                        arity: 0, // Variable arity for FFI functions
                    },
                );
            }
        }
    }

    Ok(Value::Table(Rc::new(RefCell::new(ffi_table))))
}

/// Dispatch function for FFI calls
/// This is called when any FFI function is invoked
pub fn native_ffi_dispatch(func_name: &str, args: &[Value]) -> Result<Value, String> {
    // Look up the function definition
    let func_def = {
        let ffi_funcs = FFI_FUNCTIONS.lock().unwrap();
        ffi_funcs.get(func_name).cloned()
    };

    let func_def = func_def.ok_or_else(|| format!("FFI function '{}' not found", func_name))?;

    // Get the symbol from libc
    let symbol_ptr = get_libc_symbol(&func_def.symbol_name)
        .ok_or_else(|| format!("Symbol '{}' not found in libc", func_def.symbol_name))?;

    // Build the CIF (Call Interface)
    let return_type = func_def.return_type.to_libffi_type();
    let param_types: Vec<Type> = func_def
        .param_types
        .iter()
        .map(|t| t.to_libffi_type())
        .collect();

    let cif = Builder::new().args(param_types).res(return_type).into_cif();

    // Convert Luma values to C arguments
    // We need to keep the data alive until after the call
    let mut ptr_storage: Vec<*const c_void> = Vec::new();
    let mut int_storage: Vec<libc::c_int> = Vec::new();
    let mut cstring_storage: Vec<CString> = Vec::new();

    for (i, (arg_val, param_type)) in args.iter().zip(func_def.param_types.iter()).enumerate() {
        match (arg_val, param_type) {
            (Value::External { handle, .. }, CType::Pointer) => {
                if *handle == 0 {
                    // Null pointer
                    ptr_storage.push(std::ptr::null());
                } else {
                    // Check if it's a cstring handle
                    let cstring_reg = CSTRING_REGISTRY.lock().unwrap();
                    if let Some(cstr) = cstring_reg.get(handle) {
                        ptr_storage.push(cstr.as_ptr() as *const c_void);
                    } else {
                        // Use handle as raw pointer
                        ptr_storage.push(*handle as *const c_void);
                    }
                }
            }
            (Value::String(s), CType::Pointer) => {
                // Create a temporary CString
                let cstr = CString::new(s.as_str())
                    .map_err(|_| "Invalid string for C: contains null byte".to_string())?;
                ptr_storage.push(cstr.as_ptr() as *const c_void);
                cstring_storage.push(cstr);
            }
            (Value::Number(n), CType::Int) => {
                int_storage.push(*n as libc::c_int);
            }
            (Value::Number(n), CType::Pointer) => {
                ptr_storage.push(*n as usize as *const c_void);
            }
            (Value::Null, CType::Pointer) => {
                ptr_storage.push(std::ptr::null());
            }
            _ => {
                return Err(format!(
                    "Type mismatch for argument {}: cannot convert {:?} to {:?}",
                    i, arg_val, param_type
                ));
            }
        }
    }

    // Build the argument list using indices
    let mut ffi_args: Vec<Arg> = Vec::new();
    let mut ptr_idx = 0usize;
    let mut int_idx = 0usize;

    for param_type in &func_def.param_types {
        match param_type {
            CType::Pointer => {
                ffi_args.push(arg(&ptr_storage[ptr_idx]));
                ptr_idx += 1;
            }
            CType::Int => {
                ffi_args.push(arg(&int_storage[int_idx]));
                int_idx += 1;
            }
            CType::Char => {
                // Char arguments are rare, treat as int for now
                ffi_args.push(arg(&int_storage[int_idx]));
                int_idx += 1;
            }
            CType::Void => {
                // Void parameters shouldn't happen
            }
        }
    }

    // Call the function
    let code_ptr = CodePtr::from_ptr(symbol_ptr as *const _);

    match func_def.return_type {
        CType::Pointer => {
            let result: *const c_void = unsafe { cif.call(code_ptr, &ffi_args) };
            if result.is_null() {
                Ok(Value::External {
                    handle: 0,
                    type_name: "ptr".to_string(),
                })
            } else {
                Ok(Value::External {
                    handle: result as usize,
                    type_name: "ptr".to_string(),
                })
            }
        }
        CType::Int => {
            let result: libc::c_int = unsafe { cif.call(code_ptr, &ffi_args) };
            Ok(Value::Number(result as f64))
        }
        CType::Void => {
            let _: () = unsafe { cif.call(code_ptr, &ffi_args) };
            Ok(Value::Null)
        }
        CType::Char => {
            let result: i8 = unsafe { cif.call(code_ptr, &ffi_args) };
            Ok(Value::Number(result as f64))
        }
    }
}

/// Native function: ffi.new_cstr(string: String) -> External
///
/// Creates a C-compatible string (null-terminated) from a Luma string.
/// Returns an External value that can be passed to C functions expecting
/// a `const char*` argument.
pub fn native_ffi_new_cstr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "ffi.new_cstr() expects 1 argument, got {}",
            args.len()
        ));
    }

    let string_value = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ffi.new_cstr() argument must be a string".to_string()),
    };

    // Create a CString and store it in the registry
    let cstr = CString::new(string_value.as_str())
        .map_err(|_| "String contains null byte, cannot convert to C string".to_string())?;

    let handle = new_external_handle();

    {
        let mut registry = CSTRING_REGISTRY.lock().unwrap();
        registry.insert(handle, cstr);
    }

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

/// Native function: ffi.free_cstr(ptr: External) -> Null
///
/// Frees a C string created by ffi.new_cstr()
pub fn native_ffi_free_cstr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "ffi.free_cstr() expects 1 argument, got {}",
            args.len()
        ));
    }

    match &args[0] {
        Value::External { handle, .. } => {
            let mut registry = CSTRING_REGISTRY.lock().unwrap();
            registry.remove(handle);
            Ok(Value::Null)
        }
        _ => Err("ffi.free_cstr() argument must be an External".to_string()),
    }
}

/// Native function: ffi.call(def: Table, func_name: String, ...args) -> Result
///
/// Calls an FFI function with the given arguments.
/// This is the low-level function that actually invokes the foreign function.
pub fn native_ffi_call(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("ffi.call() expects at least 2 arguments (def, func_name)".to_string());
    }

    let _def = match &args[0] {
        Value::Table(_) => &args[0],
        _ => return Err("ffi.call() first argument must be an FFI definition table".to_string()),
    };

    let func_name = match &args[1] {
        Value::String(s) => format!("ffi.{}", s),
        _ => return Err("ffi.call() second argument must be a function name string".to_string()),
    };

    let call_args = &args[2..];
    native_ffi_dispatch(&func_name, call_args)
}

/// Create the `ffi` module table with all FFI functions
pub fn create_ffi_module() -> Value {
    let mut ffi_table: HashMap<String, Value> = HashMap::new();

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
        "free_cstr".to_string(),
        Value::NativeFunction {
            name: "ffi.free_cstr".to_string(),
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
