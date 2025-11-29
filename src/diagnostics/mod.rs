use crate::ast::Span;
use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Diagnostic kind/category
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse,
    Type,
    Runtime,
}

/// A diagnostic message with location and context
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub filename: String,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(kind: DiagnosticKind, message: String, span: Span, filename: String) -> Self {
        Self {
            kind,
            severity: Severity::Error,
            message,
            span,
            filename,
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    /// Format the diagnostic with source code snippet
    pub fn format(&self, source: &str) -> String {
        let line_index = LineIndex::new(source);
        let formatter = DiagnosticFormatter {
            diagnostic: self,
            source,
            line_index: &line_index,
        };
        formatter.format()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} at {}:{}:{}",
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            },
            self.message,
            self.filename,
            self.span.start,
            self.span.end
        )
    }
}

/// Line index for efficient offset-to-line/column conversion
#[derive(Debug)]
pub struct LineIndex {
    /// Starting byte offset of each line
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    /// Convert byte offset to (line, column) (both 1-indexed)
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search for the line
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };

        let line_start = self.line_starts[line];
        let col = offset.saturating_sub(line_start);

        (line + 1, col + 1)
    }

    /// Get the byte range for a given line (1-indexed)
    pub fn line_range(&self, line: usize) -> Option<(usize, usize)> {
        if line == 0 || line > self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line - 1];
        let end = if line < self.line_starts.len() {
            self.line_starts[line].saturating_sub(1) // Exclude newline
        } else {
            usize::MAX // Last line extends to EOF
        };
        Some((start, end))
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

/// Formats a diagnostic with source code snippet
struct DiagnosticFormatter<'a> {
    diagnostic: &'a Diagnostic,
    source: &'a str,
    line_index: &'a LineIndex,
}

impl<'a> DiagnosticFormatter<'a> {
    fn format(&self) -> String {
        let mut output = String::new();

        // Header: error/warning: message
        let severity_str = match self.diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        output.push_str(&format!("{}: {}\n", severity_str, self.diagnostic.message));

        // Location
        let (start_line, start_col) = self.line_index.line_col(self.diagnostic.span.start);
        let (end_line, end_col) = self.line_index.line_col(self.diagnostic.span.end);

        output.push_str(&format!(
            "  --> {}:{}:{}\n",
            self.diagnostic.filename, start_line, start_col
        ));

        // Source snippet
        output.push_str(&self.format_snippet(start_line, start_col, end_line, end_col));

        // Notes
        for note in &self.diagnostic.notes {
            output.push_str(&format!("note: {note}\n"));
        }

        // Help
        if let Some(help) = &self.diagnostic.help {
            output.push_str(&format!("help: {help}\n"));
        }

        output
    }

    fn format_snippet(
        &self,
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
        let context_end = (end_line + 1).min(self.line_index.line_count());

        output.push_str(&format!("{:width$} |\n", "", width = line_num_width));

        for line_num in context_start..=context_end {
            if let Some((line_start, line_end)) = self.line_index.line_range(line_num) {
                let line_end = line_end.min(self.source.len());
                let line_text = &self.source[line_start..line_end];

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
    fn test_line_index_single_line() {
        let source = "hello world";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col(0), (1, 1));
        assert_eq!(index.line_col(6), (1, 7));
        assert_eq!(index.line_col(11), (1, 12));
    }

    #[test]
    fn test_line_index_multi_line() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col(0), (1, 1)); // 'l' in line1
        assert_eq!(index.line_col(5), (1, 6)); // '\n' after line1
        assert_eq!(index.line_col(6), (2, 1)); // 'l' in line2
        assert_eq!(index.line_col(12), (3, 1)); // 'l' in line3
    }

    #[test]
    fn test_line_range() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);
        assert_eq!(index.line_range(1), Some((0, 5)));
        assert_eq!(index.line_range(2), Some((6, 11)));
        assert_eq!(index.line_range(3), Some((12, usize::MAX)));
    }

    #[test]
    fn test_diagnostic_format_single_line() {
        let source = "let x = 1 + true;";
        let span = Span::new(12, 16); // "true"
        let diag = Diagnostic::error(
            DiagnosticKind::Type,
            "type mismatch: expected Int, found Bool".to_string(),
            span,
            "test.luma".to_string(),
        );
        let formatted = diag.format(source);
        assert!(formatted.contains("error:"));
        assert!(formatted.contains("test.luma:1:13"));
        assert!(formatted.contains("let x = 1 + true;"));
        assert!(formatted.contains("^~~~"));
    }

    #[test]
    fn test_diagnostic_format_multi_line() {
        let source = "fn foo() do\n  let x = 1 +\n    true\nend";
        let span = Span::new(26, 34); // "1 +\n    true"
        let diag = Diagnostic::error(
            DiagnosticKind::Type,
            "type mismatch".to_string(),
            span,
            "test.luma".to_string(),
        );
        let formatted = diag.format(source);
        assert!(formatted.contains("error:"));
        assert!(formatted.contains("test.luma:3:1")); // Points to start of span end line
        assert!(formatted.contains("let x = 1 +"));
        assert!(formatted.contains("true"));
    }
}
