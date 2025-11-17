---
sidebar_position: 1
---

# Installation

## Current Status

Luma is currently in active development. The current implementation includes:

- ✅ Lexer — Complete tokenization of source code
- ✅ Parser — Complete parsing of tokens into Abstract Syntax Tree (AST)
- 🚧 Type Checker — In development
- 🚧 Compiler — In development
- 🚧 Virtual Machine — In development

## Building from Source

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Clone and Build

```bash
git clone https://github.com/tayadev/luma.git
cd luma
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Using the Parser

The current implementation includes a CLI tool for parsing Luma source files:

```bash
./target/release/luma <file.luma>
```

This will parse the file and output the resulting AST in debug format.

## What Works Now

The parser can currently handle:

- ✅ All literal types (numbers, strings, booleans, null, arrays, tables)
- ✅ Variables and destructuring (`let`, `var`)
- ✅ Functions with parameters and closures
- ✅ All control flow (if/else, while, do-while, for-in, break, continue)
- ✅ Pattern matching (`match` expressions)
- ✅ All operators (arithmetic, comparison, logical)
- ✅ String interpolation
- ✅ Comments (single-line and multi-line)
- ✅ Type annotations

## What's Coming Next

- Type checking and inference
- Bytecode compilation
- Virtual machine/interpreter
- Standard library implementation
- Async/await runtime support
- Module system and imports

## Next Steps

Once you have Luma built, check out the [Variables](../basics/variables.md) section to start learning the language syntax.
