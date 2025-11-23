# Luma TODOs

Ordered by near-term impact on correctness, developer ergonomics, and spec compliance.
When you complete an item, check it off. If a item is too big, break it down into smaller tasks and then work on those.
If you realize the priority of an item has changed, feel free to reorder the list.
If not specified otherwise, work on the tasks in the order they appear.

- [ ] Tests expansion: interpolation complex cases; match exhaustiveness success/fail; nested & renamed patterns; loop destructuring; named arg reorder; computed/quoted keys.
- [~] Optimize logical short-circuit codegen (avoid superfluous `Dup`). (Investigated: Dup is necessary with current JumpIfFalse instruction; would require new non-consuming jump instruction)
- [~] Ensure all overloadable operators (`%`, comparisons, unary `-`) attempt method fallback consistently.
	- Added runtime tests for `%`, `<`, and unary `-`. `%` and `<` pass via method fallback. Unary `-` works at runtime when not conflated with prior statement; add typechecker alignment so negation-overload doesn't require numeric RHS when lowered to `0 - x`.
	- Follow-up: teach typechecker that unary `-x` may be valid if `x` has `__neg` (or relax subtraction when RHS has `__neg`).
- [ ] Result/Option pattern sugar (auto-detect tag fields; validation & exhaustive errors).
- [ ] Documentation updates (`SPEC.md`, README) for new behaviors (match expr, interpolation, patterns, import semantics)
- [ ] Negative test suite build-out (`should_fail`): non-exhaustive match, unreachable pattern, invalid interpolation, duplicate named args, illegal table key forms.
	- Added: `should_fail/unary_neg_no_overload.{luma,expect}` expecting typecheck failure.
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