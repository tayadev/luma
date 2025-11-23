# Luma Codebase Refactoring TODO

This document tracks refactoring work to improve code readability, maintainability, and adherence to best practices. Items are organized by priority and completion status.

---

## ✅ Completed Refactoring

### VM Interpreter - Complete Modularization (Sessions 1-2)
**Original size:** 1529 lines → **Final size:** 850 lines (44% reduction)

#### Session 1: Native Functions
- ✅ Created `src/vm/native/mod.rs` (16 lines)
- ✅ Created `src/vm/native/core.rs` (220 lines) - cast, isInstanceOf, into, typeof, iter
- ✅ Created `src/vm/native/io.rs` (128 lines) - print, write, read_file, write_file, file_exists, panic
- ✅ Created `src/vm/native/helpers.rs` (93 lines) - Result/Option helpers
- **Reduction:** 407 lines (27%)

#### Session 2: Operators & Modules
- ✅ Created `src/vm/operators.rs` (197 lines)
  - `has_method()`, `call_overload_method()`, `execute_binary_op()`, `execute_eq_op()`, `execute_cmp_op()`
- ✅ Created `src/vm/modules.rs` (141 lines)
  - `resolve_import_path()`, `load_module()`
- **Additional reduction:** 272 lines (24% from intermediate state)

#### Other Improvements
- ✅ Removed unused legacy error formatter from `src/parser/mod.rs`
- ✅ Clarified commented-out typecheck code in `src/vm/interpreter.rs`
- ✅ Fixed dead_code warning for `UpvalueInfo._name` field
- ✅ Consolidated implicit return logic in `src/parser/utils.rs` (-12 lines)
- ✅ Added comprehensive module docs to compile.rs, interpreter.rs, typecheck/mod.rs

### Session 3: Operator Overloading Test Coverage
**Status:** Complete  
**Impact:** Comprehensive validation of operators.rs module

**Tests Added:**
- ✅ `operator_overload_add` - Tests `__add` method for custom addition
- ✅ `operator_overload_sub` - Tests `__sub` method for custom subtraction
- ✅ `operator_overload_mul` - Tests `__mul` method for custom multiplication
- ✅ `operator_overload_div` - Tests `__div` method for custom division
- ✅ `operator_overload_eq` - Tests `__eq` method for custom equality
- ✅ `operator_overload_le` - Tests `__le` method for less-than-or-equal
- ✅ `operator_overload_gt` - Tests `__gt` method for greater-than

**Coverage:**
- 7 new test files created (14 files with .ron expectations)
- Joins 4 existing tests: lt, mod, ge, neg
- All 11 operator overload methods now tested
- Test suite: ✅ All 22 runtime tests passing

### Session 4: Module System Test Coverage
**Status:** Complete  
**Impact:** Comprehensive validation of modules.rs functionality

**Tests Added:**
- ✅ `import_function` - Tests importing module that exports a function
- ✅ `import_array` - Tests importing module that exports an array
- ✅ `circular_import` - Tests circular dependency detection (should_fail test)

**Coverage:**
- 3 new runtime test files + 1 should_fail test
- Validated 4 existing tests: import_number, import_cached, import_multiple, import_table, import_parent
- Features tested:
  - ✅ Simple imports (number, table, function, array exports)
  - ✅ Relative paths with `./` and `../`
  - ✅ Module caching (same module imported twice)
  - ✅ Multiple imports in one file
  - ✅ Circular dependency detection and error reporting
- Test suite: ✅ All 24 runtime tests + 17 should_fail tests passing

---

---

## 🚧 High Priority - Remaining Work

### 1. Split Large Files (>500 lines)

#### ⏸️ Split `src/bytecode/compile.rs` (1449 lines) - DEFERRED
**Identified Issues:**
- ~200 lines of duplicate code between `Stmt::Match` and `Expr::Match` 
- Helper functions are small (<10 lines) but could be extracted

