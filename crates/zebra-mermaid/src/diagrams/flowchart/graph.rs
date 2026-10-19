//! Resolve group membership, collapsed subgraphs and class styles before layout.
use super::model::*;
use crate::util::js;
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Clone, Default)]
pub struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub label_type: String,
    pub shape: String,
    pub is_group: bool,
    pub parent: Option<String>,
    pub styles: Vec<String>,
    pub compiled: Vec<String>,
    pub classes: String,
    pub dir: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub tooltip: Option<String>,
    pub icon: Option<String>,
    pub pos: Option<String>,
    pub img: Option<String>,
    pub asset_width: Option<f64>,
    pub asset_height: Option<f64>,
    pub constraint: Option<String>,
}
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: String,
    pub start: String,
    pub end: String,
    pub label: String,
    pub label_type: String,
    pub thickness: String,
    pub minlen: Option<f64>,
    pub arrow_start: String,
    pub arrow_end: String,
    pub compiled: Vec<String>,
    pub styles: Vec<String>,
    pub classes: String,
    pub curve: Option<String>,
}
pub struct FlowGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
pub fn build(model: &FlowchartModel, default_curve: Option<&str>) -> FlowGraph {
    let compiled = |names: &[String]| -> Vec<String> {
        let mut out = vec![];
        for name in names {
            if let Some(c) = model.classes.get(name) {
                out.extend(c.styles.iter().chain(&c.text_styles).map(|s| js::trim(s).to_owned()));
            }
        }
        out
    };
    let mut group_parent = IndexMap::<String, String>::new();
    for sg in &model.subgraphs {
        for child in &sg.nodes {
            if model.subgraphs.iter().any(|s| &s.id == child) {
                group_parent.insert(child.clone(), sg.id.clone());
            }
        }
    }
    let collapsed = |id: &str| {
        model
            .subgraphs
            .iter()
            .find(|s| s.id == id)
            .is_some_and(|s| s.metadata.get("view").and_then(crate::ConfigValue::as_str) == Some("collapsed"))
    };
    let mut hidden = IndexSet::new();
    let mut into = IndexMap::<String, String>::new();
    for sg in &model.subgraphs {
        let mut chain = vec![sg.id.clone()];
        let mut seen = IndexSet::new();
        seen.insert(sg.id.clone());
        let mut p = sg.id.as_str();
        while let Some(next) = group_parent.get(p) {
            if !seen.insert(next.clone()) {
                break;
            }
            chain.push(next.clone());
            p = next;
        }
        if let Some(ancestor) = chain.iter().rev().find(|s| collapsed(s)) {
            if &sg.id != ancestor {
                hidden.insert(sg.id.clone());
                into.insert(sg.id.clone(), ancestor.clone());
            }
            for child in &sg.nodes {
                if child != ancestor {
                    hidden.insert(child.clone());
                    into.insert(child.clone(), ancestor.clone());
                }
            }
        }
    }
    let mut parents = IndexMap::new();
    let mut groups = IndexSet::new();
    for sg in model.subgraphs.iter().rev() {
        if hidden.contains(&sg.id) {
            continue;
        }
        if !sg.nodes.is_empty() {
            groups.insert(sg.id.clone());
        }
        for child in &sg.nodes {
            parents.insert(child.clone(), sg.id.clone());
        }
    }
    let mut nodes: Vec<GraphNode> = vec![];
    let mut by_id = IndexMap::<String, usize>::new();
    for sg in model.subgraphs.iter().rev() {
        if hidden.contains(&sg.id) {
            continue;
        }
        let is_group = !collapsed(&sg.id);
        nodes.push(GraphNode {
            id: sg.id.clone(),
            label: Some(sg.title.clone()),
            label_type: sg.label_type.clone(),
            parent: parents.get(&sg.id).cloned(),
            compiled: compiled(&sg.classes),
            classes: sg.classes.join(" "),
            shape: if is_group { "rect" } else { "collapsedGroup" }.into(),
            dir: sg.dir.clone(),
            is_group,
            ..Default::default()
        });
        by_id.insert(sg.id.clone(), nodes.len() - 1);
    }
    for vertex in model.nodes.values() {
        if hidden.contains(&vertex.id) {
            continue;
        }
        if let Some(&i) = by_id.get(&vertex.id) {
            nodes[i].styles = vertex.styles.clone();
            nodes[i].compiled = compiled(&vertex.classes);
            nodes[i].classes = vertex.classes.join(" ");
            continue;
        }
        let is_group = groups.contains(&vertex.id);
        let shape = if is_group {
            "rect"
        } else if vertex.img.as_ref().is_some_and(|s| !s.is_empty()) {
            "imageSquare"
        } else if vertex.icon.as_ref().is_some_and(|s| !s.is_empty()) {
            match vertex.form.as_deref() {
                Some("circle") => "iconCircle",
                Some("square") => "iconSquare",
                Some("rounded") => "iconRounded",
                _ => "icon",
            }
        } else {
            match vertex.shape.as_deref() {
                None | Some("square") => "squareRect",
                Some("round") => "roundedRect",
                Some(s) => s,
            }
        };
        let mut names = vec!["default".into(), "node".into()];
        names.extend(vertex.classes.iter().cloned());
        nodes.push(GraphNode {
            id: vertex.id.clone(),
            label: vertex.text.clone(),
            label_type: vertex.label_type.clone(),
            shape: shape.into(),
            is_group,
            parent: parents.get(&vertex.id).cloned(),
            styles: vertex.styles.clone(),
            compiled: compiled(&names),
            classes: format!("default {}", vertex.classes.join(" ")),
            dir: vertex.dir.clone(),
            link: vertex.link.clone(),
            link_target: vertex.link_target.clone(),
            tooltip: model.tooltips.get(&vertex.id).cloned(),
            icon: vertex.icon.clone(),
            pos: vertex.pos.clone(),
            img: vertex.img.clone(),
            asset_width: vertex.asset_width,
            asset_height: vertex.asset_height,
            constraint: vertex.constraint.clone(),
        });
        by_id.insert(vertex.id.clone(), nodes.len() - 1);
    }
    let mut edges = vec![];
    for (i, e) in model.edges.iter().enumerate() {
        let start = into.get(&e.start).unwrap_or(&e.start);
        let end = into.get(&e.end).unwrap_or(&e.end);
        if start == end && (into.contains_key(&e.start) || into.contains_key(&e.end)) {
            continue;
        }
        let (a, b) = if e.stroke == "invisible" || e.kind == "arrow_open" {
            ("none", "none")
        } else if matches!(e.kind.as_str(), "arrow_point" | "arrow_circle" | "arrow_cross") {
            ("none", e.kind.as_str())
        } else if let Some(single) = e.kind.strip_prefix("double_").filter(|s| matches!(*s, "arrow_point" | "arrow_circle" | "arrow_cross"))
        {
            (single, single)
        } else {
            ("none", "arrow_point")
        };
        let mut styles = model.default_edge_style.clone();
        styles.extend(e.styles.iter().cloned());
        edges.push(GraphEdge {
            id: if e.id.is_empty() { format!("L_{start}_{end}_{i}") } else { e.id.clone() },
            start: start.clone(),
            end: end.clone(),
            label: e.text.clone(),
            label_type: e.label_type.clone(),
            thickness: e.stroke.clone(),
            minlen: e.length,
            arrow_start: a.into(),
            arrow_end: b.into(),
            compiled: compiled(&e.classes),
            styles,
            classes: if e.stroke == "invisible" { "" } else { "edge-thickness-normal edge-pattern-solid flowchart-link" }.into(),
            curve: e
                .interpolate
                .as_ref()
                .filter(|s| !s.is_empty())
                .or(model.default_interpolate.as_ref().filter(|s| !s.is_empty()))
                .cloned()
                .or_else(|| default_curve.map(str::to_owned)),
        });
    }
    FlowGraph { nodes, edges }
}
