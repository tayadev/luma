---
sidebar_position: 4
---

# Type Conversions

Type conversions in Luma are performed using the `__into` method.

## The __into Method

Define custom conversions by implementing `__into`:

```luma
let Weight = {
  grams = Number,
  
  __into = fn(self: Weight, target: Type): Any do
    if target == String do
      return "${self.grams}g"
    end
    return null  -- Conversion not supported
  end,
  
  new = fn(grams: Number): Weight do
    return cast(Weight, { grams = grams })
  end
}
```

## Using the into() Function

The `into()` function internally calls `__into`:

```luma
let weight = Weight.new(1000)

let str: String = into(weight, String)  -- "1000g"
```

## Print and Conversions

The `print()` function internally uses `into(value, String)`:

```luma
let weight = Weight.new(500)
print(weight)  -- Calls into(weight, String) → "500g"
```

## Multiple Target Types

Support conversion to multiple types:

```luma
let Temperature = {
  celsius = Number,
  
  __into = fn(self: Temperature, target: Type): Any do
    match target do
      String do
        return "${self.celsius}°C"
      end
      Number do
        return self.celsius
      end
      Fahrenheit do
        return Fahrenheit.new(self.celsius * 9/5 + 32)
      end
      _ do
        return null
      end
    end
  end
}
```

## Optional Conversions

If `__into` is not defined or returns `null`, the conversion fails:

```luma
let Thing = {
  value = Number
  -- No __into defined
}

let thing = Thing.new(42)
-- print(thing)  -- Error: Cannot convert Thing to String
```

## Common Conversion Patterns

### To String

Most types should support conversion to String for printing:

```luma
__into = fn(self: MyType, target: Type): Any do
  if target == String do
    return "MyType(${self.field})"
  end
  return null
end
```

### To Number

For numeric types:

```luma
__into = fn(self: Distance, target: Type): Any do
  if target == Number do
    return self.meters
  end
  if target == String do
    return "${self.meters}m"
  end
  return null
end
```

### To Boolean

For types with boolean semantics:

```luma
let Optional = {
  value = Any,
  
  __into = fn(self: Optional, target: Type): Any do
    if target == Boolean do
      return self.value != null
    end
    if target == String do
      if self.value != null do
        return "Some(${self.value})"
      else do
        return "None"
      end
    end
    return null
  end
}
```

## Best Practices

1. **Always support String conversion** for debuggability:
   ```luma
   __into = fn(self: MyType, target: Type): Any do
     if target == String do return "..." end
     return null
   end
   ```

2. **Return null for unsupported conversions**:
   ```luma
   if target == SupportedType do
     -- convert
   end
   return null  -- not supported
   ```

3. **Make conversions intuitive**:
   - Distance to Number → meters (not kilometers)
   - Money to Number → cents (smallest unit)
   - Temperature to String → include unit

4. **Document conversion behavior**:
   ```luma
   --[[
   Conversions:
   - To String: Returns "Weight{grams}g"
   - To Number: Returns grams as Number
   ]]
   __into = fn(self: Weight, target: Type): Any do
     -- implementation
   end
   ```

## Explicit Conversions

Luma conversions are always **explicit**:

```luma
let weight = Weight.new(1000)

-- Explicit conversion required
let str = into(weight, String)

-- print() does automatic String conversion
print(weight)  -- OK, calls into(weight, String) internally
```

## Conversion Chains

Conversions don't chain automatically:

```luma
-- To convert A → C, must explicitly go through B
let b = into(a, B)
let c = into(b, C)
```

## Error Handling

Check if conversion is supported:

```luma
let result = into(value, TargetType)
if result == null do
  print("Conversion not supported")
end
```
