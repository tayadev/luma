# TODOS

> If you work on a feature, please update this file to reflect its status. If during development you want to split it up into smaller tasks, feel free to add sub-tasks under the relevant section.

## MVP (v1) — In Progress

### Parser & AST
- [x] Number literal formats: hexadecimal (`0xFF`), binary (`0b1010`), scientific notation (`1.5e3`, `2.5E-4`)
- [x] Do-while loops (`do ... while condition end`)
- [x] Break/continue with levels (`break 2`, `continue 2`) - parsing only, runtime support TODO
- [x] Pattern matching with `match` expressions
- [x] Named function arguments (`add(a = 2, b = 3)`)

### Type System & Runtime
- [ ] Module system with `import` statement (local files, URLs, git repos) _(for v1 only local files)_
- [ ] Async/await support (`await` keyword, `Promise` type)
- [ ] User-defined types with `cast()` and `isInstanceOf()`
- [ ] Inheritance via `__parent`
- [ ] Traits (structural matching)
- [ ] Operator overloading (`__add`, `__sub`, `__mul`, etc.)
- [ ] Conversions (`__into` method)
- [ ] Built-in `Result(Ok, Err)` and `Option(T)` types
- [ ] Garbage collection hooks (`__gc` method)

- [x] Type Checker (MVP)
	- [x] `TcType` enum: Any, Unknown, Number, String, Boolean, Null, Array(T), Table, Function(args, ret)
	- [x] `TypeEnv` with scoped symbols (name → { ty, mutable, annotated })
	- [x] Check binary ops (`+ - * / %`) and comparisons (`== != < <= > >=`)
	- [x] Check logical ops (`and`, `or`, `not`) → Boolean
	- [x] Check functions: params/arity, return type vs implicit return
	- [x] Immutability enforcement (`let` vs `var`)

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
	- [x] `for` over arrays only (lower to while)

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
	- [x] Flags: `ast` (print AST), `check` (typecheck), `bytecode` (print bytecode), default (run)
	- [x] Pipeline: parse → typecheck → compile → run; concise error reporting

- [x] Integration Tests (end-to-end)
	- [x] `test_arith` — arithmetic, precedence, implicit program return
	- [x] `test_assign` — globals with compound assignment
	- [x] `test_cmp`, `test_logic_and`, `test_logic_or`, `test_not` — comparisons and logical ops
	- [x] arrays/tables — literals, member/index reads (`array_read`, `table_read`)
	- [x] `if_then_else`, `if_elif_else`, `if_local` — if/elif/else control flow with scoping
	- [x] `while_sum`, `do_while_sum` — while and do-while loops
	- [x] `fn_simple`, `fn_multiarg` — function definitions and calls
	- [x] `for_simple`, `for_sum` — for-loop lowering to while
	- [x] `array_write`, `table_write` — member/index assignment
	- [x] `mutable_var` — mutable variable reassignment with `var`

	## Infra/Housekeeping

	- [x] Test folder reorg: move parser fixtures to `tests/fixtures`
	- [x] Move ad-hoc samples from `tmp/` to `tests/runtime/`
	- [x] Runtime test harness with .ron expectations
	- [x] Add `bytecode` subcommand to CLI for debugging
	- [x] Negative test framework in `tests/should_fail/` with `.expect` files

## Critical Bugs to Fix (From Code Review - Nov 17, 2025)

### Priority 1: CRITICAL (Breaks Spec Compliance)

- [x] **Match Statements Not Executable** ✅ FIXED
  - Type checker implemented - checks match expression and patterns
  - Bytecode compiler implemented - compiles to property existence tests
  - VM execution working - tested with `match_simple.luma`
  - **Status**: Fully functional for simple identifier patterns

- [x] **Destructuring Declarations Silent Failure** ✅ FIXED
  - Bytecode compiler fully implemented for `Stmt::DestructuringVarDecl`
  - Array destructuring: `let [a, b] = [1, 2]` works correctly
  - Table destructuring: `let {name, age} = person` works correctly
  - Rest patterns: `let [head, ...tail] = array` works correctly with proper slicing
  - Works in both global and local scopes
  - **Status**: Fully functional for simple patterns
  - **Implementation**: Added `SliceArray` instruction to VM for efficient array slicing

- [x] **String Concatenation Type Error** ✅ FIXED - VM supports `String + String`, now type checker does too
  - Updated type checker to allow `String + String → String`
  - File: `typecheck/mod.rs:148-166`
  - Test: `tests/runtime/string_concat.luma` passes

### Priority 2: HIGH (Quality Issues)

