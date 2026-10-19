//! The C4 lexer. Ported from Pele `src/diagrams/c4/lexer.ts` and `tokens.ts`.
//!
//! Offsets are bytes. Every boundary the lexer stops at is an ASCII delimiter, the end of a run
//! of white space or the end of the input, so the tokens slice the source on characters.

use crate::util::chars::{char_at, is_line_end, is_space, is_word_byte};

pub(super) const END: u8 = 0;
pub(super) const ERROR: u8 = 1;
pub(super) const DIRECTION_TB: u8 = 2;
pub(super) const TITLE: u8 = 6;
pub(super) const ACC_DESCRIPTION: u8 = 7;
pub(super) const ACC_TITLE: u8 = 8;
pub(super) const ACC_TITLE_VALUE: u8 = 9;
pub(super) const ACC_DESCR: u8 = 10;
pub(super) const ACC_DESCR_VALUE: u8 = 11;
pub(super) const ACC_DESCR_MULTILINE_VALUE: u8 = 12;
pub(super) const NEWLINE: u8 = 13;
pub(super) const C4_CONTEXT: u8 = 14;
pub(super) const C4_CONTAINER: u8 = 15;
pub(super) const C4_COMPONENT: u8 = 16;
pub(super) const C4_DYNAMIC: u8 = 17;
pub(super) const C4_DEPLOYMENT: u8 = 18;
pub(super) const PERSON: u8 = 19;
pub(super) const PERSON_EXT: u8 = 20;
pub(super) const SYSTEM: u8 = 21;
pub(super) const SYSTEM_DB: u8 = 22;
pub(super) const SYSTEM_QUEUE: u8 = 23;
pub(super) const SYSTEM_EXT: u8 = 24;
pub(super) const SYSTEM_EXT_DB: u8 = 25;
pub(super) const SYSTEM_EXT_QUEUE: u8 = 26;
pub(super) const CONTAINER: u8 = 27;
pub(super) const CONTAINER_DB: u8 = 28;
pub(super) const CONTAINER_QUEUE: u8 = 29;
pub(super) const CONTAINER_EXT: u8 = 30;
pub(super) const CONTAINER_EXT_DB: u8 = 31;
pub(super) const CONTAINER_EXT_QUEUE: u8 = 32;
pub(super) const COMPONENT: u8 = 33;
pub(super) const COMPONENT_DB: u8 = 34;
pub(super) const COMPONENT_QUEUE: u8 = 35;
pub(super) const COMPONENT_EXT: u8 = 36;
pub(super) const COMPONENT_EXT_DB: u8 = 37;
pub(super) const COMPONENT_EXT_QUEUE: u8 = 38;
pub(super) const ENTERPRISE_BOUNDARY: u8 = 39;
pub(super) const SYSTEM_BOUNDARY: u8 = 40;
pub(super) const BOUNDARY: u8 = 41;
pub(super) const CONTAINER_BOUNDARY: u8 = 42;
pub(super) const NODE: u8 = 43;
pub(super) const NODE_L: u8 = 44;
pub(super) const NODE_R: u8 = 45;
pub(super) const REL: u8 = 46;
pub(super) const BIREL: u8 = 47;
pub(super) const REL_U: u8 = 48;
pub(super) const REL_D: u8 = 49;
pub(super) const REL_L: u8 = 50;
pub(super) const REL_R: u8 = 51;
pub(super) const REL_B: u8 = 52;
pub(super) const REL_INDEX: u8 = 53;
pub(super) const UPDATE_EL_STYLE: u8 = 54;
pub(super) const UPDATE_REL_STYLE: u8 = 55;
pub(super) const UPDATE_LAYOUT_CONFIG: u8 = 56;
pub(super) const EOF_IN_STRUCT: u8 = 57;
pub(super) const ATTRIBUTE_EMPTY: u8 = 58;
pub(super) const STR: u8 = 59;
pub(super) const STR_KEY: u8 = 60;
pub(super) const STR_VALUE: u8 = 61;
pub(super) const LBRACE: u8 = 62;
pub(super) const RBRACE: u8 = 63;
pub(super) const EOF: u8 = 64;

