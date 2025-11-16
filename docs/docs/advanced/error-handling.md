---
sidebar_position: 4
---

# Error Handling

Luma uses explicit error handling with the `Result` type. There are no exceptions.

## The Result Type

The `Result` type represents either success or failure:

```luma
let Result = {
  ok = Any,
  err = Any,
}
```

## Creating Results

### Success Result

```luma
Result.ok.new(value)
```

### Error Result

```luma
Result.err.new(error)
```

## Example Function

```luma
let readFile = fn(path: String): Result(String, Error) do
  var data, err = fs.read(path)
  if err != null do
    return Result.err.new(err)
  end
  return Result.ok.new(data)
end
```

## Handling Results

### With Pattern Matching

```luma
let result = readFile("data.txt")

match result do
  ok do
    print("File contents: " + result.ok)
  end
  err do
    print("Error reading file: " + result.err)
  end
end
```

### With Conditional

```luma
let result = readFile("data.txt")

if result.ok != null do
  print("Success: " + result.ok)
else do
  print("Error: " + result.err)
end
```

## Chaining Operations

Chain operations that return Results:

```luma
let processFile = fn(path: String): Result(Data, Error) do
  let readResult = readFile(path)
  
  match readResult do
    ok do
      let parseResult = parseData(readResult.ok)
      return parseResult
    end
    err do
      return Result.err.new(readResult.err)
    end
  end
end
```

## Error Types

You can define custom error types:

```luma
let FileError = {
  kind = String,  -- "NotFound", "PermissionDenied", etc.
  message = String,
  path = String,
  
  new = fn(kind: String, message: String, path: String): FileError do
    return cast(FileError, {
      kind = kind,
      message = message,
      path = path
    })
  end
}

let readFile = fn(path: String): Result(String, FileError) do
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
   let doWork = fn(): Result(Value, Error) do
     let result = operation()
     match result do
       ok do
         return Result.ok.new(process(result.ok))
       end
       err do
         return result  -- propagate error
       end
     end
   end
   ```

3. **Provide context in errors**
   ```luma
   if err != null do
     return Result.err.new(
       "Failed to process file ${path}: ${err.message}"
     )
   end
   ```

4. **Use specific error types**
   ```luma
   Result(User, DatabaseError)
   Result(Config, ParseError)
   ```

## No Exceptions

Unlike many languages, Luma has **no exceptions**. All errors are:
- Explicit in function signatures
- Visible in return types
- Required to be handled

This makes error handling:
- Predictable
- Type-safe
- Impossible to ignore

## Comparison with Other Languages

```javascript
// JavaScript (exceptions)
try {
  const data = readFile("data.txt");
  process(data);
} catch (error) {
  console.error(error);
}
```

```luma
-- Luma (Result type)
let result = readFile("data.txt")
match result do
  ok do
    process(result.ok)
  end
  err do
    print(result.err)
  end
end
```

The Luma approach makes errors explicit in the type system and impossible to ignore.
