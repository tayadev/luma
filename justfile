# Luma development tasks
# Install just: cargo install just
# Usage: just [recipe]

# List available recipes
default:
    @just --list

# Run all tests (using nextest)
test:
    cargo nextest run

# Run tests with all features
test-all:
    cargo nextest run --all-features

# Produce JUnit XML for CI
test-junit:
    cargo nextest run --all-features

# CI: Run tests with coverage and JUnit XML output
ci-test:
    cargo llvm-cov nextest --all-features --workspace --lcov --output-path lcov.info

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the CLI with a file
run file:
    cargo run -- {{file}}

# Generate code coverage report (requires cargo-llvm-cov, uses nextest)
coverage:
    cargo llvm-cov nextest --all-features --workspace --lcov --output-path lcov.info

# Generate HTML coverage report
coverage-html:
    cargo llvm-cov --all-features --workspace --html

# Generate coverage and show summary
coverage-summary:
    cargo llvm-cov --all-features --workspace --summary-only

# Generate coverage and open in browser (macOS only)
coverage-open:
    cargo llvm-cov --all-features --workspace --html
    open coverage/html/index.html

# Run benchmarks
bench:
    cargo bench

# Clean build artifacts
clean:
    cargo clean

# Full CI check: format, lint, test, coverage
check: fmt-check lint test coverage-summary
    @echo "✓ All checks passed!"

# Prepare for commit: format, lint, test
ready: fmt lint test
    @echo "✓ Ready to commit!"
