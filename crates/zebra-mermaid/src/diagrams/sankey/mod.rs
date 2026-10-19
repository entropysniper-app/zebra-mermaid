//! Sankey diagrams. Ported from Pele `src/diagrams/sankey/`.

mod layout;
mod parser;
mod render;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub use parser::{parse_sankey, prepare_text_for_parsing};
pub(crate) use render::render;

/// A node of the flow, in the order its name first appears.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SankeyNode {
    pub id: String,
    pub index: usize,
}

/// A flow between two nodes, by their indexes, with the amount as `parseFloat` read it.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SankeyLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SankeyModel {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub title: Option<String>,
    /// Pele's sankey grammar has no accessibility statements; these stay empty.
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
}

pub(crate) fn parse(input: &ParseInput) -> Result<SankeyModel, Error> {
    let mut model = SankeyModel { title: input.title.filter(|t| !t.is_empty()).map(str::to_string), ..Default::default() };
    parse_sankey(input.source, &mut model)?;
    Ok(model)
}
