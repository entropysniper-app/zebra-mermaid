//! `railroad-peg-beta`: parsing expression grammars. Ported from Pele
//! `src/diagrams/railroad/peg.ts`.

use super::common::{hidden_tokens, identifier, literal, parse_grammar, quoted, to_rules, unescape, Nodes, ACC_DESCR, ACC_TITLE, TITLE};
use super::{Meta, NodeId, RailroadNode, RailroadRule};
use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use crate::diagrams::common::tokens::{keyword_at, line_rest, TokenType};
use crate::error::Error;
use std::sync::LazyLock;

// Token kinds: indices into `PEG_TOKENS`.
const ARROW: usize = 1;
const SEMICOLON: usize = 2;
const SLASH: usize = 3;
const AND: usize = 4;
const NOT: usize = 5;
const QUESTION: usize = 6;
const STAR: usize = 7;
const PLUS: usize = 8;
const GROUP: usize = 9;
const GROUP_END: usize = 10;
const DOT: usize = 11;
const T_TITLE: usize = 12;
const ID: usize = 15;
const STRING: usize = 16;

fn keyword(src: &str, p: usize) -> Option<usize> {
    keyword_at(src, p, "railroad-peg-beta")
}

/// `/#[^\n\r]*/y`
fn match_line_comment(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with('#').then(|| line_rest(src, p + 1))
}

const HIDDEN: [TokenType; 4] = hidden_tokens(["PEG_WHITESPACE", "PEG_YAML", "PEG_DIRECTIVE", "PEG_SINGLE_LINE_COMMENT"]);

pub(super) static PEG_TOKENS: [TokenType; 22] = [
    TokenType::func("railroad-peg-beta", keyword, Some("r")).with_longer(&[ID]),
    literal("<-"),
    literal(";"),
    literal("/"),
    literal("&"),
    literal("!"),
    literal("?"),
    literal("*"),
    literal("+"),
    literal("("),
    literal(")"),
    literal("."),
    TITLE,
    ACC_TITLE,
    ACC_DESCR,
    identifier("PEG_ID"),
    quoted("PEG_STRING"),
    HIDDEN[0],
    HIDDEN[1],
    HIDDEN[2],
    HIDDEN[3],
    TokenType::func("PEG_LINE_COMMENT", match_line_comment, Some("#")).hidden(),
];

/// A node of the syntax tree. Children are indices into the tree's node list. Operators are
/// the token texts: `&` or `!` before an expression, `?`, `*` or `+` after one.
enum Peg {
    Choice(Vec<usize>),
    Sequence(Vec<usize>),
    Prefix { operator: Option<String>, suffix: usize },
    Suffix { primary: usize, operator: Option<String> },
    Literal(String),
    Identifier(String),
    Any(String),
    Group(usize),
}

/// One open group, or the rule itself at the bottom of the stack.
struct Frame {
    choice: usize,
    sequence: usize,
    // The predicate written before the group.
    operator: Option<String>,
}

fn push(ast: &mut Vec<Peg>, node: Peg) -> usize {
    ast.push(node);
    ast.len() - 1
}

fn add(ast: &mut [Peg], list: usize, item: usize) {
    if let Peg::Choice(items) | Peg::Sequence(items) = &mut ast[list] {
        items.push(item);
    }
}

fn frame(ast: &mut Vec<Peg>, choice: usize, operator: Option<String>) -> Frame {
    let sequence = push(ast, Peg::Sequence(Vec::new()));
    add(ast, choice, sequence);
    Frame { choice, sequence, operator }
}

