# Language Specification

## 1. Introduction

## 2. Lexical Structure

### 2.1 Source Files

Luma source files are plain text files encoded in UTF-8. The file extension for Luma source files is `.luma`.

### 2.2 Whitespace

Whitespace in Luma is generally not significant, except where it is used to separate tokens. Whitespace characters include spaces, tabs, and newline characters.

### 2.3 Comments

Luma has two types of comments: single-line comments and multi-line comments.

#### 2.3.1 Single-line Comments

Single-line comments start with `--` and continue to the end of the line. They dont have to start at the beginning of the line; they can be placed after code as well.

```luma
-- This is a single-line comment
let x = 10  -- This is a single-line comment
```

#### 2.3.2 Multi-line Comments

Multi-line comments start with `--[[` and end with `]]`. They can span multiple lines.

```luma
--[[
This is a
multi-line comment
]]
```

### 2.4 Keywords

The following identifiers are reserved as keywords in Luma and cannot be used as names for variables:

```
await     break     continue  do        else      end       false
fn        for       if        in        let       match     null
return    true      var       while
```

### 2.5 Identifiers

Identifiers in Luma must start with a letter (a-z, A-Z) or an underscore (_), followed by any combination of letters, digits (0-9), and underscores.

### 2.6 Literals

#### 2.6.1 Numeric Literals

Decimal integers: `0`, `42`, `-7`

Floating-point numbers: `3.14`, `-0.001`, `.42`

Scientific notation: `1e10`, `2.5E-3`

Hexadecimal: `0x1A3F`, `0Xabc17f`

Binary: `0b101010`, `0B1101`

#### 2.6.2 Boolean Literals

`true`, `false`

#### 2.6.3 Null Literal

`null`

#### 2.6.4 String Literals

String literals are enclosed in double quotes (`"`). They can contain escape sequences for special characters. They can span multiple lines.

```luma
"This is a string literal"
"String with escape sequences: \n \t \" \\"

"This is a multi-line
string literal"

"Result: ${1 + 2}"
```

Escape sequences supported in string literals:
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab
- `\"` - Double quote
- `\\` - Backslash
- `${expression}` - String interpolation
- `\${` - Literal `${`

### 2.6.5 List Literals

List literals are enclosed in square brackets (`[` and `]`), with elements separated by commas.

```luma
[1, 2, 3, 4]
["apple", "banana", "cherry"]
[]
```

### 2.6.6 Table Literals

Tables are unordered collections of key-value pairs, enclosed in curly braces (`{` and `}`).

```luma
{
  name = "Alice",
  age = 30,
  ["isStudent"] = false,
  nested = {
    key = "value"
  }
}

{}
```

### 2.6.7 Function Literals
Function literals are defined using the `fn` keyword, followed by parameters, an optional return type, and a function body.

