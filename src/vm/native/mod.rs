//! Native function implementations for the Luma VM.
//!
//! This module contains all built-in functions that are implemented in Rust
//! rather than in Luma bytecode. Functions are organized into submodules:
//!
//! - `core`: Core runtime functions (cast, isInstanceOf, into, typeof, iter)
//! - `io`: Input/output functions (print, read_file, write_file, etc.)
//! - `ffi`: Foreign Function Interface for calling native C code
//! - `helpers`: Shared utilities for native function implementations

pub mod core;
pub mod ffi;
pub mod helpers;
pub mod io;

// Re-export all native functions for convenience
pub use core::{native_cast, native_into, native_is_instance_of, native_iter, native_typeof};
pub use ffi::{
    create_ffi_module, native_ffi_call, native_ffi_def, native_ffi_dispatch, native_ffi_free,
    native_ffi_free_cstr, native_ffi_is_null, native_ffi_new, native_ffi_new_cstr,
    native_ffi_nullptr,
};
pub use io::{
    native_file_exists, native_panic, native_print, native_read_file, native_write,
    native_write_file,
};
