//! Tests for core native functions

use super::core::*;
use luma_core::vm::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn make_table() -> Value {
    Value::Table(Rc::new(RefCell::new(HashMap::new())))
}

fn make_type(fields: HashMap<String, Value>) -> Value {
    Value::Type(Rc::new(RefCell::new(fields)))
}

#[test]
fn test_native_cast_invalid_arg_count() {
    let result = native_cast(&[make_type(HashMap::new())]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_native_cast_non_type_first_arg() {
    let result = native_cast(&[Value::Number(42.0), make_table()]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be a type"));
}

#[test]
fn test_native_cast_non_table_value() {
    let result = native_cast(&[make_type(HashMap::new()), Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("can only cast table values"));
}

#[test]
fn test_native_cast_basic() {
    let mut type_fields = HashMap::new();
    type_fields.insert("x".to_string(), Value::Number(0.0));
    let type_def = make_type(type_fields);

    let mut table_fields = HashMap::new();
    table_fields.insert("x".to_string(), Value::Number(10.0));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let result = native_cast(&[type_def, table]);
    assert!(result.is_ok());

    if let Value::Table(cast_table) = result.unwrap() {
        let borrowed = cast_table.borrow();
        assert!(borrowed.contains_key("__type"));
        assert_eq!(borrowed.get("x"), Some(&Value::Number(10.0)));
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_is_instance_of_invalid_arg_count() {
    let result = native_is_instance_of(&[make_table()]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_native_is_instance_of_non_type_second_arg() {
    let result = native_is_instance_of(&[make_table(), Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be a type"));
}

#[test]
fn test_native_is_instance_of_non_table_value() {
    let result = native_is_instance_of(&[Value::Number(42.0), make_type(HashMap::new())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_native_is_instance_of_no_type() {
    let table = make_table();
    let type_def = make_type(HashMap::new());
    let result = native_is_instance_of(&[table, type_def]);
    assert!(result.is_ok());
    // Structural matching with empty type should return true
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_native_is_instance_of_matches() {
    let type_def = Rc::new(RefCell::new(HashMap::new()));
    let type_val = Value::Type(type_def.clone());

    let mut table_fields = HashMap::new();
    table_fields.insert("__type".to_string(), Value::Type(type_def.clone()));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let result = native_is_instance_of(&[table, type_val]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_native_into_invalid_arg_count() {
    let result = native_into(&[Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_native_into_non_type_second_arg() {
    let result = native_into(&[Value::Number(42.0), Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be a type"));
}

#[test]
fn test_native_into_number_to_string() {
    let mut type_fields = HashMap::new();
    type_fields.insert("String".to_string(), Value::Boolean(true));
    let string_type = make_type(type_fields);

    let result = native_into(&[Value::Number(42.0), string_type]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("42".to_string()));
}

#[test]
fn test_native_into_boolean_to_string() {
    let mut type_fields = HashMap::new();
    type_fields.insert("String".to_string(), Value::Boolean(true));
    let string_type = make_type(type_fields);

    let result = native_into(&[Value::Boolean(true), string_type]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("true".to_string()));
}

#[test]
fn test_native_into_null_to_string() {
    let mut type_fields = HashMap::new();
    type_fields.insert("String".to_string(), Value::Boolean(true));
    let string_type = make_type(type_fields);

    let result = native_into(&[Value::Null, string_type]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("null".to_string()));
}

#[test]
fn test_native_into_string_to_string() {
    let mut type_fields = HashMap::new();
    type_fields.insert("String".to_string(), Value::Boolean(true));
    let string_type = make_type(type_fields);

    let result = native_into(&[Value::String("hello".to_string()), string_type]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_native_into_unsupported_conversion() {
    // Empty type map is treated as String type
    let type_def = make_type(HashMap::new());
    let result = native_into(&[Value::Number(42.0), type_def]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("42".to_string()));
}

#[test]
fn test_native_typeof_invalid_arg_count() {
    let result = native_typeof(&[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 1 argument"));
}

#[test]
fn test_native_typeof_number() {
    let result = native_typeof(&[Value::Number(42.0)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Number".to_string()));
}

#[test]
fn test_native_typeof_string() {
    let result = native_typeof(&[Value::String("hello".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("String".to_string()));
}

#[test]
fn test_native_typeof_boolean() {
    let result = native_typeof(&[Value::Boolean(true)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Boolean".to_string()));
}

#[test]
fn test_native_typeof_null() {
    let result = native_typeof(&[Value::Null]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Null".to_string()));
}

#[test]
fn test_native_typeof_list() {
    let result = native_typeof(&[Value::List(Rc::new(RefCell::new(vec![])))]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("List".to_string()));
}

#[test]
fn test_native_typeof_table() {
    let result = native_typeof(&[make_table()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Table".to_string()));
}

#[test]
fn test_native_typeof_function() {
    use luma_core::bytecode::ir::Chunk;
    let result = native_typeof(&[Value::Function {
        arity: 0,
        chunk: Chunk::new_empty("test".to_string()),
    }]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Function".to_string()));
}

#[test]
fn test_native_iter_invalid_arg_count() {
    let result = native_iter(&[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 1 argument"));
}

#[test]
fn test_native_iter_list() {
    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ])));

    let result = native_iter(&[list.clone()]);
    assert!(result.is_ok());

    // Should return the same list (no copy)
    if let Value::List(result_list) = result.unwrap() {
        if let Value::List(original_list) = list {
            assert!(Rc::ptr_eq(&result_list, &original_list));
        }
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_native_iter_invalid_type() {
    let result = native_iter(&[Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires a List or Table"));
}
