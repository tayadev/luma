I AM MOVING THE SPEC TO /docs/specification.md, THIS FILE WILL BE DEPRECATED.

# The Luma Language Specification

**Version:** 0.1.0  
**Last Updated:** November 2025

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Types and Values](#3-types-and-values)
4. [Expressions](#4-expressions)
5. [Statements](#5-statements)
6. [Functions](#6-functions)
7. [Type System](#7-type-system)
8. [Pattern Matching](#8-pattern-matching)
9. [Error Handling](#9-error-handling)
10. [Concurrency and Async](#10-concurrency-and-async)
11. [Module System](#11-module-system)
12. [Memory Management](#12-memory-management)
13. [Standard Library](#13-standard-library)
14. [Grammar Reference](#14-grammar-reference)

---

## 1. Introduction

### 1.1 Design Philosophy

Luma is a typed scripting language designed with the following core principles:

- **Everything is a value** — Functions, types, and modules are first-class citizens
- **Explicit over implicit** — No hidden conversions or magical behavior
- **Safety without ceremony** — Type inference reduces boilerplate while maintaining safety
- **Errors as values** — No exceptions; all failures are explicit `Result` types
- **Async by design** — Native async/await with Promise-based concurrency
- **Modern tooling** — Built-in dependency management via URL imports
- **As little in the language core as possible** — Rely on a rich standard library for extended functionality (and the standard library is just another module and can be left out if not needed)

### 1.2 Notation Conventions

Throughout this specification:
- **Terminal symbols** are shown in `fixed-width font`
- **Non-terminal symbols** are shown in *italics*
- **Optional elements** are enclosed in square brackets: [*optional*]
- **Repeated elements** are indicated with ellipsis: *element*...

---

## 2. Lexical Structure

### 2.1 Source Code Encoding

Luma source files are UTF-8 encoded text files with the `.luma` extension.

### 2.2 Whitespace and Comments

Whitespace (spaces, tabs, newlines) is used to separate tokens but is otherwise insignificant.

**Single-line comments** begin with `--` and extend to the end of the line:
```luma
-- This is a single-line comment
```

**Multi-line comments** are enclosed in `--[[` and `]]`:
```luma
--[[
This is a multi-line comment
that spans multiple lines
]]
```

### 2.3 Keywords

The following identifiers are reserved keywords and cannot be used as variable names:

```
await     break     continue  do        else      end       false
fn        for       if        in        let       match     null
return    true      var       while
```

### 2.4 Identifiers

Identifiers start with a letter or underscore, followed by any sequence of letters, digits, or underscores:

```
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

**Naming conventions:**
- Variables and functions: `snake_case`
- Types and constructors: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### 2.5 Literals

#### 2.5.1 Numeric Literals

**Decimal integers:**
```luma
42
-17
0
```

**Floating-point numbers:**
```luma
3.14
-0.001
.5
```

**Scientific notation:**
```luma
1.5e3      -- 1500.0
2.5E-4     -- 0.00025
```

**Hexadecimal:**
```luma
0xFF       -- 255
0x1A3B     -- 6715
```

**Binary:**
```luma
0b1010     -- 10
0b1101     -- 13
```

#### 2.5.2 Boolean Literals

```luma
true
false
```

#### 2.5.3 Null Literal

```luma
null
```

#### 2.5.4 String Literals

**Basic strings** are enclosed in double quotes:
```luma
"Hello, World!"
```

**Multi-line strings** preserve line breaks:
```luma
"This is
a multiline
string."
```

**String interpolation** uses `${expression}`:
```luma
let name = "World"
"Hello, ${name}!"                           -- "Hello, World!"
"1 + 1 = ${1 + 1}"                         -- "1 + 1 = 2"
"Result: ${calculate(5, 3)}"               -- "Result: 8" (assuming calculate returns 8)
"Nested: ${if x > 0 do "positive" else do "negative" end}"  -- Depends on x
```

**Escape sequences:**
| Sequence | Meaning |
|----------|---------|
| `\"` | Literal double quote |
| `\\` | Literal backslash |
| `\n` | Newline |
| `\r` | Carriage return |
| `\t` | Tab |
| `\${` | Literal `${` (disable interpolation) |

#### 2.5.5 List Literals

Arrays are ordered, heterogeneous collections:
```luma
[1, 2, 3]
["apple", "banana", "cherry"]
[1, "mixed", true, null]
[]                          -- empty list
```

#### 2.5.6 Table Literals

Tables are unordered key-value mappings:
```luma
{
  key1 = "value1",
  key2 = 42,
  nested = {
    subkey = true
  }
}

{}                          -- empty table
```

**Key syntax:**
- Identifiers: `key = value`
- String keys: `"key with spaces" = value`
- Computed keys: `[expression] = value`

#### 2.5.7 Function Literals

```luma
fn(x: Number, y: Number): Number do
  return x + y
end
```

See [§6 Functions](#6-functions) for complete syntax.

---

## 3. Types and Values

### 3.1 Type Categories

Luma has the following type categories:

- **Primitive types**: `Number`, `Boolean`, `String`, `Null`
- **Composite types**: `List(T)`, `Table`
- **Function types**: `fn(T1, T2): R`
- **Generic types**: `Result(T, E)`, `Option(T)`, `Promise(T)`
- **User-defined types**: Custom types and traits
- **Universal type**: `Any`

### 3.2 Primitive Types

#### 3.2.1 Number

64-bit floating-point numbers (IEEE 754 double precision).

**Range:** ±5.0 × 10⁻³²⁴ to ±1.7 × 10³⁰⁸  
**Precision:** ~15-17 significant decimal digits

**Special values:**
- `Infinity` — positive infinity
- `-Infinity` — negative infinity
- `NaN` — not a number

#### 3.2.2 Boolean

Two values: `true` and `false`

#### 3.2.3 String

UTF-8 encoded, immutable text sequences.

#### 3.2.4 Null

The `null` type has a single value: `null`, representing the absence of a value.

### 3.3 Composite Types

#### 3.3.1 List(T)

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

#### 3.3.2 Table

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

### 3.4 Generic Types

#### 3.4.1 Result(T, E)

Represents a value that is either a success (`ok`) or failure (`err`).

```luma
let Result = {
  ok = Any,
  err = Any
}
```

**Usage:**
```luma
let divide = fn(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    return { ok = null, err = "Division by zero" }
  end
  return { ok = a / b, err = null }
end
```

#### 3.4.2 Option(T)

Represents an optional value that may be `some(value)` or `none`.

```luma
let Option = {
  some = Any,
  none = Boolean
}
```

#### 3.4.3 Promise(T)

Represents an asynchronous computation that will eventually produce a value of type `T`.

See [§10 Concurrency and Async](#10-concurrency-and-async).

### 3.5 Type Any

The `Any` type is the supertype of all types. Any value can be assigned to an `Any` variable.

```luma
var value: Any = 42
value = "string"
value = [1, 2, 3]
```

---

## 4. Expressions

### 4.1 Expression Categories

Expressions are syntactic constructs that evaluate to a value:

- **Literal expressions**: `42`, `"hello"`, `true`
- **Variable expressions**: `x`, `count`
- **Binary expressions**: `a + b`, `x * y`
- **Unary expressions**: `-x`, `not condition`
- **Call expressions**: `func(arg1, arg2)`
- **Member access**: `object.field`, `list[index]`
- **Block expressions**: `do ... end`
- **Conditional expressions**: `if ... then ... else ...`

### 4.2 Operator Precedence

Operators are listed from highest to lowest precedence:

| Precedence | Operator | Description | Associativity |
|------------|----------|-------------|---------------|
| 1 | `()` `[]` `.` | Call, index, member access | Left |
| 2 | `-` `!` | Unary minus, logical not | Right |
| 3 | `*` `/` `%` | Multiplication, division, modulo | Left |
| 4 | `+` `-` | Addition, subtraction | Left |
| 5 | `<` `<=` `>` `>=` | Comparison | Left |
| 6 | `==` `!=` | Equality | Left |
| 7 | `&&` | Logical and | Left |
| 8 | `||` | Logical or | Left |

### 4.3 Arithmetic Operators

```luma
x + y          -- addition (Numbers) or concatenation (Strings)
x - y          -- subtraction
x * y          -- multiplication
x / y          -- division
x % y          -- modulo
-x             -- unary negation
```

**Type requirements:** 
- `+`: Both operands must be `Number` (addition) or both must be `String` (concatenation)
- `-`, `*`, `/`, `%`, unary `-`: Both operands must be `Number`

### 4.4 Comparison Operators

```luma
x == y         -- equality
x != y         -- inequality
x < y          -- less than
x <= y         -- less than or equal
x > y          -- greater than
x >= y         -- greater than or equal
```

**Equality semantics:**
- Numbers: compared by value
- Strings: compared lexicographically
- Booleans: `true == true`, `false == false`
- Arrays/Tables: compared by reference
- `null == null` is `true`

### 4.5 Logical Operators

```luma
x && y         -- logical AND (short-circuits)
x || y         -- logical OR (short-circuits)
!x             -- logical NOT
```

**Short-circuit evaluation:**
- `x && y`: evaluates `y` only if `x` is truthy
- `x || y`: evaluates `y` only if `x` is falsy

**Truthiness:**
- Truthy: all values except `false` and `null`
- Falsy: `false` and `null`

### 4.6 String Operations

```luma
"Hello, " + "World!"       -- concatenation: "Hello, World!"
"value: ${x}"              -- interpolation
```

### 4.7 Member Access

**Dot notation:**
```luma
object.field
person.name
```

**Bracket notation:**
```luma
object["field"]
table["key with spaces"]
list[0]
```

### 4.8 Function Calls

```luma
func()                     -- no arguments
func(arg)                  -- single argument
func(arg1, arg2)           -- multiple arguments
func(a = 1, b = 2)         -- named arguments
```

**Parentheses are required** for all function calls.

### 4.8.1 Method Dispatch with `:`

Luma supports method dispatch using the colon operator `:`, similar to Lua. This provides convenient syntax for calling methods on objects where the object is automatically passed as the first argument (`self`).

**Syntax:**
```luma
object:method()
object:method(arg1, arg2)
object:method(name = value)
```

**Semantics:**

The `:` operator is syntactic sugar that automatically inserts the object as the first positional argument. These two calls are equivalent:

```luma
let dog = { name = "Rex", speak = fn(self, greeting): String do
  return "${greeting}, I'm ${self.name}"
end }

dog:speak("Woof")                -- method call with :
dog.speak(dog, "Woof")           -- equivalent regular call
```

**Key features:**
- The object before `:` is implicitly passed as the first argument
- All other arguments are passed normally after the object
- Named arguments work as expected
- The method must be a function stored in the object

**Example:**
```luma
let Vector2 = {
  x = 0,
  y = 0,
  
  magnitude = fn(self): Number do
    return (self.x * self.x + self.y * self.y).sqrt()
  end,
  
  add = fn(self, other: Table): Table do
    return {
      x = self.x + other.x,
      y = self.y + other.y
    }
  end
}

let v1 = Vector2
let v2 = { x = 3, y = 4 }

-- Method dispatch with :
let mag = v1:magnitude()              -- calls magnitude(v1)
let result = v1:add(v2)               -- calls add(v1, v2)

-- Equivalent to:
let mag2 = v1.magnitude(v1)
let result2 = v1.add(v1, v2)
```

**Operator precedence:**

The `:` operator has the same precedence as `.` and `[]`, forming left-associative postfix operations:

```luma
object:method(arg1):field[0]        -- valid chaining
obj:method1():method2()              -- chaining method calls
```

### 4.9 Block Expressions

Blocks evaluate to the value of their last expression:

```luma
let result = do
  let x = 10
  let y = 20
  x + y                    -- returns 30
end
```

If the last statement is not an expression, the block returns `null`.

---

## 5. Statements

### 5.1 Variable Declarations

#### 5.1.1 Immutable Bindings (`let`)

```luma
let name = "Luma"
let count: Number = 42
let value = calculate()
```

`let` bindings cannot be reassigned after initialization.

#### 5.1.2 Mutable Variables (`var`)

```luma
var counter = 0
counter = counter + 1              -- valid

var total: Number = 0
total = 100
```

`var` variables can be reassigned multiple times.

#### 5.1.3 Type Annotations

Type annotations are optional; types are inferred when omitted:

```luma
let x: Number = 42                 -- explicit
let y = 42                         -- inferred as Number
let z: Any = "flexible"            -- explicit Any
```

### 5.2 Destructuring

#### 5.2.1 List Destructuring

```luma
let [first, second, third] = [1, 2, 3]

let [head, ...tail] = [1, 2, 3, 4]
-- head = 1, tail = [2, 3, 4]

let [a, b, ...] = [10, 20, 30, 40]
-- a = 10, b = 20, rest ignored

let [x, _, z] = [1, 2, 3]
-- x = 1, z = 3, second element ignored via wildcard
```

**Behavior:**
- Missing elements assign `null`
- `...name` captures remaining elements as list
- `...` without name discards remaining elements
- `_` wildcard ignores a single element

#### 5.2.2 Table Destructuring

```luma
let person = { name = "Alice", age = 30, city = "NYC" }
let { name, age } = person
-- name = "Alice", age = 30

let { name: userName, age: userAge } = person
-- userName = "Alice", userAge = 30
```

### 5.3 Assignment

```luma
variable = expression
```

Only `var` variables can be reassigned. Attempting to reassign a `let` binding is a compile error.

### 5.4 Expression Statements

Any expression can be used as a statement:

```luma
print("Hello")
calculate()
x + y
```

If the expression evaluates to a value, it's discarded (unless it's the last expression in a block).

### 5.5 Return Statements

```luma
return expression
return                             -- returns null
```

**Implicit returns:**
If a function/block doesn't end with explicit `return`, the last expression becomes an implicit return:

```luma
fn add(a: Number, b: Number): Number do
  a + b                            -- implicit return
end
```

### 5.6 Conditional Statements

```luma
if condition do
  -- executed if condition is truthy
end

if condition do
  -- if branch
else do
  -- else branch
end

if condition1 do
  -- branch 1
else if condition2 do
  -- branch 2
else if condition3 do
  -- branch 3
else do
  -- default branch
end
```

**Conditional expressions:**
`if` can be used as an expression:

```luma
let max = if a > b do a else do b end
```

### 5.7 Loops

#### 5.7.1 While Loops

```luma
while condition do
  -- body
end
```

#### 5.7.2 Do-While Loops

```luma
do
  -- body (executes at least once)
while condition end
```

#### 5.7.3 For-In Loops

```luma
for item in iterable do
  -- body
end
```

**Loop variables are immutable** and scoped to the loop body.

**List iteration:**
```luma
for item in [1, 2, 3] do
  print(item)
end
```

**Table iteration:**
```luma
for [key, value] in table do
  print(key, value)
end
```

**Range iteration:**
```luma
for n in range(1, 10) do
  print(n)
end
```

**Indexed iteration:**
```luma
for [item, index] in list.indexed() do
  print(index, item)
end
```

#### 5.7.4 Break and Continue

```luma
break                              -- exit innermost loop
break 2                            -- exit 2 nested loops

continue                           -- skip to next iteration
continue 2                         -- skip in outer loop
```

---

## 6. Functions

### 6.1 Function Definition

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

### 6.2 Parameters

#### 6.2.1 Required Parameters

```luma
fn greet(name: String): String do
  return "Hello, ${name}!"
end
```

#### 6.2.2 Optional Parameters

```luma
fn greet(name: String, title: String = "Friend"): String do
  return "Hello, ${title} ${name}!"
end

greet("Alice")                     -- "Hello, Friend Alice!"
greet("Bob", "Dr.")                -- "Hello, Dr. Bob!"
```

### 6.3 Function Calls

#### 6.3.1 Positional Arguments

```luma
add(2, 3)
greet("Alice", "Ms.")
```

#### 6.3.2 Named Arguments

```luma
add(a = 2, b = 3)
greet(name = "Alice", title = "Dr.")
greet(title = "Dr.", name = "Alice")    -- order doesn't matter
```

**Mixing positional and named:**
```luma
greet("Alice", title = "Dr.")      -- positional then named
```

### 6.4 Return Types

#### 6.4.1 Explicit Returns

```luma
fn factorial(n: Number): Number do
  if n <= 1 do
    return 1
  end
  return n * factorial(n - 1)
end
```

#### 6.4.2 Implicit Returns

```luma
fn add(a: Number, b: Number): Number do
  a + b                            -- last expression returned
end
```

#### 6.4.3 Void Functions

Functions that don't return a meaningful value return `null`:

```luma
fn printMessage(msg: String): Null do
  print(msg)
end
```

### 6.5 Closures

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

### 6.6 Higher-Order Functions

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

---

## 7. Type System

### 7.1 Type Inference

Luma infers types when annotations are omitted:

```luma
let x = 42                         -- Number
let name = "Alice"                 -- String
let items = [1, 2, 3]              -- List(Number)
```

### 7.2 User-Defined Types

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

### 7.3 Type Casting

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

### 7.4 Traits

Traits define interfaces that types can implement:

```luma
let Drawable = {
  draw = fn(self: Any): String
}

let Circle = {
  radius = Number,

  draw = fn(self: Circle): String do
    return "Circle with radius ${self.radius}"
  end
}
```

**Structural typing:** Types satisfy traits if they have the required fields, checked at `cast()` time.

### 7.5 Inheritance

Types can inherit from parent types using `__parent`:

```luma
let Animal = {
  name = String,

  speak = fn(self: Animal): String do
    return "..."
  end
}

let Dog = {
  __parent = Animal,
  breed = String,

  speak = fn(self: Dog): String do
    return "Woof! I'm ${self.name}, a ${self.breed}"
  end
}
```

**Inheritance semantics:**
- Child types inherit all fields from parent
- Methods can be overridden
- Constructor must initialize both parent and child fields

### 7.6 Type Checking

Check if a value is an instance of a type:

```luma
if isInstanceOf(value, Dog) do
  print("It's a dog!")
end
```

### 7.7 Operator Overloading

Types can overload operators by defining special methods:

| Operator | Method | Signature |
|----------|--------|-----------|
| `+` | `__add` | `fn(T, T): T` |
| `-` | `__sub` | `fn(T, T): T` |
| `*` | `__mul` | `fn(T, T): T` |
| `/` | `__div` | `fn(T, T): T` |
| `%` | `__mod` | `fn(T, T): T` |
| unary `-` | `__neg` | `fn(T): T` |
| `==` | `__eq` | `fn(T, T): Boolean` |
| `<` | `__lt` | `fn(T, T): Boolean` |
| `<=` | `__le` | `fn(T, T): Boolean` |
| `>` | `__gt` | `fn(T, T): Boolean` |
| `>=` | `__ge` | `fn(T, T): Boolean` |

**Example:**
```luma
let Vector2 = {
  x = Number,
  y = Number,

  __add = fn(a: Vector2, b: Vector2): Vector2 do
    return Vector2.new(a.x + b.x, a.y + b.y)
  end,

  __eq = fn(a: Vector2, b: Vector2): Boolean do
    return a.x == b.x && a.y == b.y
  end,

  new = fn(x: Number, y: Number): Vector2 do
    return cast(Vector2, { x = x, y = y })
  end
}

let v1 = Vector2.new(1, 2)
let v2 = Vector2.new(3, 4)
let v3 = v1 + v2                   -- Vector2(4, 6)
```

**Non-overloadable operators:** `&&`, `||`, `!`, `[]`, `in`, `.`

### 7.8 Type Conversions

Types can define conversions using `__into`:

```luma
let Weight = {
  grams = Number,

  __into = fn(self: Weight, target: Type): String do
    if target == String do
      return "${self.grams}g"
    end
    return null
  end,

  new = fn(grams: Number): Weight do
    return cast(Weight, { grams = grams })
  end
}

let w = Weight.new(500)
print(w)                           -- "500g"
```

The `print()` function internally uses `into(String)`.

---

## 8. Pattern Matching

### 8.1 Match Expression

The `match` construct is an expression that evaluates to the value of the selected branch:

```luma
let status = match response.code do
  200 do "success" end
  404 do "not found" end
  500 do "server error" end
  _ do "unknown" end
end
```

Match can also be used as a statement when the result is not needed:

```luma
match value do
  pattern1 do
    -- branch 1
  end
  pattern2 do
    -- branch 2
  end
  _ do
    -- default case
  end
end
```

### 8.2 Pattern Types

#### 8.2.1 Literal Patterns

```luma
match x do
  0 do print("zero") end
  1 do print("one") end
  _ do print("other") end
end
```

#### 8.2.2 Type Patterns

Match on Result/Option types:

```luma
match result do
  ok do
    print("Success: ${result.ok}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

#### 8.2.3 Wildcard Pattern

The `_` pattern matches any value:

```luma
match value do
  _ do print("matches anything") end
end
```

### 8.3 Exhaustiveness

Pattern matching must be exhaustive. If not all cases are covered, a `_` wildcard is required.

---

## 9. Error Handling

### 9.1 Errors as Values

Luma has no exceptions. All errors are represented as values using the `Result` type.

```luma
let Result = {
  ok = Any,
  err = Any
}
```

### 9.2 Returning Errors

```luma
fn readFile(path: String): Result(String, Error) do
  let data, err = fs.read(path)
  if err != null do
    return { ok = null, err = err }
  end
  return { ok = data, err = null }
end
```

### 9.3 Handling Errors

#### 9.3.1 Pattern Matching

```luma
let result = readFile("data.txt")
match result do
  ok do
    print("File contents: ${result.ok}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

#### 9.3.2 Conditional Checking

```luma
let result = readFile("data.txt")
if result.err != null do
  print("Error: ${result.err}")
  return
end
let data = result.ok
```

### 9.4 Error Propagation

```luma
fn processFile(path: String): Result(String, Error) do
  let result = readFile(path)
  if result.err != null do
    return result                  -- propagate error
  end

  let processed = transform(result.ok)
  return { ok = processed, err = null }
end
```

### 9.5 Custom Error Types

```luma
let FileError = {
  __parent = Error,
  path = String,
  reason = String
}

fn readFile(path: String): Result(String, FileError) do
  -- implementation
end
```

---

## 10. Concurrency and Async

### 10.1 Promises

Asynchronous operations return `Promise(T)`:

```luma
let fetchData = fn(url: String): Promise(String) do
  -- returns promise
end
```

### 10.2 Async/Await

#### 10.2.1 The await Keyword

`await` suspends execution until a promise resolves:

```luma
let fetchUser = fn(id: String): Result(User, Error) do
  let response = await http.get("/users/${id}")
  return parseUser(response)
end
```

**Type transformation:**
- `await Promise(T)` → `T`

#### 10.2.2 Async Function Inference

Functions are automatically async if they:
1. Contain `await` expressions
2. Return a `Promise` type

```luma
fn fetchAndProcess(id: String): Promise(Result(Data, Error)) do
  let raw = await fetchData(id)
  return process(raw)
end
```

### 10.3 Sequential vs Concurrent

#### 10.3.1 Sequential Execution

```luma
let data1 = await fetch("/api/data1")
let data2 = await fetch("/api/data2")
-- Total time: time1 + time2
```

#### 10.3.2 Concurrent Execution

```luma
let promise1 = fetch("/api/data1")
let promise2 = fetch("/api/data2")
let data1 = await promise1
let data2 = await promise2
-- Total time: max(time1, time2)
```

### 10.4 Error Handling with Async

```luma
fn fetchUser(id: String): Promise(Result(User, Error)) do
  let response = await http.get("/users/${id}")
  if response.err != null do
    return { ok = null, err = response.err }
  end
  return parseUser(response.ok)
end
```

---

## 11. Module System

### 11.1 Import Syntax

```luma
let module = import("source")
```

`import()` is a built-in function that loads and evaluates a module, returning its exported value.

### 11.2 Import Sources

#### 11.2.1 Local Files

```luma
let utils = import("./utils.luma")
let lib = import("../lib/helpers.luma")
```

#### 11.2.2 HTTP/HTTPS URLs

```luma
let http = import("https://example.com/http.luma")
```

#### 11.2.3 Git Repositories

```luma
let lib = import("git@github.com:user/repo.git")
let tagged = import("gh:user/repo@1.2.3")
```

### 11.3 Module Resolution

**For URLs:**
1. Download file to local cache (`~/.luma/cache`)
2. Verify integrity (if lockfile exists)
3. Parse and evaluate module
4. Return module's exported value

**For directories:**
- If path is directory, look for `main.luma`

### 11.4 Module Exports

Modules export the value of their last expression:

```luma
-- math.luma
let pi = 3.14159

let add = fn(a: Number, b: Number): Number do
  return a + b
end

{
  pi = pi,
  add = add
}
```

```luma
-- main.luma
let math = import("./math.luma")
print(math.pi)                     -- 3.14159
print(math.add(2, 3))              -- 5
```

### 11.5 Dependency Locking

Dependencies are locked in `luma.lock`:

```json
{
  "https://example.com/http.luma": {
    "version": "1.2.3",
    "integrity": "sha256-...",
    "resolved": "2024-01-15T10:30:00Z"
  }
}
```

### 11.6 Circular Dependencies

Circular imports are detected and result in an error:

```
Error: Circular dependency detected:
  a.luma -> b.luma -> a.luma
```

---

## 12. Memory Management

### 12.1 Garbage Collection

Luma uses automatic garbage collection. All heap-allocated values (tables, arrays, functions, closures) are managed by the GC.

**Collection triggers:**
- When allocation threshold is reached
- When memory pressure is high
- Manual collection via `gc.collect()` (if exposed)

### 12.2 Object Lifecycle

#### 12.2.1 Allocation

```luma
let obj = { x = 10, y = 20 }       -- allocated on heap
```

#### 12.2.2 Finalization

Objects can define a `__gc` method called during garbage collection:

```luma
let Resource = {
  handle = Any,

  __gc = fn(self: Resource): Null do
    print("Resource ${self.handle} being collected")
    cleanup(self.handle)
  end
}
```

**Note:** Finalization is not guaranteed to run immediately or in any particular order.

### 12.3 Reference Semantics

**Value types** (numbers, booleans, null): copied by value  
**Reference types** (tables, arrays, functions): copied by reference

```luma
let a = [1, 2, 3]
let b = a                          -- b references same list
b[0] = 99
print(a[0])                        -- 99
```

---

## 13. Standard Library

### 13.1 Core Functions

#### 13.1.1 I/O

```luma
print(value: Any): Null
-- Prints value to stdout (uses __into(String))
```

#### 13.1.2 Type Operations

```luma
cast(type: Type, value: Any): Type
-- Validates and casts value to type

isInstanceOf(value: Any, type: Type): Boolean
-- Checks if value is instance of type

typeof(value: Any): Type
-- Returns runtime type of value
```

## 14. Grammar Reference

### 14.1 Lexical Grammar

```
Program         ::= Statement*

Statement       ::= LetStmt | VarStmt | ExprStmt | ReturnStmt 
                  | IfStmt | WhileStmt | DoWhileStmt | ForStmt
                  | BreakStmt | ContinueStmt

LetStmt         ::= "let" Pattern [":" Type] "=" Expr
VarStmt         ::= "var" Identifier [":" Type] "=" Expr
ExprStmt        ::= Expr
ReturnStmt      ::= "return" [Expr]
BreakStmt       ::= "break" [Number]
ContinueStmt    ::= "continue" [Number]

IfStmt          ::= "if" Expr "do" Block ("else" "if" Expr "do" Block)* ["else" "do" Block] "end"
WhileStmt       ::= "while" Expr "do" Block "end"
DoWhileStmt     ::= "do" Block "while" Expr "end"
ForStmt         ::= "for" Pattern "in" Expr "do" Block "end"
```

**Note:** `match` is defined as an expression (`MatchExpr`) rather than a statement. See Expression Grammar below.

### 14.2 Expression Grammar

```
Expr            ::= LogicalOrExpr

LogicalOrExpr   ::= LogicalAndExpr ("||" LogicalAndExpr)*
LogicalAndExpr  ::= EqualityExpr ("&&" EqualityExpr)*
EqualityExpr    ::= ComparisonExpr (("==" | "!=") ComparisonExpr)*
ComparisonExpr  ::= AdditiveExpr (("<" | "<=" | ">" | ">=") AdditiveExpr)*
AdditiveExpr    ::= MultiplicativeExpr (("+" | "-") MultiplicativeExpr)*
MultiplicativeExpr ::= UnaryExpr (("*" | "/" | "%") UnaryExpr)*
UnaryExpr       ::= ("-" | "!") UnaryExpr | PostfixExpr
PostfixExpr     ::= PrimaryExpr ("(" [ArgList] ")" | "[" Expr "]" | "." Identifier)*

PrimaryExpr     ::= Literal | Identifier | "(" Expr ")" | BlockExpr 
                  | FunctionExpr | IfExpr | MatchExpr | ArrayExpr | TableExpr

BlockExpr       ::= "do" Statement* "end"
FunctionExpr    ::= "fn" "(" [ParamList] ")" [":" Type] "do" Block "end"
IfExpr          ::= "if" Expr "do" Block ["else" "do" Block] "end"
MatchExpr       ::= "match" Expr "do" MatchArm+ "end"
ArrayExpr       ::= "[" [ExprList] "]"
TableExpr       ::= "{" [FieldList] "}"

MatchArm        ::= Pattern "do" Block "end"
```

### 14.3 Pattern Grammar

```
Pattern         ::= Identifier | ArrayPattern | TablePattern | "_"
ArrayPattern    ::= "[" [PatternList] ["," "..." [Identifier]] "]"
TablePattern    ::= "{" [FieldPatternList] "}"
```

### 14.4 Type Grammar

```
Type            ::= TypeIdent | GenericType | FunctionType | "Any"
GenericType     ::= TypeIdent "(" Type ("," Type)* ")"
FunctionType    ::= "fn" "(" [TypeList] ")" ":" Type
```

---

## Appendix A: Reserved Keywords

```
await     break     continue  do        else      end
false     fn        for       if        in        let
match     null      return    true      var       while
```

## Appendix B: Standard Library Module List

- `math` — Mathematical functions
- `string` — String manipulation
- `list` — List utilities
- `table` — Table utilities
- `fs` — File system operations
- `os` — Operating system interaction

## Appendix C: Error Types

- `Error` — Base error type
- `TypeError` — Type mismatch or casting error
- `ValueError` — Invalid value error
- `IOError` — I/O operation failure
- `NetworkError` — Network operation failure
- `ParseError` — Parsing failure

---

**End of Specification**
