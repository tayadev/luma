---
sidebar_position: 5
---

# Garbage Collection

Luma features automatic memory management through garbage collection.

## Automatic Memory Management

Luma automatically manages memory for:
- Tables
- Arrays
- Strings
- Functions
- Closures

You don't need to manually allocate or free memory.

## Heap Allocation

The following are heap-allocated and managed by the GC:
- **Tables** — All table instances
- **Arrays** — All array values
- **Strings** — String values
- **Functions** — Function objects and closures
- **Closures** — Functions that capture variables from outer scopes

## Reference Semantics

**Value types** (numbers, booleans, null): copied by value  
**Reference types** (tables, arrays, functions): copied by reference

```luma
let a = [1, 2, 3]
let b = a                          -- b references same array
b[0] = 99
print(a[0])                        -- 99
```

## The __gc Method

Define a `__gc` method that is called when an object is collected:

```luma
let Resource = {
  handle = Any,
  
  __gc = fn(self: Resource): Null do
    print("Cleaning up resource: ${self.handle}")
    cleanup(self.handle)
  end
}
```

**Note:** Finalization is not guaranteed to run immediately or in any particular order.

## Garbage Collection Behavior

- **Non-deterministic** — You cannot predict exactly when GC runs
- **Automatic** — Runs when allocation threshold is reached or memory pressure is high
- **Mark and sweep** — The GC marks reachable objects and collects unreachable ones

**Collection triggers:**
- When allocation threshold is reached
- When memory pressure is high
- Manual collection via `gc.collect()` (if exposed)

## Resource Management

### Using __gc for Cleanup

```luma
let File = {
  path = String,
  handle = FileHandle,
  
  __gc = fn(self: File): Null do
    if self.handle != null do
      closeFile(self.handle)
    end
  end
}
```

### Explicit Cleanup

While `__gc` provides automatic cleanup, you can also provide explicit cleanup methods:

```luma
let Connection = {
  socket = Socket,
  
  close = fn(self: Connection): Null do
    closeSocket(self.socket)
    self.socket = null
  end,
  
  __gc = fn(self: Connection): Null do
    if self.socket != null do
      closeSocket(self.socket)
    end
  end
}

-- Explicit close
let conn = Connection.new()
-- use connection
conn.close()  -- Explicitly close when done
```

## Avoiding Memory Leaks

### Circular References

Circular references are handled automatically by the garbage collector:

```luma
let nodeA = { value = 1 }
let nodeB = { value = 2 }

nodeA.next = nodeB
nodeB.prev = nodeA

-- Both will be collected when no external references exist
```

### Long-lived References

Be careful with long-lived collections:

```luma
var globalCache = {}

let addToCache = fn(key: String, value: Any): Null do
  globalCache[key] = value  -- Value stays in memory
end

-- Clear cache when no longer needed
let clearCache = fn(): Null do
  globalCache = {}
end
```

## Best Practices

1. **Use __gc for cleanup**:
   ```luma
   __gc = fn(self: Type): Null do
     -- Close files, sockets, etc.
   end
   ```

2. **Provide explicit cleanup methods**:
   ```luma
   close = fn(self: Resource): Null do
     -- Cleanup immediately
   end
   ```

3. **Clear references when done**:
   ```luma
   var bigData = loadLargeFile()
   process(bigData)
   bigData = null  -- Allow GC to collect
   ```

4. **Avoid retaining unnecessary references**:
   ```luma
   -- Bad: Keeps entire array in memory
   var cache = []
   for item in items do
     cache.push(item)
   end
   
   -- Good: Process and discard
   for item in items do
     process(item)
   end
   ```

## When __gc is NOT Called

`__gc` may not be called if:
- The program terminates abnormally
- The object is still reachable when the program exits
- The GC hasn't run yet when the program ends

For critical resources, prefer explicit cleanup:

```luma
let conn = Connection.new()
-- Use connection
conn.close()  -- Don't rely solely on __gc
```

## Performance Considerations

- **Allocation is fast** — Creating objects is cheap
- **Collection is periodic** — GC pauses are generally short
- **No manual memory management** — Focus on logic, not memory

## Compared to Manual Memory Management

Garbage collection eliminates:
- Memory leaks from forgetting to free
- Use-after-free bugs
- Double-free errors
