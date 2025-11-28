---
sidebar_position: 3
---

# Modules and Imports

:::caution Work in Progress
The module system is currently under development. This documentation describes the planned design.
:::

Luma has a modern, URL-based module system for flexible dependency management. Modules are loaded on-demand and cached locally.

## Import Syntax

```luma
let module = import("source")
```

`import()` is a built-in function that loads a module and returns its exported value.

## Import Sources

### Local Files

Use relative paths for local modules:

```luma
let utils = import("./utils.luma")
let helpers = import("../lib/helpers.luma")
let config = import("./config/app.luma")
```

### Remote URLs

Import modules from HTTP(S) URLs:

```luma
let http = import("https://example.com/http.luma")
let json = import("https://cdn.example.org/json.luma")
```

### GitHub Repositories

Use the shorthand `gh:` prefix for GitHub repos:

```luma
-- Specific version
let lib = import("gh:user/repo@1.2.3")

-- Latest version
let lib = import("gh:user/repo@latest")

-- Specific branch
let lib = import("gh:user/repo@main")
```

### Other Git Sources

```luma
let lib = import("git@github.com:user/repo.git@main")
let lib = import("https://git.example.com/user/repo.git@v1.0.0")
```

## Module Resolution

When you import a module, Luma:

1. **Checks the cache** — Is this module already loaded?
2. **Downloads (if needed)** — Fetch from the source URL
3. **Verifies integrity** — Check SHA256 hash against lock file (if present)
4. **Parses** — Load and parse the source code
5. **Evaluates** — Execute the module in an isolated scope
6. **Returns** — Exports the module's value

### Cache Location

Module cache is stored in: `~/.luma/cache/modules/`

## Exporting from Modules

Modules export the value of their **last expression**:

```luma
-- math.luma
let PI = 3.14159
let E = 2.71828

let add = fn(a, b) do a + b end
let multiply = fn(a, b) do a * b end

{
  PI = PI,
  E = E,
  add = add,
  multiply = multiply
}
```

Usage:

```luma
-- main.luma
let math = import("./math.luma")
print(math.PI)                  -- 3.14159
print(math.add(2, 3))           -- 5
print(math.multiply(4, 5))      -- 20
```

### Exporting a Single Function

Modules can export a single value:

```luma
-- utils.luma
fn transform(data) do
  -- transformation logic
end

transform
```

Usage:

```luma
let transform = import("./utils.luma")
let result = transform(myData)
```

## Dependency Locking

Create a `luma.lock` file to lock dependency versions and ensure reproducibility:

```json
{
  "dependencies": {
    "https://example.com/http.luma": {
      "version": "1.2.3",
      "integrity": "sha256-abc123...",
      "resolved": "2024-01-15T10:30:00Z"
    },
    "gh:user/repo@1.0.0": {
      "url": "https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz",
      "integrity": "sha256-def456...",
      "resolved": "2024-01-15T10:30:00Z"
    }
  }
}
```

**Best practice:** Commit `luma.lock` to version control to ensure all developers use the same versions.

## Circular Dependencies

Circular imports are detected and reported as errors:

```luma
-- a.luma
let b = import("./b.luma")

-- b.luma
let a = import("./a.luma")  -- Error! Circular dependency

--
-- Error: Circular dependency detected:
--   a.luma -> b.luma -> a.luma
```

## Module Properties

- **Synchronous** — `import()` blocks until loaded
- **Cached** — Modules are evaluated once and results are reused
- **Isolated** — Each module has its own scope
- **Deterministic** — Lock file ensures reproducible builds

## Project Structure

Example project layout:

```
myapp/
├── main.luma           # Entry point
├── luma.lock           # Dependency lock file
├── src/
│   ├── config.luma     # Configuration
│   ├── db.luma         # Database module
│   └── utils/
│       ├── string.luma
│       └── array.luma
└── lib/
    └── types.luma      # Type definitions
```

## Example: Modular Application

```luma
-- config.luma
{
  debug = true,
  port = 3000,
  database = "postgres://localhost/myapp"
}
```

```luma
-- src/db.luma
let config = import("../config.luma")

{
  connect = fn() do
    -- Connect using config.database
  end,
  
  query = fn(sql) do
    -- Execute query
  end
}
```

```luma
-- src/utils/string.luma
{
  uppercase = fn(s) do s.toUpperCase() end,
  lowercase = fn(s) do s.toLowerCase() end,
  reverse = fn(s) do s.split("").reverse().join("") end
}
```

```luma
-- main.luma
let config = import("./config.luma")
let db = import("./src/db.luma")
let string = import("./src/utils/string.luma")

-- Use the modules
db.connect()
print(string.uppercase("hello"))
```

## Best Practices

1. **Pin versions for stability:**
   ```luma
   let lib = import("gh:user/repo@1.2.3")  -- ✅ Good
   let lib = import("gh:user/repo@main")   -- ⚠️ Risky
   ```

2. **Use relative imports locally:**
   ```luma
   let utils = import("./utils.luma")
   let types = import("../types/common.luma")
   ```

3. **Create table exports for multiple items:**
   ```luma
   {
     func1 = fn() do ... end,
     func2 = fn() do ... end,
     Type = { ... }
   }
   ```

4. **Document public APIs in modules**

5. **Commit lock files** to version control

## Planned Features

- **Namespace aliasing** — `import("module.luma") as mod`
- **Selective imports** — `import({ func1, func2 } from "module.luma")`
- **Re-exports** — Pass through imports from other modules
- **Package manager** — Registry for publishing and discovering modules