**Why Deferred:**
- Pattern extraction requires careful refactoring to avoid breaking compilation
- Helper functions are tightly coupled and very small
- Would benefit from comprehensive test coverage before major refactoring
- Type system changes might affect pattern compilation approach

**Recommendation:** Defer until after type system improvements or when adding pattern compilation tests

**Original Plan:**
- Extract native function implementations into `src/bytecode/compile/native.rs`
- Extract helper functions (`block_leaves_value`, `merge_parent_fields`, etc.) into `src/bytecode/compile/helpers.rs`
- Extract pattern matching compilation into `src/bytecode/compile/patterns.rs`
- Keep main `Compiler` struct and core compilation logic in `compile.rs`

#### ⏸️ Split `src/typecheck/mod.rs` (1345 lines) - ANALYZED, NOT STARTED
**Identified Structure:**
1. **Patterns module potential:**
   - `check_pattern()` - ~120 lines
   - `KNOWN_TAG_PATTERNS` constant
   - Pattern validation logic

2. **Exhaustiveness module potential:**
   - `check_unreachable_patterns()` - ~30 lines
   - `check_match_exhaustiveness()` - ~70 lines

3. **Types module potential:**
   - `TcType` enum
   - `type_from_ast()` - ~40 lines
   - `is_compatible()` - large, complex
   - `has_operator_method()` - ~10 lines

**Why Not Started:**
- `check_pattern()` is tightly integrated with `TypeEnv` (uses self.declare, self.error)
- Splitting would require passing `TypeEnv` or creating new error collection patterns
- Risk of breaking gradual type system behavior
- Requires deep understanding of type checking invariants

**Recommendation:** Create a separate architectural refactoring task with comprehensive test coverage

**Original Plan:**
- Extract pattern checking logic into `src/typecheck/patterns.rs`
- Extract exhaustiveness checking into `src/typecheck/exhaustiveness.rs`
- Extract type compatibility and inference into `src/typecheck/types.rs`
- Keep main `TypeEnv` and `typecheck_program` in `mod.rs`

#### 📋 Split `src/parser/mod.rs` (384 lines) - NOT STARTED
- Extract type parser logic into `src/parser/types.rs`
- Extract error formatting into `src/parser/errors.rs`
- Keep main parser combinator construction in `mod.rs`
- **Rationale**: Type parsing and error formatting are distinct concerns from the main parser

#### 📋 Split `src/ast.rs` (423 lines) - NOT STARTED
- Extract type definitions into `src/ast/types.rs`
- Extract pattern definitions into `src/ast/patterns.rs`
- Extract span/location utilities into `src/ast/span.rs`
- Keep core `Expr`, `Stmt`, `Program` in `mod.rs`
- **Rationale**: AST definitions are logically groupable into types, patterns, and core expressions

### 2. Improve Error Handling

#### ⏸️ Replace `panic!` with proper error returns in compiler - DEFERRED
**Scope:** 16 panic! calls in `src/bytecode/compile.rs`

**Why Deferred:**
- Would require changing `compile_program()` signature from `Chunk` to `Result<Chunk, CompileError>`
- Affects 8+ call sites across main.rs, interpreter.rs, modules.rs, tests
- All panic! calls have clear error messages starting with "Compiler error:"
- Primarily for unimplemented features, not incorrect code
- Cost/benefit ratio doesn't justify immediate action

**Recommendation:** Address when implementing error recovery or compilation diagnostics

**Original Plan:**
- Create `CompileError` enum with variants for different error types
- Update all panic! calls to return proper errors
- Lines to fix: 364, 465, 537, 681, 696, 805, 810, 835, 840, 1025, 1035, 1037, 1048, 1056, 1067, 1226

#### 📋 Add context to VM runtime errors
- Many `VmError::runtime(...)` calls have generic messages
- Add source location information (span) where possible
- Use the `VmError::with_location` constructor more consistently
- **Rationale**: Better debugging experience for users

