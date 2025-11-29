//! Virtual machine interpreter for Luma bytecode.
//!
//! This module implements a stack-based bytecode interpreter with support for:
//!
//! - **Stack management**: Values are pushed/popped from a stack during execution
//! - **Closures and upvalues**: Functions can capture variables from outer scopes
//! - **Operator overloading**: Custom types can define behavior for operators
//! - **Module system**: Import and caching of external modules
//! - **Native functions**: Built-in functions implemented in Rust
//! - **Call frames**: Function calls maintain their own execution context
//!
//! ## Execution Model
//!
//! The VM executes bytecode instructions in a loop, maintaining:
//! - A value stack for computation
//! - A call stack (frames) for function calls
//! - Global variables accessible from all scopes
//! - Upvalues for closure captures
//!
//! ## Error Handling
//!
//! Runtime errors are returned as `VmError` with optional source location information
//! for better error messages.

use super::errors::VmError;
use super::frames::CallFrame;
use super::native::*;
use super::value::{Upvalue, Value};
use crate::ast::Span;
use crate::bytecode::ir::Chunk;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Type alias for native function signatures
pub type NativeFunction = fn(&[Value]) -> Result<Value, String>;

/// The virtual machine that executes Luma bytecode
pub struct VM {
    pub stack: Vec<Value>,
    pub ip: usize,
    pub chunk: Chunk,
    pub globals: HashMap<String, Value>,
    pub base: usize,
    pub frames: Vec<CallFrame>,
    pub upvalues: Vec<Upvalue>,
    pub captured_locals: HashMap<usize, Upvalue>,
    pub native_functions: HashMap<String, NativeFunction>,
    pub module_cache: Rc<RefCell<HashMap<String, Value>>>,
    pub loading_modules: Rc<RefCell<Vec<String>>>,
    pub current_file: Option<String>,
    pub source: Option<String>,
}

impl VM {
    /// Create a new VM with the given chunk
    pub fn new(chunk: Chunk) -> Self {
        Self::new_with_file(chunk, None)
    }

    /// Create a new VM with the given chunk and file path
    pub fn new_with_file(chunk: Chunk, current_file: Option<String>) -> Self {
        let mut vm = VM {
            stack: Vec::new(),
            ip: 0,
            chunk,
            globals: HashMap::new(),
            base: 0,
            frames: Vec::new(),
            upvalues: Vec::new(),
            captured_locals: HashMap::new(),
            native_functions: HashMap::new(),
            module_cache: Rc::new(RefCell::new(HashMap::new())),
            loading_modules: Rc::new(RefCell::new(Vec::new())),
            current_file,
            source: None,
        };

        // Register native functions
        vm.register_native_function("cast", 2, native_cast);
        vm.register_native_function("isInstanceOf", 2, native_is_instance_of);
        vm.register_native_function("into", 2, native_into);
        vm.register_native_function("typeof", 1, native_typeof);
        vm.register_native_function("iter", 1, native_iter);
        vm.register_native_function("print", 0, native_print);

        // Register I/O functions
        vm.register_native_function("write", 2, native_write);
        vm.register_native_function("read_file", 1, native_read_file);
        vm.register_native_function("write_file", 2, native_write_file);
        vm.register_native_function("file_exists", 1, native_file_exists);

        // Register panic function
        vm.register_native_function("panic", 1, native_panic);

        // Register FFI functions
        vm.register_native_function("ffi.def", 1, native_ffi_def);
        vm.register_native_function("ffi.new_cstr", 1, native_ffi_new_cstr);
        vm.register_native_function("ffi.new", 1, native_ffi_new);
        vm.register_native_function("ffi.free", 1, native_ffi_free);
        vm.register_native_function("ffi.nullptr", 0, native_ffi_nullptr);
        vm.register_native_function("ffi.is_null", 1, native_ffi_is_null);
        vm.register_native_function("ffi.free_cstr", 1, native_ffi_free_cstr);
        vm.register_native_function("ffi.call", 0, native_ffi_call);

        // Register process functions
        vm.register_native_function("process.exit", 1, native_process_exit);

        // Expose file descriptor constants
        vm.globals.insert("STDOUT".to_string(), Value::Number(1.0));
        vm.globals.insert("STDERR".to_string(), Value::Number(2.0));

        // Expose ffi module
        vm.globals.insert("ffi".to_string(), create_ffi_module());

        // Expose process module
        vm.globals
            .insert("process".to_string(), create_process_module());

        // Expose type markers for into() conversions
        vm.globals.insert(
            "String".to_string(),
            Value::Type(Rc::new(RefCell::new({
                let mut t = HashMap::new();
                t.insert("String".to_string(), Value::Boolean(true));
                t
            }))),
        );

        // Expose External type marker
        vm.globals.insert(
            "External".to_string(),
            Value::External {
                handle: 0,
                type_name: "External".to_string(),
            },
        );

        // Load prelude
        if let Err(e) = vm.load_prelude() {
            eprintln!("Warning: Failed to load prelude: {e:?}");
        }

        vm
    }

