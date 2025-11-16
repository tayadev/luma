---
sidebar_position: 1
---

# Literals

Luma supports several types of literal values.

## Numbers

### Integers and Floats

```luma
42
3.14
-7
0.001
```

### Hexadecimal

```luma
0xFF
0x1A3B
```

### Binary

```luma
0b1010
0b1101
```

### Scientific Notation

```luma
1.5e3    -- 1500
2.5E-4   -- 0.00025
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

## Strings

### Single-line Strings

```luma
"Hello, World!"
```

### Multiline Strings

```luma
"This is
a multiline
string."
```

### Format Strings (Interpolation)

```luma
let name = "World"
"Hello, ${name}!"  -- "Hello, World!"
```

### Escape Sequences

- `\"` - Literal quote
- `\\` - Literal backslash
- `\${` - Literal `${` without interpolation

```luma
"He said \"Hello\""
"Path: C:\\Users\\Documents"
"Template: \${variable}"
```

## Arrays

Arrays can contain elements of any type:

```luma
[1, 2, 3]
["apple", "banana", "cherry"]
[1, "apple", true]
[]  -- empty array
```

## Tables

Tables are key-value pairs (similar to objects or dictionaries):

```luma
{
  key1 = "value1",
  key2 = 42,
  nested = {
    subkey = true
  }
}
```

## Functions

Functions are values and can be assigned to variables:

```luma
fn(param1: Type1, param2: Type2): ReturnType do
  -- function body
end
```

See the [Functions](./functions.md) page for more details.