pub(super) const TOKEN_NAMES: [&str; 65] = [
    "$end",
    "INVALID",
    "direction_tb",
    "direction_bt",
    "direction_rl",
    "direction_lr",
    "title",
    "accDescription",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "NEWLINE",
    "C4_CONTEXT",
    "C4_CONTAINER",
    "C4_COMPONENT",
    "C4_DYNAMIC",
    "C4_DEPLOYMENT",
    "PERSON",
    "PERSON_EXT",
    "SYSTEM",
    "SYSTEM_DB",
    "SYSTEM_QUEUE",
    "SYSTEM_EXT",
    "SYSTEM_EXT_DB",
    "SYSTEM_EXT_QUEUE",
    "CONTAINER",
    "CONTAINER_DB",
    "CONTAINER_QUEUE",
    "CONTAINER_EXT",
    "CONTAINER_EXT_DB",
    "CONTAINER_EXT_QUEUE",
    "COMPONENT",
    "COMPONENT_DB",
    "COMPONENT_QUEUE",
    "COMPONENT_EXT",
    "COMPONENT_EXT_DB",
    "COMPONENT_EXT_QUEUE",
    "ENTERPRISE_BOUNDARY",
    "SYSTEM_BOUNDARY",
    "BOUNDARY",
    "CONTAINER_BOUNDARY",
    "NODE",
    "NODE_L",
    "NODE_R",
    "REL",
    "BIREL",
    "REL_U",
    "REL_D",
    "REL_L",
    "REL_R",
    "REL_B",
    "REL_INDEX",
    "UPDATE_EL_STYLE",
    "UPDATE_REL_STYLE",
    "UPDATE_LAYOUT_CONFIG",
    "EOF_IN_STRUCT",
    "ATTRIBUTE_EMPTY",
    "STR",
    "STR_KEY",
    "STR_VALUE",
    "LBRACE",
    "RBRACE",
    "EOF",
];

