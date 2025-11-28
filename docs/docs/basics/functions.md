---
sidebar_position: 3
---

# Functions

Functions are first-class values in Luma, meaning they can be assigned to variables, passed as arguments, and returned from other functions. They're the building blocks of functional programming.

## Function Definition

### Basic Syntax

Functions are declared with `fn`, parameters, and optional return type annotation:

```luma
fn add(a: Number, b: Number): Number do
  a + b                    -- implicit return
end

fn greet(name: String): String do
  "Hello, ${name}!"        -- implicit return
end

fn printLine(text: String) do
  print(text)              -- no return type = returns Null
end
```

### Implicit Returns

The last expression in a function body is automatically returned:

```luma
fn multiply(x: Number, y: Number): Number do
  x * y                   -- automatically returned, no 'return' needed
end

fn processData(data) do
  let result = transform(data)
  result                  -- implicitly returned
end
```

### Explicit Returns

You can also use `return` explicitly:

```luma
fn findFirst(items: List(Number)) do
  for item in items do
    if item > 10 do
      return item         -- early exit
    end
  end
  null                    -- default if nothing found
end
```

## Parameters

### Required Parameters

```luma
fn divide(numerator: Number, denominator: Number): Number do
  numerator / denominator
end

divide(10, 2)             -- ✅ 5
divide(10)                -- ❌ Error! denominator is required
```

### Optional Parameters (Default Values)

Parameters with default values are optional:

```luma
fn greet(name: String, title: String = "Friend"): String do
  "Hello, ${title} ${name}!"
end

greet("Alice")                    -- "Hello, Friend Alice!"
greet("Bob", "Dr.")               -- "Hello, Dr. Bob!"
greet("Alice", title = "Ms.")     -- "Hello, Ms. Alice!"
```

### Parameter Types

Each parameter can have an explicit type annotation:

```luma
fn process(
  name: String,
  count: Number,
  enabled: Boolean = true
): String do
  if enabled do
    "${name}: ${count}"
  else do
    null
  end
end
```

## Function Calls

### Positional Arguments

Arguments are matched to parameters by position:

```luma
let sum = fn(a, b, c) do a + b + c end
sum(1, 2, 3)              -- ✅ All positional
```

### Named Arguments

Arguments can be passed by name, in any order:

```luma
let greet = fn(first: String, last: String, title: String = "Mr.") do
  "${title} ${first} ${last}"
end

greet(first = "Jane", last = "Smith")
greet(title = "Dr.", last = "Jones", first = "Robert")
```

### Mixing Positional and Named

Positional arguments must come before named ones:

```luma
greet("Alice", last = "Wonder", title = "Ms.")  -- ✅
greet(last = "Wonder", "Alice")                 -- ❌ Error
```

## Anonymous Functions

Functions can be assigned to variables or passed directly:

```luma
let double = fn(x) do x * 2 end
print(double(5))          -- 10

let applyTwice = fn(f, x) do f(f(x)) end
print(applyTwice(double, 3))  -- 12 (3 * 2 * 2)
```

## Return Types

### Explicit Type Specification

The return type follows the parameter list:

```luma
fn factorial(n: Number): Number do
  if n <= 1 do
    1
  else do
    n * factorial(n - 1)
  end
end
```

### Type Inference

Return types can be inferred:

```luma
fn makeGreeter(greeting: String) do
  fn(name: String) do
    "${greeting}, ${name}!"
  end
end
```

### Void Functions

Functions that don't return meaningful values return `Null`:

```luma
fn printMessage(msg: String): Null do
  print(msg)
end
```

## Closures

Functions capture variables from their enclosing scope, creating closures:

```luma
let makeAdder = fn(x: Number) do
  fn(y: Number): Number do
    x + y              -- captures x from outer scope
  end
end

let add5 = makeAdder(5)
print(add5(3))        -- 8
print(add5(7))        -- 12
```

### Mutable Closures

Captured variables can be mutable:

```luma
let makeCounter = fn() do
  var count = 0        -- mutable
  fn(): Number do
    count = count + 1
    count
  end
end

let counter = makeCounter()
print(counter())      -- 1
print(counter())      -- 2
print(counter())      -- 3
```

## Higher-Order Functions

Functions that take other functions as arguments or return functions:

```luma
let map = fn(list: List(Any), transform: fn(Any): Any): List(Any) do
  let result = []
  for item in list do
    result.push(transform(item))
  end
  result
end

let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, fn(x) do x * 2 end)
print(doubled)        -- [2, 4, 6, 8, 10]
```

### Function Composition

Combine functions to create new functions:

```luma
let compose = fn(f, g) do
  fn(x) do
    f(g(x))
  end
end

let addOne = fn(x) do x + 1 end
let double = fn(x) do x * 2 end
let addThenDouble = compose(double, addOne)

print(addThenDouble(5))  -- double(addOne(5)) = 12
```

## Recursion

Functions can call themselves recursively:

```luma
fn fibonacci(n: Number): Number do
  if n <= 1 do
    n
  else do
    fibonacci(n - 1) + fibonacci(n - 2)
  end
end

print(fibonacci(6))   -- 8
```

## See Also

- [Variables and Bindings](./variables.md) — How to store functions
- [Control Flow](./control-flow.md) — Return statements and conditionals
- [Pattern Matching](../advanced/pattern-matching.md) — Advanced function patterns
- [Async/Await](../advanced/async-await.md) — Asynchronous functions
