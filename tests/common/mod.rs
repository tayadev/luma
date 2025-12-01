// Common test utilities and fixture loading

use std::fs;
use std::path::PathBuf;

/// Load a test fixture file by name
pub fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("{}.luma", name));

    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", path.display()))
}
