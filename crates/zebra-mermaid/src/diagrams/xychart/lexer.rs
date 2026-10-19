//! The XY chart lexer: Mermaid's Jison lexer for `xychart`, written out by hand. Ported from
//! Pele `src/diagrams/xychart/lexer.ts`.
//!
//! Pele reads the text in UTF-16 code units; this reads it in bytes. Every test against a
//! delimiter is ASCII, so a byte answers as the code unit does. Where Pele tests a character
//! that may lie outside ASCII (white space, the character before a `%%` comment), the whole
//! character is decoded, and a character outside the Basic Multilingual Plane is seen as its
//! high surrogate followed by its low one, as `charCodeAt` sees it.

use std::sync::LazyLock;

use super::tokens::*;
use crate::diagrams::common::caseless::word_at;
use crate::util::chars::{char_at, is_space, is_word_byte};

/// The tokens of a text: their types, and where each starts and ends (in bytes).
#[derive(Debug, Clone, Default)]
pub struct Tokens {
    pub types: Vec<u8>,
    pub starts: Vec<usize>,
    pub ends: Vec<usize>,
}

// Lexer states (Jison start conditions).
const INITIAL: usize = 0;
const AXIS_DATA: usize = 1;
const AXIS_BAND_DATA: usize = 2;
const DATA: usize = 3;
const DATA_INNER: usize = 4;
const STRING: usize = 5;
const S_ACC_TITLE: usize = 6;
const S_ACC_DESCR: usize = 7;
const S_ACC_DESCR_MULTILINE: usize = 8;

/// The token each state gives for the empty rest of the input.
const AT_END: [u8; 9] = [EOF, EOF, EOF, EOF, EOF, STR, ACC_TITLE_VALUE, ACC_DESCR_VALUE, ACC_DESCR_MULTILINE_VALUE];

/// Mermaid's grammar means to open a markdown string at `"` followed by a backtick, but its
/// quoting turns the rule into one that matches this text and leaves the current state.
/// Markdown strings are never read.
static BROKEN_RULE: LazyLock<String> = LazyLock::new(|| {
    format!(
        "`){}{{ this.pushstate(md_string); }}\n<md_string>(?:(?!`\").)+{}{{ return md_str; }}\n<md_string>(?:`",
        " ".repeat(36),
        " ".repeat(18)
    )
});

/// The single-character tokens, by ASCII code; 0 for none.
fn single(c: u32) -> u8 {
    match c {
        0x3a => COLON,      // :
        0x2b => PLUS,       // +
        0x2c => COMMA,      // ,
        0x3d => EQUALS,     // =
        0x2a => MULT,       // *
        0x23 => BRKT,       // #
        0x5f => UNDERSCORE, // _
        0x2e => DOT,        // .
        0x26 => AMP,        // &
        0x2d => MINUS,      // -
        0x3b => SEMI,       // ;
        _ => 0,
    }
}

/// The code unit `charCodeAt` reads at the character that starts at byte `q`: the character
/// itself in the Basic Multilingual Plane, its high surrogate past it. `None` past the end.
pub(crate) fn unit(src: &str, q: usize) -> Option<u32> {
    let c = char_at(src, q)? as u32;
    Some(if c > 0xffff { 0xd800 + ((c - 0x10000) >> 10) } else { c })
}

/// The character at `p` as a code unit, and the code unit after it with its byte position. A
/// character outside the Basic Multilingual Plane is two units, so the next one is its low
/// surrogate, which no test here accepts; it is given as 0xdc00.
pub(crate) fn unit_and_next(src: &str, p: usize) -> (u32, Option<u32>, usize) {
    match char_at(src, p) {
        None => (0, None, p),
        Some(ch) => {
            let c = ch as u32;
            if c > 0xffff {
                (0xd800 + ((c - 0x10000) >> 10), Some(0xdc00), p + ch.len_utf8())
            } else {
                let at = p + ch.len_utf8();
                (c, unit(src, at), at)
            }
        }
    }
}

fn is_digit_at(b: &[u8], q: usize) -> bool {
    b.get(q).is_some_and(u8::is_ascii_digit)
}

/// `isSpace` on the character at byte `q`; false past the end.
pub(crate) fn space_at(src: &str, q: usize) -> Option<usize> {
    let ch = char_at(src, q)?;
    is_space(ch).then(|| ch.len_utf8())
}

