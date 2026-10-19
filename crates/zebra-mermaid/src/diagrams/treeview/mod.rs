//! File trees. Ported from Pele `src/diagrams/treeview/`.

mod box_drawing;
mod icons;
mod parser;
mod render;

use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use box_drawing::{preprocess_box_drawing, PreprocessResult};
pub use parser::{parse_tree_view, TreeNodeAst, TreeViewAst};
pub(crate) use render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreeNode {
    pub id: usize,
    pub level: i64,
    pub name: String,
    pub node_type: NodeType,
    /// An icon name as written (`pack:name`, `file`, `folder`), or `none` to hide the icon.
    pub icon: Option<String>,
    pub css_class: Option<String>,
    pub description: Option<String>,
    /// The children, by index into the model's nodes.
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreeViewModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// Every node, the root first: a directory named `/` at level -1 that holds the top-level
    /// entries. Nodes are kept in one list, children by index, so that no depth of nesting
    /// needs recursion to build, walk or drop.
    pub nodes: Vec<TreeNode>,
    pub count: usize,
}

/// Mermaid's treeView database: nodes arrive in source order with their indentation.
struct TreeViewDb {
    nodes: Vec<TreeNode>,
    stack: Vec<usize>,
}

impl TreeViewDb {
    fn new() -> Self {
        let root = TreeNode {
            id: 0,
            level: -1,
            name: "/".into(),
            node_type: NodeType::Directory,
            icon: None,
            css_class: None,
            description: None,
            children: Vec::new(),
        };
        TreeViewDb { nodes: vec![root], stack: vec![0] }
    }

    fn add_node(
        &mut self,
        level: i64,
        name: String,
        node_type: NodeType,
        css_class: Option<String>,
        icon: Option<String>,
        description: Option<String>,
    ) {
        // The root is at level -1 and every entry at 0 or more, so the root is never popped.
        while self.stack.len() > 1 && self.stack.last().is_some_and(|&top| level <= self.nodes[top].level) {
            self.stack.pop();
        }
        let id = self.nodes.len();
        self.nodes.push(TreeNode { id, level, name, node_type, icon, css_class, description, children: Vec::new() });
        if let Some(&parent) = self.stack.last() {
            self.nodes[parent].children.push(id);
        }
        self.stack.push(id);
    }
}

fn populate(ast: TreeViewAst, db: &mut TreeViewDb) {
    for node in ast.nodes {
        let level = node.indent.map_or(0, |i| i as i64);
        let mut name = node.name;
        let is_directory = name.ends_with('/');
        if is_directory {
            name.pop();
        }
        db.add_node(
            level,
            name,
            if is_directory { NodeType::Directory } else { NodeType::File },
            node.class_annotation.filter(|c| !c.is_empty()),
            // `rawIcon !== undefined ? rawIcon || 'none' : undefined`
            node.icon_annotation.map(|i| if i.is_empty() { "none".to_string() } else { i }),
            node.desc_annotation.filter(|d| !d.is_empty()),
        );
    }
}

/// Reads the source to a syntax tree, accepting box-drawing input. Errors name lines of the
/// original text.
pub fn read_tree_view(source: &str) -> Result<TreeViewAst, Error> {
    let pre = preprocess_box_drawing(source).map_err(|(mut error, offset)| {
        error.offset = Some(offset);
        error
    })?;
    let error = match parse_tree_view(&pre.text) {
        Ok(ast) => return Ok(ast),
        Err(error) => error,
    };
    if pre.line_map.is_empty() {
        return Err(error);
    }
    let message = match error.message.find('\n') {
        None => box_drawing::remap_error_lines(&error.message, &pre),
        Some(head) => box_drawing::remap_error_lines(&error.message[..head], &pre) + &error.message[head..],
    };
    let line = pre.original(error.line).unwrap_or(error.line);
    let mut remapped = Error::new(message, error.code).with_location(line, error.column, error.snippet.clone());
    remapped.diagram_type = error.diagram_type;
    // The offset was in the rewritten text; the error points at the start of its original line.
    remapped.offset = Some(line_start(source, line));
    Err(remapped)
}

/// The byte offset where 1-based `line` of `text` starts.
fn line_start(text: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    text.match_indices('\n').nth(line - 2).map_or(text.len(), |(at, _)| at + 1)
}

pub fn build_tree_view(ast: TreeViewAst, title: Option<&str>) -> TreeViewModel {
    let mut db = TreeViewDb::new();
    let (ast_title, acc_title, acc_descr) = (ast.title.clone(), ast.acc_title.clone(), ast.acc_descr.clone());
    populate(ast, &mut db);
    let count = db.nodes.len();
    TreeViewModel {
        title: ast_title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        nodes: db.nodes,
        count,
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<TreeViewModel, Error> {
    Ok(build_tree_view(read_tree_view(input.source)?, input.title))
}