#### 📋 Improve diagnostic error messages
- Many type errors use debug formatting (`{:?}`) instead of user-friendly messages
- Add Display impl for `TcType` and use it in error messages
- **Rationale**: Better user experience; debug output is not user-facing

### 3. Consolidate Duplicate Logic

#### 📋 Consolidate pattern matching compilation
- Pattern matching logic duplicated between `Stmt::Match` and `Expr::Match` in `src/bytecode/compile.rs`
- Extract shared logic into helper function `compile_match_arms`
- **Rationale**: ~200 lines of nearly identical code

#### 📋 Consolidate Result/Option helper creation
- `make_result_ok` and `make_result_err` are in `vm/native/helpers.rs`
- Consider creating a standard library prelude module for these
- **Rationale**: Better organization and discoverability

### 4. Code Style & Consistency

#### 📋 Consistent naming for boolean functions
- `block_leaves_value` (predicate) vs `has_method` (predicate with "has_" prefix)
- Standardize on `is_*` or `has_*` prefix for predicates
- **Rationale**: Consistency aids readability

#### 📋 Extract magic numbers into named constants
- File descriptors: `1` (STDOUT) and `2` (STDERR) used directly in `native_write`
- Already defined as globals but not used in implementation
- **Rationale**: Self-documenting code; easier to maintain

---

---

## 📋 Medium Priority - Architecture Improvements

### 6. Improve Module Organization

- [ ] **Finalize bytecode module structure**
  ```
  src/bytecode/
    mod.rs          # Public exports
    ir.rs           # Instruction definitions (keep as is)
    compile.rs      # Main compiler (when split is done)
  ```

- [ ] **Move parser submodules into subdirectory** (when parser is split)
  ```
  src/parser/
    mod.rs          # Main parser
    expressions.rs  # (keep as is)
    statements.rs   # (keep as is)
    types.rs        # NEW: Type parsing
    errors.rs       # NEW: Error formatting
    utils/
      mod.rs
      implicit_returns.rs
      lexer.rs, operators.rs, literals.rs, patterns.rs, string.rs
  ```

### 7. Reduce Coupling

- [ ] **Extract upvalue management into separate module**
  - Upvalue logic scattered across `Compiler` in `compile.rs`
  - Create `src/bytecode/compile/upvalues.rs` with dedicated struct
  - **Rationale**: Single responsibility principle

- [ ] **Extract call frame management from VM**
  - `CallFrame` is tightly coupled to VM implementation
  - Consider separate `src/vm/callframe.rs` with clear interface
  - **Rationale**: Easier to test and reason about

- [ ] **Separate type inference from type checking**
  - `TypeEnv` handles both checking and inference
  - Consider splitting into `TypeChecker` and `TypeInferencer`
  - **Rationale**: Clearer responsibilities; easier to test independently

### 8. Improve Type Safety

- [ ] **Use newtypes for stack slots, instruction pointers, etc.**
  ```rust
  struct StackSlot(usize);
  struct InstructionPointer(usize);
  struct ConstantIndex(usize);
  ```
  - Replace raw `usize` values in VM and compiler
  - **Rationale**: Type safety prevents mixing up indices

- [ ] **Replace HashMap<String, Value> with dedicated Table type**
  - Create `struct LumaTable` with builder pattern
  - Add methods for common operations
  - **Rationale**: Better encapsulation; easier to add features like property descriptors

- [ ] **Use enum for operator overload method names**
  ```rust
  enum OperatorMethod {
      Add, Sub, Mul, Div, Mod, Neg,
      Eq, Lt, Le, Gt, Ge,
  }
  ```
  - Replace string literals like `"__add"`, `"__sub"`
  - **Rationale**: Type safety; prevents typos; easier to refactor

### 9. Performance Opportunities (Document for Future)

- [ ] **Document opportunities for instruction fusion**
  - `GetLocal` + `GetProp` → `GetLocalProp`
  - `Const` + `SetGlobal` → `ConstSetGlobal`
  - Add comments in `ir.rs` noting these potential optimizations
  - **Rationale**: Future JIT work will benefit from documented patterns

