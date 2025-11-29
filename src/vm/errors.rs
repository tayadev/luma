//! Error types for VM runtime errors

use crate::ast::Span;

/// Represents a runtime error with optional source location information
#[derive(Debug)]
pub struct VmError {
    pub message: String,
    pub span: Option<Span>,
    pub file: Option<String>,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(None))
    }
}

impl VmError {
    /// Create a simple runtime error without location info
    pub fn runtime(message: String) -> Self {
        VmError {
            message,
            span: None,
            file: None,
        }
    }

    /// Create a runtime error with location info
    pub fn with_location(message: String, span: Option<Span>, file: Option<String>) -> Self {
        VmError {
            message,
            span,
            file,
        }
    }

    /// Format the error with source location if available
    pub fn format(&self, source: Option<&str>) -> String {
        let mut result = String::new();

        if let (Some(file), Some(span)) = (&self.file, &self.span) {
            if let Some(src) = source {
                let loc = span.location(src);
                result.push_str(&format!(
                    "Runtime error at {}:{}:{}\n",
                    file, loc.line, loc.col
                ));

                // Show the line with the error
                let lines: Vec<&str> = src.lines().collect();
                if loc.line > 0 && loc.line <= lines.len() {
                    result.push_str(&format!("  {} | {}\n", loc.line, lines[loc.line - 1]));
                    result.push_str(&format!(
                        "  {} | {}{}\n",
                        " ".repeat(loc.line.to_string().len()),
                        " ".repeat(loc.col.saturating_sub(1)),
                        "^"
                    ));
                }
            } else {
                result.push_str(&format!("Runtime error at {file}\n"));
            }
        } else {
            result.push_str("Runtime error\n");
        }

        result.push_str(&self.message);
        result
    }
}
