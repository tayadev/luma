//! Command-line interface handlers for Luma
//!
//! This module organizes CLI command handlers into separate, testable functions.
//! Each command has its own handler that manages error handling and output formatting.

pub mod check;
pub mod compile;
pub mod debug;
pub mod lsp;
pub mod repl;
pub mod run;
pub mod upgrade;
pub mod utils;

pub use check::handle_check;
pub use compile::handle_compile;
pub use debug::{handle_ast, handle_bytecode};
pub use lsp::handle_lsp;
pub use repl::handle_repl;
pub use run::handle_run;
pub use upgrade::handle_upgrade;
pub use utils::{format_parse_errors, format_typecheck_errors, read_source};
