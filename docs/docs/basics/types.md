---
sidebar_position: 4
---

# Types and Instances

Luma has a rich type system that supports custom types, inheritance, and traits.

## Built-in Types

Luma provides several built-in types:

- `Any` - Supertype of all types
- `Number` - 64-bit floating point
- `String` - UTF-8 string
- `Boolean` - `true` or `false`
- `Array(T)` - Array of elements of type T
- `Table` - Untyped table (key-value pairs)
- `Result(OkType, ErrType)` - Success/failure result
- `Option(T)` - Optional value (may be `null`)

## Defining Custom Types

Types are defined as tables with fields and methods:

```luma
let Dog = {
  name = String,
  breed = String,

  speak = fn(self: Dog): String do
    return "Woof! I am a " + self.breed
  end,

  new = fn(name: String, breed: String): Dog do
    let raw = { name = name, breed = breed }
    return cast(Dog, raw)
  end
}
```

## Creating Instances

Use the `new` method (or any constructor pattern):

```luma
let rex = Dog.new("Rex", "Beagle")
let sound = rex.speak()  -- "Woof! I am a Beagle"
```

## Type Casting

The `cast()` function validates fields and attaches the type prototype:

```luma
let rex = cast(Dog, { name = "Rex", breed = "Beagle" })
```

`cast(Type, table)` performs:
1. Field validation
2. Prototype attachment
3. Inheritance merging

## Fields

Fields are declared as `field = Type`:

```luma
let Person = {
  name = String,
  age = Number,
  tags = Array(String)
}
```

## Methods

Methods are fields that hold function values:

```luma
let Counter = {
  count = Number,
  
  increment = fn(self: Counter): Number do
    self.count = self.count + 1
    return self.count
  end
}
```

## Type Checking

Use `isInstanceOf()` to check if a value is of a specific type:

```luma
let rex = Dog.new("Rex", "Beagle")
isInstanceOf(rex, Dog)  -- true
```

## Next Steps

Learn about [traits](../reference/traits.md), [inheritance](../reference/inheritance.md), or [operator overloading](../reference/operators.md).
