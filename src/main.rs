use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [--ast|--check|--run] <file.luma>", args[0]);
        process::exit(1);
    }

    // Default mode is --ast for backward compatibility
    let mut mode = "--ast".to_string();
    let mut filename_idx = 1;
    if args[1].starts_with("--") {
        mode = args[1].clone();
        filename_idx = 2;
    }
    if args.len() <= filename_idx {
        eprintln!("Missing input file. Usage: {} [--ast|--check|--run] <file.luma>", args[0]);
        process::exit(1);
    }
    let filename = &args[filename_idx];

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
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

    match mode.as_str() {
        "--ast" => {
            println!("{:#?}", ast);
        }
        "--check" => {
            match luma::typecheck::typecheck_program(&ast) {
                Ok(()) => println!("Typecheck: OK"),
                Err(errs) => {
                    eprintln!("Typecheck failed:");
                    for e in errs { eprintln!("- {}", e.message); }
                    process::exit(1);
                }
            }
        }
        "--run" => {
            // Typecheck first
            if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
                eprintln!("Typecheck failed:");
                for e in errs { eprintln!("- {}", e.message); }
                process::exit(1);
            }
            // Compile and run
            let chunk = luma::bytecode::compile::compile_program(&ast);
            let mut vm = luma::vm::vm::VM::new(chunk);
            match vm.run() {
                Ok(val) => println!("{:?}", val),
                Err(e) => {
                    eprintln!("Runtime error: {:?}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown mode '{}'. Use --ast, --check, or --run.", mode);
            process::exit(1);
        }
    }
}