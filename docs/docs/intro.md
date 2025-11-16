---
sidebar_position: 1
---

# Welcome to Luma

**Luma** is a typed scripting language designed for simplicity, safety, and expressiveness.

## Core Design Principles

- **Everything is a value** - Functions, types, and modules are all first-class values
- **No implicit assignment** - All assignments must be explicit
- **Tiny core syntax** - Simple, consistent syntax that's easy to learn
- **Explicit error handling** - Errors are values, not exceptions
- **No magic** - Predictable behavior without hidden complexity
- **Built-in dependency management** - Import modules via URL
- **Async/await support** - First-class asynchronous programming
- **Garbage-collected** - Automatic memory management

## Quick Example

```luma
let greet = fn(name: String): String do
  return "Hello, ${name}!"
end

print(greet("World"))
```

## Getting Started

To learn more about Luma, explore the documentation:

- **Getting Started** - Learn how to install and use Luma
- **Basics** - Understand the core language features
- **Advanced** - Dive into advanced topics like async/await and pattern matching
- **Reference** - Complete language reference

## Language Pipeline

```
Source → Lexer → Parser → AST → Type Checker → Typed AST → Compiler → Bytecode → VM → Output
```

The current implementation includes the Lexer, Parser, and AST phases. Type checking, compilation, and VM execution are in development.
