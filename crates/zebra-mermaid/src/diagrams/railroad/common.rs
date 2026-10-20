//! What the four railroad grammars share: the title and accessibility tokens (which, unlike the
//! ones in Mermaid's common grammar, do not take leading blanks), the hidden tokens, the
//! statements before the first rule, string values, and the walk from a syntax tree to the
//! rule model. Ported from Pele `src/diagrams/railroad/common.ts`.

use super::{Meta, NodeId, RailroadNode, RailroadRule};
use crate::detect::DiagramType;
use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use crate::diagrams::common::tokens::{self, acc_descr_value, acc_title_value, title_value, Pattern, Scanner, TokenType, EOF};
use crate::error::Error;
use crate::util::chars::is_space;

/// Pele's `isSpace`: `c === 32 || (c >= 9 && c <= 13) || (c > 127 && /\s/.test(...))` on a
/// UTF-16 code unit. For ASCII that is exactly `\s`; past ASCII it is `\s`; a surrogate is
/// never white space. So on whole characters it is `\s`, which `is_space` implements.
pub(super) fn is_space_char(c: char) -> bool {
    is_space(c)
}

pub(super) const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const LETTERS_UNDERSCORE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
pub(super) const DIGITS: &str = "0123456789";

/// The braced form scans to the next closing brace. Once none is found, none exists further
/// on either, so later braced descriptions fail without scanning again.
///
/// It answers as `/accDescr(?:[\t ]*:([^\n\r]*?(?=%%)|[^\n\r]*)|\s*{([^}]*)})/y` does.
fn acc_descr_scanner() -> Scanner {
    let mut unclosed = usize::MAX;
    Box::new(move |src: &str, at: usize| {
        if !src[at..].starts_with("accDescr") {
            return None;
        }
        let b = src.as_bytes();
        let n = b.len();
        let mut i = at + 8;
        while i < n && (b[i] == b' ' || b[i] == b'\t') {
            i += 1;
        }
        if b.get(i) == Some(&b':') {
            i += 1;
            while i < n {
                let c = b[i];
                if c == b'\n' || c == b'\r' || (c == b'%' && b.get(i + 1) == Some(&b'%')) {
                    break;
                }
                i += 1;
            }
            return Some(i);
        }
        let i = tokens::spaces(src, i);
        if b.get(i) != Some(&b'{') || i >= unclosed {
            return None;
        }
        match src[i + 1..].find('}') {
            Some(k) => Some(i + 1 + k + 1),
            None => {
                unclosed = i;
                None
            }
        }
    })
}

/// `/title(?:[\t ][^\n\r]*?(?=%%)|[\t ][^\n\r]*|)/y`. The shared matcher also takes leading
/// blanks; this one is only tried at a `t`, where they cannot occur.
pub(super) const TITLE: TokenType = TokenType::func("TITLE", tokens::match_title, Some("t"));
/// `/accTitle[\t ]*:(?:[^\n\r]*?(?=%%)|[^\n\r]*)/y`, tried only at an `a`.
pub(super) const ACC_TITLE: TokenType = TokenType::func("ACC_TITLE", tokens::match_acc_title, Some("a"));
pub(super) const ACC_DESCR: TokenType = TokenType {
    name: "ACC_DESCR",
    pattern: Pattern::Scanner(acc_descr_scanner),
    hidden: false,
    opener: None,
    first: Some("a"),
    longer: &[],
};

/// A literal token whose name is its text.
pub(super) const fn literal(text: &'static str) -> TokenType {
    TokenType::literal(text, text)
}

/// `/[A-Z_a-z][\w-]*/y`
fn match_identifier(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if !b.get(p).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_') {
        return None;
    }
    let mut i = p + 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-') {
        i += 1;
    }
    Some(i)
}

pub(super) const fn identifier(name: &'static str) -> TokenType {
    TokenType::func(name, match_identifier, Some(LETTERS_UNDERSCORE))
}

/// `/"([^"\\]|\\.)*"|'([^'\\]|\\.)*'/y`
pub(super) const fn quoted(name: &'static str) -> TokenType {
    TokenType::func(name, tokens::match_string, Some("\"'"))
}

/// `/[\t \r\n]+/y`
fn match_whitespace(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut i = p;
    while i < b.len() && matches!(b[i], b'\t' | b' ' | b'\r' | b'\n') {
        i += 1;
    }
    (i > p).then_some(i)
}

