# ⭐ Luma

Luma is a typed scripting language designed for simplicity, safety, and expressiveness.

---

## 1. Core Design Principles

* Everything is a value
* No implicit assignment
* Tiny core syntax
* Explicit error handling; errors are values
* No magic
* Built-in dependency management via URL imports
* Async/await support
* Garbage-collected

---

## 2. Literals

### 2.1 Numbers

* Integers and floats: `42`, `3.14`, `-7`, `0.001`
* Hexadecimal: `0xFF`, `0x1A3B`
* Binary: `0b1010`, `0b1101`
* Scientific notation: `1.5e3`, `2.5E-4`

### 2.2 Booleans

* `true`, `false`

### 2.3 Null

* `null`

### 2.4 Strings

* Single-line: `"Hello, World!"`
* Multiline:

  ```luma
  "This is
  a multiline
  string."
  ```
* Format strings: `"Hello, ${name}!"`

Escape sequences:

* `\"` → literal quote
* `\\` → literal backslash
* `\${` → literal `${` without interpolation

### 2.5 Arrays

```luma
[1, 2, 3]
["apple", "banana", "cherry"]
[1, "apple", true]
[]
```

### 2.6 Tables

```luma
{
  key1 = "value1",
  key2 = 42,
  nested = {
    subkey = true
  }
}
```

### 2.7 Functions

```luma
fn(param1: Type1, param2: Type2): ReturnType do
  ...
end
```

* Optional parameters: `param: Type = defaultValue`
* Functions implicitly return the value of the last expression if no `return` is used
* Functions are treated as async when they return a `Promise` and contain `await`
* Single-param calls may omit parentheses

---

## 3. Variables & Constants

* `var` → mutable variable
* `let` → immutable constant

```luma
var x = 42
let name = "Luma"

var count: Number = 10
let pi: Number = 3.14
```

> Type is inferred if not explicitly provided.

### Destructuring

```luma
let person = { name = "Alice", age = 30 }
let { name, age } = person

let numbers = [1, 2, 3]
let [first, second, third] = numbers

let [head, ...tail] = numbers  -- tail = [2,3]
let [head, ...] = numbers      -- ignore remaining
```

---

## 4. Functions & Async

```luma
let add = fn(a: Number, b: Number): Number do
  return a + b
end

let fetchData = fn(url: String): Result(String, Error) do
  let res = await http.get(url)
  return res
end

fetchData("test")        -- returns Promise(Result(String, Error))
await fetchData("test")  -- resolves Promise, returns Result(String, Error)
```

> `await` transforms `Promise(T)` → `T`.

---

## 5. Types and Instances

### Built-in Types

* `Any` : supertype of all types
* `Number` : 64-bit floating point
* `String` : UTF-8 string
* `Boolean` : true/false
* `Array(T)` : array of elements of type T
* `Table` : untyped table
* `Result(OkType, ErrType)` : success/failure
* `Option(T)` : optional value

### Defining a Type

```luma
let Dog = {
  name = String,
  breed = String,

  speak = fn(self: Dog): String do
    return "Woof! I am a " + self.breed
  end,

  new = fn(name: String, breed: String): Dog do
    let raw = { name = name, breed = breed }
    return cast(Dog, raw)
  end
}
```

### Creating Instances

```luma
let rex = Dog.new("Rex", "Beagle")
```

### Type Casting

```luma
let rex = cast(Dog, { name = "Rex", breed = "Beagle" })
```

> `cast(Type, table)` validates fields, attaches prototype, merges inheritance.

---

## 6. Fields

* Declared as `field = Type`
* Methods and operator overloads are fields holding `fn` values

```luma
name = String
age = Number
tags = Array(String)
```

---

## 7. Traits

```luma
let Speakable = {
  speak = fn(self: Any): String
}
```

* Structural matching: objects with required fields satisfy trait
* Checked at `cast()` time or compile-time

---

## 8. Inheritance

