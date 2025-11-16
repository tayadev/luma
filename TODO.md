# TODOS

- [ ] write parser tests for all syntax features

## MVP (v1) — In Progress

- [ ] Type Checker (MVP)
	- [ ] `TcType` enum: Any, Unknown, Number, String, Boolean, Null, Array(T), Table, Function(args, ret)
	- [ ] `TypeEnv` with scoped symbols (name → { ty, mutable, annotated })
	- [ ] Check binary ops (`+ - * / %`) and comparisons (`== != < <= > >=`)
	- [ ] Check logical ops (`and`, `or`, `not`) → Boolean
	- [ ] Check functions: params/arity, return type vs implicit return

- [ ] Bytecode IR & Compiler
	- [x] `Instruction` enum + `Constant` pool; `Chunk`
	- [ ] Scopes: locals vs globals; enforce `let` immutability at compile time
	- [x] Codegen: literals (Number/String/Boolean/Null)
	- [x] Codegen: arithmetic binary ops (`+ - * / %`) and unary `-`
	- [x] Codegen: comparisons (`== != < <= > >=`)
	- [x] Codegen: logical ops with short-circuit (`and`, `or`, `not`)
	- [x] Codegen: identifiers, assignment (incl. compound) as globals-only
	- [ ] Codegen: member/index
	- [ ] Control flow: `if/elif/else`, `while`
	- [ ] `for` over arrays only (lower to while)

- [ ] VM (Stack-based)
	- [x] `Value`: Number, String, Boolean, Null
	- [x] Dispatch loop, stack, basic globals placeholder
	- [x] Implement ops: `CONST/POP/DUP`, arithmetic/comparison/unary, `JUMP/JUMP_IF_FALSE`, `HALT`
	- [x] Implement ops: GET/SET (global)
	- [ ] Implement ops: GET/SET (local), GET/SET_PROP, GET/SET_INDEX
	- [ ] BUILD_ARRAY/BUILD_TABLE, LOOP, MAKE_FUNCTION/CALL/RETURN

- [ ] CLI Integration
	- [x] Flags: `--ast` (print AST), `--check` (typecheck), `--run` (execute)
	- [x] Pipeline: parse → typecheck → compile → run; concise error reporting

- [ ] Integration Tests (end-to-end)
	- [ ] `run_arith.rs` — arithmetic, precedence, implicit program return
	- [ ] `run_strings.rs` — `+` concat and coercion
	- [ ] `run_arrays_tables.rs` — literals, indexing, nested
	- [ ] `run_control.rs` — if/elif/else, while
	- [ ] `run_functions.rs` — define/call, recursion, implicit returns
	- [ ] `run_assignments.rs` — let/var, compound ops, member/index assignment
	- [ ] `run_logical.rs` — short-circuit behavior
	- [ ] `run_for_arrays.rs` — `for x in [1,2,3]` lowering

	## Infra/Housekeeping

	- [x] Test folder reorg: move parser fixtures to `test/parser/fixtures`
	- [x] Move ad-hoc samples from `tmp/` to `test/runtime/`

## Deferred to v2

- [ ] Closures/upvalues (captured locals) + `CLOSURE`/upvalue handling
- [ ] Named arguments and default parameter evaluation in function prologue
- [ ] `for` over tables and a general iterator protocol
- [ ] Destructuring in `for` (beyond simple identifiers)
- [ ] Richer typing: unions, flow-sensitive typing, typed tables/arrays, generics
- [ ] Better diagnostics with source spans (node IDs/spans throughout)
- [ ] Error recovery in parser/typechecker
- [ ] Standard library expansion (beyond `print`, `len`)
- [ ] JIT compiler or compact byte encoding for instructions