
use std::fs;
use std::io::{self, Read};
use std::process;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// The file to run (default if no subcommand)
    file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the parsed AST
    Ast {
        /// The file to parse
        file: String,
    },
    /// Typecheck the file
    Check {
        /// The file to typecheck
        file: String,
    },
    /// Print the compiled bytecode
    Bytecode {
        /// The file to compile
        file: String,
    },
    /// Start an interactive REPL session
    Repl,
}

/// Read source code from a file or stdin.
/// If `file` is "-", reads from stdin. Otherwise reads from the specified file.
fn read_source(file: &str) -> io::Result<String> {
    if file == "-" {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    } else {
        fs::read_to_string(file)
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ast { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in errors {
                        eprintln!("Parse error: {}", error);
                    }
                    process::exit(1);
                }
            };
            println!("{:#?}", ast);
        }
        Some(Commands::Check { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in errors {
                        eprintln!("Parse error: {}", error);
                    }
                    process::exit(1);
                }
            };
            match luma::typecheck::typecheck_program(&ast) {
                Ok(()) => println!("Typecheck: OK"),
                Err(errs) => {
                    eprintln!("Typecheck failed:");
                    for e in errs { eprintln!("- {}", e.message); }
                    process::exit(1);
                }
            }
        }
        Some(Commands::Bytecode { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in errors {
                        eprintln!("Parse error: {}", error);
                    }
                    process::exit(1);
                }
            };
            let chunk = luma::bytecode::compile::compile_program(&ast);
            println!("{:#?}", chunk);
        }
        Some(Commands::Repl) => {
            run_repl();
        }
        None => {
            // Default: run the file if provided
            let file = match &cli.file {
                Some(f) => f,
                None => {
                    eprintln!("No file provided. Usage: luma <file.luma> or luma <SUBCOMMAND> <file.luma>");
                    process::exit(1);
                }
            };
            run_file(file);
        }
    }

}

fn run_file(file: &str) {
    let source = match read_source(file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file, err);
            process::exit(1);
        }
    };
    let ast = match luma::parser::parse(&source) {
        Ok(ast) => ast,
        Err(errors) => {
            for error in errors {
                eprintln!("Parse error: {}", error);
            }
            process::exit(1);
        }
    };
    if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
        eprintln!("Typecheck failed:");
        for e in errs { eprintln!("- {}", e.message); }
        process::exit(1);
    }
    let chunk = luma::bytecode::compile::compile_program(&ast);
    
    // Get absolute path for the file
    // For stdin ("-"), don't try to resolve an absolute path
    let absolute_path = if file == "-" {
        Some("<stdin>".to_string())
    } else {
        match std::path::Path::new(file).canonicalize() {
            Ok(path) => Some(path.to_string_lossy().to_string()),
            Err(_) => {
                eprintln!("Warning: Could not resolve absolute path for '{}'", file);
                Some(file.to_string())
            }
        }
    };
    
    let mut vm = luma::vm::VM::new_with_file(chunk, absolute_path);
    vm.set_source(source.clone());
    match vm.run() {
        Ok(val) => println!("{}", val),
        Err(e) => {
            eprintln!("{}", e.format(Some(&source)));
            process::exit(1);
        }
    }
}

fn run_repl() {
    use std::io::{BufRead, Write};
    
    println!("Luma REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type expressions and press Enter. Use Ctrl+D (Unix) or Ctrl+Z (Windows) to exit.");
    println!();
    
    // Create an empty chunk to initialize the VM
    // The VM will be reused across evaluations to maintain state
    let empty_chunk = luma::bytecode::ir::Chunk::new_empty("<init>".to_string());
    let mut vm = luma::vm::VM::new_with_file(empty_chunk, Some("<repl>".to_string()));
    
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
                eprintln!("Error reading input: {}", e);
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
        let ast = match luma::parser::parse(&input) {
            Ok(ast) => ast,
            Err(errors) => {
                // Report parse errors
                for error in errors {
                    eprintln!("Parse error: {}", error);
                }
                continue;
            }
        };
        
        // Skip typechecking in REPL mode since each statement is evaluated independently
        // The typechecker doesn't have visibility into variables defined in previous REPL statements
        // Runtime errors will still be caught during execution
        
        // Compile the AST
        let chunk = luma::bytecode::compile::compile_program(&ast);
        
        // Set source for error reporting
        vm.set_source(input.clone());
        
        // Execute in the existing VM context
        match vm.eval(chunk) {
            Ok(val) => {
                println!("{}", val);
            }
            Err(e) => {
                eprintln!("{}", e.format(Some(&input)));
            }
        }
    }
}