- [ ] **Document stack layout invariants**
  - Add comments documenting expected stack layout for each instruction
  - Create visualization in `src/vm/README.md`
  - **Rationale**: Easier to reason about correctness and optimize

- [ ] **Consider using SmallVec for local scopes**
  - Most scopes have <8 locals; SmallVec would avoid heap allocation
  - Note in comments for future optimization
  - **Rationale**: Micro-optimization opportunity

---

## 📝 Low Priority - Nice to Have

### 10. Testing Infrastructure

- [x] **Add tests for operator overloading** ✅ COMPLETE (Session 3)
  - Created 7 new test files: add, sub, mul, div, eq, le, gt
  - Joins existing tests: lt, mod, ge, neg
  - All 11 operator overload methods now tested
  - All runtime tests passing (22 total)
  
- [ ] **Add edge case tests for operator overloading**
  - Test error cases when methods have wrong arity
  - Test operator chaining behavior
  - Test operator precedence with overloads
  - **Priority**: Medium

- [x] **Add tests for module system** ✅ COMPLETE (Session 4)
  - Created 3 new runtime tests: import_function, import_array, circular_import (should_fail)
  - Validated existing tests: import_number, import_cached, import_multiple, import_table, import_parent
  - Coverage: relative paths, parent directory (..), caching, circular dependency detection
  - Function/array/table exports all tested
  - All module tests passing (7 runtime + 1 should_fail)

- [ ] **Add edge case tests for module system**
  - Test module parse errors
  - Test module typecheck errors  
  - Test missing file errors
  - **Priority**: Low

- [ ] **Add unit tests for helper functions**
  - `apply_implicit_return` in `src/parser/utils.rs`
  - `block_leaves_value` in `src/bytecode/compile.rs`
  - **Rationale**: Catch regressions early

- [ ] **Add property-based tests for value equality**
  - `Value::PartialEq` has complex logic for Rc comparison
  - Use proptest or similar to verify reflexivity, symmetry, transitivity
  - **Rationale**: Correctness guarantee for equality

- [ ] **Add benchmarks for hot paths**
  - Instruction dispatch loop
  - Local variable lookup
  - Pattern matching compilation
  - **Rationale**: Track performance regressions

### 11. Documentation

- [ ] **Document VM and CallFrame fields**
  - Add inline comments explaining purpose of each field
  - Document relationships between fields
  - **Priority**: Recommended for maintainability

- [ ] **Document compiler passes in ARCHITECTURE.md**
  - Pre-declaration pass
  - Main compilation pass
  - Why this is needed (mutual recursion)
  - **Rationale**: Critical for understanding compiler behavior

- [ ] **Document VM stack discipline in ARCHITECTURE.md**
  - What each instruction expects/produces on stack
  - How calls manage stack frames
  - Upvalue capture mechanism
  - **Rationale**: Essential for contributors

- [ ] **Add examples to docstrings**
  - `apply_implicit_return` should show before/after
  - `check_pattern` should show binding examples
  - **Rationale**: Faster onboarding for contributors

### 12. Cleanup

- [ ] **Review all in-code comments marked TODO, FIXME, XXX**
  - Search codebase for inline TODOs and either address or document
  - **Rationale**: Ensure no forgotten tasks

- [ ] **Remove obsolete comments**
  - "MVP:" prefixes in comments - we're past MVP
  - Comments describing temporary workarounds that are now permanent
  - **Rationale**: Reduce noise in code

---

## 📊 Summary Statistics

### Completed Work
- **Files split**: 1 major file (vm/interpreter.rs)
- **New modules created**: 6 (native/, operators.rs, modules.rs)
- **Lines reduced**: 679 lines from interpreter.rs (44% reduction)
- **Dead code removed**: 3 instances
- **Duplicate logic consolidated**: 1 instance (~12 lines)
- **Module docs added**: 3 major modules
- **Test status**: ✅ All 23 tests passing

