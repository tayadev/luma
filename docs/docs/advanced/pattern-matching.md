---
sidebar_position: 2
---

# Pattern Matching

Pattern matching allows you to match values against patterns and execute code based on the match.

## Match Expression

```luma
match value do
  pattern1 do
    -- branch 1
  end
  pattern2 do
    -- branch 2
  end
  _ do
    -- default case
  end
end
```

## Pattern Types

### Literal Patterns

Match specific literal values:

```luma
match x do
  0 do print("zero") end
  1 do print("one") end
  _ do print("other") end
end
```

**Example with strings:**
```luma
let day = "Monday"

match day do
  "Monday" do print("Start of the week") end
  "Friday" do print("Almost weekend!") end
  "Saturday" do print("Weekend!") end
  "Sunday" do print("Weekend!") end
  _ do print("Regular day") end
end
```

### Type Patterns

Match on Result/Option types:

```luma
match result do
  ok do
    print("Success: ${result.ok}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

**With Option:**
```luma
let user = findUser("123")

match user do
  some do
    print("Found: ${user.value.name}")
  end
  none do
    print("User not found")
  end
end
```

### Wildcard Pattern

The `_` pattern matches any value:

```luma
match value do
  _ do print("matches anything") end
end
```

## Exhaustiveness

Pattern matching must be exhaustive. If not all cases are covered, a `_` wildcard is required.

```luma
-- Error: not exhaustive (missing default case)
match result do
  ok do print(result.ok) end
end

-- OK: exhaustive with default
match result do
  ok do print(result.ok) end
  _ do print("Not ok") end
end
```

## Common Use Cases

### Error Handling

```luma
let result = readFile("data.txt")

match result do
  ok do
    print("File contents: ${result.ok}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

### State Machines

```luma
match status do
  "active" do
    print("User is active")
  end
  "inactive" do
    print("User is inactive")
  end
  "pending" do
    print("User is pending approval")
  end
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
