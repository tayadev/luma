---
sidebar_position: 4
---

# Types and Values

Luma has a **strong, static type system** that combines the safety of explicit types with the convenience of type inference. Every value has a type, and the type checker ensures type safety at compile time.

## Type System Overview

Luma supports:

- **Primitive types** — Basic building blocks: `Number`, `Boolean`, `String`, `Null`
- **Composite types** — Collections: `List(T)`, `Table`
- **Function types** — `fn(T1, T2): R`
- **Generic types** — `Result(T, E)`, `Option(T)`, `Promise(T)`
- **Union types** — Multiple possible types (planned)
- **User-defined types** — Custom types and classes (planned)
- **Universal type** — `Any` for flexibility

## Primitive Types

### Number

IEEE 754 double-precision floating-point numbers.

**Range:** ±5.0 × 10⁻³²⁴ to ±1.7 × 10³⁰⁸  
**Precision:** ~15-17 significant decimal digits

```luma
let integer = 42
let float = 3.14159
let negative = -100
let scientific = 1.5e-10
let infinity = Infinity
let notANumber = NaN
```

**Common operations:**
```luma
10 + 5              -- 15
10 - 3              -- 7
4 * 5               -- 20
10 / 3              -- 3.333...
10 % 3              -- 1 (modulo)
2 ^ 8               -- 256 (exponentiation)
```

### Boolean

Two truth values: `true` and `false`

```luma
let isActive = true
let isDisabled = false
let hasPermission = x > 10 and y < 20
```

**Truthiness:**
- **Truthy:** any value except `false` and `null`
- **Falsy:** `false` and `null`

```luma
if x do print("x is truthy") end
if not empty do print("not empty") end
```

### String

UTF-8 encoded, immutable text sequences.

```luma
let name = "Alice"
let multiline = "Line 1
Line 2"
let empty = ""
```

**String Interpolation:**
```luma
let age = 30
let greeting = "Hello, I am ${age} years old"
print(greeting)     -- Hello, I am 30 years old
```

**Escape Sequences:**
```luma
"Quote: \"hello\""  -- "hello"
"Tab:\t\tindented"
"Newline:\nSecond line"
```

### Null

The `Null` type has a single value: `null`, representing absence or void.

```luma
let nothing = null
let result = null  -- returned from functions with no meaningful result

if value == null do
  print("Value is null")
end
```

## Composite Types

### List(T)

Generic, ordered collection of elements of type `T`.

```luma
let numbers: List(Number) = [1, 2, 3, 4, 5]
let strings: List(String) = ["a", "b", "c"]
let mixed: List(Any) = [1, "two", true]
let empty: List(Any) = []
```

**Indexing:**
```luma
let numbers = [10, 20, 30]
let first = numbers[0]        -- 10
let last = numbers[2]         -- 30
let invalid = numbers[10]     -- null (out of bounds is safe)
```

**Iteration:**
```luma
for num in [1, 2, 3] do
  print(num)
end
```

**Built-in operations (planned):**
```luma
numbers.length()              -- 5
numbers.push(6)               -- add to end
numbers.pop()                 -- remove from end
numbers.reverse()             -- reverse in place
```

### Table (Record)

Unordered collection of key-value pairs with string keys.

```luma
let person = {
  name = "Alice",
  age = 30,
  email = "alice@example.com"
}

let config = {
  debug = true,
  timeout = 5000,
  retries = 3
}

let empty = {}
```

**Field Access:**
```luma
let name = person.name        -- "Alice"
let age = person["age"]       -- 30 (bracket notation)
let missing = person.phone    -- null (safe access)
```

**Membership Testing:**
```luma
if "email" in person do
  print("Has email")
end
```

**Updating Fields:**
```luma
var data = { x = 1, y = 2 }
data.x = 10                   -- update field
data.z = 3                    -- add new field
```

## Function Types

Functions have types based on their parameters and return type:

```luma
let add: fn(Number, Number): Number = fn(a, b) do
  a + b
end

let transform: fn(String): Number = fn(s) do
  s.length()
end

let callback: fn(): Null = fn() do
  print("Done")
end
```

## Generic Types

### Result(T, E)

Represents either a success (`ok`) or failure (`err`). Used for error handling without exceptions.

```luma
fn divide(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    { ok = null, err = "Division by zero" }
  else do
    { ok = a / b, err = null }
  end
end

let result = divide(10, 2)
if result.err != null do
  print("Error: ${result.err}")
else do
  print("Result: ${result.ok}")
end
```

### Option(T)

Represents an optional value: either `some(value)` or `none`.

```luma
fn findUser(id: String): Option(User) do
  if user_exists do
    { some = user, none = null }
  else do
    { some = null, none = true }
  end
end
```

### Promise(T)

Represents an asynchronous computation that will eventually produce a value of type `T`. See [Async/Await](../advanced/async-await.md).

```luma
fn fetchData(url: String): Promise(String) do
  -- async operation
end

let data = await fetchData("https://api.example.com/data")
```

## Type Any

The `Any` type is compatible with all types. Use it for maximum flexibility when you need to accept or return values of unknown type:

```luma
let flexible: Any = 42
flexible = "now it's a string"
flexible = [1, 2, 3]
flexible = fn() do "a function" end
```

:::warning
Use `Any` sparingly. It bypasses type safety. Prefer specific types when possible.
:::

## Type Annotations

### Annotation Syntax

Use `: TypeName` to declare types explicitly:

```luma
let x: Number = 42
let name: String = "Alice"
let items: List(String) = ["a", "b"]
let result: Result(Number, String) = { ok = 5, err = null }
```

### Generic Type Parameters

Generic types use parentheses for parameters:

```luma
let numbers: List(Number) = [1, 2, 3]
let result: Result(String, Error) = { ok = "success", err = null }
let callback: fn(String): Boolean = fn(s) do s.length() > 0 end
```

## Type Inference

Luma infers types from context, reducing the need for explicit annotations:

```luma
let x = 42                    -- inferred as Number
let name = "Alice"            -- inferred as String
let active = true             -- inferred as Boolean
let items = [1, 2, 3]         -- inferred as List(Number)

let greet = fn(n) do
  "Hello, ${n}"               -- inferred to return String
end
```

## Type Conversions

:::info
Explicit type conversion is planned. The language currently emphasizes type safety to prevent implicit conversions.
:::
      name = name,
      age = age,
      email = email
    }
    return cast(Person, raw)
  end
}
```

## Creating Instances

Use the `new` method (or any constructor pattern):

```luma
let alice = Person.new("Alice", 30, "alice@example.com")
let greeting = alice.greet()
```

## Type Casting

The `cast` function validates and converts values to a specific type:

```luma
let person = cast(Person, {
  name = "Alice",
  age = 30,
  email = "alice@example.com"
})
```

**Behavior:**
- Validates all required fields are present and correct types
- Attaches type metadata
- Merges inherited fields (if `__parent` is defined)
- Returns typed value or throws error on validation failure

## Type Checking

Check if a value is an instance of a type:

```luma
if isInstanceOf(value, Person) do
  print("It's a person!")
end
```

## Next Steps

Learn about [traits](../reference/traits.md), [inheritance](../reference/inheritance.md), or [operator overloading](../reference/operators.md).
