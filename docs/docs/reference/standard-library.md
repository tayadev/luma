---
sidebar_position: 6
---

# Standard Library

Luma's design philosophy emphasizes keeping the language core minimal and relying on a rich standard library for extended functionality. The standard library is just another module and can be left out if not needed.

## Core Functions

### I/O

```luma
print(value: Any): Null
-- Prints value to stdout (uses __into(String))
```

### Type Operations

```luma
cast(type: Type, value: Any): Type
-- Validates and casts value to type

isInstanceOf(value: Any, type: Type): Boolean
-- Checks if value is instance of type

typeof(value: Any): Type
-- Returns runtime type of value
```

## Standard Library Modules

The following modules are planned for the standard library:

### math

Mathematical functions and constants

```luma
let math = import("std:math")

math.pi
math.abs(x)
math.sqrt(x)
math.sin(x)
math.cos(x)
math.floor(x)
math.ceil(x)
math.round(x)
```

### string

String manipulation utilities

```luma
let string = import("std:string")

string.split(str, delimiter)
string.join(array, separator)
string.trim(str)
string.uppercase(str)
string.lowercase(str)
string.replace(str, pattern, replacement)
```

### array

Array utilities and higher-order functions

```luma
let array = import("std:array")

array.map(arr, fn)
array.filter(arr, fn)
array.reduce(arr, fn, initial)
array.find(arr, predicate)
array.sort(arr, comparator)
```

### table

Table utilities

```luma
let table = import("std:table")

table.keys(tbl)
table.values(tbl)
table.entries(tbl)
table.merge(tbl1, tbl2)
```

### fs

File system operations

```luma
let fs = import("std:fs")

fs.read(path): Result(String, Error)
fs.write(path, content): Result(Null, Error)
fs.exists(path): Boolean
fs.isFile(path): Boolean
fs.isDirectory(path): Boolean
```

### os

Operating system interaction

```luma
let os = import("std:os")

os.getEnv(name): Option(String)
os.setEnv(name, value): Result(Null, Error)
os.platform(): String
os.arch(): String
```

### http

HTTP client (async)

```luma
let http = import("std:http")

http.get(url): Promise(Result(Response, Error))
http.post(url, body): Promise(Result(Response, Error))
http.request(options): Promise(Result(Response, Error))
```

## Implementation Status

⚠️ **Note:** The standard library is currently under development. The modules and APIs described above are planned but not yet implemented.

The core functions (`print`, `cast`, `isInstanceOf`, `typeof`) will be implemented as part of the VM.

## Importing Standard Library Modules

Standard library modules will be importable using the `std:` prefix:

```luma
let math = import("std:math")
let fs = import("std:fs")
let http = import("std:http")
```

## Philosophy

The standard library follows these principles:

1. **Minimal core** — Keep the language small
2. **Modular** — Import only what you need
3. **Consistent** — Uniform API design across modules
4. **Safe** — Use Result types for operations that can fail
5. **Async-ready** — Native async support for I/O operations
