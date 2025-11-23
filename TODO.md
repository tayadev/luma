# Luma TODOs

Ordered by near-term impact on correctness, developer ergonomics, and spec compliance.
When you complete an item, check it off. If a item is too big, break it down into smaller tasks and then work on those.
If you realize the priority of an item has changed, feel free to reorder the list.

- [x] Rename Array to List in spec & codebase for clarity.
- [ ] Mutual recursion across separate function declarations (improve pre-declare mechanism beyond functions in same pass).
- [ ] Closures & upvalues completeness review (capture semantics, lifetime tests).
- [ ] Typechecker: concrete generics (`GenericType { name, args }`) and function type validation (params & return). 
- [ ] Structural typing improvements: table field presence + simple trait/tag matching.
- [ ] Refine equality/comparison diagnostics (value vs reference semantics, arrays/tables). 
- [ ] Pattern typing inference (bind variable types from pattern shape).
- [ ] Enforce named argument semantics (reordering + mixing positional/named, detect duplicates).
- [ ] Iterator protocol formalization (table iteration, custom iterables). 
- [ ] Loop pattern destructuring (`for [k,v] in table`, `for [item,index] in array.indexed()`). Also make sure `for` loops only accept iterable expressions, aka objects that implement the iterable trait.
- [ ] Range iteration helper `range(start,end)` & indexed iteration support (implement in prelude).
- [ ] Tests expansion: interpolation complex cases; match exhaustiveness success/fail; nested & renamed patterns; loop destructuring; named arg reorder; computed/quoted keys.
- [ ] Refactor jump patching (eliminate global `Jump(usize::MAX)` scans; track jumps explicitly per construct).
- [~] Optimize logical short-circuit codegen (avoid superfluous `Dup`). (Investigated: Dup is necessary with current JumpIfFalse instruction; would require new non-consuming jump instruction)
- [ ] Implement real `__into` dispatch (invoke method; fallback conversions) and tests.
- [ ] Ensure all overloadable operators (`%`, comparisons, unary `-`) attempt method fallback consistently.
- [ ] Result/Option pattern sugar (auto-detect tag fields; validation & exhaustive errors).
- [ ] Documentation updates (`SPEC.md`, README) for new behaviors (match expr, interpolation, patterns, import semantics)
- [ ] Negative test suite build-out (`should_fail`): non-exhaustive match, unreachable pattern, invalid interpolation, duplicate named args, illegal table key forms.
- [ ] Performance microbench harness (parse + compile + run) for regression tracking.
- [ ] Decide on async: implement minimal `await` + `Promise` placeholder or remove keyword until runtime ready.
- [ ] Import resolution future-proofing (remote sources hooks, lockfile placeholder alignment).
- [ ] GC hooks (`__gc`): implement or remove from spec pending design; integrate lifecycle tests.
- [ ] Error type hierarchy (Error, TypeError, IOError, etc.) surfaced via structured Result values.
- [ ] Clarify & finalize reserved keyword list in docs (remove unused, add missing if spec updated).
- [ ] Remote import caching & integrity verification pipeline (URL/git) groundwork.
- [ ] Async scheduler & promise resolution event loop prototype (post keyword decision).
- [ ] Rich diagnostics: node IDs, span propagation through AST → bytecode → runtime errors.
- [ ] Enhanced parser/typechecker error recovery strategies.
- [ ] Prelude trimming & boundary: core vs optional modules; curation of std extensions.
- [ ] Additional ergonomic helpers (range, indexed, collection utilities) in prelude.
- [ ] Performance: investigate JIT or more compact bytecode layout.
- [ ] Unions, refinement types, advanced generics (beyond MVP generics handling).
- [ ] Conversions chaining & failure categorization improvements.
- [ ] Enhanced error types mapping to runtime categories with standard formatting.
- [ ] Implement GC hook protocol tests & stress scenarios.
- [ ] JIT feasibility spike & roadmap doc.