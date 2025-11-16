---
sidebar_position: 5
---

# Control Flow

Luma provides standard control flow statements with a consistent `do...end` block syntax.

## Conditional Statements

### If Statement

```luma
if condition do
  -- code block
end
```

### If-Elif-Else

```luma
if condition1 do
  -- code block 1
elif condition2 do
  -- code block 2
else do
  -- code block 3
end
```

### Example

```luma
let age = 18

if age < 13 do
  print("Child")
elif age < 18 do
  print("Teenager")
else do
  print("Adult")
end
```

## Loops

### While Loop

```luma
var count = 0
while count < 5 do
  print(count)
  count = count + 1
end
```

### Do-While Loop

Executes the block at least once:

```luma
var count = 0
do
  print(count)
  count = count + 1
while count < 5 end
```

### For Loop

Iterate over arrays and other iterables:

```luma
for n in [1, 2, 3] do
  print(n)
end
```

#### Range-based Loops

```luma
for n in range(1, 10) do
  print(n)
end
```

#### Loop Variables

Loop variables are **always immutable** and scoped to the loop body:

```luma
for item in items do
  -- item is immutable here
  -- item = "new"  -- Error!
end
```

### Destructuring in Loops

#### Table Iteration

```luma
let myTable = { a = 1, b = 2 }
for [key, value] in myTable do
  print(key, value)
end
```

#### Array with Index

```luma
let myArray = [10, 20, 30]
for [value, index] in myArray.indexed() do
  print(index, value)
end
```

## Break and Continue

### Break

Exit the loop immediately:

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

Specify levels to break or continue:

```luma
for i in [1, 2, 3] do
  for j in [1, 2, 3] do
    if j == 2 do break 2 end  -- break out of both loops
    print(i, j)
  end
end
```

## Blocks

All blocks use `do...end` syntax. Indentation is irrelevant:

```luma
if x > 0 do
  print("positive")
end

-- This is also valid (though not recommended style):
if x > 0 do print("positive") end
```
