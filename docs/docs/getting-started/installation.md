---
sidebar_position: 1
---

# Installation & Setup

## Current Development Status

Luma is currently in **active development**. The following is available:

| Component | Status | What You Can Do |
|-----------|--------|-----------------|
| Lexer | ✅ Complete | Tokenize Luma source code |
| Parser | ✅ Complete | Parse into Abstract Syntax Tree (AST) |
| AST | ✅ Complete | Inspect the abstract representation of your code |
| Type Checker | 🚧 In Development | Type checking coming soon |
| Compiler | 🚧 In Development | Bytecode generation coming soon |
| VM/Interpreter | 🚧 In Development | Code execution coming soon |

:::tip
You can already write and parse complete Luma programs! You just can't execute them yet. This is perfect for experimenting with the language syntax and structure.
:::

## Building from Source

### Prerequisites

- **Rust 1.70+** — Install from [rustup.rs](https://rustup.rs/)
- **Cargo** — Comes with Rust
- **Git** — For cloning the repository

### Installation Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/tayadev/luma.git
   cd luma
   ```

2. **Build in release mode:**
   ```bash
   cargo build --release
   ```

3. **Run tests to verify installation:**
   ```bash
   cargo test
   ```

4. **Verify the binary works:**
   ```bash
   ./target/release/luma --help
   ```

## Using the Parser

The current CLI tool parses Luma source files and outputs the resulting AST:

```bash
luma <file.luma>        # Parse and print AST
luma --help             # Show help information
```

### Example

Create a file `hello.luma`:

```luma
let greet = fn(name: String): String do
  "Hello, ${name}!"
end

print(greet("World"))
```

Parse it:

```bash
./target/release/luma hello.luma
```

Output (pretty-printed AST):

```
Program([
  Stmt::Let(
    Binding {
      name: "greet",
      value: Expr::Function { ... }
    }
  ),
  Stmt::ExprStmt(
    Expr::Call { ... }
  )
])
```

## What Works Now

The parser handles all these language features:

### Data Types
- ✅ Numbers (integers and floats)
- ✅ Strings with interpolation
- ✅ Booleans (`true`, `false`)
- ✅ Null values
- ✅ Arrays and Tables (records)
- ✅ Type annotations and type literals

### Variables & Binding
- ✅ Immutable let bindings
- ✅ Mutable var bindings
- ✅ Destructuring patterns
- ✅ Nested destructuring

### Functions
- ✅ Function definition and declaration
- ✅ Named and optional parameters
- ✅ Default parameter values
- ✅ Closures and anonymous functions
- ✅ Implicit returns

### Control Flow
- ✅ If/elseif/else expressions
- ✅ While loops
- ✅ For-in loops
- ✅ Do-end blocks
- ✅ Break and continue statements
- ✅ Return statements

### Operators & Expressions
- ✅ Arithmetic: `+`, `-`, `*`, `/`, `%`, `^`
- ✅ Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- ✅ Logical: `and`, `or`, `not`
- ✅ Assignment: `=`
- ✅ Field access: `.`
- ✅ Indexing: `[]`
- ✅ Function calls with positional and named arguments

### Advanced Features
- ✅ Pattern matching with `match`/`case`
- ✅ Comments (single-line `--` and multi-line `--[[ ]]`)
- ✅ String interpolation with `${}`
- ✅ Type annotations
- ✅ First-class types

## Development & Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suite

```bash
cargo test --test parser_tests
cargo test --test runtime_tests
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Add New Tests

Tests use a fixture-based system. Create a pair of files in `tests/fixtures/{category}/`:

- `test_name.luma` — Your Luma source code
- `test_name.ron` — Expected AST in RON format

Example: `tests/fixtures/functions/simple.luma` and `tests/fixtures/functions/simple.ron`

The test framework automatically discovers and runs these pairs.

## Project Structure

```
luma/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── ast.rs               # AST definitions
│   ├── parser/
│   │   ├── mod.rs           # Parser entry point
│   │   ├── lexer.rs         # Tokenization
│   │   ├── expressions.rs   # Expression parsers
│   │   ├── statements.rs    # Statement parsers
│   │   └── ...
│   ├── typecheck/           # Type checker (in progress)
│   ├── bytecode/            # Compiler (in progress)
│   └── vm/                  # Virtual machine (in progress)
├── tests/
│   ├── parser_tests.rs      # Parser test framework
│   ├── fixtures/            # Test fixtures
│   │   ├── functions/
│   │   ├── operators/
│   │   └── ...
│   └── runtime/             # Runtime tests (future)
├── docs/                    # Documentation (you are here)
├── SPEC.md                  # Complete language specification
└── Cargo.toml               # Rust project manifest
```

## What's Next?

### Short Term (Weeks)
- Type checking and inference implementation
- Bytecode compiler for AST to bytecode translation
- Basic interpreter for simple programs

### Medium Term (Months)
- Full VM implementation with bytecode execution
- Standard library with common functions
- Improved error messages and diagnostics

### Long Term
- Async/await runtime support
- Module system with URL-based imports
- Performance optimizations and JIT compilation
- Package manager and ecosystem

## Troubleshooting

### Build Fails
Ensure Rust 1.70+ is installed:
```bash
rustc --version
rustup update
```

### Tests Fail
Run in verbose mode:
```bash
cargo test -- --nocapture
```

### Parser Output Seems Wrong
Check the test fixtures in `tests/fixtures/` for expected behavior. File an issue on [GitHub](https://github.com/tayadev/luma/issues) with a minimal example.

## Get Help

- **[GitHub Issues](https://github.com/tayadev/luma/issues)** — Report bugs or request features
- **[Language Specification](https://github.com/tayadev/luma/blob/main/SPEC.md)** — Complete language reference
- **[Project README](https://github.com/tayadev/luma)** — Overview and quick links

## Next Steps

Once you've installed Luma, explore the language:

- [Learn Variables](../basics/variables.md) — Start with the basics
- [Define Functions](../basics/functions.md) — Master function syntax
- [Control Flow](../basics/control-flow.md) — Understand if/else, loops, and more
