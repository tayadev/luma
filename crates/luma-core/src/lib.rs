pub mod ast;
pub mod bytecode;
pub mod diagnostics;
pub mod parser;
pub mod pipeline;
pub mod typecheck;
pub mod vm;

#[doc(hidden)]
pub mod test_utils;

// Re-export commonly used types for convenience
pub use ast::{Expr, Program, Stmt};
pub use diagnostics::{Diagnostic, Severity};
pub use pipeline::Pipeline;
