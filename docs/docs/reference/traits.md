---
sidebar_position: 1
---

# Traits

Traits define interfaces that types can implement through structural typing.

## Defining a Trait

Traits are defined as tables with method signatures:

```luma
let Drawable = {
  draw = fn(self: Any): String
}
```

## Structural Typing

Luma uses **structural typing** for traits, not nominal typing. Types satisfy traits if they have the required fields, checked at `cast()` time.

```luma
let Circle = {
  radius = Number,

  draw = fn(self: Circle): String do
    return "Circle with radius ${self.radius}"
  end
}

-- Circle satisfies Drawable because it has a draw method
```

No explicit "implements" declaration is needed. If a type has the right shape, it satisfies the trait.

## Using Traits

### Generic Functions with Traits

Use traits to constrain generic functions:

```luma
let makeSpeak = fn(speaker: Speakable): String do
  return speaker.speak()
end

-- Works with any type that satisfies Speakable
let dog = Dog.new("Rex", "Beagle")
makeSpeak(dog)  -- "Woof!"
```

### Trait Checking

Traits are checked:
- At `cast()` time
- At compile-time (when type checker is implemented)

```luma
let drawable: Drawable = cast(Drawable, circle)
```

## Multiple Traits

A type can satisfy multiple traits:

```luma
let Nameable = {
  getName = fn(self: Any): String
}

let Describable = {
  describe = fn(self: Any): String
}

let Person = {
  name = String,
  age = Number,
  
  getName = fn(self: Person): String do
    return self.name
  end,
  
  describe = fn(self: Person): String do
    return "${self.name} is ${self.age} years old"
  end
}

-- Person satisfies both Nameable and Describable
```

## Trait Composition

Traits can include other traits by combining their fields:

```luma
let Animal = {
  getName = fn(self: Any): String,
  speak = fn(self: Any): String,
  
  -- Combines Nameable and Speakable
}
```

## Example: Iterable Trait

```luma
let Iterable = {
  __iter = fn(self: Any): Iterator,
  __next = fn(self: Any): Option(Any)
}

-- Arrays satisfy Iterable by default
let numbers = [1, 2, 3]
for n in numbers do
  print(n)
end
```

## Benefits of Structural Typing

1. **Flexibility** — No need to declare trait implementations upfront
2. **Duck Typing** — If it walks like a duck and quacks like a duck...
3. **Compile-time Safety** — Still type-checked at compile time
4. **Code Reuse** — Write generic functions that work with any compatible type

## When to Use Traits

Use traits when:
- You want to define interfaces without inheritance
- You need multiple interface implementation
- Types are unrelated but share behavior
- You want structural typing flexibility

Consider using inheritance when:
- There's a clear "is-a" relationship
- Child types are specializations of parent
- You want to share implementation
