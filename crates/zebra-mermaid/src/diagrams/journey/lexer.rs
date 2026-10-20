//! The journey lexer, ported from Pele `src/diagrams/journey/lexer.ts` (itself a port of the
//! compiled Jison rules). Offsets are bytes; every character the rules test is ASCII or a
//! white space character of the Basic Multilingual Plane, one UTF-16 unit in Pele.

use crate::diagrams::common::scan::{accessibility, keyword_at, skip_trivia, Tokens};
use crate::util::chars::{char_at, is_space, is_word};

pub const END: usize = 0;
pub const EOF: usize = 1;
pub const INVALID: usize = 2;
pub const NEWLINE: usize = 3;
pub const JOURNEY: usize = 4;
pub const TITLE: usize = 5;
pub const ACC_TITLE: usize = 6;
// The value tokens are read through the token before them; the numbers keep their places.
#[allow(dead_code)]
pub const ACC_TITLE_VALUE: usize = 7;
pub const ACC_DESCR: usize = 8;
#[allow(dead_code)]
pub const ACC_DESCR_VALUE: usize = 9;
pub const ACC_DESCR_MULTILINE_VALUE: usize = 10;
pub const SECTION: usize = 11;
pub const TASK_NAME: usize = 12;
pub const TASK_DATA: usize = 13;
pub const COLON: usize = 14;

pub const TOKEN_NAMES: [&str; 15] = [
    "$end",
    "EOF",
    "INVALID",
    "NEWLINE",
    "journey",
    "title",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "section",
    "taskName",
    "taskData",
    ":",
];

/// `#`, a line break, and `;` end every kind of text; a colon also ends a task name or a
/// section. All are ASCII, so a byte scan finds what the code unit scan finds.
fn text_end(src: &str, from: usize, colon: bool) -> usize {
    let b = src.as_bytes();
    let mut e = from;
    while e < b.len() {
        let c = b[e];
        if c == b'#' || c == b'\n' || c == b';' || (colon && c == b':') {
            break;
        }
        e += 1;
    }
    e
}

/// The white space character at `p`, as `isSpace(src.charCodeAt(p))` tests it, with its
/// length in bytes. Every such character is one UTF-16 unit, so Pele's `p + 1` after it is
/// the byte after it here.
fn space_at(src: &str, p: usize) -> Option<usize> {
    char_at(src, p).filter(|&c| is_space(c)).map(char::len_utf8)
}

#[cfg(test)]
pub fn tokenize(src: &str) -> Tokens {
    tokenize_into(src)
}

pub(super) fn tokenize_borrowed(src: &str) -> Tokens<&str> {
    tokenize_into(src)
}

fn tokenize_into<'s, T: From<&'s str> + Default>(src: &'s str) -> Tokens<T> {
    let n = src.len();
    let mut out = Tokens::default();
    let mut p = 0;

    fn emit<'s, T: From<&'s str>>(out: &mut Tokens<T>, src: &'s str, p: &mut usize, kind: usize, e: usize) {
        out.types.push(kind);
        out.texts.push(src[*p..e].into());
        out.starts.push(*p);
        *p = e;
    }

    loop {
        p = skip_trivia(src, p);
        if p >= n {
            break;
        }
        let c = src.as_bytes()[p];
        if c == b'\n' {
            let mut e = p + 1;
            while src.as_bytes().get(e) == Some(&b'\n') {
                e += 1;
            }
            emit(&mut out, src, &mut p, NEWLINE, e);
            continue;
        }
        if keyword_at(src, p, "journey") && !char_at(src, p + 7).is_some_and(is_word) {
            let e = p + 7;
            emit(&mut out, src, &mut p, JOURNEY, e);
            continue;
        }
        // `\s` after a keyword also takes a line break, so `title` alone on a line takes the
        // next line as its text.
        if keyword_at(src, p, "title") {
            if let Some(len) = space_at(src, p + 5) {
                let from = p + 5 + len;
                let e = text_end(src, from, false);
                if e > from {
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
                let from = p + 7 + len;
                let e = text_end(src, from, true);
                if e > from {
                    emit(&mut out, src, &mut p, SECTION, e);
                    continue;
                }
            }
        }
        if c == b':' {
            let e = text_end(src, p + 1, false);
            if e > p + 1 {
                emit(&mut out, src, &mut p, TASK_DATA, e);
            } else {
                let e = p + 1;
                emit(&mut out, src, &mut p, COLON, e);
            }
        } else if c == b';' {
            let e = p + 1;
            emit(&mut out, src, &mut p, INVALID, e);
        } else {
            // Pele scans on from the next code unit. After a character outside the Basic
            // Multilingual Plane that is its low surrogate, which ends nothing, so scanning
            // from the next character finds the same end.
            let next = p + char_at(src, p).map_or(1, char::len_utf8);
            let e = text_end(src, next, true);
            emit(&mut out, src, &mut p, TASK_NAME, e);
        }
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

    #[test]
    fn tokens() {
        let t = tokenize("journey\ntitle My day\nsection Go\u{3000}x: y\n  Task: 5: Me\n");
        assert_eq!(t.types, vec![JOURNEY, NEWLINE, TITLE, NEWLINE, SECTION, TASK_DATA, NEWLINE, TASK_NAME, TASK_DATA, NEWLINE, EOF, END]);
        assert_eq!(t.texts[2], "title My day");
        assert_eq!(t.texts[4], "section Go\u{3000}x");
        assert_eq!(t.texts[5], ": y");
        // A section keyword with a wide space before its text.
        let t = tokenize("journey\nsection\u{3000}A:\n");
        assert_eq!(t.types[2], SECTION);
        assert_eq!(t.texts[2], "section\u{3000}A");
        // `title` alone on a line takes the next line.
        let t = tokenize("journey\ntitle\nfoo\n");
        assert_eq!(t.types[2], TITLE);
        assert_eq!(t.texts[2], "title\nfoo");
        // An unclosed accDescr block at the end is followed by no EOF.
        let t = tokenize("journey\naccDescr {\nabc");
        assert_eq!(t.types[t.types.len() - 2], ACC_DESCR_MULTILINE_VALUE);
        assert_eq!(*t.types.last().unwrap(), END);
        let t = tokenize("journeys");
        assert_eq!(t.types[0], TASK_NAME);
    }
}
