use criterion::{Criterion, black_box, criterion_group, criterion_main};
use luma::bytecode::compile::compile_program;
use luma::parser;
use luma::vm::VM;

fn bench_vm_arithmetic(c: &mut Criterion) {
    let source = "1 + 2 * 3 - 4 / 2";
    let program = parser::parse(source, "bench.luma").unwrap();
    let chunk = compile_program(&program);

    c.bench_function("vm execute arithmetic", |b| {
        b.iter(|| {
            let mut vm = VM::new(chunk.clone());
            black_box(vm.run())
        })
    });
}

fn bench_vm_fibonacci(c: &mut Criterion) {
    let source = r#"
        fn fib(n: Number): Number do
            if n <= 1 do
                return n
            end
            return fib(n - 1) + fib(n - 2)
        end
        fib(10)
    "#;
    let program = parser::parse(source, "bench.luma").unwrap();
    let chunk = compile_program(&program);

    c.bench_function("vm execute fibonacci(10)", |b| {
        b.iter(|| {
            let mut vm = VM::new(chunk.clone());
            black_box(vm.run())
        })
    });
}

fn bench_vm_loop(c: &mut Criterion) {
    let source = r#"
        var sum = 0
        var i = 0
        while i < 100 do
            sum = sum + i
            i = i + 1
        end
        sum
    "#;
    let program = parser::parse(source, "bench.luma").unwrap();
    let chunk = compile_program(&program);

    c.bench_function("vm execute loop sum", |b| {
        b.iter(|| {
            let mut vm = VM::new(chunk.clone());
            black_box(vm.run())
        })
    });
}

fn bench_vm_list_operations(c: &mut Criterion) {
    let source = r#"
        let list = [1, 2, 3, 4, 5]
        var sum = 0
        for x in list do
            sum = sum + x
        end
        sum
    "#;
    let program = parser::parse(source, "bench.luma").unwrap();
    let chunk = compile_program(&program);

    c.bench_function("vm execute list iteration", |b| {
        b.iter(|| {
            let mut vm = VM::new(chunk.clone());
            black_box(vm.run())
        })
    });
}

criterion_group!(
    benches,
    bench_vm_arithmetic,
    bench_vm_fibonacci,
    bench_vm_loop,
    bench_vm_list_operations
);
criterion_main!(benches);
