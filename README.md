# Luma

Reference implementation of the Luma programming language.




Source --[Lexer]--> Tokens --[Parser]--> AST --[Type Checker]--> Typed AST --[Compiler]--> Bytecode

Bytecode --[VM]--> Output
Bytecode --[JIT Compiler]--> Native Code --[Execution]--> Output