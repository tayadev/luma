# Should-Fail Tests

This directory contains tests that are expected to fail at various stages of compilation/execution.

## Format

Each test consists of two files:
- `<test_name>.luma` - The source code that should fail
- `<test_name>.expect` - The expected failure stage

## Failure Types

The `.expect` file should contain one of:
- `parse` - Expected to fail during parsing
- `typecheck` - Expected to fail during type checking
- `runtime` - Expected to fail during execution

## Current Tests

- **immutable_let** - Attempts to reassign an immutable `let` variable (typecheck)
- **type_mismatch_add** - Attempts to add incompatible types (typecheck)
- **undefined_var** - References an undefined variable (typecheck)
- **wrong_arity** - Calls a function with wrong number of arguments (typecheck)

## Adding New Tests

1. Create a `.luma` file with code that should fail
2. Create a matching `.expect` file with the failure type
3. Run `cargo test` to verify the test behaves as expected
