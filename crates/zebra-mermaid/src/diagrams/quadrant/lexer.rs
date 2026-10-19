//! The quadrant chart lexer: Mermaid's Jison lexer for `quadrantChart`, written out by hand.
//! Ported from Pele `src/diagrams/quadrant/lexer.ts`.
//!
//! Pele reads the text in UTF-16 code units; this reads it in bytes. Every delimiter is ASCII,
//! and a run of characters outside ASCII is a run of bytes outside ASCII, so bytes answer as
//! code units do. Where Pele tests a character that may lie outside ASCII (white space, the
//! character before a `%%` comment, the "any character" of a coordinate), the whole character
//! is decoded, and one outside the Basic Multilingual Plane counts as two code units.

use super::tokens::*;
use crate::diagrams::common::caseless::word_at;
use crate::util::chars::{char_at, is_line_end, is_space, is_word_byte};

/// The tokens of a text: their types, and where each starts and ends (in bytes).
#[derive(Debug, Clone, Default)]
pub struct Tokens {
    pub types: Vec<u8>,
    pub starts: Vec<usize>,
    pub ends: Vec<usize>,
}

// Lexer states (Jison start conditions).
const INITIAL: usize = 0;
const S_TITLE: usize = 1;
const S_ACC_TITLE: usize = 2;
const S_ACC_DESCR: usize = 3;
const S_ACC_DESCR_MULTILINE: usize = 4;
const S_STRING: usize = 5;
const S_MD_STRING: usize = 6;
const S_CLASS_NAME: usize = 7;
const S_POINT_START: usize = 8;
const S_POINT_X: usize = 9;
const S_POINT_Y: usize = 10;

/// The token each state gives for the empty rest of the input, where it gives one.
const AT_END: [u8; 6] = [EOF, TITLE_VALUE, ACC_TITLE_VALUE, ACC_DESCR_VALUE, ACC_DESCR_MULTILINE_VALUE, STR];

/// The single-character tokens, by ASCII code; 0 for none.
fn single(c: u8) -> u8 {
    match c {
        b':' => COLON,
        b'+' => PLUS,
        b',' => COMMA,
        b'=' => EQUALS,
        b'*' => MULT,
        b'#' => BRKT,
        b'_' => UNDERSCORE,
        b'.' => DOT,
        b'&' => AMP,
        b'-' => MINUS,
        b' ' | b'\t' | 0x0b | 0x0c => SPACE,
        b';' => SEMI,
        b'!' | b'$' | b'%' | b'\'' | b'`' | b'?' | b'\\' | b'/' => PUNCTUATION,
        _ => 0,
    }
}

