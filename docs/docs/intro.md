---
sidebar_position: 1
---

# Welcome to Luma

**Luma** is a typed scripting language designed for simplicity, safety, and expressiveness.

## Design Philosophy

Luma is built on the following core principles:

- **Everything is a value** — Functions, types, and modules are first-class citizens
- **Explicit over implicit** — No hidden conversions or magical behavior
- **Safety without ceremony** — Type inference reduces boilerplate while maintaining safety
- **Errors as values** — No exceptions; all failures are explicit `Result` types
- **Async by design** — Native async/await with Promise-based concurrency
- **Modern tooling** — Built-in dependency management via URL imports
- **Minimal core** — As little in the language core as possible; rely on a rich standard library

## Quick Example

```luma
let greet = fn(name: String): String do
  return "Hello, ${name}!"
end

print(greet("World"))
```

## Key Features

### Type Safety with Inference

```luma
let x = 42                    -- inferred as Number
let name = "Luma"             -- inferred as String
let items: List(Number) = [1, 2, 3]
```

### Error Handling Without Exceptions

```luma
fn divide(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    return { ok = null, err = "Division by zero" }
  end
  return { ok = a / b, err = null }
end
```

### Native Async/Await

```luma
fn fetchUser(id: String): Result(User, Error) do
  let response = await http.get("/users/${id}")
  return parseUser(response)
end
```

### URL-Based Imports

```luma
let http = import("https://example.com/http.luma")
let utils = import("./utils.luma")
let lib = import("gh:user/repo@1.2.3")
```

## Getting Started

To learn more about Luma, explore the documentation:

- **Getting Started** - Learn how to install and use Luma
- **Basics** - Understand the core language features
- **Advanced** - Dive into async/await, pattern matching, and modules
- **Reference** - Complete language reference including types, traits, and operators

## Language Pipeline

```
Source → Lexer → Parser → AST → Type Checker → Typed AST → Compiler → Bytecode → VM → Output
```

**Current Status:**
- ✅ Lexer — Complete
- ✅ Parser — Complete  
- 🚧 Type Checker — In development
- 🚧 Compiler — In development
- 🚧 VM — In development

The current implementation includes the Lexer, Parser, and AST phases. Type checking, bytecode compilation, and VM execution are under active development.
