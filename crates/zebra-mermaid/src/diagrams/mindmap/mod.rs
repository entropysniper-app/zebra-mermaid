//! Mind maps. Ported from Pele `src/diagrams/mindmap/`.

mod render;

use crate::detect::DiagramType;
use crate::diagrams::common::outline::{parse_outline, OutlineDb};
use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MindmapNode {
    /// The node's index in [`MindmapModel::nodes`].
    pub id: usize,
    /// The id written in the source, or the text of a node written without one.
    pub node_id: String,
    pub level: i64,
    pub descr: String,
    /// Mermaid's node type, see [`crate::diagrams::common::outline::node_type`].
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: u8,
    /// Indices of the children in [`MindmapModel::nodes`].
    pub children: Vec<usize>,
    pub is_root: bool,
    /// Index of the first-level branch the node belongs to, wrapping after eleven as in Mermaid.
    pub section: Option<usize>,
    pub class: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MindmapModel {
    pub title: Option<String>,
    /// In source order, which is depth-first. The first node is the root.
    pub nodes: Vec<MindmapNode>,
}

const SECTIONS: usize = 11;

/// Pele's `MindmapDb`.
#[derive(Default)]
struct MindmapDb {
    model: MindmapModel,
    base_level: i64,
    /// The chain of nodes that can still take children, with strictly increasing levels.
    open: Vec<usize>,
}

impl OutlineDb for MindmapDb {
    fn add_node(&mut self, level: i64, id: &str, descr: &str, kind: u8, _shape_data: Option<&str>) -> Result<(), Error> {
        let nodes = &mut self.model.nodes;
        let is_root = nodes.is_empty();
        if is_root {
            self.base_level = level;
        }
        let level = level - self.base_level;

        // Mermaid's parent is the last node with a smaller indentation, wherever it is in the tree.
        let open = &mut self.open;
        let mut depth = open.len();
        while depth > 0 && nodes[open[depth - 1]].level >= level {
            depth -= 1;
        }
        let parent = if depth > 0 { Some(open[depth - 1]) } else { None };
        if parent.is_none() && !is_root {
            return Err(Error::semantic(format!("There can be only one root. No parent could be found for (\"{}\")", descr))
                .with_type(DiagramType::Mindmap));
        }
        let index = nodes.len();
        let mut node = MindmapNode {
            id: index,
            node_id: id.to_string(),
            level,
            descr: descr.to_string(),
            kind,
            children: Vec::new(),
            is_root,
            section: None,
            class: None,
            icon: None,
        };
        if let Some(p) = parent {
            let up = &mut nodes[p];
            node.section = if up.is_root { Some(up.children.len() % SECTIONS) } else { up.section };
            up.children.push(index);
        }
        open.truncate(depth);
        open.push(index);
        nodes.push(node);
        Ok(())
    }

    /// Mermaid throws a TypeError when a decoration comes before any node. Here it is ignored.
    fn decorate_node(&mut self, icon: Option<&str>, class: Option<&str>) {
        let Some(node) = self.model.nodes.last_mut() else { return };
        if let Some(icon) = icon.filter(|s| !s.is_empty()) {
            node.icon = Some(icon.to_string());
        }
        if let Some(class) = class.filter(|s| !s.is_empty()) {
            node.class = Some(class.to_string());
        }
    }
}

/// Parses a mind map with Pele's outline grammar.
pub fn parse_mindmap(src: &str, title: Option<&str>) -> Result<MindmapModel, Error> {
    let mut db = MindmapDb::default();
    db.model.title = title.map(str::to_string);
    parse_outline(src, &mut db, false)?;
    Ok(db.model)
}

pub(crate) fn parse(input: &ParseInput) -> Result<MindmapModel, Error> {
    parse_mindmap(input.source, input.title)
}
