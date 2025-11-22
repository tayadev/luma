
# Luma Project Checklist

## MVP Features
- [x] Module system (local `import()`)
- [x] Operator overloading
- [x] Core ADTs: `Result`, `Option`
- [x] `typeof()` intrinsic
- [x] Conversions (`__into`)
- [x] Prelude scope hygiene

## Deferred to V2
- [ ] GC hooks (`__gc`)
- [ ] Async/await & `Promise`
- [ ] Mutual recursion across separate function declarations
- [ ] Closures & upvalues (capture locals)
- [ ] Iterator protocol + `for` over tables
- [ ] Rich typing: unions, refinement, generics
- [ ] Diagnostics: node IDs, span propagation
- [ ] Error recovery (parser & typechecker)
- [ ] Prelude curation / extended std modules
- [ ] Performance: JIT compiler or compact bytecode

## Pending / Planned
- [ ] Prelude trimming & clear boundary between core vs optional packages
- [ ] Additional ergonomic helpers
