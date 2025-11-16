---
sidebar_position: 1
---

# Installation

## Current Status

Luma is currently in active development. The current implementation includes:

- ✅ Lexer - Tokenization of source code
- ✅ Parser - Parsing tokens into Abstract Syntax Tree (AST)
- 🚧 Type Checker - In development
- 🚧 Compiler - In development
- 🚧 Virtual Machine - In development

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

## Next Steps

Once you have Luma built, check out the [Variables](../basics/variables.md) section to start learning the language syntax.
