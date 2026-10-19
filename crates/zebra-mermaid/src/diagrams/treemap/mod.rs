//! Treemaps. Ported from Pele `src/diagrams/treemap/`.

mod format;
mod layout;
mod parser;
mod render;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use format::{formatter, Formatter};
pub use parser::{parse_treemap, ItemAst, RowAst, TreemapAst};
pub(crate) use render::render;

/// A node of the tree. The nodes of a model are kept in one list, children by index, so that
/// no depth of nesting needs recursion to build, walk or drop.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreemapNode {
    pub name: String,
    /// Sections have a list, which may be empty. Leaves have none.
    pub children: Option<Vec<usize>>,
    pub value: Option<f64>,
    pub class_selector: Option<String>,
    pub css_compiled_styles: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreemapModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// Every node, parents before their children.
    pub nodes: Vec<TreemapNode>,
    /// The nodes at the top level, by index.
    pub roots: Vec<usize>,
    pub classes: IndexMap<String, Vec<String>>,
}

struct Item {
    level: usize,
    name: String,
    leaf: bool,
    value: Option<f64>,
    class_selector: Option<String>,
    css_compiled_styles: Option<Vec<String>>,
}

/// A row belongs to the nearest section above it that is indented less.
fn build_hierarchy(items: Vec<Item>) -> (Vec<TreemapNode>, Vec<usize>) {
    let mut nodes: Vec<TreemapNode> = Vec::with_capacity(items.len());
    let mut roots = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for item in items {
        let index = nodes.len();
        nodes.push(TreemapNode {
            name: item.name,
            children: if item.leaf { None } else { Some(Vec::new()) },
            value: if item.leaf { item.value } else { None },
            class_selector: item.class_selector,
            css_compiled_styles: item.css_compiled_styles,
        });
        while stack.last().is_some_and(|&(_, level)| level >= item.level) {
            stack.pop();
        }
        match stack.last() {
            None => roots.push(index),
            Some(&(parent, _)) => {
                if let Some(children) = nodes[parent].children.as_mut() {
                    children.push(index);
                }
            }
        }
        if !item.leaf {
            stack.push((index, item.level));
        }
    }
    (nodes, roots)
}

pub fn build_treemap(ast: TreemapAst, title: Option<&str>) -> TreemapModel {
    let mut classes: IndexMap<String, Vec<String>> = IndexMap::new();
    for row in &ast.rows {
        let RowAst::ClassDef { class_name, style_text, .. } = row else { continue };
        let styles = classes.entry(class_name.clone()).or_default();
        // An escaped comma stays in the value; the others separate declarations.
        let text = style_text.as_deref().unwrap_or("").replace("\\,", "§§§").replace(',', ";").replace("§§§", ",");
        for style in text.split(';') {
            styles.push(style.to_string());
        }
    }

    let mut items = Vec::new();
    for row in &ast.rows {
        let RowAst::Row { indent, item } = row else { continue };
        let (name, leaf, value, class_selector) = match item {
            ItemAst::Section { name, class_selector } => (name, false, None, class_selector),
            ItemAst::Leaf { name, value, class_selector } => (name, true, Some(*value), class_selector),
        };
        // `item.classSelector ? classes.get(...)`: an empty selector cannot be read.
        let styles = class_selector.as_deref().filter(|s| !s.is_empty()).and_then(|s| classes.get(s));
        items.push(Item {
            level: indent.unwrap_or(0),
            name: name.clone(),
            leaf,
            value,
            class_selector: class_selector.clone(),
            css_compiled_styles: styles.filter(|s| !s.is_empty()).cloned(),
        });
    }

    let (nodes, roots) = build_hierarchy(items);
    TreemapModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        nodes,
        roots,
        classes,
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<TreemapModel, Error> {
    Ok(build_treemap(parse_treemap(input.source)?, input.title))
}
