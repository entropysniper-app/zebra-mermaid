//! Errors. Ported from Pele `src/errors.ts`.
//!
//! An error keeps Pele's four codes and its line, column and snippet, which are counted in the
//! original source, with the column in UTF-16 code units as JavaScript counts it. Direct
//! parser calls locate errors in their input; the engine maps preprocessing edits back to
//! the caller's source and adds its byte span.

use std::fmt;
use std::ops::Range;

use crate::detect::{Detected, DiagramType, MermaidType};
use crate::util::js;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum ErrorCode {
    /// No diagram type was detected, or the type is not built into this library.
    UnsupportedDiagram,
    /// The text does not follow the diagram's grammar.
    Syntax,
    /// The text parses but asks for something the diagram cannot hold.
    Semantic,
    /// A resource limit was reached: source or output size, edges, nesting, layout work.
    Limit,
    /// The render options are not usable, such as a palette with a non-canonical color.
    /// Pele has no such code; it is new in this library.
    InvalidOptions,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::UnsupportedDiagram => "unsupported-diagram",
            ErrorCode::Syntax => "syntax",
            ErrorCode::Semantic => "semantic",
            ErrorCode::Limit => "limit",
            ErrorCode::InvalidOptions => "invalid-options",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub diagram_type: Option<DiagramType>,
    /// Mermaid's type name, once the engine has detected the diagram.
    pub mermaid_type: Option<MermaidType>,
    /// 1-based line in the original source, or 0 when the error has no position.
    pub line: usize,
    /// 1-based column in UTF-16 code units, or 0 when the error has no position.
    pub column: usize,
    /// The line of text and a caret under the column, as Pele writes it.
    pub snippet: String,
    /// The byte range in the source given to `parse` or `render`, when the error has a position.
    pub span: Option<Range<usize>>,
    /// Byte offset in the text the diagram parser read; mapped to `span` by the engine.
    pub(crate) offset: Option<usize>,
}

