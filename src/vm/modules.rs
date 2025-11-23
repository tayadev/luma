//! Module loading and import resolution for the Luma VM
//!
//! This module handles:
//! - Resolving import paths (relative and absolute)
//! - Loading and caching modules
//! - Detecting circular dependencies
//! - Module-level execution in isolated VM context
//!
//! # Module Resolution
//!
//! Import paths are resolved as follows:
//! 1. Absolute paths are used as-is (after canonicalization)
//! 2. Relative paths are resolved relative to the current file's directory
//! 3. If no current file exists, relative to the current working directory
//!
//! # Caching
//!
//! Modules are cached after first load using their canonical path as the key.
//! This ensures each module is loaded exactly once, even if imported multiple times.
//!
//! # Circular Dependencies
//!
//! The module system tracks which modules are currently loading and detects
//! circular import chains, returning an error with the full cycle path.

use super::value::Value;
use super::{VM, VmError};
use std::fs;
use std::path::Path;

/// Resolve an import path to an absolute canonical path
///
/// # Arguments
/// * `path` - The import path (relative or absolute)
/// * `current_file` - The file from which the import is being made (if any)
///
/// # Returns
/// Canonical absolute path string, or error if path doesn't exist
pub fn resolve_import_path(path: &str, current_file: Option<&String>) -> Result<String, VmError> {
    let path_obj = Path::new(path);

    // If it's an absolute path, use it as-is
    if path_obj.is_absolute() {
        return Ok(path_obj
            .canonicalize()
            .map_err(|e| VmError::runtime(format!("Failed to resolve path '{}': {}", path, e)))?
            .to_string_lossy()
            .to_string());
    }

    // For relative paths, resolve relative to the current file
    let base_dir = if let Some(current_file) = current_file {
        Path::new(current_file)
            .parent()
            .ok_or_else(|| {
                VmError::runtime(format!("Invalid current file path: {}", current_file))
            })?
            .to_path_buf()
    } else {
        // No current file, use current working directory
        std::env::current_dir()
            .map_err(|e| VmError::runtime(format!("Failed to get current directory: {}", e)))?
    };

    let full_path = base_dir.join(path);

    // Canonicalize to get absolute path and resolve .. and .
    let canonical = full_path.canonicalize().map_err(|e| {
        VmError::runtime(format!("Failed to resolve import path '{}': {}", path, e))
    })?;

    Ok(canonical.to_string_lossy().to_string())
}

/// Load and execute a module, returning its result value
///
/// This function:
/// 1. Marks the module as "loading" for circular dependency detection
/// 2. Reads and parses the module source
/// 3. Typechecks the module
/// 4. Compiles to bytecode
/// 5. Executes in a new VM with shared module cache
/// 6. Caches the result
/// 7. Unmarks the module as loading
///
/// # Arguments
/// * `vm` - The VM instance making the import (for shared state)
/// * `path` - Canonical path to the module file
///
/// # Returns
/// The value returned by the module's execution
pub fn load_module(vm: &mut VM, path: &str) -> Result<Value, VmError> {
    // Mark module as loading (for circular dependency detection)
    vm.loading_modules.borrow_mut().push(path.to_string());

    // Ensure we always unmark the module, even on error
    let result = (|| {
        // Read the module source
        let source = fs::read_to_string(path)
            .map_err(|e| VmError::runtime(format!("Failed to read module '{}': {}", path, e)))?;

        // Parse the module
        let ast = crate::parser::parse(&source, path).map_err(|errors| {
            VmError::runtime(format!(
                "Parse error in module '{}': {}",
                path,
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

        // Typecheck the module (if enabled)
        crate::typecheck::typecheck_program(&ast).map_err(|errs| {
            VmError::runtime(format!(
                "Typecheck error in module '{}': {}",
                path,
                errs.iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

        // Compile the module
        let chunk = crate::bytecode::compile::compile_program(&ast);

        // Create a new VM for the module with the module's path as current file
        let mut module_vm = VM::new_with_file(chunk, Some(path.to_string()));

        // Share the module cache and loading stack
        module_vm.module_cache = std::rc::Rc::clone(&vm.module_cache);
        module_vm.loading_modules = std::rc::Rc::clone(&vm.loading_modules);

        // Execute the module
        let module_value = module_vm
            .run()
            .map_err(|e| VmError::runtime(format!("Error executing module '{}': {:?}", path, e)))?;

        // Cache the module value
        vm.module_cache
            .borrow_mut()
            .insert(path.to_string(), module_value.clone());

        Ok(module_value)
    })();

    // Always unmark module as loading, even on error
    vm.loading_modules.borrow_mut().pop();

    result
}
