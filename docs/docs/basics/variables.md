---
sidebar_position: 2
---

# Variables and Bindings

Luma provides two ways to declare variables: **immutable bindings** with `let` and **mutable variables** with `var`. Immutability is the default, encouraging safer, more predictable code.

## Immutable Bindings with `let`

Use `let` to declare immutable bindings. Once assigned, they cannot be changed:

```luma
let name = "Luma"
let pi = 3.14159
let items = [1, 2, 3]

-- name = "Other"  -- ❌ Error! let bindings are immutable
```

### Benefits of Immutability
- **Predictability** — Variables don't change unexpectedly
- **Concurrency safety** — Safe to share across async operations
- **Debugging** — Easier to reason about program state
- **Functional patterns** — Enables higher-order functions and transformations

## Mutable Variables with `var`

Use `var` when you need to reassign a variable:

```luma
var counter = 0
counter = counter + 1          -- ✅ Allowed
counter = 42                   -- ✅ Can reassign anytime

var count = 10
count = count + 1
print(count)                   -- Output: 11
```

### When to Use `var`
- Loop counters and accumulators
- State that changes over time
- When reassignment is essential to your algorithm

## Type Annotations

### Explicit Type Annotations

Provide explicit types with the `:` syntax:

```luma
let name: String = "Alice"
let age: Number = 30
let items: List(String) = ["a", "b", "c"]
let data: Any = someUnknownValue
```

### Type Inference

When you omit the type, Luma infers it from the value:

```luma
let x = 42                     -- inferred as Number
let greeting = "Hello"         -- inferred as String  
let flags = [true, false]      -- inferred as List(Boolean)
let mixed = [1, "two"]         -- inferred as List(Any)
```

### When to Annotate
- **For clarity** — When the type isn't obvious
- **For documentation** — Help readers understand intent
- **For API boundaries** — Function parameters and returns
- **For complex types** — Nested generics like `Result(List(Number), String)`

## Scope

Variables are scoped to the block they're declared in:

```luma
let outer = "visible"
do
  let inner = "only here"
  print(outer)          -- ✅ Can access outer
end
print(inner)            -- ❌ Error! inner is out of scope
```

Closures capture variables from their enclosing scope:

```luma
let makeCounter = fn() do
  var count = 0
  fn() do
    count = count + 1
    count
  end
end

let counter = makeCounter()
print(counter())        -- 1
print(counter())        -- 2
print(counter())        -- 3
```

## Destructuring Assignment

Destructuring lets you extract values from collections into separate variables.

### Array Destructuring

```luma
let [first, second, third] = [10, 20, 30]
-- first = 10, second = 20, third = 30

let [a, b] = [1, 2, 3, 4]  -- Only take first two
-- a = 1, b = 2

let [head, ...tail] = [1, 2, 3, 4]
-- head = 1, tail = [2, 3, 4]

let [x, _, z] = [10, 20, 30]  -- Use _ to skip elements
-- x = 10, z = 30
```

### Table (Record) Destructuring

```luma
let person = { name = "Alice", age = 30, city = "NYC" }

-- Extract specific fields
let { name, age } = person
-- name = "Alice", age = 30

-- Rename during extraction
let { name: userName, age: userAge } = person  
-- userName = "Alice", userAge = 30

-- Nested destructuring
let { user = { name } } = { user = { name = "Bob", age = 25 } }
-- name = "Bob"
```

## Variable Shadowing

You can declare a new variable with the same name, shadowing the outer one:

```luma
let x = 10
do
  let x = 20        -- Shadows outer x
  print(x)          -- Output: 20
end
print(x)            -- Output: 10 (outer x still exists)
```

## Constants (Compile-Time)

:::info
True compile-time constants are a planned feature. Currently, `let` bindings are immutable at runtime.
:::

Attempting to reassign a `let` binding is a compile error.
