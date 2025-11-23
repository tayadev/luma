# Literate Tests for Luma Language

Types of Tests Included:
- **ast_should_match**: Validates that the abstract syntax tree (AST) generated from the source code matches the expected structure.
- **should_error**: Checks that the provided code produces the expected compilation or runtime error.
- **should_run**: Ensures that the code executes successfully without errors.

Tests if constant variable declaration with string literal works correctly.
> ast_should_match
```luma
let x = "hello"
```
```ron
statements: [
    VarDecl(
    mutable: false,
    name: "x",
    value: Literal(String("hello")),
    ),
],
```

Makes sure that type checking fails when assigning a number to a string variable.
> should_error
```luma
let x: String = 42
```
```text
Type error: expected type 'String', found type 'Number' in variable declaration 'x'
```

Tests if pattern matching with literals works as expected.
> should_run
```luma
let x = "hello"
match x do
  "hello" do print("greeting") end
  "bye" do print("farewell") end
  _ do print("unknown") end
end
```
```text
greeting
```