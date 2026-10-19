//! Cynefin framework diagrams. Ported from Pele `src/diagrams/cynefin/`.

mod boundaries;
mod parser;
mod render;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_cynefin, CynefinAst, DomainBlockAst, TransitionAst};
pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CynefinItem {
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CynefinDomain {
    /// One of `complex`, `complicated`, `clear`, `chaotic` and `confusion`.
    pub name: String,
    pub items: Vec<CynefinItem>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CynefinTransition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CynefinModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// The domains the source declares, by name, in the order first declared. All five are
    /// drawn either way.
    pub domains: IndexMap<String, CynefinDomain>,
    pub transitions: Vec<CynefinTransition>,
}

/// Pele's `buildCynefin`.
pub fn build_cynefin(ast: CynefinAst, title: Option<&str>) -> CynefinModel {
    // A later block for a domain replaces an earlier one, in the place of the first, as a
    // `Map` keeps it.
    let mut domains = IndexMap::new();
    for block in ast.domains {
        let items = block.items.into_iter().map(|label| CynefinItem { label }).collect();
        domains.insert(block.domain.clone(), CynefinDomain { name: block.domain, items });
    }
    let transitions = ast
        .transitions
        .into_iter()
        .filter(|t| t.from != t.to)
        .map(|t| CynefinTransition { from: t.from, to: t.to, label: t.label.filter(|l| !l.is_empty()) })
        .collect();
    CynefinModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        domains,
        transitions,
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<CynefinModel, Error> {
    Ok(build_cynefin(parse_cynefin(input.source)?, input.title))
}
