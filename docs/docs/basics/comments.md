---
sidebar_position: 6
---

# Comments

Luma supports both single-line and multi-line comments.

## Single-Line Comments

Use `--` for single-line comments that extend to the end of the line:

```luma
-- This is a single-line comment

let x = 42  -- Comments can appear after code
```

## Multi-Line Comments

Use `--[[` and `]]` for multi-line comments:

```luma
--[[
This is a multi-line comment
that spans multiple lines.
Useful for documentation or temporarily disabling code.
]]

let y = 10
```

## Naming Conventions

According to the Luma specification:

- **Variables and functions:** `snake_case`
- **Types and constructors:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`

```luma
let my_variable = 42
let calculate_total = fn() do ... end

let Person = { ... }

let MAX_RETRIES = 3
```

## Style Guide

- Use consistent indentation (2 spaces recommended)
- Add comments to explain complex logic or non-obvious behavior
- Keep comments up-to-date with code changes
- Use multi-line comments for documentation blocks
