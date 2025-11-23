---
sidebar_position: 4
---

# Types and Values

Luma has a rich type system that supports primitive types, composite types, generic types, and user-defined types.

## Type Categories

Luma has the following type categories:

- **Primitive types**: `Number`, `Boolean`, `String`, `Null`
- **Composite types**: `List(T)`, `Table`
- **Function types**: `fn(T1, T2): R`
- **Generic types**: `Result(T, E)`, `Option(T)`, `Promise(T)`
- **User-defined types**: Custom types and traits
- **Universal type**: `Any`

## Primitive Types

### Number

64-bit floating-point numbers (IEEE 754 double precision).

**Range:** ±5.0 × 10⁻³²⁴ to ±1.7 × 10³⁰⁸  
**Precision:** ~15-17 significant decimal digits

**Special values:**
- `Infinity` — positive infinity
- `-Infinity` — negative infinity
- `NaN` — not a number

### Boolean

Two values: `true` and `false`

### String

UTF-8 encoded, immutable text sequences.

### Null

The `null` type has a single value: `null`, representing the absence of a value.

## Composite Types

### List(T)

Generic, ordered collection of elements of type `T`.

**Operations:**
- Indexing: `list[index]`
- Length: `list.length()`
- Iteration: `for item in list`

**Example:**
```luma
let numbers: List(Number) = [1, 2, 3, 4, 5]
let first = numbers[0]              -- 1
let length = numbers.length()       -- 5
```

### Table

Unordered key-value mapping with string keys.

**Operations:**
- Access: `table.key` or `table["key"]`
- Membership: `"key" in table`
- Iteration: `for [key, value] in table`

**Example:**
```luma
let person = {
  name = "Alice",
  age = 30,
  email = "alice@example.com"
}

let name = person.name              -- "Alice"
let hasAge = "age" in person        -- true
```

## Generic Types

### Result(T, E)

Represents a value that is either a success (`ok`) or failure (`err`).

```luma
let Result = {
  ok = Any,
  err = Any
}
```

**Usage:**
```luma
fn divide(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    return { ok = null, err = "Division by zero" }
  end
  return { ok = a / b, err = null }
end
```

### Option(T)

Represents an optional value that may be `some(value)` or `none`.

```luma
let Option = {
  some = Any,
  none = Boolean
}
```

### Promise(T)

Represents an asynchronous computation that will eventually produce a value of type `T`.

See [§10 Concurrency and Async](../advanced/async-await.md).

## Type Any

The `Any` type is the supertype of all types. Any value can be assigned to an `Any` variable.

```luma
var value: Any = 42
value = "string"
value = [1, 2, 3]
```

## Defining Custom Types

Types are defined as tables with field specifications:

```luma
let Person = {
  name = String,
  age = Number,
  email = String,

  greet = fn(self: Person): String do
    return "Hello, I'm ${self.name}"
  end,

  new = fn(name: String, age: Number, email: String): Person do
    let raw = {
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