fn definition(r: &mut Reader, ast: &mut Vec<Peg>) -> Result<usize, Error> {
    let mut stack: Vec<Frame> = Vec::new();
    let root = push(ast, Peg::Choice(Vec::new()));
    let mut top = frame(ast, root, None);
    loop {
        let mut operator = if r.kind() == AND || r.kind() == NOT { Some(r.take().to_string()) } else { None };
        let mut primary = match r.kind() {
            STRING => {
                let value = unescape(r.take());
                push(ast, Peg::Literal(value))
            }
            ID => {
                let name = r.take().to_string();
                push(ast, Peg::Identifier(name))
            }
            DOT => {
                let dot = r.take().to_string();
                push(ast, Peg::Any(dot))
            }
            GROUP => {
                r.i += 1;
                let element = push(ast, Peg::Choice(Vec::new()));
                let inner = frame(ast, element, operator);
                stack.push(std::mem::replace(&mut top, inner));
                continue;
            }
            _ => return Err(r.fail("a literal, a name, a dot, or an opening parenthesis")),
        };
        loop {
            let suffix_operator = if matches!(r.kind(), QUESTION | STAR | PLUS) { Some(r.take().to_string()) } else { None };
            let suffix = push(ast, Peg::Suffix { primary, operator: suffix_operator });
            let prefix = push(ast, Peg::Prefix { operator: operator.take(), suffix });
            add(ast, top.sequence, prefix);
            let k = r.kind();
            if k == GROUP_END {
                if let Some(parent) = stack.pop() {
                    r.i += 1;
                    let closed = std::mem::replace(&mut top, parent);
                    primary = push(ast, Peg::Group(closed.choice));
                    operator = closed.operator;
                    continue;
                }
            }
            if k == SLASH {
                r.i += 1;
                top.sequence = push(ast, Peg::Sequence(Vec::new()));
                add(ast, top.choice, top.sequence);
            } else if !matches!(k, AND | NOT | STRING | ID | DOT | GROUP) {
                if stack.is_empty() {
                    return Ok(top.choice);
                }
                return Err(r.fail("token of type ')'"));
            }
            break;
        }
    }
}

pub(super) fn parse(src: &str, nodes: &mut Vec<RailroadNode>) -> Result<(Meta, Vec<RailroadRule>), Error> {
    let mut ast: Vec<Peg> = Vec::new();
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&PEG_TOKENS));
    let (meta, rules) = parse_grammar(src, &GRAMMAR, T_TITLE, unescape, |r| {
        let name = r.expect(ID)?.to_string();
        r.expect(ARROW)?;
        let definition = definition(r, &mut ast)?;
        r.expect(SEMICOLON)?;
        Ok((name, definition))
    })?;
    Ok((meta, to_rules(rules, ast.as_slice(), nodes, children, node)))
}

fn children(ast: &[Peg], n: usize) -> Vec<usize> {
    match &ast[n] {
        Peg::Choice(items) | Peg::Sequence(items) => items.clone(),
        Peg::Prefix { suffix, .. } => vec![*suffix],
        Peg::Suffix { primary, .. } => vec![*primary],
        Peg::Group(element) => vec![*element],
        Peg::Literal(_) | Peg::Identifier(_) | Peg::Any(_) => Vec::new(),
    }
}

/// What a predicate shows of the expression it looks ahead for.
fn label(node: &RailroadNode) -> String {
    match node {
        RailroadNode::Terminal { value } => format!("\"{}\"", value),
        RailroadNode::Nonterminal { name } => name.clone(),
        RailroadNode::Special { text } => text.clone(),
        _ => "(...)".to_string(),
    }
}

fn node(ast: &[Peg], n: usize, kids: Vec<NodeId>, model: &mut Nodes) -> NodeId {
    match &ast[n] {
        Peg::Choice(_) => model.choice(kids),
        Peg::Sequence(_) => model.sequence(kids),
        Peg::Prefix { operator, .. } => match operator.as_deref().filter(|o| !o.is_empty()) {
            Some(operator) => {
                let text = format!("{}{}", operator, label(model.get(kids[0])));
                model.special(text)
            }
            None => kids[0],
        },
        Peg::Suffix { operator, .. } => match operator.as_deref().filter(|o| !o.is_empty()) {
            None => kids[0],
            Some("?") => model.optional(kids[0]),
            Some(op) => model.repetition(kids[0], if op == "+" { 1.0 } else { 0.0 }, f64::INFINITY),
        },
        Peg::Literal(value) => model.terminal(value.clone()),
        Peg::Identifier(name) => model.nonterminal(name.clone()),
        Peg::Any(dot) => model.special(dot.clone()),
        Peg::Group(_) => kids[0],
    }
}
#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("railroad-peg", &super::PEG_TOKENS, crate::DiagramType::Railroad);
    }
}
