//! `railroad-ebnf-beta`: EBNF in the W3C and ISO 14977 styles. Ported from Pele
//! `src/diagrams/railroad/ebnf.ts`.

use super::common::{
    block_comment_opener, hidden_tokens, identifier, is_space_char, literal, match_block_comment, parse_grammar, quoted, slice_inner,
    to_rules, unescape, Nodes, ACC_DESCR, ACC_TITLE, TITLE,
};
use super::{Meta, NodeId, RailroadNode, RailroadRule};
use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use crate::diagrams::common::tokens::{keyword_at, Pattern, TokenType};
use crate::error::Error;
use crate::util::js;
use std::sync::LazyLock;

// Token kinds: indices into `EBNF_TOKENS`.
const DEFINES: usize = 1;
const EQUALS: usize = 2;
const SEMICOLON: usize = 3;
const BAR: usize = 4;
const COMMA: usize = 5;
const GROUP: usize = 6;
const OPTION: usize = 8;
const REPEAT: usize = 10;
const QUESTION: usize = 12;
const STAR: usize = 13;
const PLUS: usize = 14;
const MINUS: usize = 15;
const T_TITLE: usize = 16;
const ID: usize = 19;
const STRING: usize = 20;
const SPECIAL: usize = 21;
const YAML: usize = 23;
const ISO_COMMENT: usize = 27;

fn keyword(src: &str, p: usize) -> Option<usize> {
    keyword_at(src, p, "railroad-ebnf-beta")
}

/// A question mark, text with something other than blanks in it and no `?` or `;`, and a
/// question mark: `/\?(?=[^?;]*[^?\s;][^?;]*\?)[^?;]*\?/y`. Mermaid's pattern looks ahead with
/// two nested runs, which rescans; this reads the text once.
fn special_sequence(src: &str, at: usize) -> Option<usize> {
    if src.as_bytes().get(at) != Some(&b'?') {
        return None;
    }
    let mut solid = false;
    for (k, c) in src[at + 1..].char_indices() {
        if c == '?' {
            return solid.then_some(at + 1 + k + 1);
        }
        if c == ';' {
            return None;
        }
        solid = solid || !is_space_char(c);
    }
    None
}

/// `/\(\*[\s\S]*?\*\)/y`
fn match_iso_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("(*") {
        return None;
    }
    src[p + 2..].find("*)").map(|k| p + 2 + k + 2)
}

/// `/\(\*/y`
fn iso_comment_opener(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with("(*").then_some(p + 2)
}

const HIDDEN: [TokenType; 4] = hidden_tokens(["EBNF_WHITESPACE", "EBNF_YAML", "EBNF_DIRECTIVE", "EBNF_SINGLE_LINE_COMMENT"]);

pub(super) static EBNF_TOKENS: [TokenType; 28] = [
    TokenType::func("railroad-ebnf-beta", keyword, Some("r")).with_longer(&[ID]),
    literal("::="),
    literal("="),
    literal(";"),
    literal("|"),
    literal(","),
    literal("(").with_longer(&[ISO_COMMENT]),
    literal(")"),
    literal("["),
    literal("]"),
    literal("{"),
    literal("}"),
    literal("?").with_longer(&[SPECIAL]),
    literal("*"),
    literal("+"),
    literal("-").with_longer(&[YAML]),
    TITLE,
    ACC_TITLE,
    ACC_DESCR,
    identifier("EBNF_ID"),
    quoted("EBNF_STRING"),
    TokenType {
        name: "EBNF_SPECIAL_SEQUENCE",
        pattern: Pattern::Fn(special_sequence),
        hidden: false,
        opener: None,
        first: Some("?"),
        longer: &[],
    },
    HIDDEN[0],
    HIDDEN[1],
    HIDDEN[2],
    HIDDEN[3],
    TokenType::func("EBNF_BLOCK_COMMENT", match_block_comment, Some("/")).hidden().with_opener(block_comment_opener),
    TokenType::func("EBNF_ISO_COMMENT", match_iso_comment, Some("(")).hidden().with_opener(iso_comment_opener),
];

#[derive(Clone, Copy)]
enum Bracket {
    Group,
    Optional,
    Repetition,
}

enum Postfix {
    Optional,
    ZeroOrMore,
    OneOrMore,
    /// The right side of an exception, an index into the tree.
    Exception(usize),
}

