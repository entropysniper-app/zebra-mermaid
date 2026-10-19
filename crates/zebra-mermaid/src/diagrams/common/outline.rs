//! The indented outline grammar that Mermaid's mindmap and kanban diagrams share. Kanban adds
//! `@{ ... }` metadata after a node and keeps `@` out of node ids. Ported from Pele
//! `src/diagrams/common/outline.ts`.
//!
//! Pele's lexer walks UTF-16 code units. This one walks bytes: every delimiter it tests is
//! ASCII, so a byte test answers as the code unit test does, and every run it scans ends on an
//! ASCII byte or on a whole character. Positions are byte offsets, which `syntax_error` turns
//! into UTF-16 columns; an indentation's level is counted in UTF-16 code units, as Pele counts
//! it.

use std::borrow::Cow;

use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::chars::{char_at, is_space};
use crate::util::js;

// Token types, numbered as in Pele.
pub const END: u8 = 0;
pub const ERROR: u8 = 1;
pub const SPACELINE: u8 = 2;
pub const NL: u8 = 3;
pub const EOF: u8 = 4;
pub const KEYWORD: u8 = 5;
pub const SPACELIST: u8 = 6;
pub const ICON: u8 = 7;
pub const CLASS: u8 = 8;
pub const NODE_DSTART: u8 = 9;
pub const NODE_ID: u8 = 10;
pub const NODE_DESCR: u8 = 11;
pub const NODE_DEND: u8 = 12;
pub const SHAPE_DATA: u8 = 13;

/// As the generated parser prints them. It has no name for the end of input and prints its number.
const NAMES: [&str; 14] = [
    "1",
    "INVALID",
    "SPACELINE",
    "NL",
    "EOF",
    "",
    "SPACELIST",
    "ICON",
    "CLASS",
    "NODE_DSTART",
    "NODE_ID",
    "NODE_DESCR",
    "NODE_DEND",
    "SHAPE_DATA",
];

pub fn token_name(kind: u8, kanban: bool) -> &'static str {
    if kind == KEYWORD {
        if kanban {
            "KANBAN"
        } else {
            "MINDMAP"
        }
    } else {
        NAMES.get(kind as usize).copied().unwrap_or("")
    }
}

/// The tokens of a source: type, text and byte offset of each.
#[derive(Debug, Clone, Default)]
pub struct Tokens<'a> {
    pub types: Vec<u8>,
    pub texts: Vec<Cow<'a, str>>,
    pub starts: Vec<usize>,
}

/// Mermaid's node types.
pub mod node_type {
    pub const DEFAULT: u8 = 0;
    pub const NO_BORDER: u8 = 0;
    pub const ROUNDED_RECT: u8 = 1;
    pub const RECT: u8 = 2;
    pub const CIRCLE: u8 = 3;
    pub const CLOUD: u8 = 4;
    pub const BANG: u8 = 5;
    pub const HEXAGON: u8 = 6;
}

/// Any opening delimiter pairs with any closing one, as in Mermaid.
pub fn get_type(start: &str, end: &str) -> u8 {
    match start {
        "[" => node_type::RECT,
        "(" => {
            if end == ")" {
                node_type::ROUNDED_RECT
            } else {
                node_type::CLOUD
            }
        }
        "((" => node_type::CIRCLE,
        ")" => node_type::CLOUD,
        "))" => node_type::BANG,
        "{{" => node_type::HEXAGON,
        _ => node_type::DEFAULT,
    }
}

/// What the grammar's actions do. Both diagrams take the node type from [`get_type`].
pub trait OutlineDb {
    fn add_node(&mut self, level: i64, id: &str, descr: &str, kind: u8, shape_data: Option<&str>) -> Result<(), Error>;
    fn decorate_node(&mut self, icon: Option<&str>, class: Option<&str>);
}

// Lexer states.
const S_INITIAL: u8 = 0;
const S_CLASS: u8 = 1;
const S_ICON: u8 = 2;
const S_NODE: u8 = 3;
const S_NSTR: u8 = 4;
const S_NSTR2: u8 = 5;
const S_DATA: u8 = 6;
const S_DATA_STR: u8 = 7;

/// The end of a run of bytes from `from` that `keep` accepts, or `None` for an empty run: a
/// sticky negated character class of ASCII characters, such as `/[^`"]+/y`.
fn run(b: &[u8], from: usize, keep: impl Fn(u8) -> bool) -> Option<usize> {
    let mut i = from;
    while i < b.len() && keep(b[i]) {
        i += 1;
    }
    (i > from).then_some(i)
}

