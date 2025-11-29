# Tests & Benchmarks

This project uses Rust's built-in integration tests under `tests/` and Criterion benchmarks under `benches/`.

## Layout

- `tests/fixtures/`: Parser and runtime fixtures grouped by category
  - Create pairs of `.luma` (source) and `.ron` (expected) files
  - Categories include `blocks/`, `functions/`, `operators/`, `strings/`, etc.
- `tests/runtime/`: Runtime-specific programs and expected outputs
- `tests/should_fail/`: Negative tests expected to fail
- Top-level `tests/*.rs`: Test harnesses (`parser_tests.rs`, `runtime_tests.rs`, etc.) that auto-discover fixtures.

## Conventions

- Always add both `.luma` and `.ron` for parser fixtures.
- Prefer adding a fixture over one-off inline tests.
- Keep categories small and focused; add a new folder if needed.

## Running

```pwsh
cargo test
```

## Performance Benchmarks (Criterion)

Benchmarks live under `benches/` and use Criterion.

Run them with:

```pwsh
cargo bench
```

Benchmarks load real fixtures to measure end-to-end performance of the lexer/parser and VM.

## Adding a New Fixture

1. Choose a category under `tests/fixtures/` or create a new one.
2. Add `<name>.luma` and `<name>.ron` with matching base names.
3. Run `cargo test`.

## Adding a New Benchmark

1. Create a file under `benches/` (e.g., `my_bench.rs`).
2. Use `criterion` and the helper in `src/test_utils.rs` to load fixtures.
3. Register a `criterion_group!` and `criterion_main!`.
