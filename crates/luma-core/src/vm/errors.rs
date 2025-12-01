//! Error types for VM runtime errors

use crate::ast::Span;
use crate::diagnostics::LineIndex;

/// Represents a runtime error with optional source location information
#[derive(Debug)]
pub struct VmError {
    pub message: String,
    pub span: Option<Span>,
    pub file: Option<String>,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_display())
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

    /// Format error without source code (brief format)
    pub fn format_display(&self) -> String {
        if let (Some(file), Some(_span)) = (&self.file, &self.span) {
            format!("runtime error in {}: {}", file, self.message)
        } else {
            format!("runtime error: {}", self.message)
        }
    }

    /// Legacy format method for compatibility - use format_with_source() instead
    pub fn format(&self, source: Option<&str>) -> String {
        match source {
            Some(src) => self.format_with_source(src),
            None => self.format_display(),
        }
    }

    /// Format error with source code context (like parser diagnostics)
    pub fn format_with_source(&self, source: &str) -> String {
        if let (Some(span), Some(file)) = (self.span, self.file.as_ref()) {
            let line_index = LineIndex::new(source);
            let (start_line, start_col) = line_index.line_col(span.start);
            let (end_line, end_col) = line_index.line_col(span.end);

            let mut output = String::new();

            // Header
            output.push_str(&format!("error: {}\n", self.message));

            // Location
            output.push_str(&format!("  --> {}:{}:{}\n", file, start_line, start_col));

            // Source snippet
            output.push_str(&self.format_snippet(
                source,
                &line_index,
                start_line,
                start_col,
                end_line,
                end_col,
            ));

            output
        } else {
            self.format_display()
        }
    }

    fn format_snippet(
        &self,
        source: &str,
        line_index: &LineIndex,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> String {
        let mut output = String::new();

        // Determine line number width for padding
        let max_line = end_line.max(start_line);
        let line_num_width = max_line.to_string().len();

        // Show lines with context (1 line before/after)
        let context_start = start_line.saturating_sub(1).max(1);
        let context_end = (end_line + 1).min(line_index.line_count());

        output.push_str(&format!("{:width$} |\n", "", width = line_num_width));

        for line_num in context_start..=context_end {
            if let Some((line_start, line_end)) = line_index.line_range(line_num) {
                let line_end = line_end.min(source.len());
                let line_text = &source[line_start..line_end];

                // Print line number and source
                output.push_str(&format!("{line_num:line_num_width$} | {line_text}\n"));

                // Print underline/caret for error span
                if line_num >= start_line && line_num <= end_line {
                    output.push_str(&format!("{:width$} | ", "", width = line_num_width));

                    let line_span_start = if line_num == start_line {
                        start_col - 1
                    } else {
                        0
                    };
                    let line_span_end = if line_num == end_line {
                        end_col - 1
                    } else {
                        line_text.chars().count()
                    };

                    // Add spaces up to start of error
                    for _ in 0..line_span_start {
                        output.push(' ');
                    }

                    // Add carets/tildes for error span
                    let span_width = (line_span_end.saturating_sub(line_span_start)).max(1);
                    if span_width == 1 {
                        output.push('^');
                    } else {
                        for i in 0..span_width {
                            if i == 0 {
                                output.push('^');
                            } else {
                                output.push('~');
                            }
                        }
                    }

                    output.push('\n');
                }
            }
        }

        output.push_str(&format!("{:width$} |\n", "", width = line_num_width));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_error_display_no_source() {
        let err = VmError::runtime("test error".to_string());
        assert_eq!(err.format_display(), "runtime error: test error");
    }

    #[test]
    fn test_vm_error_display_with_file() {
        let err = VmError::with_location(
            "division by zero".to_string(),
            Some(Span::new(10, 15)),
            Some("script.luma".to_string()),
        );
        let result = err.format_display();
        assert!(result.contains("script.luma"));
        assert!(result.contains("division by zero"));
    }

    #[test]
    fn test_vm_error_format_with_source() {
        let source = "let x = 1\nlet y = x / 0\nlet z = 2";
        let err = VmError::with_location(
            "division by zero".to_string(),
            Some(Span::new(20, 25)), // Points to "x / 0"
            Some("script.luma".to_string()),
        );

        let formatted = err.format_with_source(source);

        // Should contain error header
        assert!(formatted.contains("error: division by zero"));

        // Should contain file location
        assert!(formatted.contains("script.luma"));

        // Should contain source line
        assert!(formatted.contains("let y = x / 0"));

        // Should contain pointer indicator
        assert!(formatted.contains("^"));
    }

    #[test]
    fn test_vm_error_legacy_format_method() {
        let source = "x / 0";
        let err = VmError::with_location(
            "division by zero".to_string(),
            Some(Span::new(0, 5)),
            Some("test.luma".to_string()),
        );

        // format with source should work
        let formatted = err.format(Some(source));
        assert!(formatted.contains("division by zero"));
        assert!(formatted.contains("test.luma"));

        // format without source should work
        let formatted = err.format(None);
        assert!(formatted.contains("division by zero"));
    }

    #[test]
    fn test_vm_error_multiline_context() {
        let source = "fn add(a, b)\n  a + b\nend\n\nx / 0";
        // "fn add(a, b)\n" = 14 chars
        // "  a + b\n" = 8 chars
        // "end\n" = 4 chars
        // "\n" = 1 char
        // So "x / 0" starts at byte 27
        let err = VmError::with_location(
            "division by zero".to_string(),
            Some(Span::new(27, 32)), // Points to "x / 0"
            Some("math.luma".to_string()),
        );

        let formatted = err.format_with_source(source);

        // Should show the problematic line
        assert!(formatted.contains("x / 0"));

        // Should have file location
        assert!(formatted.contains("math.luma"));
    }
}
