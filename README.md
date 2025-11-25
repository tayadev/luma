# Luma

Reference implementation of the Luma programming language.

## Quick Links

- **[Language Specification (SPEC.md)](SPEC.md)** - Complete language reference with syntax, semantics, and examples

## Download Nightly Builds

- [Linux x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-linux-x86_64)
- [MacOS aarch64](https://nightly.link/tayadev/luma/workflows/build/main/luma-macos-aarch64)
- [MacOS x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-macos-x86_64)
- [Windows x86_64](https://nightly.link/tayadev/luma/workflows/build/main/luma-windows-x86_64.exe)

## Installation

### Windows
```
powershell -c "irm https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.ps1 | iex"
```

## CLI

Running `luma` or `luma --help` will print the following usage information:

```
Usage: luma <command> [...flags] [...args]

Commands:
  run       ./my-script.luma     Execute a file with Luma
  repl                           Start a REPL session with Luma
  check     ./my-script.luma     Typecheck a Luma script without executing it
  compile   ./my-script.luma     Compile a Luma script to a .lumac bytecode file

  upgrade                        Upgrade to latest version of Luma.

  <command> --help               Print help text for command.
```

> `run` can be omitted to execute a script directly: `luma ./my-script.luma` is equivalent to `luma run ./my-script.luma`