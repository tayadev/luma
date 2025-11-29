use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use std::path::{Path, PathBuf};

fn collect_luma_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if root.is_dir() {
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("luma") {
                files.push(p.to_path_buf());
            }
        }
    }
    files
}

fn sanitize(name: &str) -> String {
    name.replace('\\', "/")
}

fn bench_parser(c: &mut Criterion) {
    let fixtures_dir = Path::new("tests/fixtures");
    let files = collect_luma_files(fixtures_dir);
    let mut group = c.benchmark_group("parser_categories");
    group.sample_size(30); // reduce runtime & memory pressure

    // Group files by top-level directory (category)
    use std::collections::BTreeMap;
    let mut by_category: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for file in files {
        let rel = file.strip_prefix(fixtures_dir).unwrap_or(&file);
        let rel_str = rel.to_string_lossy().to_string();
        let parts: Vec<_> = rel
            .iter()
            .map(|c| c.to_string_lossy().to_string())
            .collect();
        let category = if parts.len() > 1 {
            parts[0].clone()
        } else {
            "root".to_string()
        };
        let src = fs::read_to_string(&file).expect("read fixture");
        by_category
            .entry(category)
            .or_default()
            .push((rel_str, src));
    }

    for (category, entries) in by_category {
        group.bench_function(format!("parse_category/{}", sanitize(&category)), |b| {
            b.iter(|| {
                for (name, src) in &entries {
                    let program =
                        luma::parser::parse(src, &format!("<bench:{name}>")).expect("parse ok");
                    black_box(&program);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
