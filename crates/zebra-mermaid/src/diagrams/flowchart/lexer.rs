//! The flowchart lexer. Ported from Pele `src/diagrams/flowchart/lexer.ts`.
//!
//! Pele reads the source as UTF-16 code units with `charCodeAt`. This lexer reads UTF-8 bytes:
//! every delimiter it looks for is ASCII, so a byte test gives the answer the code unit test
//! gives, and the places that test a character outside ASCII (white space, Unicode letters)
//! decode the character there. A character outside the Basic Multilingual Plane is a pair of
//! surrogates to Pele, which no test accepts; [`code`] reports such a character as a lone
//! surrogate for the same answer. Positions are byte offsets.

use std::borrow::Cow;

use super::tokens::*;
use crate::util::unicode::is_unicode_letter;

pub struct Tokens<'a> {
    pub types: Vec<u8>,
    pub texts: Vec<Cow<'a, str>>,
    pub starts: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum S {
    Initial,
    String,
    MdString,
    AccTitle,
    AccDescr,
    AccDescrMultiline,
    Dir,
    Text,
    EllipseText,
    TrapText,
    EdgeText,
    ThickEdgeText,
    DottedEdgeText,
    Click,
    Callbackname,
    Callbackargs,
    ShapeData,
    ShapeDataStr,
}

/// What `charCodeAt` reads past the end of the string: NaN, which no test accepts.
const NONE: u32 = u32::MAX;

/// The UTF-16 code unit that Pele sees at byte `q`: the character for one in the Basic
/// Multilingual Plane, a high surrogate for one beyond it, and for a byte inside a character a
/// value that no test accepts.
#[inline]
fn code(s: &str, q: usize) -> u32 {
    let b = s.as_bytes();
    if q >= b.len() {
        return NONE;
    }
    let c = b[q];
    if c < 0x80 {
        c as u32
    } else if c >= 0xC0 {
        match s[q..].chars().next() {
            Some(ch) if (ch as u32) <= 0xFFFF => ch as u32,
            _ => 0xD800,
        }
    } else {
        0x80
    }
}

/// The length in bytes of the character at `q` (1 inside a character or past the end).
#[inline]
fn char_len(s: &str, q: usize) -> usize {
    let b = s.as_bytes();
    if q >= b.len() {
        return 1;
    }
    match b[q] {
        c if c < 0x80 => 1,
        c if c >= 0xF0 => 4,
        c if c >= 0xE0 => 3,
        c if c >= 0xC0 => 2,
        _ => 1,
    }
}

/// `isSpace` on a code unit: white space as `\s` sees it.
#[inline]
fn sp(c: u32) -> bool {
    if c < 128 {
        return c == 32 || (9..=13).contains(&c);
    }
    c == 0xa0
        || c == 0x1680
        || (0x2000..=0x200a).contains(&c)
        || c == 0x2028
        || c == 0x2029
        || c == 0x202f
        || c == 0x205f
        || c == 0x3000
        || c == 0xfeff
}

/// `isWord` on a code unit: `\w`.
#[inline]
fn word(c: u32) -> bool {
    c < 128 && ((c as u8).is_ascii_alphanumeric() || c == 95)
}

/// `\s*` from `q`: the end of the run of white space there.
#[inline]
fn skip_ws(s: &str, mut q: usize) -> usize {
    while q < s.len() && sp(code(s, q)) {
        q += char_len(s, q);
    }
    q
}

#[inline]
fn byte(s: &str, q: usize) -> u32 {
    s.as_bytes().get(q).map_or(NONE, |&b| b as u32)
}

/// `/word\s*sep\s*/y`: the accessibility openers.
fn keyword_then(s: &str, q: usize, word: &str, sep: u8) -> Option<usize> {
    if !s[q..].starts_with(word) {
        return None;
    }
    let e = skip_ws(s, q + word.len());
    if byte(s, e) != sep as u32 {
        return None;
    }
    Some(skip_ws(s, e + 1))
}

/// `/word\s+/y`
fn keyword_space(s: &str, q: usize, word: &str) -> Option<usize> {
    if !s[q..].starts_with(word) {
        return None;
    }
    let from = q + word.len();
    let e = skip_ws(s, from);
    if e > from {
        Some(e)
    } else {
        None
    }
}

/// `/(?:\r?\n)*\s*\n/y`: white space that holds a line break, up to the last break in it. The
/// run is all white space, and backtracking finds the last `\n` of the run whichever way the
/// two parts share it.
fn re_nodir(s: &str, q: usize) -> Option<usize> {
    let e = skip_ws(s, q);
    s[q..e].rfind('\n').map(|k| q + k + 1)
}

