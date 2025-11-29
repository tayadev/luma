//! Operator overloading support for the Luma VM
//!
//! This module provides operator overloading functionality, allowing tables to
//! implement custom behavior for operators like `+`, `-`, `*`, `/`, `==`, `<`, etc.
//!
//! # Operator Methods
//!
//! Tables can define the following operator methods:
//! - `__add`, `__sub`, `__mul`, `__div`, `__mod` - Arithmetic operators
//! - `__neg` - Unary negation
//! - `__eq` - Equality comparison
//! - `__lt`, `__le`, `__gt`, `__ge` - Comparison operators
//!
//! Methods are looked up in two places:
//! 1. Directly on the table instance
//! 2. On the table's `__type` metadata (if present)
//!
//! This allows both per-instance behavior and type-level behavior.

use super::value::Value;
use super::{CallFrame, VM, VmError};
use std::collections::HashMap;

/// Check if a value is a table with a specific method
///
/// Checks both the value itself and its type definition (if it has __type metadata)
pub fn has_method(value: &Value, method_name: &str) -> Option<Value> {
    match value {
        Value::Table(table) => {
            let borrowed = table.borrow();

            // First check if the method exists directly on the value
            if let Some(method) = borrowed.get(method_name) {
                return Some(method.clone());
            }

            // If not, check the type definition (if value has __type metadata)
            // __type can be either Value::Type (created by cast()) or Value::Table (user-defined)
            if let Some(Value::Type(type_table) | Value::Table(type_table)) = borrowed.get("__type")
            {
                let type_borrowed = type_table.borrow();
                if let Some(method) = type_borrowed.get(method_name) {
                    return Some(method.clone());
                }
            }

            None
        }
        _ => None,
    }
}

/// Set up and execute a method call for operator overloading
///
/// This function:
/// 1. Validates the method is a function with the expected arity
/// 2. Saves the current call frame
/// 3. Pushes arguments onto the stack
/// 4. Sets up a new call frame for the method
/// 5. Returns control to the VM to execute the method
pub fn call_overload_method(
    vm: &mut VM,
    method: Value,
    args: Vec<Value>,
    expected_arity: usize,
    method_name: &str,
) -> Result<(), VmError> {
    match &method {
        Value::Function { chunk, arity } => {
            if *arity != expected_arity {
                return Err(VmError::runtime(format!(
                    "{method_name} method must have arity {expected_arity}, got {arity}"
                )));
            }

            // Save current frame
            let frame = CallFrame {
                chunk: vm.chunk.clone(),
                ip: vm.ip,
                base: vm.base,
                upvalues: vm.upvalues.clone(),
                captured_locals: std::mem::take(&mut vm.captured_locals),
            };
            vm.frames.push(frame);

            // Set up stack for function call
            vm.stack.push(method.clone());
            for arg in args {
                vm.stack.push(arg);
            }

            // Set new base to point to first argument
            vm.base = vm.stack.len() - expected_arity;
            // Switch to method chunk
            vm.chunk = chunk.clone();
            vm.ip = 0;
            // Methods are plain functions: reset upvalues and captured locals
            vm.upvalues = Vec::new();
            vm.captured_locals = HashMap::new();
            Ok(())
        }
        Value::Closure {
            chunk,
            arity,
            upvalues,
        } => {
            if *arity != expected_arity {
                return Err(VmError::runtime(format!(
                    "{method_name} method must have arity {expected_arity}, got {arity}"
                )));
            }

            // Save current frame
            let frame = CallFrame {
                chunk: vm.chunk.clone(),
                ip: vm.ip,
                base: vm.base,
                upvalues: vm.upvalues.clone(),
                captured_locals: std::mem::take(&mut vm.captured_locals),
            };
            vm.frames.push(frame);

            // Set up stack for function call
            vm.stack.push(method.clone());
            for arg in args {
                vm.stack.push(arg);
            }

            // Set new base to point to first argument
            vm.base = vm.stack.len() - expected_arity;
            // Switch to method chunk and restore closure upvalues
            vm.chunk = chunk.clone();
            vm.ip = 0;
            vm.upvalues = upvalues.clone();
            Ok(())
        }
        _ => Err(VmError::runtime(format!(
            "{method_name} must be a function"
        ))),
    }
}

/// Execute a binary operator with optional operator overloading
///
/// First tries the default operation (e.g., numeric addition).
/// If that fails, looks for an operator overload method on the left operand.
pub fn execute_binary_op(
    vm: &mut VM,
    a: Value,
    b: Value,
    method_name: &str,
    default_op: impl FnOnce(&Value, &Value) -> Result<Value, String>,
) -> Result<(), VmError> {
    // Try default operation first
    match default_op(&a, &b) {
        Ok(result) => {
            vm.stack.push(result);
            Ok(())
        }
        Err(_) => {
            // Try operator overloading
            if let Some(method) = has_method(&a, method_name) {
                call_overload_method(vm, method, vec![a, b], 2, method_name)
            } else {
                Err(VmError::runtime(format!(
                    "Operator {} not supported for {} and {} (no {} method)",
                    method_name,
                    value_type_name(&a),
                    value_type_name(&b),
                    method_name
                )))
            }
        }
    }
}

/// Execute equality comparison with operator overloading
///
/// Tables can define __eq method for custom equality semantics.
/// If no __eq method is found, uses default value equality (PartialEq).
pub fn execute_eq_op(vm: &mut VM, a: Value, b: Value) -> Result<(), VmError> {
    // Try operator overloading first for tables
    if let Some(method) = has_method(&a, "__eq") {
        call_overload_method(vm, method, vec![a, b], 2, "__eq")
    } else {
        // Default equality
        vm.stack.push(Value::Boolean(a == b));
        Ok(())
    }
}

/// Execute comparison operation with operator overloading
///
/// First tries numeric comparison. If operands are not both numbers,
/// looks for operator overload method on the left operand.
pub fn execute_cmp_op(
    vm: &mut VM,
    a: Value,
    b: Value,
    method_name: &str,
    default_cmp: impl FnOnce(f64, f64) -> bool,
) -> Result<(), VmError> {
    // Try default numeric comparison
    match (&a, &b) {
        (Value::Number(x), Value::Number(y)) => {
            vm.stack.push(Value::Boolean(default_cmp(*x, *y)));
            Ok(())
        }
        _ => {
            // Try operator overloading
            if let Some(method) = has_method(&a, method_name) {
                call_overload_method(vm, method, vec![a, b], 2, method_name)
            } else {
                Err(VmError::runtime(format!(
                    "Comparison {} requires numbers or {} method; got {} and {}",
                    method_name,
                    method_name,
                    value_type_name(&a),
                    value_type_name(&b)
                )))
            }
        }
    }
}

/// Get a human-readable type name for error messages
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Boolean(_) => "Boolean",
        Value::Null => "Null",
        Value::List(_) => "List",
        Value::Table(_) => "Table",
        Value::Function { .. } | Value::Closure { .. } | Value::NativeFunction { .. } => "Function",
        Value::Type(_) => "Type",
        Value::External { .. } => "External",
    }
}
