use std::{fs, path::{PathBuf, Path}};

#[test]
fn test_should_fail_programs() {
    let should_fail_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/should_fail");

    fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("Failed to read should_fail tests directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() { collect(&path, files); continue; }
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
            failures.push(format!("❌ {}: Missing .expect file", test_name));
            continue;
        };

        // Parse
        let ast = match luma::parser::parse(source.as_str()) {
            Ok(ast) => {
                if expected_failure == "parse" {
                    println!("✓ {} (expected parse failure, got success - FAIL)", test_name);
                    failures.push(format!("❌ {}: Expected parse failure but parsing succeeded", test_name));
                    continue;
                }
                ast
            }
            Err(_errors) => {
                if expected_failure == "parse" {
                    println!("✓ {} (parse failed as expected)", test_name);
                    continue;
                } else {
                    failures.push(format!(
                        "❌ {}: Unexpected parse failure (expected {} failure)",
                        test_name, expected_failure
                    ));
                    continue;
                }
            }
        };

        // Typecheck
        match luma::typecheck::typecheck_program(&ast) {
            Ok(_) => {
                if expected_failure == "typecheck" {
                    failures.push(format!("❌ {}: Expected typecheck failure but typechecking succeeded", test_name));
                    continue;
                } else {
                    // Continue to runtime
                }
            }
            Err(_errs) => {
                if expected_failure == "typecheck" {
                    println!("✓ {} (typecheck failed as expected)", test_name);
                    continue;
                } else {
                    failures.push(format!(
                        "❌ {}: Unexpected typecheck failure (expected {} failure)",
                        test_name, expected_failure
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
                    failures.push(format!("❌ {}: Expected runtime failure but execution succeeded", test_name));
                } else {
                    failures.push(format!(
                        "❌ {}: Test succeeded but expected {} failure",
                        test_name, expected_failure
                    ));
                }
            }
            Err(_e) => {
                if expected_failure == "runtime" {
                    println!("✓ {} (runtime failed as expected)", test_name);
                } else {
                    failures.push(format!(
                        "❌ {}: Unexpected runtime failure (expected {} failure)",
                        test_name, expected_failure
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n{} should_fail test(s) had issues:\n\n{}", failures.len(), failures.join("\n"));
    }
}