```luma
let Animal = {
  name = String,
  speak = fn(self: Animal): String do return "noise" end
}

let Dog = {
  __parent = Animal,
  breed = String,
  speak = fn(self: Dog): String do return "woof" end
}
```

* Fields/methods merged from parent
* Methods can be overridden
* Check type: `isInstanceOf(instance, Type)`

---

## 9. Operator Overloading

```luma
__add = fn(a: Vector2, b: Vector2): Vector2 do
  return Vector2.new(a.x + b.x, a.y + b.y)
end
```

* Supported: `+`, `-`, `*`, `/`, `%`, unary `-`, `==`, `<`, `<=`, `>`, `>=`
* Auto-derived: `!=`
* `+` also used for string concatenation
* Non-overloadable: `and`, `or`, `not`, `[]`, `in`, `.`

---

## 10. Conversions (`into()`)

```luma
__into = fn(self: Weight, target: Type): String do
  return self.grams + "g"
end
```

* Optional; missing `__into` means type cannot convert/print
* `print()` internally uses `into(String)`

---

## 11. Result / Error Handling

* Errors are values; no exceptions

```luma
let Result = {
  ok = Any,
  err = Any,
}
```

* Example:

```luma
let readFile = fn(path: String): Result(String, Error) do
  var data, err = fs.read(path)
  if err != null do
    return Result.err.new(err)
  end
  return Result.ok.new(data)
end
```

---

## 12. Pattern Matching

```luma
match r do
  ok do print(r.ok) end
  err do print(r.err) end
  _ do print("default case") end
end
```

* Works with `Result`, `Option`, unions, enums
* `_` is required if not all cases are covered

---

## 13. Modules / Imports

```luma
let local = import "./module.luma"
let http = import "https://example.com/module.luma"
let git = import "git@github.com:user/repo.git"
let github = import "gh:user/repo@1.2.3"
```

* Imports return value of module
* Cached locally after first download
* Folder URL → looks for `main.luma`
* Lock file: `luma.lock`
* `import()` is synchronous

---

## 14. Async / Await

```luma
let fetchDog = fn(id: String): Result(Dog, Error) do
  let data = await http.get("https://dogs.api/" + id)
  return cast(Dog, data)
end
```

* `await` suspends execution
* Async inferred by `Promise` return type + presence of `await`

---

## 15. Loops

### 15.1 While Loop

```luma
var count = 0
while count < 5 do
  print(count)
  count = count + 1
end
```

### 15.2 Do-While Loop

```luma
var count = 0
do
  print(count)
  count = count + 1
while count < 5 end
```

### 15.3 For Loop

```luma
for n in [1, 2, 3] do
  print(n)
end
```

* Loop variables are **always immutable** and scoped to loop body
* Range-based loops: `for n in range(1, 10) do ... end`
* Iterables must implement `__iter` / `__next` (arrays and tables by default)

#### Multiple Values via Destructuring

```luma
let myTable = { a = 1, b = 2 }
for [key, value] in myTable do
  print(key, value)
end

let myArray = [10, 20, 30]
for [value, index] in myArray.indexed() do
  print(index, value)
end
```

#### Break and Continue

```luma
for n in [1, 2, 3, 4, 5] do
  if n == 3 do continue end
  if n == 5 do break end
  print(n)
end
```

* Nested loops: specify levels to break/continue: `break 2`

---

## 16. Blocks

* Indentation is irrelevant
* Use `do ... end` for all blocks

---

## 17. Destructuring Assignment

* Works for arrays and tables
* `...` ignores remaining elements
* Missing elements → `null`

```luma
let [head, ...tail] = [1,2,3]  -- head=1, tail=[2,3]
```

---

## 18. Comments

* Single-line: `--`
* Multi-line: `--[[ ... ]]`

---

## 19. Garbage Collection

* Automatic memory management
* Tables, functions, closures are heap-allocated
* `__gc` method is called when an object is collected

---

## 20. Style Guide

* 2 spaces for indentation
* `snake_case` for variable and function names
