use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [--ast|--compile|--check] <file.luma>", args[0]);
        eprintln!("  (default: runs the file)");
        eprintln!("  --ast      : Print the AST");
        eprintln!("  --compile  : Compile to .lumab bytecode file");
        eprintln!("  --check    : Only typecheck without running");
        process::exit(1);
    }

    // Default mode is --run (execute the file)
    let mut mode = "--run".to_string();
    let mut filename_idx = 1;
    if args[1].starts_with("--") {
        mode = args[1].clone();
        filename_idx = 2;
    }
    if args.len() <= filename_idx {
        eprintln!("Missing input file. Usage: {} [--ast|--compile|--check] <file.luma>", args[0]);
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

    // Typecheck for all modes
    if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
        eprintln!("Typecheck failed:");
        for e in errs { 
            eprintln!("  - {}", e.message); 
        }
        process::exit(1);
    }

    match mode.as_str() {
        "--ast" => {
            println!("{:#?}", ast);
        }
        "--check" => {
            println!("Typecheck: OK");
        }
        "--compile" => {
            // Compile to bytecode
            let chunk = luma::bytecode::compile::compile_program(&ast);
            
            // Generate output filename (.lumab)
            let output_path = if filename.ends_with(".luma") {
                filename.replace(".luma", ".lumab")
            } else {
                format!("{}.lumab", filename)
            };
            
            // Serialize to RON format
            let serialized = match ron::ser::to_string_pretty(&chunk, ron::ser::PrettyConfig::default()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to serialize bytecode: {}", e);
                    process::exit(1);
                }
            };
            
            // Write to file
            if let Err(e) = fs::write(&output_path, serialized) {
                eprintln!("Failed to write bytecode file: {}", e);
                process::exit(1);
            }
            
            println!("Compiled to: {}", output_path);
        }
        "--run" => {
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
            eprintln!("Unknown mode '{}'. Use --ast, --compile, or --check.", mode);
            process::exit(1);
        }
    }
}