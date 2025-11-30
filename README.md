# Luma
[![Build](https://github.com/tayadev/luma/actions/workflows/build.yml/badge.svg)](https://github.com/tayadev/luma/actions/workflows/build.yml)
[![codecov](https://codecov.io/github/tayadev/luma/graph/badge.svg?token=1DOMJ3CFKH)](https://codecov.io/github/tayadev/luma)

Reference implementation of the Luma programming language.

## Quick Links

- **[Docs](https://tayadev.github.io/luma/)** 
- **[Language Specification](https://tayadev.github.io/luma/specification)**

## Installation

### Windows
```
powershell -c "irm https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.ps1 | iex"
```

### macOS / Linux
```
curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | sh
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

