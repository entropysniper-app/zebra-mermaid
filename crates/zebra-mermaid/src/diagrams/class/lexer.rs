#![allow(non_upper_case_globals)]
//! State lexer for class diagrams, following Pele's rule ordering.
use super::tokens::*;
use crate::util::{
    chars::{char_at, is_space, is_word},
    unicode::is_unicode_letter,
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
    String,
    Backtick,
    Generic,
    CallbackName,
    CallbackArgs,
    Title,
    Descr,
    Multiline,
    Class,
    ClassBody,
    Namespace,
    NamespaceBody,
}
struct Lexer<'a> {
    s: &'a str,
    p: usize,
    state: State,
    stack: Vec<State>,
    out: Vec<Token>,
    dirs: [Vec<(usize, usize)>; 4],
    dir_seen: [usize; 4],
    dir_last: [Option<(usize, usize)>; 4],
    line_end: Option<usize>,
}
impl Lexer<'_> {
    fn kw(&self, word: &str) -> bool {
        self.s[self.p..].starts_with(word) && !char_at(self.s, self.p + word.len()).is_some_and(is_word)
    }
    fn skip(&self, mut q: usize) -> usize {
        while let Some(c) = char_at(self.s, q).filter(|&c| is_space(c)) {
            q += c.len_utf8();
        }
        q
    }
    fn until(&self, c: char) -> usize {
        self.s[self.p..].find(c).map_or(self.s.len(), |i| self.p + i)
    }
    fn emit(&mut self, kind: u8, end: usize) -> bool {
        self.out.push(Token { kind, text: self.s[self.p..end].into(), start: self.p });
        self.p = end;
        true
    }
    fn push(&mut self, state: State) {
        self.stack.push(self.state);
        self.state = state;
    }
    fn pop(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
    }
    fn space(&mut self, leaves: bool) {
        let q = self.skip(self.p);
        if let Some(k) = self.s[self.p..q].rfind('\n') {
            if leaves {
                self.pop();
            }
            self.emit(NEWLINE, self.p + k + 1);
        } else {
            self.p = q;
        }
    }
    fn keyword(&mut self) -> bool {
        for (word, t) in [
            ("cssClass", CSSCLASS),
            ("callback", CALLBACK),
            ("click", CLICK),
            ("link", LINK),
            ("note for", NOTE_FOR),
            ("note", NOTE),
            ("href", HREF),
        ] {
            if self.kw(word) {
                return self.emit(t, self.p + word.len());
            }
        }
        if self.s[self.p..].starts_with("<<") {
            return self.emit(ANNOTATION_START, self.p + 2);
        }
        if self.s[self.p..].starts_with(">>") {
            return self.emit(ANNOTATION_END, self.p + 2);
        }
        false
    }
    fn common(&mut self) -> bool {
        if self.keyword() {
            return true;
        }
        let bytes = self.s.as_bytes();
        let c = bytes[self.p];
        let d = bytes.get(self.p + 1).copied();
        if let Some(state) = match c {
            b'"' => Some(State::String),
            b'~' => Some(State::Generic),
            b'`' => Some(State::Backtick),
            _ => None,
        } {
            self.push(state);
            self.p += 1;
            return true;
        }
        if c == b'_' {
            for word in ["_self", "_blank", "_parent", "_top"] {
                if self.kw(word) {
                    return self.emit(LINK_TARGET, self.p + word.len());
                }
            }
        }
        let t = match c {
            b'<' => Some((if d == Some(b'|') { EXTENSION } else { DEPENDENCY }, if d == Some(b'|') { 2 } else { 1 })),
            b'|' => {
                if d == Some(b'>') {
                    Some((EXTENSION, 2))
                } else {
                    None
                }
            }
            b'>' => Some((DEPENDENCY, 1)),
            b'*' => Some((COMPOSITION, 1)),
            b'o' if !char_at(self.s, self.p + 1).is_some_and(is_word) => Some((AGGREGATION, 1)),
            b'(' if d == Some(b')') => Some((LOLLIPOP, 2)),
            b'-' => Some(if d == Some(b'-') { (LINE, 2) } else { (MINUS, 1) }),
            b'.' => Some(if d == Some(b'.') { (DOTTED_LINE, 2) } else { (DOT, 1) }),
            b'+' => Some((PLUS, 1)),
            b'%' => Some((PCT, 1)),
            b'=' => Some((EQUALS, 1)),
            b'[' => Some((SQS, 1)),
            b']' => Some((SQE, 1)),
            b'!' | b'#' | b'$' | b'&' | b'\'' | b',' | b'?' | b'\\' | b'/' => Some((PUNCTUATION, 1)),
            _ => None,
        };
        if let Some((t, len)) = t {
            return self.emit(t, self.p + len);
        }
        if c == b':' {
            let start = self.p + 1;
            let end = self.s[start..].find([':', '\n', ';']).map_or(self.s.len(), |i| start + i);
            if end > start {
                return self.emit(LABEL, end);
            }
            if self.s[self.p..].starts_with(":::") {
                return self.emit(STYLE_SEPARATOR, self.p + 3);
            }
            return false;
        }
        if is_word(c as char) {
            let mut q = self.p + 1;
            while self.s.as_bytes().get(q).is_some_and(|&b| is_word(b as char)) {
                q += 1;
            }
            return self.emit(ALPHA, q);
        }
        let ch = char_at(self.s, self.p).unwrap();
        if is_unicode_letter(ch as u32) {
            return self.emit(UNICODE_TEXT, self.p + ch.len_utf8());
        }
        false
    }
    fn direction(&mut self) -> bool {
        if self.line_end.is_none_or(|e| self.p > e) {
            let end = self.s[self.p..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(self.s.len(), |i| self.p + i);
            self.line_end = Some(end);
            for k in 0..4 {
                while self.dir_seen[k] < self.dirs[k].len() && self.dirs[k][self.dir_seen[k]].0 < end {
                    self.dir_seen[k] += 1;
                }
                self.dir_last[k] = self.dir_seen[k].checked_sub(1).map(|i| self.dirs[k][i]);
            }
        }
        for k in 0..4 {
            if let Some((start, end)) = self.dir_last[k] {
                if start >= self.p {
                    let end = self.s[end..].find('\n').map_or(self.s.len(), |i| end + i);
                    return self.emit(direction_tb + k as u8, end);
                }
            }
        }
        false
    }
    fn initial(&mut self) -> bool {
        if self.direction() {
            return true;
        }
        let c = char_at(self.s, self.p).unwrap();
        if is_space(c) {
            self.space(false);
            return true;
        }
        if self.s[self.p..].starts_with("%%") {
            self.p = self.until('\n');
            while matches!(self.s.as_bytes().get(self.p), Some(b'\n' | b'\r')) {
                self.p += 1;
            }
            return true;
        }
        if self.s[self.p..].starts_with("accTitle") || self.s[self.p..].starts_with("accDescr") {
            let title = self.s[self.p..].starts_with("accTitle");
            let q = self.skip(self.p + 8);
            if self.s.as_bytes().get(q) == Some(&b':') {
                let end = self.skip(q + 1);
                self.emit(if title { acc_title } else { acc_descr }, end);
                self.push(if title { State::Title } else { State::Descr });
                return true;
            }
            if !title && self.s.as_bytes().get(q) == Some(&b'{') {
                self.p = self.skip(q + 1);
                self.push(State::Multiline);
                return true;
            }
        }
        for (word, t) in [("classDiagram-v2", CLASS_DIAGRAM), ("classDiagram", CLASS_DIAGRAM), ("classDef", CLASSDEF), ("style", STYLE)] {
            if self.kw(word) {
                return self.emit(t, self.p + word.len());
            }
        }
        if self.s[self.p..].starts_with("call") && char_at(self.s, self.p + 4).is_some_and(is_space) {
            self.p = self.skip(self.p + 4);
            self.push(State::CallbackName);
            return true;
        }
        if self.kw("class") {
            self.push(State::Class);
            return self.emit(CLASS, self.p + 5);
        }
        if self.kw("namespace") {
            self.push(State::Namespace);
            return self.emit(NAMESPACE, self.p + 9);
        }
        if self.s[self.p..].starts_with("[*]") {
            return self.emit(EDGE_STATE, self.p + 3);
        }
        if c == ',' {
            return self.emit(COMMA, self.p + 1);
        }
        if c == '#' {
            return self.emit(BRKT, self.p + 1);
        }
        self.common() || (c == ':' && self.emit(COLON, self.p + 1))
    }
    fn run(mut self) -> Vec<Token> {
        loop {
            if self.p >= self.s.len() {
                let k = match self.state {
                    State::Initial | State::Backtick | State::Class | State::Namespace => EOF,
                    State::String => STR,
                    State::Generic => GENERICTYPE,
                    State::CallbackName => CALLBACK_NAME,
                    State::CallbackArgs => CALLBACK_ARGS,
                    State::Title => acc_title_value,
                    State::Descr => acc_descr_value,
                    State::Multiline => acc_descr_multiline_value,
                    State::ClassBody | State::NamespaceBody => EOF_IN_STRUCT,
                };
                self.emit(k, self.s.len());
                break;
            }
            let c = char_at(self.s, self.p).unwrap();
            let mut ok = true;
            match self.state {
                State::Initial => ok = self.initial(),
                State::Class => {
                    if is_space(c) {
                        self.space(true);
                    } else if c == '}' {
                        self.pop();
                        self.pop();
                        self.emit(STRUCT_STOP, self.p + 1);
                    } else if c == '{' {
                        self.push(State::ClassBody);
                        self.emit(STRUCT_START, self.p + 1);
                    } else {
                        ok = self.common();
                    }
                }
                State::ClassBody => {
                    if c == '"' {
                        self.push(State::String);
                        self.p += 1;
                    } else if c == '}' {
                        self.pop();
                        self.emit(STRUCT_STOP, self.p + 1);
                    } else if self.s[self.p..].starts_with("[*]") {
                        self.emit(EDGE_STATE, self.p + 3);
                    } else if c == '{' {
                        self.emit(OPEN_IN_STRUCT, self.p + 1);
                    } else if c == '\n' {
                        self.p += 1;
                    } else {
                        let next = self.p + c.len_utf8();
                        let end = self.s[next..].find(['{', '}', '\n']).map_or(self.s.len(), |i| next + i);
                        self.emit(MEMBER, end);
                    }
                }
                State::Namespace => {
                    if is_space(c) {
                        self.space(true);
                    } else if c == '{' {
                        self.push(State::NamespaceBody);
                        self.emit(STRUCT_START, self.p + 1);
                    } else if c == '}' {
                        self.pop();
                    } else if self.kw("namespace") {
                        self.push(State::Namespace);
                        self.emit(NAMESPACE, self.p + 9);
                    } else {
                        ok = self.common();
                    }
                }
                State::NamespaceBody => {
                    if is_space(c) {
                        self.space(false);
                    } else if c == '}' {
                        self.pop();
                        self.emit(STRUCT_STOP, self.p + 1);
                    } else if self.kw("namespace") {
                        self.push(State::Namespace);
                        self.emit(NAMESPACE, self.p + 9);
                    } else if self.kw("class") {
                        self.push(State::Class);
                        self.emit(CLASS, self.p + 5);
                    } else if self.s[self.p..].starts_with("[*]") {
                        self.emit(EDGE_STATE, self.p + 3);
                    } else {
                        ok = self.common();
                    }
                }
                State::String => {
                    if c == '"' {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = self.until('"');
                        self.emit(STR, e);
                    }
                }
                State::Generic | State::Backtick => {
                    let generic = matches!(self.state, State::Generic);
                    if c == '"' {
                        self.push(State::String);
                        self.p += 1;
                    } else if self.keyword() {
                    } else if c == '~' {
                        if generic {
                            self.pop();
                        } else {
                            self.push(State::Generic);
                        }
                        self.p += 1;
                    } else if !generic && c == '`' {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = self.until(if generic { '~' } else { '`' });
                        self.emit(if generic { GENERICTYPE } else { BQUOTE_STR }, e);
                    }
                }
                State::CallbackName => {
                    if c == '(' {
                        let q = self.skip(self.p + 1);
                        self.pop();
                        if self.s.as_bytes().get(q) == Some(&b')') {
                            self.p = q + 1;
                        } else {
                            self.push(State::CallbackArgs);
                            self.p += 1;
                        }
                    } else {
                        let e = self.until('(');
                        self.emit(CALLBACK_NAME, e);
                    }
                }
                State::CallbackArgs => {
                    if c == ')' {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = self.until(')');
                        self.emit(CALLBACK_ARGS, e);
                    }
                }
                State::Title | State::Descr => {
                    let k = if matches!(self.state, State::Title) { acc_title_value } else { acc_descr_value };
                    let e = self.until('\n');
                    self.emit(k, e);
                    self.pop();
                }
                State::Multiline => {
                    if c == '}' {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = self.until('}');
                        self.emit(acc_descr_multiline_value, e);
                    }
                }
            }
            if !ok {
                self.out.push(Token { kind: ERROR, text: String::new(), start: self.p });
                return self.out;
            }
        }
        self.emit(END, self.s.len());
        self.out
    }
}
pub fn tokenize(s: &str) -> Vec<Token> {
    let mut l = Lexer {
        s,
        p: 0,
        state: State::Initial,
        stack: vec![],
        out: vec![],
        dirs: std::array::from_fn(|_| vec![]),
        dir_seen: [0; 4],
        dir_last: [None; 4],
        line_end: None,
    };
    let mut from = 0;
    while let Some(off) = s[from..].find("direction") {
        let p = from + off;
        let q = l.skip(p + 9);
        if q > p + 9 {
            for (k, d) in ["TB", "BT", "RL", "LR"].iter().enumerate() {
                if s[q..].starts_with(d) {
                    l.dirs[k].push((p, q + 2));
                    break;
                }
            }
        }
        from = p + 9;
    }
    l.run()
}
