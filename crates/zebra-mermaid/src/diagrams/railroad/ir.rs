//! `railroad-beta`: Mermaid's own notation, one constructor call per node. Ported from Pele
//! `src/diagrams/railroad/ir.ts`.

use super::common::{
    block_comment_opener, hidden_tokens, identifier, literal, match_block_comment, parse_grammar, quoted, to_rules, unescape, Nodes,
    ACC_DESCR, ACC_TITLE, TITLE,
};
use super::{Meta, NodeId, RailroadNode, RailroadRule};
use crate::diagrams::common::tokens::{keyword_at, Reader, TokenType};
use crate::error::Error;

// Token kinds: indices into `IR_TOKENS`.
const NONTERMINAL: usize = 1;
const ZERO_OR_MORE: usize = 2;
const ONE_OR_MORE: usize = 3;
const SEQUENCE: usize = 4;
const OPTIONAL: usize = 5;
const TERMINAL: usize = 6;
const SPECIAL: usize = 7;
const CHOICE: usize = 8;
const EQUALS: usize = 9;
const SEMICOLON: usize = 10;
const OPEN: usize = 11;
const COMMA: usize = 12;
const CLOSE: usize = 13;
const T_TITLE: usize = 14;
const ID: usize = 17;
const STRING: usize = 18;

fn keyword(src: &str, p: usize) -> Option<usize> {
    keyword_at(src, p, "railroad-beta")
}

/// A word of the notation. A name that starts with it and goes on is a name.
const fn word(name: &'static str) -> TokenType {
    literal(name).with_longer(&[ID])
}

const HIDDEN: [TokenType; 4] = hidden_tokens(["RR_WHITESPACE", "RR_YAML", "RR_DIRECTIVE", "RR_SINGLE_LINE_COMMENT"]);

pub(super) static IR_TOKENS: [TokenType; 24] = [
    TokenType::func("railroad-beta", keyword, Some("r")).with_longer(&[ID]),
    word("nonterminal"),
    word("zeroOrMore"),
    word("oneOrMore"),
    word("sequence"),
    word("optional"),
    word("terminal"),
    word("special"),
    word("choice"),
    literal("="),
    literal(";"),
    literal("("),
    literal(","),
    literal(")"),
    TITLE,
    ACC_TITLE,
    ACC_DESCR,
    identifier("RR_ID"),
    quoted("RR_STRING"),
    HIDDEN[0],
    HIDDEN[1],
    HIDDEN[2],
    HIDDEN[3],
    TokenType::func("RR_BLOCK_COMMENT", match_block_comment, Some("/")).hidden().with_opener(block_comment_opener),
];

/// A node of the syntax tree. Children are indices into the tree's node list.
enum Ir {
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
    Optional(usize),
    OneOrMore(usize),
    ZeroOrMore(usize),
    Terminal(String),
    NonTerminal(String),
    Special(String),
}

/// A constructor whose arguments are still being read.
enum Open {
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
    Optional,
    OneOrMore,
    ZeroOrMore,
}

/// `kind ( STRING )`
fn argument(r: &mut Reader) -> Result<String, Error> {
    r.i += 1;
    r.expect(OPEN)?;
    let value = unescape(r.expect(STRING)?);
    r.expect(CLOSE)?;
    Ok(value)
}

fn push(ast: &mut Vec<Ir>, node: Ir) -> usize {
    ast.push(node);
    ast.len() - 1
}

/// Reads one expression. Constructors that take expressions are kept on a stack until their
/// closing parenthesis, so nesting does not recurse.
fn expression(r: &mut Reader, ast: &mut Vec<Ir>) -> Result<usize, Error> {
    let mut open: Vec<Open> = Vec::new();
    loop {
        let leaf = match r.kind() {
            TERMINAL => Ir::Terminal(argument(r)?),
            NONTERMINAL => Ir::NonTerminal(argument(r)?),
            SPECIAL => Ir::Special(argument(r)?),
            kind @ (SEQUENCE | CHOICE | OPTIONAL | ONE_OR_MORE | ZERO_OR_MORE) => {
                open.push(match kind {
                    SEQUENCE => Open::Sequence(Vec::new()),
                    CHOICE => Open::Choice(Vec::new()),
                    OPTIONAL => Open::Optional,
                    ONE_OR_MORE => Open::OneOrMore,
                    _ => Open::ZeroOrMore,
                });
                r.i += 1;
                r.expect(OPEN)?;
                continue;
            }
            _ => return Err(r.fail("an expression")),
        };
        let mut value = push(ast, leaf);
        loop {
            let Some(parent) = open.last_mut() else {
                return Ok(value);
            };
            // A wrapper takes its one element; a list takes the next one and, after a comma,
            // another expression.
            let node = match parent {
                Open::Sequence(items) | Open::Choice(items) => {
                    items.push(value);
                    if r.accept(COMMA) {
                        break;
                    }
                    None
                }
                Open::Optional => Some(Ir::Optional(value)),
                Open::OneOrMore => Some(Ir::OneOrMore(value)),
                Open::ZeroOrMore => Some(Ir::ZeroOrMore(value)),
            };
            r.expect(CLOSE)?;
            let node = match (open.pop(), node) {
                (_, Some(node)) => node,
                (Some(Open::Sequence(items)), None) => Ir::Sequence(items),
                (Some(Open::Choice(items)), None) => Ir::Choice(items),
                // A wrapper always made its node above.
                _ => return Err(r.fail("an expression")),
            };
            value = push(ast, node);
        }
    }
}

pub(super) fn parse(src: &str, nodes: &mut Vec<RailroadNode>) -> Result<(Meta, Vec<RailroadRule>), Error> {
    let mut ast: Vec<Ir> = Vec::new();
    let (meta, rules) = parse_grammar(src, &IR_TOKENS, T_TITLE, unescape, |r| {
        let name = r.expect(ID)?.to_string();
        r.expect(EQUALS)?;
        let definition = expression(r, &mut ast)?;
        r.expect(SEMICOLON)?;
        Ok((name, definition))
    })?;
    Ok((meta, to_rules(rules, ast.as_slice(), nodes, children, node)))
}

fn children(ast: &[Ir], n: usize) -> Vec<usize> {
    match &ast[n] {
        Ir::Sequence(items) | Ir::Choice(items) => items.clone(),
        Ir::Optional(e) | Ir::OneOrMore(e) | Ir::ZeroOrMore(e) => vec![*e],
        Ir::Terminal(_) | Ir::NonTerminal(_) | Ir::Special(_) => Vec::new(),
    }
}

fn node(ast: &[Ir], n: usize, kids: Vec<NodeId>, model: &mut Nodes) -> NodeId {
    match &ast[n] {
        Ir::Terminal(value) => model.terminal(value.clone()),
        Ir::NonTerminal(name) => model.nonterminal(name.clone()),
        Ir::Special(text) => model.special(text.clone()),
        Ir::Sequence(_) => model.sequence(kids),
        Ir::Choice(_) => model.choice(kids),
        Ir::Optional(_) => model.optional(kids[0]),
        Ir::OneOrMore(_) => model.repetition(kids[0], 1.0, f64::INFINITY),
        Ir::ZeroOrMore(_) => model.repetition(kids[0], 0.0, f64::INFINITY),
    }
}
