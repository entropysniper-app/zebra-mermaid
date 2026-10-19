#![allow(non_upper_case_globals)]
//! ER lexer states and ordered cardinality aliases, ported from Pele.
use super::tokens::*;
use crate::util::chars::{char_at, is_space, is_word};
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: u8,
    pub text: String,
    pub start: usize,
}
#[derive(Clone, Copy)]
enum State {
    Initial,
    Block,
    Backtick,
    Title,
    Descr,
    Multiline,
    Style,
}
fn skip(s: &str, mut p: usize) -> usize {
    while let Some(c) = char_at(s, p).filter(|&c| is_space(c)) {
        p += c.len_utf8();
    }
    p
}
fn end_line(s: &str, p: usize) -> usize {
    s[p..].find('\n').map_or(s.len(), |i| p + i)
}
fn word_at(s: &str, p: usize) -> bool {
    char_at(s, p).is_some_and(is_word)
}
fn text_char(c: char, style: bool) -> bool {
    !c.is_ascii() || is_word(c) || matches!(c, '-' | '*') || (!style && c == '.')
}
fn attr_char(c: char, first: bool) -> bool {
    if !c.is_ascii() {
        return c as u32 >= 0xc0 || c == 'µ';
    }
    if first {
        (is_word(c) && !c.is_ascii_digit()) || c == '*'
    } else {
        is_word(c) || "-*.[ ](),".replace(' ', "").contains(c)
    }
}
pub fn tokenize(src: &str) -> Vec<Token> {
    let n = src.len();
    let lower = src.to_ascii_lowercase();
    let bytes = src.as_bytes();
    let mut out = vec![];
    let mut p = 0;
    let mut state = State::Initial;
    let mut dir_candidates: [Vec<(usize, usize)>; 4] = std::array::from_fn(|_| vec![]);
    let mut search = 0;
    while let Some(off) = lower[search..].find("direction") {
        let at = search + off;
        let end = skip(src, at + 9);
        if end > at + 9 {
            for (k, d) in ["tb", "bt", "rl", "lr"].iter().enumerate() {
                if lower[end..].starts_with(d) {
                    dir_candidates[k].push((at, end + 2));
                    break;
                }
            }
        }
        search = at + 9;
    }
    let mut dir_ends = [0; 4];
    let mut line_end = None;
    let mut last_dirs: [Option<(usize, usize)>; 4] = [None; 4];
    let emit = |out: &mut Vec<Token>, kind: u8, start: usize, end: usize| out.push(Token { kind, text: src[start..end].into(), start });
    while p < n {
        let start = p;
        let c = src[p..].chars().next().unwrap();
        let advance = c.len_utf8();
        match state {
            State::Title | State::Descr => {
                let k = if matches!(state, State::Title) { acc_title_value } else { acc_descr_value };
                p = end_line(src, p);
                emit(&mut out, k, start, p);
                state = State::Initial;
            }
            State::Multiline => {
                if c == '}' {
                    p += 1;
                    state = State::Initial;
                } else {
                    p = src[p..].find('}').map_or(n, |i| p + i);
                    emit(&mut out, acc_descr_multiline_value, start, p);
                }
            }
            State::Backtick => {
                if c == '`' {
                    p += 1;
                    state = State::Block;
                } else {
                    p = src[p..].find('`').map_or(n, |i| p + i);
                    emit(&mut out, ATTRIBUTE_WORD, start, p);
                }
            }
            State::Style => {
                if c == '\n' {
                    p += 1;
                    while bytes.get(p) == Some(&b'\n') {
                        p += 1;
                    }
                    state = State::Initial;
                    emit(&mut out, NEWLINE, start, p);
                } else if is_space(c) {
                    p = skip(src, p);
                } else if let Some(k) = match c {
                    ':' => Some(COLON),
                    ',' => Some(COMMA),
                    '#' => Some(BRKT),
                    ';' => Some(SEMI),
                    _ => None,
                } {
                    p += 1;
                    emit(&mut out, k, start, p);
                } else if text_char(c, true) {
                    p += advance;
                    while char_at(src, p).is_some_and(|c| text_char(c, true)) {
                        p += char_at(src, p).unwrap().len_utf8();
                    }
                    emit(&mut out, STYLE_TEXT, start, p);
                } else {
                    out.push(Token { kind: ERROR, text: String::new(), start });
                    return out;
                }
            }
            State::Block => {
                if is_space(c) {
                    p = skip(src, p);
                    continue;
                }
                if ["pk", "fk", "uk"].iter().any(|k| lower[p..].starts_with(k) && !word_at(src, p + 2)) {
                    p += 2;
                    emit(&mut out, ATTRIBUTE_KEY, start, p);
                    continue;
                }
                let whitespace_end = src[p..].find(is_space).map_or(n, |i| p + i);
                let tilde = src[p..whitespace_end].find('~').map(|i| p + i);
                if let Some(first) = tilde {
                    let line_end = src[p..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(n, |i| p + i);
                    if let Some(last) = src[p..line_end].rfind('~').map(|i| p + i) {
                        if last > first {
                            p = last + 1;
                            while let Some(c) = char_at(src, p).filter(|&c| !is_space(c)) {
                                p += c.len_utf8();
                            }
                            emit(&mut out, ATTRIBUTE_WORD, start, p);
                            continue;
                        }
                    }
                }
                if attr_char(c, true) {
                    p += advance;
                    while let Some(c) = char_at(src, p).filter(|&c| attr_char(c, false)) {
                        p += c.len_utf8();
                    }
                    emit(&mut out, ATTRIBUTE_WORD, start, p);
                } else if c == '`' {
                    state = State::Backtick;
                    p += 1;
                } else if c == '"' && src[p + 1..].contains('"') {
                    p = p + 1 + src[p + 1..].find('"').unwrap() + 1;
                    emit(&mut out, COMMENT, start, p);
                } else if c == '}' {
                    state = State::Initial;
                    p += 1;
                    emit(&mut out, BLOCK_STOP, start, p);
                } else {
                    p += advance;
                    emit(&mut out, CHAR, start, p);
                }
            }
            State::Initial => {
                let mut acc = None;
                for (name, delimiter, next, k) in [
                    ("acctitle", b':', State::Title, Some(acc_title)),
                    ("accdescr", b':', State::Descr, Some(acc_descr)),
                    ("accdescr", b'{', State::Multiline, None),
                ] {
                    if lower[p..].starts_with(name) {
                        let q = skip(src, p + name.len());
                        if bytes.get(q) == Some(&delimiter) {
                            acc = Some((skip(src, q + 1), next, k));
                            break;
                        }
                    }
                }
                if let Some((end, next, k)) = acc {
                    p = end;
                    state = next;
                    if let Some(k) = k {
                        emit(&mut out, k, start, p);
                    }
                    continue;
                }
                if line_end.is_none_or(|end| p > end) {
                    let end = src[p..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(n, |i| p + i);
                    line_end = Some(end);
                    for k in 0..4 {
                        while dir_ends[k] < dir_candidates[k].len() && dir_candidates[k][dir_ends[k]].0 < end {
                            dir_ends[k] += 1;
                        }
                        last_dirs[k] = dir_ends[k].checked_sub(1).map(|j| dir_candidates[k][j]);
                    }
                }
                if let Some((k, (_, end))) = last_dirs.iter().enumerate().find_map(|(k, d)| d.filter(|(at, _)| *at >= p).map(|d| (k, d))) {
                    p = end_line(src, end);
                    emit(&mut out, direction_tb + k as u8, start, p);
                    continue;
                }
                if matches!(c, ' ' | '\t' | '\r') {
                    p += advance;
                    while matches!(bytes.get(p), Some(b' ' | b'\t' | b'\r')) {
                        p += 1;
                    }
                    continue;
                }
                if c == '\n' {
                    p += 1;
                    while bytes.get(p) == Some(&b'\n') {
                        p += 1;
                    }
                    emit(&mut out, NEWLINE, start, p);
                    continue;
                }
                if c == '"' {
                    if let Some(end) = src[p + 1..].find('"').map(|i| p + 1 + i) {
                        let text = &src[p + 1..end];
                        let valid = !text.is_empty() && !text.contains(['%', '\\', '\r', '\n', '\u{b}', '\u{8}']);
                        p = end + 1;
                        emit(&mut out, if valid { ENTITY_NAME } else { WORD }, start, p);
                    } else {
                        p += 1;
                        emit(&mut out, CHAR, start, p);
                    }
                    continue;
                }
                if let Some((k, step)) = match c {
                    '{' => Some((BLOCK_START, 1)),
                    '#' => Some((BRKT, 1)),
                    ',' => Some((COMMA, 1)),
                    ':' => Some(if src[p..].starts_with(":::") { (STYLE_SEPARATOR, 3) } else { (COLON, 1) }),
                    '[' => Some((SQS, 1)),
                    ']' => Some((SQE, 1)),
                    _ => None,
                } {
                    p += step;
                    if k == BLOCK_START {
                        state = State::Block;
                    }
                    emit(&mut out, k, start, p);
                    continue;
                }
                if c.is_ascii_digit() {
                    let d = bytes.get(p + 1).copied();
                    if d == Some(b'+') && matches!(c, '0' | '1') {
                        p += 2;
                        emit(&mut out, if c == '1' { ONE_OR_MORE } else { ZERO_OR_MORE }, start, p);
                        continue;
                    }
                    let mut q = p + 1;
                    while bytes.get(q).is_some_and(u8::is_ascii_digit) {
                        q += 1;
                    }
                    if bytes.get(q) == Some(&b'.') && bytes.get(q + 1).is_some_and(u8::is_ascii_digit) {
                        p = q + 2;
                        while bytes.get(p).is_some_and(u8::is_ascii_digit) {
                            p += 1;
                        }
                        emit(&mut out, DECIMAL_NUM, start, p);
                        continue;
                    }
                    if c == '1' {
                        let after = char_at(src, p + 1);
                        let follows = if after.is_some_and(is_space) {
                            let r = skip(src, p + 1);
                            word_at(src, r) || matches!(bytes.get(r), Some(b'"' | b'\''))
                        } else {
                            matches!(d, Some(b'-' | b'.')) && matches!(bytes.get(p + 2), Some(b'-' | b'.'))
                        };
                        if follows {
                            p += 1;
                            emit(&mut out, ONLY_ONE, start, p);
                            continue;
                        }
                        if !word_at(src, p + 1) {
                            p += 1;
                            emit(&mut out, ENTITY_ONE, start, p);
                            continue;
                        }
                    }
                    p = q;
                    emit(&mut out, NUM, start, p);
                    continue;
                }
                let mut found = None;
                for (word, k, boundary) in [
                    ("erdiagram", ER_DIAGRAM, true),
                    ("end", END_KW, true),
                    ("style", STYLE, true),
                    ("subgraph", SUBGRAPH, true),
                    ("classdef", CLASSDEF, true),
                    ("class", CLASS, true),
                    ("one or zero", ZERO_OR_ONE, true),
                    ("one or more", ONE_OR_MORE, true),
                    ("one or many", ONE_OR_MORE, true),
                    ("one", ONLY_ONE, true),
                    ("only one", ONLY_ONE, true),
                    ("optionally to", NON_IDENTIFYING, true),
                    ("zero or one", ZERO_OR_ONE, true),
                    ("zero or more", ZERO_OR_MORE, true),
                    ("zero or many", ZERO_OR_MORE, true),
                    ("many(0)", ZERO_OR_MORE, false),
                    ("many(1)", ONE_OR_MORE, false),
                    ("many", ZERO_OR_MORE, true),
                    ("to", IDENTIFYING, true),
                ] {
                    if lower[p..].starts_with(word) && (!boundary || !word_at(src, p + word.len())) {
                        found = Some((k, word.len()));
                        break;
                    }
                }
                if found.is_none() {
                    let d = bytes.get(p + 1).copied();
                    let lc = c.to_ascii_lowercase();
                    found = match (lc, d) {
                        ('o', Some(b'|')) => Some((ZERO_OR_ONE, 2)),
                        ('o', Some(b'{')) => Some((ZERO_OR_MORE, 2)),
                        ('u', Some(b'.' | b'-' | b'|')) => Some((MD_PARENT, 1)),
                        ('|', Some(b'o' | b'O')) if !word_at(src, p + 2) => Some((ZERO_OR_ONE, 2)),
                        ('|', Some(b'|')) => Some((ONLY_ONE, 2)),
                        ('|', Some(b'{')) | ('}', Some(b'|')) => Some((ONE_OR_MORE, 2)),
                        ('}', Some(b'o' | b'O')) if !word_at(src, p + 2) => Some((ZERO_OR_MORE, 2)),
                        ('.', Some(b'.' | b'-')) | ('-', Some(b'.')) => Some((NON_IDENTIFYING, 2)),
                        ('-', Some(b'-')) => Some((IDENTIFYING, 2)),
                        _ => None,
                    };
                }
                if let Some((k, step)) = found {
                    p += step;
                    if k == END_KW {
                        p = skip(src, p);
                    }
                    if matches!(k, STYLE | CLASSDEF) {
                        state = State::Style;
                    }
                    emit(&mut out, k, start, p);
                    continue;
                }
                if text_char(c, false) {
                    p += advance;
                    while let Some(c) = char_at(src, p).filter(|&c| text_char(c, false)) {
                        p += c.len_utf8();
                    }
                    emit(&mut out, UNICODE_TEXT, start, p);
                } else {
                    p += advance;
                    emit(&mut out, CHAR, start, p);
                }
            }
        }
    }
    let last = match state {
        State::Initial => Some(EOF),
        State::Title => Some(acc_title_value),
        State::Descr => Some(acc_descr_value),
        State::Multiline => Some(acc_descr_multiline_value),
        _ => None,
    };
    if let Some(k) = last {
        emit(&mut out, k, n, n);
    }
    emit(&mut out, END, n, n);
    out
}