/// The length of the white space character at byte `q`, if there is one.
fn space_at(src: &str, q: usize) -> Option<usize> {
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
    // The last run scanned of each kind, so that a long run is not scanned again from each of
    // its characters.
    let mut space_from = 0usize;
    let mut space_to = 0usize;
    let mut dash_from = 0usize;
    let mut dash_to = 0usize;
    let mut ws_from = 0usize;
    let mut ws_to = 0usize;
    let mut gap_from = 0usize;
    let mut gap_to = 0usize;

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
    let byte = |q: usize| -> Option<u8> { b.get(q).copied() };
    let is_digit = |q: usize| byte(q).is_some_and(|c| c.is_ascii_digit());
    let line_end = |q: usize| -> usize {
        if q >= n {
            return n;
        }
        src[q..].find('\n').map_or(n, |k| q + k)
    };
    let spaces = |mut q: usize| -> usize {
        while byte(q) == Some(32) {
            q += 1;
        }
        q
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
    // Mermaid's `(1)|(0(.\d+)?)`, where the dot is any character within a line. A character
    // outside the Basic Multilingual Plane is two code units, so no digit follows its first.
    let coordinate = |q: usize| -> Option<usize> {
        match byte(q) {
            Some(b'1') => return Some(q + 1),
            Some(b'0') => {}
            _ => return None,
        }
        let mut e = q + 1;
        if let Some(ch) = char_at(src, e) {
            if !is_line_end(ch) && (ch as u32) <= 0xffff && is_digit(e + ch.len_utf8()) {
                e += ch.len_utf8() + 1;
                while is_digit(e) {
                    e += 1;
                }
            }
        }
        Some(e)
    };

    'lexing: while p < n {
        let c = b[p];
        let next = byte(p + 1);

        match state {
            S_TITLE | S_ACC_TITLE | S_ACC_DESCR => {
                let e = line_end(p);
                p = emit(&mut out, AT_END[state], p, e);
                pop(&mut stack, &mut state);
                continue;
            }
            S_ACC_DESCR_MULTILINE => {
                if c == b'}' {
                    p += 1;
                    pop(&mut stack, &mut state);
                } else {
                    let e = src[p..].find('}').map_or(n, |k| p + k);
                    p = emit(&mut out, ACC_DESCR_MULTILINE_VALUE, p, e);
                }
                continue;
            }
            S_STRING => {
                if c == b'"' {
                    p += 1;
                    pop(&mut stack, &mut state);
                } else {
                    let e = src[p..].find('"').map_or(n, |k| p + k);
                    p = emit(&mut out, STR, p, e);
                }
                continue;
            }
            S_MD_STRING => {
                if c == b'`' && next == Some(b'"') {
                    p += 2;
                    pop(&mut stack, &mut state);
                    continue;
                }
                if c != b'`' && c != b'"' {
                    let mut e = p + 1;
                    while e < n && b[e] != b'`' && b[e] != b'"' {
                        e += 1;
                    }
                    p = emit(&mut out, MD_STR, p, e);
                    continue;
                }
                break 'lexing;
            }
            S_CLASS_NAME => {
                if is_word_byte(c) {
                    let mut e = p + 1;
                    while byte(e).is_some_and(is_word_byte) {
                        e += 1;
                    }
                    p = emit(&mut out, CLASS_NAME, p, e);
                    pop(&mut stack, &mut state);
                    continue;
                }
                break 'lexing;
            }
            S_POINT_START => {
                if let Some(e) = coordinate(p) {
                    p = emit(&mut out, POINT_X, p, e);
                    push(&mut stack, &mut state, S_POINT_X);
                    continue;
                }
                let k = run(p);
                if byte(k) == Some(b']') {
                    p = spaces(k + 1);
                    pop(&mut stack, &mut state);
                    continue;
                }
                break 'lexing;
            }
            S_POINT_X => {
                let k = run(p);
                if byte(k) == Some(b',') {
                    p = run(k + 1);
                    state = S_POINT_Y;
                    continue;
                }
                break 'lexing;
            }
            S_POINT_Y => {
                if let Some(e) = coordinate(p) {
                    p = emit(&mut out, POINT_Y, p, e);
                    pop(&mut stack, &mut state);
                    continue;
                }
                break 'lexing;
            }
            _ => {}
        }

        if c == b'%' && next == Some(b'%') {
            p = line_end(p + 2);
            continue;
        }
        // Mermaid's second comment rule takes any character before `%%` with it, a line break
        // included. The code unit after a character outside the Basic Multilingual Plane is its
        // low surrogate, never `%`.
        if c != b'}' {
            if let Some(ch) = char_at(src, p) {
                let at = p + ch.len_utf8();
                if (ch as u32) <= 0xffff && byte(at) == Some(b'%') && byte(at + 1) == Some(b'%') {
                    p = line_end(at + 2);
                    continue;
                }
            }
        }
        if c == 10 || c == 13 {
            let mut e = p + 1;
            while matches!(byte(e), Some(10) | Some(13)) {
                e += 1;
            }
            p = emit(&mut out, NEWLINE, p, e);
            continue;
        }

        let lower = c | 32;
        if lower == b't' && word_at(src, p, "title") && !byte(p + 5).is_some_and(is_word_byte) {
            p = emit(&mut out, TITLE, p, p + 5);
            push(&mut stack, &mut state, S_TITLE);
            continue;
        }
        if lower == b'a' && word_at(src, p, "acc") {
            let descr = word_at(src, p + 3, "descr");
            if descr || word_at(src, p + 3, "title") {
                let k = run(p + 8);
                let d = byte(k);
                if d == Some(b':') {
                    let e = run(k + 1);
                    p = emit(&mut out, if descr { ACC_DESCR } else { ACC_TITLE }, p, e);
                    push(&mut stack, &mut state, if descr { S_ACC_DESCR } else { S_ACC_TITLE });
                    continue;
                }
                if d == Some(b'{') && descr {
                    p = run(k + 1);
                    push(&mut stack, &mut state, S_ACC_DESCR_MULTILINE);
                    continue;
                }
            }
        }

        // Axis, quadrant and chart keywords and the axis arrow take the spaces around them.
        let mut j = p;
        if c == 32 {
            if p < space_from || p >= space_to {
                space_from = p;
                space_to = spaces(p);
            }
            j = space_to;
        }
        let d = byte(j);
        let dl = d.map(|d| d | 32);
        let mut kind = 0u8;
        let mut end = 0usize;
        if dl == Some(b'x') && word_at(src, j, "x-axis") {
            kind = X_AXIS;
            end = j + 6;
        } else if dl == Some(b'y') && word_at(src, j, "y-axis") {
            kind = Y_AXIS;
            end = j + 6;
        } else if d == Some(b'-') {
            if j < dash_from || j >= dash_to {
                dash_from = j;
                dash_to = j;
                while byte(dash_to) == Some(b'-') {
                    dash_to += 1;
                }
            }
            if dash_to - j >= 2 && byte(dash_to) == Some(b'>') {
                kind = DELIMITER;
                end = dash_to + 1;
            }
        } else if dl == Some(b'q') && word_at(src, j, "quadrant") {
            let q = byte(j + 9);
            if byte(j + 8) == Some(b'-') && q.is_some_and(|q| (b'1'..=b'4').contains(&q)) {
                kind = QUADRANT_1 + (q.unwrap_or(b'1') - b'1');
                end = j + 10;
            } else if word_at(src, j + 8, "chart") {
                kind = QUADRANT;
                end = j + 13;
            }
        }
        if kind != 0 {
            let e = spaces(end);
            p = emit(&mut out, kind, p, e);
            continue;
        }

        if lower == b'c' && word_at(src, p, "classdef") && !byte(p + 8).is_some_and(is_word_byte) {
            p = emit(&mut out, CLASSDEF, p, p + 8);
            continue;
        }
        if c == b'"' {
            if next == Some(b'`') {
                p += 2;
                push(&mut stack, &mut state, S_MD_STRING);
            } else {
                p += 1;
                push(&mut stack, &mut state, S_STRING);
            }
            continue;
        }
        if c == b':' && next == Some(b':') && byte(p + 2) == Some(b':') {
            p += 3;
            push(&mut stack, &mut state, S_CLASS_NAME);
            continue;
        }
        // A point's `: [`, with any whitespace before, between and after, line breaks included.
        if c == b':' || space_at(src, p).is_some() {
            let mut k = p;
            if c != b':' {
                if p < ws_from || p >= ws_to {
                    ws_from = p;
                    ws_to = run(p);
                }
                k = ws_to;
            }
            if byte(k) == Some(b':') {
                if k + 1 < gap_from || k + 1 >= gap_to {
                    gap_from = k + 1;
                    gap_to = run(k + 1);
                }
                if byte(gap_to) == Some(b'[') {
                    let e = run(gap_to + 1);
                    p = emit(&mut out, POINT_START, p, e);
                    push(&mut stack, &mut state, S_POINT_START);
                    continue;
                }
            }
        }

        if c.is_ascii_alphabetic() {
            let mut e = p + 1;
            while byte(e).is_some_and(|c| c.is_ascii_alphabetic()) {
                e += 1;
            }
            p = emit(&mut out, ALPHA, p, e);
            continue;
        }
        if c >= 128 {
            let mut e = p + 1;
            while byte(e).is_some_and(|c| c >= 128) {
                e += 1;
            }
            p = emit(&mut out, UNICODE_TEXT, p, e);
            continue;
        }
        if c.is_ascii_digit() {
            let mut e = p + 1;
            while is_digit(e) {
                e += 1;
            }
            p = emit(&mut out, NUM, p, e);
            continue;
        }
        if single(c) != 0 {
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
    if state < AT_END.len() {
        out.types.push(AT_END[state]);
        out.starts.push(n);
        out.ends.push(n);
    }
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
    fn points_and_axes() {
        assert_eq!(
            kinds("quadrantChart\n  x-axis Low --> High\n  A:::c1: [0.3, 1]\n"),
            vec![
                QUADRANT,
                NEWLINE,
                X_AXIS,
                ALPHA,
                DELIMITER,
                ALPHA,
                NEWLINE,
                SPACE,
                SPACE,
                ALPHA,
                CLASS_NAME,
                POINT_START,
                POINT_X,
                POINT_Y,
                NEWLINE,
                EOF,
                END
            ]
        );
    }

    #[test]
    fn coordinates_take_any_character_within_a_line() {
        let t = tokenize("p: [0é5, 0😀5]\n");
        let src = "p: [0é5, 0😀5]\n";
        assert_eq!(t.types[2], POINT_X);
        assert_eq!(&src[t.starts[2]..t.ends[2]], "0é5");
        // The character outside the plane is two code units: the coordinate is "0" alone,
        // and the lexer stops at what follows.
        assert_eq!(t.types[3], POINT_Y);
        assert_eq!(&src[t.starts[3]..t.ends[3]], "0");
        assert_eq!(*t.types.last().unwrap(), ERROR);
    }

    #[test]
    fn long_runs_are_scanned_once() {
        let src = format!("quadrantChart\n{}x\n", " ".repeat(200_000));
        let t = tokenize(&src);
        assert_eq!(t.types.len(), 2 + 200_000 + 4);
    }

    #[test]
    fn comments_take_the_character_before() {
        assert_eq!(kinds("quadrantChart\né%% x\n"), vec![QUADRANT, NEWLINE, NEWLINE, EOF, END]);
        assert_eq!(kinds("quadrantChart\n😀%% x\n"), vec![QUADRANT, NEWLINE, UNICODE_TEXT, NEWLINE, EOF, END]);
    }
}
