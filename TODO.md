
# Luma Unified Priority Backlog

Ordered by near-term impact on correctness, developer ergonomics, and spec compliance. Completed items retained at end.

1. [x] Full string interpolation `${expr}` (use general expression parser; remove manual scan); multiline preservation tests. (Implemented)
2. [x] Table literal enhancements: quoted keys and computed keys `[expr] = value`. (Implemented)
3. [x] Keyword audit & removal of `import` as keyword (treat as normal call); align reserved list with spec. (Completed: removed 'import' from KEYWORDS)
4. [x] Introduce `Expr::Match` (currently statement-only); allow match in expression contexts. (Implemented)
5. [ ] Exhaustiveness enforcement for match (wildcard `_` or all variant tags like `ok/err`, `some/none`).
6. [ ] Pattern system upgrade: nested array/table patterns, field renames `name: userName`, literal patterns, tag patterns.
7. [ ] Update compiler lowering for match to produce single value without temporary Null hacks.
8. [ ] Loop pattern destructuring (`for [k,v] in table`, `for [item,index] in array.indexed()`).
9. [ ] Range iteration helper `range(start,end)` & indexed iteration support.
10. [ ] Enforce named argument semantics (reordering + mixing positional/named, detect duplicates).
11. [ ] Tests expansion: interpolation complex cases; match exhaustiveness success/fail; nested & renamed patterns; loop destructuring; named arg reorder; computed/quoted keys.
12. [x] Centralize implicit return injection into a shared helper (block/function/if/match). (Completed: created utils.rs with apply_implicit_return helpers)
13. [ ] Refactor jump patching (eliminate global `Jump(usize::MAX)` scans; track jumps explicitly per construct).
14. [x] Remove unnecessary trailing `Const Null` from loop codegen (statement contexts). (Completed: removed from For loops; VM Halt handles empty stack)
15. [~] Optimize logical short-circuit codegen (avoid superfluous `Dup`). (Investigated: Dup is necessary with current JumpIfFalse instruction; would require new non-consuming jump instruction)
16. [ ] Typechecker: concrete generics (`GenericType { name, args }`) and function type validation (params & return). 
17. [ ] Structural typing improvements: table field presence + simple trait/tag matching.
18. [ ] Refine equality/comparison diagnostics (value vs reference semantics, arrays/tables). 
19. [ ] Pattern typing inference (bind variable types from pattern shape).
20. [ ] Implement real `__into` dispatch (invoke method; fallback conversions) and tests.
21. [ ] Ensure all overloadable operators (`%`, comparisons, unary `-`) attempt method fallback consistently.
22. [ ] Result/Option pattern sugar (auto-detect tag fields; validation & exhaustive errors).
23. [ ] Documentation updates (`SPEC.md`, README) for new behaviors (match expr, interpolation, patterns, import semantics) + changelog section.
24. [ ] Negative test suite build-out (`should_fail`): non-exhaustive match, unreachable pattern, invalid interpolation, duplicate named args, illegal table key forms.
25. [ ] Performance microbench harness (parse + compile + run) for regression tracking.
26. [ ] Decide on async: implement minimal `await` + `Promise` placeholder or remove keyword until runtime ready.
27. [ ] Import resolution future-proofing (remote sources hooks, lockfile placeholder alignment).
28. [ ] GC hooks (`__gc`): implement or remove from spec pending design; integrate lifecycle tests.
29. [ ] Error type hierarchy (Error, TypeError, IOError, etc.) surfaced via structured Result values.
30. [ ] Clarify & finalize reserved keyword list in docs (remove unused, add missing if spec updated).
31. [ ] Remote import caching & integrity verification pipeline (URL/git) groundwork.
32. [ ] Async scheduler & promise resolution event loop prototype (post keyword decision).
33. [ ] Rich diagnostics: node IDs, span propagation through AST → bytecode → runtime errors.
34. [ ] Enhanced parser/typechecker error recovery strategies.
35. [ ] Prelude trimming & boundary: core vs optional modules; curation of std extensions.
36. [ ] Additional ergonomic helpers (range, indexed, collection utilities) in prelude.
37. [ ] Performance: investigate JIT or more compact bytecode layout.
38. [ ] Mutual recursion across separate function declarations (improve pre-declare mechanism beyond functions in same pass).
39. [ ] Iterator protocol formalization (table iteration, custom iterables). 
40. [ ] Closures & upvalues completeness review (capture semantics, lifetime tests).
41. [ ] Unions, refinement types, advanced generics (beyond MVP generics handling).
42. [ ] Conversions chaining & failure categorization improvements.
43. [ ] Enhanced error types mapping to runtime categories with standard formatting.
44. [ ] Implement GC hook protocol tests & stress scenarios.
45. [ ] JIT feasibility spike & roadmap doc.

## Unsorted
- [ ] Rename Array to List in spec & codebase for clarity.

## Completed (Reference)
- [x] Module system (local `import()`)
- [x] Operator overloading (basic + method fallback)
- [x] Core ADTs: `Result`, `Option`
- [x] `typeof()` intrinsic
- [x] Conversions placeholder (`__into`) — to be completed (see item 20)
- [x] Prelude scope hygiene


