//! Block diagram lexer. Ported from Pele `src/diagrams/block/lexer.ts` and `tokens.ts`.
//!
//! Pele reads UTF-16 code units with `charCodeAt`. Every test here is against an ASCII
//! character, except white space, which is decoded as a whole character: every character that
//! counts as white space lies in the Basic Multilingual Plane, so a code unit test and a
//! character test agree, and a byte of a multi-byte character never equals an ASCII byte.

use crate::diagrams::common::tokens::spaces;
use crate::util::chars::{char_at, is_space, is_word_byte};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum T {
    End,
    Error,
    Key,
    IdBlock,
    Columns,
    MdStr,
    Str,
    SpaceBlock,
    Default,
    LinkStyle,
    Interpolate,
    ClassDef,
    DefaultClassDefId,
    ClassDefId,
    ClassDefStyleOpts,
    Class,
    ClassEntityIds,
    StyleClass,
    Style,
    StyleEntityIds,
    StyleDefinitionData,
    AccTitle,
    AccTitleValue,
    AccDescr,
    AccDescrValue,
    AccDescrMultilineValue,
    EndKeyword,
    NodeDStart,
    NodeDEnd,
    BlockArrowStart,
    BlockArrowEnd,
    Dir,
    NodeId,
    Link,
    StartLink,
    LinkLabel,
    Size,
    Eof,
}

/// Mermaid's names of the tokens, as its parse errors give them.
pub(super) fn token_name(t: T) -> &'static str {
    match t {
        T::End => "$end",
        T::Error => "INVALID",
        T::Key => "BLOCK_DIAGRAM_KEY",
        T::IdBlock => "id-block",
        T::Columns => "COLUMNS",
        T::MdStr => "MD_STR",
        T::Str => "STR",
        T::SpaceBlock => "SPACE_BLOCK",
        T::Default => "DEFAULT",
        T::LinkStyle => "LINKSTYLE",
        T::Interpolate => "INTERPOLATE",
        T::ClassDef => "classDef",
        T::DefaultClassDefId => "DEFAULT_CLASSDEF_ID",
        T::ClassDefId => "CLASSDEF_ID",
        T::ClassDefStyleOpts => "CLASSDEF_STYLEOPTS",
        T::Class => "class",
        T::ClassEntityIds => "CLASSENTITY_IDS",
        T::StyleClass => "STYLECLASS",
        T::Style => "style",
        T::StyleEntityIds => "STYLE_ENTITY_IDS",
        T::StyleDefinitionData => "STYLE_DEFINITION_DATA",
        T::AccTitle => "acc_title",
        T::AccTitleValue => "acc_title_value",
        T::AccDescr => "acc_descr",
        T::AccDescrValue => "acc_descr_value",
        T::AccDescrMultilineValue => "acc_descr_multiline_value",
        T::EndKeyword => "end",
        T::NodeDStart => "NODE_DSTART",
        T::NodeDEnd => "NODE_DEND",
        T::BlockArrowStart => "BLOCK_ARROW_START",
        T::BlockArrowEnd => "BLOCK_ARROW_END",
        T::Dir => "DIR",
        T::NodeId => "NODE_ID",
        T::Link => "LINK",
        T::StartLink => "START_LINK",
        T::LinkLabel => "LINK_LABEL",
        T::Size => "SIZE",
        T::Eof => "EOF",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum S {
    Initial,
    String,
    MdString,
    Node,
    BlockArrow,
    ArrowDir,
    LLabel,
    ClassDef,
    ClassDefId,
    Class,
    ClassStyle,
    StyleStmnt,
    StyleDefinition,
    AccTitle,
    AccDescr,
    AccDescrMultiline,
}

const DIRECTIONS: [&str; 6] = ["right", "left", "x", "y", "up", "down"];

/// Characters that end a node id: ( ) [ { } < > - : = and white space.
fn id_stop(c: u8) -> bool {
    matches!(c, 9..=13 | 32 | 40 | 41 | 45 | 58 | 60 | 61 | 62 | 91 | 123 | 125)
}

pub(super) struct Tokens<'a> {
    pub types: Vec<T>,
    pub texts: Vec<&'a str>,
    pub starts: Vec<usize>,
}

