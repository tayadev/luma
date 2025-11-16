---
sidebar_position: 5
---

# Garbage Collection

Luma features automatic memory management through garbage collection.

## Automatic Memory Management

Luma automatically manages memory for:
- Tables
- Functions
- Closures
- Strings
- Arrays

You don't need to manually allocate or free memory.

## Heap Allocation

The following are heap-allocated:
- **Tables** - All table instances
- **Functions** - Function objects and closures
- **Closures** - Functions that capture variables from outer scopes
- **Strings** - String values
- **Arrays** - Array values

## The __gc Method

Define a `__gc` method that is called when an object is collected:

```luma
let Resource = {
  handle = Number,
  name = String,
  
  __gc = fn(self: Resource): None do
    print("Cleaning up resource: " + self.name)
    closeHandle(self.handle)
  end,
  
  new = fn(name: String): Resource do
    let handle = openHandle()
    return cast(Resource, {
      handle = handle,
      name = name
    })
  end
}
```

## Garbage Collection Behavior

- **Non-deterministic** - You cannot predict exactly when GC runs
- **Automatic** - Runs when memory pressure increases
- **Mark and sweep** - The GC marks reachable objects and collects unreachable ones

## Resource Management

### Using __gc for Cleanup

```luma
let File = {
  path = String,
  handle = FileHandle,
  
  __gc = fn(self: File): None do
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
  
  close = fn(self: Connection): None do
    closeSocket(self.socket)
    self.socket = null
  end,
  
  __gc = fn(self: Connection): None do
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

## Memory Leaks

Garbage collection prevents most memory leaks, but be aware of:

### Circular References

Circular references are handled automatically:

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

let addToCache = fn(key: String, value: Any): None do
  globalCache[key] = value  -- Value stays in memory
end

-- Clear cache when no longer needed
let clearCache = fn(): None do
  globalCache = {}
end
```

## Best Practices

1. **Use __gc for cleanup**:
   ```luma
   __gc = fn(self: Type): None do
     -- Close files, sockets, etc.
   end
   ```

2. **Provide explicit cleanup methods**:
   ```luma
   close = fn(self: Resource): None do
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

## GC Tuning

The GC behavior is currently not configurable, but may become so in future versions:

```luma
-- Future: Configure GC
-- gc.setThreshold(megabytes)
-- gc.collect()  -- Force collection
```

## Performance Considerations

- **Allocation is fast** - Creating objects is cheap
- **Collection is periodic** - GC pauses are generally short
- **No manual memory management** - Focus on logic, not memory

## Compared to Manual Memory Management

### Luma (GC)
```luma
let data = loadData()
process(data)
-- Automatically freed when no longer referenced
```

### C (Manual)
```c
Data* data = malloc(sizeof(Data));
loadData(data);
process(data);
free(data);  // Must remember to free
```

Garbage collection eliminates:
- Memory leaks from forgetting to free
- Use-after-free bugs
- Double-free errors

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
