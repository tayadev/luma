//! Core native functions: cast, isInstanceOf, into, typeof, iter

use super::helpers::*;
use luma_core::vm::value::Value;
use std::rc::Rc;

/// Native function: cast(type, value) -> typed_value
pub fn native_cast(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("cast() expects 2 arguments, got {}", args.len()));
    }

    let type_def = match &args[0] {
        Value::Table(t) => t.clone(),
        Value::Type(t) => t.clone(),
        _ => return Err("cast() first argument must be a type (table)".to_string()),
    };

    let value = &args[1];

    // Check that the value is a table (castable)
    if !is_castable(value) {
        return Err("cast() can only cast table values".to_string());
    }

    // Merge inherited fields from parent types
    let type_borrowed = type_def.borrow();
    let merged_type = merge_parent_fields(&type_borrowed);

    // At this point, value is guaranteed to be a Table
    match value {
        Value::Table(table) => {
            let value_borrowed = table.borrow();
            let mut new_table = value_borrowed.clone();

            // Merge inherited fields from parent if any
            for (key, val) in merged_type.iter() {
                if !key.starts_with("__") && !new_table.contains_key(key) {
                    // Don't copy methods, only data fields
                    if !matches!(val, Value::Function { .. } | Value::NativeFunction { .. }) {
                        new_table.insert(key.clone(), val.clone());
                    }
                }
            }

            // Attach the type definition as metadata
            new_table.insert("__type".to_string(), Value::Type(type_def.clone()));

            Ok(Value::Table(Rc::new(std::cell::RefCell::new(new_table))))
        }
        _ => unreachable!("is_castable ensures value is a Table"),
    }
}

/// Native function: isInstanceOf(value, type) -> boolean
pub fn native_is_instance_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "isInstanceOf() expects 2 arguments, got {}",
            args.len()
        ));
    }

    let value = &args[0];
    let type_def = match &args[1] {
        Value::Table(t) => t.clone(),
        Value::Type(t) => t.clone(),
        _ => return Err("isInstanceOf() second argument must be a type (table)".to_string()),
    };

    // Check if the value is a table with __type metadata
    if let Value::Table(table) = value {
        let borrowed = table.borrow();

        // Check direct type match
        if let Some(Value::Type(value_type)) = borrowed.get("__type") {
            // Compare type references
            if Rc::ptr_eq(value_type, &type_def) {
                return Ok(Value::Boolean(true));
            }

            // Check if value_type inherits from type_def via __parent chain
            let mut current_type = value_type.clone();
            loop {
                let has_parent = {
                    let current_borrowed = current_type.borrow();
                    current_borrowed.get("__parent").cloned()
                };

                if let Some(parent_val) = has_parent {
                    if let Some(parent_map) = get_type_map(&parent_val) {
                        // Check if this parent matches the target type
                        if Rc::ptr_eq(&parent_map, &type_def) {
                            return Ok(Value::Boolean(true));
                        }
                        current_type = parent_map;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Fallback to structural matching (trait-like behavior)
        let type_borrowed = type_def.borrow();
        if has_required_fields(value, &type_borrowed)? {
            return Ok(Value::Boolean(true));
        }
    }

    Ok(Value::Boolean(false))
}

/// Native function: into(value, target_type) -> converted_value
/// Calls the __into method on the value with the target type
pub fn native_into(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("into() expects 2 arguments, got {}", args.len()));
    }

    let value = &args[0];
    let target_type = &args[1];

    // Check if value has __into method
    match value {
        Value::Table(table) => {
            let borrowed = table.borrow();
            if borrowed.get("__into").is_some() {
                // We need to call the __into method with (self, target_type)
                // But we can't easily call it from here without access to the VM
                // For now, return an error suggesting that __into calls must happen in VM context
                Err(
                    "Type conversions via __into are not fully implemented yet. \
                     For now, use explicit conversion methods or wait for v2. \
                     See GC_HOOKS.md for details."
                        .to_string(),
                )
            } else {
                Err("Type does not support conversion (no __into method)".to_string())
            }
        }
        _ => {
            // For non-table values, provide default conversions
            match target_type {
                Value::Type(type_map) | Value::Table(type_map) => {
                    let type_borrowed = type_map.borrow();
                    // Check if target is String type (basic heuristic)
                    // TODO: Improve type matching logic
                    if type_borrowed.contains_key("String") || type_borrowed.is_empty() {
                        // Default string conversion for primitive types
                        match value {
                            Value::Number(n) => Ok(Value::String(n.to_string())),
                            Value::String(s) => Ok(Value::String(s.clone())),
                            Value::Boolean(b) => Ok(Value::String(b.to_string())),
                            Value::Null => Ok(Value::String("null".to_string())),
                            _ => Err("Cannot convert value to String".to_string()),
                        }
                    } else {
                        Err("Unsupported conversion target".to_string())
                    }
                }
                _ => Err("Second argument to into() must be a type".to_string()),
            }
        }
    }
}

/// Native function: typeof(value: Any) -> String
/// Returns the runtime type name of a value
pub fn native_typeof(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("typeof() expects 1 argument, got {}", args.len()));
    }

    let type_name = match &args[0] {
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Boolean(_) => "Boolean",
        Value::Null => "Null",
        Value::List(_) => "List",
        Value::Table(table) => {
            // Check if this table has a __type field (from cast())
            let borrowed = table.borrow();
            if let Some(Value::Type(_)) = borrowed.get("__type") {
                // This is a typed instance, return "Table" as the base type
                // The actual type information is in the __type field
                "Table"
            } else {
                "Table"
            }
        }
        Value::Function { .. } => "Function",
        Value::Closure { .. } => "Function",
        Value::NativeFunction { .. } => "Function",
        Value::Type(_) => "Type",
        Value::External { .. } => "External",
    };

    Ok(Value::String(type_name.to_string()))
}

/// Native function: iter(value: List|Table) -> List
/// - List: returns the same list (no copy)
/// - Table: returns list of [key, value] pairs
pub fn native_iter(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("iter() expects 1 argument, got {}", args.len()));
    }

    match &args[0] {
        Value::List(list_rc) => Ok(Value::List(list_rc.clone())),
        Value::Table(map_rc) => {
            let map = map_rc.borrow();
            let mut out: Vec<Value> = Vec::with_capacity(map.len());
            for (k, v) in map.iter() {
                let pair = vec![Value::String(k.clone()), v.clone()];
                out.push(Value::List(Rc::new(std::cell::RefCell::new(pair))));
            }
            Ok(Value::List(Rc::new(std::cell::RefCell::new(out))))
        }
        _ => Err("iter() requires a List or Table".to_string()),
    }
}