    /// Set the source code for error reporting
    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }

    /// Get the current span based on IP
    pub(crate) fn _current_span(&self) -> Option<Span> {
        self.chunk.get_span(self.ip)
    }

    /// Create a runtime error with current location
    pub(crate) fn _error(&self, message: String) -> VmError {
        VmError::with_location(message, self._current_span(), self.current_file.clone())
    }

    /// Register a native function
    fn register_native_function(&mut self, name: &str, arity: usize, func: NativeFunction) {
        let native_val = Value::NativeFunction {
            name: name.to_string(),
            arity,
        };
        self.globals.insert(name.to_string(), native_val);
        self.native_functions.insert(name.to_string(), func);
    }

    /// Load and execute the prelude (standard library)
    fn load_prelude(&mut self) -> Result<(), VmError> {
        let prelude_source = include_str!("../prelude.luma");

        let ast = match crate::parser::parse(prelude_source, "<prelude>") {
            Ok(ast) => ast,
            Err(errors) => {
                let error_msg = errors
                    .iter()
                    .map(|e| format!("{e}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(VmError::runtime(format!(
                    "Failed to parse prelude: {error_msg}"
                )));
            }
        };

        // Compile prelude in REPL mode so variables become globals.
        // This ensures closures in the prelude (like those in iterator functions)
        // capture globals that persist for the lifetime of the VM.
        let prelude_chunk = crate::bytecode::compile::compile_repl_program(&ast);

        let saved_chunk = self.chunk.clone();
        let saved_ip = self.ip;
        let saved_base = self.base;

        self.chunk = prelude_chunk;
        self.ip = 0;
        self.base = 0;

        let result = self.run();

        self.chunk = saved_chunk;
        self.ip = saved_ip;
        self.base = saved_base;

        match result {
            Ok(prelude_exports) => {
                // Store the prelude exports table itself as 'prelude'
                self.globals
                    .insert("prelude".to_string(), prelude_exports.clone());

                // Load all exported values into global scope for direct access
                if let Value::Table(table_ref) = prelude_exports {
                    let table = table_ref.borrow();
                    for (name, value) in table.iter() {
                        self.globals.insert(name.clone(), value.clone());
                    }
                }
                Ok(())
            }
            Err(e) => Err(VmError::runtime(format!(
                "Failed to execute prelude: {e:?}"
            ))),
        }
    }

    /// Run the VM until completion
    pub fn run(&mut self) -> Result<Value, VmError> {
        self.execute()
    }

    /// Evaluate a chunk in the context of this VM's existing state
    pub fn eval(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        let saved_chunk = std::mem::replace(&mut self.chunk, chunk);
        let saved_ip = self.ip;
        let saved_base = self.base;

        self.ip = 0;
        self.base = 0;

        let result = self.run();

        self.chunk = saved_chunk;
        self.ip = saved_ip;
        self.base = saved_base;

        result
    }
}
