//! Kanban boards. Ported from Pele `src/diagrams/kanban/`.

mod render;

use crate::config::yaml::parse_yaml;
use crate::config::ConfigValue;
use crate::detect::DiagramType;
use crate::diagrams::common::outline::{parse_outline, OutlineDb};
use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct KanbanNode {
    pub id: String,
    pub level: i64,
    pub label: String,
    pub icon: Option<String>,
    pub assigned: Option<String>,
    pub ticket: Option<String>,
    pub priority: Option<String>,
    pub css_classes: Option<String>,
}

/// A column: the node that opens it and its cards.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct KanbanSection {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub node: KanbanNode,
    pub items: Vec<KanbanNode>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct KanbanModel {
    pub title: Option<String>,
    pub sections: Vec<KanbanSection>,
}

/// Pele's `KanbanDb`.
#[derive(Default)]
struct KanbanDb {
    model: KanbanModel,
    /// The last node added, as its section and its place among the section's items.
    last: Option<(usize, Option<usize>)>,
    section_level: i64,
    /// The label of a node indented less than the first column.
    misplaced: Option<String>,
}

fn kanban_error(message: String, semantic: bool) -> Error {
    let e = if semantic { Error::semantic(message) } else { Error::syntax(message) };
    e.with_type(DiagramType::Kanban)
}

impl OutlineDb for KanbanDb {
    fn add_node(&mut self, level: i64, id: &str, descr: &str, _kind: u8, shape_data: Option<&str>) -> Result<(), Error> {
        let mut node = KanbanNode { id: id.to_string(), level, label: descr.to_string(), ..Default::default() };

        if let Some(shape_data) = shape_data {
            let yaml = if shape_data.contains('\n') { format!("{}\n", shape_data) } else { format!("{{\n{}\n}}", shape_data) };
            let doc = parse_yaml(&yaml)
                .map_err(|e| kanban_error(format!("Metadata of \"{}\" is not valid YAML. {}", node.id, e.message), false))?;
            // Mermaid throws a TypeError on metadata that is empty or not a mapping. Here it is ignored.
            if let ConfigValue::Object(doc) = &doc {
                let text = |key: &str| doc.get(key).filter(|v| v.truthy()).map(ConfigValue::to_js_string);
                let shape = text("shape").unwrap_or_default();
                if shape != shape.to_lowercase() || shape.contains('_') {
                    return Err(kanban_error(format!("No such shape: {}. Shape names should be lowercase.", shape), true));
                }
                if let Some(label) = text("label") {
                    node.label = label;
                }
                if let Some(icon) = text("icon") {
                    node.icon = Some(icon);
                }
                if let Some(assigned) = text("assigned") {
                    node.assigned = Some(assigned);
                }
                if let Some(ticket) = text("ticket") {
                    node.ticket = Some(ticket);
                }
                if let Some(priority) = text("priority") {
                    node.priority = Some(priority);
                }
            }
        }

        // A node indented less than the first column becomes a card, and Mermaid rejects whatever follows it.
        if let Some(label) = &self.misplaced {
            return Err(kanban_error(format!("Items without section detected, found section (\"{}\")", label), true));
        }
        let sections = &mut self.model.sections;
        if sections.is_empty() {
            self.section_level = level;
        }
        if level == self.section_level {
            sections.push(KanbanSection { node, items: Vec::new() });
            self.last = Some((sections.len() - 1, None));
        } else {
            let index = sections.len() - 1;
            if level < self.section_level {
                self.misplaced = Some(node.label.clone());
            }
            let items = &mut sections[index].items;
            items.push(node);
            self.last = Some((index, Some(items.len() - 1)));
        }
        Ok(())
    }

    /// Mermaid throws a TypeError when a decoration comes before any node. Here it is ignored.
    fn decorate_node(&mut self, icon: Option<&str>, class: Option<&str>) {
        let Some((section, item)) = self.last else { return };
        let section = &mut self.model.sections[section];
        let node = match item {
            Some(k) => &mut section.items[k],
            None => &mut section.node,
        };
        if let Some(icon) = icon.filter(|s| !s.is_empty()) {
            node.icon = Some(icon.to_string());
        }
        if let Some(class) = class.filter(|s| !s.is_empty()) {
            node.css_classes = Some(class.to_string());
        }
    }
}

/// Parses a kanban board with Pele's outline grammar.
pub fn parse_kanban(src: &str, title: Option<&str>) -> Result<KanbanModel, Error> {
    let mut db = KanbanDb::default();
    db.model.title = title.map(str::to_string);
    parse_outline(src, &mut db, true)?;
    Ok(db.model)
}

pub(crate) fn parse(input: &ParseInput) -> Result<KanbanModel, Error> {
    parse_kanban(input.source, input.title)
}
