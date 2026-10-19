//! Wardley maps. Ported from Pele `src/diagrams/wardley/` (`model.ts`, `builder.ts`).

mod parser;
mod render;

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_wardley, LabelAst, WardleyAst};
pub(crate) use render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Flow {
    Forward,
    Backward,
    Bidirectional,
}

/// A point on the map. Coordinates are percentages: x is evolution, y is visibility.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyNode {
    pub id: String,
    pub label: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    /// `anchor`, `component` or `pipeline-component`.
    pub class_name: Option<String>,
    pub label_offset_x: Option<f64>,
    pub label_offset_y: Option<f64>,
    pub in_pipeline: bool,
    pub is_pipeline_parent: bool,
    pub inertia: Option<bool>,
    /// `build`, `buy`, `outsource` or `market`.
    pub source_strategy: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyLink {
    pub source: String,
    pub target: String,
    pub dashed: bool,
    pub label: Option<String>,
    pub flow: Option<Flow>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyTrend {
    pub node_id: String,
    pub target_x: f64,
    pub target_y: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyPipeline {
    pub node_id: String,
    pub component_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyAnnotation {
    pub number: f64,
    pub coordinates: Vec<(f64, f64)>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyNote {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

/// An accelerator or a deaccelerator.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyForce {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyAxes {
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub stages: Option<Vec<String>>,
    /// Right edge of each stage, from 0 to 1.
    pub stage_boundaries: Option<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WardleyModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub nodes: Vec<WardleyNode>,
    pub links: Vec<WardleyLink>,
    pub trends: Vec<WardleyTrend>,
    pub pipelines: Vec<WardleyPipeline>,
    pub annotations: Vec<WardleyAnnotation>,
    pub notes: Vec<WardleyNote>,
    pub accelerators: Vec<WardleyForce>,
    pub deaccelerators: Vec<WardleyForce>,
    pub annotations_box: Option<(f64, f64)>,
    pub axes: WardleyAxes,
    pub size: Option<(f64, f64)>,
}

fn fail(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Wardley)
}

/// Values up to 1 are fractions; larger ones are already percentages.
fn to_percent(value: f64, context: &str) -> Result<f64, Error> {
    let normalized = if value <= 1.0 { value * 100.0 } else { value };
    if normalized < 0.0 || normalized > 100.0 {
        return Err(fail(format!("{} must be between 0-1 (decimal) or 0-100 (percentage). Received: {}", context, js::to_string(value))));
    }
    Ok(normalized)
}

/// `{ x: evolution, y: visibility }` as percentages; evolution is checked first.
fn to_coordinates(visibility: f64, evolution: f64, context: &str) -> Result<(f64, f64), Error> {
    let x = to_percent(evolution, &format!("{} evolution", context))?;
    let y = to_percent(visibility, &format!("{} visibility", context))?;
    Ok((x, y))
}

fn flow_from_port(port: Option<&str>) -> Option<Flow> {
    match port {
        Some("+<>") => Some(Flow::Bidirectional),
        Some("+<") => Some(Flow::Backward),
        Some("+>") => Some(Flow::Forward),
        _ => None,
    }
}

fn flow_from_arrow(arrow: Option<&str>) -> (Option<Flow>, Option<String>) {
    let Some(arrow) = arrow.filter(|a| a.starts_with('+')) else {
        return (None, None);
    };
    // `arrow.indexOf("'", 2)`; the arrow starts with `+` and a quote or not, both ASCII.
    let end = arrow.get(2..).and_then(|rest| rest.find('\'')).map(|k| k + 2);
    let label = match end {
        Some(end) if arrow.as_bytes().get(1) == Some(&b'\'') => Some(arrow[2..end].to_string()),
        _ => None,
    };
    if arrow.contains("<>") {
        (Some(Flow::Bidirectional), label)
    } else if arrow.contains('<') {
        (Some(Flow::Backward), label)
    } else if arrow.contains('>') {
        (Some(Flow::Forward), label)
    } else {
        (None, label)
    }
}

fn offsets(label: Option<&LabelAst>) -> (Option<f64>, Option<f64>) {
    match label {
        Some(l) => (Some(if l.neg_x { -1.0 } else { 1.0 } * l.offset_x), Some(if l.neg_y { -1.0 } else { 1.0 } * l.offset_y)),
        None => (None, None),
    }
}

/// What a node added again takes from the new one. Pele spreads the new node over the old, so
/// a key the new one sets replaces the old value even when it is undefined; class and label
/// offsets keep the old value unless the new one gives one.
struct NewNode {
    id: String,
    label: String,
    x: f64,
    y: f64,
    class_name: &'static str,
    label_offset_x: Option<f64>,
    label_offset_y: Option<f64>,
    /// Set for components and pipeline components, which name these keys; anchors do not.
    traits: Option<(Option<bool>, Option<String>)>,
}

/// Pele's `WardleyBuilder`.
#[derive(Default)]
struct Builder {
    nodes: IndexMap<String, WardleyNode>,
    labels: Option<HashMap<String, String>>,
    links: Vec<WardleyLink>,
    trends: IndexMap<String, WardleyTrend>,
    pipelines: IndexMap<String, WardleyPipeline>,
    annotations: Vec<WardleyAnnotation>,
    notes: Vec<WardleyNote>,
    accelerators: Vec<WardleyForce>,
    deaccelerators: Vec<WardleyForce>,
    annotations_box: Option<(f64, f64)>,
    axes: WardleyAxes,
    size: Option<(f64, f64)>,
}

impl Builder {
    fn add_node(&mut self, node: NewNode) {
        let existing = self.nodes.get(&node.id).cloned();
        let mut merged = existing.clone().unwrap_or(WardleyNode {
            id: node.id.clone(),
            label: node.label.clone(),
            x: None,
            y: None,
            class_name: None,
            label_offset_x: None,
            label_offset_y: None,
            in_pipeline: false,
            is_pipeline_parent: false,
            inertia: None,
            source_strategy: None,
        });
        merged.id = node.id.clone();
        merged.label = node.label;
        merged.x = Some(node.x);
        merged.y = Some(node.y);
        merged.class_name = Some(node.class_name.to_string());
        if node.label_offset_x.is_some() {
            merged.label_offset_x = node.label_offset_x;
        }
        if node.label_offset_y.is_some() {
            merged.label_offset_y = node.label_offset_y;
        }
        if let Some((inertia, strategy)) = node.traits {
            merged.inertia = inertia;
            merged.source_strategy = strategy;
        }
        self.nodes.insert(node.id, merged);
        self.labels = None;
    }

    fn start_pipeline(&mut self, node_id: &str) {
        self.pipelines.insert(node_id.to_string(), WardleyPipeline { node_id: node_id.to_string(), component_ids: Vec::new() });
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.is_pipeline_parent = true;
        }
    }

    fn add_pipeline_component(&mut self, pipeline_node_id: &str, component_id: &str) {
        if let Some(p) = self.pipelines.get_mut(pipeline_node_id) {
            p.component_ids.push(component_id.to_string());
        }
        if let Some(node) = self.nodes.get_mut(component_id) {
            node.in_pipeline = true;
        }
    }

    /// A name is a node id, or else the label of the first node that has it. Pipeline
    /// components have ids like "Parent_Child" and are linked by label.
    fn resolve_node_id(&mut self, name: &str) -> String {
        if self.nodes.contains_key(name) {
            return name.to_string();
        }
        let nodes = &self.nodes;
        let labels = self.labels.get_or_insert_with(|| {
            let mut labels = HashMap::new();
            for (id, node) in nodes {
                labels.entry(node.label.clone()).or_insert_with(|| id.clone());
            }
            labels
        });
        labels.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    fn build(self) -> Result<WardleyModel, Error> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.into_values() {
            if node.x.is_none() || node.y.is_none() {
                return Err(fail(format!("Node \"{}\" is missing coordinates", node.label)));
            }
            nodes.push(node);
        }
        Ok(WardleyModel {
            title: None,
            acc_title: None,
            acc_descr: None,
            nodes,
            links: self.links,
            trends: self.trends.into_values().collect(),
            pipelines: self.pipelines.into_values().collect(),
            annotations: self.annotations,
            notes: self.notes,
            accelerators: self.accelerators,
            deaccelerators: self.deaccelerators,
            annotations_box: self.annotations_box,
            axes: self.axes,
            size: self.size,
        })
    }
}

/// Mermaid's populateDb: fills the builder from the syntax tree, in Mermaid's order.
fn populate(ast: &WardleyAst, db: &mut Builder) -> Result<(), Error> {
    if let Some(size) = ast.size {
        db.size = Some(size);
    }

    if let Some(stages) = &ast.evolution {
        db.axes.stages = Some(
            stages
                .iter()
                .map(|stage| match stage.second_name.as_deref().filter(|s| !s.is_empty()) {
                    Some(second) => format!("{} / {}", js::trim(&stage.name), js::trim(second)),
                    None => js::trim(&stage.name).to_string(),
                })
                .collect(),
        );
        db.axes.stage_boundaries = Some(stages.iter().filter_map(|stage| stage.boundary).collect());
    }

    for anchor in &ast.anchors {
        let (x, y) = to_coordinates(anchor.visibility, anchor.evolution, &format!("Anchor \"{}\"", anchor.name))?;
        db.add_node(NewNode {
            id: anchor.name.clone(),
            label: anchor.name.clone(),
            x,
            y,
            class_name: "anchor",
            label_offset_x: None,
            label_offset_y: None,
            traits: None,
        });
    }

    for component in &ast.components {
        let (x, y) = to_coordinates(component.visibility, component.evolution, &format!("Component \"{}\"", component.name))?;
        let (label_offset_x, label_offset_y) = offsets(component.label.as_ref());
        db.add_node(NewNode {
            id: component.name.clone(),
            label: component.name.clone(),
            x,
            y,
            class_name: "component",
            label_offset_x,
            label_offset_y,
            traits: Some((Some(component.inertia), component.strategy.clone())),
        });
    }

    for note in &ast.notes {
        let (x, y) = to_coordinates(note.visibility, note.evolution, &format!("Note \"{}\"", note.text))?;
        db.notes.push(WardleyNote { text: note.text.clone(), x, y });
    }

    for pipeline in &ast.pipelines {
        let y = match db.nodes.get(&pipeline.parent).and_then(|p| p.y) {
            Some(y) => y,
            None => {
                return Err(fail(format!("Pipeline \"{}\" must reference an existing component with coordinates.", pipeline.parent)));
            }
        };
        db.start_pipeline(&pipeline.parent);
        for component in &pipeline.components {
            let id = format!("{}_{}", pipeline.parent, component.name);
            let (label_offset_x, label_offset_y) = offsets(component.label.as_ref());
            let x = to_percent(component.evolution, &format!("Pipeline component \"{}\" evolution", component.name))?;
            db.add_node(NewNode {
                id: id.clone(),
                label: component.name.clone(),
                x,
                y,
                class_name: "pipeline-component",
                label_offset_x,
                label_offset_y,
                traits: Some((None, None)),
            });
            db.add_pipeline_component(&pipeline.parent, &id);
        }
    }

    for link in &ast.links {
        let arrow = link.arrow.as_deref();
        let dashed = arrow.is_some_and(|a| !a.is_empty() && (a.contains("-.->") || a.contains(".-.")));
        let (arrow_flow, arrow_label) = flow_from_arrow(arrow);
        let flow = flow_from_port(link.from_port.as_deref()).or(flow_from_port(link.to_port.as_deref())).or(arrow_flow);
        let source = db.resolve_node_id(&link.from);
        let target = db.resolve_node_id(&link.to);
        db.links.push(WardleyLink { source, target, dashed, label: arrow_label.or_else(|| link.link_label.clone()), flow });
    }

    for evolve in &ast.evolves {
        if let Some(y) = db.nodes.get(&evolve.component).and_then(|n| n.y) {
            let target_x = to_percent(evolve.target, &format!("Evolve target for \"{}\"", evolve.component))?;
            db.trends.insert(evolve.component.clone(), WardleyTrend { node_id: evolve.component.clone(), target_x, target_y: y });
        }
    }

    if let Some(&(x, y)) = ast.annotations.first() {
        let at = to_coordinates(x, y, "Annotations box")?;
        db.annotations_box = Some(at);
    }

    for annotation in &ast.annotation {
        let at = to_coordinates(annotation.x, annotation.y, &format!("Annotation {}", js::to_string(annotation.number)))?;
        db.annotations.push(WardleyAnnotation { number: annotation.number, coordinates: vec![at], text: Some(annotation.text.clone()) });
    }

    for accelerator in &ast.accelerators {
        let (x, y) = to_coordinates(accelerator.x, accelerator.y, &format!("Accelerator \"{}\"", accelerator.name))?;
        db.accelerators.push(WardleyForce { name: accelerator.name.clone(), x, y });
    }

    for deaccelerator in &ast.deaccelerators {
        let (x, y) = to_coordinates(deaccelerator.x, deaccelerator.y, &format!("Deaccelerator \"{}\"", deaccelerator.name))?;
        db.deaccelerators.push(WardleyForce { name: deaccelerator.name.clone(), x, y });
    }
    Ok(())
}

pub fn build_wardley(ast: WardleyAst, title: Option<&str>) -> Result<WardleyModel, Error> {
    let mut db = Builder::default();
    populate(&ast, &mut db)?;
    let mut model = db.build()?;
    model.title = ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string));
    model.acc_title = ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string());
    model.acc_descr = ast.acc_descr.filter(|d| !d.is_empty()).map(|d| crate::diagrams::squeeze_line_starts(&d));
    Ok(model)
}

pub(crate) fn parse(input: &ParseInput) -> Result<WardleyModel, Error> {
    build_wardley(parse_wardley(input.source)?, input.title)
}
