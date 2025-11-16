---
sidebar_position: 1
---

# Traits

Traits define interfaces that types can implement through structural matching.

## Defining a Trait

Traits are defined as tables with method signatures:

```luma
let Speakable = {
  speak = fn(self: Any): String
}
```

## Structural Matching

Objects that have the required fields automatically satisfy the trait:

```luma
let Dog = {
  name = String,
  speak = fn(self: Dog): String do
    return "Woof!"
  end
}

-- Dog satisfies Speakable because it has a speak method
```

## Trait Checking

Traits are checked:
- At `cast()` time
- At compile-time (when type checker is implemented)

```luma
let speakable: Speakable = cast(Speakable, dog)
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

## Generic Functions with Traits

Use traits to constrain generic functions:

```luma
let makeSpeak = fn(speaker: Speakable): String do
  return speaker.speak()
end

-- Works with any type that satisfies Speakable
let dog = Dog.new("Rex", "Beagle")
makeSpeak(dog)  -- "Woof!"
```

## Trait Composition

Traits can include other traits:

```luma
let Animal = {
  getName = fn(self: Any): String,
  speak = fn(self: Any): String,
  
  -- Combines Nameable and Speakable
}
```

## Structural vs Nominal Typing

Luma uses **structural typing** for traits, not nominal typing:

```luma
-- No explicit "implements" declaration needed
-- If it has the right shape, it satisfies the trait

let Cat = {
  speak = fn(self: Cat): String do "Meow!" end
}

-- Cat automatically satisfies Speakable
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

## Benefits

1. **Flexibility** - No need to declare trait implementations upfront
2. **Duck Typing** - If it walks like a duck and quacks like a duck...
3. **Compile-time Safety** - Still type-checked at compile time
4. **Code Reuse** - Write generic functions that work with any compatible type
