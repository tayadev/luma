---
sidebar_position: 2
---

# Pattern Matching

Pattern matching allows you to match values against patterns and execute code based on the match.

## Basic Match

```luma
match value do
  pattern1 do
    -- code for pattern1
  end
  pattern2 do
    -- code for pattern2
  end
  _ do
    -- default case
  end
end
```

## Matching Result Types

Pattern matching is commonly used with `Result` types:

```luma
let readFile = fn(path: String): Result(String, Error) do
  -- implementation
end

let result = readFile("data.txt")

match result do
  ok do
    print("Success: " + result.ok)
  end
  err do
    print("Error: " + result.err)
  end
end
```

## Matching Option Types

```luma
let findUser = fn(id: String): Option(User) do
  -- implementation
end

let user = findUser("123")

match user do
  some do
    print("Found: " + user.value.name)
  end
  none do
    print("User not found")
  end
end
```

## Default Case

The `_` pattern matches anything and is required if not all cases are covered:

```luma
match status do
  "active" do
    print("User is active")
  end
  "inactive" do
    print("User is inactive")
  end
  _ do
    print("Unknown status")
  end
end
```

## Pattern Matching with Values

Match specific values:

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

## Pattern Matching with Types

Match based on type:

```luma
match value do
  Number do print("It's a number") end
  String do print("It's a string") end
  _ do print("Unknown type") end
end
```

## Exhaustiveness

Pattern matching must be exhaustive. Either:
1. Cover all possible cases, or
2. Include a `_` default case

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

## Combining Patterns

Patterns work with:
- `Result(OkType, ErrType)`
- `Option(T)`
- Enums (when available)
- Union types (when available)
- Literal values
- Type matching
