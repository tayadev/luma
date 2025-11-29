use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn test_should_fail_programs() {
    let should_fail_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/should_fail");

    fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("Failed to read should_fail tests directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                // Skip helpers directory
                if path.file_name().and_then(|s| s.to_str()) == Some("helpers") {
                    continue;
                }
                collect(&path, files);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) == Some("luma") {
                files.push(path.clone());
            }
        }
    }

    let mut files = Vec::new();
    collect(&should_fail_dir, &mut files);
    files.sort();

    let mut failures = Vec::new();

    for luma_path in files {
        let test_name = luma_path.file_stem().unwrap().to_str().unwrap().to_string();

        let source_raw = fs::read_to_string(&luma_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", luma_path.display(), e));
        let source = source_raw.replace("\r\n", "\n").replace("\r", "\n");

        // Load expected failure type
        let expect_path = luma_path.with_extension("expect");
        let expected_failure = if expect_path.exists() {
            fs::read_to_string(&expect_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", expect_path.display(), e))
                .trim()
                .to_lowercase()
        } else {
            failures.push(format!("❌ {test_name}: Missing .expect file"));
            continue;
        };

        // Parse
        let ast = match luma::parser::parse(source.as_str(), luma_path.to_str().unwrap()) {
            Ok(ast) => {
                if expected_failure == "parse" {
                    println!("✓ {test_name} (expected parse failure, got success - FAIL)");
                    failures.push(format!(
                        "❌ {test_name}: Expected parse failure but parsing succeeded"
                    ));
                    continue;
                }
                ast
            }
            Err(_errors) => {
                if expected_failure == "parse" {
                    println!("✓ {test_name} (parse failed as expected)");
                    continue;
                } else {
                    failures.push(format!(
                        "❌ {test_name}: Unexpected parse failure (expected {expected_failure} failure)"
                    ));
                    continue;
                }
            }
        };

        // Typecheck
        match luma::typecheck::typecheck_program(&ast) {
            Ok(_) => {
                if expected_failure == "typecheck" {
                    failures.push(format!(
                        "❌ {test_name}: Expected typecheck failure but typechecking succeeded"
                    ));
                    continue;
                } else {
                    // Continue to runtime
                }
            }
            Err(_errs) => {
                if expected_failure == "typecheck" {
                    println!("✓ {test_name} (typecheck failed as expected)");
                    continue;
                } else {
                    failures.push(format!(
                        "❌ {test_name}: Unexpected typecheck failure (expected {expected_failure} failure)"
                    ));
                    continue;
                }
            }
        }

        // Compile
        let chunk = luma::bytecode::compile::compile_program(&ast);

        // Run
        let mut vm = luma::vm::VM::new(chunk);
        match vm.run() {
            Ok(_) => {
                if expected_failure == "runtime" {
                    failures.push(format!(
                        "❌ {test_name}: Expected runtime failure but execution succeeded"
                    ));
                } else {
                    failures.push(format!(
                        "❌ {test_name}: Test succeeded but expected {expected_failure} failure"
                    ));
                }
            }
            Err(_e) => {
                if expected_failure == "runtime" {
                    println!("✓ {test_name} (runtime failed as expected)");
                } else {
                    failures.push(format!(
                        "❌ {test_name}: Unexpected runtime failure (expected {expected_failure} failure)"
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n{} should_fail test(s) had issues:\n\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