### Remaining Work
- **Files to split**: 4 major files (compile.rs, typecheck/mod.rs, parser/mod.rs, ast.rs)
- **Duplicate logic to consolidate**: ~200 lines (match compilation)
- **Panic calls to address**: 16 in compiler (deferred)
- **Tests to add**: Operator overloading, module system, helpers

### Code Quality Metrics

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| vm/interpreter.rs | 1529 lines | 850 lines | -679 (-44%) |
| VM module count | 1 | 6 | +5 |
| Average VM module size | 1529 lines | ~200 lines | -87% |
| Module documentation | 0 major modules | 6 modules | +6 |

---

## 🎯 Guiding Principles

1. **Preserve functionality**: All refactoring should be behavior-preserving
2. **One change at a time**: Split, move, then cleanup - don't mix concerns
3. **Test after each step**: Run full test suite after each refactoring step
4. **Document as you go**: Add module docs when creating new files
5. **No warnings**: Fix all warnings introduced by refactoring immediately
6. **Know when to defer**: Some refactorings require broader architectural changes

## 🛠️ Implementation Notes

- Use `cargo test` after each change to ensure no regressions
- Use `cargo clippy` to catch common issues
- Use `cargo fmt` to maintain consistent style
- Consider using `cargo-modules` to visualize module structure
- Update ARCHITECTURE.md with new module organization

---

## 📚 Lessons Learned (Sessions 1-2)

1. **Small, focused extractions work best** - Operator and module extractions were successful because they had clear boundaries
2. **Test coverage is crucial** - All changes verified by existing test suite
3. **Public fields enable refactoring** - Making VM/CallFrame fields public allowed module extraction without getter/setter overhead
4. **Documentation pays off** - Module-level docs clarify intent and usage patterns
5. **Know when to defer** - Some refactorings (panic! replacement, large file splits) require broader architectural changes

---

*Generated by analyzing the Luma codebase for refactoring opportunities*  
*Sessions completed: November 23, 2024*  
*Status: VM refactoring complete, other major files analyzed*


### 6. Improve Module Organization

- [ ] **Create proper module structure for bytecode**
  ```
  src/bytecode/
    mod.rs          # Public exports
    ir.rs           # Instruction definitions (keep as is)
    compile/
      mod.rs        # Main compiler
      patterns.rs   # Pattern compilation
      native.rs     # Native function setup
      helpers.rs    # Utility functions
  ```
  - **Rationale**: Better separation of concerns

- [ ] **Create proper module structure for VM**
  ```
  src/vm/
    mod.rs          # Public exports
    interpreter.rs  # Main VM loop
    value.rs        # Value definitions (keep as is)
    operators.rs    # Operator overloading
    modules.rs      # Import/module system
    native/
      mod.rs        # Native function registration
      core.rs       # Core native functions
      io.rs         # I/O functions
      helpers.rs    # Helper utilities
  ```
  - **Rationale**: Clear module boundaries make code easier to navigate

- [ ] **Move parser submodules into subdirectory**
  ```
  src/parser/
    mod.rs          # Main parser (keep as is)
    expressions.rs  # (keep as is)
    statements.rs   # (keep as is)
    types.rs        # NEW: Type parsing (extracted from mod.rs)
    errors.rs       # NEW: Error formatting (extracted from mod.rs)
    utils/
      mod.rs
      implicit_returns.rs  # Rename from utils.rs
      lexer.rs      # (move here)
      operators.rs  # (move here)
      literals.rs   # (move here)
      patterns.rs   # (move here)
      string.rs     # (move here)
  ```
  - **Rationale**: Clearer hierarchy; utilities separated from core parsing

### 7. Reduce Coupling

- [ ] **Extract upvalue management into separate module**
  - Upvalue logic scattered across `Compiler` in `compile.rs`
  - Create `src/bytecode/compile/upvalues.rs` with dedicated struct
  - **Rationale**: Single responsibility principle

