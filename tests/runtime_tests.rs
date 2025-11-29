use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn test_runtime_programs() {
    let runtime_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/runtime");

    fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("Failed to read runtime tests directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                collect(&path, files);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) == Some("luma") {
                files.push(path.clone());
            }
        }
    }

    let mut files = Vec::new();
    collect(&runtime_dir, &mut files);
    files.sort();

    let mut failures = Vec::new();

    for luma_path in files {
        let test_name = luma_path.file_stem().unwrap().to_str().unwrap().to_string();

        let source_raw = fs::read_to_string(&luma_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", luma_path.display(), e));
        let source = source_raw.replace("\r\n", "\n").replace("\r", "\n");

        // Parse
        let ast = match luma::parser::parse(source.as_str(), luma_path.to_str().unwrap()) {
            Ok(ast) => ast,
            Err(errors) => {
                failures.push(format!(
                    "❌ {}: Parse failed with errors:\n{}",
                    test_name,
                    errors
                        .iter()
                        .map(|e| format!("  {e}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
                continue;
            }
        };

        // Typecheck (MVP stubbed OK)
        if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
            failures.push(format!(
                "❌ {}: Typecheck failed:\n{}",
                test_name,
                errs.iter()
                    .map(|e| format!("  {}", e.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
            continue;
        }

        // Compile
        let chunk = luma::bytecode::compile::compile_program(&ast);

        // Get absolute path for the test file
        let absolute_path = luma_path
            .canonicalize()
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        // Run
        let mut vm = luma::vm::VM::new_with_file(chunk, absolute_path);
        let value = match vm.run() {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("❌ {test_name}: Runtime error: {e:?}"));
                continue;
            }
        };

        // Load expected RON if present
        let ron_path = luma_path.with_extension("ron");
        if ron_path.exists() {
            let expected_ron = fs::read_to_string(&ron_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", ron_path.display(), e));
            let expected_val: luma::vm::value::Value = match ron::from_str(&expected_ron) {
                Ok(v) => v,
                Err(e) => {
                    failures.push(format!("❌ {test_name}: Failed to parse expected RON: {e}"));
                    continue;
                }
            };
            if value != expected_val {
                failures.push(format!(
                    "❌ {test_name}: Value mismatch\nExpected:\n{expected_ron}\n\nActual:\n{value:?}\n"
                ));
            } else {
                println!("✓ {test_name}");
            }
        } else {
            // No expectation: only ensure it ran
            println!("✓ {test_name} (no expectation)");
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n{} runtime test(s) failed:\n\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
