//! Tests for CLI utilities

#[cfg(test)]
mod tests {
    use super::super::utils::*;
    use std::fs;

    #[test]
    fn test_read_source_from_file() {
        let temp_path = std::env::temp_dir().join("luma_cli_test_read.txt");
        let content = "let x = 42;";
        fs::write(&temp_path, content).unwrap();

        let result = read_source(temp_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_read_source_nonexistent_file() {
        let result = read_source("/nonexistent/file/path.luma");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_source_empty_file() {
        let temp_path = std::env::temp_dir().join("luma_cli_test_empty.txt");
        fs::write(&temp_path, "").unwrap();

        let result = read_source(temp_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }
}
