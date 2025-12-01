use criterion::{Criterion, black_box, criterion_group, criterion_main};
use luma::parser;

fn bench_parse_simple(c: &mut Criterion) {
    let source = "1 + 2";
    c.bench_function("parse simple expression", |b| {
        b.iter(|| parser::parse(black_box(source), "bench.luma"))
    });
}

fn bench_parse_arithmetic(c: &mut Criterion) {
    let source = "(1 + 2) * 3 - 4 / 5";
    c.bench_function("parse arithmetic expression", |b| {
        b.iter(|| parser::parse(black_box(source), "bench.luma"))
    });
}

fn bench_parse_function(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: Number): Number do
            if n <= 1 do
                return 1
            end
            return n * factorial(n - 1)
        end
    "#;
    c.bench_function("parse function definition", |b| {
        b.iter(|| parser::parse(black_box(source), "bench.luma"))
    });
}

fn bench_parse_medium_program(c: &mut Criterion) {
    let source = r#"
        fn fibonacci(n: Number): Number do
            if n <= 1 do
                return n
            end
            return fibonacci(n - 1) + fibonacci(n - 2)
        end
        
        let result = fibonacci(10)
        print(result)
    "#;
    c.bench_function("parse medium program", |b| {
        b.iter(|| parser::parse(black_box(source), "bench.luma"))
    });
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_arithmetic,
    bench_parse_function,
    bench_parse_medium_program
);
criterion_main!(benches);
