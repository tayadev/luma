//! Unified pipeline for executing Luma programs
//!
//! This module provides a `Pipeline` abstraction that encapsulates the complete
//! workflow of parsing, type-checking, compiling, and executing Luma code.
//!
//! ## Usage
//!
//! ```no_run
//! # use luma::pipeline::Pipeline;
//! let pipeline = Pipeline::new("let x = 1 + 2".to_string(), "example.luma".to_string());
//!
//! match pipeline.run_all() {
//!     Ok(value) => println!("Result: {}", value),
//!     Err(e) => eprintln!("Error: {}", e.format_display()),
//! }
//! ```
//!
//! ## Individual Stages
//!
//! You can also run individual stages:
//!
//! ```no_run
//! # use luma::pipeline::Pipeline;
//! let pipeline = Pipeline::new("let x = 1".to_string(), "example.luma".to_string());
//!
//! let ast = pipeline.parse()?;
//! pipeline.typecheck(&ast)?;
//! let chunk = pipeline.compile(&ast);
//! # Ok::<(), luma::pipeline::PipelineError>(())
//! ```

use crate::ast::Program;
use crate::bytecode::ir::Chunk;
use crate::diagnostics::Diagnostic;
use crate::typecheck::{self, TypeError};
use crate::vm::value::Value;
use crate::vm::{self, VmError};
use std::fmt;
use std::path::Path;

/// Errors that can occur during pipeline execution
#[derive(Debug)]
pub enum PipelineError {
    /// Parse error(s)
    Parse(Vec<Diagnostic>),
    /// Type checking error(s)
    Typecheck(Vec<TypeError>),
    /// Runtime error
    Runtime(VmError),
}

impl PipelineError {
    /// Format error for display to user
    pub fn format_display(&self) -> String {
        match self {
            PipelineError::Parse(diagnostics) => diagnostics
                .iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
                .join("\n"),
            PipelineError::Typecheck(errors) => {
                errors
                    .iter()
                    .map(|e| {
                        if let Some(span) = &e.span {
                            // Create a dummy source to calculate line/col, or use byte range
                            format!(
                                "Type error at bytes {}..{}: {}",
                                span.start, span.end, e.message
                            )
                        } else {
                            format!("Type error: {}", e.message)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            PipelineError::Runtime(err) => err.to_string(),
        }
    }

    /// Format error with source code context
    pub fn format_with_source(&self, source: &str) -> String {
        match self {
            PipelineError::Parse(diagnostics) => diagnostics
                .iter()
                .map(|d| d.format(source))
                .collect::<Vec<_>>()
                .join("\n"),
            PipelineError::Typecheck(errors) => errors
                .iter()
                .map(|e| {
                    if let Some(span) = &e.span {
                        let loc = span.location(source);
                        format!("Type error at {}:{}: {}", loc.line, loc.col, e.message)
                    } else {
                        format!("Type error: {}", e.message)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            PipelineError::Runtime(err) => err.format(Some(source)),
        }
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_display())
    }
}

impl From<Vec<Diagnostic>> for PipelineError {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        PipelineError::Parse(diagnostics)
    }
}

impl From<Vec<TypeError>> for PipelineError {
    fn from(errors: Vec<TypeError>) -> Self {
        PipelineError::Typecheck(errors)
    }
}

impl From<VmError> for PipelineError {
    fn from(error: VmError) -> Self {
        PipelineError::Runtime(error)
    }
}

/// Result type for pipeline operations
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Unified pipeline for parsing, type-checking, compiling, and executing Luma code
pub struct Pipeline {
    /// Source code to execute
    source: String,
    /// Filename for error reporting
    filename: String,
}

impl Pipeline {
    /// Create a new pipeline with source code and filename
    pub fn new(source: String, filename: String) -> Self {
        Pipeline { source, filename }
    }

    /// Parse the source code into an AST
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Parse` if parsing fails
    pub fn parse(&self) -> PipelineResult<Program> {
        crate::parser::parse(&self.source, &self.filename)
            .map_err(PipelineError::Parse)
    }

    /// Type-check the AST
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Typecheck` if type checking fails
    pub fn typecheck(&self, ast: &Program) -> PipelineResult<()> {
        typecheck::typecheck_program(ast).map_err(PipelineError::Typecheck)
    }

    /// Compile the AST to bytecode
    ///
    /// This operation never fails - invalid ASTs are rejected during type checking
    pub fn compile(&self, ast: &Program) -> Chunk {
        crate::bytecode::compile::compile_program(ast)
    }

    /// Execute bytecode in a new VM
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Runtime` if execution fails
    pub fn execute(&self, chunk: Chunk) -> PipelineResult<Value> {
        let absolute_path = if self.filename == "-" {
            Some("<stdin>".to_string())
        } else {
            match Path::new(&self.filename).canonicalize() {
                Ok(path) => Some(path.to_string_lossy().to_string()),
                Err(_) => Some(self.filename.clone()),
            }
        };

        let mut vm = vm::VM::new_with_file(chunk, absolute_path);
        vm.set_source(self.source.clone());
        vm.run().map_err(PipelineError::Runtime)
    }

    /// Execute the complete pipeline: parse → typecheck → compile → run
    ///
    /// # Arguments
    ///
    /// * `absolute_path` - Optional file path for error reporting. If `None`, uses the filename
    ///
    /// # Errors
    ///
    /// Returns error at the first stage that fails:
    /// 1. Parse errors
    /// 2. Type check errors
    /// 3. Runtime errors
    pub fn run_all(&self) -> PipelineResult<Value> {
        let ast = self.parse()?;
        self.typecheck(&ast)?;
        let chunk = self.compile(&ast);
        self.execute(chunk)
    }

    /// Get the source code
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the filename
    pub fn filename(&self) -> &str {
        &self.filename
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_simple_execution() {
        let pipeline = Pipeline::new("1 + 2".to_string(), "test.luma".to_string());
        let result = pipeline.run_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_parse_error() {
        let pipeline = Pipeline::new("1 +".to_string(), "test.luma".to_string());
        let result = pipeline.run_all();
        assert!(matches!(result, Err(PipelineError::Parse(_))));
    }

    #[test]
    fn test_pipeline_type_error() {
        let pipeline = Pipeline::new(
            "let x: Number = \"string\"".to_string(),
            "test.luma".to_string(),
        );
        let result = pipeline.run_all();
        assert!(matches!(result, Err(PipelineError::Typecheck(_))));
    }

    #[test]
    fn test_pipeline_individual_stages() {
        let pipeline = Pipeline::new("1 + 2".to_string(), "test.luma".to_string());

        // Parse
        let ast = pipeline.parse().expect("parse failed");

        // Typecheck
        pipeline.typecheck(&ast).expect("typecheck failed");

        // Compile
        let chunk = pipeline.compile(&ast);

        // Execute
        let result = pipeline.execute(chunk).expect("execute failed");
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_pipeline_error_formatting() {
        let pipeline = Pipeline::new("1 +".to_string(), "test.luma".to_string());
        let error = pipeline.run_all().unwrap_err();

        let formatted = error.format_display();
        assert!(!formatted.is_empty());
    }
}