/// `/\s*(?:LR|RL|TB|BT|TD|BR)\b/y`
fn re_dir(s: &str, q: usize) -> Option<usize> {
    let at = skip_ws(s, q);
    let two = s.as_bytes().get(at..at + 2)?;
    if !matches!(two, b"LR" | b"RL" | b"TB" | b"BT" | b"TD" | b"BR") {
        return None;
    }
    if word(code(s, at + 2)) {
        return None;
    }
    Some(at + 2)
}

/// `/\s*[<>^]/y`
fn re_dir_sym(s: &str, q: usize) -> Option<usize> {
    let at = skip_ws(s, q);
    matches!(byte(s, at), 60 | 62 | 94).then_some(at + 1)
}

/// `/\s*v\b/y`
fn re_dir_v(s: &str, q: usize) -> Option<usize> {
    let at = skip_ws(s, q);
    (byte(s, at) == 118 && !word(code(s, at + 1))).then_some(at + 1)
}

/// The first line terminator (`[\n\r  ]`) at or after `q`, or the end.
fn line_terminator(s: &str, q: usize) -> usize {
    let b = s.as_bytes();
    let mut i = q;
    while i < b.len() {
        match b[i] {
            b'\n' | b'\r' => return i,
            0xE2 if b.get(i + 1) == Some(&0x80) && matches!(b.get(i + 2), Some(&0xA8) | Some(&0xA9)) => return i,
            _ => i += 1,
        }
    }
    b.len()
}

/// `/.*direction\s+XX[^\n]*/y` for one of the five directions. `.*` reaches to the end of the
/// line and gives back until a `direction` statement follows, so the last one on the line
/// counts; `[^\n]*` then reaches past `\r` and the other terminators to the next `\n`.
fn re_direction(s: &str, q: usize, dir: &str) -> Option<usize> {
    let limit = line_terminator(s, q);
    let line = &s[q..limit];
    let mut found: Option<usize> = None;
    let mut from = 0;
    while let Some(k) = line[from..].find("direction") {
        let at = q + from + k;
        let after = at + "direction".len();
        let e = skip_ws(s, after);
        if e > after && s[e..].starts_with(dir) {
            found = Some(e + dir.len());
        }
        from = from + k + 1;
    }
    let tail = found?;
    Some(s[tail..].find('\n').map_or(s.len(), |k| tail + k))
}

/// `[xo<]?` at `q`.
#[inline]
fn opt_head(s: &str, q: usize) -> usize {
    if matches!(byte(s, q), 120 | 111 | 60) {
        q + 1
    } else {
        q
    }
}

/// `/\s*[xo<]?--+[-xo>]\s*/y` and its thick form `==+[=xo>]`, with `ch` the line character.
fn re_link(s: &str, q: usize, ch: u8) -> Option<usize> {
    let at = opt_head(s, skip_ws(s, q));
    let b = s.as_bytes();
    let mut e = at;
    while e < b.len() && b[e] == ch {
        e += 1;
    }
    let run = e - at;
    if run < 2 {
        return None;
    }
    let end = if matches!(byte(s, e), 120 | 111 | 62) {
        e + 1
    } else if run >= 3 {
        // `--+` gives back its last dash to `[-xo>]`.
        e
    } else {
        return None;
    };
    Some(skip_ws(s, end))
}

/// `/\s*[xo<]?--\s*/y` and `/\s*[xo<]?==\s*/y`.
fn re_start_link(s: &str, q: usize, ch: u8) -> Option<usize> {
    let at = opt_head(s, skip_ws(s, q));
    if byte(s, at) == ch as u32 && byte(s, at + 1) == ch as u32 {
        Some(skip_ws(s, at + 2))
    } else {
        None
    }
}

/// `/\s*[xo<]?-?\.+-[xo>]?\s*/y`
fn re_dotted_link(s: &str, q: usize) -> Option<usize> {
    let mut at = opt_head(s, skip_ws(s, q));
    if byte(s, at) == 45 {
        at += 1;
    }
    let b = s.as_bytes();
    let mut e = at;
    while e < b.len() && b[e] == b'.' {
        e += 1;
    }
    if e == at || byte(s, e) != 45 {
        return None;
    }
    e += 1;
    if matches!(byte(s, e), 120 | 111 | 62) {
        e += 1;
    }
    Some(skip_ws(s, e))
}

/// `/\s*[xo<]?-\.\s*/y`
fn re_dotted_start(s: &str, q: usize) -> Option<usize> {
    let at = opt_head(s, skip_ws(s, q));
    if byte(s, at) == 45 && byte(s, at + 1) == 46 {
        Some(skip_ws(s, at + 2))
    } else {
        None
    }
}

/// `/\s*~~~+\s*/y`
fn re_invis_link(s: &str, q: usize) -> Option<usize> {
    let at = skip_ws(s, q);
    let b = s.as_bytes();
    let mut e = at;
    while e < b.len() && b[e] == b'~' {
        e += 1;
    }
    if e - at < 3 {
        return None;
    }
    Some(skip_ws(s, e))
}

