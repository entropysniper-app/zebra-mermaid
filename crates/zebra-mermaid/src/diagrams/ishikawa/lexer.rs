//! Ishikawa lexer. Ported from Pele `src/diagrams/ishikawa/lexer.ts`. Pele counts UTF-16 code
//! units; offsets here are bytes, and the parser measures indentation in code units.

use crate::util::chars::{char_at, is_space, is_word_byte};

pub const END: usize = 0;
pub const EOF: usize = 1;
pub const SPACELINE: usize = 2;
pub const NL: usize = 3;
pub const ISHIKAWA: usize = 4;
pub const SPACELIST: usize = 5;
pub const TEXT: usize = 6;

pub const TOKEN_NAMES: [&str; 7] = ["$end", "EOF", "SPACELINE", "NL", "ISHIKAWA", "SPACELIST", "TEXT"];

#[derive(Debug, Clone, Default)]
pub struct Tokens {
    pub types: Vec<usize>,
    pub starts: Vec<usize>,
    pub ends: Vec<usize>,
}

fn byte(src: &str, at: usize) -> Option<u8> {
    src.as_bytes().get(at).copied()
}

/// `ishikawa-beta`, or failing that `ishikawa`, either one ending at a word boundary, with
/// ASCII letters of either case: the length of the keyword at `at`, or 0.
fn keyword(src: &str, at: usize) -> usize {
    let word = b"ishikawa-beta";
    let mut k = 0;
    while k < 13 {
        let Some(c) = byte(src, at + k) else { break };
        let w = word[k];
        if c != w && (c.wrapping_add(32) != w || !c.is_ascii_uppercase()) {
            break;
        }
        k += 1;
    }
    if k == 13 && !byte(src, at + 13).is_some_and(is_word_byte) {
        return 13;
    }
    if k >= 8 && !byte(src, at + 8).is_some_and(is_word_byte) {
        8
    } else {
        0
    }
}

/// Produces the tokens of Mermaid's Jison lexer for Ishikawa diagrams, where the first rule
/// that matches wins: white space up to a comment or up to the last line break in it is a blank
/// line, a single line break before text is a new line, other white space is indentation, and
/// the rest of a line is text.
pub fn tokenize(src: &str) -> Tokens {
    let mut out = Tokens::default();
    let n = src.len();
    let mut i = 0;
    while i < n {
        let mut j = i;
        let mut last_break: Option<usize> = None;
        while let Some(c) = char_at(src, j).filter(|&c| is_space(c)) {
            if c == '\n' {
                last_break = Some(j);
            }
            j += c.len_utf8();
        }
        let kind;
        let end;
        if byte(src, j) == Some(b'%') && byte(src, j + 1) == Some(b'%') {
            kind = SPACELINE;
            // `[^\n\r  ]*`
            end = src[j + 2..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(n, |k| j + 2 + k);
        } else if j == i {
            let length = keyword(src, i);
            if length > 0 {
                kind = ISHIKAWA;
                end = i + length;
            } else {
                kind = TEXT;
                end = src[i..].find('\n').map_or(n, |k| i + k);
            }
        } else if last_break.is_some_and(|b| b > i) {
            kind = SPACELINE;
            end = last_break.unwrap_or(i) + 1;
        } else if last_break == Some(i) {
            kind = NL;
            end = i + 1;
        } else {
            kind = SPACELIST;
            end = j;
        }
        out.types.push(kind);
        out.starts.push(i);
        out.ends.push(end);
        i = end;
    }
    out.types.extend([EOF, END]);
    out.starts.extend([n, n]);
    out.ends.extend([n, n]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<&'static str> {
        tokenize(src).types.iter().map(|&t| TOKEN_NAMES[t]).collect()
    }

    #[test]
    fn keywords() {
        assert_eq!(keyword("ishikawa-beta\n", 0), 13);
        assert_eq!(keyword("ISHIKAWA-Beta", 0), 13);
        assert_eq!(keyword("ishikawa\n", 0), 8);
        assert_eq!(keyword("ishikawa-x", 0), 8);
        assert_eq!(keyword("ishikawa-betax", 0), 8);
        assert_eq!(keyword("ishikawax", 0), 0);
        assert_eq!(keyword("ishika", 0), 0);
    }

    #[test]
    fn lines() {
        assert_eq!(kinds("ishikawa\nEffect\n  Cause\n"), ["ISHIKAWA", "NL", "TEXT", "NL", "SPACELIST", "TEXT", "NL", "EOF", "$end"]);
        assert_eq!(kinds("ishikawa\n\n  %% c\nA\n"), ["ISHIKAWA", "SPACELINE", "NL", "TEXT", "NL", "EOF", "$end"]);
        assert_eq!(kinds("ishikawa\n \n A\n"), ["ISHIKAWA", "SPACELINE", "SPACELIST", "TEXT", "NL", "EOF", "$end"]);
    }
}
