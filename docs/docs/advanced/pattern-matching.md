---
sidebar_position: 2
---

# Pattern Matching

Pattern matching is a powerful feature that allows you to match values against patterns and branch accordingly. It's more expressive than traditional switch statements.

## Match Expressions

The `match` keyword evaluates to the result of the matched branch:

```luma
let message = match statusCode do
  200 do "OK"
  404 do "Not Found"
  500 do "Server Error"
  _ do "Unknown"
end
print(message)  -- One of the above
```

## Match Statements

When you don't need the result, use match as a statement:

```luma
match action do
  "create" do
    db.insert(item)
    print("Created")
  end
  
  "update" do
    db.update(item)
    print("Updated")
  end
  
  "delete" do
    db.delete(item)
    print("Deleted")
  end
  
  _ do
    print("Unknown action")
  end
end
```

## Pattern Types

### Literal Patterns

Match exact values:

```luma
match count do
  0 do print("Empty") end
  1 do print("One item") end
  _ do print("Multiple items") end
end
```

**String literals:**
```luma
let status = match role do
  "admin" do "Administrator" end
  "user" do "Regular User" end
  "guest" do "Guest User" end
  _ do "Unknown Role" end
end
```

**Boolean literals:**
```luma
match canAccess do
  true do
    showContent()
  end
  false do
    showLoginForm()
  end
end
```

### Wildcard Pattern

The `_` pattern matches any value and is required for non-exhaustive matches:

```luma
match value do
  0 do print("Zero") end
  _ do print("Non-zero") end
end
```

### Type Patterns

Match on Result/Option types:

```luma
let result = trySomething()

match result do
  ok do
    print("Success: ${result.ok}")
  end
  err do
    print("Failed: ${result.err}")
  end
end
```

**With Option types:**
```luma
let user = findUserById("123")

match user do
  some do
    print("Found user: ${user.some}")
  end
  none do
    print("User not found")
  end
end
```

## Exhaustiveness Checking

Patterns must be exhaustive. Either cover all cases or provide a default:

```luma
-- ❌ Error: Missing case
match value do
  true do print("Yes") end
end

-- ✅ OK: Has default
match value do
  true do print("Yes") end
  _ do print("No") end
end

-- ✅ OK: Exhaustive for this type
match result do
  ok do print("Success") end
  err do print("Error") end
end
```

## Common Patterns

### Result Handling

```luma
fn processData(input: String): Result(Number, String) do
  if input.empty() do
    { ok = null, err = "Input is empty" }
  else do
    { ok = input.toNumber(), err = null }
  end
end

let result = processData("42")
match result do
  ok do
    let value = result.ok
    print("Processed: ${value}")
  end
  err do
    let error = result.err
    print("Error: ${error}")
  end
end
```

### User Actions

```luma
let command = getUserInput()

match command do
  "help" do
    showHelp()
  end
  "version" do
    printVersion()
  end
  "exit" do
    exit()
  end
  _ do
    print("Unknown command: ${command}")
  end
end
```

### State Transitions

```luma
let nextState = match currentState do
  "idle" do
    if hasWork() do "running" else do "idle" end
  end
  
  "running" do
    if isComplete() do "done" else do "running" end
  end
  
  "done" do "idle" end
  
  _ do currentState
end
```

### Type Branching

```luma
fn describe(value: Any): String do
  match value do
    _ if value == true do "Boolean true"
    _ if value == false do "Boolean false"
    _ if value == null do "Null value"
    _ do "Some other value"
  end
end
```

:::info
**Guard clauses with `if` conditions are planned** for more complex pattern matching.
:::

## Nested Matching

Match within match branches:

```luma
match result do
  ok do
    match result.ok do
      0 do print("Got zero") end
      _ do print("Got non-zero") end
    end
  end
  err do
    print("Error occurred")
  end
end
```

## Combining with Loops

```luma
for item in items do
  match item.status do
    "pending" do
      processItem(item)
    end
    "done" do
      continue  -- skip to next iteration
    end
    "error" do
      print("Skipping: ${item.error}")
      continue
    end
    _ do
      printUnknown(item)
    end
  end
end
```

## Best Practices

1. **Always provide a default case** unless patterns are truly exhaustive
2. **Use match for complex conditionals** instead of deeply nested if-else
3. **Keep cases simple** — use helper functions for complex logic
4. **Match early** in functions to validate input before processing

## Related Documentation

- [Control Flow](../basics/control-flow.md) — Conditionals and loops
- [Error Handling](./error-handling.md) — Working with Result types
- [Destructuring](../basics/variables.md#destructuring-assignment) — Extracting values in bindings
  _ do
    print("Unknown status")
  end
end
```

### Pattern Matching vs Conditionals

Pattern matching provides cleaner syntax for multi-way branching compared to if-else chains:

```luma
-- With if-else
if result.ok != null do
  print("Success")
else do
  print("Error")
end

-- With pattern matching (preferred)
match result do
  ok do print("Success") end
  err do print("Error") end
end
```
