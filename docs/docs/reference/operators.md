---
sidebar_position: 3
---

# Operator Overloading

Luma allows you to define custom behavior for operators on your types.

## Supported Operators

You can overload the following operators:

### Arithmetic Operators
- `+` - Addition (`__add`)
- `-` - Subtraction (`__sub`)
- `*` - Multiplication (`__mul`)
- `/` - Division (`__div`)
- `%` - Modulo (`__mod`)
- Unary `-` - Negation (`__neg`)

### Comparison Operators
- `==` - Equality (`__eq`)
- `<` - Less than (`__lt`)
- `<=` - Less than or equal (`__le`)
- `>` - Greater than (`__gt`)
- `>=` - Greater than or equal (`__ge`)

### Auto-derived
- `!=` - Not equal (auto-derived from `__eq`)

## Defining Operator Overloads

Define operator overloads as methods on your type:

```luma
let Vector2 = {
  x = Number,
  y = Number,
  
  __add = fn(a: Vector2, b: Vector2): Vector2 do
    return Vector2.new(a.x + b.x, a.y + b.y)
  end,
  
  new = fn(x: Number, y: Number): Vector2 do
    return cast(Vector2, { x = x, y = y })
  end
}

let v1 = Vector2.new(1, 2)
let v2 = Vector2.new(3, 4)
let v3 = v1 + v2  -- Vector2(4, 6)
```

## Arithmetic Examples

### Addition

```luma
let Complex = {
  real = Number,
  imag = Number,
  
  __add = fn(a: Complex, b: Complex): Complex do
    return Complex.new(a.real + b.real, a.imag + b.imag)
  end,
  
  new = fn(real: Number, imag: Number): Complex do
    return cast(Complex, { real = real, imag = imag })
  end
}

let c1 = Complex.new(1, 2)
let c2 = Complex.new(3, 4)
let c3 = c1 + c2  -- Complex(4, 6)
```

### Multiplication

```luma
let Matrix = {
  values = Array(Array(Number)),
  
  __mul = fn(a: Matrix, b: Matrix): Matrix do
    -- Matrix multiplication logic
  end
}
```

### Unary Negation

```luma
let Vector2 = {
  x = Number,
  y = Number,
  
  __neg = fn(self: Vector2): Vector2 do
    return Vector2.new(-self.x, -self.y)
  end
}

let v = Vector2.new(3, 4)
let negV = -v  -- Vector2(-3, -4)
```

## Comparison Examples

### Equality

```luma
let Point = {
  x = Number,
  y = Number,
  
  __eq = fn(a: Point, b: Point): Boolean do
    return a.x == b.x and a.y == b.y
  end
}

let p1 = Point.new(1, 2)
let p2 = Point.new(1, 2)
let p3 = Point.new(3, 4)

p1 == p2  -- true
p1 == p3  -- false
p1 != p3  -- true (auto-derived from __eq)
```

### Ordering

```luma
let Version = {
  major = Number,
  minor = Number,
  patch = Number,
  
  __lt = fn(a: Version, b: Version): Boolean do
    if a.major != b.major do return a.major < b.major end
    if a.minor != b.minor do return a.minor < b.minor end
    return a.patch < b.patch
  end,
  
  __eq = fn(a: Version, b: Version): Boolean do
    return a.major == b.major and 
           a.minor == b.minor and 
           a.patch == b.patch
  end
}

let v1 = Version.new(1, 2, 3)
let v2 = Version.new(1, 3, 0)

v1 < v2   -- true
v1 <= v2  -- true
v2 > v1   -- true
v2 >= v1  -- true
```

## String Concatenation

The `+` operator is also used for string concatenation:

```luma
let Name = {
  first = String,
  last = String,
  
  __add = fn(a: Name, b: Name): String do
    return a.first + " " + a.last + " & " + b.first + " " + b.last
  end
}
```

## Non-Overloadable Operators

These operators cannot be overloaded:
- `and` - Logical AND
- `or` - Logical OR  
- `not` - Logical NOT
- `[]` - Array/table indexing
- `in` - Membership test
- `.` - Field access

## Best Practices

1. **Follow mathematical conventions**
   - `+` should be commutative when appropriate
   - Operators should behave intuitively

2. **Maintain consistency**
   ```luma
   -- If you define __add, consider defining __sub
   -- If you define __lt, consider defining __le, __gt, __ge
   ```

3. **Return appropriate types**
   ```luma
   __add = fn(a: Vector, b: Vector): Vector  -- same type
   __lt = fn(a: Version, b: Version): Boolean  -- comparison returns bool
   ```

4. **Document behavior**
   ```luma
   --[[
   Defines vector addition.
   Returns a new vector with component-wise sum.
   ]]
   __add = fn(a: Vector, b: Vector): Vector do
     -- implementation
   end
   ```
