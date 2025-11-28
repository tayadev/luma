---
sidebar_position: 5
---

# Control Flow

Control flow statements direct how your program executes. Luma uses a consistent `do...end` block syntax for all control structures.

## If Expressions

`if` checks a condition and executes code based on the result. It can be used as a statement or expression.

### Basic If

```luma
if age >= 18 do
  print("You are an adult")
end
```

### If-Else

```luma
if temperature > 30 do
  print("It's hot!")
else do
  print("It's cool")
end
```

### If-Else If-Else

Chain multiple conditions:

```luma
if score >= 90 do
  print("A")
else if score >= 80 do
  print("B")
else if score >= 70 do
  print("C")
else do
  print("F")
end
```

### If as Expression

`if` returns a value, so it can be used in assignments:

```luma
let status = if active do
  "Running"
else do
  "Stopped"
end

let max = if a > b do a else do b end

let message = if error != null do
  "Error: ${error}"
else do
  "Success"
end
```

## Truthiness

Conditions are evaluated for truthiness:

- **Truthy:** everything except `false` and `null`
- **Falsy:** `false` and `null`

```luma
if name do print("Has name") end       -- true if name is not empty string
if count do print("Has items") end     -- true if count > 0
if data.user do print("Has user") end  -- true if user exists
```

### Logical Operators

Combine conditions with `and`, `or`, and `not`:

```luma
if age >= 18 and hasId do
  print("Can vote")
end

if empty or error do
  print("Skip this item")
end

if not disabled do
  print("Enabled")
end
```

## While Loops

Repeat a block while a condition is true:

```luma
var count = 0
while count < 5 do
  print(count)
  count = count + 1
end
-- Output: 0, 1, 2, 3, 4
```

### Infinite Loops

Create an infinite loop and break out when needed:

```luma
var running = true
while running do
  let input = getUserInput()
  if input == "exit" do
    running = false
  else do
    processInput(input)
  end
end
```

## Do-While Loops

Executes at least once, then repeats while the condition is true:

```luma
var attempt = 0
do
  print("Trying...")
  attempt = attempt + 1
while attempt < 3 end
-- Always executes at least once
```

### Use Cases

Do-while is useful when you must execute code at least once:

```luma
var valid = false
do
  let value = getUserInput()
  valid = validateInput(value)
while not valid end
```

## For-In Loops

Iterate over collections. Loop variables are **immutable** and scoped to the loop.

### Iterating Over Lists

```luma
for item in [1, 2, 3] do
  print(item)
end

for name in ["Alice", "Bob", "Charlie"] do
  print("Hello, ${name}")
end
```

### Iterating Over Tables

```luma
let person = { name = "Alice", age = 30, city = "NYC" }
for [key, value] in person do
  print("${key}: ${value}")
end
```

### Numeric Ranges

```luma
for i in range(1, 5) do
  print(i)
end
-- Output: 1, 2, 3, 4
```

### Indexed Iteration

```luma
let colors = ["red", "green", "blue"]
for [color, index] in colors.indexed() do
  print("${index}: ${color}")
end
-- Output:
-- 0: red
-- 1: green
-- 2: blue
```

## Break and Continue

### Break

Exit the current loop immediately:

```luma
for item in items do
  if shouldStop(item) do
    break                 -- exit loop
  end
  process(item)
end
```

### Continue

Skip the rest of the current iteration:

```luma
for item in items do
  if skip(item) do
    continue              -- next iteration
  end
  process(item)
end
```

### Multi-level Break/Continue

Break or continue out of nested loops by specifying the level:

```luma
for i in range(1, 10) do
  for j in range(1, 10) do
    if shouldExit(i, j) do
      break 2             -- exit both loops
    end
    print("${i}, ${j}")
  end
end
```

## Return Statements

Exit a function immediately with a value:

```luma
fn findUser(id: String) do
  for user in users do
    if user.id == id do
      return user         -- exit early with value
    end
  end
  null                    -- implicit return if not found
end
```

### Early Exit Pattern

Return early to reduce nesting:

```luma
fn process(data) do
  if not isValid(data) do
    return null           -- early exit
  end
  
  -- rest of processing
  transform(data)
end
```

## Blocks

Any code can be grouped in a `do...end` block:

```luma
let value = do
  let x = 10
  let y = 20
  x + y                   -- implicit return
end
print(value)              -- 30
```

### Scope

Variables in blocks are scoped to that block:

```luma
let outer = "visible"
do
  let inner = "scoped"
  print(outer)            -- ✅ Can access outer
end
print(inner)              -- ❌ Error: inner out of scope
```

## Pattern with Break/Continue

### Search with Early Exit

```luma
let target = 5
var found = false
for item in items do
  if item == target do
    found = true
    break
  end
end
```

### Skip Invalid Items

```luma
for line in file.lines() do
  if line.empty() do
    continue              -- skip empty lines
  end
  processLine(line)
end
```

## Next Steps

- [Pattern Matching](../advanced/pattern-matching.md) — More advanced control flow
- [Functions](./functions.md) — Return statements and function bodies
