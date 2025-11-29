# Command Line Interface (CLI)

```
$ luma
Luma programming language

Usage: luma [FILE] [COMMAND]

Commands:
  run      Execute a file with Luma
  repl     Start a REPL session with Luma
  lsp      Start the Language Server Protocol server
  check    Typecheck a Luma script without executing it
  compile  Compile a Luma script to a .lumac bytecode file
  upgrade  Upgrade to latest version of Luma

Arguments:
  [FILE]  The file to run (default if no subcommand)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Luma provides a command line interface (CLI) for executing scripts, starting a REPL session, typechecking code, compiling to bytecode, and more.

## Commands

### Upgrade

The `upgrade` command allows you to upgrade your Luma installation to the latest version.

You can specify a specific version with the `--version` flag:

```
$ luma upgrade --version 1.2.3
```

> If you set `--version` to `nightly`, it will install the latest nightly build.