# Notes

- Lexer -> Parser -> AST -> IR -> Interpreter/VM
  - in the future we can slot in a JIT compiler as the last step

- stack-based VM
  - simpler to implement than a register-based VM
  - easier to generate code for from the AST/IR
  - good enough performance for a high-level language like Luma
  
- type checking during parsing/AST generation
  - allows for better error messages with line/column info
  - simpler implementation than a separate type checking pass
  - types are still enforced at runtime for dynamic features like tables and functions

- IDE integration
  - syntax highlighting
  - code completion
  - inline error messages


# TODOS

- [ ] write parser tests for all syntax features