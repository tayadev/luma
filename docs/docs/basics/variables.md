---
sidebar_position: 2
---

# Variables and Constants

Luma provides two ways to declare variables: immutable bindings (`let`) and mutable variables (`var`).

## Immutable Bindings with `let`

Use `let` to declare immutable bindings that cannot be reassigned:

```luma
let name = "Luma"
-- name = "Other"  -- Error! name is immutable

let pi: Number = 3.14
```

`let` bindings cannot be reassigned after initialization.

## Mutable Variables with `var`

Use `var` to declare mutable variables that can be reassigned:

```luma
var x = 42
x = 50  -- OK, x is mutable

var count: Number = 10
count = count + 1
```

`var` variables can be reassigned multiple times.

## Type Inference

Types are inferred when annotations are omitted:

```luma
let x = 42                         -- inferred as Number
let name = "Alice"                 -- inferred as String
let items = [1, 2, 3]              -- inferred as List(Number)
```

## Type Annotations

Type annotations are optional but can be provided explicitly:

```luma
let x: Number = 42
let name: String = "Luma"
let items: List(String) = ["a", "b", "c"]
let value: Any = "flexible"            -- explicit Any
```

## Destructuring

### List Destructuring

```luma
let [first, second, third] = [1, 2, 3]

let [head, ...tail] = [1, 2, 3, 4]
-- head = 1, tail = [2, 3, 4]

let [a, b, ...] = [10, 20, 30, 40]
-- a = 10, b = 20, rest ignored
```

**Behavior:**
- Missing elements assign `null`
- `...name` captures remaining elements as list
- `...` without name discards remaining elements

### Table Destructuring

```luma
let person = { name = "Alice", age = 30, city = "NYC" }
let { name, age } = person
-- name = "Alice", age = 30

let { name: userName, age: userAge } = person
-- userName = "Alice", userAge = 30
```

## Assignment

Only `var` variables can be reassigned:

```luma
var counter = 0
counter = counter + 1              -- valid

let constant = 100
-- constant = 200                  -- Error! let bindings are immutable
```

Attempting to reassign a `let` binding is a compile error.