/// `/(?:\r?\n)+/y`
fn re_newline(s: &str, q: usize) -> Option<usize> {
    let mut e = q;
    loop {
        if byte(s, e) == 13 && byte(s, e + 1) == 10 {
            e += 2;
        } else if byte(s, e) == 10 {
            e += 1;
        } else {
            break;
        }
    }
    (e > q).then_some(e)
}

/// `/[^\S\n\r]+/y`
fn re_space(s: &str, q: usize) -> Option<usize> {
    let mut e = q;
    while e < s.len() {
        let c = code(s, e);
        if !sp(c) || c == 10 || c == 13 {
            break;
        }
        e += char_len(s, e);
    }
    (e > q).then_some(e)
}

/// `/\(\s*\)/y`
fn re_empty_call(s: &str, q: usize) -> Option<usize> {
    if byte(s, q) != 40 {
        return None;
    }
    let e = skip_ws(s, q + 1);
    (byte(s, e) == 41).then_some(e + 1)
}

/// `.replace(/\n\s*/g, '<br/>')`
fn line_breaks(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 8);
    let mut i = 0;
    while i < text.len() {
        if text.as_bytes()[i] == b'\n' {
            out.push_str("<br/>");
            i = skip_ws(text, i + 1);
            continue;
        }
        let l = char_len(text, i);
        out.push_str(&text[i..i + l]);
        i += l;
    }
    Cow::Owned(out)
}

const NODE_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!\"#$%&'*+.`?\\_/";

fn node_char(c: u32) -> bool {
    c < 128 && NODE_CHARS.contains(&(c as u8))
}

struct Lexer<'a> {
    src: &'a str,
    n: usize,
    types: Vec<u8>,
    texts: Vec<Cow<'a, str>>,
    starts: Vec<usize>,
    stack: Vec<S>,
    state: S,
    p: usize,
    first_graph: bool,
    dir_next: Option<usize>,
    at_next: Option<usize>,
    line_end: isize,
    dir_failed: usize,
    run_end: usize,
    run_at: isize,
    ws_from: isize,
    ws_to: isize,
}

impl<'a> Lexer<'a> {
    fn c(&self, q: usize) -> u32 {
        code(self.src, q)
    }

