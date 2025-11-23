//! Source location tracking (spans) for the Luma AST

use serde::{Deserialize, Serialize};

/// Represents a location in the source code as byte offsets
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct Span {
    /// Byte offset of the start of the span (inclusive)
    pub start: usize,
    /// Byte offset of the end of the span (exclusive)
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Create a span from a Chumsky SimpleSpan
    pub fn from_chumsky(span: chumsky::span::SimpleSpan) -> Self {
        Span {
            start: span.start,
            end: span.end,
        }
    }

    /// Calculate line and column from source text
    pub fn location(&self, source: &str) -> Location {
        let mut line = 1;
        let mut col = 1;

        for (byte_idx, ch) in source.char_indices() {
            if byte_idx >= self.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Location {
            line,
            col,
            offset: self.start,
        }
    }

    /// Get the source text for this span
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end.min(source.len())]
    }
}

/// Represents a specific location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

/// A value with an associated source span
/// Serializes transparently to the inner value for backward compatibility
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

// Serialize as the inner value only (skip span for now to avoid breaking fixtures)
impl<T: Serialize> Serialize for Spanned<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

// Deserialize as the inner value with a dummy span
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Spanned<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Spanned {
            value,
            span: Span::new(0, 0), // Dummy span during deserialization
        })
    }
}