struct Lexer<'a> {
    src: &'a str,
    b: &'a [u8],
    n: usize,
    types: Vec<T>,
    texts: Vec<&'a str>,
    starts: Vec<usize>,
    stack: Vec<S>,
    state: S,
    p: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            src,
            b: src.as_bytes(),
            n: src.len(),
            types: Vec::new(),
            texts: Vec::new(),
            starts: Vec::new(),
            stack: Vec::new(),
            state: S::Initial,
            p: 0,
        }
    }

    /// `charCodeAt(q)` compared with ASCII: the byte, or nothing past the end.
    #[inline]
    fn at(&self, q: usize) -> Option<u8> {
        self.b.get(q).copied()
    }

    #[inline]
    fn word_at(&self, q: usize) -> bool {
        self.b.get(q).is_some_and(|&c| is_word_byte(c))
    }

    fn put(&mut self, ty: T, text: &'a str, end: usize) -> bool {
        self.types.push(ty);
        self.texts.push(text);
        self.starts.push(self.p);
        self.p = end;
        true
    }

    fn emit(&mut self, ty: T, end: usize) -> bool {
        let text = &self.src[self.p..end];
        self.put(ty, text, end)
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

    fn ws(&self, q: usize) -> usize {
        spaces(self.src, q)
    }

    fn digits(&self, mut q: usize) -> usize {
        while q < self.n && self.b[q].is_ascii_digit() {
            q += 1;
        }
        q
    }

    fn word(&self, mut q: usize) -> usize {
        while q < self.n && is_word_byte(self.b[q]) {
            q += 1;
        }
        q
    }

    fn keyword(&self, text: &str) -> bool {
        self.src[self.p..].starts_with(text) && !self.word_at(self.p + text.len())
    }

    fn quote(&mut self) -> bool {
        if self.at(self.p + 1) == Some(b'`') {
            self.push(S::MdString);
            self.p += 2;
        } else {
            self.push(S::String);
            self.p += 1;
        }
        true
    }

    /// End of `.+-[xo>]?` at q.
    fn dotted(&self, mut q: usize) -> Option<usize> {
        if self.at(q) != Some(b'.') {
            return None;
        }
        loop {
            q += 1;
            if self.at(q) != Some(b'.') {
                break;
            }
        }
        if self.at(q) != Some(b'-') {
            return None;
        }
        match self.at(q + 1) {
            Some(b'x' | b'o' | b'>') => Some(q + 2),
            _ => Some(q + 1),
        }
    }

    /// End of a complete link at q, before its trailing white space.
    fn link_end(&self, mut q: usize) -> Option<usize> {
        if matches!(self.at(q), Some(b'x' | b'o' | b'<')) {
            q += 1;
        }
        let line = self.at(q);
        if line == Some(b'.') {
            return self.dotted(q);
        }
        let line = match line {
            Some(c @ (b'-' | b'=')) => c,
            _ => return None,
        };
        let mut k = q + 1;
        while self.at(k) == Some(line) {
            k += 1;
        }
        if k - q == 1 {
            return if line == b'-' { self.dotted(k) } else { None };
        }
        if matches!(self.at(k), Some(b'x' | b'o' | b'>')) {
            return Some(k + 1);
        }
        if k - q >= 3 {
            Some(k)
        } else {
            None
        }
    }

    fn link(&mut self) -> bool {
        if let Some(e) = self.link_end(self.p) {
            let end = self.ws(e);
            return self.emit(T::Link, end);
        }
        let q = if self.at(self.p) == Some(b'<') { self.p + 1 } else { self.p };
        let a = self.at(q);
        let b = self.at(q + 1);
        if (a == Some(b'-') && (b == Some(b'-') || b == Some(b'.'))) || (a == Some(b'=') && b == Some(b'=')) {
            self.push(S::LLabel);
            let end = self.ws(q + 2);
            return self.emit(T::StartLink, end);
        }
        false
    }

    fn line_state(&mut self, ty: T) -> bool {
        let e = self.src[self.p..].find('\n').map_or(self.n, |k| self.p + k);
        self.pop();
        self.emit(ty, e)
    }

    /// `\w+(,\s*\w+)*`
    fn id_list(&mut self, ty: T, next: S) -> bool {
        let mut k = self.word(self.p);
        if k == self.p {
            return false;
        }
        while self.at(k) == Some(b',') {
            let from = self.ws(k + 1);
            let to = self.word(from);
            if to == from {
                break;
            }
            k = to;
        }
        self.state = next;
        self.emit(ty, k)
    }

    fn initial(&mut self, c: u8) -> bool {
        let p = self.p;
        if char_at(self.src, p).is_some_and(is_space) {
            self.p = self.ws(p);
            return true;
        }
        match c {
            b'b' => {
                if self.src[p..].starts_with("block") {
                    if self.keyword("block-beta") {
                        return self.emit(T::Key, p + 10);
                    }
                    if self.at(p + 5) == Some(b':') {
                        return self.emit(T::IdBlock, p + 6);
                    }
                    if !self.word_at(p + 5) {
                        return self.emit(T::Key, p + 5);
                    }
                }
            }
            b'c' => {
                if self.src[p..].starts_with("columns") {
                    let q = self.ws(p + 7);
                    if q > p + 7 {
                        if self.src[q..].starts_with("auto") && !self.word_at(q + 4) {
                            return self.put(T::Columns, "-1", q + 4);
                        }
                        let e = self.digits(q);
                        if e > q {
                            let text = &self.src[q..e];
                            return self.put(T::Columns, text, e);
                        }
                    }
                } else if self.src[p..].starts_with("class") {
                    let def = self.src[p + 5..].starts_with("Def");
                    let from = if def { p + 8 } else { p + 5 };
                    let q = self.ws(from);
                    if q > from {
                        self.push(if def { S::ClassDef } else { S::Class });
                        return self.emit(if def { T::ClassDef } else { T::Class }, q);
                    }
                }
            }
            b'"' => return self.quote(),
            b's' => {
                if self.src[p..].starts_with("space") {
                    if self.at(p + 5) == Some(b':') {
                        let e = self.digits(p + 6);
                        if e > p + 6 {
                            let text = &self.src[p + 6..e];
                            return self.put(T::SpaceBlock, text, e);
                        }
                    }
                    if !self.word_at(p + 5) {
                        return self.put(T::SpaceBlock, "1", p + 5);
                    }
                } else if self.src[p..].starts_with("style") {
                    let q = self.ws(p + 5);
                    if q > p + 5 {
                        self.push(S::StyleStmnt);
                        return self.emit(T::Style, q);
                    }
                }
            }
            b'd' => {
                if self.keyword("default") {
                    return self.emit(T::Default, p + 7);
                }
            }
            b'l' => {
                if self.keyword("linkStyle") {
                    return self.emit(T::LinkStyle, p + 9);
                }
            }
            b'i' => {
                if self.keyword("interpolate") {
                    return self.emit(T::Interpolate, p + 11);
                }
            }
            b'a' => {
                if self.src[p..].starts_with("accTitle") {
                    let q = self.ws(p + 8);
                    if self.at(q) == Some(b':') {
                        self.push(S::AccTitle);
                        let end = self.ws(q + 1);
                        return self.emit(T::AccTitle, end);
                    }
                } else if self.src[p..].starts_with("accDescr") {
                    let q = self.ws(p + 8);
                    let d = self.at(q);
                    if d == Some(b':') {
                        self.push(S::AccDescr);
                        let end = self.ws(q + 1);
                        return self.emit(T::AccDescr, end);
                    }
                    if d == Some(b'{') {
                        self.push(S::AccDescrMultiline);
                        self.p = self.ws(q + 1);
                        return true;
                    }
                }
            }
            b'e' => {
                if self.keyword("end") {
                    let end = self.ws(p + 3);
                    return self.emit(T::EndKeyword, end);
                }
            }
            b'-' => {
                if self.at(p + 1) == Some(b')') {
                    self.push(S::Node);
                    return self.emit(T::NodeDStart, p + 2);
                }
                return self.link();
            }
            b'=' => return self.link(),
            b'<' => {
                if self.at(p + 1) == Some(b'[') {
                    self.push(S::BlockArrow);
                    return self.emit(T::BlockArrowStart, p + 2);
                }
                return self.link();
            }
            b'(' => {
                let d = self.at(p + 1);
                let len = match d {
                    Some(b'-' | b'[') => 2,
                    Some(b'(') => {
                        if self.at(p + 2) == Some(b'(') {
                            3
                        } else {
                            2
                        }
                    }
                    _ => 1,
                };
                self.push(S::Node);
                return self.emit(T::NodeDStart, p + len);
            }
            b')' | b'{' => {
                self.push(S::Node);
                let end = if self.at(p + 1) == Some(c) { p + 2 } else { p + 1 };
                return self.emit(T::NodeDStart, end);
            }
            b'>' => {
                self.push(S::Node);
                return self.emit(T::NodeDStart, p + 1);
            }
            b'[' => {
                let d = self.at(p + 1);
                self.push(S::Node);
                let len = if matches!(d, Some(b'[' | b'|' | b'(' | b'\\' | b'/')) { 2 } else { 1 };
                return self.emit(T::NodeDStart, p + len);
            }
            b':' => {
                let e = self.digits(p + 1);
                if e > p + 1 {
                    let text = &self.src[p + 1..e];
                    return self.put(T::Size, text, e);
                }
                return false;
            }
            _ => {}
        }

        let mut q = p;
        while q < self.n {
            let ch = self.b[q];
            if ch < 128 {
                if id_stop(ch) {
                    break;
                }
                q += 1;
            } else {
                let Some(full) = char_at(self.src, q) else { break };
                if is_space(full) {
                    break;
                }
                q += full.len_utf8();
            }
        }
        q > p && self.emit(T::NodeId, q)
    }

    fn node_end(&mut self, c: u8) -> bool {
        let p = self.p;
        let d = self.at(p + 1);
        let len = match c {
            b'"' => return self.quote(),
            b'(' => match d {
                Some(b'-') => 2,
                Some(b'(') => {
                    if self.at(p + 2) == Some(b'(') {
                        3
                    } else {
                        2
                    }
                }
                _ => 1,
            },
            b')' => match d {
                Some(b')') => {
                    if self.at(p + 2) == Some(b')') {
                        3
                    } else {
                        2
                    }
                }
                Some(b']') => 2,
                _ => 1,
            },
            b'}' => {
                if d == Some(b'}') {
                    2
                } else {
                    1
                }
            }
            b'-' => {
                if d == Some(b')') {
                    2
                } else {
                    0
                }
            }
            b']' => {
                if matches!(d, Some(b']' | b')' | b'>')) {
                    2
                } else {
                    1
                }
            }
            b'\\' | b'/' => {
                if d == Some(b']') {
                    2
                } else {
                    0
                }
            }
            _ => 0,
        };
        if len == 0 {
            return false;
        }
        self.pop();
        self.emit(T::NodeDEnd, p + len)
    }

    fn arrow_dir(&mut self, c: u8) -> bool {
        let p = self.p;
        if c == b')' {
            self.pop();
            self.pop();
            let end = self.ws(p + 1);
            return self.put(T::BlockArrowEnd, "]>", end);
        }
        let from = self.ws(if c == b',' { p + 1 } else { p });
        for dir in DIRECTIONS {
            if self.src[from..].starts_with(dir) {
                let e = self.ws(from + dir.len());
                let text = &self.src[if c == b',' { from } else { p }..e];
                return self.put(T::Dir, text, e);
            }
        }
        false
    }

    fn step(&mut self, c: u8) -> bool {
        let p = self.p;
        match self.state {
            S::Initial => self.initial(c),
            S::String => {
                if c == b'"' {
                    self.pop();
                    self.p += 1;
                } else {
                    let e = self.src[p..].find('"').map_or(self.n, |k| p + k);
                    self.emit(T::Str, e);
                }
                true
            }
            S::MdString => {
                if c == b'`' {
                    let ok = self.at(p + 1) == Some(b'"');
                    if ok {
                        self.pop();
                        self.p += 2;
                    }
                    ok
                } else if c == b'"' {
                    false
                } else {
                    // Bytes of a multi-byte character are never ASCII, so a byte scan finds the
                    // same quote or backtick that the code unit scan finds.
                    let mut q = p + 1;
                    while q < self.n {
                        let ch = self.b[q];
                        if ch == b'`' || ch == b'"' {
                            break;
                        }
                        q += 1;
                    }
                    self.emit(T::MdStr, q);
                    true
                }
            }
            S::Node => self.node_end(c),
            S::BlockArrow => {
                if c == b'"' {
                    self.quote();
                    true
                } else if c == b']' && self.at(p + 1) == Some(b'>') {
                    let q = self.ws(p + 2);
                    let ok = self.at(q) == Some(b'(');
                    if ok {
                        self.push(S::ArrowDir);
                        self.p = q + 1;
                    }
                    ok
                } else {
                    false
                }
            }
            S::ArrowDir => self.arrow_dir(c),
            S::LLabel => {
                if c == b'"' {
                    if self.at(p + 1) == Some(b'`') {
                        self.quote();
                    } else {
                        self.emit(T::LinkLabel, p + 1);
                        self.push(S::String);
                    }
                    true
                } else {
                    match self.link_end(self.ws(p)) {
                        Some(e) => {
                            self.pop();
                            let end = self.ws(e);
                            self.emit(T::Link, end);
                            true
                        }
                        None => false,
                    }
                }
            }
            S::ClassDef => {
                let mut ty = T::DefaultClassDefId;
                let mut k = if self.src[p..].starts_with("DEFAULT") { p + 7 } else { p };
                let mut q = self.ws(k);
                if k == p || q == k {
                    ty = T::ClassDefId;
                    k = self.word(p);
                    q = self.ws(k);
                }
                let ok = k > p && q > k;
                if ok {
                    self.state = S::ClassDefId;
                    self.emit(ty, q);
                }
                ok
            }
            S::ClassDefId => self.line_state(T::ClassDefStyleOpts),
            S::Class => self.id_list(T::ClassEntityIds, S::ClassStyle),
            S::ClassStyle => self.line_state(T::StyleClass),
            S::StyleStmnt => self.id_list(T::StyleEntityIds, S::StyleDefinition),
            S::StyleDefinition => self.line_state(T::StyleDefinitionData),
            S::AccTitle => self.line_state(T::AccTitleValue),
            S::AccDescr => self.line_state(T::AccDescrValue),
            S::AccDescrMultiline => {
                if c == b'}' {
                    self.pop();
                    self.p += 1;
                } else {
                    let e = self.src[p..].find('}').map_or(self.n, |k| p + k);
                    self.emit(T::AccDescrMultilineValue, e);
                }
                true
            }
        }
    }
}

