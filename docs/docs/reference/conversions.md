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
    match target do
      String do
        return self.grams + "g"
      end
      Number do
        return self.grams
      end
      _ do
        return null  -- Conversion not supported
      end
    end
  end,
  
  new = fn(grams: Number): Weight do
    return cast(Weight, { grams = grams })
  end
}
```

## Using Conversions

Use `into()` to convert values:

```luma
let weight = Weight.new(1000)

let str: String = into(weight, String)  -- "1000g"
let num: Number = into(weight, Number)  -- 1000
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
        return self.celsius + "°C"
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

## Common Conversions

### To String

Most types should support conversion to String for printing:

```luma
__into = fn(self: MyType, target: Type): Any do
  match target do
    String do
      return "MyType(" + self.field + ")"
    end
    _ do
      return null
    end
  end
end
```

### To Number

For numeric types:

```luma
__into = fn(self: Distance, target: Type): Any do
  match target do
    Number do
      return self.meters
    end
    String do
      return self.meters + "m"
    end
    _ do
      return null
    end
  end
end
```

### To Boolean

For types with boolean semantics:

```luma
let Optional = {
  value = Any,
  
  __into = fn(self: Optional, target: Type): Any do
    match target do
      Boolean do
        return self.value != null
      end
      String do
        if self.value != null do
          return "Some(" + self.value + ")"
        else do
          return "None"
        end
      end
      _ do
        return null
      end
    end
  end
}
```

## Best Practices

1. **Always support String conversion** for debuggability:
   ```luma
   __into = fn(self: MyType, target: Type): Any do
     match target do
       String do return "..." end
       _ do return null end
     end
   end
   ```

2. **Return null for unsupported conversions**:
   ```luma
   match target do
     SupportedType do -- convert
     _ do return null end  -- not supported
   end
   ```

3. **Make conversions intuitive**:
   ```luma
   -- Distance to Number → meters (not kilometers)
   -- Money to Number → cents (smallest unit)
   -- Temperature to String → include unit
   ```

4. **Document conversion behavior**:
   ```luma
   --[[
   Conversions:
   - To String: Returns "Weight{grams}g"
   - To Number: Returns grams as Number
   - To Kilograms: Returns Weight in kg
   ]]
   __into = fn(self: Weight, target: Type): Any do
     -- implementation
   end
   ```

## Implicit vs Explicit Conversions

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
