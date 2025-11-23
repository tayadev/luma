---
sidebar_position: 3
---

# Functions

Functions are first-class values in Luma.

## Function Definition

```luma
fn name(param1: Type1, param2: Type2): ReturnType do
  -- body
end
```

**Anonymous functions:**
```luma
let add = fn(a: Number, b: Number): Number do
  return a + b
end
```

## Parameters

### Required Parameters

```luma
fn greet(name: String): String do
  return "Hello, ${name}!"
end
```

### Optional Parameters

Parameters can have default values:

```luma
fn greet(name: String, title: String = "Friend"): String do
  return "Hello, ${title} ${name}!"
end

greet("Alice")                     -- "Hello, Friend Alice!"
greet("Bob", "Dr.")                -- "Hello, Dr. Bob!"
```

## Function Calls

### Positional Arguments

Parentheses are **required** for all function calls:

```luma
add(2, 3)
greet("Alice", "Ms.")
func()                             -- no arguments
```

### Named Arguments

Arguments can be passed by name:

```luma
add(a = 2, b = 3)
greet(name = "Alice", title = "Dr.")
greet(title = "Dr.", name = "Alice")    -- order doesn't matter
```

**Mixing positional and named:**
```luma
greet("Alice", title = "Dr.")      -- positional then named
```

## Return Types

### Explicit Returns

```luma
fn factorial(n: Number): Number do
  if n <= 1 do
    return 1
  end
  return n * factorial(n - 1)
end
```

### Implicit Returns

If a function/block doesn't end with explicit `return`, the last expression becomes an implicit return:

```luma
fn add(a: Number, b: Number): Number do
  a + b                            -- implicitly returned
end
```

### Void Functions

Functions that don't return a meaningful value return `null`:

```luma
fn printMessage(msg: String): Null do
  print(msg)
end
```

## Closures

Functions capture variables from their enclosing scope:

```luma
fn makeCounter(): fn(): Number do
  var count = 0
  return fn(): Number do
    count = count + 1
    return count
  end
end

let counter = makeCounter()
print(counter())                   -- 1
print(counter())                   -- 2
print(counter())                   -- 3
```

## Higher-Order Functions

Functions can accept and return other functions:

```luma
fn map(list: List(Any), f: fn(Any): Any): List(Any) do
  let result = []
  for item in list do
    result.push(f(item))
  end
  return result
end

let doubled = map([1, 2, 3], fn(x) do x * 2 end)
-- [2, 4, 6]
```

## Next Steps

Learn about [async functions](../advanced/async-await.md) or explore the [type system](./types.md).
