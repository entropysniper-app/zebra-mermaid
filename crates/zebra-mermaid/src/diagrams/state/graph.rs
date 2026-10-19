//! Mermaid's state document traversal, preserving first insertion and later updates.
use super::{direction, Description, State, StateModel, Stmt};
use crate::{util::js, Error};
use indexmap::IndexMap;
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Node {
    pub id: String,
    pub shape: Option<String>,
    pub label: Option<String>,
    pub description: Vec<String>,
    pub classes: String,
    pub compiled: Vec<String>,
    pub styles: Vec<String>,
    pub kind: Option<String>,
    pub group: bool,
    pub parent: Option<String>,
    pub dir: Option<String>,
    pub position: Option<String>,
    pub note_for: Option<String>,
    pub color_index: Option<usize>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Edge {
    pub id: String,
    pub start: String,
    pub end: String,
    pub label: String,
    pub dashed: bool,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
struct Draft {
    shape: Option<String>,
    description: Description,
    classes: String,
    styles: Vec<String>,
    kind: Option<String>,
    dir: Option<String>,
    color_index: Option<usize>,
}
struct Builder<'a> {
    db: &'a StateModel,
    nodes: IndexMap<String, Node>,
    edges: Vec<Edge>,
    drafts: IndexMap<String, Draft>,
    slots: IndexMap<String, Option<usize>>,
    counter: usize,
    next_slot: usize,
}
impl Builder<'_> {
    fn insert(&mut self, mut node: Node) {
        if node.id.is_empty() || matches!(node.id.as_str(), "</join></fork>" | "</choice>") {
            return;
        }
        for name in node.classes.split(' ') {
            if let Some(def) = self.db.classes.get(name) {
                node.compiled.extend(def.iter().cloned());
            }
        }
        if node.parent.is_none() {
            node.parent = self.nodes.get(&node.id).and_then(|n| n.parent.clone());
        }
        self.nodes.insert(node.id.clone(), node);
    }
    fn visit(&mut self, parent: Option<&str>, item: &State, alt: bool) {
        let id = &item.id;
        if id == "root" {
            return;
        }
        let known = self.db.known.get(id);
        let class_str = known.map_or_else(String::new, |s| s.classes.join(" "));
        let styles = known.map_or_else(Vec::new, |s| s.styles.clone());
        let draft = self.drafts.entry(id.clone()).or_insert_with(|| {
            let mut shape = Some("rect".to_string());
            if let Some(start) = item.start {
                shape = Some(if start { "stateStart" } else { "stateEnd" }.into());
            }
            if item.kind.as_deref() != Some("default") {
                shape = item.kind.clone();
            }
            Draft {
                shape,
                description: Description::Text(id.clone()),
                classes: format!("{class_str} statediagram-state"),
                styles: styles.clone(),
                kind: None,
                dir: None,
                color_index: None,
            }
        });
        if let Some(desc) = item.description.as_ref().filter(|s| s.truthy()) {
            match &draft.description {
                Description::Lines(v) => {
                    draft.shape = Some("rectWithTitle".into());
                    let mut v = v.clone();
                    v.extend(desc.values());
                    draft.description = Description::Lines(v);
                }
                Description::Text(s) if !s.is_empty() => {
                    draft.shape = Some("rectWithTitle".into());
                    let mut v = if s == id { vec![] } else { vec![s.clone()] };
                    v.extend(desc.values());
                    draft.description = Description::Lines(v);
                }
                _ => {
                    draft.shape = Some("rect".into());
                    draft.description = desc.clone();
                }
            }
        }
        if draft.description.len() == 1 && draft.shape.as_deref() == Some("rectWithTitle") {
            draft.shape = Some(if draft.kind.as_deref() == Some("group") { "roundedWithTitle" } else { "rect" }.into());
        }
        if draft.kind.is_none() {
            if let Some(doc) = item.doc {
                draft.kind = Some("group".into());
                draft.dir = Some(direction(&self.db.docs[doc]));
                draft.shape = Some(if item.kind.as_deref() == Some("divider") { "divider" } else { "roundedWithTitle" }.into());
                draft.color_index = if draft.shape.as_deref() == Some("divider") && parent.is_some_and(|p| self.slots.contains_key(p)) {
                    self.slots[parent.unwrap()]
                } else {
                    let slot = self.next_slot;
                    self.next_slot += 1;
                    if !js::trim(&class_str).is_empty() || !styles.is_empty() {
                        None
                    } else {
                        Some(slot)
                    }
                };
                self.slots.insert(id.clone(), draft.color_index);
                draft.classes = format!("{} statediagram-cluster {}", draft.classes, if alt { "statediagram-cluster-alt" } else { "" });
            }
        }
        let (label, description) = match &draft.description {
            Description::Text(s) => (Some(s.clone()), vec![]),
            Description::Lines(v) => (v.first().cloned(), v.get(1..).unwrap_or_default().to_vec()),
        };
        let mut node = Node {
            id: id.clone(),
            shape: draft.shape.clone(),
            label,
            description,
            classes: draft.classes.clone(),
            styles: draft.styles.clone(),
            kind: draft.kind.clone(),
            group: draft.kind.as_deref() == Some("group"),
            dir: draft.dir.clone(),
            color_index: draft.color_index,
            ..Default::default()
        };
        if node.shape.as_deref() == Some("divider") {
            node.label = Some(String::new());
        }
        node.parent = parent.filter(|p| *p != "root").map(str::to_owned);
        let Some(note) = &item.note else {
            self.insert(node);
            return;
        };
        let classes = draft.classes.clone();
        let group_id = format!("{id}----parent");
        let note_id = format!("{id}----note-{}", self.counter);
        self.counter += 1;
        self.insert(Node {
            id: group_id.clone(),
            shape: Some("noteGroup".into()),
            label: Some(note.text.clone()),
            classes,
            kind: Some("group".into()),
            group: true,
            position: Some(note.position.clone()),
            ..Default::default()
        });
        self.insert(Node {
            id: note_id.clone(),
            shape: Some("note".into()),
            label: Some(note.text.clone()),
            classes: "statediagram-note".into(),
            kind: Some("node".into()),
            parent: Some(group_id),
            note_for: Some(id.clone()),
            position: Some(note.position.clone()),
            ..Default::default()
        });
        self.insert(node);
        let (start, end) = if note.position == "left of" { (note_id, id.clone()) } else { (id.clone(), note_id) };
        self.edges.push(Edge { id: format!("{start}-{end}"), start, end, label: String::new(), dashed: true });
    }
}
pub fn build(db: &StateModel) -> Result<Graph, Error> {
    let mut b =
        Builder { db, nodes: IndexMap::new(), edges: vec![], drafts: IndexMap::new(), slots: IndexMap::new(), counter: 0, next_slot: 0 };
    let mut stack = vec![(None::<String>, 0usize, 0usize, true)];
    while let Some((parent, doc, at, alt)) = stack.last_mut() {
        let Some(item) = db.docs[*doc].get(*at) else {
            stack.pop();
            continue;
        };
        *at += 1;
        match item {
            Stmt::State(s) => {
                b.visit(parent.as_deref(), s, *alt);
                if let Some(child) = s.doc {
                    let child_alt = !*alt;
                    stack.push((Some(s.id.clone()), child, 0, child_alt));
                }
            }
            Stmt::Relation { first, second, label } => {
                b.visit(parent.as_deref(), first, *alt);
                b.visit(parent.as_deref(), second, *alt);
                b.edges.push(Edge {
                    id: format!("edge{}", b.counter),
                    start: first.id.clone(),
                    end: second.id.clone(),
                    label: label.clone().unwrap_or_default(),
                    dashed: false,
                });
                b.counter += 1;
            }
            _ => {}
        }
    }
    for node in b.nodes.values() {
        if node.group && !node.description.is_empty() {
            return Err(Error::semantic(format!(
                "Group nodes can only have label. Remove the additional description for node [{}]",
                node.id
            ))
            .with_type(crate::DiagramType::State));
        }
    }
    Ok(Graph { nodes: b.nodes.into_values().collect(), edges: b.edges })
}
