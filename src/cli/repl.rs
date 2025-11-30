//! `repl` subcommand handler

use crate::bytecode;
use crate::parser;
use crate::vm;
use std::io::{self, BufRead, Write};

/// Run the interactive REPL session
pub fn handle_repl() {
    println!("Luma REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type expressions and press Enter. Use Ctrl+D (Unix) or Ctrl+Z (Windows) to exit.");
    println!();

    // Create an empty chunk to initialize the VM
    // The VM will be reused across evaluations to maintain state
    let empty_chunk = bytecode::ir::Chunk::new_empty("<init>".to_string());
    let mut vm = vm::VM::new_with_file(empty_chunk, Some("<repl>".to_string()));

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        // Read input line by line, accumulating until we have a complete expression
        let mut input = String::new();

        match lines.next() {
            Some(Ok(line)) => {
                input.push_str(&line);
                input.push('\n');
            }
            Some(Err(e)) => {
                eprintln!("Error reading input: {e}");
                continue;
            }
            None => {
                // EOF reached
                println!();
                break;
            }
        }

        // Skip empty lines
        if input.trim().is_empty() {
            continue;
        }

        // Try to parse the input
        let ast = match parser::parse(&input, "<repl>") {
            Ok(ast) => ast,
            Err(errors) => {
                // Report parse errors
                for error in &errors {
                    eprintln!("{}", error.format(&input));
                }
                continue;
            }
        };

        // Skip typechecking in REPL mode since each statement is evaluated independently
        // The typechecker doesn't have visibility into variables defined in previous REPL statements
        // Runtime errors will still be caught during execution

        // Compile the AST using REPL mode (variables are globals)
        let chunk = bytecode::compile::compile_repl_program(&ast);

        // Set source for error reporting
        vm.set_source(input.clone());

        // Execute in the existing VM context
        match vm.eval(chunk) {
            Ok(val) => {
                println!("{val}");
            }
            Err(e) => {
                eprintln!("{}", e.format(Some(&input)));
            }
        }
    }
}