- [x] **For Loop Pattern Error Handling** ✅ FIXED - `compile.rs:212`
  - Changed from silent `return` to panic with clear error message
  - Now: "Compiler error: Complex patterns in for loops are not yet supported"

- [x] **Missing "match" Keyword** ✅ FIXED - `lexer.rs:21`
  - Added "match" to keyword list for consistency

### Priority 3: MEDIUM (Can Defer but Document)

- [ ] **Break/Continue Levels Runtime Support**
  - Parser accepts `break 2`, `continue 3`
  - Compiler ignores level parameter (treats all as level 1)
  - **Fix Options**: (1) Implement nested loop tracking, OR (2) Emit error if level > 1

- [ ] **Clarify Spec: String Concatenation Behavior**
  - VM allows `String + Any` with debug formatting
  - Should spec allow only `String + String`, or broader coercion?
  - Update SPEC.md Section 9 with clear rules

- [ ] **Unused `annotated` Field in VarInfo**
  - `typecheck/mod.rs:82` has `#[allow(dead_code)]`
  - Could use for better error messages
  - **Fix**: Either use it or remove it

## Known Limitations (Current MVP)

- **Recursion**: Functions cannot recursively call themselves by name because the function value is assigned to the global *after* compilation. Requires either forward declarations or a two-pass approach.
- **Closures**: Captured variables from outer scopes not supported (deferred to v2).

## Deferred to v2

- [ ] Closures/upvalues (captured locals) + `CLOSURE`/upvalue handling
- [ ] `for` over tables and a general iterator protocol
- [ ] Destructuring in `for` (beyond simple identifiers)
- [ ] Richer typing: unions, flow-sensitive typing, typed tables/arrays, generics
- [ ] Better diagnostics with source spans (node IDs/spans throughout)
- [ ] Error recovery in parser/typechecker
- [ ] Standard library expansion (beyond `print`, `len`)
- [ ] JIT compiler or compact byte encoding for instructions

## Implementation Notes

- **String Interpolation**: Desugared to chained `Binary::Add` operations (no separate `Concat` AST node)
- **Logical vs Binary Operators**: Separate expression types because logical ops are non-overloadable and use short-circuit evaluation
- **Break/Continue Levels**: AST supports `break N` and `continue N` syntax, but runtime execution of multi-level breaks is not yet implemented

## Recent Changes (Session Notes)

### Session: November 17, 2025

1. **For-Loop Implementation** - Complete lowering to while loops:
   - Compiler: Lowers `for x in array do ... end` to a while loop with hidden `__iter` and `__i` locals
   - Uses `GetLen` instruction to get array length
   - Loop variable stored in local slot and updated each iteration
   - Tests: `for_simple.luma`, `for_sum.luma` 

2. **Additional Runtime Tests**:
   - `array_write.luma` - Array index assignment
   - `table_write.luma` - Table property assignment  
   - `mutable_var.luma` - Variable reassignment with `var`
   - `do_while_sum.luma` - Do-while loop execution

3. **CLI Enhancement**:
   - Added `bytecode` subcommand to print compiled bytecode for debugging
   - Subcommands: `ast`, `check`, `bytecode`, or default (run)

4. **Negative Test Framework**:
   - Created `tests/should_fail/` directory for expected-failure tests
   - Tests use `.expect` files to specify failure type (`parse`, `typecheck`, or `runtime`)
   - Added tests: `immutable_let`, `type_mismatch_add`, `undefined_var`, `wrong_arity`
   - Framework integrated into main test suite

5. **Type Checker**:
   - Immutability enforcement working correctly (prevents reassignment to `let` variables)
   - All binary ops, logical ops, and function checking implemented

### Completed Features (Previous Session)

1. **Number Literals** - Added support for:
   - Hexadecimal: `0xFF`, `0x1A3B` 
   - Binary: `0b1010`, `0b1101`
   - Scientific notation: `1.5e3`, `2.5E-4`
   
2. **Do-While Loops** - Full implementation:
   - AST: `Stmt::DoWhile { body, condition }`
   - Parser with proper `do ... while ... end` syntax
   - Typecheck support
   - Bytecode compiler support
   - Test fixtures and runtime tests
   
3. **Break/Continue with Levels**:
   - AST: `Stmt::Break(Option<u32>)` and `Stmt::Continue(Option<u32>)`
   - Parser accepts `break 2`, `continue 3` syntax
   - Typecheck updated
   - Note: Runtime support for multi-level loop exit deferred

4. **Code Cleanup**:
   - Removed `Expr::Concat` in favor of chained `Binary::Add` for string interpolation
   - Updated all test fixtures to reflect AST changes