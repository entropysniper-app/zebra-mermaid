//! Agentflow lexer with Jison line/column accounting and retained comments.
use super::{tokens::*, types::Loc};
use crate::util::{chars::is_space_code, unicode::is_unicode_letter};
#[derive(Clone, Copy, PartialEq, Eq)]
enum S {
    Initial,
    String,
    MdString,
    AccTitle,
    AccDescr,
    AccMultiline,
    Dir,
    Text,
    Ellipse,
    Trap,
    EdgeText,
    Click,
    CallbackName,
    CallbackArgs,
    ShapeData,
    ShapeString,
}
pub struct Tokens {
    pub types: Vec<u8>,
    pub texts: Vec<String>,
    pub starts: Vec<usize>,
    pub locs: Vec<Loc>,
}
struct Lexer {
    src: Vec<u16>,
    p: usize,
    line: usize,
    col: usize,
    out: Tokens,
    state: S,
    stack: Vec<S>,
    first_graph: bool,
    dir_next: Option<usize>,
    at_next: Option<usize>,
    line_end: isize,
    dir_failed: usize,
    run_end: usize,
    run_at: Option<usize>,
    ws: Option<(usize, usize)>,
    blank: Option<(usize, usize)>,
    no_link: Option<usize>,
    no_arrow: Option<usize>,
}
fn space(c: u16) -> bool {
    is_space_code(Some(c as u32))
}
fn word(c: u16) -> bool {
    c < 128 && ((c as u8).is_ascii_alphanumeric() || c == 95)
}
fn blank(c: u16) -> bool {
    matches!(c, 32 | 9)
}
impl Lexer {
    fn n(&self) -> usize {
        self.src.len()
    }
    fn c(&self, i: usize) -> u16 {
        self.src.get(i).copied().unwrap_or(u16::MAX)
    }
    fn starts(&self, i: usize, s: &str) -> bool {
        self.src.get(i..i + s.len()).is_some_and(|v| v.iter().zip(s.bytes()).all(|(&a, b)| a == b as u16))
    }
    fn slice(&self, a: usize, b: usize) -> String {
        String::from_utf16_lossy(&self.src[a..b])
    }
    fn find(&self, what: &str, from: usize) -> Option<usize> {
        (from..self.n()).find(|&i| self.starts(i, what))
    }
    fn advance(&mut self, end: usize) {
        let mut brk = None;
        let mut k = self.p;
        while k < end {
            let ch = self.c(k);
            if ch == 10 {
                self.line += 1;
                brk = Some(k);
            } else if ch == 13 {
                if k + 1 < end && self.c(k + 1) == 10 {
                    k += 1;
                }
                self.line += 1;
                brk = Some(k);
            }
            k += 1;
        }
        if let Some(brk) = brk {
            let mut k = brk + 1;
            while k < end && !matches!(self.c(k), 0x2028 | 0x2029) {
                k += 1;
            }
            self.col = k - brk - 1;
        } else {
            self.col += end - self.p;
        }
        self.p = end;
    }
    fn emit_mode(&mut self, kind: u8, end: usize, mode: u8, override_text: Option<String>) {
        let start = self.p;
        let first_line = self.line;
        let first_column = self.col;
        self.out.types.push(kind);
        self.out.texts.push(override_text.unwrap_or_else(|| self.slice(start, end)));
        self.out.starts.push(start);
        if mode == 1 {
            self.col += end - self.p;
            self.p = end;
        } else if mode == 2 {
            for k in self.p..end {
                if matches!(self.c(k), 10 | 13) {
                    self.line += 1;
                    self.col = 0;
                } else {
                    self.col += 1;
                }
            }
            self.p = end;
        } else {
            self.advance(end);
        }
        self.out.locs.push(Loc { first_line, first_column, last_line: self.line, last_column: self.col });
    }
    fn emit(&mut self, k: u8, e: usize) {
        self.emit_mode(k, e, 0, None)
    }
    fn flat(&mut self, k: u8, e: usize) {
        self.emit_mode(k, e, 1, None)
    }
    fn chars(&mut self, k: u8, e: usize) {
        self.emit_mode(k, e, 2, None)
    }
    fn emitted_text(&mut self, k: u8, text: String, e: usize) {
        self.emit_mode(k, e, 0, Some(text))
    }
    fn push(&mut self, s: S) {
        self.stack.push(self.state);
        self.state = s;
    }
    fn pop(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.state = s;
        }
    }
    fn kw(&self, w: &str) -> bool {
        self.starts(self.p, w) && !word(self.c(self.p + w.len()))
    }
    fn ws_end(&mut self, q: usize) -> usize {
        if let Some((from, to)) = self.ws {
            if q >= from && q <= to {
                return to;
            }
        }
        let mut j = q;
        while j < self.n() && space(self.c(j)) {
            j += 1;
        }
        self.ws = Some((q, j));
        j
    }
    fn blank_end(&mut self, q: usize) -> usize {
        if let Some((from, to)) = self.blank {
            if q >= from && q <= to {
                return to;
            }
        }
        let mut j = q;
        while j < self.n() && blank(self.c(j)) {
            j += 1;
        }
        self.blank = Some((q, j));
        j
    }
    fn arrow_end(&mut self, q: usize) -> Option<usize> {
        let j = self.ws_end(q);
        if self.no_arrow == Some(j) || self.c(j) != 45 || self.c(j + 1) != 45 {
            return None;
        }
        let mut k = j + 2;
        while self.c(k) == 45 {
            k += 1;
        }
        if matches!(self.c(k), 120 | 62) {
            return Some(self.ws_end(k + 1));
        }
        self.no_arrow = Some(j);
        None
    }
    fn link(&mut self) -> bool {
        let j = self.ws_end(self.p);
        if self.no_link == Some(j) || self.c(j) != 45 {
            return false;
        }
        let d = self.c(j + 1);
        let mut k = j + 2;
        if d == 45 {
            while self.c(k) == 45 {
                k += 1;
            }
            if matches!(self.c(k), 120 | 62) {
                let e = self.ws_end(k + 1);
                self.emit(LINK, e);
            } else {
                self.push(S::EdgeText);
                let e = self.ws_end(j + 2);
                self.emit(START_LINK, e);
            }
            return true;
        }
        if d == 46 {
            while self.c(k) == 46 {
                k += 1;
            }
            if self.c(k) == 45 {
                let e = self.ws_end(k + 1);
                self.emit(LINK, e);
                return true;
            }
        }
        self.no_link = Some(j);
        false
    }
    fn comment_end(&self, q: usize) -> Option<usize> {
        if self.c(q) != 37 || self.c(q + 1) != 37 || self.c(q + 2) == 123 {
            return None;
        }
        Some(self.find("\n", q + 2).unwrap_or(self.n()))
    }
    fn quote(&mut self) {
        if self.c(self.p + 1) == 96 {
            self.push(S::MdString);
            self.advance(self.p + 2);
        } else {
            self.push(S::String);
            self.advance(self.p + 1);
        }
    }
    fn opener(&mut self, c: u16) -> bool {
        let p = self.p;
        let d = self.c(p + 1);
        let (k, e, state) = match c {
            40 => {
                if d == 45 {
                    (ELLIPSE_START, p + 2, S::Ellipse)
                } else if d == 91 {
                    (STADIUMSTART, p + 2, S::Text)
                } else if d == 40 && self.c(p + 2) == 40 {
                    (DOUBLECIRCLESTART, p + 3, S::Text)
                } else {
                    (PS, p + 1, S::Text)
                }
            }
            91 => match d {
                91 => (SUBROUTINESTART, p + 2, S::Text),
                40 => (CYLINDERSTART, p + 2, S::Text),
                47 => (TRAPSTART, p + 2, S::Trap),
                92 => (INVTRAPSTART, p + 2, S::Trap),
                _ => (SQS, p + 1, S::Text),
            },
            124 => (PIPE, p + 1, S::Text),
            123 => (DIAMOND_START, p + 1, S::Text),
            _ => return false,
        };
        self.push(state);
        self.flat(k, e);
        true
    }
    fn node_string(&mut self) -> bool {
        let mut q = self.p;
        while q < self.n() {
            let ch = self.c(q);
            if ch < 128 && b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!\"#$%&'*+.`?\\_/".contains(&(ch as u8)) {
                q += 1;
            } else if ch == 45 && q + 1 < self.n() {
                if matches!(self.c(q + 1), 62 | 45 | 46) {
                    break;
                }
                q += 1;
            } else {
                break;
            }
        }
        if q == self.p {
            return false;
        }
        self.flat(NODE_STRING, q);
        true
    }
    fn keyword_then(&mut self, w: &str, sep: u16) -> Option<usize> {
        if !self.starts(self.p, w) {
            return None;
        }
        let j = self.ws_end(self.p + w.len());
        if self.c(j) != sep {
            return None;
        }
        Some(self.ws_end(j + 1))
    }
    fn keyword_space(&mut self, w: &str) -> Option<usize> {
        if !self.starts(self.p, w) {
            return None;
        }
        let start = self.p + w.len();
        let end = self.ws_end(start);
        (end > start).then_some(end)
    }
    fn direction_statement(&mut self, dir: &str) -> Option<usize> {
        let mut p = self.p;
        let mut found = None;
        let limit = (p..self.n()).find(|&i| matches!(self.c(i), 10 | 13 | 0x2028 | 0x2029)).unwrap_or(self.n());
        while let Some(at) = self.find("direction", p) {
            if at >= limit {
                break;
            }
            let end = self.ws_end(at + 9);
            if end > at + 9 && self.starts(end, dir) {
                found = Some(end + 2);
            }
            p = at + 1;
        }
        found.map(|end| self.find("\n", end).unwrap_or(self.n()))
    }
    fn initial(&mut self, c: u16) -> bool {
        let p = self.p;
        match c {
            97 => {
                if self.starts(p, "acc") {
                    if let Some(e) = self.keyword_then("accTitle", 58) {
                        self.push(S::AccTitle);
                        self.emit(ACC_TITLE, e);
                        return true;
                    }
                    if let Some(e) = self.keyword_then("accDescr", 58) {
                        self.push(S::AccDescr);
                        self.emit(ACC_DESCR, e);
                        return true;
                    }
                    if let Some(e) = self.keyword_then("accDescr", 123) {
                        self.push(S::AccMultiline);
                        self.advance(e);
                        return true;
                    }
                }
                if self.kw("agentflow-beta") {
                    if self.first_graph {
                        self.first_graph = false;
                        self.push(S::Dir);
                    }
                    self.flat(GRAPH, p + 14);
                    return true;
                }
            }
            64 => {
                if self.c(p + 1) == 123 {
                    self.push(S::ShapeData);
                    self.emitted_text(SHAPE_DATA, String::new(), p + 2);
                    return true;
                }
            }
            99 => {
                if let Some(e) = self.keyword_space("call") {
                    self.push(S::CallbackName);
                    self.advance(e);
                    return true;
                }
                if self.kw("classDef") {
                    self.flat(CLASSDEF, p + 8);
                    return true;
                }
                if self.kw("class") {
                    self.flat(CLASS, p + 5);
                    return true;
                }
                if let Some(e) = self.keyword_space("click") {
                    self.push(S::Click);
                    self.advance(e);
                    return true;
                }
                if self.kw("connector") {
                    self.flat(CONNECTOR_KW, p + 9);
                    return true;
                }
            }
            34 => {
                self.quote();
                return true;
            }
            115 => {
                if self.kw("style") {
                    self.flat(STYLE, p + 5);
                    return true;
                }
            }
            100 => {
                if self.kw("default") {
                    self.flat(DEFAULT, p + 7);
                    return true;
                }
            }
            108 => {
                if self.kw("linkStyle") {
                    self.flat(LINKSTYLE, p + 9);
                    return true;
                }
            }
            105 => {
                if self.kw("interpolate") {
                    self.flat(INTERPOLATE, p + 11);
                    return true;
                }
            }
            104 => {
                if self.starts(p, "href") && space(self.c(p + 4)) {
                    self.emit(HREF, p + 5);
                    return true;
                }
            }
            102 => {
                if self.kw("flow") {
                    self.flat(FLOW_KW, p + 4);
                    return true;
                }
            }
            103 => {
                if self.kw("global") {
                    let e = self.blank_end(p + 6);
                    self.flat(GLOBAL_KW, e);
                    return true;
                }
            }
            101 => {
                if self.kw("end") {
                    let e = self.blank_end(p + 3);
                    self.flat(END_KW, e);
                    return true;
                }
            }
            95 => {
                for w in ["_self", "_blank", "_parent", "_top"] {
                    if self.kw(w) {
                        self.flat(LINK_TARGET, p + w.len());
                        return true;
                    }
                }
            }
            _ => {}
        }
        if self.dir_next.is_some() && p >= self.dir_failed {
            if self.dir_next.is_some_and(|q| q < p) {
                self.dir_next = self.find("direction", p);
            }
            if let Some(next) = self.dir_next {
                if self.line_end < p as isize {
                    self.line_end = (p..self.n()).find(|&i| matches!(self.c(i), 10 | 13 | 0x2028 | 0x2029)).unwrap_or(self.n()) as isize;
                }
                if next < (self.line_end as usize) {
                    for (i, dir) in ["TB", "BT", "RL", "LR", "TD"].iter().enumerate() {
                        if let Some(e) = self.direction_statement(dir) {
                            self.emit(DIRECTION_TB + i as u8, e);
                            return true;
                        }
                    }
                    self.dir_failed = self.line_end as usize;
                }
            }
        }
        if self.at_next.is_some() && c != 34 && !space(c) {
            if p >= self.run_end {
                if self.at_next.is_some_and(|q| q < p) {
                    self.at_next = self.find("@", p);
                }
                let mut q = p;
                while q < self.n() && self.c(q) != 34 && !space(self.c(q)) {
                    q += 1;
                }
                self.run_end = q;
                self.run_at = None;
                if let Some(next) = self.at_next {
                    if next < q {
                        let top = q.min(self.n().saturating_sub(1));
                        for k in (next..top).rev() {
                            if self.c(k) == 64 && !matches!(self.c(k + 1), 123 | 34) {
                                self.run_at = Some(k);
                                break;
                            }
                        }
                    }
                }
            }
            if let Some(k) = self.run_at.filter(|&k| k > p) {
                self.flat(LINK_ID, k + 1);
                return true;
            }
        }
        if (48..=57).contains(&c) {
            let mut q = p + 1;
            while (48..=57).contains(&self.c(q)) {
                q += 1;
            }
            self.flat(NUM, q);
            return true;
        }
        match c {
            35 => {
                self.flat(BRKT, p + 1);
                return true;
            }
            58 => {
                if self.c(p + 1) == 58 && self.c(p + 2) == 58 {
                    self.flat(STYLE_SEPARATOR, p + 3);
                } else {
                    self.flat(COLON, p + 1);
                }
                return true;
            }
            38 => {
                self.flat(AMP, p + 1);
                return true;
            }
            59 => {
                self.flat(SEMI, p + 1);
                return true;
            }
            44 => {
                self.flat(COMMA, p + 1);
                return true;
            }
            42 => {
                self.flat(MULT, p + 1);
                return true;
            }
            60 => {
                self.flat(TAGSTART, p + 1);
                return true;
            }
            45 => {
                if self.link() || self.node_string() {
                    return true;
                }
                self.flat(MINUS, p + 1);
                return true;
            }
            40 | 124 | 123 => return self.opener(c),
            91 => {
                if self.c(p + 1) == 124 {
                    self.flat(VERTEX_WITH_PROPS_START, p + 2);
                    return true;
                }
                return self.opener(c);
            }
            62 => {
                self.push(S::Text);
                self.flat(TAGEND, p + 1);
                return true;
            }
            94 => {
                self.flat(UP, p + 1);
                return true;
            }
            92 => {
                if self.c(p + 1) == 124 {
                    self.flat(SEP, p + 2);
                    return true;
                }
                return self.node_string();
            }
            118 => {
                if !word(self.c(p + 1)) {
                    self.flat(DOWN, p + 1);
                    return true;
                }
                return self.node_string();
            }
            37 => {
                if let Some(e) = self.comment_end(p) {
                    self.emit(COMMENT, e);
                    return true;
                }
                return self.node_string();
            }
            _ => {}
        }
        if space(c) {
            if self.link() {
                return true;
            }
            if blank(c) {
                let q = self.blank_end(p);
                if let Some(e) = self.comment_end(q) {
                    self.emit(COMMENT, e);
                    return true;
                }
            }
            let mut e = p;
            loop {
                if self.c(e) == 10 {
                    e += 1;
                } else if self.c(e) == 13 && self.c(e + 1) == 10 {
                    e += 2;
                } else {
                    break;
                }
            }
            if e > p {
                self.emit(NEWLINE, e);
            } else {
                self.emit(SPACE, p + 1);
            }
            return true;
        }
        if c < 128 {
            return self.node_string();
        }
        if is_unicode_letter(c as u32) {
            self.flat(UNICODE_TEXT, p + 1);
            return true;
        }
        false
    }
    fn run(mut self) -> Tokens {
        loop {
            if self.p >= self.n() {
                let k = match self.state {
                    S::Initial => Some(EOF),
                    S::AccTitle => Some(ACC_TITLE_VALUE),
                    S::AccDescr => Some(ACC_DESCR_VALUE),
                    S::AccMultiline => Some(ACC_DESCR_MULTILINE_VALUE),
                    S::Click => Some(CLICK),
                    S::CallbackName => Some(CALLBACKNAME),
                    S::CallbackArgs => Some(CALLBACKARGS),
                    _ => None,
                };
                if let Some(k) = k {
                    self.flat(k, self.n());
                }
                break;
            }
            let p = self.p;
            let c = self.c(p);
            let d = self.c(p + 1);
            let n = self.n();
            let mut ok = true;
            match self.state {
                S::Initial => ok = self.initial(c),
                S::String => {
                    if c == 34 {
                        if d == 96 {
                            self.push(S::MdString);
                            self.advance(p + 2);
                        } else {
                            self.pop();
                            self.advance(p + 1);
                        }
                    } else {
                        let e = self.find("\"", p).unwrap_or(n);
                        self.emit(STR, e);
                    }
                }
                S::MdString => {
                    if c == 96 {
                        if d == 34 {
                            self.pop();
                            self.advance(p + 2);
                        } else {
                            ok = false;
                        }
                    } else if c == 34 {
                        self.quote();
                    } else {
                        let mut q = p + 1;
                        while q < n && !matches!(self.c(q), 96 | 34) {
                            q += 1;
                        }
                        self.emit(MD_STR, q);
                    }
                }
                S::AccTitle | S::AccDescr => {
                    let e = self.find("\n", p).unwrap_or(n);
                    let k = if self.state == S::AccTitle { ACC_TITLE_VALUE } else { ACC_DESCR_VALUE };
                    self.pop();
                    self.emit(k, e);
                }
                S::AccMultiline => {
                    if c == 125 {
                        self.pop();
                        self.advance(p + 1);
                    } else {
                        let e = self.find("}", p).unwrap_or(n);
                        self.emit(ACC_DESCR_MULTILINE_VALUE, e);
                    }
                }
                S::Dir => {
                    if c == 34 {
                        self.quote();
                    } else {
                        let e = self.ws_end(p);
                        let last = (p..e).rev().find(|&i| self.c(i) == 10).map(|i| i + 1);
                        if let Some(e) = last {
                            self.pop();
                            self.emit(NODIR, e);
                        } else {
                            let valid = ["LR", "RL", "TB", "BT", "TD", "BR"].iter().any(|s| self.starts(e, s)) && !word(self.c(e + 2));
                            let end = if valid {
                                Some(e + 2)
                            } else if matches!(self.c(e), 60 | 62 | 94) || (self.c(e) == 118 && !word(self.c(e + 1))) {
                                Some(e + 1)
                            } else {
                                None
                            };
                            if let Some(e) = end {
                                self.pop();
                                self.emit(DIR, e);
                            } else {
                                ok = self.opener(c);
                            }
                        }
                    }
                }
                S::Text => match c {
                    34 => self.quote(),
                    40 | 91 | 123 => {
                        self.opener(c);
                    }
                    93 => {
                        self.pop();
                        self.flat(
                            if d == 41 {
                                STADIUMEND
                            } else if d == 93 {
                                SUBROUTINEEND
                            } else {
                                SQE
                            },
                            p + if matches!(d, 41 | 93) { 2 } else { 1 },
                        );
                    }
                    41 => {
                        self.pop();
                        if d == 93 {
                            self.flat(CYLINDEREND, p + 2);
                        } else if d == 41 && self.c(p + 2) == 41 {
                            self.flat(DOUBLECIRCLEEND, p + 3);
                        } else {
                            self.flat(PE, p + 1);
                        }
                    }
                    124 => {
                        self.pop();
                        self.flat(PIPE, p + 1);
                    }
                    125 => {
                        self.pop();
                        self.flat(DIAMOND_STOP, p + 1);
                    }
                    37 => {
                        if let Some(e) = self.comment_end(p) {
                            self.advance(e);
                        } else {
                            ok = false;
                        }
                    }
                    _ => {
                        let mut q = p + 1;
                        while q < n && !matches!(self.c(q), 91 | 93 | 40 | 41 | 123 | 125 | 124 | 34 | 37) {
                            q += 1;
                        }
                        self.emit(TEXT, q);
                    }
                },
                S::Ellipse => {
                    if c == 34 {
                        self.quote();
                    } else if c == 37 && self.comment_end(p).is_some() {
                        self.advance(self.comment_end(p).unwrap());
                    } else if matches!(c, 45 | 47 | 41) && d == 41 {
                        self.pop();
                        self.flat(ELLIPSE_END, p + 2);
                    } else if matches!(c, 40 | 91 | 123) {
                        self.opener(c);
                    } else if matches!(c, 41 | 93 | 125) {
                        ok = false;
                    } else {
                        let mut q = p + 1;
                        while q < n {
                            let ch = self.c(q);
                            if matches!(ch, 34 | 40 | 41 | 91 | 93 | 123 | 125)
                                || (matches!(ch, 45 | 47) && self.c(q + 1) == 41)
                                || (ch == 37 && self.c(q + 1) == 37 && self.c(q + 2) != 123)
                            {
                                break;
                            }
                            q += 1;
                        }
                        self.chars(TEXT, q);
                    }
                }
                S::Trap => {
                    if c == 34 {
                        self.quote();
                    } else {
                        match c {
                            40 => {
                                if d == 45 || d == 91 || (d == 40 && self.c(p + 2) == 40) {
                                    self.opener(c);
                                } else if d == 93 {
                                    self.pop();
                                    self.flat(TRAPEND, p + 2);
                                } else {
                                    self.opener(c);
                                }
                            }
                            91 | 123 => {
                                self.opener(c);
                            }
                            92 | 47 => {
                                if d == 93 {
                                    self.pop();
                                    self.flat(if c == 92 { TRAPEND } else { INVTRAPEND }, p + 2);
                                } else {
                                    self.flat(TEXT, p + 1);
                                }
                            }
                            93 | 41 => {
                                if d == 93 {
                                    self.pop();
                                    self.flat(TRAPEND, p + 2);
                                } else {
                                    ok = false;
                                }
                            }
                            125 => ok = false,
                            37 => {
                                if let Some(e) = self.comment_end(p) {
                                    self.advance(e);
                                } else {
                                    ok = false;
                                }
                            }
                            _ => {
                                if matches!(c, 63 | 61) && d == 93 {
                                    self.pop();
                                    self.flat(TRAPEND, p + 2);
                                } else {
                                    let mut q = p + 1;
                                    while q < n && !matches!(self.c(q), 92 | 91 | 93 | 40 | 41 | 123 | 125 | 47 | 37) {
                                        q += 1;
                                    }
                                    self.emit(TEXT, q);
                                }
                            }
                        }
                    }
                }
                S::EdgeText => {
                    if c == 34 {
                        self.quote();
                    } else if let Some(e) = self.arrow_end(p) {
                        self.pop();
                        self.emit(LINK, e);
                    } else if c == 45 && d == 45 {
                        ok = false;
                    } else {
                        let mut q = p + 1;
                        while q < n {
                            let ch = self.c(q);
                            if ch == 34 || (ch == 45 && self.c(q + 1) == 45) {
                                break;
                            }
                            if ch != 45 && space(ch) && self.arrow_end(q).is_some() {
                                break;
                            }
                            q += 1;
                        }
                        self.chars(EDGE_TEXT, q);
                    }
                }
                S::Click => {
                    if c == 34 {
                        self.quote();
                    } else if space(c) {
                        self.pop();
                        self.advance(p + 1);
                    } else {
                        let mut q = p + 1;
                        while q < n && !space(self.c(q)) {
                            q += 1;
                        }
                        self.flat(CLICK, q);
                    }
                }
                S::CallbackName => {
                    if c == 40 {
                        let at = self.ws_end(p + 1);
                        self.pop();
                        if self.c(at) == 41 {
                            self.advance(at + 1);
                        } else {
                            self.push(S::CallbackArgs);
                            self.advance(p + 1);
                        }
                    } else {
                        let e = self.find("(", p).unwrap_or(n);
                        self.emit(CALLBACKNAME, e);
                    }
                }
                S::CallbackArgs => {
                    if c == 41 {
                        self.pop();
                        self.advance(p + 1);
                    } else {
                        let e = self.find(")", p).unwrap_or(n);
                        self.emit(CALLBACKARGS, e);
                    }
                }
                S::ShapeData => {
                    if c == 34 {
                        self.push(S::ShapeString);
                        self.flat(SHAPE_DATA, p + 1);
                    } else if c == 125 {
                        self.pop();
                        self.advance(p + 1);
                    } else if c == 94 {
                        ok = false;
                    } else {
                        let mut q = p + 1;
                        while q < n && !matches!(self.c(q), 125 | 94 | 34) {
                            q += 1;
                        }
                        self.emit(SHAPE_DATA, q);
                    }
                }
                S::ShapeString => {
                    if c == 34 {
                        self.pop();
                        self.flat(SHAPE_DATA, p + 1);
                    } else {
                        let e = self.find("\"", p).unwrap_or(n);
                        let mut s = String::new();
                        let mut at = p;
                        let mut from = p;
                        while at < e {
                            if self.c(at) == 10 {
                                s.push_str(&self.slice(from, at));
                                s.push_str("<br/>");
                                at += 1;
                                while at < e && space(self.c(at)) {
                                    at += 1;
                                }
                                from = at;
                            } else {
                                at += 1;
                            }
                        }
                        s.push_str(&self.slice(from, e));
                        self.emitted_text(SHAPE_DATA, s, e);
                    }
                }
            }
            if !ok {
                self.out.types.push(ERROR);
                self.out.texts.push(String::new());
                self.out.starts.push(self.p);
                self.out.locs.push(Loc { first_line: self.line, first_column: self.col, last_line: self.line, last_column: self.col });
                return self.out;
            }
        }
        self.out.types.push(END);
        self.out.texts.push(String::new());
        self.out.starts.push(self.n());
        self.out.locs.push(Loc { first_line: self.line, first_column: self.col, last_line: self.line, last_column: self.col });
        self.out
    }
}
pub fn tokenize(src: &str) -> Tokens {
    let mut lex = Lexer {
        src: src.encode_utf16().collect(),
        p: 0,
        line: 1,
        col: 0,
        out: Tokens { types: vec![], texts: vec![], starts: vec![], locs: vec![] },
        state: S::Initial,
        stack: vec![],
        first_graph: true,
        dir_next: None,
        at_next: None,
        line_end: -1,
        dir_failed: 0,
        run_end: 0,
        run_at: None,
        ws: None,
        blank: None,
        no_link: None,
        no_arrow: None,
    };
    lex.dir_next = lex.find("direction", 0);
    lex.at_next = lex.find("@", 0);
    lex.run()
}
