# TODOS

> If you work on a feature, please update this file to reflect its status. If during development you want to split it up into smaller tasks, feel free to add sub-tasks under the relevant section.

## MVP (v1) — In Progress

### Type System & Runtime
- [ ] Module system with `import()` (local files, URLs, git repos) _(for v1 only local files)_
- [x] Operator overloading (`__add`, `__sub`, `__mul`, etc.) — **COMPLETED**
  - [x] Arithmetic operators: `__add`, `__sub`, `__mul`, `__div`, `__mod`
  - [x] Unary negation: `__neg`
  - [x] Comparison operators: `__eq`, `__lt`, `__le`, `__gt`, `__ge`
  - [x] VM checks both value and type definition (via `__type`) for special methods
  - Note: Typechecker limitations prevent comprehensive automated testing
- [ ] Conversions (`__into` method) — **PARTIAL**
  - [x] `into()` native function registered
  - [ ] Full `__into` method calling support (requires VM context)
- [ ] Garbage collection hooks (`__gc` method) — **TODO**
  - Current implementation uses Rc<RefCell<>> reference counting
  - Need to add finalizer support via Drop trait or explicit GC
- [ ] Built-in `Result(Ok, Err)` and `Option(T)` types
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