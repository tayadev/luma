---
sidebar_position: 2
---

# Inheritance

Luma supports single inheritance through the `__parent` field.

## Basic Inheritance

Define a parent type and extend it:

```luma
let Animal = {
  name = String,
  
  speak = fn(self: Animal): String do
    return "..."
  end
}

let Dog = {
  __parent = Animal,
  breed = String,
  
  speak = fn(self: Dog): String do
    return "Woof! I'm ${self.name}, a ${self.breed}"
  end
}
```

## Inheritance Semantics

**Field merging:**
- Child types inherit all fields from parent
- Methods can be overridden
- Constructor must initialize both parent and child fields

## Method Overriding

Child types can override parent methods:

```luma
let animal = Animal.new("Generic")
animal.speak()  -- "..."

let dog = Dog.new("Rex", "Beagle")
dog.speak()     -- "Woof! I'm Rex, a Beagle"
```

## Type Checking

Use `isInstanceOf()` to check type hierarchy:

```luma
let dog = Dog.new("Rex", "Beagle")

isInstanceOf(dog, Dog)     -- true
isInstanceOf(dog, Animal)  -- true (Dog extends Animal)
```

## Multiple Levels of Inheritance

```luma
let Animal = {
  name = String,
  breathe = fn(self: Animal): String do
    return "Breathing..."
  end
}

let Mammal = {
  __parent = Animal,
  furColor = String
}

let Dog = {
  __parent = Mammal,
  breed = String
}

-- Dog inherits from Mammal and Animal
let dog = Dog.new("Rex", "brown", "Beagle")
dog.breathe()  -- Inherited from Animal
```

## Constructor Patterns

Child constructors should initialize parent fields:

```luma
let Animal = {
  name = String,
  
  new = fn(name: String): Animal do
    return cast(Animal, { name = name })
  end
}

let Dog = {
  __parent = Animal,
  breed = String,
  
  new = fn(name: String, breed: String): Dog do
    let raw = {
      name = name,    -- Parent field
      breed = breed   -- Child field
    }
    return cast(Dog, raw)
  end
}
```

## Calling Parent Methods

Access parent methods through the parent type:

```luma
let Dog = {
  __parent = Animal,
  breed = String,
  
  speak = fn(self: Dog): String do
    let parentSound = Animal.speak(self)
    return parentSound + " Woof!"
  end
}
```

## Limitations

- **Single inheritance only** — A type can have only one `__parent`
- **No diamond problem** — The inheritance hierarchy must be a tree
- **No multiple inheritance** — Use traits for multiple interface implementation

## When to Use Inheritance

Use inheritance when:
- There's a clear "is-a" relationship
- Child types are specializations of parent
- You want to share implementation

Consider using traits when:
- You need multiple interface implementation
- Types are unrelated but share behavior
- You want structural typing

## Example: Shape Hierarchy

```luma
let Shape = {
  x = Number,
  y = Number,
  
  area = fn(self: Shape): Number do
    return 0  -- Override in child
  end,
  
  move = fn(self: Shape, dx: Number, dy: Number): Shape do
    self.x = self.x + dx
    self.y = self.y + dy
    return self
  end
}

let Circle = {
  __parent = Shape,
  radius = Number,
  
  area = fn(self: Circle): Number do
    return 3.14159 * self.radius * self.radius
  end
}

let Rectangle = {
  __parent = Shape,
  width = Number,
  height = Number,
  
  area = fn(self: Rectangle): Number do
    return self.width * self.height
  end
}
```
