//! Architecture diagrams. Ported from Pele `src/diagrams/architecture/`.

mod icons;
mod layout;
mod parser;
mod render;

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_architecture, AlignDirection, ArchitectureAst, Side};
pub(crate) use render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum ArchNodeKind {
    Service,
    Junction,
}

/// A service or a junction. A junction has no icon or title.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArchNode {
    pub id: String,
    pub kind: ArchNodeKind,
    pub icon: Option<String>,
    pub icon_text: Option<String>,
    pub title: Option<String>,
    /// The group the node is in.
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArchGroup {
    pub id: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArchEdge {
    pub lhs_id: String,
    pub lhs_dir: Side,
    pub lhs_into: bool,
    pub lhs_group: bool,
    pub rhs_id: String,
    pub rhs_dir: Side,
    pub rhs_into: bool,
    pub rhs_group: bool,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArchLayoutHint {
    pub direction: AlignDirection,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArchitectureModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub nodes: IndexMap<String, ArchNode>,
    pub groups: IndexMap<String, ArchGroup>,
    pub edges: Vec<ArchEdge>,
    pub layout_hints: Vec<ArchLayoutHint>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Registered {
    Node,
    Group,
}

impl Registered {
    fn name(self) -> &'static str {
        match self {
            Registered::Node => "node",
            Registered::Group => "group",
        }
    }
}

fn fail(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Architecture)
}

/// Pele's `ArchitectureDb`: the model with the checks Mermaid's database makes as it is filled.
struct Db {
    model: ArchitectureModel,
    registered: HashMap<String, Registered>,
}

impl Db {
    fn register(&mut self, kind: &str, id: &str, parent: Option<&str>) -> Result<(), Error> {
        if let Some(other) = self.registered.get(id) {
            return Err(fail(format!("The {} id [{}] is already in use by another {}", kind, id, other.name())));
        }
        if let Some(parent) = parent {
            if id == parent {
                return Err(fail(format!("The {} [{}] cannot be placed within itself", kind, id)));
            }
            match self.registered.get(parent) {
                None => {
                    return Err(fail(format!(
                        "The {} [{}]'s parent does not exist. Please make sure the parent is created before this {}",
                        kind, id, kind
                    )))
                }
                Some(Registered::Node) => return Err(fail(format!("The {} [{}]'s parent is not a group", kind, id))),
                Some(Registered::Group) => {}
            }
        }
        self.registered.insert(id.to_string(), if kind == "group" { Registered::Group } else { Registered::Node });
        Ok(())
    }

    fn add_edge(&mut self, edge: ArchEdge) -> Result<(), Error> {
        let ends = [("left", &edge.lhs_id, edge.lhs_group), ("right", &edge.rhs_id, edge.rhs_group)];
        for (side, id, _) in ends {
            if !self.registered.contains_key(id.as_str()) {
                return Err(fail(format!(
                    "The {}-hand id [{}] does not yet exist. Please create the service/group before declaring an edge to it.",
                    side, id
                )));
            }
        }
        // Mermaid lets a group id through the check above and then fails with a TypeError.
        for (side, id, _) in ends {
            if !self.model.nodes.contains_key(id.as_str()) {
                return Err(fail(format!("The {}-hand id [{}] is a group. Add {{group}} to a service inside it instead.", side, id)));
            }
        }
        let parent = &self.model.nodes[edge.lhs_id.as_str()].parent;
        if parent.is_some() && *parent == self.model.nodes[edge.rhs_id.as_str()].parent {
            for (side, id, crosses) in ends {
                if crosses {
                    return Err(fail(format!(
                        "The {}-hand id [{}] is modified to traverse the group boundary, but the edge does not pass through two groups.",
                        side, id
                    )));
                }
            }
        }
        self.model.edges.push(edge);
        Ok(())
    }

    fn add_layout_hint(&mut self, hint: ArchLayoutHint) -> Result<(), Error> {
        if hint.members.len() < 2 {
            return Err(fail(format!("An align directive requires at least two members; got {}", hint.members.len())));
        }
        let mut seen: HashSet<&str> = HashSet::new();
        for id in &hint.members {
            if self.registered.get(id.as_str()) != Some(&Registered::Node) {
                return Err(fail(format!("align {} references [{}], which is not a service or junction", hint.direction.as_str(), id)));
            }
            if !seen.insert(id.as_str()) {
                return Err(fail(format!("align {} lists [{}] more than once", hint.direction.as_str(), id)));
            }
        }
        self.model.layout_hints.push(hint);
        Ok(())
    }
}

/// Mermaid's populateDb: every group, then services, junctions, edges, and alignments.
pub fn build_architecture(ast: ArchitectureAst, title: Option<&str>) -> Result<ArchitectureModel, Error> {
    let mut db = Db {
        model: ArchitectureModel {
            title: title.filter(|t| !t.is_empty()).map(str::to_string),
            acc_title: None,
            acc_descr: None,
            nodes: IndexMap::new(),
            groups: IndexMap::new(),
            edges: Vec::new(),
            layout_hints: Vec::new(),
        },
        registered: HashMap::new(),
    };
    if let Some(d) = ast.acc_descr.filter(|d| !d.is_empty()) {
        db.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(&d));
    }
    if let Some(t) = ast.acc_title.filter(|t| !t.is_empty()) {
        db.model.acc_title = Some(js::trim_start(&t).to_string());
    }
    if let Some(t) = ast.title.filter(|t| !t.is_empty()) {
        db.model.title = Some(t);
    }
    for group in ast.groups {
        db.register("group", &group.id, group.parent.as_deref())?;
        db.model.groups.insert(group.id.clone(), ArchGroup { id: group.id, icon: group.icon, title: group.title, parent: group.parent });
    }
    for service in ast.services {
        db.register("service", &service.id, service.parent.as_deref())?;
        db.model.nodes.insert(
            service.id.clone(),
            ArchNode {
                id: service.id,
                kind: ArchNodeKind::Service,
                icon: service.icon,
                icon_text: service.icon_text,
                title: service.title,
                parent: service.parent,
            },
        );
    }
    for junction in ast.junctions {
        db.register("junction", &junction.id, junction.parent.as_deref())?;
        db.model.nodes.insert(
            junction.id.clone(),
            ArchNode { id: junction.id, kind: ArchNodeKind::Junction, icon: None, icon_text: None, title: None, parent: junction.parent },
        );
    }
    for edge in ast.edges {
        db.add_edge(ArchEdge {
            lhs_id: edge.lhs_id,
            lhs_dir: edge.lhs_dir,
            lhs_into: edge.lhs_into,
            lhs_group: edge.lhs_group,
            rhs_id: edge.rhs_id,
            rhs_dir: edge.rhs_dir,
            rhs_into: edge.rhs_into,
            rhs_group: edge.rhs_group,
            title: edge.title,
        })?;
    }
    for alignment in ast.alignments {
        db.add_layout_hint(ArchLayoutHint { direction: alignment.direction, members: alignment.members })?;
    }
    Ok(db.model)
}

pub(crate) fn parse(input: &ParseInput) -> Result<ArchitectureModel, Error> {
    build_architecture(parse_architecture(input.source)?, input.title)
}
