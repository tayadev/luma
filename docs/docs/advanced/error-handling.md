---
sidebar_position: 4
---

# Error Handling

Luma uses **explicit error handling with no exceptions**. All errors are ordinary values represented using the `Result` type, making error handling predictable and type-safe.

## Why No Exceptions?

Unlike many languages, Luma completely eliminates exceptions in favor of explicit error values:

✅ **Explicit** — Error possibilities are visible in function signatures  
✅ **Type-safe** — Return types declare what can go wrong  
✅ **Composable** — Errors flow through your program predictably  
✅ **No surprises** — No hidden failure paths  

## The Result Type

`Result` represents either success (`ok`) or failure (`err`):

```luma
fn divide(a: Number, b: Number): Result(Number, String) do
  if b == 0 do
    { ok = null, err = "Division by zero" }
  else do
    { ok = a / b, err = null }
  end
end
```

The `Result` type is generic: `Result(T, E)` where:
- `T` = Type of success value
- `E` = Type of error value

## Creating Results

### Success Results

```luma
{ ok = 42, err = null }
{ ok = "data", err = null }
{ ok = [1, 2, 3], err = null }
```

### Error Results

```luma
{ ok = null, err = "Something went wrong" }
{ ok = null, err = "File not found" }
{ ok = null, err = "Invalid input" }
```

## Checking Results

### Pattern Matching

```luma
let result = divide(10, 2)

match result do
  ok do
    let value = result.ok
    print("Result: ${value}")
  end
  err do
    let error = result.err
    print("Error: ${error}")
  end
end
```

### Conditional Checking

```luma
let result = divide(10, 2)

if result.err != null do
  print("Error: ${result.err}")
  return result
end

let value = result.ok
print("Success: ${value}")
```

### Early Return Pattern

```luma
fn processFile(path: String): Result(String, String) do
  let content = readFile(path)
  if content.err != null do
    return content    -- propagate error
  end
  
  let transformed = transform(content.ok)
  { ok = transformed, err = null }
end
```

## Error Propagation

Pass errors up the call chain:

```luma
fn parseAndProcess(text: String): Result(Number, String) do
  -- Try to parse
  let parsed = parseNumber(text)
  
  -- If parsing failed, propagate the error
  if parsed.err != null do
    return parsed
  end
  
  -- Otherwise, process the parsed value
  let processed = process(parsed.ok)
  { ok = processed, err = null }
end

let result = parseAndProcess("42")
if result.err != null do
  print("Failed: ${result.err}")
else do
  print("Success: ${result.ok}")
end
```

## Custom Error Types

You can create structured error types:

```luma
let FileError = {
  code = Number,
  path = String,
  reason = String
}

fn readFile(path: String): Result(String, FileError) do
  if not fileExists(path) do
    return {
      ok = null,
      err = {
        code = 404,
        path = path,
        reason = "File not found"
      }
    }
  end
  
  -- successful read
  { ok = fileContents(path), err = null }
end
```

## Combining Multiple Operations

Chain results together safely:

```luma
fn processUserData(userId: String): Result(Data, String) do
  -- Step 1: Fetch user
  let userResult = fetchUser(userId)
  if userResult.err != null do
    return { ok = null, err = "User not found" }
  end
  let user = userResult.ok
  
  -- Step 2: Fetch user's data
  let dataResult = fetchData(user.id)
  if dataResult.err != null do
    return { ok = null, err = "Data not found" }
  end
  let data = dataResult.ok
  
  -- Step 3: Transform
  let transformed = transform(data)
  { ok = transformed, err = null }
end
```

## Option Type (Planned)

For cases where absence of a value is not an error:

```luma
fn findUser(id: String): Option(User) do
  if userExists(id) do
    { some = user, none = null }
  else do
    { some = null, none = true }
  end
end
```

## Best Practices

### 1. Be Specific with Error Types

```luma
-- ✅ Good: Clear error type
fn parse(text: String): Result(Number, String) do
  -- ...
end

-- ❌ Vague: Any becomes too flexible
fn parse(text: String): Result(Number, Any) do
  -- ...
end
```

### 2. Include Context in Errors

```luma
-- ✅ Good: Helpful context
if fileSize > maxSize do
  return {
    ok = null,
    err = "File too large: ${fileSize}bytes (max: ${maxSize})"
  }
end

-- ❌ Not helpful
if fileSize > maxSize do
  return { ok = null, err = "Error" }
end
```

### 3. Check Errors Early

```luma
-- ✅ Good: Check and handle immediately
fn doWork() do
  let step1 = operation1()
  if step1.err != null do return step1 end
  
  let step2 = operation2(step1.ok)
  if step2.err != null do return step2 end
  
  operation3(step2.ok)
end
```

### 4. Document Error Possibilities

```luma
-- Clearly state what can go wrong
fn readConfig(path: String): Result(Config, String) do
  -- May fail with: "File not found", "Invalid JSON", "Missing required field"
  -- ...
end
```

### 5. Test Error Cases

```luma
-- Test both success and failure paths
let result1 = divide(10, 2)       -- success case
let result2 = divide(10, 0)       -- error case

match result1 do ok do print("✅") end err do print("❌") end
match result2 do ok do print("❌") end err do print("✅") end
```

## Comparison: Exception vs Result

### With Exceptions (❌ not Luma)
```javascript
try {
  let user = fetchUser(id);
  let posts = fetchPosts(user.id);
  return processPosts(posts);
} catch (e) {
  // Which operation failed? Not clear!
  print("Error: " + e);
}
```

### With Result (✅ Luma way)
```luma
let userResult = fetchUser(id)
if userResult.err != null do return userResult end

let postsResult = fetchPosts(userResult.ok.id)
if postsResult.err != null do return postsResult end

processPosts(postsResult.ok)
```

The Luma approach is:
- **Explicit** about where errors occur
- **Type-safe** with checked return types
- **Impossible to ignore** errors
- **Composable** with clear flow

## Related Documentation

- [Pattern Matching](./pattern-matching.md) — Match on Result types
- [Functions](../basics/functions.md) — Return types and signatures
- [Types](../basics/types.md) — The Result generic type
