#![allow(non_upper_case_globals)]
//! State grammar lexer. Source positions stay in UTF-8; diagnostics map to UTF-16.
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
pub struct Tokens {
    pub items: Vec<Token>,
    pub error: Option<String>,
}
#[derive(Clone, Copy)]
enum State {
    Initial,
    Struct,
    State,
    String,
    Id,
    Scale,
    Title,
    Descr,
    Multiline,
    ClassDef,
    ClassDefId,
    Class,
    ClassStyle,
    Style,
    StyleValues,
    Note,
    NoteId,
    NoteText,
    Floating,
    FloatingId,
}
struct Lexer<'a> {
    s: &'a str,
    lower: String,
    p: usize,
    state: State,
    stack: Vec<State>,
    out: Vec<Token>,
    error: Option<String>,
    dirs: [Vec<(usize, usize)>; 4],
    stereo: [Vec<(usize, usize)>; 6],
}
impl Lexer<'_> {
    fn word(&self, w: &str, q: usize) -> bool {
        self.lower.get(q..).is_some_and(|s| s.starts_with(w))
    }
    fn word_end(&self, mut q: usize) -> usize {
        while char_at(self.s, q).is_some_and(is_word) {
            q += 1;
        }
        q
    }
    fn ws(&self, mut q: usize) -> usize {
        while let Some(c) = char_at(self.s, q).filter(|&c| is_space(c)) {
            q += c.len_utf8();
        }
        q
    }
    fn eol(&self, q: usize) -> usize {
        self.s[q..].find('\n').map_or(self.s.len(), |i| q + i)
    }
    fn line_end(&self, q: usize) -> usize {
        self.s[q..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(self.s.len(), |i| q + i)
    }
    fn emit(&mut self, k: u8, end: usize) {
        self.emit_text(k, self.s[self.p..end].into(), end);
    }
    fn emit_text(&mut self, kind: u8, text: String, end: usize) {
        self.out.push(Token { kind, text, start: self.p });
        self.p = end;
    }
    fn push(&mut self, state: State) {
        self.stack.push(self.state);
        self.state = state;
    }
    fn pop(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.state = s;
        }
    }
    fn finish(mut self, k: u8) -> Tokens {
        self.out.push(Token { kind: k, text: String::new(), start: self.p });
        Tokens { items: self.out, error: self.error }
    }
    fn id(&mut self, end: usize) -> bool {
        let at = self.s[self.p..].find("%%").map(|i| self.p + i);
        if let Some(at) = at.filter(|i| i + 2 <= end) {
            if at == self.p {
                self.p = end;
                false
            } else {
                self.emit(ID, at);
                true
            }
        } else {
            self.emit(ID, end);
            true
        }
    }
    fn direction(&self) -> Option<(usize, usize)> {
        let end = self.line_end(self.p);
        for (k, list) in self.dirs.iter().enumerate() {
            let at = list.partition_point(|&(at, _)| at < end);
            if let Some((start, finish)) = at.checked_sub(1).map(|i| list[i]).filter(|&(at, _)| at >= self.p) {
                let _ = start;
                return Some((k, finish));
            }
        }
        None
    }
    fn stereotype(&self) -> Option<(usize, usize, usize)> {
        let end = self.line_end(self.p);
        for (k, list) in self.stereo.iter().enumerate() {
            let at = list.partition_point(|&(at, _)| at < end);
            if let Some((start, finish)) = at.checked_sub(1).map(|i| list[i]).filter(|&(at, _)| at >= self.p) {
                return Some((k, start, finish));
            }
        }
        None
    }
    fn run(mut self) -> Tokens {
        loop {
            let n = self.s.len();
            if self.p >= n {
                let last = match self.state {
                    State::Initial => Some(NL),
                    State::Title => Some(acc_title_value),
                    State::Descr => Some(acc_descr_value),
                    State::Multiline => Some(acc_descr_multiline_value),
                    State::ClassDefId => Some(CLASSDEF_STYLEOPTS),
                    State::ClassStyle => Some(STYLECLASS),
                    State::StyleValues => Some(STYLEDEF_STYLEOPTS),
                    State::String => Some(STATE_DESCR),
                    State::Floating => Some(NOTE_TEXT),
                    State::Id | State::FloatingId => Some(ID),
                    _ => None,
                };
                if let Some(k) = last {
                    self.emit(k, n);
                }
                return self.finish(END);
            }
            let c = char_at(self.s, self.p).unwrap();
            let bytes = self.s.as_bytes();
            match self.state {
                State::Initial | State::Struct => {
                    let root = matches!(self.state, State::Initial);
                    if root {
                        let mut keyword = None;
                        for (w, k) in [("click", CLICK), ("href", HREF), ("default", DEFAULT)] {
                            if self.word(w, self.p) && !char_at(self.s, self.p + w.len()).is_some_and(is_word) {
                                keyword = Some((k, w.len()));
                                break;
                            }
                        }
                        if let Some((k, len)) = keyword {
                            self.emit(k, self.p + len);
                            continue;
                        }
                        if c == '"' {
                            if let Some(end) = self.s[self.p + 1..].find('"') {
                                self.emit(STRING, self.p + end + 2);
                                continue;
                            }
                        }
                        if let Some((kind, end)) = self.direction() {
                            self.emit(direction_tb + kind as u8, self.eol(end));
                            continue;
                        }
                        if c == '\n' {
                            let mut p = self.p + 1;
                            while bytes.get(p) == Some(&b'\n') {
                                p += 1;
                            }
                            self.emit(NL, p);
                            continue;
                        }
                        if is_space(c) {
                            self.p = self.ws(self.p);
                            continue;
                        }
                    } else if c != '\n' && is_space(c) {
                        let mut p = self.p + c.len_utf8();
                        while let Some(ch) = char_at(self.s, p).filter(|&ch| ch != '\n' && is_space(ch)) {
                            p += ch.len_utf8();
                        }
                        self.p = p;
                        continue;
                    }
                    if c == '#' || (self.s[self.p..].starts_with("%%") && bytes.get(self.p + 2) != Some(&b'{')) {
                        self.p = self.eol(self.p);
                        continue;
                    }
                    if root && self.word("scale", self.p) {
                        let end = self.ws(self.p + 5);
                        if end > self.p + 5 {
                            self.emit(scale, end);
                            self.push(State::Scale);
                            continue;
                        }
                    }
                    if self.word("style", self.p) {
                        let end = self.ws(self.p + 5);
                        if end > self.p + 5 {
                            self.emit(style, end);
                            self.push(State::Style);
                            continue;
                        }
                    }
                    if self.word("state", self.p) {
                        let end = self.ws(self.p + 5);
                        if end > self.p + 5 {
                            self.p = end;
                            self.push(State::State);
                            continue;
                        }
                        if root && self.word("diagram", self.p + 5) {
                            let from = if self.word("-v2", self.p + 12) { self.p + 15 } else { self.p + 12 };
                            let end = self.ws(from);
                            if end > from {
                                self.emit(SD, end);
                                continue;
                            }
                        }
                    }
                    if self.word("class", self.p) {
                        let def = self.word("def", self.p + 5);
                        let from = self.p + if def { 8 } else { 5 };
                        let end = self.ws(from);
                        if end > from {
                            self.emit(if def { classDef } else { class }, end);
                            self.push(if def { State::ClassDef } else { State::Class });
                            continue;
                        }
                    }
                    if root && (self.word("acctitle", self.p) || self.word("accdescr", self.p)) {
                        let title = self.word("acctitle", self.p);
                        let q = self.ws(self.p + 8);
                        if bytes.get(q) == Some(&b':') {
                            self.emit(if title { acc_title } else { acc_descr }, self.ws(q + 1));
                            self.push(if title { State::Title } else { State::Descr });
                            continue;
                        }
                        if !title && bytes.get(q) == Some(&b'{') {
                            self.p = self.ws(q + 1);
                            self.push(State::Multiline);
                            continue;
                        }
                    }
                    if root && c == '{' {
                        self.emit(STRUCT_START, self.p + 1);
                        self.push(State::Struct);
                        continue;
                    }
                    if !root {
                        if let Some((kind, end)) = self.direction() {
                            self.emit(direction_tb + kind as u8, self.eol(end));
                            continue;
                        }
                        if c == '}' {
                            self.emit(STRUCT_STOP, self.p + 1);
                            self.pop();
                            continue;
                        }
                        if c == '\n' {
                            self.p += 1;
                            continue;
                        }
                    }
                    if self.word("note", self.p) {
                        let end = self.ws(self.p + 4);
                        if end > self.p + 4 {
                            self.emit(note, end);
                            self.push(State::Note);
                            continue;
                        }
                    }
                    if root && self.word("hide empty description", self.p) && !char_at(self.s, self.p + 22).is_some_and(is_word) {
                        self.emit(HIDE_EMPTY, self.p + 22);
                        continue;
                    }
                    if self.s[self.p..].starts_with("[*]") {
                        self.emit(EDGE_STATE, self.p + 3);
                        continue;
                    }
                    let mut j = self.p;
                    while let Some(ch) = char_at(self.s, j).filter(|&ch| !matches!(ch, ':' | '-' | '{') && !is_space(ch)) {
                        j += ch.len_utf8();
                    }
                    if j > self.p {
                        self.id(j);
                        continue;
                    }
                    if c == ':' {
                        j = self.p + 1;
                        while j < n {
                            let d = bytes[j];
                            if matches!(d, b'\n' | b';') {
                                break;
                            }
                            if d == b':' {
                                if j + 1 >= n || matches!(bytes.get(j + 1), Some(b':' | b'\n' | b';')) {
                                    break;
                                }
                                j += 1;
                            }
                            j += 1;
                        }
                        if j > self.p + 1 {
                            self.emit_text(DESCR, js::trim(&self.s[self.p..j]).into(), j);
                            continue;
                        }
                        if self.s[self.p..].starts_with(":::") {
                            self.emit(STYLE_SEPARATOR, self.p + 3);
                            continue;
                        }
                    } else if self.s[self.p..].starts_with("--") {
                        if self.s[self.p..].starts_with("-->") {
                            self.emit(ARROW, self.p + 3);
                            continue;
                        }
                        if !root {
                            self.emit(CONCURRENT, self.p + 2);
                            continue;
                        }
                    }
                    if !root {
                        return self.finish(ERROR);
                    }
                    self.emit(INVALID, self.p + c.len_utf8());
                }
                State::State => {
                    if c != '\n' && is_space(c) {
                        let mut j = self.p + c.len_utf8();
                        while let Some(ch) = char_at(self.s, j).filter(|&ch| ch != '\n' && is_space(ch)) {
                            j += ch.len_utf8();
                        }
                        self.p = j;
                        continue;
                    }
                    if c == '#' || (self.s[self.p..].starts_with("%%") && bytes.get(self.p + 2) != Some(&b'{')) {
                        self.p = self.eol(self.p);
                        continue;
                    }
                    if let Some((kind, at, end)) = self.stereotype() {
                        self.emit_text(
                            if kind % 3 == 2 {
                                CHOICE
                            } else if kind % 3 == 0 {
                                FORK
                            } else {
                                JOIN
                            },
                            js::trim(&self.s[self.p..at]).into(),
                            end,
                        );
                        self.pop();
                        continue;
                    }
                    if c == '"' {
                        self.p += 1;
                        self.push(State::String);
                        continue;
                    }
                    let q = if c == '\n' { self.ws(self.p) } else { self.p };
                    if self.word("as", q) {
                        let end = self.ws(q + 2);
                        if end > q + 2 {
                            self.emit(AS, end);
                            self.push(State::Id);
                            continue;
                        }
                    }
                    if c == '\n' {
                        self.p += 1;
                        self.pop();
                        continue;
                    }
                    if c == '{' {
                        self.emit(STRUCT_START, self.p + 1);
                        self.pop();
                        self.push(State::Struct);
                        continue;
                    }
                    if is_word(c) {
                        let first = self.word_end(self.p);
                        let second = self.ws(first);
                        if second > first && char_at(self.s, second).is_some_and(is_word) {
                            if let Some(brace) = self
                                .s
                                .get(second + 1..)
                                .and_then(|s| s.find('{'))
                                .map(|i| second + 1 + i)
                                .filter(|&b| b < self.line_end(second))
                            {
                                self.error = Some(format!(
                                    "Error: State name must be a single word. Found: \"{}\"",
                                    js::trim(&self.s[self.p..brace + 1])
                                ));
                                return self.finish(ERROR);
                            }
                        }
                    }
                    let mut j = self.p + c.len_utf8();
                    while let Some(ch) = char_at(self.s, j).filter(|&ch| ch != '{' && !is_space(ch)) {
                        j += ch.len_utf8();
                    }
                    self.emit(COMPOSIT_STATE, j);
                }
                State::String => {
                    if c == '"' {
                        self.p += 1;
                        self.pop();
                    } else {
                        let end = self.s[self.p..].find('"').map_or(n, |i| self.p + i);
                        self.emit(STATE_DESCR, end);
                    }
                }
                State::Id => {
                    let end = self.s[self.p..].find(['\n', '{']).map_or(n, |i| self.p + i);
                    if self.id(end) {
                        self.pop();
                    }
                }
                State::Scale => {
                    if c.is_ascii_digit() {
                        let mut j = self.p + 1;
                        while bytes.get(j).is_some_and(u8::is_ascii_digit) {
                            j += 1;
                        }
                        self.emit(WIDTH, j);
                    } else {
                        let q = self.ws(self.p);
                        if q > self.p && self.word("width", q) && !char_at(self.s, q + 5).is_some_and(is_word) {
                            self.p = q + 5;
                            self.pop();
                        } else {
                            return self.finish(ERROR);
                        }
                    }
                }
                State::Title | State::Descr => {
                    self.emit(if matches!(self.state, State::Title) { acc_title_value } else { acc_descr_value }, self.eol(self.p));
                    self.pop();
                }
                State::Multiline => {
                    if c == '}' {
                        self.p += 1;
                        self.pop();
                    } else {
                        let end = self.s[self.p..].find('}').map_or(n, |i| self.p + i);
                        self.emit(acc_descr_multiline_value, end);
                    }
                }
                State::ClassDef => {
                    let default = self.word("default", self.p) && self.ws(self.p + 7) > self.p + 7;
                    let j = if default { self.p + 7 } else { self.word_end(self.p) };
                    let end = self.ws(j);
                    if j == self.p || end == j {
                        return self.finish(ERROR);
                    }
                    self.emit(if default { DEFAULT_CLASSDEF_ID } else { CLASSDEF_ID }, end);
                    self.pop();
                    self.push(State::ClassDefId);
                }
                State::ClassDefId | State::ClassStyle | State::StyleValues => {
                    let k = match self.state {
                        State::ClassDefId => CLASSDEF_STYLEOPTS,
                        State::ClassStyle => STYLECLASS,
                        _ => STYLEDEF_STYLEOPTS,
                    };
                    self.emit(k, self.eol(self.p));
                    self.pop();
                }
                State::Class => {
                    let mut j = self.word_end(self.p);
                    if j == self.p {
                        return self.finish(ERROR);
                    }
                    while bytes.get(j) == Some(&b',') {
                        let k = self.ws(j + 1);
                        let e = self.word_end(k);
                        if e == k {
                            break;
                        }
                        j = e;
                    }
                    self.emit(CLASSENTITY_IDS, j);
                    self.pop();
                    self.push(State::ClassStyle);
                }
                State::Style => {
                    let mut j = self.p;
                    while char_at(self.s, j).is_some_and(|c| is_word(c) || c == ',') {
                        j += 1;
                    }
                    let end = self.ws(j);
                    if j == self.p || end == j {
                        return self.finish(ERROR);
                    }
                    self.emit(STYLE_IDS, end);
                    self.pop();
                    self.push(State::StyleValues);
                }
                State::Note => {
                    if self.word("left of", self.p) && !char_at(self.s, self.p + 7).is_some_and(is_word) {
                        self.emit(left_of, self.p + 7);
                        self.pop();
                        self.push(State::NoteId);
                    } else if self.word("right of", self.p) && !char_at(self.s, self.p + 8).is_some_and(is_word) {
                        self.emit(right_of, self.p + 8);
                        self.pop();
                        self.push(State::NoteId);
                    } else if c == '"' {
                        self.p += 1;
                        self.pop();
                        self.push(State::Floating);
                    } else {
                        return self.finish(ERROR);
                    }
                }
                State::NoteId => {
                    let q = self.ws(self.p);
                    let mut j = q;
                    while let Some(ch) = char_at(self.s, j).filter(|&ch| !matches!(ch, ':' | '-') && !is_space(ch)) {
                        j += ch.len_utf8();
                    }
                    if j == q {
                        return self.finish(ERROR);
                    }
                    if self.id(j) {
                        self.pop();
                        self.push(State::NoteText);
                    }
                }
                State::NoteText => {
                    let q = self.ws(self.p);
                    if bytes.get(q) == Some(&b':') {
                        let end = self.s[q + 1..].find([':', '\n', ';']).map_or(n, |i| q + 1 + i);
                        if end > q + 1 {
                            let raw = &self.s[self.p..end];
                            let text = js::trim(js::slice16(raw, 2, js::len16(raw))).into();
                            self.emit_text(NOTE_TEXT, text, end);
                            self.pop();
                            continue;
                        }
                    }
                    let mut next = self.s[self.p..].find('\n').map(|i| self.p + i);
                    let mut found = None;
                    while let Some(k) = next {
                        let end = self.ws(k);
                        if self.word("end note", end) && !char_at(self.s, end + 8).is_some_and(is_word) {
                            found = Some(end);
                            break;
                        }
                        next = self.s[end..].find('\n').map(|i| end + i);
                    }
                    let Some(end) = found else {
                        return self.finish(ERROR);
                    };
                    self.emit_text(NOTE_TEXT, js::trim(&self.s[self.p..end]).into(), end + 8);
                    self.pop();
                }
                State::Floating => {
                    let q = self.ws(self.p);
                    if self.word("as", q) {
                        self.emit(AS, self.ws(q + 2));
                        self.pop();
                        self.push(State::FloatingId);
                    } else if c == '"' {
                        self.p += 1;
                    } else {
                        let end = self.s[self.p..].find('"').map_or(n, |i| self.p + i);
                        self.emit(NOTE_TEXT, end);
                    }
                }
                State::FloatingId => {
                    if self.id(self.eol(self.p)) {
                        self.pop();
                    }
                }
            }
        }
    }
}
pub fn tokenize(s: &str) -> Tokens {
    let lower = s.to_ascii_lowercase();
    let mut l = Lexer {
        s,
        lower,
        p: 0,
        state: State::Initial,
        stack: vec![],
        out: vec![],
        error: None,
        dirs: std::array::from_fn(|_| vec![]),
        stereo: std::array::from_fn(|_| vec![]),
    };
    let mut from = 0;
    while let Some(off) = l.lower[from..].find("direction") {
        let at = from + off;
        let q = l.ws(at + 9);
        if q > at + 9 {
            for (k, d) in ["tb", "bt", "rl", "lr"].iter().enumerate() {
                if l.word(d, q) {
                    l.dirs[k].push((at, q + 2));
                    break;
                }
            }
        }
        from = at + 9;
    }
    for (k, word) in ["<<fork>>", "<<join>>", "<<choice>>", "[[fork]]", "[[join]]", "[[choice]]"].iter().enumerate() {
        for (at, _) in l.lower.match_indices(word) {
            l.stereo[k].push((at, at + word.len()));
        }
    }
    l.run()
}
