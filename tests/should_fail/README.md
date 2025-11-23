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

### Type Check Failures

- **duplicate_named_arg** - Attempts to call a function with duplicate named arguments (typecheck)
- **function_generic_arg_mismatch** - Calls a generic function with incorrect type arguments (typecheck)
- **function_return_mismatch** - Function returns a type that doesn't match its declared return type (typecheck)
- **immutable_let** - Attempts to reassign an immutable `let` variable (typecheck)
- **match_missing_tag** - Uses a match tag that is not present in the matched table type (typecheck)
- **match_not_exhaustive** - Match expression that doesn't cover all possible cases (typecheck)
- **number_plus_string** - Attempts to add incompatible types (typecheck)
- **table_unknown_field** - Accesses a field that doesn't exist on a table (typecheck)
- **type_mismatch_add** - Attempts to add incompatible types (typecheck)
- **unary_neg_no_overload** - Attempts unary negation without the required overload (typecheck)
- **undefined_var** - References an undefined variable (typecheck)
- **unreachable_pattern** - Match pattern that can never be reached due to earlier catch-all pattern (typecheck)
- **wrong_arity** - Calls a function with wrong number of arguments (typecheck)

### Parse Failures

- **illegal_table_key** - Uses an illegal form for a table key (e.g., numeric literal without brackets) (parse)
- **invalid_interpolation** - Invalid string interpolation syntax (e.g., missing closing quote) (parse)

## Adding New Tests

1. Create a `.luma` file with code that should fail
2. Create a matching `.expect` file with the failure type
3. Run `cargo test` to verify the test behaves as expected