/// `/\s+/y`
fn spaces_end(src: &str, from: usize) -> Option<usize> {
    let mut i = from;
    while let Some(c) = char_at(src, i) {
        if !is_space(c) {
            break;
        }
        i += c.len_utf8();
    }
    (i > from).then_some(i)
}

/// `/.+/y`: up to a line terminator (`\n`, `\r`, U+2028, U+2029).
fn line_end(b: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i < b.len() {
        match b[i] {
            b'\n' | b'\r' => break,
            0xe2 if b.get(i + 1) == Some(&0x80) && matches!(b.get(i + 2), Some(0xa8 | 0xa9)) => break,
            _ => i += 1,
        }
    }
    (i > from).then_some(i)
}

/// Jison's case-insensitive flag folds ASCII letters only.
fn letters(b: &[u8], p: usize, word: &str) -> bool {
    word.bytes().enumerate().all(|(k, w)| b.get(p + k).is_some_and(|&c| c | 32 == w))
}

/// `src.slice(p, e).replace(/\n\s*/g, '<br/>')`
fn breaks(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            out.push_str("<br/>");
            while chars.peek().is_some_and(|&n| is_space(n)) {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

struct Lexer<'a> {
    src: &'a str,
    out: Tokens<'a>,
    p: usize,
    state: u8,
}

impl<'a> Lexer<'a> {
    fn put(&mut self, kind: u8, text: Cow<'a, str>, end: usize) {
        self.out.types.push(kind);
        self.out.texts.push(text);
        self.out.starts.push(self.p);
        self.p = end;
    }

    fn emit(&mut self, kind: u8, end: usize, next: u8) {
        let src = self.src;
        self.put(kind, Cow::Borrowed(&src[self.p..end]), end);
        self.state = next;
    }

    fn up_to(&self, ch: u8) -> usize {
        let b = self.src.as_bytes();
        b[self.p..].iter().position(|&x| x == ch).map_or(b.len(), |k| self.p + k)
    }
}

pub fn tokenize(src: &str, kanban: bool) -> Tokens<'_> {
    let b = src.as_bytes();
    let n = b.len();
    let keyword = if kanban { "kanban" } else { "mindmap" };
    // `/[^([\n){}]+/y`, and `/[^([\n){}@]+/y` for kanban.
    let id_char = |c: u8| !(matches!(c, b'(' | b'[' | b'\n' | b')' | b'{' | b'}') || (kanban && c == b'@'));
    let mut lx = Lexer { src, out: Tokens::default(), p: 0, state: S_INITIAL };
    // The first line break at or after `p`, once looked for.
    let mut next_break: Option<usize> = None;

    while lx.p < n {
        let p = lx.p;
        let c = b[p];
        let d = b.get(p + 1).copied().unwrap_or(0);
        match lx.state {
            S_INITIAL => {
                // Pele tries `/\s+/y` only at a control, a space or a code unit from U+00A0 up.
                // No other character is white space, so trying it everywhere answers the same.
                let w = spaces_end(src, p).unwrap_or(p);
                if kanban && c == b'@' && d == b'{' {
                    lx.put(SHAPE_DATA, Cow::Borrowed(""), p + 2);
                    lx.state = S_DATA;
                } else if b.get(w) == Some(&b'%') && b.get(w + 1) == Some(&b'%') {
                    let end = line_end(b, w).unwrap_or(w);
                    lx.emit(SPACELINE, end, S_INITIAL);
                } else if w > p {
                    let nb = match next_break {
                        Some(nb) if nb >= p => nb,
                        _ => {
                            let nb = lx.up_to(b'\n');
                            next_break = Some(nb);
                            nb
                        }
                    };
                    if nb >= w {
                        lx.emit(SPACELIST, w, S_INITIAL);
                    } else {
                        // `src.lastIndexOf('\n', w - 1)`, which finds at least the break at `nb`.
                        let e = b[..w].iter().rposition(|&x| x == b'\n').unwrap_or(nb);
                        if e > p {
                            lx.emit(SPACELINE, e + 1, S_INITIAL);
                        } else {
                            lx.emit(NL, p + 1, S_INITIAL);
                        }
                    }
                } else if letters(b, p, keyword) && !b.get(p + keyword.len()).is_some_and(|&x| crate::util::chars::is_word_byte(x)) {
                    lx.emit(KEYWORD, p + keyword.len(), S_INITIAL);
                } else if c == b':' && d == b':' && b.get(p + 2) == Some(&b':') {
                    lx.p += 3;
                    lx.state = S_CLASS;
                } else if c == b':' && d == b':' && letters(b, p + 2, "icon") && b.get(p + 6) == Some(&b'(') {
                    lx.p += 7;
                    lx.state = S_ICON;
                } else if (c == b'-' && d == b')') || (c == b'(' && (d == b'-' || d == b'(')) || ((c == b')' || c == b'{') && d == c) {
                    lx.emit(NODE_DSTART, p + 2, S_NODE);
                } else if c == b')' || c == b'(' || c == b'[' {
                    lx.emit(NODE_DSTART, p + 1, S_NODE);
                } else if let Some(e) = run(b, p, id_char) {
                    lx.emit(NODE_ID, e, S_INITIAL);
                } else {
                    break;
                }
            }
            S_NODE => {
                if c == b'"' {
                    lx.state = if d == b'`' { S_NSTR2 } else { S_NSTR };
                    lx.p += if d == b'`' { 2 } else { 1 };
                } else if c == b')' || c == b'(' {
                    let end = if d == c || (c == b'(' && d == b'-') { p + 2 } else { p + 1 };
                    lx.emit(NODE_DEND, end, S_INITIAL);
                } else if c == b']' {
                    lx.emit(NODE_DEND, p + 1, S_INITIAL);
                } else if (c == b'}' && d == b'}') || (c == b'-' && d == b')') {
                    lx.emit(NODE_DEND, p + 2, S_INITIAL);
                } else {
                    // A lone closing brace takes the rest of its line as text. Otherwise
                    // `/[^)\](}]+/y`, which the tests above leave at least one character.
                    let end = if c == b'}' { line_end(b, p) } else { run(b, p, |x| !matches!(x, b')' | b']' | b'(' | b'}')) };
                    match end {
                        Some(e) => lx.emit(NODE_DESCR, e, S_NODE),
                        None => break,
                    }
                }
            }
            S_NSTR => {
                if c == b'"' {
                    lx.p += 1;
                    lx.state = S_NODE;
                } else {
                    let e = lx.up_to(b'"');
                    lx.emit(NODE_DESCR, e, S_NSTR);
                }
            }
            S_NSTR2 => {
                if c == b'`' && d == b'"' {
                    lx.p += 2;
                    lx.state = S_NODE;
                } else if let Some(e) = run(b, p, |x| x != b'`' && x != b'"') {
                    lx.emit(NODE_DESCR, e, S_NSTR2);
                } else {
                    break;
                }
            }
            S_CLASS => {
                if let Some(e) = line_end(b, p) {
                    lx.emit(CLASS, e, S_INITIAL);
                } else if c == b'\n' {
                    lx.p += 1;
                    lx.state = S_INITIAL;
                } else {
                    break;
                }
            }
            S_ICON => {
                if c == b')' {
                    lx.p += 1;
                    lx.state = S_INITIAL;
                } else {
                    let e = lx.up_to(b')');
                    lx.emit(ICON, e, S_ICON);
                }
            }
            S_DATA => {
                if c == b'"' {
                    lx.emit(SHAPE_DATA, p + 1, S_DATA_STR);
                } else if c == b'}' {
                    lx.p += 1;
                    lx.state = S_INITIAL;
                } else if let Some(e) = run(b, p, |x| !matches!(x, b'}' | b'^' | b'"')) {
                    lx.emit(SHAPE_DATA, e, S_DATA);
                } else {
                    break;
                }
            }
            _ => {
                // Inside a quoted string of metadata.
                if c == b'"' {
                    lx.emit(SHAPE_DATA, p + 1, S_DATA);
                } else {
                    let e = lx.up_to(b'"');
                    lx.put(SHAPE_DATA, breaks(&src[p..e]), e);
                }
            }
        }
    }

    if lx.p < n {
        lx.put(ERROR, Cow::Borrowed(""), lx.p);
    } else {
        if lx.state == S_INITIAL {
            lx.put(EOF, Cow::Borrowed(""), n);
        }
        lx.put(END, Cow::Borrowed(""), n);
    }
    lx.out
}

