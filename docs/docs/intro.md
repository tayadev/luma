---
sidebar_position: 1
slug: /
---

# Welcome to Luma

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

## Development Status

Luma is currently in **active development**, expect things to be broken and changing.
Until version 1.0.0:
- The standard library is incomplete and may change frequently
- The compiler and tooling are under heavy development
- The semver spec is not followed properly

## Getting Started

Ready to learn Luma? Here's how you can get install it on your system:
> We will support package managers once luma reaches version 1.0.0.

### Windows

Run this command in your terminal:
```
powershell -c "irm https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.ps1 | iex"
```

### Linux and macOS

Run this command in your terminal:
```
curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | sh
```

## Language Specification

The full language specification is available in the [Luma Language Specification](https://github.com/tayadev/luma/blob/main/SPEC.md) document.