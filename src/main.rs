use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    
    let filename = &args[1];
    
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };
    
    match luma::parser::parse(&source) {
        Ok(ast) => println!("{:#?}", ast),
        Err(errors) => {
            for error in errors {
                eprintln!("Parse error: {}", error);
            }
            process::exit(1);
        }
    }
}