impl Error {
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Error {
            code,
            message: message.into(),
            diagram_type: None,
            mermaid_type: None,
            line: 0,
            column: 0,
            snippet: String::new(),
            span: None,
            offset: None,
        }
    }

    pub fn syntax(message: impl Into<String>) -> Self {
        Error::new(message, ErrorCode::Syntax)
    }

    pub fn semantic(message: impl Into<String>) -> Self {
        Error::new(message, ErrorCode::Semantic)
    }

    pub fn limit(message: impl Into<String>) -> Self {
        Error::new(message, ErrorCode::Limit)
    }

    pub fn with_type(mut self, diagram_type: DiagramType) -> Self {
        self.diagram_type = Some(diagram_type);
        self
    }

    /// Adds detected names without replacing more specific information from the parser.
    pub(crate) fn tag(mut self, found: Detected) -> Self {
        self.diagram_type.get_or_insert(found.diagram_type);
        self.mermaid_type.get_or_insert(found.mermaid_type);
        if let std::borrow::Cow::Owned(message) = crate::text::entities::written_codes(&self.message) {
            self.message = message;
        }
        if let std::borrow::Cow::Owned(snippet) = crate::text::entities::written_codes(&self.snippet) {
            self.snippet = snippet;
        }
        self
    }

    /// Relocates a parser diagnostic after a text rewrite, keeping its original detail.
    pub(crate) fn relocate(&mut self, source: &str, offset: usize) {
        let loc = locate(source, offset);
        if !self.snippet.is_empty() {
            self.message = self.message.replacen(&self.snippet, &loc.snippet, 1);
            self.snippet = loc.snippet;
        }
        for prefix in ["Parse error on line ", "Lexical error on line ", "Line ", "line "] {
            let before = format!("{prefix}{}", self.line);
            if self.message.starts_with(&before) {
                self.message.replace_range(..before.len(), &format!("{prefix}{}", loc.line));
                break;
            }
        }
        if self.message.starts_with("Metadata of ") {
            let tail = format!(" ({}:{})", self.line, self.column);
            if self.message.ends_with(&tail) {
                self.message.truncate(self.message.len() - tail.len());
                self.message.push_str(&format!(" ({}:{})", loc.line, loc.column));
            }
        } else if self.message.starts_with("Duplicate event modeling frame ID ") {
            let tail = format!(" on line {}", self.line);
            if self.message.ends_with(&tail) {
                self.message.truncate(self.message.len() - tail.len());
                self.message.push_str(&format!(" on line {}", loc.line));
            }
        }
        self.line = loc.line;
        if self.column > 0 {
            self.column = loc.column;
        }
        self.offset = Some(offset.min(source.len()));
    }

    /// Sets the position from a byte offset into `src`, the text the parser read.
    pub fn at(mut self, src: &str, offset: usize) -> Self {
        let loc = locate(src, offset);
        self.line = loc.line;
        self.column = loc.column;
        self.snippet = loc.snippet;
        self.offset = Some(offset.min(src.len()));
        self
    }

    /// Sets line, column and snippet as given, with no byte offset.
    pub fn with_location(mut self, line: usize, column: usize, snippet: impl Into<String>) -> Self {
        self.line = line;
        self.column = column;
        self.snippet = snippet.into();
        self
    }

    /// The offset the parser reported, in the text it read.
    pub fn parser_offset(&self) -> Option<usize> {
        self.offset
    }

    /// Line and column of the error in the source the caller gave: the 1-based line, the
    /// column in Unicode scalar values, and the column in UTF-16 code units.
    pub fn source_position(&self, source: &str) -> Option<SourcePosition> {
        let span = self.span.as_ref()?;
        let at = span.start.min(source.len());
        let at = (0..=at).rev().find(|&i| source.is_char_boundary(i)).unwrap_or(0);
        let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
        let line = source[..at].matches('\n').count() + 1;
        let head = &source[line_start..at];
        Some(SourcePosition { line, column: head.chars().count() + 1, column_utf16: js::len16(head) + 1, byte: at })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub column_utf16: usize,
    pub byte: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub struct Location {
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

/// The line, UTF-16 column and snippet of byte `offset` in `src`, as Pele's `locate` gives them.
pub fn locate(src: &str, offset: usize) -> Location {
    let mut offset = offset.min(src.len());
    while !src.is_char_boundary(offset) {
        offset -= 1;
    }
    let line = src[..offset].matches('\n').count() + 1;
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line_end = src[offset..].find('\n').map_or(src.len(), |k| offset + k);
    let column = js::len16(&src[line_start..offset]) + 1;
    let text = &src[line_start..line_end];
    let snippet = format!("{}\n{}^", text, "-".repeat(column - 1));
    Location { line, column, snippet }
}

/// A syntax error at byte `offset` of `src`, worded as Mermaid's parsers word them.
pub fn syntax_error(diagram: DiagramType, src: &str, offset: usize, detail: &str, lexical: bool) -> Error {
    let loc = locate(src, offset);
    let message = if lexical {
        format!("Lexical error on line {}. Unrecognized text.\n{}", loc.line, loc.snippet)
    } else {
        format!("Parse error on line {}:\n{}\n{}", loc.line, loc.snippet, detail)
    };
    let mut e = Error::syntax(message).with_type(diagram);
    e.line = loc.line;
    e.column = loc.column;
    e.snippet = loc.snippet;
    e.offset = Some(offset.min(src.len()));
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_in_utf16_columns() {
        let loc = locate("pie\n  中文😀x", 4 + 2 + 6 + 4);
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 7);
        assert_eq!(loc.snippet, "  中文😀x\n------^");
    }

    #[test]
    fn messages() {
        let e = syntax_error(DiagramType::Pie, "pie\nfoo\n", 4, "Expecting x but found `foo`.", false);
        assert_eq!(e.message, "Parse error on line 2:\nfoo\n^\nExpecting x but found `foo`.");
        assert_eq!(e.code, ErrorCode::Syntax);
        let e = syntax_error(DiagramType::Pie, "pie\n?", 4, "", true);
        assert_eq!(e.message, "Lexical error on line 2. Unrecognized text.\n?\n^");
    }
}
