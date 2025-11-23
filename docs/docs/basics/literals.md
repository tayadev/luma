---
sidebar_position: 1
---

# Literals

Luma supports several types of literal values.

## Numeric Literals

### Decimal Integers

```luma
42
-17
0
```

### Floating-Point Numbers

```luma
3.14
-0.001
.5
```

### Scientific Notation

```luma
1.5e3      -- 1500.0
2.5E-4     -- 0.00025
```

### Hexadecimal

```luma
0xFF       -- 255
0x1A3B     -- 6715
```

### Binary

```luma
0b1010     -- 10
0b1101     -- 13
```

## Booleans

```luma
true
false
```

## Null

```luma
null
```

## String Literals

### Basic Strings

Strings are enclosed in double quotes:

```luma
"Hello, World!"
```

### Multi-line Strings

Multi-line strings preserve line breaks:

```luma
"This is
a multiline
string."
```

### String Interpolation

Use `${expression}` for string interpolation:

```luma
let name = "World"
"Hello, ${name}!"        -- "Hello, World!"
"1 + 1 = ${1 + 1}"      -- "1 + 1 = 2"
```

### Escape Sequences

| Sequence | Meaning |
|----------|----------|
| `\"` | Literal double quote |
| `\\` | Literal backslash |
| `\n` | Newline |
| `\r` | Carriage return |
| `\t` | Tab |
| `\${` | Literal `${` (disable interpolation) |

```luma
"He said \"Hello\""
"Line 1\nLine 2"
"Disable: \${variable}"
```

## List Literals

Arrays are ordered, heterogeneous collections:

```luma
[1, 2, 3]
["apple", "banana", "cherry"]
[1, "mixed", true, null]
[]                          -- empty list
```

## Table Literals

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

### Key Syntax

- **Identifiers:** `key = value`
- **String keys:** `"key with spaces" = value`
- **Computed keys:** `[expression] = value`

## Function Literals

Functions are first-class values:

```luma
fn(x: Number, y: Number): Number do
  return x + y
end
```

See [§6 Functions](./functions.md) for complete syntax.
