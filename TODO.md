# TODOS

> If you work on a feature, please update this file to reflect its status. If during development you want to split it up into smaller tasks, feel free to add sub-tasks under the relevant section.

## CRITICAL: Spec Inconsistencies to Fix (PRIORITY 1)

These are implementation mistakes that don't match the language specification. They must be fixed before v1.

### Missing Core Features
- [ ] **Add wildcard pattern `_`**: Implement pattern matching wildcard as specified
  - Add `Pattern::Wildcard` variant to `src/ast.rs`
  - Update `src/parser/patterns.rs` to parse `_` as wildcard pattern
  - Add test fixtures for wildcard patterns
- [ ] **Add if expressions**: Implement `if` as expression (not just statement)
  - Add `Expr::If` variant to `src/ast.rs` with structure: `{ condition, then_expr, else_expr }`
  - Update `src/parser/expressions.rs` to parse if expressions in primary position
  - Note: Spec grammar shows `IfExpr ::= "if" Expr "do" Block ["else" "do" Block] "end"`
  - Add test fixtures for if expressions (e.g., `let max = if a > b do a else do b end`)
- [ ] **Add parenthesized expressions**: Support `(expr)` for precedence override
  - Update primary parser in `src/parser/mod.rs` to include `expr.delimited_by(just('('), just(')'))`
  - Add test fixtures for parenthesized expressions
- [ ] **Implement complete Type system**: Add GenericType and FunctionType
  - Extend `Type` enum in `src/ast.rs` with:
    - `GenericType { name: String, type_args: Vec<Type> }` (e.g., `Array(Number)`)
    - `FunctionType { param_types: Vec<Type>, return_type: Box<Type> }` (e.g., `fn(Number, String): Boolean`)
    - `Any` variant
  - Update type parser in `src/parser/mod.rs` to parse all type forms
  - Add test fixtures for generic and function types

### Missing Async Support (Deferred - see v1 tasks below)
- [ ] **Add `await` keyword**: Reserved in spec but not implemented (awaiting full async implementation)
  - Add `await` to KEYWORDS in `src/parser/lexer.rs`
  - Note: Full implementation deferred to "Async/await support" task below

## MVP (v1) — In Progress

### Type System & Runtime
- [ ] Module system with `import()` (local files, URLs, git repos) _(for v1 only local files)_
- [ ] Async/await support (`await` keyword, `Promise` type)
- [ ] User-defined types with `cast()` and `isInstanceOf()`
- [ ] Inheritance via `__parent`
- [ ] Traits (structural matching)
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