/// Token text is `src[starts[i]..ends[i]]`; nothing is copied until the parser asks for it.
#[derive(Debug, Default)]
pub(super) struct Tokens {
    pub types: Vec<u8>,
    pub starts: Vec<usize>,
    pub ends: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum S {
    Initial,
    Struct,
    Attribute,
    String,
    StringKv,
    StringKvKey,
    StringKvValue,
    Acc,
    AccDescrMultiline,
}

fn keyword(word: &str) -> Option<u8> {
    Some(match word {
        "C4Context" => C4_CONTEXT,
        "C4Container" => C4_CONTAINER,
        "C4Component" => C4_COMPONENT,
        "C4Dynamic" => C4_DYNAMIC,
        "C4Deployment" => C4_DEPLOYMENT,
        "Person" => PERSON,
        "Person_Ext" => PERSON_EXT,
        "System" => SYSTEM,
        "SystemDb" => SYSTEM_DB,
        "SystemQueue" => SYSTEM_QUEUE,
        "System_Ext" => SYSTEM_EXT,
        "SystemDb_Ext" => SYSTEM_EXT_DB,
        "SystemQueue_Ext" => SYSTEM_EXT_QUEUE,
        "Container" => CONTAINER,
        "ContainerDb" => CONTAINER_DB,
        "ContainerQueue" => CONTAINER_QUEUE,
        "Container_Ext" => CONTAINER_EXT,
        "ContainerDb_Ext" => CONTAINER_EXT_DB,
        "ContainerQueue_Ext" => CONTAINER_EXT_QUEUE,
        "Component" => COMPONENT,
        "ComponentDb" => COMPONENT_DB,
        "ComponentQueue" => COMPONENT_QUEUE,
        "Component_Ext" => COMPONENT_EXT,
        "ComponentDb_Ext" => COMPONENT_EXT_DB,
        "ComponentQueue_Ext" => COMPONENT_EXT_QUEUE,
        "Enterprise_Boundary" => ENTERPRISE_BOUNDARY,
        "System_Boundary" => SYSTEM_BOUNDARY,
        "Boundary" => BOUNDARY,
        "Container_Boundary" => CONTAINER_BOUNDARY,
        "Deployment_Node" => NODE,
        "Node" => NODE,
        "Node_L" => NODE_L,
        "Node_R" => NODE_R,
        "Rel" => REL,
        "BiRel" => BIREL,
        "Rel_Up" => REL_U,
        "Rel_U" => REL_U,
        "Rel_Down" => REL_D,
        "Rel_D" => REL_D,
        "Rel_Left" => REL_L,
        "Rel_L" => REL_L,
        "Rel_Right" => REL_R,
        "Rel_R" => REL_R,
        "Rel_Back" => REL_B,
        "RelIndex" => REL_INDEX,
        "UpdateElementStyle" => UPDATE_EL_STYLE,
        "UpdateRelStyle" => UPDATE_REL_STYLE,
        "UpdateLayoutConfig" => UPDATE_LAYOUT_CONFIG,
        _ => return None,
    })
}

const DIRECTIONS: [&str; 4] = ["TB", "BT", "RL", "LR"];

/// `/.*direction\s+XX[^\n]*/y` at `p`, where `line_end` is the first line terminator at or after
/// `p` (or the end): where the match ends, if it matches. `.*` cannot cross a line terminator
/// and is greedy, so the statement is the last `direction` before `line_end` that the white
/// space and the keyword follow (`\s+` may run over line breaks). The match then runs to the
/// next `\n`.
fn direction_at(src: &str, p: usize, line_end: usize, word: &str) -> Option<usize> {
    let b = src.as_bytes();
    let head = &src[p..line_end];
    for (k, _) in head.rmatch_indices("direction") {
        let mut q = p + k + "direction".len();
        let after = q;
        while let Some(c) = char_at(src, q).filter(|&c| is_space(c)) {
            q += c.len_utf8();
        }
        if q > after && src[q..].starts_with(word) {
            let from = q + word.len();
            return Some(b[from..].iter().position(|&c| c == b'\n').map_or(b.len(), |e| from + e));
        }
    }
    None
}

/// The first line terminator (`[\n\r  ]`) at or after `p`, or the end.
fn line_end_from(src: &str, p: usize) -> usize {
    src[p..].char_indices().find(|&(_, c)| is_line_end(c)).map_or(src.len(), |(k, _)| p + k)
}

/// Records a token and returns where the next one starts.
fn emit(tokens: &mut Tokens, kind: u8, s: usize, e: usize) -> usize {
    tokens.types.push(kind);
    tokens.starts.push(s);
    tokens.ends.push(e);
    e
}

fn skip_ws(src: &str, mut q: usize) -> usize {
    while let Some(c) = char_at(src, q).filter(|&c| is_space(c)) {
        q += c.len_utf8();
    }
    q
}

fn up_to(src: &str, ch: u8, from: usize) -> usize {
    let b = src.as_bytes();
    if from >= b.len() {
        return b.len();
    }
    b[from..].iter().position(|&c| c == ch).map_or(b.len(), |k| from + k)
}

/// `word\s[^#\n;]+` at `p`: where it ends.
fn statement(src: &str, p: usize, word: &str) -> Option<usize> {
    if !src[p..].starts_with(word) {
        return None;
    }
    let b = src.as_bytes();
    let mut q = p + word.len();
    let c = char_at(src, q).filter(|&c| is_space(c))?;
    q += c.len_utf8();
    let from = q;
    while q < b.len() {
        let c = b[q];
        if c == b'#' || c == b'\n' || c == b';' {
            break;
        }
        q += 1;
    }
    (q != from).then_some(q)
}

/// Follows the lexer Jison compiles from c4Diagram.jison: its rule order, its start conditions,
/// and what it returns once the input runs out in each of them.
pub(super) fn tokenize(src: &str) -> Tokens {
    let b = src.as_bytes();
    let n = b.len();
    let mut tk = Tokens::default();
    let mut state = S::Initial;
    let mut acc_value = ACC_TITLE_VALUE;
    let mut p = 0usize;
    let mut done = false;
    // Where the next `direction` is: None until the first search, Some(None) when there is none
    // further on. Pele searches lazily, so that a long input is not scanned for every statement.
    let mut dir_next: Option<Option<usize>> = None;
    let mut line_end: Option<usize> = None;
    let mut dir_failed = 0usize;

    loop {
        // Jison notes the end of input one call before it reports it, so a rule that matches
        // nothing gets one last turn.
        if done {
            emit(&mut tk, END, n, n);
            break;
        }
        if p >= n {
            done = true;
        }
        let c = b.get(p).copied();

        match state {
            S::Initial => {
                if p >= n {
                    p = emit(&mut tk, EOF, n, n);
                    continue;
                }
                if dir_next != Some(None) && p >= dir_failed {
                    if dir_next.is_none_or(|d| d.is_some_and(|d| d < p)) {
                        dir_next = Some(src[p..].find("direction").map(|k| p + k));
                    }
                    if let Some(Some(next)) = dir_next {
                        if line_end.is_none_or(|e| e < p) {
                            line_end = Some(line_end_from(src, p));
                        }
                        let end_of_line = line_end.unwrap_or(n);
                        if next < end_of_line {
                            let mut matched = false;
                            for (i, word) in DIRECTIONS.iter().enumerate() {
                                if let Some(e) = direction_at(src, p, end_of_line, word) {
                                    p = emit(&mut tk, DIRECTION_TB + i as u8, p, e);
                                    matched = true;
                                    break;
                                }
                            }
                            if matched {
                                continue;
                            }
                            // No direction statement starts anywhere in the rest of this line either.
                            dir_failed = end_of_line;
                        }
                    }
                }
                if c == Some(b't') {
                    if let Some(e) = statement(src, p, "title") {
                        p = emit(&mut tk, TITLE, p, e);
                        continue;
                    }
                }
                if c == Some(b'a') {
                    if let Some(e) = statement(src, p, "accDescription") {
                        p = emit(&mut tk, ACC_DESCRIPTION, p, e);
                        continue;
                    }
                    let title = src[p..].starts_with("accTitle");
                    if title || src[p..].starts_with("accDescr") {
                        let q = skip_ws(src, p + 8);
                        let d = b.get(q).copied();
                        if d == Some(b':') {
                            let e = skip_ws(src, q + 1);
                            p = emit(&mut tk, if title { ACC_TITLE } else { ACC_DESCR }, p, e);
                            acc_value = if title { ACC_TITLE_VALUE } else { ACC_DESCR_VALUE };
                            state = S::Acc;
                            continue;
                        }
                        if d == Some(b'{') && !title {
                            p = skip_ws(src, q + 1);
                            state = S::AccDescrMultiline;
                            continue;
                        }
                    }
                }
                if c == Some(b'%') && b.get(p + 1) == Some(&b'%') {
                    // A comment takes the line breaks after it, so no NEWLINE follows.
                    let mut q = up_to(src, b'\n', p + 2);
                    while q < n && (b[q] == b'\n' || b[q] == b'\r') {
                        q += 1;
                    }
                    p = q;
                    continue;
                }
                if char_at(src, p).is_some_and(is_space) {
                    let mut q = p;
                    let mut last = None;
                    while let Some(d) = char_at(src, q).filter(|&d| is_space(d)) {
                        if d == '\n' {
                            last = Some(q);
                        }
                        q += d.len_utf8();
                    }
                    match last {
                        Some(l) => p = emit(&mut tk, NEWLINE, p, l + 1),
                        None => p = q,
                    }
                    continue;
                }
                if c == Some(b'{') {
                    p = emit(&mut tk, LBRACE, p, p + 1);
                    continue;
                }
                if c == Some(b'}') {
                    p = emit(&mut tk, RBRACE, p, p + 1);
                    continue;
                }
                if c.is_some_and(is_word_byte) {
                    let mut q = p + 1;
                    while q < n && is_word_byte(b[q]) {
                        q += 1;
                    }
                    let kind = if q - p <= 19 { keyword(&src[p..q]) } else { None };
                    if let Some(kind) = kind {
                        if kind > C4_DEPLOYMENT {
                            state = S::Struct;
                        }
                        p = emit(&mut tk, kind, p, q);
                        continue;
                    }
                }
                emit(&mut tk, ERROR, p, p);
                break;
            }
            S::Struct => {
                if p >= n {
                    p = emit(&mut tk, EOF_IN_STRUCT, n, n);
                } else if c == Some(b'(') {
                    let mut q = p + 1;
                    while b.get(q) == Some(&b' ') {
                        q += 1;
                    }
                    if b.get(q) == Some(&b',') {
                        p = emit(&mut tk, ATTRIBUTE_EMPTY, p, q + 1);
                    } else {
                        p += 1;
                    }
                    state = S::Attribute;
                } else if c == Some(b')') {
                    p += 1;
                    state = S::Initial;
                } else {
                    emit(&mut tk, ERROR, p, p);
                    break;
                }
            }
            S::Attribute => {
                if p >= n {
                    continue;
                }
                if c == Some(b')') {
                    p += 1;
                    state = S::Initial;
                } else if c == Some(b',') {
                    if b.get(p + 1) == Some(&b',') {
                        p = emit(&mut tk, ATTRIBUTE_EMPTY, p, p + 2);
                    } else {
                        p += 1;
                    }
                } else {
                    let mut q = p;
                    while b.get(q) == Some(&b' ') {
                        q += 1;
                    }
                    let d = b.get(q).copied();
                    if d == Some(b'"') {
                        if b.get(q + 1) == Some(&b'"') {
                            p = emit(&mut tk, ATTRIBUTE_EMPTY, p, q + 2);
                        } else {
                            p = q + 1;
                            state = S::String;
                        }
                    } else if d == Some(b'$') {
                        p = q + 1;
                        state = S::StringKv;
                    } else {
                        // An unquoted value runs to the next comma, through closing parentheses
                        // and line breaks.
                        let e = up_to(src, b',', p);
                        p = emit(&mut tk, STR, p, e);
                    }
                }
            }
            S::String => {
                if c == Some(b'"') {
                    p += 1;
                    state = S::Attribute;
                } else {
                    let e = up_to(src, b'"', p);
                    p = emit(&mut tk, STR, p, e);
                }
            }
            S::StringKv => {
                let e = up_to(src, b'=', p);
                p = emit(&mut tk, STR_KEY, p, e);
                state = S::StringKvKey;
            }
            S::StringKvKey => {
                if p >= n {
                    continue;
                }
                let mut q = p + 1;
                if c == Some(b'=') {
                    while b.get(q) == Some(&b' ') {
                        q += 1;
                    }
                }
                if c != Some(b'=') || b.get(q) != Some(&b'"') {
                    emit(&mut tk, ERROR, p, p);
                    break;
                }
                p = q + 1;
                state = S::StringKvValue;
            }
            S::StringKvValue => {
                if p >= n {
                    continue;
                }
                if c == Some(b'"') {
                    p += 1;
                    state = S::Attribute;
                } else {
                    let e = up_to(src, b'"', p);
                    p = emit(&mut tk, STR_VALUE, p, e);
                }
            }
            S::Acc => {
                let e = up_to(src, b'\n', p);
                p = emit(&mut tk, acc_value, p, e);
                state = S::Initial;
            }
            S::AccDescrMultiline => {
                if c == Some(b'}') {
                    p += 1;
                    state = S::Initial;
                } else {
                    let e = up_to(src, b'}', p);
                    p = emit(&mut tk, ACC_DESCR_MULTILINE_VALUE, p, e);
                }
            }
        }
    }
    tk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<&'static str> {
        tokenize(src).types.iter().map(|&t| TOKEN_NAMES[t as usize]).collect()
    }

