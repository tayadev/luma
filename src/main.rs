
use std::fs;
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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ast { file }) => {
            let source = match fs::read_to_string(file) {
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
            let source = match fs::read_to_string(file) {
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
            let source = match fs::read_to_string(file) {
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
    let source = match fs::read_to_string(file) {
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
    let mut vm = luma::vm::VM::new(chunk);
    match vm.run() {
        Ok(val) => println!("{:?}", val),
        Err(e) => {
            eprintln!("Runtime error: {:?}", e);
            process::exit(1);
        }
    }
}
