# TODOS

> If you work on a feature, please update this file to reflect its status. If during development you want to split it up into smaller tasks, feel free to add sub-tasks under the relevant section.

## MVP (v1) — In Progress

### Type System & Runtime
- [x] Module system with `import()` (local files, URLs, git repos) _(for v1 only local files)_

- [x] **Operator overloading (`__add`, `__sub`, `__mul`, etc.) — COMPLETED**
  - [x] Arithmetic operators: `__add`, `__sub`, `__mul`, `__div`, `__mod`
  - [x] Unary negation: `__neg` (added new `Neg` bytecode instruction)
  - [x] Comparison operators: `__eq`, `__lt`, `__le`, `__gt`, `__ge` (auto-derives `!=`)
  - [x] VM checks both value and type definition (via `__type`) for special methods
  - **Implementation**: 
    - When operator fails default operation (e.g., Number + Number), VM checks for special method
    - Method lookup: first checks value itself, then `__type` metadata from `cast()`
    - Uses `call_overload_method` helper to set up function call with proper stack frames
    - Code locations: `src/vm/interpreter.rs` (lines 55-305), `src/bytecode/ir.rs` (Neg instruction)
  - **Usage example**:
    ```luma
    let Vector2 = {
      x = 0, y = 0,
      __add = fn(a: Any, b: Any): Any do
        return { x = a["x"] + b["x"], y = a["y"] + b["y"] }
      end
    }
    let v1 = cast(Vector2, { x = 1, y = 2 })
    let v2 = cast(Vector2, { x = 3, y = 4 })
    let v3 = v1 + v2  -- Result: {x: 4, y: 6}
    ```
  - **Known limitation**: Typechecker doesn't allow member access on `Any` type parameters
    - Workaround: Use bracket notation `value["field"]` instead of `value.field`
    - This is pre-existing, tracked under "Mutual recursion" in v2 items

- [ ] **Conversions (`__into` method) — PARTIAL**
  - [x] `into()` native function registered with basic primitive conversions
  - [ ] Full `__into` method calling support (deferred to v2)
  - **Why incomplete**: Native functions can't easily call Luma functions (requires VM execution context)
  - **Current workaround**: Use explicit conversion methods on types
  - **For v2**: Refactor to allow native functions to invoke user-defined `__into` methods

- [ ] **Garbage collection hooks (`__gc` method) — DEFERRED TO V2**
  - Current implementation uses `Rc<RefCell<>>` reference counting (automatic via Rust's Drop)
  - **Why deferred**: 
    - `__gc` method needs VM context to execute (instruction pointer, stack, frames)
    - Rust's Drop trait runs in destructor context where VM is unavailable
    - Cannot safely call Luma functions during Drop
  - **Recommended approach for v2** (Option A: Weak References + Explicit GC Pass):
    1. Store weak references to objects with `__gc` methods in a registry
    2. Run periodic GC pass that checks weak refs for unreachable objects
    3. Temporarily resurrect objects and call their `__gc` methods
    4. Clean up dead weak references from registry
  - **Alternative approaches considered**:
    - Option B: Finalizer queue (Drop adds to queue, VM processes periodically)
    - Option C: Manual resource management only (simpler but loses automatic cleanup)
  - **User workaround**: Define explicit cleanup methods like `close()` or `dispose()`
    ```luma
    let File = {
      close = fn(self: File): Null do
        closeFile(self.handle)
        self.handle = null
      end
    }
    -- Manual cleanup
    let f = File.open("data.txt")
    f.close()
    ```
  - **Related work**: Python `__del__`, Java `finalize()` (deprecated), C# `IDisposable`, Rust Drop trait

- [x] **Built-in `Result(Ok, Err)` and `Option(T)` types — COMPLETED**
  - [x] `Result` type with `new_ok`, `new_err`, `is_ok`, `is_err`, `unwrap`, `unwrap_or` methods
  - [x] `Option` type with `new_some`, `new_none`, `is_some`, `is_none`, `unwrap`, `unwrap_or` methods
  - **Implementation**: Defined in `src/prelude.luma` (lines 9-82)
  - **Tests**: `tests/runtime/prelude_result.luma` and `tests/runtime/prelude_option.luma`
  - **Usage example**:
    ```luma
    let result = Result.new_ok(42)
    if Result.is_ok(result) do
      print(Result.unwrap(result))  -- prints 42
    end
    
    let maybe = Option.new_some("hello")
    let value = Option.unwrap_or(maybe, "default")
    ```
  - **Pattern matching**: Works with standard `match` statements checking `err == null` or `none == true`
  - **Convention**: For Result, exactly one of `ok`/`err` is null; for Option, `none` is a boolean flag

- [x] **Built-in `typeof()` function — COMPLETED**
  - Returns the runtime type name as a string
  - **Implementation**: `native_typeof` in `src/vm/interpreter.rs` (lines 1265-1295)
  - **Test**: `tests/runtime/typeof_test.luma`
  - **Usage example**:
    ```luma
    let type_name = typeof(42)        -- "Number"
    let str_type = typeof("hello")    -- "String"
    let arr_type = typeof([1, 2, 3])  -- "Array"
    ```
  - **Supported types**: Number, String, Boolean, Null, Array, Table, Function, Type

- [ ] Other std (prelude.luma) enhancements and expansion (only the very basics should be in the prelude, rest should be in optional packages)
- [ ] Async/await support (`await` keyword, `Promise` type)

## Deferred to v2

- [ ] Mutual recursion across separate function declarations (requires typecheck to pre-declare all globals before checking them)
- [ ] Closures/upvalues (captured locals) + `CLOSURE`/upvalue handling
- [ ] `for` over tables and a general iterator protocol (should be a trait)
- [ ] Destructuring in `for` (beyond simple identifiers)
- [ ] Richer typing: unions, flow-sensitive typing, typed tables/arrays, generics
- [ ] Better diagnostics with source spans (node IDs/spans throughout)
- [ ] Error recovery in parser/typechecker
- [ ] Standard library expansion (beyond `print`)

- [ ] JIT compiler or compact byte encoding for instructions
