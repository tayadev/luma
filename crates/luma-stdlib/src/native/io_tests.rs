//! Tests for I/O native functions

use super::io::*;
use luma_core::vm::value::Value;

#[test]
fn test_native_print_no_args() {
    let result = native_print(&[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_native_print_single_arg() {
    let result = native_print(&[Value::String("Hello".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_native_print_multiple_args() {
    let result = native_print(&[
        Value::String("Hello".to_string()),
        Value::Number(42.0),
        Value::Boolean(true),
    ]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_native_write_invalid_arg_count() {
    let result = native_write(&[Value::Number(1.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_native_write_non_number_fd() {
    let result = native_write(&[
        Value::String("not a number".to_string()),
        Value::String("content".to_string()),
    ]);
    assert!(result.is_ok());
    // Should return error result table
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("err"));
        if let Some(Value::String(err_msg)) = map.get("err") {
            assert!(err_msg.contains("must be a number"));
        }
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_valid_stdout() {
    let result = native_write(&[Value::Number(1.0), Value::String("test".to_string())]);
    assert!(result.is_ok());
    // Should return ok result
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("ok"));
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_valid_stderr() {
    let result = native_write(&[Value::Number(2.0), Value::String("error".to_string())]);
    assert!(result.is_ok());
    // Should return ok result
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("ok"));
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_invalid_fd() {
    let result = native_write(&[Value::Number(99.0), Value::String("test".to_string())]);
    assert!(result.is_ok());
    // Should return error result
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("err"));
        if let Some(Value::String(err_msg)) = map.get("err") {
            assert!(err_msg.contains("Invalid file descriptor"));
        }
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_converts_types() {
    // Test number conversion
    let result = native_write(&[Value::Number(1.0), Value::Number(42.0)]);
    assert!(result.is_ok());

    // Test boolean conversion
    let result = native_write(&[Value::Number(1.0), Value::Boolean(true)]);
    assert!(result.is_ok());

    // Test null conversion
    let result = native_write(&[Value::Number(1.0), Value::Null]);
    assert!(result.is_ok());
}

#[test]
fn test_native_read_file_invalid_arg_count() {
    let result = native_read_file(&[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 1 argument"));
}

#[test]
fn test_native_read_file_non_string_path() {
    let result = native_read_file(&[Value::Number(42.0)]);
    assert!(result.is_ok());
    // Should return error result table
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("err"));
        if let Some(Value::String(err_msg)) = map.get("err") {
            assert!(err_msg.contains("must be a string"));
        }
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_read_file_nonexistent() {
    let result = native_read_file(&[Value::String("/nonexistent/file.txt".to_string())]);
    assert!(result.is_ok());
    // Should return error result table
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("err"));
        if let Some(Value::String(err_msg)) = map.get("err") {
            assert!(err_msg.contains("Failed to read file"));
        }
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_file_invalid_arg_count() {
    let result = native_write_file(&[Value::String("test.txt".to_string())]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_native_write_file_non_string_path() {
    let result = native_write_file(&[Value::Number(42.0), Value::String("content".to_string())]);
    assert!(result.is_ok());
    // Should return error result table
    if let Value::Table(map) = result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("err"));
        if let Some(Value::String(err_msg)) = map.get("err") {
            assert!(err_msg.contains("must be a string"));
        }
    } else {
        panic!("Expected table result");
    }
}

#[test]
fn test_native_write_file_converts_content_types() {
    // These tests write to a temp file to ensure the conversion works
    let temp_path = std::env::temp_dir().join("luma_test_write.txt");
    let path_str = temp_path.to_str().unwrap().to_string();

    // Test number conversion
    let result = native_write_file(&[Value::String(path_str.clone()), Value::Number(42.0)]);
    assert!(result.is_ok());

    // Test boolean conversion
    let result = native_write_file(&[Value::String(path_str.clone()), Value::Boolean(true)]);
    assert!(result.is_ok());

    // Test null conversion
    let result = native_write_file(&[Value::String(path_str.clone()), Value::Null]);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_native_file_exists_invalid_arg_count() {
    let result = native_file_exists(&[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 1 argument"));
}

#[test]
fn test_native_file_exists_non_string_path() {
    let result = native_file_exists(&[Value::Number(42.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be a string"));
}

#[test]
fn test_native_file_exists_nonexistent() {
    let result = native_file_exists(&[Value::String("/nonexistent/file.txt".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_native_file_exists_existing() {
    // Create a temp file to test
    let temp_path = std::env::temp_dir().join("luma_test_exists.txt");
    std::fs::write(&temp_path, "test").unwrap();

    let path_str = temp_path.to_str().unwrap().to_string();
    let result = native_file_exists(&[Value::String(path_str)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));

    // Cleanup
    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_native_panic_invalid_arg_count() {
    let result = native_panic(&[]);
    assert!(result.is_err());
}

#[test]
fn test_read_write_file_roundtrip() {
    let temp_path = std::env::temp_dir().join("luma_test_roundtrip.txt");
    let path_str = temp_path.to_str().unwrap().to_string();
    let content = "Hello, Luma!";

    // Write the file
    let write_result = native_write_file(&[
        Value::String(path_str.clone()),
        Value::String(content.to_string()),
    ]);
    assert!(write_result.is_ok());
    if let Value::Table(map) = write_result.unwrap() {
        let map = map.borrow();
        assert!(map.contains_key("ok"));
    } else {
        panic!("Expected table result");
    }

    // Read the file back
    let read_result = native_read_file(&[Value::String(path_str.clone())]);
    assert!(read_result.is_ok());
    if let Value::Table(map) = read_result.unwrap() {
        let map = map.borrow();
        if let Some(Value::String(read_content)) = map.get("ok") {
            assert_eq!(read_content, content);
        } else {
            panic!("Expected ok result with string content");
        }
    } else {
        panic!("Expected table result");
    }

    // Cleanup
    let _ = std::fs::remove_file(temp_path);
}
