---
sidebar_position: 1
---

# Welcome to Luma 🔶

**Luma** is a typed scripting language designed for simplicity, safety, and expressiveness. It combines the ease of scripting languages with the reliability of strong static typing.

## Why Luma?

Luma is built on the following core principles:

- **Everything is a value** — Functions, types, and modules are first-class citizens, enabling functional programming patterns
- **Explicit over implicit** — No hidden conversions or magical behavior; you always know what your code does
- **Safety without ceremony** — Strong static type inference reduces boilerplate while maintaining comprehensive safety
- **Errors as values** — No exceptions; all failures are explicit `Result` types for predictable error handling
- **Async-first design** — Native async/await with Promise-based concurrency built into the language
- **Modern tooling** — URL-based module system for seamless dependency management
- **Minimal core** — The language core is intentionally lean; rich functionality lives in the standard library

## Quick Example

```luma
let greet = fn(name: String): String do
  "Hello, ${name}!"
end

print(greet("World"))  -- Output: Hello, World!
```

## Key Language Features

### Type Safety with Inference

Write type-safe code without excessive annotations. The compiler infers types from context:

```luma
let x = 42                      -- inferred as Number
let name = "Luma"               -- inferred as String
let items: List(Number) = [1, 2, 3]
let add = fn(a, b) do a + b end -- parameter types inferred from usage
```

### Immutability by Default

Variables are immutable by default for safer, more predictable code:

```luma
let x = 10      -- immutable
var y = 20      -- mutable (explicit)
y = 25          -- allowed
```

### Expressive Pattern Matching

Match on structure and values with exhaustive pattern checking:

```luma
match value do
  case 0 then print("Zero")
  case x if x > 0 then print("Positive")
  case _ then print("Negative")
end
```

### First-Class Functions

Pass functions as values, enabling higher-order programming:

```luma
let map = fn(list, transform) do
  let result = []
  for item in list do
    result.push(transform(item))
  end
  result
end

let doubled = map([1, 2, 3], fn(x) do x * 2 end)
```

### Result Types for Error Handling

No exceptions—all errors are values:

```luma
fn divide(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    return { ok = null, err = "Division by zero" }
  end
  { ok = a / b, err = null }
end
```

### Native Async/Await

Non-blocking concurrent code without callback hell:

```luma
fn fetchAndProcess(id: String): Promise(Result(Data, Error)) do
  let raw = await fetchData(id)
  process(raw)
end
```

## Development Status

Luma is currently in **active development** with the following phases:

### ✅ Completed
- **Lexer** — Complete tokenization and lexical analysis
- **Parser** — Full parsing to Abstract Syntax Tree (AST)
- **AST** — Comprehensive representation of all language constructs

### 🚧 In Development
- **Type Checker** — Static type checking and inference
- **Compiler** — Bytecode generation
- **Virtual Machine** — Execution engine

### 📋 Planned
- Standard library implementation
- Module system and URL-based imports
- Advanced async/await runtime
- Performance optimizations

:::info
The language specification is complete, and the parser supports all core language features. You can explore the language syntax and structure today. Runtime capabilities are actively being implemented.
:::

## Getting Started

Ready to learn Luma? Here's your journey:

1. **[Installation](./getting-started/installation.md)** — Get Luma running on your system
2. **[Basics](./basics/variables.md)** — Learn variables, types, functions, and control flow
3. **[Advanced](./advanced/pattern-matching.md)** — Master pattern matching, async/await, and modules
4. **[Reference](./reference/operators.md)** — Deep dive into the language reference

## Language Architecture

```
Source Code
    ↓ (Lexer)
Tokens
    ↓ (Parser)
Abstract Syntax Tree (AST)
    ↓ (Type Checker) [In Progress]
Typed AST
    ↓ (Compiler) [In Progress]
Bytecode
    ↓ (Virtual Machine) [In Progress]
Output
```

The pipeline is modular and each phase is independently testable. See the [Language Specification](https://github.com/tayadev/luma/blob/main/SPEC.md) for comprehensive details on syntax and semantics.
