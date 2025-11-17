---
sidebar_position: 4
---

# Error Handling

Luma uses explicit error handling with the `Result` type. **There are no exceptions.**

## Errors as Values

Luma has no exceptions. All errors are represented as values using the `Result` type.

```luma
let Result = {
  ok = Any,
  err = Any
}
```

## Returning Errors

```luma
fn readFile(path: String): Result(String, Error) do
  let data, err = fs.read(path)
  if err != null do
    return { ok = null, err = err }
  end
  return { ok = data, err = null }
end
```

## Handling Errors

### Pattern Matching

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

### Conditional Checking

```luma
let result = readFile("data.txt")
if result.err != null do
  print("Error: ${result.err}")
  return
end
let data = result.ok
```

## Error Propagation

```luma
fn processFile(path: String): Result(String, Error) do
  let result = readFile(path)
  if result.err != null do
    return result                  -- propagate error
  end

  let processed = transform(result.ok)
  return { ok = processed, err = null }
end
```

## Custom Error Types

```luma
let FileError = {
  __parent = Error,
  path = String,
  reason = String
}

fn readFile(path: String): Result(String, FileError) do
  -- implementation
end
```

## Best Practices

1. **Always handle errors explicitly**
   ```luma
   let result = operation()
   match result do
     ok do -- handle success
     err do -- handle error
   end
   ```

2. **Propagate errors upward**
   ```luma
   fn doWork(): Result(Value, Error) do
     let result = operation()
     if result.err != null do
       return result  -- propagate
     end
     return { ok = process(result.ok), err = null }
   end
   ```

3. **Provide context in errors**
   ```luma
   if err != null do
     return { ok = null, err = "Failed to process ${path}: ${err}" }
   end
   ```

## Why No Exceptions?

Unlike many languages, Luma has **no exceptions**. All errors are:
- Explicit in function signatures
- Visible in return types
- Required to be handled

This makes error handling:
- Predictable
- Type-safe
- Impossible to ignore
