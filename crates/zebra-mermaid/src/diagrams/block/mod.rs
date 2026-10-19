//! Block diagrams. Ported from Pele `src/diagrams/block/`.

mod arrows;
mod layout;
mod lexer;
mod parser;
mod render;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

/// One shape for blocks, edges and the statements that set columns, classes and styles, as in
/// Mermaid. Pele's blocks are objects that later statements change in place; here they live in
/// [`BlockModel::blocks`] and refer to each other by index.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Block {
    /// Unset only on a composite whose header was a link statement, until the hierarchy is
    /// built.
    pub id: Option<String>,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: String,
    pub label: Option<String>,
    /// The blocks inside a composite, by index.
    pub children: Vec<usize>,
    pub columns: Option<f64>,
    pub width_in_columns: Option<f64>,
    /// Number of cells a `space` statement asked for.
    pub width: Option<f64>,
    pub directions: Option<Vec<String>>,
    /// Index into [`BlockModel::class_lists`]. Pele shares one array between a block and the
    /// copies made of it, so that a class added through one shows on all of them.
    pub classes: Option<usize>,
    pub styles: Option<Vec<String>>,
    pub color_index: Option<usize>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub arrow_type_start: Option<String>,
    pub arrow_type_end: Option<String>,
    pub thickness: Option<String>,
    pub pattern: Option<String>,
    pub css: Option<String>,
    pub style_class: Option<String>,
    pub styles_str: Option<String>,
    /// The statements of a composite before the hierarchy is built.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) statements: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BlockClassDef {
    pub id: String,
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BlockModel {
    pub title: Option<String>,
    /// Every block, edge and statement. Block 0 is the root.
    pub blocks: Vec<Block>,
    /// Indices of the edges, in the order the hierarchy met them.
    pub edges: Vec<usize>,
    pub classes: IndexMap<String, BlockClassDef>,
    pub class_lists: Vec<Vec<String>>,
}

impl BlockModel {
    pub const ROOT: usize = 0;

    /// The classes of a block, if it has any.
    pub fn classes_of(&self, block: &Block) -> Option<&[String]> {
        block.classes.map(|k| self.class_lists[k].as_slice())
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<BlockModel, Error> {
    parser::parse_block(input.source, input.title)
}
