#![allow(non_upper_case_globals)] // Token spelling follows the upstream grammar.
//! Ordered state lexer from Pele requirement/lexer.ts.
use super::tokens::*;
use crate::util::{
    chars::{char_at, is_space, is_word},
    js,
};
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: u8,
    pub text: String,
    pub start: usize,
}
#[derive(Clone, Copy)]
enum State {
    Initial,
    Quoted,
    Style,
    Title,
    Descr,
    Multiline,
}
fn whitespace(s: &str, mut p: usize) -> usize {
    while let Some(c) = char_at(s, p).filter(|&c| is_space(c)) {
        p += c.len_utf8();
    }
    p
}
fn line_break(s: &str, p: usize) -> usize {
    s[p..].find('\n').map_or(s.len(), |i| p + i)
}
pub fn tokenize(src: &str) -> Vec<Token> {
    let lower = src.to_ascii_lowercase();
    let n = src.len();
    let bytes = src.as_bytes();
    let mut dirs: [Vec<(usize, usize)>; 4] = std::array::from_fn(|_| vec![]);
    let mut from = 0;
    while let Some(off) = lower[from..].find("direction") {
        let start = from + off;
        let q = whitespace(src, start + 9);
        if q > start + 9 {
            for (kind, word) in ["tb", "bt", "rl", "lr"].iter().enumerate() {
                if lower[q..].starts_with(word) {
                    dirs[kind].push((start, q + 2));
                    break;
                }
            }
        }
        from = start + 9;
    }
    let mut out = vec![];
    let mut stack = vec![];
    let mut state = State::Initial;
    let mut p = 0;
    let mut cached_end = None;
    let mut cached_dirs: [Option<(usize, usize)>; 4] = [None; 4];
    let mut seen = [0; 4];
    let emit = |out: &mut Vec<Token>, kind: u8, start: usize, end: usize| out.push(Token { kind, text: src[start..end].into(), start });
    while p < n {
        let start = p;
        let c = bytes[p];
        match state {
            State::Quoted => {
                if c == b'"' {
                    state = stack.pop().unwrap_or(State::Initial);
                    p += 1;
                } else {
                    let end = src[p..].find('"').map_or(n, |i| p + i);
                    emit(&mut out, qString, p, end);
                    p = end;
                }
            }
            State::Title | State::Descr => {
                let token = if matches!(state, State::Title) { acc_title_value } else { acc_descr_value };
                state = stack.pop().unwrap_or(State::Initial);
                p = line_break(src, p);
                emit(&mut out, token, start, p);
            }
            State::Multiline => {
                if c == b'}' {
                    state = stack.pop().unwrap_or(State::Initial);
                    p += 1;
                } else {
                    let end = src[p..].find('}').map_or(n, |i| p + i);
                    emit(&mut out, acc_descr_multiline_value, p, end);
                    p = end;
                }
            }
            State::Style => {
                if is_word(c as char) {
                    p += 1;
                    while p < n && is_word(bytes[p] as char) {
                        p += 1;
                    }
                    emit(&mut out, ALPHA, start, p);
                    continue;
                }
                let token = match c {
                    b':' => Some(COLON),
                    b';' => Some(SEMICOLON),
                    b'%' => Some(PERCENT),
                    b'-' => Some(MINUS),
                    b'#' => Some(BRKT),
                    b',' => Some(COMMA),
                    _ => None,
                };
                if let Some(t) = token {
                    p += 1;
                    emit(&mut out, t, start, p);
                } else if c == b' ' {
                    p += 1;
                } else if c == b'\n' {
                    state = stack.pop().unwrap_or(State::Initial);
                    p += 1;
                } else if c == b'"' {
                    stack.push(state);
                    state = State::Quoted;
                    p += 1;
                } else {
                    out.push(Token { kind: ERROR, text: String::new(), start: p });
                    return out;
                }
            }
            State::Initial => {
                if lower[p..].starts_with("title") && char_at(src, p + 5).is_some_and(is_space) {
                    let begin = p + 5 + char_at(src, p + 5).unwrap().len_utf8();
                    let end = src[begin..].find(['#', '\n', ';']).map_or(n, |i| begin + i);
                    if end > begin {
                        p = end;
                        emit(&mut out, title, start, p);
                        continue;
                    }
                }
                let mut accessible = None;
                for (name, delimiter, next, tok) in [
                    ("acctitle", b':', State::Title, Some(acc_title)),
                    ("accdescr", b':', State::Descr, Some(acc_descr)),
                    ("accdescr", b'{', State::Multiline, None),
                ] {
                    if lower[p..].starts_with(name) {
                        let q = whitespace(src, p + name.len());
                        if bytes.get(q) == Some(&delimiter) {
                            accessible = Some((whitespace(src, q + 1), next, tok));
                            break;
                        }
                    }
                }
                if let Some((end, next, tok)) = accessible {
                    stack.push(state);
                    state = next;
                    p = end;
                    if let Some(t) = tok {
                        emit(&mut out, t, start, p);
                    }
                    continue;
                }
                if !matches!(char_at(src, p), Some('\n' | '\r' | '\u{2028}' | '\u{2029}')) {
                    if cached_end.is_none_or(|e| p > e) {
                        let end = src[p..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(n, |i| p + i);
                        cached_end = Some(end);
                        for k in 0..4 {
                            while seen[k] < dirs[k].len() && dirs[k][seen[k]].0 <= end {
                                seen[k] += 1;
                            }
                            cached_dirs[k] = seen[k].checked_sub(1).map(|j| dirs[k][j]);
                        }
                    }
                    if let Some((k, (_, end))) =
                        cached_dirs.iter().enumerate().find_map(|(k, d)| d.filter(|(a, _)| *a >= p).map(|d| (k, d)))
                    {
                        p = line_break(src, end);
                        emit(&mut out, direction_tb + k as u8, start, p);
                        continue;
                    }
                }
                if c == b'\n' || (c == b'\r' && bytes.get(p + 1) == Some(&b'\n')) {
                    while p < n {
                        if bytes[p] == b'\n' {
                            p += 1;
                        } else if bytes[p] == b'\r' && bytes.get(p + 1) == Some(&b'\n') {
                            p += 2;
                        } else {
                            break;
                        }
                    }
                    emit(&mut out, NEWLINE, start, p);
                    continue;
                }
                if char_at(src, p).is_some_and(is_space) {
                    p = whitespace(src, p);
                    continue;
                }
                if c == b'#' || c == b'%' {
                    p = line_break(src, p);
                    continue;
                }
                if is_word(c as char) {
                    p += 1;
                    while p < n && is_word(bytes[p] as char) {
                        p += 1;
                    }
                    if p - start <= 22 {
                        if let Some(t) = keyword(&lower[start..p]) {
                            if matches!(t, STYLE | CLASSDEF | CLASS) {
                                stack.push(state);
                                state = State::Style;
                            }
                            emit(&mut out, t, start, p);
                            continue;
                        }
                    }
                    while p < n && !b":,\r\n{<>-=".contains(&bytes[p]) {
                        p += 1;
                    }
                    out.push(Token { kind: unqString, text: js::trim(&src[start..p]).into(), start });
                    continue;
                }
                let token = match c {
                    b'{' => Some((STRUCT_START, 1)),
                    b'}' => Some((STRUCT_STOP, 1)),
                    b':' => Some(if src[p..].starts_with(":::") { (STYLE_SEPARATOR, 3) } else { (COLONSEP, 1) }),
                    b'<' => {
                        if bytes.get(p + 1) == Some(&b'-') {
                            Some((END_ARROW_L, 2))
                        } else {
                            None
                        }
                    }
                    b'-' => Some(if bytes.get(p + 1) == Some(&b'>') { (END_ARROW_R, 2) } else { (LINE, 1) }),
                    b',' => Some((COMMA, 1)),
                    _ => None,
                };
                if let Some((t, len)) = token {
                    p += len;
                    emit(&mut out, t, start, p);
                } else if c == b'"' {
                    stack.push(state);
                    state = State::Quoted;
                    p += 1;
                } else {
                    out.push(Token { kind: ERROR, text: String::new(), start });
                    return out;
                }
            }
        }
    }
    let last = match state {
        State::Initial => Some(EOF),
        State::Quoted => Some(qString),
        State::Title => Some(acc_title_value),
        State::Descr => Some(acc_descr_value),
        State::Multiline => Some(acc_descr_multiline_value),
        State::Style => None,
    };
    if let Some(t) = last {
        emit(&mut out, t, n, n);
    }
    emit(&mut out, END, n, n);
    out
}