    fn emit(&mut self, kind: u8, s: usize, e: usize) {
        self.types.push(kind);
        self.texts.push(Cow::Borrowed(&self.src[s..e]));
        self.starts.push(s);
        self.p = e;
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

    fn kw(&self, word: &str) -> bool {
        self.src[self.p..].starts_with(word) && !self::word(self.c(self.p + word.len()))
    }

    /// The end of the white space at `q`, remembered for the next call from inside it.
    fn ws_end(&mut self, q: usize) -> usize {
        let qi = q as isize;
        if qi >= self.ws_from && qi <= self.ws_to {
            self.ws_to as usize
        } else if sp(self.c(q)) {
            let j = skip_ws(self.src, q);
            self.ws_from = qi;
            self.ws_to = j as isize;
            j
        } else {
            q
        }
    }

    /// First character after any run of whitespace starting at q, skipping one optional x, o
    /// or <.
    fn link_char(&mut self, q: usize) -> u32 {
        let j = self.ws_end(q);
        let d = self.c(j);
        if d == 120 || d == 111 || d == 60 {
            self.c(j + 1)
        } else {
            d
        }
    }

    fn invis_ahead(&mut self, q: usize) -> bool {
        let j = self.ws_end(q);
        self.c(j) == 126
    }

    fn quote(&mut self) {
        if self.c(self.p + 1) == 96 {
            self.push(S::MdString);
            self.p += 2;
        } else {
            self.push(S::String);
            self.p += 1;
        }
    }

    fn invis_link(&mut self) -> bool {
        if !self.invis_ahead(self.p) {
            return false;
        }
        match re_invis_link(self.src, self.p) {
            Some(e) => {
                self.emit(LINK, self.p, e);
                true
            }
            None => false,
        }
    }

    fn opener(&mut self, c: u32) -> bool {
        let p = self.p;
        let d = self.c(p + 1);
        if c == 40 {
            if d == 45 {
                self.push(S::EllipseText);
                self.emit(ELLIPSE_START, p, p + 2);
            } else if d == 91 {
                self.push(S::Text);
                self.emit(STADIUMSTART, p, p + 2);
            } else if d == 40 && self.c(p + 2) == 40 {
                self.push(S::Text);
                self.emit(DOUBLECIRCLESTART, p, p + 3);
            } else {
                self.push(S::Text);
                self.emit(PS, p, p + 1);
            }
            return true;
        }
        if c == 91 {
            if d == 91 {
                self.push(S::Text);
                self.emit(SUBROUTINESTART, p, p + 2);
            } else if d == 40 {
                self.push(S::Text);
                self.emit(CYLINDERSTART, p, p + 2);
            } else if d == 47 {
                self.push(S::TrapText);
                self.emit(TRAPSTART, p, p + 2);
            } else if d == 92 {
                self.push(S::TrapText);
                self.emit(INVTRAPSTART, p, p + 2);
            } else {
                self.push(S::Text);
                self.emit(SQS, p, p + 1);
            }
            return true;
        }
        if c == 124 {
            self.push(S::Text);
            self.emit(PIPE, p, p + 1);
            return true;
        }
        if c == 123 {
            self.push(S::Text);
            self.emit(DIAMOND_START, p, p + 1);
            return true;
        }
        false
    }

    fn graph(&mut self, len: usize) {
        if self.first_graph {
            self.first_graph = false;
            self.push(S::Dir);
        }
        self.emit(GRAPH, self.p, self.p + len);
    }

    fn node_string(&mut self) -> bool {
        let p = self.p;
        let n = self.n;
        let mut q = p;
        while q < n {
            let ch = self.c(q);
            if node_char(ch) {
                q += 1;
            } else if ch == 45 && q + 1 < n {
                let d = self.c(q + 1);
                if d == 62 || d == 45 || d == 46 {
                    break;
                }
                q += 1;
            } else {
                break;
            }
        }
        if q == p {
            return false;
        }
        self.emit(NODE_STRING, p, q);
        true
    }

    fn link(&mut self, c: u32) -> bool {
        let d = self.link_char(self.p);
        let p = self.p;
        if d == 45 {
            if let Some(e) = re_link(self.src, p, b'-') {
                self.emit(LINK, p, e);
                return true;
            }
            if let Some(e) = re_start_link(self.src, p, b'-') {
                self.push(S::EdgeText);
                self.emit(START_LINK, p, e);
                return true;
            }
            if let Some(e) = re_dotted_link(self.src, p) {
                self.emit(LINK, p, e);
                return true;
            }
            if let Some(e) = re_dotted_start(self.src, p) {
                self.push(S::DottedEdgeText);
                self.emit(START_LINK, p, e);
                return true;
            }
        } else if d == 61 {
            if let Some(e) = re_link(self.src, p, b'=') {
                self.emit(LINK, p, e);
                return true;
            }
            if let Some(e) = re_start_link(self.src, p, b'=') {
                self.push(S::ThickEdgeText);
                self.emit(START_LINK, p, e);
                return true;
            }
        } else if d == 46 {
            if let Some(e) = re_dotted_link(self.src, p) {
                self.emit(LINK, p, e);
                return true;
            }
        }
        c != 45 && c != 61 && c != 46 && self.invis_link()
    }

    fn initial(&mut self, c: u32) -> bool {
        let p = self.p;
        let src = self.src;
        match c {
            97 => {
                if src[p..].starts_with("acc") {
                    if let Some(e) = keyword_then(src, p, "accTitle", b':') {
                        self.push(S::AccTitle);
                        self.emit(ACC_TITLE, p, e);
                        return true;
                    }
                    if let Some(e) = keyword_then(src, p, "accDescr", b':') {
                        self.push(S::AccDescr);
                        self.emit(ACC_DESCR, p, e);
                        return true;
                    }
                    if let Some(e) = keyword_then(src, p, "accDescr", b'{') {
                        self.push(S::AccDescrMultiline);
                        self.p = e;
                        return true;
                    }
                }
            }
            64 => {
                if self.c(p + 1) == 123 {
                    self.push(S::ShapeData);
                    self.types.push(SHAPE_DATA);
                    self.texts.push(Cow::Borrowed(""));
                    self.starts.push(p);
                    self.p += 2;
                    return true;
                }
            }
            99 => {
                if let Some(e) = keyword_space(src, p, "call") {
                    self.push(S::Callbackname);
                    self.p = e;
                    return true;
                }
                if self.kw("classDef") {
                    self.emit(CLASSDEF, p, p + 8);
                    return true;
                }
                if self.kw("class") {
                    self.emit(CLASS, p, p + 5);
                    return true;
                }
                if let Some(e) = keyword_space(src, p, "click") {
                    self.push(S::Click);
                    self.p = e;
                    return true;
                }
            }
            34 => {
                self.quote();
                return true;
            }
            115 => {
                if self.kw("style") {
                    self.emit(STYLE, p, p + 5);
                    return true;
                }
                if self.kw("swimlane-beta") {
                    self.graph(13);
                    return true;
                }
                if self.kw("subgraph") {
                    self.emit(SUBGRAPH_KW, p, p + 8);
                    return true;
                }
            }
            100 => {
                if self.kw("default") {
                    self.emit(DEFAULT, p, p + 7);
                    return true;
                }
            }
            108 => {
                if self.kw("linkStyle") {
                    self.emit(LINKSTYLE, p, p + 9);
                    return true;
                }
            }
            105 => {
                if self.kw("interpolate") {
                    self.emit(INTERPOLATE, p, p + 11);
                    return true;
                }
            }
            104 => {
                if src[p..].starts_with("href") && sp(self.c(p + 4)) {
                    self.emit(HREF, p, p + 5);
                    return true;
                }
            }
            102 => {
                if self.kw("flowchart-elk") {
                    self.graph(13);
                    return true;
                }
                if self.kw("flowchart") {
                    self.graph(9);
                    return true;
                }
            }
            103 => {
                if self.kw("graph") {
                    self.graph(5);
                    return true;
                }
            }
            101 => {
                if self.kw("end") {
                    let e = skip_ws(src, p + 3);
                    self.emit(END_KW, p, e);
                    return true;
                }
            }
            95 => {
                for word in ["_self", "_blank", "_parent", "_top"] {
                    if self.kw(word) {
                        self.emit(LINK_TARGET, p, p + word.len());
                        return true;
                    }
                }
            }
            _ => {}
        }

        if let Some(dir_next) = self.dir_next {
            if p >= self.dir_failed {
                let mut next = Some(dir_next);
                if dir_next < p {
                    next = src[p..].find("direction").map(|k| p + k);
                    self.dir_next = next;
                }
                if let Some(dir_next) = next {
                    if self.line_end < p as isize {
                        self.line_end = line_terminator(src, p) as isize;
                    }
                    if (dir_next as isize) < self.line_end {
                        for (i, dir) in ["TB", "BT", "RL", "LR", "TD"].iter().enumerate() {
                            if let Some(e) = re_direction(src, p, dir) {
                                self.emit(DIRECTION_TB + i as u8, p, e);
                                return true;
                            }
                        }
                        // No direction statement starts anywhere in the rest of this line either.
                        self.dir_failed = self.line_end as usize;
                    }
                }
            }
        }

        // A link id is a run of non-space characters up to its last `@` that is not followed by
        // `{` or `"`.
        if self.at_next.is_some() && c != 34 && !sp(c) {
            if p >= self.run_end {
                if self.at_next.is_some_and(|a| a < p) {
                    self.at_next = src[p..].find('@').map(|k| p + k);
                }
                let mut q = p;
                while q < self.n {
                    let ch = self.c(q);
                    if ch == 34 || sp(ch) {
                        break;
                    }
                    q += 1;
                }
                self.run_end = q;
                self.run_at = -1;
                if let Some(at_next) = self.at_next {
                    if at_next < q {
                        let b = src.as_bytes();
                        let mut k = q.min(self.n - 1) as isize - 1;
                        while k >= at_next as isize {
                            let ku = k as usize;
                            if b[ku] == b'@' {
                                let after = self.c(ku + 1);
                                if after != 123 && after != 34 {
                                    self.run_at = k;
                                    break;
                                }
                            }
                            k -= 1;
                        }
                    }
                }
            }
            if self.run_at > p as isize {
                self.emit(LINK_ID, p, self.run_at as usize + 1);
                return true;
            }
        }

        if (48..=57).contains(&c) {
            let mut q = p + 1;
            while q < self.n {
                let ch = self.c(q);
                if !(48..=57).contains(&ch) {
                    break;
                }
                q += 1;
            }
            self.emit(NUM, p, q);
            return true;
        }

        match c {
            35 => {
                self.emit(BRKT, p, p + 1);
                return true;
            }
            58 => {
                if self.c(p + 1) == 58 && self.c(p + 2) == 58 {
                    self.emit(STYLE_SEPARATOR, p, p + 3);
                } else {
                    self.emit(COLON, p, p + 1);
                }
                return true;
            }
            38 => {
                self.emit(AMP, p, p + 1);
                return true;
            }
            59 => {
                self.emit(SEMI, p, p + 1);
                return true;
            }
            44 => {
                self.emit(COMMA, p, p + 1);
                return true;
            }
            42 => {
                self.emit(MULT, p, p + 1);
                return true;
            }
            120 | 111 => return self.link(c) || self.node_string(),
            60 => {
                if self.link(c) {
                    return true;
                }
                self.emit(TAGSTART, p, p + 1);
                return true;
            }
            45 => {
                if self.link(c) || self.node_string() {
                    return true;
                }
                self.emit(MINUS, p, p + 1);
                return true;
            }
            61 => return self.link(c),
            46 => return self.link(c) || self.node_string(),
            126 => return self.invis_link(),
            40 => return self.opener(c),
            91 => {
                if self.c(p + 1) == 124 {
                    self.emit(VERTEX_WITH_PROPS_START, p, p + 2);
                    return true;
                }
                return self.opener(c);
            }
            124 | 123 => return self.opener(c),
            62 => {
                self.push(S::Text);
                self.emit(TAGEND, p, p + 1);
                return true;
            }
            94 => {
                self.emit(UP, p, p + 1);
                return true;
            }
            92 => {
                if self.c(p + 1) == 124 {
                    self.emit(SEP, p, p + 2);
                    return true;
                }
                return self.node_string();
            }
            118 => {
                if !word(self.c(p + 1)) {
                    self.emit(DOWN, p, p + 1);
                    return true;
                }
                return self.node_string();
            }
            _ => {}
        }

        if sp(c) {
            if self.link(c) {
                return true;
            }
            if let Some(e) = re_newline(src, p) {
                self.emit(NEWLINE, p, e);
                return true;
            }
            if let Some(e) = re_space(src, p) {
                self.emit(SPACE, p, e);
                return true;
            }
            return false;
        }

        if c < 128 {
            return self.node_string();
        }
        if is_unicode_letter(c) {
            let l = char_len(src, p);
            self.emit(UNICODE_TEXT, p, p + l);
            return true;
        }
        false
    }

    fn edge_text(&mut self, c: u32, link_re: fn(&str, usize) -> Option<usize>, a: u32, b: u32) -> bool {
        if c == 34 {
            self.quote();
            return true;
        }
        let p = self.p;
        let d = self.link_char(p);
        if d == a || d == b {
            if let Some(e) = link_re(self.src, p) {
                self.pop();
                self.emit(LINK, p, e);
                return true;
            }
        }
        if c == a && (a != 45 || self.c(p + 1) == 45) {
            return false;
        }
        let mut q = p + 1;
        while q < self.n {
            let ch = self.c(q);
            if ch == 34 {
                break;
            }
            if ch == a {
                if a != 45 || self.c(q + 1) == 45 {
                    break;
                }
            } else if ch == 120 || ch == 111 || ch == 60 || ch == b || sp(ch) {
                let dd = self.link_char(q);
                if (dd == a || dd == b) && link_re(self.src, q).is_some() {
                    break;
                }
            }
            q += 1;
        }
        self.emit(EDGE_TEXT, p, q);
        true
    }

    fn run(mut self) -> Tokens<'a> {
        loop {
            let p = self.p;
            let n = self.n;
            if p >= n {
                let kind = match self.state {
                    S::Initial => Some(EOF),
                    S::AccTitle => Some(ACC_TITLE_VALUE),
                    S::AccDescr => Some(ACC_DESCR_VALUE),
                    S::AccDescrMultiline => Some(ACC_DESCR_MULTILINE_VALUE),
                    S::Click => Some(CLICK),
                    S::Callbackname => Some(CALLBACKNAME),
                    S::Callbackargs => Some(CALLBACKARGS),
                    _ => None,
                };
                if let Some(kind) = kind {
                    self.emit(kind, n, n);
                }
                break;
            }

            let src = self.src;
            let c = self.c(p);
            let mut ok = true;

            match self.state {
                S::Initial => ok = self.initial(c),

                S::String => {
                    if c == 34 {
                        if self.c(p + 1) == 96 {
                            self.push(S::MdString);
                            self.p += 2;
                        } else {
                            self.pop();
                            self.p += 1;
                        }
                    } else {
                        let e = src[p..].find('"').map_or(n, |k| p + k);
                        self.emit(STR, p, e);
                    }
                }

                S::MdString => {
                    if c == 96 {
                        if self.c(p + 1) == 34 {
                            self.pop();
                            self.p += 2;
                        } else {
                            ok = false;
                        }
                    } else if c == 34 {
                        self.quote();
                    } else {
                        let mut q = p + 1;
                        while q < n {
                            let ch = self.c(q);
                            if ch == 96 || ch == 34 {
                                break;
                            }
                            q += 1;
                        }
                        self.emit(MD_STR, p, q);
                    }
                }

                S::AccTitle | S::AccDescr => {
                    let e = src[p..].find('\n').map_or(n, |k| p + k);
                    let kind = if self.state == S::AccTitle { ACC_TITLE_VALUE } else { ACC_DESCR_VALUE };
                    self.pop();
                    self.emit(kind, p, e);
                }

                S::AccDescrMultiline => {
                    if c == 125 {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = src[p..].find('}').map_or(n, |k| p + k);
                        self.emit(ACC_DESCR_MULTILINE_VALUE, p, e);
                    }
                }

                S::Dir => {
                    if c == 34 {
                        self.quote();
                    } else if let Some(e) = re_nodir(src, p) {
                        self.pop();
                        self.emit(NODIR, p, e);
                    } else if let Some(e) = re_dir(src, p).or_else(|| re_dir_sym(src, p)).or_else(|| re_dir_v(src, p)) {
                        self.pop();
                        self.emit(DIR, p, e);
                    } else {
                        ok = self.invis_link() || self.opener(c);
                    }
                }

                S::Text => {
                    let d = self.c(p + 1);
                    match c {
                        34 => self.quote(),
                        40 | 91 | 123 => {
                            self.opener(c);
                        }
                        93 => {
                            self.pop();
                            if d == 41 {
                                self.emit(STADIUMEND, p, p + 2);
                            } else if d == 93 {
                                self.emit(SUBROUTINEEND, p, p + 2);
                            } else {
                                self.emit(SQE, p, p + 1);
                            }
                        }
                        41 => {
                            self.pop();
                            if d == 93 {
                                self.emit(CYLINDEREND, p, p + 2);
                            } else if d == 41 && self.c(p + 2) == 41 {
                                self.emit(DOUBLECIRCLEEND, p, p + 3);
                            } else {
                                self.emit(PE, p, p + 1);
                            }
                        }
                        124 => {
                            self.pop();
                            self.emit(PIPE, p, p + 1);
                        }
                        125 => {
                            self.pop();
                            self.emit(DIAMOND_STOP, p, p + 1);
                        }
                        _ => {
                            if (c == 126 || sp(c)) && self.invis_link() {
                            } else {
                                let mut q = p + 1;
                                while q < n {
                                    let ch = self.c(q);
                                    if matches!(ch, 91 | 93 | 40 | 41 | 123 | 125 | 124 | 34) {
                                        break;
                                    }
                                    q += 1;
                                }
                                self.emit(TEXT, p, q);
                            }
                        }
                    }
                }

                S::EllipseText => {
                    let d = self.c(p + 1);
                    if c == 34 {
                        self.quote();
                    } else if (c == 126 || sp(c)) && self.invis_link() {
                    } else if (c == 45 || c == 47 || c == 41) && d == 41 {
                        self.pop();
                        self.emit(ELLIPSE_END, p, p + 2);
                    } else if c == 40 || c == 91 || c == 123 {
                        self.opener(c);
                    } else if c == 41 || c == 93 || c == 125 {
                        ok = false;
                    } else {
                        let mut q = p + 1;
                        while q < n {
                            let ch = self.c(q);
                            if matches!(ch, 34 | 40 | 41 | 91 | 93 | 123 | 125) {
                                break;
                            }
                            if (ch == 45 || ch == 47) && self.c(q + 1) == 41 {
                                break;
                            }
                            if (ch == 126 || sp(ch)) && self.invis_ahead(q) && re_invis_link(src, q).is_some() {
                                break;
                            }
                            q += 1;
                        }
                        self.emit(TEXT, p, q);
                    }
                }

                S::TrapText => {
                    let d = self.c(p + 1);
                    if c == 34 {
                        self.quote();
                    } else if (c == 126 || sp(c)) && self.invis_link() {
                    } else {
                        match c {
                            40 => {
                                if d == 45 || d == 91 || (d == 40 && self.c(p + 2) == 40) {
                                    self.opener(c);
                                } else if d == 93 {
                                    self.pop();
                                    self.emit(TRAPEND, p, p + 2);
                                } else {
                                    self.opener(c);
                                }
                            }
                            91 | 123 => {
                                self.opener(c);
                            }
                            92 => {
                                if d == 93 {
                                    self.pop();
                                    self.emit(TRAPEND, p, p + 2);
                                } else {
                                    self.emit(TEXT, p, p + 1);
                                }
                            }
                            47 => {
                                if d == 93 {
                                    self.pop();
                                    self.emit(INVTRAPEND, p, p + 2);
                                } else {
                                    self.emit(TEXT, p, p + 1);
                                }
                            }
                            93 | 41 => {
                                if d == 93 {
                                    self.pop();
                                    self.emit(TRAPEND, p, p + 2);
                                } else {
                                    ok = false;
                                }
                            }
                            125 => ok = false,
                            _ => {
                                if (c == 63 || c == 61) && d == 93 {
                                    self.pop();
                                    self.emit(TRAPEND, p, p + 2);
                                } else {
                                    let mut q = p + 1;
                                    while q < n {
                                        let ch = self.c(q);
                                        if matches!(ch, 92 | 91 | 93 | 40 | 41 | 123 | 125 | 47) {
                                            break;
                                        }
                                        q += 1;
                                    }
                                    self.emit(TEXT, p, q);
                                }
                            }
                        }
                    }
                }

                S::EdgeText => ok = self.edge_text(c, |s, q| re_link(s, q, b'-'), 45, NONE),
                S::ThickEdgeText => ok = self.edge_text(c, |s, q| re_link(s, q, b'='), 61, NONE),
                S::DottedEdgeText => ok = self.edge_text(c, re_dotted_link, 46, 45),

                S::Click => {
                    if c == 34 {
                        self.quote();
                    } else if sp(c) {
                        self.pop();
                        self.p += char_len(src, p);
                    } else {
                        let mut q = p + 1;
                        while q < n && !sp(self.c(q)) {
                            q += 1;
                        }
                        self.emit(CLICK, p, q);
                    }
                }

                S::Callbackname => {
                    if c == 40 {
                        let e = re_empty_call(src, p);
                        self.pop();
                        match e {
                            Some(e) => self.p = e,
                            None => {
                                self.push(S::Callbackargs);
                                self.p += 1;
                            }
                        }
                    } else {
                        let e = src[p..].find('(').map_or(n, |k| p + k);
                        self.emit(CALLBACKNAME, p, e);
                    }
                }

                S::Callbackargs => {
                    if c == 41 {
                        self.pop();
                        self.p += 1;
                    } else {
                        let e = src[p..].find(')').map_or(n, |k| p + k);
                        self.emit(CALLBACKARGS, p, e);
                    }
                }

                S::ShapeData => {
                    if c == 34 {
                        self.push(S::ShapeDataStr);
                        self.emit(SHAPE_DATA, p, p + 1);
                    } else if c == 125 {
                        self.pop();
                        self.p += 1;
                    } else if c == 94 {
                        ok = false;
                    } else {
                        let mut q = p + 1;
                        while q < n {
                            let ch = self.c(q);
                            if ch == 125 || ch == 94 || ch == 34 {
                                break;
                            }
                            q += 1;
                        }
                        self.emit(SHAPE_DATA, p, q);
                    }
                }

                S::ShapeDataStr => {
                    if c == 34 {
                        self.pop();
                        self.emit(SHAPE_DATA, p, p + 1);
                    } else {
                        let e = src[p..].find('"').map_or(n, |k| p + k);
                        self.types.push(SHAPE_DATA);
                        self.texts.push(line_breaks(&src[p..e]));
                        self.starts.push(p);
                        self.p = e;
                    }
                }
            }

            if !ok {
                self.types.push(ERROR);
                self.texts.push(Cow::Borrowed(""));
                self.starts.push(self.p);
                return Tokens { types: self.types, texts: self.texts, starts: self.starts };
            }
        }

        self.types.push(END);
        self.texts.push(Cow::Borrowed(""));
        self.starts.push(self.n);
        Tokens { types: self.types, texts: self.texts, starts: self.starts }
    }
}

