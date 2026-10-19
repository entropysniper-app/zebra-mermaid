//! Venn lexer. Ported from Pele `src/diagrams/venn/lexer.ts`.
//!
//! Positions are bytes of the UTF-8 text. Every test the original makes on a code unit is on an
//! ASCII character or on white space within the Basic Multilingual Plane, so a byte answers it,
//! except where a character outside ASCII may stand at the place tested; there the character is
//! decoded and its UTF-16 length is respected.

use super::db::VennDb;
use crate::util::chars::{char_at, is_space, is_word_byte};
use crate::util::js;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum T {
    End,
    Error,
    Eof,
    Newline,
    Venn,
    Title,
    Set,
    Union,
    Text,
    IndentText,
    Style,
    BracketLabel,
    Numeric,
    Hexcolor,
    Rgbcolor,
    Rgbacolor,
    Identifier,
    String,
    Comma,
    Colon,
}

impl T {
    pub fn name(self) -> &'static str {
        match self {
            T::End => "$end",
            T::Error => "INVALID",
            T::Eof => "EOF",
            T::Newline => "NEWLINE",
            T::Venn => "VENN",
            T::Title => "TITLE",
            T::Set => "SET",
            T::Union => "UNION",
            T::Text => "TEXT",
            T::IndentText => "INDENT_TEXT",
            T::Style => "STYLE",
            T::BracketLabel => "BRACKET_LABEL",
            T::Numeric => "NUMERIC",
            T::Hexcolor => "HEXCOLOR",
            T::Rgbcolor => "RGBCOLOR",
            T::Rgbacolor => "RGBACOLOR",
            T::Identifier => "IDENTIFIER",
            T::String => "STRING",
            T::Comma => "COMMA",
            T::Colon => "COLON",
        }
    }
}

const KEYWORDS: [(&str, T); 5] = [("venn-beta", T::Venn), ("set", T::Set), ("union", T::Union), ("text", T::Text), ("style", T::Style)];

#[inline]
fn is_digit(b: Option<&u8>) -> bool {
    b.is_some_and(u8::is_ascii_digit)
}

#[inline]
fn is_blank(b: Option<&u8>) -> bool {
    matches!(b, Some(b' ' | b'\t'))
}

/// Reads tokens one at a time, as Mermaid's Jison lexer for venn diagrams does: every rule
/// ignores case, the first rule that matches wins, and a keyword must end at a word boundary.
pub(crate) struct VennLexer<'a> {
    src: &'a str,
    /// The text and position of the token `next` returned last.
    pub text: &'a str,
    pub start: usize,
    at: usize,
    line_start: bool,
    ended: bool,
}

impl<'a> VennLexer<'a> {
    pub fn new(src: &'a str) -> Self {
        VennLexer { src, text: "", start: 0, at: 0, line_start: false, ended: false }
    }

    /// `word` at `at`, its letters in either case.
    fn word(&self, at: usize, word: &str) -> bool {
        let b = self.src.as_bytes();
        for (k, w) in word.bytes().enumerate() {
            let Some(&c) = b.get(at + k) else { return false };
            if c != w && (c.wrapping_add(32) != w || !c.is_ascii_uppercase()) {
                return false;
            }
        }
        true
    }

    fn keyword(&self, at: usize, word: &str) -> bool {
        self.word(at, word) && !self.src.as_bytes().get(at + word.len()).is_some_and(|&c| is_word_byte(c))
    }

    /// `isSpace(src.charCodeAt(i))` and the code units the character takes.
    fn space_at(&self, i: usize) -> Option<usize> {
        char_at(self.src, i).filter(|&c| is_space(c)).map(char::len_utf8)
    }

    /// The end of `rgb(1, 2, 3)` or `rgba(1, 2, 3, 0.5)` whose parenthesis opens at `at`.
    fn color(&self, at: usize, parts: usize) -> Option<usize> {
        let b = self.src.as_bytes();
        let mut i = at + 1;
        for part in 0..parts {
            while let Some(len) = self.space_at(i) {
                i += len;
            }
            let from = i;
            while is_digit(b.get(i)) || b.get(i) == Some(&b'.') {
                i += 1;
            }
            if i == from {
                return None;
            }
            while let Some(len) = self.space_at(i) {
                i += len;
            }
            if b.get(i) != Some(if part == parts - 1 { &b')' } else { &b',' }) {
                return None;
            }
            i += 1;
        }
        Some(i)
    }

    fn line_end(&self, from: usize) -> usize {
        let b = self.src.as_bytes();
        if from >= b.len() {
            return b.len();
        }
        b[from..].iter().position(|&c| c == b'\n').map_or(b.len(), |k| from + k)
    }

