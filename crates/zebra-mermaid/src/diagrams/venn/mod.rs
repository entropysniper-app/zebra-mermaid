//! Venn diagrams. Ported from Pele `src/diagrams/venn/`.

mod db;
mod layout;
mod lexer;
mod parser;
mod render;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub use db::normalize_text;
pub use parser::parse_venn;
pub(crate) use render::render;

/// A set (one id) or an overlap of sets (two or more ids, sorted), with its relative area.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VennSubset {
    pub sets: Vec<String>,
    pub size: f64,
    pub label: Option<String>,
}

/// A line of text shown inside the region of the given sets.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VennText {
    pub sets: Vec<String>,
    pub id: String,
    pub label: Option<String>,
}

/// Targets are the sorted ids of a set or overlap, or the id of a text.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VennStyle {
    pub targets: Vec<String>,
    pub styles: IndexMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VennModel {
    pub title: Option<String>,
    pub subsets: Vec<VennSubset>,
    pub text_nodes: Vec<VennText>,
    pub styles: Vec<VennStyle>,
}

pub(crate) fn parse(input: &ParseInput) -> Result<VennModel, Error> {
    let mut model = VennModel { title: input.title.filter(|t| !t.is_empty()).map(str::to_string), ..Default::default() };
    parse_venn(input.source, &mut model)?;
    Ok(model)
}
