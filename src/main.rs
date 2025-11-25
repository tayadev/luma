use clap::{CommandFactory, Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use std::process;

/// Get the version string including git revision
fn version() -> &'static str {
    concat!(env!("CARGO_PKG_VERSION"), " (git:", env!("GIT_HASH"), ")")
}

#[derive(Parser)]
#[command(
    author,
    version = version(),
    about = "Luma programming language",
    long_about = None,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// The file to run (default if no subcommand)
    file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a file with Luma
    Run {
        /// The file to execute
        file: String,
    },
    /// Start a REPL session with Luma
    Repl,
    /// Typecheck a Luma script without executing it
    Check {
        /// The file to typecheck
        file: String,
    },
    /// Compile a Luma script to a .lumac bytecode file
    Compile {
        /// The file to compile
        file: String,
        /// Output file (defaults to input.lumac)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Upgrade to latest version of Luma
    Upgrade {
        /// Specific version to upgrade to (e.g., "0.2.0" or "v0.2.0")
        #[arg(long)]
        version: Option<String>,
    },
    /// Print the parsed AST (debug)
    #[command(hide = true)]
    Ast {
        /// The file to parse
        file: String,
    },
    /// Print the compiled bytecode (debug)
    #[command(hide = true)]
    Bytecode {
        /// The file to compile
        file: String,
    },
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
        Some(Commands::Run { file }) => {
            run_file(file);
        }
        Some(Commands::Repl) => {
            run_repl();
        }
        Some(Commands::Check { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source, file) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in &errors {
                        eprintln!("{}", error.format(&source));
                    }
                    process::exit(1);
                }
            };
            match luma::typecheck::typecheck_program(&ast) {
                Ok(()) => println!("Typecheck: OK"),
                Err(errs) => {
                    eprintln!("Typecheck failed:");
                    for e in errs {
                        eprintln!("- {}", e.message);
                    }
                    process::exit(1);
                }
            }
        }
        Some(Commands::Compile { file, output }) => {
            compile_file(file, output.as_deref());
        }
        Some(Commands::Upgrade { version }) => {
            upgrade_luma(version.as_deref());
        }
        Some(Commands::Ast { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source, file) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in &errors {
                        eprintln!("{}", error.format(&source));
                    }
                    process::exit(1);
                }
            };
            println!("{:#?}", ast);
        }
        Some(Commands::Bytecode { file }) => {
            let source = match read_source(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", file, err);
                    process::exit(1);
                }
            };
            let ast = match luma::parser::parse(&source, file) {
                Ok(ast) => ast,
                Err(errors) => {
                    for error in &errors {
                        eprintln!("{}", error.format(&source));
                    }
                    process::exit(1);
                }
            };
            let chunk = luma::bytecode::compile::compile_program(&ast);
            println!("{:#?}", chunk);
        }
        None => {
            // Default: run the file if provided, otherwise print help
            let file = match &cli.file {
                Some(f) => f,
                None => {
                    // No file and no subcommand - print help
                    Cli::command().print_help().unwrap();
                    println!();
                    process::exit(0);
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
    let ast = match luma::parser::parse(&source, file) {
        Ok(ast) => ast,
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error.format(&source));
            }
            process::exit(1);
        }
    };
    if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
        eprintln!("Typecheck failed:");
        for e in errs {
            eprintln!("- {}", e.message);
        }
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