```luma
fn(x: Number, y: Number): Number do
  x + y
end
```
See [§6 Functions](#6-functions) for complete syntax.

## 3. Types and Values

### 3.1 Primitive Types

Luma has the following primitive types:
- `Number` - 64-bit floating-point numbers (IEEE 754 double precision float)
  - Special values: `NaN`, `Infinity`, `-Infinity`
- `Boolean` - `true` and `false`
- `null` - Represents the absence of a value
- `String` - UTF-8 encoded text

### 3.2 Composite Types

- `List(T)` - Ordered collections of values of type `T`
- `Table(K, V)` - Unordered collections of key-value pairs, where keys are of type `K` and values are of type `V`

> Both can be heterogeneous, e.g., `List(Any)` or `Table(Any, Any)`

### 3.3 Any Type

The `Any` type is a supertype that can represent any value in Luma.

### 3.4 Options, Results, and Promises

- `Option(T)` - Represents an optional value of type `T`, which can be either `Some(value)` or `None`
- `Result(T, E)` - Represents either a success (`Ok(value)`) of type `T` or an error (`Err(error)`) of type `E`
- `Promise(T)` - Represents a async computation that will eventually yield a value of type `T`

### 3.5 Function Types
Function types are denoted as `Function(T1, T2, ..., Tn): R`, where `T1` to `Tn` are the parameter types and `R` is the return type.

### 3.6 Type Inference

Luma features strong static type inference, allowing the compiler to automatically deduce types in many cases, reducing the need for explicit type annotations.

## 4. Expressions

### 4.1 Expression Categories

Expressions are constructs that evaluate to values.

- Literals
- Variable references
- Binary expressions
- Unary expressions
- Function calls
- Member access
- Block expressions
- Conditional expressions
- Match expressions

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

> Note: Addition (`+`) is overloaded for both numeric addition and string concatenation.

### 4.4 Comparison Operators

```luma
x == y         -- equality
x != y         -- inequality
x < y          -- less than
x <= y         -- less than or equal to
x > y          -- greater than
x >= y         -- greater than or equal to
```

### 4.5 Logical Operators

```luma
x && y        -- logical and (short-circuiting)
x || y        -- logical or (short-circuiting)
!x            -- logical not
```

### 4.6 Member Access

```luma
obj.field        -- access field 'field' of object 'obj'
obj["field"]     -- access field 'field' of object 'obj' using string key
list[0]         -- access first element of list 'list'
```

### 4.7 Function Calls

```luma
func()                  -- call function 'func' with no arguments
func(arg1, arg2, ...)   -- call function 'func' with arguments
func(arg1 = val1, arg2 = val2)  -- call function 'func' with named arguments
func(arg1, arg2 = val2)  -- call function 'func' with mixed positional and named arguments
```

### 4.8 Method Dispatch

Luma supports method dispatch using the colon operator `:`, similar to Lua. This provides convenient syntax for calling methods on objects where the object is automatically passed as the first argument.
> It is convention to call the first parameter `self` in method definitions.

```luma
obj.method()          -- equivalent to obj.method(obj)
obj:method(arg1, arg2)  -- equivalent to obj.method(obj, arg1, arg2)
obj:method(name = val)  -- equivalent to obj.method(obj, name = val)
```

### 4.9 Block Expressions

Blocks are enclosed in `do` and `end`, containing a sequence of expressions. The  value of the block is the value of the last expression or what is returned using the `return` statement.

```luma
do
  let x = 10
  let y = 20
  x + y  -- value of the block is 30
end
```

> If the last statement isnt an expression, the block evaluates to `null`.

### 4.10 Conditional Expressions

Conditional expressions use `if`, `else if`, and `else` to evaluate conditions.

```luma
let result = if condition1 do
  "Condition 1 is true"
else if condition2 do
  "Condition 2 is true"
else
  "Neither condition is true"
end
```

## 5. Statements

### 5.1 Variable Declaration

#### 5.1.1 Immutable Bindings

Immutable variables are declared using the `let` keyword. Once assigned, their values cannot be changed.

```luma
let x: Number = 10
let name = "Alice"
```

> Type annotations are optional; the compiler can infer types in most cases.

#### 5.1.2 Mutable Bindings

Mutable variables are declared using the `var` keyword. Their values can be changed after assignment.

```luma
var count: Number = 0
count = count + 1
```

> Mutable variables require a initial value, if you dont know the value yet, use `Option(T)` type with `None`.

### 5.2 Assignment

Assignment is done using the `=` operator for mutable variables.

```luma
var x: Number = 10
x = 20  -- valid
```

### 5.3 Destructuring Assignment

#### 5.3.1 List Destructuring

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

#### 5.3.2 Table Destructuring

```luma
let person = { name = "Alice", age = 30, city = "NYC" }
let { name, age } = person
-- name = "Alice", age = 30

let { name: userName, age: userAge } = person
-- userName = "Alice", userAge = 30
```

### 5.4 Expression Statements

Expressions can be used as statements. The expression is evaluated, and its value is discarded unless it is the last expression in a block.

### 5.5 Return Statement

The `return` statement is used to exit a function and optionally return a value.

```luma
return expression
return   -- returns null
```

> Reminder: the value of the last expression in a block is also returned implicitly.

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

#### 5.7.4 Break and Continue

```luma
break                              -- exit innermost loop
break 2                            -- exit 2 nested loops

continue                           -- skip to next iteration
continue 2                         -- skip in outer loop
```

### 6. Functions

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

Functions that don't return a meaningful value return `null` and dont need an explicit return type.

```luma
fn printMessage(msg: String) do
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

### 7. Type System

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

The `print()` function internally uses `.into(String)`.

### 8. Pattern Matching

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

### 9. Modules and Imports

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

Dependencies are locked by adding a second argument to `import()`:

```luma
let lib = import("https://example.com/lib.luma", "sha256:abcdef1234567890...")
```

### 11.6 Circular Dependencies

Circular imports are detected and result in an error:

```
Error: Circular dependency detected:
  a.luma -> b.luma -> a.luma
```

### 10. Async and Concurrency

### 11. Memory Management

### 11.1 Garbage Collection

Luma uses automatic garbage collection. All heap-allocated values (tables, arrays, functions, closures) are managed by the GC.

**Collection triggers:**
- When allocation threshold is reached
- When memory pressure is high
- Manual collection via `gc.collect()` (if exposed)

### 11.2 Object Lifecycle

#### 11.2.1 Allocation

```luma
let obj = { x = 10, y = 20 }       -- allocated on heap
```

#### 11.2.2 Finalization

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

### 11.3 Reference Semantics

**Value types** (numbers, booleans, null): copied by value  
**Reference types** (tables, arrays, functions): copied by reference

```luma
let a = [1, 2, 3]
let b = a                          -- b references same list
b[0] = 99
print(a[0])                        -- 99
```

### 12. Error Handling

### 13. Standard Library