/// A node of the syntax tree. Children are indices into the tree's node list.
enum Ebnf {
    Choice(Vec<usize>),
    Sequence(Vec<usize>),
    Term { base: usize, postfixes: Vec<Postfix> },
    Terminal(String),
    NonTerminal(String),
    Special(String),
    Bracket(Bracket, usize),
}

/// One open bracket, or the rule itself at the bottom of the stack.
struct Frame {
    choice: usize,
    sequence: usize,
    term: Option<usize>,
    // The bracket, its closing token, and whether it is the right side of an exception.
    node: Option<Bracket>,
    close: usize,
    except: bool,
}

/// The closing token of the rule's own frame: none.
const NO_CLOSE: usize = usize::MAX - 1;

fn push(ast: &mut Vec<Ebnf>, node: Ebnf) -> usize {
    ast.push(node);
    ast.len() - 1
}

fn list(ast: &mut [Ebnf], at: usize) -> Option<&mut Vec<usize>> {
    match &mut ast[at] {
        Ebnf::Choice(items) | Ebnf::Sequence(items) => Some(items),
        _ => None,
    }
}

fn frame(ast: &mut Vec<Ebnf>, node: Option<Bracket>, choice: usize, close: usize, except: bool) -> Frame {
    let sequence = push(ast, Ebnf::Sequence(Vec::new()));
    if let Some(alternatives) = list(ast, choice) {
        alternatives.push(sequence);
    }
    Frame { choice, sequence, term: None, node, close, except }
}

fn definition(r: &mut Reader, ast: &mut Vec<Ebnf>) -> Result<usize, Error> {
    let mut stack: Vec<Frame> = Vec::new();
    let root = push(ast, Ebnf::Choice(Vec::new()));
    let mut top = frame(ast, None, root, NO_CLOSE, false);
    let mut except = false;
    loop {
        let kind = r.kind();
        let mut primary = match kind {
            STRING => {
                let value = unescape(r.take());
                push(ast, Ebnf::Terminal(value))
            }
            ID => {
                let name = r.take().to_string();
                push(ast, Ebnf::NonTerminal(name))
            }
            SPECIAL => {
                let text = js::trim(slice_inner(r.take())).to_string();
                push(ast, Ebnf::Special(text))
            }
            GROUP | OPTION | REPEAT => {
                r.i += 1;
                let element = push(ast, Ebnf::Choice(Vec::new()));
                let bracket = match kind {
                    GROUP => Bracket::Group,
                    OPTION => Bracket::Optional,
                    _ => Bracket::Repetition,
                };
                let inner = frame(ast, Some(bracket), element, kind + 1, except);
                stack.push(std::mem::replace(&mut top, inner));
                except = false;
                continue;
            }
            _ => return Err(r.fail("a terminal, a name, or an opening bracket")),
        };
        loop {
            match top.term {
                // An exception's right side belongs to the term before the minus.
                Some(term) if except => {
                    if let Ebnf::Term { postfixes, .. } = &mut ast[term] {
                        postfixes.push(Postfix::Exception(primary));
                    }
                    except = false;
                }
                _ => {
                    let term = push(ast, Ebnf::Term { base: primary, postfixes: Vec::new() });
                    top.term = Some(term);
                    if let Some(elements) = list(ast, top.sequence) {
                        elements.push(term);
                    }
                }
            }
            if let Some(term) = top.term {
                loop {
                    let postfix = match r.kind() {
                        QUESTION => Postfix::Optional,
                        STAR => Postfix::ZeroOrMore,
                        PLUS => Postfix::OneOrMore,
                        _ => break,
                    };
                    r.i += 1;
                    if let Ebnf::Term { postfixes, .. } = &mut ast[term] {
                        postfixes.push(postfix);
                    }
                }
            }
            let k = r.kind();
            if k == top.close {
                r.i += 1;
                let Some(parent) = stack.pop() else {
                    return Err(r.fail("a terminal, a name, or an opening bracket"));
                };
                let closed = std::mem::replace(&mut top, parent);
                primary = push(ast, Ebnf::Bracket(closed.node.unwrap_or(Bracket::Group), closed.choice));
                except = closed.except;
                continue;
            }
            if k == MINUS {
                r.i += 1;
                except = true;
            } else if k == COMMA {
                r.i += 1;
            } else if k == BAR {
                r.i += 1;
                top.sequence = push(ast, Ebnf::Sequence(Vec::new()));
                let sequence = top.sequence;
                if let Some(alternatives) = list(ast, top.choice) {
                    alternatives.push(sequence);
                }
            } else if !matches!(k, STRING | ID | SPECIAL | GROUP | OPTION | REPEAT) {
                if stack.is_empty() {
                    return Ok(top.choice);
                }
                return Err(r.fail(&format!("token of type '{}'", EBNF_TOKENS[top.close].name)));
            }
            break;
        }
    }
}

