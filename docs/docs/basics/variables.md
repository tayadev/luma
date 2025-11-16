---
sidebar_position: 2
---

# Variables and Constants

Luma provides two ways to declare variables: mutable (`var`) and immutable (`let`).

## Mutable Variables with `var`

Use `var` to declare a variable that can be reassigned:

```luma
var x = 42
x = 50  -- OK, x is mutable

var count: Number = 10
count = count + 1
```

## Immutable Constants with `let`

Use `let` to declare a constant that cannot be reassigned:

```luma
let name = "Luma"
-- name = "Other"  -- Error! name is immutable

let pi: Number = 3.14
```

## Type Inference

Types are inferred if not explicitly provided:

```luma
let message = "Hello"  -- inferred as String
var count = 0          -- inferred as Number
```

## Explicit Type Annotations

You can explicitly specify types:

```luma
var x: Number = 42
let name: String = "Luma"
let items: Array(String) = ["a", "b", "c"]
```

## Destructuring

### Table Destructuring

```luma
let person = { name = "Alice", age = 30 }
let { name, age } = person

print(name)  -- "Alice"
print(age)   -- 30
```

### Array Destructuring

```luma
let numbers = [1, 2, 3]
let [first, second, third] = numbers

print(first)   -- 1
print(second)  -- 2
print(third)   -- 3
```

### Rest Pattern

Use `...` to capture remaining elements:

```luma
let numbers = [1, 2, 3, 4, 5]
let [head, ...tail] = numbers

print(head)  -- 1
print(tail)  -- [2, 3, 4, 5]
```

Or ignore remaining elements:

```luma
let [head, ...] = numbers  -- only capture head
```

### Missing Elements

Missing elements are assigned `null`:

```luma
let [a, b, c] = [1, 2]
print(c)  -- null
```
