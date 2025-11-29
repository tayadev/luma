use std::fs;
use walkdir::WalkDir;

fn main() {
    let fixtures_dir = "tests/fixtures"; // Only parser test fixtures, not runtime
    let mut updated = 0;
    let mut failed = 0;

    for entry in WalkDir::new(fixtures_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("luma") {
            let ron_path = path.with_extension("ron");

            // Read source
            let source_raw = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", path.display(), e);
                    failed += 1;
                    continue;
                }
            };
            // Normalize Windows CRLF line endings for parser (same as test does)
            let source = source_raw.replace("\r\n", "\n").replace("\r", "\n");

            // Parse
            let program = match luma::parser::parse(&source, path.to_str().unwrap()) {
                Ok(p) => p,
                Err(errs) => {
                    eprintln!("Parse error in {}:", path.display());
                    for err in errs {
                        eprintln!("  {}", err.message);
                    }
                    failed += 1;
                    continue;
                }
            };

            // Serialize to RON
            let ron_string = match ron::ser::to_string_pretty(
                &program,
                ron::ser::PrettyConfig::new()
                    .depth_limit(100)
                    .extensions(ron::extensions::Extensions::IMPLICIT_SOME),
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to serialize {}: {}", path.display(), e);
                    failed += 1;
                    continue;
                }
            };

            // Write RON file
            if let Err(e) = fs::write(&ron_path, ron_string) {
                eprintln!("Failed to write {}: {}", ron_path.display(), e);
                failed += 1;
                continue;
            }

            updated += 1;
            println!("Updated {}", ron_path.display());
        }
    }

    println!("\nRegeneration complete:");
    println!("  Updated: {updated}");
    println!("  Failed: {failed}");
    println!("MAKE SURE TO MANUALLY CHECK THE FIXTURES FOR CORRECTNESS!");

    if failed > 0 {
        std::process::exit(1);
    }
}
