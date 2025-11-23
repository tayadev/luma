# Luma TODOs

Ordered by near-term impact on correctness, developer ergonomics, and spec compliance.
When you complete an item, check it off. If an item is too big, break it down into smaller tasks first.
If you realize the priority of an item has changed, feel free to reorder the list.


- [ ] Rich diagnostics across pipeline
	- Propagate stable node IDs and precise spans AST → bytecode → runtime errors.
	- Improve error messages and locations in REPL and test outputs.
- [ ] Enhanced parser/typechecker error recovery strategies
	- Recover past first error where possible; improve chumsky recovery and reporting.
- [ ] Documentation sync pass (SPEC + README)
	- Verify coverage and examples for: match as expression, string interpolation, pattern forms, import semantics.
	- Clarify and finalize reserved keywords list to match lexer.
- [ ] Import resolution future-proofing
	- Hooks for remote sources; lockfile placeholder alignment for integrity.
- [ ] Decide on async direction
	- Implement minimal `await` + `Promise` placeholder in runtime, or hide/disable keyword until ready.
	- If kept: plan scheduler/event loop prototype and testing approach.
- [~] Optimize logical short-circuit codegen (avoid superfluous `Dup`)
	- Investigated: `Dup` currently required with `JumpIfFalse`; would need a non-consuming conditional jump instruction.
- [ ] Performance microbench harness
	- Small suite to run parse → compile → run for regression tracking (CI-friendly).
- [ ] Structured error hierarchy and mapping
	- Define `Error`, `TypeError`, `IOError`, etc., and map runtime categories with standard formatting.
- [ ] Prelude scope and ergonomics
	- Trim/scope prelude core vs optional; add helpers (range, indexed, collection utilities) as needed.
- [ ] Remote import caching & integrity verification groundwork
	- URL/Git fetching cache and integrity checks design sketch and minimal scaffolding.
- [ ] Async scheduler & event loop prototype (post decision)
- [ ] Performance: investigate JIT or more compact bytecode layout
- [ ] Unions, refinement types, advanced generics (beyond MVP)
- [ ] Conversions chaining & failure categorization improvements
- [ ] Implement GC hook (`__gc`) protocol and tests, or remove from spec pending design
- [ ] JIT feasibility spike & roadmap doc