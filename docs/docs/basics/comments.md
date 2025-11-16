---
sidebar_position: 6
---

# Comments

Luma supports both single-line and multi-line comments.

## Single-line Comments

Use `--` for single-line comments:

```luma
-- This is a single-line comment

let x = 42  -- Comments can appear after code
```

## Multi-line Comments

Use `--[[ ... ]]` for multi-line comments:

```luma
--[[
This is a multi-line comment
that spans multiple lines.
Useful for documentation or temporarily disabling code.
]]

let y = 10
```

## Documentation Comments

While Luma doesn't have special documentation comment syntax yet, you can use multi-line comments for documenting functions and types:

```luma
--[[
Calculates the factorial of a number.

Parameters:
  n: Number - The number to calculate factorial for

Returns:
  Number - The factorial of n
]]
let factorial = fn(n: Number): Number do
  if n <= 1 do
    return 1
  end
  return n * factorial(n - 1)
end
```

## Style Guide

According to the Luma style guide:

- Use 2 spaces for indentation
- Use `snake_case` for variable and function names
- Add comments to explain complex logic or non-obvious behavior
- Keep comments up-to-date with code changes
