---
sidebar_position: 3
---

# Functions

Functions are first-class values in Luma.

## Basic Function Syntax

```luma
let add = fn(a: Number, b: Number): Number do
  return a + b
end

let result = add(2, 3)  -- 5
```

## Implicit Returns

Functions implicitly return the value of the last expression if no `return` statement is used:

```luma
let add = fn(a: Number, b: Number): Number do
  a + b  -- implicitly returned
end
```

## Optional Parameters

Parameters can have default values:

```luma
let greet = fn(name: String = "World"): String do
  return "Hello, ${name}!"
end

greet()         -- "Hello, World!"
greet("Alice")  -- "Hello, Alice!"
```

## Single-Parameter Calls

Parentheses are optional for single-argument calls:

```luma
let square = fn(x: Number): Number do
  return x * x
end

square 5  -- 25
square(5) -- also valid
```

## Named Arguments

Arguments can be passed by name:

```luma
let divide = fn(a: Number, b: Number): Number do
  return a / b
end

divide(a = 10, b = 2)  -- 5
divide(b = 2, a = 10)  -- order doesn't matter with named args
```

## Multiple Parameters

Multiple arguments require parentheses:

```luma
let result = add(2, 3)
let result = add(a = 2, b = 3)
```

## Higher-Order Functions

Functions can accept and return other functions:

```luma
let makeMultiplier = fn(factor: Number): Function do
  return fn(x: Number): Number do
    return x * factor
  end
end

let double = makeMultiplier(2)
double(5)  -- 10
```

## Anonymous Functions

Functions don't need to be assigned to variables:

```luma
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(fn(x: Number): Number do x * 2 end)
```

## Next Steps

Learn about [async functions](../advanced/async-await.md) or explore the [type system](./types.md).
