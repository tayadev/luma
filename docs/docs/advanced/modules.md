---
sidebar_position: 3
---

# Modules and Imports

Luma has built-in dependency management through URL-based imports.

## Import Syntax

```luma
let module = import "path/to/module.luma"
```

## Import Types

### Local File Import

```luma
let utils = import "./utils.luma"
let config = import "../config.luma"
```

### HTTP(S) Import

```luma
let http = import "https://example.com/http-client.luma"
```

### Git Repository Import

```luma
let lib = import "git@github.com:user/repo.git"
```

### GitHub Shorthand

```luma
let lib = import "gh:user/repo@1.2.3"
```

The `@version` specifier is optional and supports:
- Tags: `@1.2.3`
- Branches: `@main`
- Commits: `@abc123`

## Module Structure

Modules export their value (usually a table with functions):

```luma
-- math_utils.luma
let MathUtils = {
  add = fn(a: Number, b: Number): Number do
    return a + b
  end,
  
  multiply = fn(a: Number, b: Number): Number do
    return a * b
  end
}

-- This is what gets imported
MathUtils
```

```luma
-- main.luma
let math = import "./math_utils.luma"
let result = math.add(2, 3)
```

## Directory Imports

When importing a directory URL, Luma looks for `main.luma`:

```luma
let package = import "https://example.com/my-package/"
-- Looks for https://example.com/my-package/main.luma
```

## Caching

- Imports are synchronous
- Remote modules are cached locally after first download
- Cache location varies by platform

## Lock File

Dependencies are locked in `luma.lock` for reproducible builds:

```luma
-- After first import, luma.lock records:
-- - Module URL
-- - Version/commit hash
-- - Integrity hash
```

## Import Properties

- `import()` is **synchronous** - it blocks until the module is loaded
- Modules are evaluated once and cached
- Circular imports are detected and reported as errors

## Best Practices

1. **Pin versions** for remote dependencies:
   ```luma
   let lib = import "gh:user/repo@1.2.3"
   ```

2. **Use relative imports** for local modules:
   ```luma
   let utils = import "./utils.luma"
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
let utils = import "./lib/utils.luma"
let config = import "./config.luma"
let http = import "gh:luma-lang/http@1.0.0"

-- Use imported modules
let result = utils.process(config.settings)
```