- [ ] **Extract call frame management from VM**
  - `CallFrame` is tightly coupled to VM implementation
  - Consider separate `src/vm/callframe.rs` with clear interface
  - **Rationale**: Easier to test and reason about

- [ ] **Separate type inference from type checking**
  - `TypeEnv` handles both checking and inference
  - Consider splitting into `TypeChecker` and `TypeInferencer`
  - **Rationale**: Clearer responsibilities; easier to test independently

### 8. Improve Type Safety

- [ ] **Use newtypes for stack slots, instruction pointers, etc.**
  ```rust
  struct StackSlot(usize);
  struct InstructionPointer(usize);
  struct ConstantIndex(usize);
  ```
  - Replace raw `usize` values in VM and compiler
  - **Rationale**: Type safety prevents mixing up indices

- [ ] **Replace HashMap<String, Value> with dedicated Table type**
  - Create `struct LumaTable` with builder pattern
  - Add methods for common operations
  - **Rationale**: Better encapsulation; easier to add features like property descriptors

- [ ] **Use enum for operator overload method names**
  ```rust
  enum OperatorMethod {
      Add, Sub, Mul, Div, Mod, Neg,
      Eq, Lt, Le, Gt, Ge,
  }
  ```
  - Replace string literals like `"__add"`, `"__sub"`
  - **Rationale**: Type safety; prevents typos; easier to refactor

### 9. Performance Opportunities (Document for Future)

- [ ] **Document opportunities for instruction fusion**
  - `GetLocal` + `GetProp` → `GetLocalProp`
  - `Const` + `SetGlobal` → `ConstSetGlobal`
  - Add comments in `ir.rs` noting these potential optimizations
  - **Rationale**: Future JIT work will benefit from documented patterns

- [ ] **Document stack layout invariants**
  - Add comments documenting expected stack layout for each instruction
  - Create visualization in `src/vm/README.md`
  - **Rationale**: Easier to reason about correctness and optimize

- [ ] **Consider using SmallVec for local scopes**
  - Most scopes have <8 locals; SmallVec would avoid heap allocation
  - Note in comments for future optimization
  - **Rationale**: Micro-optimization opportunity

## Low Priority - Nice to Have

### 10. Testing Infrastructure

- [ ] **Add unit tests for helper functions**
  - `apply_implicit_return` in `src/parser/utils.rs`
  - `block_leaves_value` in `src/bytecode/compile.rs`
  - **Rationale**: Catch regressions early

- [ ] **Add property-based tests for value equality**
  - `Value::PartialEq` has complex logic for Rc comparison
  - Use proptest or similar to verify reflexivity, symmetry, transitivity
  - **Rationale**: Correctness guarantee for equality

- [ ] **Add benchmarks for hot paths**
  - Instruction dispatch loop
  - Local variable lookup
  - Pattern matching compilation
  - **Rationale**: Track performance regressions

### 11. Documentation

- [ ] **Document compiler passes in ARCHITECTURE.md**
  - Pre-declaration pass (line 32-45 in compile.rs)
  - Main compilation pass (line 48-51)
  - Why this is needed (mutual recursion)
  - **Rationale**: Critical for understanding compiler behavior

- [ ] **Document VM stack discipline in ARCHITECTURE.md**
  - What each instruction expects/produces on stack
  - How calls manage stack frames
  - Upvalue capture mechanism
  - **Rationale**: Essential for contributors

- [ ] **Add examples to docstrings**
  - `apply_implicit_return` should show before/after
  - `check_pattern` should show binding examples
  - **Rationale**: Faster onboarding for contributors

### 12. Cleanup TODOs

- [ ] **Review all in-code comments marked TODO, FIXME, XXX**
  - Only found references in git history and external files
  - Search codebase for inline TODOs and either address or document
  - **Rationale**: Ensure no forgotten tasks

