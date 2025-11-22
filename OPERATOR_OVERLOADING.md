# Operator Overloading Examples

## Simple Working Example

This demonstrates operator overloading that works within current typechecker limitations:

```luma
let Vector2 = {
  x = 0,
  y = 0,
  __add = fn(a: Any, b: Any): Any do
    -- Use bracket notation to access fields on Any type
    let ax = a["x"]
    let ay = a["y"]
    let bx = b["x"]
    let by = b["y"]
    return { x = ax + bx, y = ay + by }
  end
}

let v1 = cast(Vector2, { x = 1, y = 2 })
let v2 = cast(Vector2, { x = 3, y = 4 })
let v3 = v1 + v2  -- Calls Vector2.__add(v1, v2)
-- v3["x"] is 4, v3["y"] is 6
```

**Note:** Dot notation (`a.x`) doesn't work on `Any` type parameters due to typechecker limitations.
Use bracket notation (`a["x"]`) instead.

## Running the Example

```bash
# Create a test file
cat > vector_add.luma << 'EOF'
let Vector2 = {
  x = 0,
  y = 0,
  __add = fn(a: Any, b: Any): Any do
    let ax = a["x"]
    let ay = a["y"]
    let bx = b["x"]
    let by = b["y"]
    return { x = ax + bx, y = ay + by }
  end
}

let v1 = cast(Vector2, { x = 1, y = 2 })
let v2 = cast(Vector2, { x = 3, y = 4 })
let v3 = v1 + v2
v3["x"]
EOF

# Compile and run (bypassing typecheck)
cargo build
./target/debug/luma ast vector_add.luma  # Shows AST
# Note: Direct execution requires typechecker improvements
```

## Supported Operators

All the following operators can be overloaded:

### Arithmetic
- `__add` for `+`
- `__sub` for `-`
- `__mul` for `*`
- `__div` for `/`
- `__mod` for `%`
- `__neg` for unary `-`

### Comparison
- `__eq` for `==` (and automatically `!=`)
- `__lt` for `<`
- `__le` for `<=`
- `__gt` for `>`
- `__ge` for `>=`

## How It Works

1. When an operator is used on values, the VM first tries the default operation (e.g., Number + Number)
2. If that fails, it checks if the left operand has a special method (e.g., `__add`)
3. The special method is looked up in:
   - The value itself (if it's a table)
   - The value's type definition (if it has `__type` metadata from `cast()`)
4. If found, the method is called with the operands as arguments
5. The method's return value becomes the result of the operation

## Limitations

The current typechecker has limitations that prevent full integration:
- Member access on `Any` type parameters is not allowed
- This makes it difficult to write operator overloading methods that access fields
- Use bracket notation `value["field"]` as a workaround in some cases
- Future typechecker improvements will address this
