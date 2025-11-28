---
sidebar_position: 3
---

# Operators Reference

This is a complete reference of all operators in Luma, their precedence, and usage examples.

## Operator Precedence

Operators are evaluated in order from highest to lowest precedence. Use parentheses to override precedence.

| Level | Operators | Name | Associativity |
|-------|-----------|------|---------------|
| 1 | `()` `[]` `.` | Postfix (call, index, member) | Left |
| 2 | `-` `not` | Unary (negation, logical NOT) | Right |
| 3 | `^` | Exponentiation | Right |
| 4 | `*` `/` `%` | Multiplication/division | Left |
| 5 | `+` `-` | Addition/subtraction | Left |
| 6 | `<` `<=` `>` `>=` | Comparison | Left |
| 7 | `==` `!=` | Equality | Left |
| 8 | `and` | Logical AND | Left |
| 9 | `or` | Logical OR | Left |
| 10 | `=` | Assignment | Right |

## Arithmetic Operators

### Addition `+`

```luma
10 + 5            -- 15
"Hello" + " World"  -- "Hello World"
[1, 2] + [3, 4]   -- [1, 2, 3, 4] (list concatenation)
```

### Subtraction `-`

```luma
10 - 3            -- 7
-5                -- Unary negation
```

### Multiplication `*`

```luma
4 * 5             -- 20
3 * 2.5           -- 7.5
[1, 2] * 3        -- [1, 2, 1, 2, 1, 2] (list repetition)
```

### Division `/`

```luma
10 / 2            -- 5
10 / 3            -- 3.333...
```

### Modulo `%`

Returns the remainder after division:

```luma
10 % 3            -- 1
-10 % 3           -- -1
```

### Exponentiation `^`

```luma
2 ^ 3             -- 8
2 ^ 0.5           -- 1.414... (square root)
10 ^ -1           -- 0.1
```

## Comparison Operators

All comparison operators return `true` or `false`.

### Less Than `<`

```luma
5 < 10            -- true
10 < 5            -- false
"a" < "b"         -- true (lexicographic)
```

### Less Than or Equal `<=`

```luma
5 <= 5            -- true
5 <= 4            -- false
```

### Greater Than `>`

```luma
10 > 5            -- true
5 > 10            -- false
```

### Greater Than or Equal `>=`

```luma
5 >= 5            -- true
5 >= 6            -- false
```

### Equality `==`

```luma
5 == 5            -- true
5 == 5.0          -- true
"hello" == "hello" -- true
```

### Inequality `!=`

```luma
5 != 5            -- false
5 != 6            -- true
"a" != "b"        -- true
```

## Logical Operators

### Logical AND `and`

```luma
true and true     -- true
true and false    -- false
false and true    -- false

if x > 0 and x < 10 do
  print("Between 0 and 10")
end
```

### Logical OR `or`

```luma
true or false     -- true
false or false    -- false

if x < 0 or x > 100 do
  print("Out of range")
end
```

### Logical NOT `not`

```luma
not true          -- false
not false         -- true

if not isReady do
  print("Not ready")
end
```

## Assignment Operators

### Assignment `=`

Assigns a value to a variable (only for `var`):

```luma
var x = 10
x = 20            -- Valid
x = x + 5         -- 25

let y = 10
y = 20            -- Error! let bindings are immutable
```

## String Operators

### String Concatenation `+`

```luma
"Hello" + " " + "World"  -- "Hello World"
"Number: " + 42          -- "Number: 42"
```

### String Interpolation

Use `${}` inside strings:

```luma
let name = "Alice"
let age = 30
"My name is ${name} and I'm ${age} years old"
```

## Collection Operators

### Indexing `[]`

Access elements by index (0-based):

```luma
let arr = [10, 20, 30]
arr[0]            -- 10
arr[2]            -- 30
arr[-1]           -- null (out of bounds)

let obj = { name = "Alice", age = 30 }
obj["name"]       -- "Alice"
```

### Member Access `.`

Access fields of tables/objects:

```luma
let person = { name = "Bob", age = 25 }
person.name       -- "Bob"
person.age        -- 25
```

### Membership Test `in`

Check if a key exists in a table:

```luma
let user = { name = "Charlie", email = "charlie@example.com" }
"name" in user    -- true
"age" in user     -- false

let items = [1, 2, 3]
-- Note: 'in' works with for loops, not direct membership
```

## Operator Combinations

### Compound Operations

While Luma doesn't have compound assignment operators (`+=`, `-=`, etc.), you can write:

```luma
var x = 10
x = x + 5         -- instead of x += 5

var count = 0
count = count + 1 -- instead of count++
```

## Type Coercion

Luma has **no implicit type coercion**. Operations require compatible types:

```luma
5 + "5"           -- Error! Can't add number and string
true + 1          -- Error! Can't add boolean and number
```

## Operator Overloading

:::info
Operator overloading is a planned feature for custom types. It's not yet available in the current implementation.
:::

## Truthiness in Conditions

Values are evaluated for truthiness in boolean contexts:

- **Truthy:** Everything except `false` and `null`
- **Falsy:** `false` and `null`

```luma
if 0 do print("0 is truthy") end         -- Prints (0 ≠ false)
if "" do print("empty string is truthy") end  -- Prints
if null do print("This won't print") end -- null is falsy
if false do print("This won't print") end -- false is falsy
```

## Operator Parsing Tips

### Parentheses for Clarity

```luma
-- Without parentheses (follows precedence)
2 + 3 * 4         -- 14 (multiply first)

-- With parentheses
(2 + 3) * 4       -- 20 (addition first)
```

### String Operators

```luma
-- These work:
"a" + "b"         -- "ab"
"test" + 123      -- "test123"

-- These don't:
"test" * 2        -- Error! Can't multiply strings
"a" - "b"         -- Error! Can't subtract strings
```

## Related Documentation

- [Functions](../basics/functions.md) — Function calls (highest precedence)
- [Control Flow](../basics/control-flow.md) — Using operators in conditions
- [Pattern Matching](../advanced/pattern-matching.md) — Match on values using operators
