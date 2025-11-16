use std::{fs, path::{PathBuf, Path}};

#[test]
fn test_parser_fixtures() {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    
    // Recursively collect .luma/.ron fixture pairs
    fn collect(dir: &Path, fixtures: &mut Vec<(PathBuf, PathBuf)>) {
        for entry in fs::read_dir(dir).expect("Failed to read fixtures directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() { collect(&path, fixtures); continue; }
            if path.extension().and_then(|s| s.to_str()) == Some("luma") {
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let ron_path = path.parent().unwrap().join(format!("{}.ron", stem));
                if ron_path.exists() { fixtures.push((path.clone(), ron_path)); }
            }
        }
    }
    let mut fixtures = Vec::new();
    collect(&fixtures_dir, &mut fixtures);
    
    // Sort for consistent test ordering
    fixtures.sort_by(|a, b| a.0.cmp(&b.0));
    
    let mut failed_tests = Vec::new();
    
    for (luma_path, ron_path) in fixtures {
        let test_name = luma_path.file_stem().unwrap().to_str().unwrap();
        
        // Read the luma source
        let source_raw = fs::read_to_string(&luma_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", luma_path.display(), e));
        // Normalize Windows CRLF line endings for parser (treat CR as whitespace)
        let source = source_raw.replace("\r\n", "\n").replace("\r", "\n");
        
        // Read the expected RON output
        let expected_ron = fs::read_to_string(&ron_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", ron_path.display(), e));
        
        // Parse the source
        let ast = match luma::parser::parse(source.as_str()) {
            Ok(ast) => ast,
            Err(errors) => {
                failed_tests.push(format!(
                    "❌ {}: Parse failed with errors:\n{}",
                    test_name,
                    errors.iter()
                        .map(|e| format!("  {}", e))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
                continue;
            }
        };
        
        // Serialize to RON
        let actual_ron = ron::ser::to_string_pretty(&ast, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize AST to RON");
        
        // Parse both RON strings for comparison (to normalize formatting)
        let expected_ast: luma::ast::Program = ron::from_str(&expected_ron)
            .unwrap_or_else(|e| panic!("Failed to parse expected RON for {}: {}", test_name, e));
        
        if ast != expected_ast {
            failed_tests.push(format!(
                "❌ {}: AST mismatch\nExpected:\n{}\n\nActual:\n{}\n",
                test_name, expected_ron, actual_ron
            ));
        } else {
            println!("✓ {}", test_name);
        }
    }
    
    if !failed_tests.is_empty() {
        panic!("\n{} test(s) failed:\n\n{}", failed_tests.len(), failed_tests.join("\n"));
    }
}
