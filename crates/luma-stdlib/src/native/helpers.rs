//! Helper functions for native function implementations.

use luma_core::vm::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Create a Result value with ok field set
pub fn make_result_ok(value: Value) -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), value);
    map.insert("err".to_string(), Value::Null);
    Value::Table(Rc::new(RefCell::new(map)))
}

/// Create a Result value with err field set
pub fn make_result_err(error: String) -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), Value::Null);
    map.insert("err".to_string(), Value::String(error));
    Value::Table(Rc::new(RefCell::new(map)))
}

/// Helper to extract type definition from a Value (either Table or Type)
pub fn get_type_map(value: &Value) -> Option<Rc<RefCell<HashMap<String, Value>>>> {
    match value {
        Value::Type(t) => Some(t.clone()),
        Value::Table(t) => Some(t.clone()),
        _ => None,
    }
}

/// Helper function to check if a value has all required fields for a type (for trait matching)
pub fn has_required_fields(
    value: &Value,
    type_def: &HashMap<String, Value>,
) -> Result<bool, String> {
    match value {
        Value::Table(table) => {
            let borrowed = table.borrow();

            // Check all required fields in the type definition
            for (field_name, field_type) in type_def.iter() {
                // Skip special fields like __parent and methods (functions)
                if field_name.starts_with("__") {
                    continue;
                }

                // If the field type is a function, it's a method - skip validation for methods
                if matches!(
                    field_type,
                    Value::Function { .. } | Value::NativeFunction { .. }
                ) {
                    continue;
                }

                // Check if the field exists in the value
                if !borrowed.contains_key(field_name) {
                    return Ok(false);
                }

                // For now, we do structural matching - just check if field exists
                // Full type checking would require recursively validating field types
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Helper function to check if a value is compatible for casting
/// For casting, we allow missing fields (they'll be filled with defaults)
pub fn is_castable(value: &Value) -> bool {
    matches!(value, Value::Table(_))
}

/// Helper function to merge parent fields into child
pub fn merge_parent_fields(type_def: &HashMap<String, Value>) -> HashMap<String, Value> {
    let mut merged = type_def.clone();

    // Check if there's a __parent field
    if let Some(parent_val) = type_def.get("__parent")
        && let Some(parent_map) = get_type_map(parent_val)
    {
        let parent_borrowed = parent_map.borrow();

        // Recursively merge parent's fields
        let parent_merged = merge_parent_fields(&parent_borrowed);

        // Add parent fields that don't exist in child
        for (key, value) in parent_merged.iter() {
            if !merged.contains_key(key) {
                merged.insert(key.clone(), value.clone());
            }
        }
    }

    merged
}