    pub fn next(&mut self, host: &mut VennDb) -> T {
        let src = self.src;
        let b = src.as_bytes();
        let n = b.len();
        loop {
            let mut i = self.at;
            self.start = i;
            self.text = "";
            if i >= n {
                if self.ended {
                    return T::End;
                }
                self.ended = true;
                return T::Eof;
            }
            let c = b[i];
            // A comment runs to the end of the line. It also takes the character before it along,
            // whatever that is: a line break, or the last letter of a word. The code unit after a
            // character outside the Basic Multilingual Plane is its low surrogate, never `%`.
            let width = char_at(src, i).map_or(1, char::len_utf8);
            if (c == b'%' && b.get(i + 1) == Some(&b'%') && b.get(i + 2) != Some(&b'{'))
                || (c != b'}' && width < 4 && b.get(i + width) == Some(&b'%') && b.get(i + width + 1) == Some(&b'%'))
            {
                // `lineEnd(i + 2)`: from the second `%`.
                self.at = self.line_end(i + width + 1);
                continue;
            }
            if self.line_start {
                if c == b' ' || c == b'\t' {
                    let mut j = i + 1;
                    while is_blank(b.get(j)) {
                        j += 1;
                    }
                    self.at = j;
                    if self.keyword(j, "text") && host.indent_mode() {
                        // The word `text` itself is dropped; the indentation stands for it.
                        self.at = j + 4;
                        self.line_start = false;
                        self.text = &src[i..j];
                        return T::IndentText;
                    }
                    continue;
                }
                if c != b'\n' && c != b'\r' {
                    host.set_indent_mode(false);
                    self.line_start = false;
                }
            }
            if c == b'\n' || c == b'\r' {
                i += 1;
                while matches!(b.get(i), Some(b'\n' | b'\r')) {
                    i += 1;
                }
                self.line_start = true;
                return self.token(T::Newline, i);
            }
            if c == b'%' && b.get(i + 1) == Some(&b'%') {
                self.at = self.line_end(i);
                continue;
            }
            if c == b' ' || c == b'\t' {
                i += 1;
                while is_blank(b.get(i)) {
                    i += 1;
                }
                self.at = i;
                continue;
            }

            if c.is_ascii_alphabetic() || c == b'_' {
                if self.word(i, "title") {
                    if let Some(space) = self.space_at(i + 5) {
                        let first = i + 5 + space;
                        let mut end = first;
                        while end < n {
                            let ch = b[end];
                            if ch == b'#' || ch == b'\n' || ch == b';' {
                                break;
                            }
                            end += 1;
                        }
                        if end > first {
                            return self.token(T::Title, end);
                        }
                    }
                }
                for (word, kind) in KEYWORDS {
                    if self.keyword(i, word) {
                        return self.token(kind, i + word.len());
                    }
                }
                if self.word(i, "rgb") {
                    let alpha = b.get(i + 3).is_some_and(|&x| x | 32 == b'a');
                    let open = i + if alpha { 4 } else { 3 };
                    let end = if b.get(open) == Some(&b'(') { self.color(open, if alpha { 4 } else { 3 }) } else { None };
                    if let Some(end) = end {
                        return self.token(if alpha { T::Rgbacolor } else { T::Rgbcolor }, end);
                    }
                }
                i += 1;
                while b.get(i).is_some_and(|&x| is_word_byte(x) || x == b'-') {
                    i += 1;
                }
                return self.token(T::Identifier, i);
            }
            if c == b'[' {
                if b.get(i + 1) == Some(&b'"') {
                    let quote = b[i + 2..].iter().position(|&x| x == b'"').map(|k| i + 2 + k);
                    let Some(quote) = quote.filter(|&q| b.get(q + 1) == Some(&b']')) else {
                        return T::Error;
                    };
                    self.at = quote + 2;
                    self.text = &src[i + 2..quote];
                    return T::BracketLabel;
                }
                let mut end = i + 1;
                while end < n && b[end] != b']' && b[end] != b'"' {
                    end += 1;
                }
                if end == i + 1 || b.get(end) != Some(&b']') {
                    return T::Error;
                }
                self.at = end + 1;
                self.text = js::trim(&src[i + 1..end]);
                return T::BracketLabel;
            }
            if c.is_ascii_digit() || c == b'.' || c == b'+' || c == b'-' {
                if c == b'+' || c == b'-' {
                    i += 1;
                }
                let from = i;
                while is_digit(b.get(i)) {
                    i += 1;
                }
                if i > from {
                    if b.get(i) == Some(&b'.') && is_digit(b.get(i + 1)) {
                        i += 2;
                        while is_digit(b.get(i)) {
                            i += 1;
                        }
                    }
                } else if b.get(i) == Some(&b'.') && is_digit(b.get(i + 1)) {
                    i += 2;
                    while is_digit(b.get(i)) {
                        i += 1;
                    }
                } else {
                    return T::Error;
                }
                return self.token(T::Numeric, i);
            }
            if c == b'#' {
                let mut end = i + 1;
                while end < i + 9 && b.get(end).is_some_and(u8::is_ascii_hexdigit) {
                    end += 1;
                }
                return if end >= i + 4 { self.token(T::Hexcolor, end) } else { T::Error };
            }
            if c == b'"' {
                return match b[i + 1..].iter().position(|&x| x == b'"') {
                    None => T::Error,
                    Some(k) => self.token(T::String, i + 1 + k + 1),
                };
            }
            if c == b',' {
                return self.token(T::Comma, i + 1);
            }
            if c == b':' {
                return self.token(T::Colon, i + 1);
            }
            return T::Error;
        }
    }

    fn token(&mut self, kind: T, end: usize) -> T {
        self.text = &self.src[self.at..end];
        self.at = end;
        kind
    }
}
