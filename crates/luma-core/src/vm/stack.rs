//! Stack manipulation utilities for the VM

use super::errors::VmError;
use super::value::Value;

/// Helper function to flip a boolean on the top of the stack
pub fn flip_bool(stack: &mut Vec<Value>) -> Result<(), VmError> {
    let v = stack
        .pop()
        .ok_or_else(|| VmError::runtime("flip bool underflow".into()))?;
    match v {
        Value::Boolean(b) => {
            stack.push(Value::Boolean(!b));
            Ok(())
        }
        _ => Err(VmError::runtime("flip bool type error".into())),
    }
}

/// Helper function to determine if a value is truthy
pub fn truthy(v: &Value) -> bool {
    !matches!(v, Value::Boolean(false) | Value::Null)
}