pub fn tokenize(src: &str) -> Tokens {
    let b = src.as_bytes();
    let n = b.len();
    let mut out = Tokens::default();
    let mut stack: Vec<usize> = Vec::new();
    let mut state = INITIAL;
    let mut p = 0usize;

    fn emit(out: &mut Tokens, t: u8, s: usize, e: usize) -> usize {
        out.types.push(t);
        out.starts.push(s);
        out.ends.push(e);
        e
    }
    fn push(stack: &mut Vec<usize>, state: &mut usize, s: usize) {
        stack.push(*state);
        *state = s;
    }
    fn pop(stack: &mut Vec<usize>, state: &mut usize) {
        if let Some(s) = stack.pop() {
            *state = s;
        }
    }
    let line_end = |q: usize| -> usize {
        if q >= n {
            return n;
        }
        src[q..].find('\n').map_or(n, |k| q + k)
    };
    let run = |mut q: usize| -> usize {
        while q < n {
            match space_at(src, q) {
                Some(len) => q += len,
                None => break,
            }
        }
        q
    };
    let digits = |mut q: usize| -> usize {
        while is_digit_at(b, q) {
            q += 1;
        }
        q
    };
    let keyword = |p: usize, word: &str| -> bool { word_at(src, p, word) && !b.get(p + word.len()).is_some_and(|&c| is_word_byte(c)) };

    while p < n {
        let (c, next, next_at) = unit_and_next(src, p);

        if state == STRING {
            if c == 34 {
                p += 1;
                pop(&mut stack, &mut state);
            } else {
                let e = src[p..].find('"').map_or(n, |k| p + k);
                p = emit(&mut out, STR, p, e);
            }
            continue;
        }
        if state == S_ACC_TITLE || state == S_ACC_DESCR {
            let e = line_end(p);
            p = emit(&mut out, AT_END[state], p, e);
            pop(&mut stack, &mut state);
            continue;
        }
        if state == S_ACC_DESCR_MULTILINE {
            if c == 125 {
                p += 1;
                pop(&mut stack, &mut state);
            } else {
                let e = src[p..].find('}').map_or(n, |k| p + k);
                p = emit(&mut out, ACC_DESCR_MULTILINE_VALUE, p, e);
            }
            continue;
        }

        if c == 37 && next == Some(37) {
            p = line_end(p + 2);
            continue;
        }
        // Mermaid's second comment rule takes any character before `%%` with it, a line break
        // included.
        if c != 125 && next == Some(37) && b.get(next_at + 1) == Some(&b'%') {
            p = line_end(next_at + 2);
            continue;
        }
        if c == 10 || c == 13 {
            // Only a line break that directly follows an axis or a series ends it; one after
            // spaces is skipped with them, and the state carries on to the next line.
            if (state == AXIS_DATA || state == DATA) && (c == 10 || next == Some(10)) {
                p = emit(&mut out, NEWLINE, p, if c == 10 { p + 1 } else { p + 2 });
                pop(&mut stack, &mut state);
                continue;
            }
            let mut e = p + 1;
            while matches!(b.get(e), Some(&10) | Some(&13)) {
                e += 1;
            }
            p = emit(&mut out, NEWLINE, p, e);
            continue;
        }

        match c | 32 {
            116 if keyword(p, "title") => {
                p = emit(&mut out, TITLE, p, p + 5);
                continue;
            }
            97 if word_at(src, p, "acc") => {
                let descr = word_at(src, p + 3, "descr");
                if descr || word_at(src, p + 3, "title") {
                    let k = run(p + 8);
                    let d = b.get(k).copied();
                    if d == Some(58) {
                        let e = run(k + 1);
                        p = emit(&mut out, if descr { ACC_DESCR } else { ACC_TITLE }, p, e);
                        push(&mut stack, &mut state, if descr { S_ACC_DESCR } else { S_ACC_TITLE });
                        continue;
                    }
                    if d == Some(123) && descr {
                        p = run(k + 1);
                        push(&mut stack, &mut state, S_ACC_DESCR_MULTILINE);
                        continue;
                    }
                }
            }
            120 => {
                if keyword(p, "xychart-beta") {
                    p = emit(&mut out, XYCHART, p, p + 12);
                    continue;
                }
                if keyword(p, "xychart") {
                    p = emit(&mut out, XYCHART, p, p + 7);
                    continue;
                }
                if keyword(p, "x-axis") {
                    p = emit(&mut out, X_AXIS, p, p + 6);
                    push(&mut stack, &mut state, AXIS_DATA);
                    continue;
                }
            }
            121 if keyword(p, "y-axis") => {
                p = emit(&mut out, Y_AXIS, p, p + 6);
                push(&mut stack, &mut state, AXIS_DATA);
                continue;
            }
            118 if word_at(src, p, "vertical") => {
                p = emit(&mut out, CHART_ORIENTATION, p, p + 8);
                continue;
            }
            104 if word_at(src, p, "horizontal") => {
                p = emit(&mut out, CHART_ORIENTATION, p, p + 10);
                continue;
            }
            108 if keyword(p, "line") => {
                p = emit(&mut out, LINE, p, p + 4);
                push(&mut stack, &mut state, DATA);
                continue;
            }
            98 if keyword(p, "bar") => {
                p = emit(&mut out, BAR, p, p + 3);
                push(&mut stack, &mut state, DATA);
                continue;
            }
            _ => {}
        }

        if c == 91 {
            if state == AXIS_DATA {
                push(&mut stack, &mut state, AXIS_BAND_DATA);
            } else if state == DATA {
                push(&mut stack, &mut state, DATA_INNER);
            }
            p = emit(&mut out, SQUARE_BRACES_START, p, p + 1);
            continue;
        }
        if c == 93 {
            if state == DATA_INNER || state == AXIS_BAND_DATA {
                pop(&mut stack, &mut state);
            }
            p = emit(&mut out, SQUARE_BRACES_END, p, p + 1);
            continue;
        }
        if state == AXIS_DATA && c == 45 && next == Some(45) && b.get(p + 2) == Some(&62) {
            p = emit(&mut out, ARROW_DELIMITER, p, p + 3);
            continue;
        }
        if state == AXIS_DATA || state == DATA_INNER {
            let q = if c == 43 || c == 45 { p + 1 } else { p };
            let d = b.get(q).copied();
            let mut e = None;
            if d.is_some_and(|d| d.is_ascii_digit()) {
                let mut end = digits(q + 1);
                if b.get(end) == Some(&46) && is_digit_at(b, end + 1) {
                    end = digits(end + 2);
                }
                e = Some(end);
            } else if d == Some(46) && is_digit_at(b, q + 1) {
                e = Some(digits(q + 2));
            }
            if let Some(e) = e {
                p = emit(&mut out, NUMBER_WITH_DECIMAL, p, e);
                continue;
            }
        }
        if c == 96 && word_at(src, p, &BROKEN_RULE) {
            p += BROKEN_RULE.len();
            pop(&mut stack, &mut state);
            continue;
        }
        if c == 34 {
            p += 1;
            push(&mut stack, &mut state, STRING);
            continue;
        }
        if c < 128 && (c as u8).is_ascii_alphabetic() {
            let mut e = p + 1;
            while b.get(e).is_some_and(u8::is_ascii_alphabetic) {
                e += 1;
            }
            p = emit(&mut out, ALPHA, p, e);
            continue;
        }
        if c < 128 && (c as u8).is_ascii_digit() {
            let e = digits(p + 1);
            p = emit(&mut out, NUM, p, e);
            continue;
        }
        if let Some(len) = space_at(src, p) {
            p = run(p + len);
            continue;
        }
        if c < 128 && single(c) != 0 {
            p = emit(&mut out, single(c), p, p + 1);
            continue;
        }
        break;
    }

    if p < n {
        out.types.push(ERROR);
        out.starts.push(p);
        out.ends.push(p);
        return out;
    }
    // Jison tries the rules of the current state once more against the empty rest of the input.
    out.types.push(AT_END[state]);
    out.starts.push(n);
    out.ends.push(n);
    out.types.push(END);
    out.starts.push(n);
    out.ends.push(n);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<u8> {
        tokenize(src).types
    }

    #[test]
    fn the_broken_rule_is_mermaids_text() {
        assert_eq!(BROKEN_RULE.len(), 144);
        assert!(BROKEN_RULE.starts_with("`)    "));
        assert!(BROKEN_RULE.ends_with("{ return md_str; }\n<md_string>(?:`"));
    }

    #[test]
    fn axes_and_series() {
        assert_eq!(
            kinds("xychart-beta\nx-axis [a, \"b\"]\nbar [1.5, -.5]\n"),
            vec![
                XYCHART,
                NEWLINE,
                X_AXIS,
                SQUARE_BRACES_START,
                ALPHA,
                COMMA,
                STR,
                SQUARE_BRACES_END,
                NEWLINE,
                BAR,
                SQUARE_BRACES_START,
                NUMBER_WITH_DECIMAL,
                COMMA,
                NUMBER_WITH_DECIMAL,
                SQUARE_BRACES_END,
                NEWLINE,
                EOF,
                END
            ]
        );
        assert_eq!(kinds("y-axis 1 --> 2\n"), vec![Y_AXIS, NUMBER_WITH_DECIMAL, ARROW_DELIMITER, NUMBER_WITH_DECIMAL, NEWLINE, EOF, END]);
    }

    #[test]
    fn comments_take_the_character_before() {
        // "é%%" is a comment; "😀%%" is not, as the character is two code units.
        assert_eq!(kinds("xychart\né%% x\n"), vec![XYCHART, NEWLINE, NEWLINE, EOF, END]);
        let t = tokenize("xychart\n😀%%\n");
        assert_eq!(t.types, vec![XYCHART, NEWLINE, ERROR]);
        assert_eq!(t.starts[2], 8);
    }

    #[test]
    fn white_space_outside_ascii() {
        assert_eq!(kinds("title\u{a0}\u{3000}a\n"), vec![TITLE, ALPHA, NEWLINE, EOF, END]);
    }
}
