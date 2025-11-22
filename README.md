# Luma

Reference implementation of the Luma programming language.

## Download Nightly Builds


- [Linux x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-linux-x86_64)
- [MacOS aarch64](https://nightly.link/tayadev/luma/workflows/build/main/luma-macos-aarch64)
- [MacOS x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-macos-x86_64)
- [Windows x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-windows-x86_64.exe)

## Architecture

Source --[Lexer]--> Tokens --[Parser]--> AST --[Type Checker]--> Typed AST --[Compiler]--> Bytecode

Bytecode --[VM]--> Output
Bytecode --[JIT Compiler]--> Native Code --[Execution]--> Output