//! Tests for helper functions

use super::helpers::*;
use luma_core::vm::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_make_result_ok() {
    let result = make_result_ok(Value::Number(42.0));

    if let Value::Table(map) = result {
        let borrowed = map.borrow();
        assert_eq!(borrowed.get("ok"), Some(&Value::Number(42.0)));
        assert_eq!(borrowed.get("err"), Some(&Value::Null));
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_make_result_err() {
    let result = make_result_err("Error message".to_string());

    if let Value::Table(map) = result {
        let borrowed = map.borrow();
        assert_eq!(borrowed.get("ok"), Some(&Value::Null));
        assert_eq!(
            borrowed.get("err"),
            Some(&Value::String("Error message".to_string()))
        );
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_get_type_map_from_type() {
    let map = Rc::new(RefCell::new(HashMap::new()));
    let type_val = Value::Type(map.clone());

    let result = get_type_map(&type_val);
    assert!(result.is_some());
    assert!(Rc::ptr_eq(&result.unwrap(), &map));
}

#[test]
fn test_get_type_map_from_table() {
    let map = Rc::new(RefCell::new(HashMap::new()));
    let table_val = Value::Table(map.clone());

    let result = get_type_map(&table_val);
    assert!(result.is_some());
    assert!(Rc::ptr_eq(&result.unwrap(), &map));
}

#[test]
fn test_get_type_map_from_non_type() {
    let result = get_type_map(&Value::Number(42.0));
    assert!(result.is_none());
}

#[test]
fn test_is_castable_table() {
    let table = Value::Table(Rc::new(RefCell::new(HashMap::new())));
    assert!(is_castable(&table));
}

#[test]
fn test_is_castable_non_table() {
    assert!(!is_castable(&Value::Number(42.0)));
    assert!(!is_castable(&Value::String("hello".to_string())));
    assert!(!is_castable(&Value::Boolean(true)));
    assert!(!is_castable(&Value::Null));
}

#[test]
fn test_has_required_fields_empty_type() {
    let mut table_fields = HashMap::new();
    table_fields.insert("x".to_string(), Value::Number(10.0));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let type_fields = HashMap::new();
    let result = has_required_fields(&table, &type_fields);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_has_required_fields_matching() {
    let mut table_fields = HashMap::new();
    table_fields.insert("x".to_string(), Value::Number(10.0));
    table_fields.insert("y".to_string(), Value::String("hello".to_string()));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let mut type_fields = HashMap::new();
    type_fields.insert("x".to_string(), Value::Number(0.0));
    type_fields.insert("y".to_string(), Value::String("".to_string()));

    let result = has_required_fields(&table, &type_fields);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_has_required_fields_missing() {
    let mut table_fields = HashMap::new();
    table_fields.insert("x".to_string(), Value::Number(10.0));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let mut type_fields = HashMap::new();
    type_fields.insert("x".to_string(), Value::Number(0.0));
    type_fields.insert("y".to_string(), Value::String("".to_string()));

    let result = has_required_fields(&table, &type_fields);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_has_required_fields_ignores_special_fields() {
    let mut table_fields = HashMap::new();
    table_fields.insert("x".to_string(), Value::Number(10.0));
    let table = Value::Table(Rc::new(RefCell::new(table_fields)));

    let mut type_fields = HashMap::new();
    type_fields.insert("x".to_string(), Value::Number(0.0));
    type_fields.insert("__parent".to_string(), Value::Null);

    let result = has_required_fields(&table, &type_fields);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_has_required_fields_non_table() {
    let type_fields = HashMap::new();
    let result = has_required_fields(&Value::Number(42.0), &type_fields);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_merge_parent_fields_no_parent() {
    let mut type_fields = HashMap::new();
    type_fields.insert("x".to_string(), Value::Number(10.0));

    let merged = merge_parent_fields(&type_fields);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged.get("x"), Some(&Value::Number(10.0)));
}

#[test]
fn test_merge_parent_fields_with_parent() {
    let mut parent_fields = HashMap::new();
    parent_fields.insert("a".to_string(), Value::Number(1.0));
    parent_fields.insert("b".to_string(), Value::Number(2.0));
    let parent_type = Value::Type(Rc::new(RefCell::new(parent_fields)));

    let mut child_fields = HashMap::new();
    child_fields.insert("b".to_string(), Value::Number(20.0)); // Override
    child_fields.insert("c".to_string(), Value::Number(3.0));
    child_fields.insert("__parent".to_string(), parent_type);

    let merged = merge_parent_fields(&child_fields);
    assert_eq!(merged.get("a"), Some(&Value::Number(1.0))); // Inherited
    assert_eq!(merged.get("b"), Some(&Value::Number(20.0))); // Overridden
    assert_eq!(merged.get("c"), Some(&Value::Number(3.0))); // Own field
}