    #[test]
    fn statements_and_attributes() {
        assert_eq!(
            kinds("C4Context\nPerson(a, \"A\", $tags=\"x\")\n"),
            ["C4_CONTEXT", "NEWLINE", "PERSON", "STR", "STR", "STR_KEY", "STR_VALUE", "NEWLINE", "EOF", "$end"]
        );
        // An unquoted value runs to the next comma, over the end of the statement.
        let t = tokenize("C4Context\nPerson(a, b)\nPerson(c, d)");
        assert_eq!(t.types[4], STR);
        assert_eq!(&"C4Context\nPerson(a, b)\nPerson(c, d)"[t.starts[4]..t.ends[4]], " b)\nPerson(c");
        assert_eq!(
            kinds("C4Context\nPerson(,,\"\")"),
            ["C4_CONTEXT", "NEWLINE", "PERSON", "ATTRIBUTE_EMPTY", "ATTRIBUTE_EMPTY", "EOF", "$end"]
        );
    }

    #[test]
    fn direction_takes_the_last_match_on_the_line() {
        // `.*` is greedy: the last `direction ... TB` before the line break, whose `\s+` may run
        // over line breaks, decides where the token ends.
        let src = "direction TB direction\n\nTB x\nmore";
        assert_eq!(direction_at(src, 0, line_end_from(src, 0), "TB"), Some(src.find(" x\n").unwrap() + 2));
        assert_eq!(direction_at("a direction  LR b\n", 0, 17, "LR"), Some(17));
        assert_eq!(direction_at("a direction LR b\n", 0, 16, "TB"), None);
        assert_eq!(direction_at("directionTB\n", 0, 11, "TB"), None);
        let t = tokenize("C4Context\ndirection\u{3000}RL;x\n");
        assert_eq!(t.types[2], DIRECTION_TB + 2);
    }

    #[test]
    fn titles_and_accessibility() {
        // A lexical error ends the token list.
        assert_eq!(kinds("C4Context\ntitle\u{a0}T;x"), ["C4_CONTEXT", "NEWLINE", "title", "INVALID"]);
        assert_eq!(
            kinds("C4Context\naccTitle : x\naccDescr {\n y\n}\n"),
            ["C4_CONTEXT", "NEWLINE", "acc_title", "acc_title_value", "NEWLINE", "acc_descr_multiline_value", "NEWLINE", "EOF", "$end"]
        );
        assert_eq!(kinds("C4Context %% c\n\nPerson"), ["C4_CONTEXT", "PERSON", "EOF_IN_STRUCT", "$end"]);
    }
}