/// Whitespace, front matter, directive, and `%%` comment, in the order every railroad grammar
/// lists them, with the names `<prefix>_WHITESPACE`, `<prefix>_YAML`, `<prefix>_DIRECTIVE` and
/// `<prefix>_SINGLE_LINE_COMMENT`.
pub(super) const fn hidden_tokens(names: [&'static str; 4]) -> [TokenType; 4] {
    [
        TokenType::func(names[0], match_whitespace, Some("\t \r\n")).hidden(),
        TokenType { name: names[1], first: Some("-"), ..tokens::YAML },
        TokenType { name: names[2], first: Some("\t %"), ..tokens::DIRECTIVE },
        TokenType { name: names[3], first: Some("\t %"), ..tokens::SINGLE_LINE_COMMENT },
    ]
}

/// `/\/\*[\s\S]*?\*\//y`, the block comment of Mermaid's notation and of EBNF.
pub(super) fn match_block_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("/*") {
        return None;
    }
    src[p + 2..].find("*/").map(|k| p + 2 + k + 2)
}

/// `/\/\*/y`
pub(super) fn block_comment_opener(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with("/*").then_some(p + 2)
}

/// Mermaid's railroad value converters know three escapes; any other escaped character
/// stands for itself.
///
/// The text starts and ends with a quote, so the code units between them are whole
/// characters, and a backslash before a character outside the Basic Multilingual Plane takes
/// its first surrogate, which the next step completes: reading characters gives the same.
pub(super) fn unescape(text: &str) -> String {
    let inner = slice_inner(text);
    if !text.contains('\\') {
        return inner.to_string();
    }
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        // `c === '\\' && i + 1 < end`: a backslash with something after it before the end.
        if c == '\\' && chars.peek().is_some() {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `text.slice(1, -1)` for a text whose first and last characters are ASCII quotes, or
/// shorter than two characters.
pub(super) fn slice_inner(text: &str) -> &str {
    let len = crate::util::js::len16(text);
    if len < 2 {
        return "";
    }
    crate::util::js::slice16(text, 1, len - 1)
}

/// Parses the keyword, then any titles and accessibility statements, then rules to the end
/// of the text. Every table starts with its keyword and has TITLE, ACC_TITLE and ACC_DESCR in
/// a row from `meta`.
pub(super) fn parse_grammar<'a, R>(
    src: &'a str,
    grammar: &Grammar,
    meta: usize,
    unquote: fn(&str) -> String,
    mut rule: impl FnMut(&mut Reader<'a, '_>) -> Result<R, Error>,
) -> Result<(Meta, Vec<R>), Error> {
    let mut r = grammar.reader(src, DiagramType::Railroad)?;
    let mut ast = Meta::default();
    let mut rules = Vec::new();
    r.expect(0)?;
    loop {
        let kind = r.kind();
        if kind == meta {
            let title = title_value(r.take());
            // `title[0]` is a quote and the title ends with it: a quoted title is unquoted.
            let quote = title.chars().next();
            ast.title = Some(match quote {
                Some(q @ ('"' | '\'')) if title.ends_with(q) => unquote(&title),
                _ => title,
            });
        } else if kind == meta + 1 {
            ast.acc_title = Some(acc_title_value(r.take()));
        } else if kind == meta + 2 {
            ast.acc_descr = Some(acc_descr_value(r.take()));
        } else {
            break;
        }
    }
    while r.kind() != EOF {
        rules.push(rule(&mut r)?);
    }
    Ok((ast, rules))
}

/// Builds rule model nodes into the model's node list.
pub(super) struct Nodes<'n>(pub &'n mut Vec<RailroadNode>);

impl Nodes<'_> {
    fn push(&mut self, node: RailroadNode) -> NodeId {
        self.0.push(node);
        self.0.len() - 1
    }

    pub fn terminal(&mut self, value: String) -> NodeId {
        self.push(RailroadNode::Terminal { value })
    }

    pub fn nonterminal(&mut self, name: String) -> NodeId {
        self.push(RailroadNode::Nonterminal { name })
    }

    pub fn special(&mut self, text: String) -> NodeId {
        self.push(RailroadNode::Special { text })
    }

    pub fn optional(&mut self, element: NodeId) -> NodeId {
        self.push(RailroadNode::Optional { element })
    }

    pub fn repetition(&mut self, element: NodeId, min: f64, max: f64) -> NodeId {
        self.push(RailroadNode::Repetition { element, min, max })
    }

    /// A sequence of exactly the elements given, even one.
    pub fn sequence_of(&mut self, elements: Vec<NodeId>) -> NodeId {
        self.push(RailroadNode::Sequence { elements })
    }

    /// A sequence or a choice of one is that one, in every notation.
    pub fn sequence(&mut self, elements: Vec<NodeId>) -> NodeId {
        if elements.len() == 1 {
            return elements[0];
        }
        self.push(RailroadNode::Sequence { elements })
    }

    pub fn choice(&mut self, alternatives: Vec<NodeId>) -> NodeId {
        if alternatives.len() == 1 {
            return alternatives[0];
        }
        self.push(RailroadNode::Choice { alternatives })
    }

    /// A node built so far.
    pub fn get(&self, id: NodeId) -> &RailroadNode {
        &self.0[id]
    }
}

