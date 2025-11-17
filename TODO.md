# TODOS

- [ ] write parser tests for all syntax features

## MVP (v1) — In Progress

- [ ] Type Checker (MVP)
	- [ ] `TcType` enum: Any, Unknown, Number, String, Boolean, Null, Array(T), Table, Function(args, ret)
	- [ ] `TypeEnv` with scoped symbols (name → { ty, mutable, annotated })
	- [ ] Check binary ops (`+ - * / %`) and comparisons (`== != < <= > >=`)
	- [ ] Check logical ops (`and`, `or`, `not`) → Boolean
	- [ ] Check functions: params/arity, return type vs implicit return

- [x] Bytecode IR & Compiler
	- [x] `Instruction` enum + `Constant` pool; `Chunk`
	- [x] Scopes: locals vs globals; PopNPreserve for preserving return values
	- [x] Codegen: literals (Number/String/Boolean/Null)
	- [x] Codegen: arithmetic binary ops (`+ - * / %`) and unary `-`
	- [x] Codegen: comparisons (`== != < <= > >=`)
	- [x] Codegen: logical ops with short-circuit (`and`, `or`, `not`)
	- [x] Codegen: identifiers, assignment for locals and globals
	- [x] Codegen: member/index reads
	- [x] Codegen: member/index writes
	- [x] Control flow: `if/elif/else`, `while` with proper scoping
	- [x] Functions: definitions and calls with call frames
	- [ ] `for` over arrays only (lower to while)

- [x] VM (Stack-based)
	- [x] `Value`: Number, String, Boolean, Null, Array, Table, Function
	- [x] Dispatch loop, stack, globals HashMap
	- [x] Implement ops: `CONST/POP/DUP/PopNPreserve`, arithmetic/comparison/unary, `JUMP/JUMP_IF_FALSE`, `HALT`
	- [x] Implement ops: GET/SET (global)
	- [x] Implement ops: GET/SET (local) with base pointer
	- [x] Implement ops: GET_PROP, GET_INDEX
	- [x] Implement ops: SET_PROP, SET_INDEX
	- [x] BUILD_ARRAY/BUILD_TABLE
	- [x] MAKE_FUNCTION/CALL/RETURN with call frames

- [x] CLI Integration
	- [x] Flags: `--ast` (print AST), `--check` (typecheck), `--run` (execute)
	- [x] Pipeline: parse → typecheck → compile → run; concise error reporting

- [ ] Integration Tests (end-to-end)
	- [x] `test_arith` — arithmetic, precedence, implicit program return
	- [x] `test_assign` — globals with compound assignment
	- [x] `test_cmp`, `test_logic_and`, `test_logic_or`, `test_not` — comparisons and logical ops
	- [x] arrays/tables — literals, member/index reads (`array_read`, `table_read`)
	- [x] `if_then_else`, `if_elif_else`, `if_local` — if/elif/else control flow with scoping
	- [x] `while_sum` — while loops
	- [x] `fn_simple`, `fn_multiarg` — function definitions and calls
	- [ ] `run_for_arrays.rs` — `for x in [1,2,3]` lowering
	- [ ] member/index assignment tests
	- [ ] immutability enforcement (`let` vs `var`)

	## Infra/Housekeeping

	- [x] Test folder reorg: move parser fixtures to `tests/fixtures`
	- [x] Move ad-hoc samples from `tmp/` to `tests/runtime/`
	- [x] Runtime test harness with .ron expectations

## Known Limitations (Current MVP)

- **Recursion**: Functions cannot recursively call themselves by name because the function value is assigned to the global *after* compilation. Requires either forward declarations or a two-pass approach.
- **Closures**: Captured variables from outer scopes not supported (deferred to v2).

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