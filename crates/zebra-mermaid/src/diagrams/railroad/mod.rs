//! Railroad diagrams in Mermaid's four grammars: `railroad-beta` (Mermaid's own notation),
//! `railroad-ebnf-beta`, `railroad-abnf-beta` and `railroad-peg-beta`. Ported from Pele
//! `src/diagrams/railroad/` (`index.ts`, `types.ts`, `db.ts`).
//!
//! Pele parses nested brackets with explicit stacks and rebuilds and draws the trees with
//! loops, so no input depth can exhaust its stack: it renders any nesting that fits in memory
//! (checked with the oracle up to a million nested groups). The port does the same. The model
//! keeps its nodes in one list and refers to children by index, so that dropping, cloning,
//! printing or serializing a deeply nested model does not recurse either. There is therefore
//! no depth limit here, and no input is refused that Pele draws.

mod abnf;
mod common;
mod ebnf;
mod ir;
mod peg;
mod render;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::chars::is_space;
use crate::util::js;

pub(crate) use render::render;

/// Which of Mermaid's four grammars the text was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Notation {
    Railroad,
    Ebnf,
    Abnf,
    Peg,
}

/// The index of a node in [`RailroadModel::nodes`].
pub type NodeId = usize;

/// A part of a rule's definition. Children are indices into [`RailroadModel::nodes`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum RailroadNode {
    Terminal {
        value: String,
    },
    Nonterminal {
        name: String,
    },
    Special {
        text: String,
    },
    Sequence {
        elements: Vec<NodeId>,
    },
    Choice {
        alternatives: Vec<NodeId>,
    },
    Optional {
        element: NodeId,
    },
    /// `max` is infinite when the count has no upper bound.
    Repetition {
        element: NodeId,
        min: f64,
        max: f64,
    },
}

impl RailroadNode {
    /// The node's children in order: Pele's `parts()` on the rule model.
    pub fn parts(&self) -> &[NodeId] {
        match self {
            RailroadNode::Sequence { elements } => elements,
            RailroadNode::Choice { alternatives } => alternatives,
            RailroadNode::Optional { element } | RailroadNode::Repetition { element, .. } => std::slice::from_ref(element),
            RailroadNode::Terminal { .. } | RailroadNode::Nonterminal { .. } | RailroadNode::Special { .. } => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RailroadRule {
    pub name: String,
    pub definition: NodeId,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RailroadModel {
    pub notation: Notation,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// Every rule in source order. A name defined twice appears twice.
    pub rules: Vec<RailroadRule>,
    /// The nodes of every rule's definition, each child before its parent.
    pub nodes: Vec<RailroadNode>,
}

impl RailroadModel {
    pub fn node(&self, id: NodeId) -> &RailroadNode {
        &self.nodes[id]
    }

    /// The rule a name refers to. A name defined again is drawn again; looking the name up
    /// gives the later definition, as Pele's `getRule` does.
    pub fn rule(&self, name: &str) -> Option<&RailroadRule> {
        self.rules.iter().rev().find(|r| r.name == name)
    }
}

/// `/^\s*railroad-(ebnf-|abnf-|peg-)?beta/i`: Mermaid has a detector per keyword, each
/// matching without regard to case. `/i` without `/u` folds ASCII letters only.
pub fn notation_of(source: &str) -> Option<Notation> {
    let start = source.find(|c: char| !is_space(c)).unwrap_or(source.len());
    let rest = &source[start..];
    let starts_ci = |s: &str, word: &str| s.len() >= word.len() && s.as_bytes()[..word.len()].eq_ignore_ascii_case(word.as_bytes());
    if !starts_ci(rest, "railroad-") {
        return None;
    }
    let rest = &rest[9..];
    for (infix, notation) in [("ebnf-", Notation::Ebnf), ("abnf-", Notation::Abnf), ("peg-", Notation::Peg)] {
        if starts_ci(rest, infix) && starts_ci(&rest[infix.len()..], "beta") {
            return Some(notation);
        }
    }
    starts_ci(rest, "beta").then_some(Notation::Railroad)
}

/// What every grammar's syntax tree gives the model besides its rules.
#[derive(Default)]
struct Meta {
    title: Option<String>,
    acc_title: Option<String>,
    acc_descr: Option<String>,
}

/// Parses the text in one notation and loads the result into the model, as Mermaid's four
/// parsers do (`populate` in Pele's `db.ts`).
fn populate(model: &mut RailroadModel, source: &str) -> Result<(), Error> {
    let (meta, rules) = match model.notation {
        Notation::Ebnf => ebnf::parse(source, &mut model.nodes)?,
        Notation::Abnf => abnf::parse(source, &mut model.nodes)?,
        Notation::Peg => peg::parse(source, &mut model.nodes)?,
        Notation::Railroad => ir::parse(source, &mut model.nodes)?,
    };
    // `setAccDescription`, `setAccTitle` and `setTitle`, each only for a value that is set.
    if let Some(d) = meta.acc_descr.filter(|d| !d.is_empty()) {
        model.acc_descr = Some(crate::diagrams::squeeze_line_starts(&d));
    }
    if let Some(t) = meta.acc_title.filter(|t| !t.is_empty()) {
        model.acc_title = Some(js::trim_start(&t).to_string());
    }
    if let Some(t) = meta.title.filter(|t| !t.is_empty()) {
        model.title = Some(t);
    }
    model.rules = rules;
    Ok(())
}

pub fn parse_railroad(source: &str, title: Option<&str>) -> Result<RailroadModel, Error> {
    let Some(notation) = notation_of(source) else {
        return Err(Error::syntax("No railroad diagram keyword found.").with_type(DiagramType::Railroad));
    };
    let mut model = RailroadModel {
        notation,
        title: title.filter(|t| !t.is_empty()).map(str::to_string),
        acc_title: None,
        acc_descr: None,
        rules: Vec::new(),
        nodes: Vec::new(),
    };
    populate(&mut model, source)?;
    Ok(model)
}

pub(crate) fn parse(input: &ParseInput) -> Result<RailroadModel, Error> {
    parse_railroad(input.source, input.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notations() {
        assert_eq!(notation_of("railroad-beta\n"), Some(Notation::Railroad));
        assert_eq!(notation_of("  \n Railroad-EBNF-Beta"), Some(Notation::Ebnf));
        assert_eq!(notation_of("railroad-abnf-betax"), Some(Notation::Abnf));
        assert_eq!(notation_of("RAILROAD-PEG-BETA"), Some(Notation::Peg));
        assert_eq!(notation_of("railroad-ebnf-x"), None);
        assert_eq!(notation_of("railroad-peg"), None);
        assert_eq!(notation_of("\u{feff}railroad-beta"), Some(Notation::Railroad));
        assert_eq!(notation_of("ſailroad-beta"), None);
    }

    #[test]
    fn deep_models_drop_without_recursion() {
        let n = 200_000;
        let src = format!("railroad-ebnf-beta\nr = {}\"a\"{};\n", "[".repeat(n), "]".repeat(n));
        let model = parse_railroad(&src, None).unwrap();
        assert_eq!(model.nodes.len(), n + 1);
        let copy = model.clone();
        assert_eq!(copy, model);
        drop(copy);
    }
}
