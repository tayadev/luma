---
sidebar_position: 5
---

# Control Flow

Luma provides standard control flow statements with a consistent `do...end` block syntax.

## Conditional Statements

### Basic If Statement

```luma
if condition do
  -- executed if condition is truthy
end
```

### If-Else

```luma
if condition do
  -- if branch
else do
  -- else branch
end
```

### If-Elif-Else

```luma
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

### Example

```luma
let age = 18

if age < 13 do
  print("Child")
else if age < 18 do
  print("Teenager")
else do
  print("Adult")
end
```

### Conditional Expressions

`if` can be used as an expression:

```luma
let max = if a > b do a else do b end
```

## Truthiness

- **Truthy:** all values except `false` and `null`
- **Falsy:** `false` and `null`

## Loops

### While Loop

```luma
while condition do
  -- body
end
```

**Example:**
```luma
var count = 0
while count < 5 do
  print(count)
  count = count + 1
end
```

### Do-While Loop

Executes the body at least once:

```luma
do
  -- body (executes at least once)
while condition end
```

**Example:**
```luma
var count = 0
do
  print(count)
  count = count + 1
while count < 5 end
```

### For-In Loops

```luma
for item in iterable do
  -- body
end
```

**Loop variables are immutable** and scoped to the loop body.

#### List Iteration

```luma
for item in [1, 2, 3] do
  print(item)
end
```

#### Table Iteration

```luma
for [key, value] in table do
  print(key, value)
end
```

#### Range Iteration

```luma
for n in range(1, 10) do
  print(n)
end
```

#### Indexed Iteration

```luma
for [item, index] in list.indexed() do
  print(index, item)
end
```

## Break and Continue

### Break

Exit the innermost loop immediately:

```luma
for n in [1, 2, 3, 4, 5] do
  if n == 3 do break end
  print(n)
end
-- Output: 1, 2
```

### Continue

Skip to the next iteration:

```luma
for n in [1, 2, 3, 4, 5] do
  if n == 3 do continue end
  print(n)
end
-- Output: 1, 2, 4, 5
```

### Nested Loops

Specify levels to break or continue in outer loops:

```luma
break                              -- exit innermost loop
break 2                            -- exit 2 nested loops

continue                           -- skip to next iteration
continue 2                         -- skip in outer loop
```

**Example:**
```luma
for i in [1, 2, 3] do
  for j in [1, 2, 3] do
    if j == 2 do break 2 end  -- break out of both loops
    print(i, j)
  end
end
```

## Blocks

All blocks use `do...end` syntax:

```luma
if x > 0 do
  print("positive")
end

while condition do
  -- loop body
end

for item in items do
  -- loop body
end
```

Indentation is for readability but is not syntactically significant.
