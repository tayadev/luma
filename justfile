# Luma development tasks
# Install just: cargo install just
# Usage: just [recipe]

set windows-shell := ["powershell", "-Command"]

# List available recipes
default:
    @just --list

# Run all tests (using nextest)
test:
    cargo nextest run --config-file nextest.toml

# Run tests with all features
test-all:
    cargo nextest rua --config-file nextest.toml --all-features
# CI: Run tests with coverage and JUnit XML output
ci-test:
    cargo llvm-cov nextest --config-file nextest.toml --all-features --workspace --lcov --output-path lcov.info

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

coverage-text:
    cargo llvm-cov nextest --all-features --workspace --text

# Run benchmarks
bench:
    cargo bench

# Clean build artifacts
clean:
    cargo clean

check: fmt-check lint test
    @echo "✓ All checks passed!"

# Prepare for commit: format, lint, test
ready: fmt lint test
    @echo "✓ Ready to commit!"

commit: ready
    git add .
    copilot -p "Write a git commit message for the currently staged changes" --allow-tool 'shell(git)'