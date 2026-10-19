//! `railroad-abnf-beta`: ABNF as in RFC 5234, with each rule closed by a semicolon. Ported
//! from Pele `src/diagrams/railroad/abnf.ts`.

use super::common::{hidden_tokens, literal, parse_grammar, slice_inner, to_rules, Nodes, ACC_DESCR, ACC_TITLE, DIGITS, LETTERS, TITLE};
use super::{Meta, NodeId, RailroadNode, RailroadRule};
use crate::diagrams::common::tokens::{keyword_at, line_rest, Reader, TokenType};
use crate::error::Error;
use crate::util::js;

// Token kinds: indices into `ABNF_TOKENS`.
const EQUALS: usize = 1;
const SEMICOLON: usize = 2;
const SLASH: usize = 3;
const GROUP: usize = 4;
const OPTION: usize = 6;
const T_TITLE: usize = 8;
const NAME: usize = 11;
const STRING: usize = 12;
const NUMBER: usize = 13;
const REPEAT: usize = 14;
const EXACT: usize = 15;
const COMMENT: usize = 20;

fn keyword(src: &str, p: usize) -> Option<usize> {
    keyword_at(src, p, "railroad-abnf-beta")
}

/// `/[A-Za-z][A-Za-z0-9-]*/y`
fn match_name(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if !b.get(p).is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    let mut i = p + 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
        i += 1;
    }
    Some(i)
}

/// `/;[^\n\r]*/y`
fn match_comment(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with(';').then(|| line_rest(src, p + 1))
}

/// `/"[^"]*"/y`
fn match_string(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with('"') {
        return None;
    }
    src[p + 1..].find('"').map(|k| p + 1 + k + 1)
}

fn hex_run(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < b.len() && b[i].is_ascii_hexdigit() {
        i += 1;
    }
    i
}

fn digit_run(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i
}

/// `/%[xXdDbB][0-9A-Fa-f]+(?:-[0-9A-Fa-f]+|\.[0-9A-Fa-f]+)*/y`. Each run of digits is
/// greedy and must be followed by `-` or `.` to go on, so no backtracking changes the match.
fn match_number(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if b.get(p) != Some(&b'%') || !matches!(b.get(p + 1), Some(b'x' | b'X' | b'd' | b'D' | b'b' | b'B')) {
        return None;
    }
    let mut i = hex_run(b, p + 2);
    if i == p + 2 {
        return None;
    }
    while matches!(b.get(i), Some(b'-' | b'.')) {
        let end = hex_run(b, i + 1);
        if end == i + 1 {
            break;
        }
        i = end;
    }
    Some(i)
}

/// `/[0-9]*\*[0-9]*/y`
fn match_repeat(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let star = digit_run(b, p);
    if b.get(star) != Some(&b'*') {
        return None;
    }
    Some(digit_run(b, star + 1))
}

/// `/[0-9]+/y`
fn match_exact(src: &str, p: usize) -> Option<usize> {
    let end = digit_run(src.as_bytes(), p);
    (end > p).then_some(end)
}

const HIDDEN: [TokenType; 4] = hidden_tokens(["ABNF_WHITESPACE", "ABNF_YAML", "ABNF_DIRECTIVE", "ABNF_SINGLE_LINE_COMMENT"]);
const DIGITS_STAR: &str = "0123456789*";

pub(super) static ABNF_TOKENS: [TokenType; 21] = [
    TokenType::func("railroad-abnf-beta", keyword, Some("r")).with_longer(&[NAME]),
    literal("="),
    // A semicolon closes a rule only at the end of its line; with anything after it, it
    // starts a comment.
    literal(";").with_longer(&[COMMENT]),
    literal("/"),
    literal("("),
    literal(")"),
    literal("["),
    literal("]"),
    TITLE,
    ACC_TITLE,
    ACC_DESCR,
    TokenType::func("ABNF_RULENAME", match_name, Some(LETTERS)),
    TokenType::func("ABNF_STRING", match_string, Some("\"")),
    TokenType::func("ABNF_NUMVAL", match_number, Some("%")),
    TokenType::func("ABNF_REPEAT", match_repeat, Some(DIGITS_STAR)),
    TokenType::func("ABNF_EXACT_REPEAT", match_exact, Some(DIGITS)),
    HIDDEN[0],
    HIDDEN[1],
    HIDDEN[2],
    HIDDEN[3],
    TokenType::func("ABNF_COMMENT", match_comment, Some(";")).hidden(),
];

/// A node of the syntax tree. Children are indices into the tree's node list.
enum Abnf {
    Alternation(Vec<usize>),
    Concatenation(Vec<usize>),
    Element {
        repeat: Option<String>,
        primary: usize,
    },
    /// A string literal or a numeric value: both are drawn as terminals.
    Value(String),
    RuleName(String),
    Group(usize),
    OptionalGroup(usize),
}

/// One open bracket, or the rule itself at the bottom of the stack.
struct Frame {
    alternation: usize,
    concatenation: usize,
    // Whether the bracket is optional, its closing token, and the repeat written before it.
    optional: bool,
    close: usize,
    repeat: Option<String>,
}

/// The closing token of the rule's own frame: none.
const NO_CLOSE: usize = usize::MAX - 1;

fn push(ast: &mut Vec<Abnf>, node: Abnf) -> usize {
    ast.push(node);
    ast.len() - 1
}

fn add(ast: &mut [Abnf], list: usize, item: usize) {
    if let Abnf::Alternation(items) | Abnf::Concatenation(items) = &mut ast[list] {
        items.push(item);
    }
}

fn frame(ast: &mut Vec<Abnf>, optional: bool, alternation: usize, close: usize, repeat: Option<String>) -> Frame {
    let concatenation = push(ast, Abnf::Concatenation(Vec::new()));
    add(ast, alternation, concatenation);
    Frame { alternation, concatenation, optional, close, repeat }
}