pub(super) fn tokenize(src: &str) -> Tokens<'_> {
    let mut lx = Lexer::new(src);
    loop {
        if lx.p >= lx.n {
            let ty = match lx.state {
                S::Initial => Some(T::Eof),
                S::String => Some(T::Str),
                S::ClassDefId => Some(T::ClassDefStyleOpts),
                S::ClassStyle => Some(T::StyleClass),
                S::StyleDefinition => Some(T::StyleDefinitionData),
                S::AccTitle => Some(T::AccTitleValue),
                S::AccDescr => Some(T::AccDescrValue),
                S::AccDescrMultiline => Some(T::AccDescrMultilineValue),
                _ => None,
            };
            lx.p = lx.n;
            if let Some(ty) = ty {
                lx.put(ty, "", lx.n);
            }
            break;
        }
        let c = lx.b[lx.p];
        if !lx.step(c) {
            lx.types.push(T::Error);
            lx.texts.push("");
            lx.starts.push(lx.p);
            return Tokens { types: lx.types, texts: lx.texts, starts: lx.starts };
        }
    }
    lx.types.push(T::End);
    lx.texts.push("");
    lx.starts.push(lx.n);
    Tokens { types: lx.types, texts: lx.texts, starts: lx.starts }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(T, String)> {
        let t = tokenize(src);
        t.types.iter().zip(t.texts.iter()).map(|(&k, s)| (k, s.to_string())).collect()
    }

    #[test]
    fn links_and_labels() {
        let k = kinds("block\na --> b\nc -- \"x\" --> d\n");
        let types: Vec<T> = k.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            types,
            vec![T::Key, T::NodeId, T::Link, T::NodeId, T::NodeId, T::StartLink, T::LinkLabel, T::Str, T::Link, T::NodeId, T::Eof, T::End]
        );
        assert_eq!(k[2].1, "--> ");
        assert_eq!(k[7].1, "x");
    }

    #[test]
    fn link_shapes() {
        let end = |s: &'static str| Lexer::new(s).link_end(0);
        assert_eq!(end("-->"), Some(3));
        assert_eq!(end("---"), Some(3));
        assert_eq!(end("--"), None);
        assert_eq!(end("-.-"), Some(3));
        assert_eq!(end("-.->"), Some(4));
        assert_eq!(end("..->"), Some(4));
        assert_eq!(end("==>"), Some(3));
        assert_eq!(end("=="), None);
        assert_eq!(end("<-->"), Some(4));
        assert_eq!(end("x--x"), Some(4));
    }

    #[test]
    fn arrows_and_errors() {
        let k = kinds("block\n  a<[\"go\"]>(right, down)\n");
        let types: Vec<T> = k.iter().map(|(t, _)| *t).collect();
        assert_eq!(types, vec![T::Key, T::NodeId, T::BlockArrowStart, T::Str, T::Dir, T::Dir, T::BlockArrowEnd, T::Eof, T::End]);
        assert_eq!(k[5].1, "down");
        let k = kinds("block\n a[b]\n");
        assert_eq!(k.last().unwrap().0, T::Error);
        // A node id runs up to white space outside ASCII too.
        let k = kinds("block\n 中文\u{3000}x\n");
        assert_eq!(k[1].1, "中文");
        assert_eq!(k[2].1, "x");
    }
}
