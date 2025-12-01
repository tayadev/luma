use clap::{CommandFactory, Parser, Subcommand};

mod check;
mod compile;
mod debug;
mod lsp;
mod repl;
mod run;
mod upgrade;
mod utils;

#[cfg(test)]
mod tests;

use check::handle_check;
use compile::handle_compile;
use debug::{handle_ast, handle_bytecode};
use lsp::handle_lsp;
use repl::handle_repl;
use run::handle_run;
use upgrade::handle_upgrade;

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
    /// Start the Language Server Protocol server
    Lsp,
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { file }) => {
            handle_run(file);
        }
        Some(Commands::Repl) => {
            handle_repl();
        }
        Some(Commands::Lsp) => {
            handle_lsp();
        }
        Some(Commands::Check { file }) => {
            handle_check(file);
        }
        Some(Commands::Compile { file, output }) => {
            handle_compile(file, output.as_deref());
        }
        Some(Commands::Upgrade { version }) => {
            handle_upgrade(version.as_deref());
        }
        Some(Commands::Ast { file }) => {
            handle_ast(file);
        }
        Some(Commands::Bytecode { file }) => {
            handle_bytecode(file);
        }
        None => {
            // Default: run the file if provided, otherwise print help
            let file = match &cli.file {
                Some(f) => f,
                None => {
                    // No file and no subcommand - print help
                    Cli::command().print_help().unwrap();
                    println!();
                    std::process::exit(0);
                }
            };
            handle_run(file);
        }
    }
}