- [ ] **Remove obsolete comments**
  - "MVP:" prefixes in comments - we're past MVP
  - Comments describing temporary workarounds that are now permanent
  - **Rationale**: Reduce noise in code

## Specific File Refactoring Plans

### src/bytecode/compile.rs Refactoring Plan

**Phase 1: Extract Native Functions**
1. Create `src/bytecode/compile/native.rs`
2. Move all `native_*` functions (lines 1183-1430)
3. Move helper functions: `get_type_map`, `has_required_fields`, `is_castable`, `merge_parent_fields`

**Phase 2: Extract Pattern Compilation**
1. Create `src/bytecode/compile/patterns.rs`
2. Move pattern matching compilation helpers
3. Extract common logic between `Stmt::Match` and `Expr::Match`

**Phase 3: Extract Utilities**
1. Create `src/bytecode/compile/helpers.rs`
2. Move `block_leaves_value`, `push_const`
3. Move `LoopContext`, `UpvalueInfo` definitions

**Phase 4: Cleanup**
1. Update imports in `mod.rs`
2. Ensure all functions have appropriate visibility
3. Add module-level documentation

### src/vm/interpreter.rs Refactoring Plan

**Phase 1: Extract Native Functions**
1. Create `src/vm/native/mod.rs`, `core.rs`, `io.rs`, `helpers.rs`
2. Move all native function implementations
3. Keep registration logic in interpreter.rs initially

**Phase 2: Extract Operator Overloading**
1. Create `src/vm/operators.rs`
2. Move `has_method`, `call_overload_method`, `execute_*_op` methods
3. Make them standalone functions or associated functions on VM

**Phase 3: Extract Module System**
1. Create `src/vm/modules.rs`
2. Move `resolve_import_path`, `load_module`, module caching logic
3. Consider separate struct for module management

**Phase 4: Cleanup Main Loop**
1. Simplify instruction dispatch using extracted functions
2. Add better comments for complex instructions
3. Ensure error handling is consistent

### src/typecheck/mod.rs Refactoring Plan

**Phase 1: Extract Pattern Logic**
1. Create `src/typecheck/patterns.rs`
2. Move `check_pattern`, pattern-related helpers
3. Move `KNOWN_TAG_PATTERNS` constant

**Phase 2: Extract Exhaustiveness Checking**
1. Create `src/typecheck/exhaustiveness.rs`
2. Move `check_unreachable_patterns`, `check_match_exhaustiveness`

**Phase 3: Extract Type System**
1. Create `src/typecheck/types.rs`
2. Move `TcType` enum, `is_compatible`, `type_from_ast`
3. Move `has_operator_method`

**Phase 4: Cleanup**
1. Keep `TypeEnv`, `typecheck_program` in `mod.rs`
2. Simplify public API
3. Add comprehensive module docs

## Summary Statistics

- **Total files to split**: 5 major files (compile.rs, interpreter.rs, typecheck/mod.rs, parser/mod.rs, ast.rs)
- **Dead code to remove**: 3+ instances
- **Duplicate logic to consolidate**: 3+ major instances
- **Panic calls to fix**: 12+ in compiler
- **Missing documentation**: 3+ major modules

## Guiding Principles

1. **Preserve functionality**: All refactoring should be behavior-preserving
2. **One change at a time**: Split, move, then cleanup - don't mix concerns
3. **Test after each step**: Run full test suite after each refactoring step
4. **Document as you go**: Add module docs when creating new files
5. **No warnings**: Fix all warnings introduced by refactoring immediately

## Implementation Notes

- Use `cargo test` after each change to ensure no regressions
- Use `cargo clippy` to catch common issues
- Use `cargo fmt` to maintain consistent style
- Consider using `cargo-modules` to visualize module structure
- Update ARCHITECTURE.md with new module organization

---

*Generated by analyzing the Luma codebase for refactoring opportunities*
*Last updated: 2024-11-23*