pub fn tokenize(src: &str) -> Tokens<'_> {
    let lexer = Lexer {
        src,
        n: src.len(),
        types: Vec::new(),
        texts: Vec::new(),
        starts: Vec::new(),
        stack: Vec::new(),
        state: S::Initial,
        p: 0,
        first_graph: true,
        dir_next: src.find("direction"),
        at_next: src.find('@'),
        line_end: -1,
        dir_failed: 0,
        run_end: 0,
        run_at: -1,
        ws_from: -1,
        ws_to: -1,
    };
    lexer.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_patterns() {
        assert_eq!(re_link("-->", 0, b'-'), Some(3));
        assert_eq!(re_link(" --- x", 0, b'-'), Some(5));
        assert_eq!(re_link("--", 0, b'-'), None);
        assert_eq!(re_link("--x", 0, b'-'), Some(3));
        assert_eq!(re_link("<-->", 0, b'-'), Some(4));
        assert_eq!(re_link("x--o", 0, b'-'), Some(4));
        assert_eq!(re_link("==>", 0, b'='), Some(3));
        assert_eq!(re_start_link("-- a", 0, b'-'), Some(3));
        assert_eq!(re_dotted_link("-.->", 0), Some(4));
        assert_eq!(re_dotted_link("..-", 0), Some(3));
        assert_eq!(re_dotted_link("-..x", 0), None);
        assert_eq!(re_dotted_start("-. a", 0), Some(3));
        assert_eq!(re_invis_link(" ~~~ ", 0), Some(5));
        assert_eq!(re_invis_link("~~", 0), None);
    }

    #[test]
    fn line_patterns() {
        assert_eq!(re_nodir("  \n  \n x", 0), Some(6));
        assert_eq!(re_nodir("  x", 0), None);
        assert_eq!(re_newline("\r\n\n\r", 0), Some(3));
        assert_eq!(re_space(" \u{a0}\n", 0), Some(3));
        assert_eq!(re_dir(" LR;", 0), Some(3));
        assert_eq!(re_dir(" LRx", 0), None);
        assert_eq!(re_dir_v(" v\n", 0), Some(2));
        assert_eq!(re_direction("  direction LR\nx", 0, "LR"), Some(14));
        assert_eq!(re_direction("a direction LR direction TB;\n", 0, "TB"), Some(28));
        assert_eq!(re_direction("a direction LR direction TB;\n", 0, "LR"), Some(28));
        // The keyword's whitespace matcher includes a newline.
        assert_eq!(re_direction("direction\nLR", 0, "LR"), Some(12));
        assert_eq!(line_breaks("a\n  b\n"), "a<br/>b<br/>");
        assert_eq!(re_empty_call("( )", 0), Some(3));
    }

    #[test]
    fn surrogates_are_not_letters() {
        assert_eq!(code("😀", 0), 0xD800);
        assert_eq!(code("é", 1), 0x80);
        assert!(sp(code("\u{a0}", 0)));
    }
}