const STOP: &str = "'SPACELINE', 'NL', 'EOF'";

fn is_start(kind: u8) -> bool {
    matches!(kind, SPACELINE | SPACELIST | ICON | CLASS | NODE_DSTART | NODE_ID)
}

fn is_stop(kind: u8) -> bool {
    kind == SPACELINE || kind == NL || kind == EOF
}

/// Makes the calls on `db` that the grammar's actions make, in the same order, and reports
/// errors in the state where the generated parser would.
pub fn parse_outline(src: &str, db: &mut dyn OutlineDb, kanban: bool) -> Result<(), Error> {
    let diagram = if kanban { DiagramType::Kanban } else { DiagramType::Mindmap };
    let Tokens { types, texts, starts } = tokenize(src, kanban);
    let data = if kanban { ", 'SHAPE_DATA'" } else { "" };
    // The list always ends with END or ERROR, and the parser stops at either.
    let kind = |i: usize| types.get(i).copied().unwrap_or(END);
    let fail = |i: usize, expected: &str| -> Error {
        let t = kind(i);
        let at = starts.get(i).copied().unwrap_or(src.len());
        if t == ERROR {
            return syntax_error(diagram, src, at, "", true);
        }
        syntax_error(diagram, src, at, &format!("Expecting {}, got '{}'", expected, token_name(t, kanban)), false)
    };
    let mut i = 0;

    let keyword = format!("'{}'", token_name(KEYWORD, kanban));
    if kind(i) == SPACELINE {
        i += 1;
        while kind(i) == SPACELINE || kind(i) == NL {
            i += 1;
        }
        if kind(i) != KEYWORD {
            return Err(fail(i, &format!("'SPACELINE', 'NL', {}", keyword)));
        }
    } else if kind(i) != KEYWORD {
        return Err(fail(i, &format!("'SPACELINE', {}", keyword)));
    }
    i += 1;
    let mut first = "'SPACELINE', 'NL', 'SPACELIST', 'ICON', 'CLASS', 'NODE_DSTART', 'NODE_ID'";
    if kind(i) == NL {
        i += 1;
        first = "'SPACELINE', 'SPACELIST', 'ICON', 'CLASS', 'NODE_DSTART', 'NODE_ID'";
    }

    loop {
        let mut t = kind(i);
        let mut level = 0i64;
        let indented = t == SPACELIST;
        if indented {
            level = js::len16(&texts[i]) as i64;
            i += 1;
            t = kind(i);
        }
        if t == NODE_ID || t == NODE_DSTART {
            let mut id: Option<&str> = None;
            let descr: &str;
            let mut node_kind = 0;
            if t == NODE_ID {
                id = Some(texts[i].as_ref());
                i += 1;
            }
            if kind(i) == NODE_DSTART {
                let start: &str = texts[i].as_ref();
                i += 1;
                if kind(i) != NODE_DESCR {
                    return Err(fail(i, "'NODE_DESCR'"));
                }
                descr = texts[i].as_ref();
                i += 1;
                if kind(i) != NODE_DEND {
                    return Err(fail(i, "'NODE_DEND'"));
                }
                let end: &str = texts[i].as_ref();
                i += 1;
                if !is_stop(kind(i)) && kind(i) != SHAPE_DATA {
                    return Err(fail(i, &format!("{}{}", STOP, data)));
                }
                node_kind = get_type(start, end);
            } else {
                if !is_stop(kind(i)) && kind(i) != SHAPE_DATA {
                    return Err(fail(i, &format!("{}, 'NODE_DSTART'{}", STOP, data)));
                }
                descr = id.unwrap_or_default();
            }
            let id = id.unwrap_or(descr);
            if kind(i) == SHAPE_DATA {
                let mut shape_data = texts[i].to_string();
                i += 1;
                while kind(i) == SHAPE_DATA {
                    shape_data.push_str(&texts[i]);
                    i += 1;
                }
                if !is_stop(kind(i)) {
                    return Err(fail(i, &format!("{}{}", STOP, data)));
                }
                db.add_node(level, id, descr, node_kind, Some(&shape_data))?;
            } else {
                db.add_node(level, id, descr, node_kind, None)?;
            }
        } else if t == ICON || t == CLASS {
            let text: &str = texts[i].as_ref();
            i += 1;
            if !is_stop(kind(i)) {
                return Err(fail(i, STOP));
            }
            if t == ICON {
                db.decorate_node(Some(text), None);
            } else {
                db.decorate_node(None, Some(text));
            }
        } else if indented {
            if !is_stop(t) {
                return Err(fail(i, &format!("{}, 'ICON', 'CLASS', 'NODE_DSTART', 'NODE_ID'", STOP)));
            }
        } else if t == SPACELINE {
            i += 1;
            if !is_stop(kind(i)) {
                return Err(fail(i, STOP));
            }
        } else {
            return Err(fail(i, first));
        }

        i += 1;
        while kind(i) == NL || kind(i) == EOF {
            i += 1;
        }
        if kind(i) == END {
            return Ok(());
        }
        if !is_start(kind(i)) {
            return Err(fail(i, &format!("{}, 'SPACELIST', 'ICON', 'CLASS', 'NODE_DSTART', 'NODE_ID'", STOP)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str, kanban: bool) -> Vec<(u8, String, usize)> {
        let t = tokenize(src, kanban);
        (0..t.types.len()).map(|i| (t.types[i], t.texts[i].to_string(), t.starts[i])).collect()
    }

    #[test]
    fn indentation_and_breaks() {
        let t = kinds("mindmap\n  root\n\n    a\n", false);
        let types: Vec<u8> = t.iter().map(|x| x.0).collect();
        assert_eq!(types, vec![KEYWORD, NL, SPACELIST, NODE_ID, SPACELINE, SPACELIST, NODE_ID, NL, EOF, END]);
        assert_eq!(t[4].1, "\n\n");
        assert_eq!(t[5].1, "    ");
    }

    #[test]
    fn delimiters_and_strings() {
        let t = kinds("mindmap\nid((\"a `b`\"))\n", false);
        let types: Vec<u8> = t.iter().map(|x| x.0).collect();
        assert_eq!(types, vec![KEYWORD, NL, NODE_ID, NODE_DSTART, NODE_DESCR, NODE_DEND, NL, EOF, END]);
        assert_eq!(t[4].1, "a `b`");
        let t = kinds("mindmap\nx[\"`**md**`\"]\n", false);
        assert_eq!(t[4].1, "**md**");
        // A lone closing brace takes the rest of the line.
        let t = kinds("mindmap\nx(a}b)c\n", false);
        assert_eq!(t[4].1, "a");
        assert_eq!(t[5].1, "}b)c");
    }

    #[test]
    fn keyword_needs_a_word_end() {
        let t = kinds("MindMap\nmindmapx\n", false);
        assert_eq!(t[0].0, KEYWORD);
        assert_eq!(t[2].0, NODE_ID);
        assert_eq!(t[2].1, "mindmapx");
    }

    #[test]
    fn decorations() {
        let t = kinds("mindmap\na\n::ICON(fa fa-book)\n:::big  red\n", false);
        let types: Vec<u8> = t.iter().map(|x| x.0).collect();
        assert_eq!(types, vec![KEYWORD, NL, NODE_ID, NL, ICON, NL, CLASS, NL, EOF, END]);
        assert_eq!(t[4].1, "fa fa-book");
        assert_eq!(t[6].1, "big  red");
    }

    #[test]
    fn kanban_metadata() {
        let t = kinds("kanban\na@{ label: \"x\n   y\", b: 1 }\n", true);
        let data: Vec<String> = t.iter().filter(|x| x.0 == SHAPE_DATA).map(|x| x.1.clone()).collect();
        assert_eq!(data, vec!["", " label: ", "\"", "x<br/>y", "\"", ", b: 1 "]);
        // `@` ends an id only in kanban.
        assert_eq!(kinds("mindmap\na@b\n", false)[2].1, "a@b");
        assert_eq!(kinds("kanban\na@b\n", true)[3].0, ERROR);
    }

    #[test]
    fn non_ascii_spaces_count_utf16_units() {
        let t = kinds("mindmap\n\u{3000}\u{3000}a\n", false);
        assert_eq!(t[2].0, SPACELIST);
        assert_eq!(js::len16(&t[2].1), 2);
    }

    #[test]
    fn line_end_stops_at_line_separators() {
        // The comment takes the white space before it too.
        let t = kinds("mindmap\n%% a\u{2028}b\n", false);
        assert_eq!(t[1].0, SPACELINE);
        assert_eq!(t[1].1, "\n%% a");
        // U+2028 is white space to `\s`, so it indents what follows.
        assert_eq!((t[2].0, t[3].0), (SPACELIST, NODE_ID));
    }
}
