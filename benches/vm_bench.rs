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
                // Skip module import tests - they require working directory context
                let skip = p.components().any(|c| {
                    c.as_os_str() == "modules" || c.as_os_str().to_string_lossy().contains("import")
                });
                if !skip {
                    files.push(p.to_path_buf());
                }
            }
        }
    }
    files
}

fn sanitize(name: &str) -> String {
    name.replace('\\', "/")
}

fn bench_vm(c: &mut Criterion) {
    let fixtures_dir = Path::new("tests/runtime");
    let files = collect_luma_files(fixtures_dir);

    // Benchmark individually for more granular results
    let mut group = c.benchmark_group("vm");
    group.sample_size(20);

    for file in files {
        let name = file
            .strip_prefix(fixtures_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .to_string();
        let src = fs::read_to_string(&file).expect("read runtime fixture");
        group.bench_function(format!("execute/{}", sanitize(&name)), |b| {
            b.iter(|| {
                let program =
                    luma::parser::parse(&src, &format!("<bench:{name}>")).expect("parse ok");
                let bytecode = luma::bytecode::compile::compile_program(&program);
                let mut vm = luma::vm::VM::new(bytecode);
                let result = vm.run().expect("vm ok");
                black_box(&result);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_vm);
criterion_main!(benches);
