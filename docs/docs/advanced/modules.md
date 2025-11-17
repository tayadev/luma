---
sidebar_position: 3
---

# Modules and Imports

Luma has built-in dependency management through URL-based imports.

## Import Syntax

```luma
let module = import("source")
```

`import()` is a built-in function that loads and evaluates a module, returning its exported value.

## Import Sources

### Local Files

```luma
let utils = import("./utils.luma")
let lib = import("../lib/helpers.luma")
```

### HTTP/HTTPS URLs

```luma
let http = import("https://example.com/http.luma")
```

### Git Repositories

```luma
let lib = import("git@github.com:user/repo.git")
let tagged = import("gh:user/repo@1.2.3")
```

## Module Resolution

**For URLs:**
1. Download file to local cache (`~/.luma/cache`)
2. Verify integrity (if lockfile exists)
3. Parse and evaluate module
4. Return module's exported value

**For directories:**
- If path is directory, look for `main.luma`

## Module Exports

Modules export the value of their last expression:

```luma
-- math.luma
let pi = 3.14159

let add = fn(a: Number, b: Number): Number do
  return a + b
end

{
  pi = pi,
  add = add
}
```

```luma
-- main.luma
let math = import("./math.luma")
print(math.pi)                     -- 3.14159
print(math.add(2, 3))              -- 5
```

## Dependency Locking

Dependencies are locked in `luma.lock`:

```json
{
  "https://example.com/http.luma": {
    "version": "1.2.3",
    "integrity": "sha256-...",
    "resolved": "2024-01-15T10:30:00Z"
  }
}
```

## Circular Dependencies

Circular imports are detected and result in an error:

```
Error: Circular dependency detected:
  a.luma -> b.luma -> a.luma
```

## Import Properties

- `import()` is **synchronous** - it blocks until the module is loaded
- Modules are evaluated once and cached
- Circular imports are detected and reported as errors

## Best Practices

1. **Pin versions** for remote dependencies:
   ```luma
   let lib = import("gh:user/repo@1.2.3")
   ```

2. **Use relative imports** for local modules:
   ```luma
   let utils = import("./utils.luma")
   ```

3. **Export a single value** from modules (usually a table):
   ```luma
   let MyModule = {
     func1 = ...,
     func2 = ...
   }
   MyModule  -- exported value
   ```

4. **Document public APIs** in module files

## Example Project Structure

```
project/
├── main.luma
├── luma.lock
├── lib/
│   ├── utils.luma
│   └── types.luma
└── config.luma
```

```luma
-- main.luma
let utils = import("./lib/utils.luma")
let config = import("./config.luma")
let http = import("gh:luma-lang/http@1.0.0")

-- Use imported modules
let result = utils.process(config.settings)
```
