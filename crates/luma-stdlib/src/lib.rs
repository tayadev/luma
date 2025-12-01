pub mod native;

use luma_core::vm::{VM, value::Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The Luma standard library prelude source code.
/// This is automatically loaded before user code runs.
pub const PRELUDE: &str = include_str!("prelude.luma");

// Re-export native functions for convenience
pub use native::*;

/// Initialize a VM with the standard library (native functions + prelude).
/// This registers all native functions, globals, and loads the prelude.
pub fn init_vm(mut vm: VM) -> Result<VM, luma_core::vm::VmError> {
    // Set FFI dispatch function
    vm.ffi_dispatch = Some(native_ffi_dispatch);

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
    vm.load_prelude(PRELUDE)?;

    Ok(vm)
}

/// Execute a Luma program with the standard library loaded.
/// This is a convenience function that creates a pipeline, compiles, and runs with stdlib.
pub fn run_program(
    source: String,
    filename: String,
) -> Result<Value, luma_core::pipeline::PipelineError> {
    use luma_core::bytecode::ir::Chunk;
    use luma_core::pipeline::Pipeline;

    let pipeline = Pipeline::new(source, filename.clone());
    let ast = pipeline.parse()?;
    pipeline.typecheck(&ast)?;
    let chunk = pipeline.compile(&ast);

    // Create and initialize VM with stdlib
    let vm_chunk = Chunk::new_empty(filename.clone());
    let vm = VM::new_with_file(vm_chunk, Some(filename));
    let mut vm = init_vm(vm).map_err(luma_core::pipeline::PipelineError::Runtime)?;

    // Execute with the initialized VM
    pipeline.execute_with_vm(chunk, &mut vm)
}