fn compile_file(file: &str, output: Option<&str>) {
    let source = match read_source(file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file, err);
            process::exit(1);
        }
    };
    let ast = match luma::parser::parse(&source, file) {
        Ok(ast) => ast,
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error.format(&source));
            }
            process::exit(1);
        }
    };
    if let Err(errs) = luma::typecheck::typecheck_program(&ast) {
        eprintln!("Typecheck failed:");
        for e in errs {
            eprintln!("- {}", e.message);
        }
        process::exit(1);
    }
    let chunk = luma::bytecode::compile::compile_program(&ast);

    // Determine output filename
    let output_file = match output {
        Some(o) => o.to_string(),
        None => {
            if file == "-" {
                eprintln!("Error: Cannot compile from stdin without --output flag");
                process::exit(1);
            }
            // Replace extension with .lumac
            let path = std::path::Path::new(file);
            path.with_extension("lumac").to_string_lossy().to_string()
        }
    };

    // Serialize the bytecode chunk
    let serialized = match ron::ser::to_string_pretty(&chunk, ron::ser::PrettyConfig::default()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error serializing bytecode: {}", e);
            process::exit(1);
        }
    };

    // Write to file
    if let Err(e) = fs::write(&output_file, serialized) {
        eprintln!("Error writing to '{}': {}", output_file, e);
        process::exit(1);
    }

    println!("Compiled '{}' to '{}'", file, output_file);
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
        let ast = match luma::parser::parse(&input, "<repl>") {
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

#[cfg(target_os = "windows")]
fn upgrade_luma(version: Option<&str>) {
    use std::env;
    use std::path::PathBuf;

    println!("Upgrading Luma...");

    // Determine installation directory
    let luma_root = if let Ok(install_dir) = env::var("LUMA_INSTALL") {
        PathBuf::from(install_dir)
    } else if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join(".luma")
    } else {
        eprintln!("Error: Could not determine installation directory");
        process::exit(1);
    };

    let luma_bin = luma_root.join("bin");
    let luma_exe = luma_bin.join("luma.exe");

    // Get current executable path to verify we're upgrading the right installation
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Could not determine current executable path: {}", e);
            process::exit(1);
        }
    };

    // Warn if current exe is not in the expected location
    if current_exe != luma_exe {
        eprintln!(
            "Warning: Current executable is at {}, but will upgrade {}",
            current_exe.display(),
            luma_exe.display()
        );
    }

    // Determine if we're upgrading to nightly or a release
    let is_nightly = version == Some("nightly");

    let zip_bytes = if is_nightly {
        download_nightly_artifact()
    } else {
        download_release_artifact(version)
    };

    println!("Download complete. Extracting...");

    // Extract the zip file
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!("Error: Failed to read zip archive: {}", e);
            process::exit(1);
        }
    };

    // Create bin directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&luma_bin) {
        eprintln!("Error: Failed to create bin directory: {}", e);
        process::exit(1);
    }

    // Extract luma.exe from the archive
    let exe_name = "luma.exe";
    let mut found = false;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error: Failed to read file from archive: {}", e);
                continue;
            }
        };

        let file_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        // Look for luma.exe in any subdirectory
        if file_path.file_name().and_then(|n| n.to_str()) == Some(exe_name) {
            found = true;

            // Need to rename current exe before replacing it
            let backup_exe = luma_bin.join("luma.exe.old");
            if luma_exe.exists() {
                if let Err(e) = fs::rename(&luma_exe, &backup_exe) {
                    eprintln!("Error: Failed to backup current executable: {}", e);
                    process::exit(1);
                }
            }

            // Write new executable
            let mut outfile = match fs::File::create(&luma_exe) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: Failed to create new executable: {}", e);
                    // Restore backup
                    let _ = fs::rename(&backup_exe, &luma_exe);
                    process::exit(1);
                }
            };

            if let Err(e) = std::io::copy(&mut file, &mut outfile) {
                eprintln!("Error: Failed to write new executable: {}", e);
                // Restore backup
                drop(outfile);
                let _ = fs::remove_file(&luma_exe);
                let _ = fs::rename(&backup_exe, &luma_exe);
                process::exit(1);
            }

            // Remove backup on success
            let _ = fs::remove_file(&backup_exe);

            println!("✓ Successfully extracted luma.exe");
            break;
        }
    }

    if !found {
        eprintln!("Error: luma.exe not found in archive");
        process::exit(1);
    }

    // Verify the new installation
    let output = match process::Command::new(&luma_exe).arg("--version").output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Warning: Could not verify new installation: {}", e);
            println!("\nUpgrade complete!");
            return;
        }
    };

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("\n✓ Luma {} upgraded successfully!", version_output.trim());
        println!("Binary location: {}", luma_exe.display());
    } else {
        eprintln!("Warning: New installation may not be working correctly");
        println!("\nUpgrade complete!");
    }
}

#[cfg(target_os = "windows")]
fn download_release_artifact(version: Option<&str>) -> bytes::Bytes {
    // Determine version string
    let version_tag = match version {
        Some(v) => {
            // Normalize version format
            if v.starts_with("v") {
                format!("{}", v)
            } else if v.chars().next().unwrap().is_ascii_digit() {
                format!("v{}", v)
            } else {
                v.to_string()
            }
        }
        None => "latest".to_string(),
    };

    // Construct download URL
    let arch = "x64";
    let target = format!("luma-windows-{}", arch);
    let base_url = "https://github.com/tayadev/luma/releases";
    let url = if version_tag == "latest" {
        format!("{}/latest/download/{}.zip", base_url, target)
    } else {
        format!("{}/download/{}/{}.zip", base_url, version_tag, target)
    };

    println!("Downloading from: {}", url);

    // Download the release
    let client = reqwest::blocking::Client::builder()
        .user_agent("luma-upgrade")
        .build()
        .expect("Failed to create HTTP client");

    let response = match client.get(&url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: Failed to download release: {}", e);
            process::exit(1);
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "Error: Failed to download release (HTTP {})",
            response.status()
        );
        process::exit(1);
    }

    match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read download: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn download_nightly_artifact() -> bytes::Bytes {
    println!("Fetching latest nightly build from CI...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("luma-upgrade")
        .build()
        .expect("Failed to create HTTP client");

    // Use nightly.link service to download artifacts from the latest successful build workflow
    let artifact_name = "luma-windows-x64";
    let url = format!(
        "https://nightly.link/tayadev/luma/workflows/build/main/{}.zip",
        artifact_name
    );

    println!("Downloading from: {}", url);

    let response = match client.get(&url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: Failed to download nightly build: {}", e);
            eprintln!("Make sure there is a successful build on the main branch.");
            process::exit(1);
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "Error: Failed to download nightly build (HTTP {})",
            response.status()
        );
        eprintln!("Make sure there is a successful build on the main branch.");
        process::exit(1);
    }

    match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read download: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn upgrade_luma(_version: Option<&str>) {
    eprintln!("Error: The upgrade command is currently only supported on Windows.");
    eprintln!("Please download the latest release manually from:");
    eprintln!("https://github.com/tayadev/luma/releases");
    process::exit(1);
}
