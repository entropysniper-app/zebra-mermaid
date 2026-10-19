//! Timeline lexer. Ported from Pele `src/diagrams/timeline/lexer.ts`, which reproduces the
//! tokens of Mermaid's Jison lexer. Pele counts UTF-16 code units; offsets here are bytes. The
//! scans stop only at ASCII characters, so every offset they reach is a character boundary.

use crate::diagrams::common::scan::{accessibility, keyword_at, line_end, skip_trivia, Tokens};
use crate::util::chars::{char_at, is_space, is_word_byte, next_boundary};

pub const END: usize = 0;
pub const EOF: usize = 1;
pub const INVALID: usize = 2;
pub const NEWLINE: usize = 3;
pub const TIMELINE: usize = 4;
pub const TIMELINE_LR: usize = 5;
pub const TIMELINE_TD: usize = 6;
pub const TITLE: usize = 7;
pub const ACC_TITLE: usize = 8;
// The accessibility rule numbers its tokens from ACC_TITLE; the parser reads the values by position.
#[allow(dead_code)]
pub const ACC_TITLE_VALUE: usize = 9;
pub const ACC_DESCR: usize = 10;
#[allow(dead_code)]
pub const ACC_DESCR_VALUE: usize = 11;
pub const ACC_DESCR_MULTILINE_VALUE: usize = 12;
pub const SECTION: usize = 13;
pub const PERIOD: usize = 14;
pub const EVENT: usize = 15;

pub const TOKEN_NAMES: [&str; 16] = [
    "$end",
    "EOF",
    "INVALID",
    "NEWLINE",
    "timeline",
    "timeline_lr",
    "timeline_td",
    "title",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "section",
    "period",
    "event",
];

fn byte(src: &str, p: usize) -> Option<u8> {
    src.as_bytes().get(p).copied()
}

/// `isSpace(src.charCodeAt(p))`: the length of the white space character at `p`, if there is
/// one. Every such character is one UTF-16 unit, so skipping it is Pele's `+ 1`.
fn space_at(src: &str, p: usize) -> Option<usize> {
    char_at(src, p).filter(|&c| is_space(c)).map(char::len_utf8)
}

fn emit(out: &mut Tokens, src: &str, p: &mut usize, kind: usize, e: usize) {
    out.types.push(kind);
    out.texts.push(src[*p..e].to_string());
    out.starts.push(*p);
    *p = e;
}

pub fn tokenize(src: &str) -> Tokens {
    let n = src.len();
    let b = src.as_bytes();
    let mut out = Tokens::default();
    let mut p = 0;

    loop {
        p = skip_trivia(src, p);
        if p >= n {
            break;
        }
        let c = b[p];
        if c == b'\n' {
            let mut e = p + 1;
            while byte(src, e) == Some(b'\n') {
                e += 1;
            }
            emit(&mut out, src, &mut p, NEWLINE, e);
            continue;
        }
        if keyword_at(src, p, "timeline") {
            let mut e = p + 8;
            while matches!(byte(src, e), Some(b' ' | b'\t')) {
                e += 1;
            }
            let lr = keyword_at(src, e, "lr");
            if e > p + 8 && (lr || keyword_at(src, e, "td")) && !byte(src, e + 2).is_some_and(is_word_byte) {
                emit(&mut out, src, &mut p, if lr { TIMELINE_LR } else { TIMELINE_TD }, e + 2);
                continue;
            }
            if !byte(src, p + 8).is_some_and(is_word_byte) {
                let e = p + 8;
                emit(&mut out, src, &mut p, TIMELINE, e);
                continue;
            }
        }
        // `\s` after a keyword also takes a line break, so `title` alone on a line takes the
        // next line as its text.
        if keyword_at(src, p, "title") {
            if let Some(len) = space_at(src, p + 5) {
                let q = p + 5 + len;
                if q < n && b[q] != b'\n' {
                    let e = line_end(src, q);
                    emit(&mut out, src, &mut p, TITLE, e);
                    continue;
                }
            }
        }
        let e = accessibility(src, p, &mut out, ACC_TITLE);
        if e != p {
            p = e;
            continue;
        }
        if keyword_at(src, p, "section") {
            if let Some(len) = space_at(src, p + 7) {
                let q = p + 7 + len;
                let mut e = q;
                while e < n && b[e] != b':' && b[e] != b'\n' {
                    e += 1;
                }
                if e > q {
                    emit(&mut out, src, &mut p, SECTION, e);
                    continue;
                }
            }
        }
        if c != b':' {
            // The first character is taken whatever it is, a character outside the BMP whole.
            let mut e = next_boundary(src, p);
            while e < n && !matches!(b[e], b'#' | b':' | b'\n') {
                e += 1;
            }
            emit(&mut out, src, &mut p, PERIOD, e);
            continue;
        }
        // An event runs to the end of the line or to a colon that is followed by whitespace.
        if let Some(len) = space_at(src, p + 1) {
            let q = p + 1 + len;
            let mut e = q;
            while e < n {
                let k = b[e];
                if k == b'\n' || (k == b':' && space_at(src, e + 1).is_some()) {
                    break;
                }
                e += 1;
            }
            if e > q {
                emit(&mut out, src, &mut p, EVENT, e);
                continue;
            }
        }
        let e = p + 1;
        emit(&mut out, src, &mut p, INVALID, e);
    }

    // Jison marks its lexer done when a rule runs on empty input, so an accessibility value
    // that matches nothing at the very end of the text is followed by no EOF token.
    if out.starts.last().is_none_or(|&s| s < n) {
        emit(&mut out, src, &mut p, EOF, n);
    }
    emit(&mut out, src, &mut p, END, n);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<&'static str> {
        tokenize(src).types.iter().map(|&t| TOKEN_NAMES[t]).collect()
    }

    #[test]
    fn statements() {
        assert_eq!(kinds("timeline TD\n"), ["timeline_td", "NEWLINE", "EOF", "$end"]);
        assert_eq!(kinds("timeline TDx\n"), ["timeline", "period", "NEWLINE", "EOF", "$end"]);
        assert_eq!(kinds("timeline\n2020 : a : b\n"), ["timeline", "NEWLINE", "period", "event", "event", "NEWLINE", "EOF", "$end"]);
        assert_eq!(kinds("timeline\nsection A: b\n"), ["timeline", "NEWLINE", "section", "event", "NEWLINE", "EOF", "$end"]);
        // A colon that no white space follows stays in the event.
        let t = tokenize("timeline\n2020 : a:b\n");
        assert_eq!(t.texts[3], ": a:b");
    }

    #[test]
    fn wide_spaces_and_characters() {
        // U+3000 is one UTF-16 unit and three bytes.
        let t = tokenize("timeline\ntitle\u{3000}T\nsection\u{3000}S\n😀x : e\n");
        assert_eq!(t.texts[2], "title\u{3000}T");
        assert_eq!(t.texts[4], "section\u{3000}S");
        assert_eq!(t.texts[6], "😀x ");
        assert_eq!(t.types[7], EVENT);
    }

    #[test]
    fn title_alone_takes_the_next_line() {
        let t = tokenize("timeline\ntitle\nNext\n");
        assert_eq!(t.types[2], TITLE);
        assert_eq!(t.texts[2], "title\nNext");
    }

    #[test]
    fn accessibility_value_at_the_end_has_no_eof() {
        assert_eq!(kinds("timeline\naccTitle:   \n"), ["timeline", "NEWLINE", "acc_title", "acc_title_value", "$end"]);
    }
}
