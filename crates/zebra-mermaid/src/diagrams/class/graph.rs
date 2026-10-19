//! Class graph with distinct indexes for ambiguous note/interface names.
use super::{ClassModel, ClassNode};
use indexmap::IndexMap;
#[derive(Clone, Copy, PartialEq)]
pub enum Shape {
    Rect,
    Class,
    Note,
    Interface,
}
pub struct Node<'a> {
    pub id: String,
    pub label: String,
    pub group: bool,
    pub shape: Shape,
    pub parent: isize,
    pub class: Option<&'a ClassNode>,
}
pub struct Edge {
    pub id: String,
    pub source: isize,
    pub target: isize,
    pub label: Option<String>,
    pub tail: &'static str,
    pub head: &'static str,
    pub tail_label: String,
    pub head_label: String,
    pub pattern: &'static str,
}
pub fn build(model: &ClassModel, hierarchical: bool) -> (Vec<Node<'_>>, Vec<Edge>) {
    let mut nodes: Vec<Node> = vec![];
    let mut edges = vec![];
    let mut ns_at = IndexMap::<String, usize>::new();
    let mut class_at = IndexMap::<String, usize>::new();
    let mut interface_at = IndexMap::<String, usize>::new();
    let ancestor = |id: Option<&str>| -> Option<String> {
        let mut id = id?;
        if hierarchical {
            return Some(id.into());
        }
        for _ in 0..=model.namespaces.len() {
            let ns = model.namespaces.get(id)?;
            if ns.explicit {
                return Some(id.into());
            }
            id = ns.parent.as_deref()?;
        }
        None
    };
    for ns in model.namespaces.values() {
        if !hierarchical && !ns.explicit {
            continue;
        }
        ns_at.insert(ns.id.clone(), nodes.len());
        nodes.push(Node {
            id: ns.id.clone(),
            label: if hierarchical { ns.label.clone() } else { ns.id.clone() },
            group: true,
            shape: Shape::Rect,
            parent: -1,
            class: None,
        });
    }
    let group = |id: Option<&str>| id.and_then(|s| ns_at.get(s)).map_or(-1, |&i| i as isize);
    for n in &mut nodes {
        if hierarchical {
            n.parent = group(model.namespaces[&n.id].parent.as_deref());
        }
    }
    for cls in model.classes.values() {
        class_at.insert(cls.id.clone(), nodes.len());
        nodes.push(Node {
            id: cls.id.clone(),
            label: cls.label.clone(),
            group: false,
            shape: Shape::Class,
            parent: group(ancestor(cls.parent.as_deref()).as_deref()),
            class: Some(cls),
        });
    }
    for note in model.notes.values() {
        let parent = group(ancestor(note.parent.as_deref()).as_deref());
        if let Some(&target) = note.class.as_ref().and_then(|s| class_at.get(s)) {
            edges.push(Edge {
                id: format!("edgeNote{}", note.index),
                source: nodes.len() as isize,
                target: target as isize,
                label: None,
                tail: "none",
                head: "none",
                tail_label: String::new(),
                head_label: String::new(),
                pattern: "dotted",
            });
        }
        nodes.push(Node { id: note.id.clone(), label: note.text.clone(), group: false, shape: Shape::Note, parent, class: None });
    }
    for item in &model.interfaces {
        interface_at.insert(item.id.clone(), nodes.len());
        nodes.push(Node {
            id: item.id.clone(),
            label: item.label.clone(),
            group: false,
            shape: Shape::Interface,
            parent: class_at.get(&item.class_id).map_or(-1, |&i| nodes[i].parent),
            class: None,
        });
    }
    let marker = |i: Option<usize>| {
        i.and_then(|i| ["aggregation", "extension", "composition", "dependency", "lollipop"].get(i).copied()).unwrap_or("none")
    };
    for (i, r) in model.relations.iter().enumerate() {
        let start_interface = r.end1 == Some(4) && r.end2.is_none();
        let end_interface = !start_interface && r.end2 == Some(4) && r.end1.is_none();
        let source = if start_interface { &interface_at } else { &class_at }.get(&r.id1).map_or(-1, |&i| i as isize);
        let target = if end_interface { &interface_at } else { &class_at }.get(&r.id2).map_or(-1, |&i| i as isize);
        edges.push(Edge {
            id: format!("id_{}_{}_{}", r.id1, r.id2, i + 1),
            source,
            target,
            label: r.title.clone(),
            tail: marker(r.end1),
            head: marker(r.end2),
            tail_label: if r.title1 == "none" { String::new() } else { r.title1.clone() },
            head_label: if r.title2 == "none" { String::new() } else { r.title2.clone() },
            pattern: if r.dotted { "dashed" } else { "solid" },
        });
    }
    (nodes, edges)
}
