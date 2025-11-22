# Garbage Collection Hooks - Implementation Notes

## Current Status: Not Implemented

The `__gc` method is defined in the SPEC but not yet implemented in the VM.

## Why It's Complex

1. **Reference Counting**: Luma currently uses Rust's `Rc<RefCell<>>` for automatic memory management
   - Objects are freed when their reference count reaches zero
   - This happens automatically via Rust's Drop trait

2. **Calling `__gc` Requires VM Context**:
   - The `__gc` method is a Luma function that needs to be executed
   - Executing functions requires the VM (instruction pointer, stack, frames, etc.)
   - Rust's Drop trait is called in a destructor context where VM is not available

3. **Potential Solutions**:
   
   **Option A: Weak References + Explicit GC Pass**
   - Store weak references to objects with `__gc` methods
   - Run a periodic GC pass that checks weak refs and calls `__gc` for unreachable objects
   - Requires significant VM refactoring

   **Option B: Finalizer Queue**
   - When an object with `__gc` is created, register it in a finalizer queue
   - Custom Drop implementation adds object to "pending finalization" queue
   - VM processes the queue at appropriate times (e.g., before program exit)
   - Still complex because Drop can't safely interact with VM

   **Option C: Manual Resource Management**
   - Don't use `__gc` automatically
   - Require explicit `close()` or `dispose()` methods
   - Simpler but loses the automatic cleanup benefit

## Recommended Approach

For v2, implement **Option A** (Weak References + GC Pass):

```rust
struct GcRegistry {
    objects: Vec<Weak<RefCell<HashMap<String, Value>>>>,
}

impl VM {
    fn gc_collect(&mut self) {
        // Check weak references
        for weak in &self.gc_registry.objects {
            if weak.strong_count() == 0 {
                // Object is unreachable
                if let Some(table) = weak.upgrade() {
                    // Temporarily resurrected - call __gc
                    if let Some(gc_method) = table.borrow().get("__gc") {
                        // Execute gc_method(table)
                        // ...
                    }
                }
            }
        }
        // Clean up dead weak references
        self.gc_registry.objects.retain(|w| w.strong_count() > 0);
    }
}
```

## Workaround for Users

Until `__gc` is implemented, users should:

1. Define explicit cleanup methods:
```luma
let File = {
  handle = null,
  
  close = fn(self: File): Null do
    if self.handle != null do
      closeFile(self.handle)
      self.handle = null
    end
  end
}

let f = File.open("data.txt")
-- use file
f.close()  -- Explicit cleanup
```

2. Use defer patterns (when implemented):
```luma
let f = File.open("data.txt")
defer f.close()  -- Automatically called at end of scope
```

## Related Work

- Python: `__del__` method (similar issues, not guaranteed to run)
- Java: `finalize()` method (deprecated due to issues)
- C#: `IDisposable` interface with `using` statements
- Rust: Drop trait (but requires ownership model)

The consensus in the language design community is that finalizers are tricky and explicit resource management is often better.