/// Rebuilds a tree bottom-up without recursion, so that nesting depth is bounded by memory,
/// not by the stack. `children` gives a node's parts in order; `build` makes a node's result
/// from its own and its parts' results.
pub(super) fn fold<R>(root: usize, children: impl Fn(usize) -> Vec<usize>, mut build: impl FnMut(usize, Vec<R>) -> R) -> R {
    let mut nodes: Vec<usize> = vec![root];
    let mut lists: Vec<Vec<usize>> = vec![children(root)];
    let mut done: Vec<Vec<R>> = vec![Vec::new()];
    loop {
        let top = nodes.len() - 1;
        let count = done[top].len();
        if count < lists[top].len() {
            let child = lists[top][count];
            nodes.push(child);
            lists.push(children(child));
            done.push(Vec::new());
        } else {
            let results = done.pop().unwrap_or_default();
            let node = nodes.pop().unwrap_or(root);
            lists.pop();
            let value = build(node, results);
            match done.last_mut() {
                None => return value,
                Some(parent) => parent.push(value),
            }
        }
    }
}

/// Turns each rule's syntax tree into the rule model, given how a syntax node's parts are
/// found and how a model node is built from them.
pub(super) fn to_rules<A: ?Sized>(
    rules: Vec<(String, usize)>,
    ast: &A,
    nodes: &mut Vec<RailroadNode>,
    children: impl Fn(&A, usize) -> Vec<usize>,
    build: impl Fn(&A, usize, Vec<NodeId>, &mut Nodes) -> NodeId,
) -> Vec<RailroadRule> {
    let mut out = Vec::with_capacity(rules.len());
    let mut model = Nodes(nodes);
    for (name, definition) in rules {
        let definition = fold(definition, |n| children(ast, n), |n, kids| build(ast, n, kids, &mut model));
        out.push(RailroadRule { name, definition });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acc_descr_scanner_matches_the_expression() {
        let mut s = acc_descr_scanner();
        assert_eq!(s("accDescr: a b %% c", 0), Some(14));
        assert_eq!(s("accDescr \t: x\ny", 0), Some(13));
        assert_eq!(s("accDescr \n { a\n b } c", 0), Some(19));
        assert_eq!(s("accDescrX", 0), None);
        assert_eq!(s("accDescr {", 0), None);
        // Once no closing brace is found, a later opening brace fails without a scan.
        assert_eq!(s("accDescr { accDescr {", 11), None);
        let mut fresh = acc_descr_scanner();
        assert_eq!(fresh("x accDescr {}", 2), Some(13));
    }

    #[test]
    fn identifiers_and_comments() {
        assert_eq!(match_identifier("ab-c_1 x", 0), Some(6));
        assert_eq!(match_identifier("_a", 0), Some(2));
        assert_eq!(match_identifier("1a", 0), None);
        assert_eq!(match_identifier("a中", 0), Some(1));
        assert_eq!(match_block_comment("/* a */b", 0), Some(7));
        assert_eq!(match_block_comment("/*/", 0), None);
        assert_eq!(match_whitespace(" \t\r\nx", 0), Some(4));
        assert_eq!(match_whitespace("\u{a0}", 0), None);
    }

    #[test]
    fn unescapes_three_escapes() {
        assert_eq!(unescape("\"a\\nb\\tc\\rd\\qe\""), "a\nb\tc\rdqe");
        assert_eq!(unescape("'plain'"), "plain");
        assert_eq!(unescape("\"x\\\""), "x\\");
        assert_eq!(unescape("\"\\😀\""), "😀");
        assert_eq!(unescape("\""), "");
        assert_eq!(slice_inner("'中'"), "中");
    }

    #[test]
    fn fold_is_post_order() {
        // 0 has children 1 and 2; 1 has child 3.
        let kids = |n: usize| match n {
            0 => vec![1, 2],
            1 => vec![3],
            _ => vec![],
        };
        let mut order = Vec::new();
        let total = fold(0, kids, |n, results: Vec<usize>| {
            order.push(n);
            n + results.iter().sum::<usize>()
        });
        assert_eq!(order, vec![3, 1, 2, 0]);
        assert_eq!(total, 6);
    }
}