fn alternation(r: &mut Reader, ast: &mut Vec<Abnf>) -> Result<usize, Error> {
    let mut stack: Vec<Frame> = Vec::new();
    let root = push(ast, Abnf::Alternation(Vec::new()));
    let mut top = frame(ast, false, root, NO_CLOSE, None);
    loop {
        let mut repeat = if r.kind() == REPEAT || r.kind() == EXACT { Some(r.take().to_string()) } else { None };
        let kind = r.kind();
        let mut primary = match kind {
            STRING => {
                let value = slice_inner(r.take()).to_string();
                push(ast, Abnf::Value(value))
            }
            NUMBER => {
                let value = r.take().to_string();
                push(ast, Abnf::Value(value))
            }
            NAME => {
                let name = r.take().to_string();
                push(ast, Abnf::RuleName(name))
            }
            GROUP | OPTION => {
                r.i += 1;
                let element = push(ast, Abnf::Alternation(Vec::new()));
                let inner = frame(ast, kind == OPTION, element, kind + 1, repeat);
                stack.push(std::mem::replace(&mut top, inner));
                continue;
            }
            _ => return Err(r.fail("a string, a number, a name, or an opening bracket")),
        };
        loop {
            let element = push(ast, Abnf::Element { repeat: repeat.take(), primary });
            add(ast, top.concatenation, element);
            let k = r.kind();
            if k == top.close {
                r.i += 1;
                let Some(parent) = stack.pop() else {
                    return Err(r.fail("a string, a number, a name, or an opening bracket"));
                };
                let closed = std::mem::replace(&mut top, parent);
                primary =
                    push(ast, if closed.optional { Abnf::OptionalGroup(closed.alternation) } else { Abnf::Group(closed.alternation) });
                repeat = closed.repeat;
                continue;
            }
            if k == SLASH {
                r.i += 1;
                top.concatenation = push(ast, Abnf::Concatenation(Vec::new()));
                add(ast, top.alternation, top.concatenation);
            } else if !matches!(k, STRING | NUMBER | NAME | GROUP | OPTION | REPEAT | EXACT) {
                if stack.is_empty() {
                    return Ok(top.alternation);
                }
                return Err(r.fail(&format!("token of type '{}'", ABNF_TOKENS[top.close].name)));
            }
            break;
        }
    }
}

/// ABNF strings have no escapes.
fn unquote(text: &str) -> String {
    slice_inner(text).to_string()
}

pub(super) fn parse(src: &str, nodes: &mut Vec<RailroadNode>) -> Result<(Meta, Vec<RailroadRule>), Error> {
    let mut ast: Vec<Abnf> = Vec::new();
    let (meta, rules) = parse_grammar(src, &ABNF_TOKENS, T_TITLE, unquote, |r| {
        let name = r.expect(NAME)?.to_string();
        r.expect(EQUALS)?;
        let definition = alternation(r, &mut ast)?;
        r.expect(SEMICOLON)?;
        Ok((name, definition))
    })?;
    Ok((meta, to_rules(rules, ast.as_slice(), nodes, children, node)))
}

fn children(ast: &[Abnf], n: usize) -> Vec<usize> {
    match &ast[n] {
        Abnf::Alternation(items) | Abnf::Concatenation(items) => items.clone(),
        Abnf::Element { primary, .. } => vec![*primary],
        Abnf::Group(element) | Abnf::OptionalGroup(element) => vec![*element],
        Abnf::Value(_) | Abnf::RuleName(_) => Vec::new(),
    }
}

fn node(ast: &[Abnf], n: usize, kids: Vec<NodeId>, model: &mut Nodes) -> NodeId {
    match &ast[n] {
        Abnf::Alternation(_) => model.choice(kids),
        Abnf::Concatenation(_) => model.sequence(kids),
        Abnf::Element { repeat, .. } => {
            let Some(repeat) = repeat.as_deref().filter(|r| !r.is_empty()) else {
                return kids[0];
            };
            // `n*m`, `n*`, `*m`, `*` or `n`.
            let star = repeat.find('*');
            let exact = star.is_none();
            let min = if exact || star.is_some_and(|s| s > 0) { js::parse_int(repeat, 10) } else { 0.0 };
            let max = match star {
                None => min,
                Some(s) if s < repeat.len() - 1 => js::parse_int(&repeat[s + 1..], 10),
                Some(_) => f64::INFINITY,
            };
            if min == 0.0 && max == 1.0 {
                model.optional(kids[0])
            } else {
                model.repetition(kids[0], min, max)
            }
        }
        Abnf::Value(value) => model.terminal(value.clone()),
        Abnf::RuleName(name) => model.nonterminal(name.clone()),
        Abnf::Group(_) => kids[0],
        Abnf::OptionalGroup(_) => model.optional(kids[0]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matchers() {
        assert_eq!(match_number("%x41-5A.b z", 0), Some(9));
        assert_eq!(match_number("%x41-", 0), Some(4));
        assert_eq!(match_number("%x", 0), None);
        assert_eq!(match_number("%%x", 0), None);
        assert_eq!(match_repeat("12*34x", 0), Some(5));
        assert_eq!(match_repeat("*", 0), Some(1));
        assert_eq!(match_repeat("12", 0), None);
        assert_eq!(match_exact("12a", 0), Some(2));
        assert_eq!(match_comment("; a\nb", 0), Some(3));
        assert_eq!(match_string("\"a\" b", 0), Some(3));
        assert_eq!(match_string("\"a", 0), None);
        assert_eq!(match_name("a-1_", 0), Some(3));
    }
}
