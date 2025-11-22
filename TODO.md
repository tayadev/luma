# TODOS

> If you work on a feature, please update this file to reflect its status. If during development you want to split it up into smaller tasks, feel free to add sub-tasks under the relevant section.

## MVP (v1) — In Progress

### Type System & Runtime
- [ ] Module system with `import()` (local files, URLs, git repos) _(for v1 only local files)_
- [ ] Async/await support (`await` keyword, `Promise` type)
- [ ] Operator overloading (`__add`, `__sub`, `__mul`, etc.)
- [ ] Conversions (`__into` method)
- [ ] Built-in `Result(Ok, Err)` and `Option(T)` types
- [ ] Garbage collection hooks (`__gc` method)

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