pub(super) fn parse(src: &str, nodes: &mut Vec<RailroadNode>) -> Result<(Meta, Vec<RailroadRule>), Error> {
    let mut ast: Vec<Ebnf> = Vec::new();
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&EBNF_TOKENS));
    let (meta, rules) = parse_grammar(src, &GRAMMAR, T_TITLE, unescape, |r| {
        let name = r.expect(ID)?.to_string();
        if !r.accept(EQUALS) {
            r.expect(DEFINES)?;
        }
        let definition = definition(r, &mut ast)?;
        r.expect(SEMICOLON)?;
        Ok((name, definition))
    })?;
    Ok((meta, to_rules(rules, ast.as_slice(), nodes, children, node)))
}

fn children(ast: &[Ebnf], n: usize) -> Vec<usize> {
    match &ast[n] {
        Ebnf::Choice(items) | Ebnf::Sequence(items) => items.clone(),
        // A term's parts are its base and the right side of each exception.
        Ebnf::Term { base, postfixes } => {
            let mut list = vec![*base];
            for postfix in postfixes {
                if let Postfix::Exception(except) = postfix {
                    list.push(*except);
                }
            }
            list
        }
        Ebnf::Bracket(_, element) => vec![*element],
        Ebnf::Terminal(_) | Ebnf::NonTerminal(_) | Ebnf::Special(_) => Vec::new(),
    }
}

fn node(ast: &[Ebnf], n: usize, kids: Vec<NodeId>, model: &mut Nodes) -> NodeId {
    match &ast[n] {
        Ebnf::Choice(_) => model.choice(kids),
        Ebnf::Sequence(_) => model.sequence(kids),
        Ebnf::Term { postfixes, .. } => {
            // Each postfix operator applies to everything before it in the term.
            let mut result = kids[0];
            let mut next = 1;
            for postfix in postfixes {
                result = match postfix {
                    Postfix::Optional => model.optional(result),
                    Postfix::Exception(_) => {
                        let minus = model.terminal("-".to_string());
                        let except = kids[next];
                        next += 1;
                        model.sequence_of(vec![result, minus, except])
                    }
                    Postfix::ZeroOrMore => model.repetition(result, 0.0, f64::INFINITY),
                    Postfix::OneOrMore => model.repetition(result, 1.0, f64::INFINITY),
                };
            }
            result
        }
        Ebnf::Terminal(value) => model.terminal(value.clone()),
        Ebnf::NonTerminal(name) => model.nonterminal(name.clone()),
        Ebnf::Special(text) => model.special(text.clone()),
        Ebnf::Bracket(Bracket::Group, _) => kids[0],
        Ebnf::Bracket(Bracket::Optional, _) => model.optional(kids[0]),
        Ebnf::Bracket(Bracket::Repetition, _) => model.repetition(kids[0], 0.0, f64::INFINITY),
    }
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("railroad-ebnf", &super::EBNF_TOKENS, crate::DiagramType::Railroad);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_sequences() {
        assert_eq!(special_sequence("? a ?", 0), Some(5));
        assert_eq!(special_sequence("?  ?", 0), None);
        assert_eq!(special_sequence("? a ; ?", 0), None);
        assert_eq!(special_sequence("? a", 0), None);
        assert_eq!(special_sequence("?😀?x", 0), Some(6));
        assert_eq!(special_sequence("?\u{a0}?", 0), None);
    }

    #[test]
    fn iso_comments() {
        assert_eq!(match_iso_comment("(* a *) b", 0), Some(7));
        assert_eq!(match_iso_comment("(*)", 0), None);
        assert_eq!(match_iso_comment("(**)", 0), Some(4));
    }
}
