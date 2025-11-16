# Luma Language Implementation - AI Agent Guide

## Project Overview
Luma is a typed scripting language implementation in Rust. Current phase: **Parser + AST** (no type checker, compiler, or VM yet). See `SPEC.md` for complete language specification.

## Architecture

**Pipeline (Current State):**
```
Source → [Lexer] → [Parser] → AST
```

**Future Pipeline:**
```
AST → [Type Checker] → Typed AST → [Compiler] → Bytecode → [VM/JIT] → Output
```

### Key Components

- **`src/ast.rs`**: Complete AST definitions using serde-serializable enums (`Expr`, `Stmt`, `Program`)
- **`src/parser/mod.rs`**: Main parser entry point using Chumsky combinator library with recursive parsers
- **`src/parser/lexer.rs`**: Whitespace/comment handling and keyword list
- **`src/parser/{expressions,statements,literals,operators,patterns,string}.rs`**: Modular parser components
- **`src/main.rs`**: CLI that parses `.luma` files and prints AST
- **`tests/parser_tests.rs`**: Fixture-based testing framework

## Critical Patterns

### Implicit Returns
**The language spec requires implicit returns** - the last expression in a block/function/program becomes a `Return` statement automatically:

```rust
// In parser/expressions.rs and parser/mod.rs:
.map(|(mut stmts, ret)| {
    if let Some(expr) = ret {
        stmts.push(Stmt::Return(expr));  // Explicit trailing expression
    } else if let Some(last) = stmts.pop() {
        match last {
            Stmt::ExprStmt(e) => stmts.push(Stmt::Return(e)),  // Convert trailing ExprStmt
            other => stmts.push(other),
        }
    }
    // ...
})
```

**Why:** Expressions like `true`, `1 + 2`, or function calls are parsed as `ExprStmt`. The last one must become `Return` for proper semantics (blocks are values, functions return last expression).

### Recursive Parser Pattern
Use `Recursive::declare()` for mutually-recursive expressions/statements:

```rust
let mut expr_ref = Recursive::declare();
let mut stmt_ref = Recursive::declare();
// ... build parsers that reference expr_ref and stmt_ref ...
expr_ref.define(logical_expr);  // Define after construction
stmt_ref.define(stmt);
```

### Operator Precedence
Built manually via chained `foldl` parsers (not Chumsky's pratt parser):
```
logical > comparison > addition > multiplication > postfix > unary > primary
```

### AST Serialization
Uses RON (Rusty Object Notation) via serde. Enum variants like `Expr::Function` serialize with named fields for readability.

## Development Workflows

### Testing
```bash
cargo test                    # Run all tests
cargo test --test parser_tests -- --nocapture  # See individual test names
```

**Fixture structure:** `tests/fixtures/{category}/{test_name}.{luma,ron}`
- `.luma` = source code
- `.ron` = expected AST serialization
- Test framework auto-discovers pairs recursively

**Adding tests:** Create both files in appropriate subdirectory. The test compares parsed AST against RON using `PartialEq`.

### Running the Parser CLI
```bash
cargo build
./target/debug/luma <file.luma>  # Prints debug AST
```

### Debugging Parse Failures
Chumsky errors show span/position. Common issues:
- Missing whitespace parser (`ws.clone()`) between tokens
- Keyword collision (check `KEYWORDS` in lexer.rs)
- Incorrect `Recursive` ordering (define after all references)

## Language-Specific Conventions

**Keywords:** All lowercase (`let`, `var`, `fn`, `do`, `end`, `if`, `elif`, `else`, `while`, `for`, `in`, `return`, `break`, `continue`, `and`, `or`, `not`, `true`, `false`, `null`)

**Blocks:** Always `do ... end` (no braces or significant whitespace)

**Types:** Prefix notation in AST (`Type::TypeIdent`), first-class values in language semantics

**Function arguments:** Stored as `Argument { name, type, default }` struct (not tuple) because serde needs named fields for clear serialization

## Known Limitations

- No type checking yet (AST accepts any types)
- No runtime/VM (parser-only implementation)
- No error recovery (first parse error stops)
- Windows line endings normalized in tests but not parser itself

## When Modifying Parser

1. Update `src/ast.rs` if adding new AST nodes
2. Add parser logic in appropriate `src/parser/*.rs` module
3. Wire into `src/parser/mod.rs` main combinator chain
4. Add test fixtures in `tests/fixtures/{category}/`
5. Ensure implicit return handling for expression contexts
6. Run `cargo test` to verify all fixtures pass
