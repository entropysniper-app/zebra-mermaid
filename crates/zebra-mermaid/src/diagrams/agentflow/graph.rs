//! Collapse, containment, class style and semantic color-slot projection.
use super::{db::AgentflowModel, shapes};
use crate::{util::js, ConfigMap, ConfigValue};
use indexmap::{IndexMap, IndexSet};
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Node {
    pub id: String,
    pub label: Option<String>,
    pub label_type: String,
    pub shape: String,
    pub group: bool,
    pub parent: Option<String>,
    pub styles: Vec<String>,
    pub compiled: Vec<String>,
    pub classes: String,
    pub dir: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub tooltip: Option<String>,
    pub color_index: Option<usize>,
    pub kind: Option<String>,
    pub metadata: Option<ConfigMap>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Edge {
    pub id: String,
    pub user_id: bool,
    pub start: String,
    pub end: String,
    pub kind: String,
    pub label: String,
    pub label_type: String,
    pub thickness: Option<String>,
    pub pattern: Option<String>,
    pub minlen: Option<f64>,
    pub tail: String,
    pub head: String,
    pub compiled: Vec<String>,
    pub styles: Vec<String>,
    pub classes: String,
    pub animate: Option<ConfigValue>,
    pub animation: Option<ConfigValue>,
    pub curve: Option<ConfigValue>,
    pub metadata: Option<ConfigMap>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
pub fn build(db: &mut AgentflowModel, palette: usize, default_curve: Option<&str>) -> Graph {
    let subgraphs = db.subgraphs.clone();
    let mut hidden = IndexSet::new();
    let mut owner = IndexMap::new();
    let mut walked = IndexSet::new();
    for sg in &subgraphs {
        if sg.metadata.as_ref().and_then(|m| m.get("view")).and_then(ConfigValue::as_str) != Some("collapsed") || hidden.contains(&sg.id) {
            continue;
        }
        let mut pending = vec![sg.id.clone()];
        while let Some(id) = pending.pop() {
            let Some(group) = db.subgraph(&id) else {
                continue;
            };
            for child in &group.nodes {
                if child == &sg.id || hidden.contains(child) {
                    continue;
                }
                hidden.insert(child.clone());
                owner.insert(child.clone(), sg.id.clone());
                if !walked.contains(child) {
                    pending.push(child.clone());
                }
            }
        }
        walked.insert(sg.id.clone());
    }
    let mut collapsed = IndexMap::<String, String>::new();
    for id in &hidden {
        let mut path = vec![id.clone()];
        let mut top = owner[id].clone();
        loop {
            let known = collapsed.get(&top);
            let up = known.or_else(|| owner.get(&top));
            let Some(up) = up else {
                break;
            };
            if known.is_none() {
                path.push(top.clone());
            }
            top = up.clone();
            if known.is_some() {
                break;
            }
        }
        for p in path {
            collapsed.insert(p, top.clone());
        }
    }
    let position: IndexMap<_, _> = subgraphs.iter().enumerate().map(|(i, s)| (s.id.clone(), i)).collect();
    let mut parents = IndexMap::<String, String>::new();
    for i in (0..subgraphs.len()).rev() {
        let sg = &subgraphs[i];
        if hidden.contains(&sg.id) {
            continue;
        }
        for id in &sg.nodes {
            let mut closes = false;
            if position.get(id).is_some_and(|&at| at >= i) {
                let mut current = Some(sg.id.as_str());
                for _ in 0..=subgraphs.len() {
                    let Some(at) = current else {
                        break;
                    };
                    if at == id {
                        closes = true;
                        break;
                    }
                    current = parents.get(at).map(String::as_str);
                }
            }
            if closes {
                db.emit("CONTAINMENT_VIOLATION","warning",format!("Container \"{}\" cannot contain \"{id}\" because \"{id}\" already contains it. The nesting that would close the loop is dropped.",sg.id),Some(id),None);
                continue;
            }
            parents.insert(id.clone(), sg.id.clone());
        }
    }
    let compiled = |names: &[String]| {
        let mut out = vec![];
        for name in names {
            if let Some(def) = db.classes.get(name) {
                out.extend(def.styles.iter().cloned());
                out = out.into_iter().map(|s| js::trim(&s).to_string()).collect();
                out.extend(def.text_styles.iter().cloned());
                out = out.into_iter().map(|s| js::trim(&s).to_string()).collect();
            }
        }
        out
    };
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut by_id = IndexMap::new();
    for sg in subgraphs.iter().rev() {
        if hidden.contains(&sg.id) {
            continue;
        }
        let is_collapsed = sg.metadata.as_ref().and_then(|m| m.get("view")).and_then(ConfigValue::as_str) == Some("collapsed");
        let metadata = if is_collapsed {
            let mut m = sg.metadata.clone().unwrap_or_default();
            m.insert("containerType", ConfigValue::String("flow".into()));
            Some(m)
        } else {
            sg.metadata.clone()
        };
        by_id.insert(sg.id.clone(), nodes.len());
        nodes.push(Node {
            id: sg.id.clone(),
            label: Some(sg.title.clone()),
            label_type: sg.label_type.clone(),
            shape: if is_collapsed { "collapsedGroup" } else { "flowGroup" }.into(),
            group: !is_collapsed,
            parent: parents.get(&sg.id).cloned(),
            compiled: compiled(&sg.classes),
            classes: sg.classes.join(" "),
            dir: sg.dir.clone(),
            metadata,
            ..Default::default()
        });
    }
    for vertex in db.nodes.values() {
        if hidden.contains(&vertex.id) {
            continue;
        }
        if let Some(&i) = by_id.get(&vertex.id) {
            let n = &mut nodes[i];
            n.styles = vertex.styles.clone();
            n.compiled = compiled(&vertex.classes);
            n.classes = vertex.classes.join(" ");
            continue;
        }
        let kind = vertex.shape.as_deref().map(shapes::alias);
        let mut names = vec!["default".into(), "node".into()];
        names.extend(vertex.classes.iter().cloned());
        by_id.insert(vertex.id.clone(), nodes.len());
        nodes.push(Node {
            id: vertex.id.clone(),
            label: vertex.text.clone(),
            label_type: vertex.label_type.clone(),
            shape: if kind.is_none() || kind == Some("square") { "squareRect" } else { kind.unwrap() }.into(),
            group: false,
            parent: parents.get(&vertex.id).cloned(),
            styles: vertex.styles.clone(),
            compiled: compiled(&names),
            classes: format!("default {}", vertex.classes.join(" ")),
            dir: vertex.dir.clone(),
            link: vertex.link.clone(),
            link_target: vertex.link_target.clone(),
            tooltip: db.tooltips.get(&vertex.id).cloned(),
            metadata: vertex.metadata.clone(),
            ..Default::default()
        });
    }
    for (index, raw) in db.edges.iter().enumerate() {
        let start = collapsed.get(&raw.start).unwrap_or(&raw.start).clone();
        let end = collapsed.get(&raw.end).unwrap_or(&raw.end).clone();
        if start == end && raw.start != raw.end {
            continue;
        }
        let mut styles = db.default_edge_style.clone();
        styles.extend(raw.styles.iter().cloned());
        let arrow = match raw.kind.as_deref() {
            Some("arrow_open") => "none",
            Some("arrow_cross") => "arrow_cross",
            _ => "arrow_point",
        };
        edges.push(Edge {
            id: if raw.id.is_empty() { format!("L_{start}_{end}_{index}") } else { raw.id.clone() },
            user_id: raw.user_id,
            start,
            end,
            kind: raw.kind.clone().unwrap_or_else(|| "normal".into()),
            label: raw.text.clone(),
            label_type: raw.label_type.clone(),
            thickness: raw.stroke.clone(),
            pattern: raw.stroke.clone(),
            minlen: raw.length,
            tail: "none".into(),
            head: arrow.into(),
            compiled: compiled(&raw.classes),
            styles,
            classes: "edge-thickness-normal edge-pattern-solid flowchart-link".into(),
            animate: raw.animate.clone(),
            animation: raw.animation.clone(),
            curve: raw
                .interpolate
                .clone()
                .filter(ConfigValue::truthy)
                .or_else(|| db.default_interpolate.clone().filter(|s| !s.is_empty()).map(ConfigValue::String))
                .or_else(|| default_curve.map(|s| ConfigValue::String(s.into()))),
            metadata: raw.metadata.clone(),
        });
    }
    for node in &mut nodes {
        if node.group {
            continue;
        }
        if node.shape.is_empty() || matches!(node.shape.as_str(), "squareRect" | "rect") {
            node.shape = "roundedRect".into();
        }
        if shapes::removed(&node.shape) {
            db.emit(
                "SHAPE_REMOVED",
                "error",
                format!("shape \"{}\" was removed in v0.8.1, using \"roundedRect\"", node.shape),
                Some(&node.id),
                None,
            );
            node.shape = "roundedRect".into();
        } else if !shapes::allowed(&node.shape) {
            db.emit(
                "SHAPE_UNSUPPORTED",
                "warning",
                format!("shape \"{}\" is not supported, using \"roundedRect\"", node.shape),
                Some(&node.id),
                None,
            );
            node.shape = "roundedRect".into();
        }
    }
    let mut children = IndexMap::<String, Vec<String>>::new();
    for sg in &subgraphs {
        if let Some(parent) = parents.get(&sg.id) {
            children.entry(parent.clone()).or_default().push(sg.id.clone());
        }
    }
    let mut order = IndexMap::new();
    for sg in &subgraphs {
        if parents.contains_key(&sg.id) {
            continue;
        }
        let mut pending = vec![sg.id.clone()];
        while let Some(id) = pending.pop() {
            if order.contains_key(&id) {
                continue;
            }
            order.insert(id.clone(), order.len());
            if let Some(kids) = children.get(&id) {
                pending.extend(kids.iter().rev().cloned());
            }
        }
    }
    let mut fallback = order.len();
    for n in &mut nodes {
        if n.group || n.shape == "collapsedGroup" {
            let at = order.get(&n.id).copied().unwrap_or_else(|| {
                let n = fallback;
                fallback += 1;
                n
            });
            n.color_index = Some(shapes::container_slot(at, palette));
            continue;
        }
        let kind = if db.connectors.contains(&n.id) { Some("connector") } else { db.nodes.get(&n.id).map(AgentflowModel::vertex_kind) };
        if let Some(kind) = kind.filter(|k| shapes::kind_slot(k).is_some()) {
            n.kind = Some(kind.into());
        }
    }
    Graph { nodes, edges }
}
