---
sidebar_position: 7
---

# Expressions

Expressions are syntactic constructs that evaluate to a value.

## Expression Categories

- **Literal expressions**: `42`, `"hello"`, `true`
- **Variable expressions**: `x`, `count`
- **Binary expressions**: `a + b`, `x * y`
- **Unary expressions**: `-x`, `!condition`
- **Call expressions**: `func(arg1, arg2)`
- **Member access**: `object.field`, `array[index]`
- **Block expressions**: `do ... end`
- **Conditional expressions**: `if ... then ... else ...`

## Arithmetic Operators

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

## Comparison Operators

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

## Logical Operators

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

## String Operations

```luma
"Hello, " + "World!"       -- concatenation: "Hello, World!"
"value: ${x}"              -- interpolation
```

## Member Access

**Dot notation:**
```luma
object.field
person.name
```

**Bracket notation:**
```luma
object["field"]
table["key with spaces"]
array[0]
```

## Function Calls

```luma
func()                     -- no arguments
func(arg)                  -- single argument
func(arg1, arg2)           -- multiple arguments
func(a = 1, b = 2)         -- named arguments
```

**Parentheses are required** for all function calls.

## Block Expressions

Blocks evaluate to the value of their last expression:

```luma
let result = do
  let x = 10
  let y = 20
  x + y                    -- returns 30
end
```

If the last statement is not an expression, the block returns `null`.

## Operator Precedence

Operators are evaluated in the following order (highest to lowest):

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

## Examples

### Arithmetic

```luma
let sum = 10 + 5           -- 15
let product = 3 * 4        -- 12
let quotient = 20 / 4      -- 5
let remainder = 17 % 5     -- 2
let negative = -10         -- -10
```

### Comparison

```luma
let isEqual = 5 == 5       -- true
let isLess = 3 < 5         -- true
let isGreater = 10 > 20    -- false
```

### Logical

```luma
let and = true && false    -- false
let or = true || false     -- true
let not = !true            -- false
```

### Combined

```luma
let result = (10 + 5) * 2 / 3  -- 10
let check = x > 0 && x < 100   -- range check
```
