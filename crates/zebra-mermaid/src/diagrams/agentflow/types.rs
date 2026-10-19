//! Agentflow model and Jison-compatible statement locations.
use crate::{ConfigMap, ConfigValue};
use indexmap::IndexMap;
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Loc {
    pub first_line: usize,
    pub first_column: usize,
    pub last_line: usize,
    pub last_column: usize,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlowText {
    pub text: String,
    pub kind: String,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Node {
    pub id: String,
    pub text: Option<String>,
    pub label_type: String,
    pub shape: Option<String>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    pub dir: Option<String>,
    pub props: IndexMap<String, String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub metadata: Option<ConfigMap>,
    pub is_connector: bool,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Edge {
    pub start: String,
    pub end: String,
    pub kind: Option<String>,
    pub text: String,
    pub label_type: String,
    pub stroke: Option<String>,
    pub semantic: Option<String>,
    pub length: Option<f64>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    pub id: String,
    pub user_id: bool,
    pub interpolate: Option<ConfigValue>,
    pub animate: Option<ConfigValue>,
    pub animation: Option<ConfigValue>,
    pub metadata: Option<ConfigMap>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Class {
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Subgraph {
    pub id: String,
    pub nodes: Vec<String>,
    pub title: String,
    pub classes: Vec<String>,
    pub dir: Option<String>,
    pub label_type: String,
    pub metadata: Option<ConfigMap>,
}
#[derive(Clone, Debug, Default)]
pub struct Link {
    pub kind: String,
    pub stroke: String,
    pub length: Option<f64>,
    pub semantic: Option<String>,
    pub text: Option<FlowText>,
    pub id: Option<String>,
}
#[derive(Clone, Debug)]
pub enum DocItem {
    Text(String),
    Nodes(Vec<String>),
    Dir(String),
}
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Position {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_index: usize,
    pub end_index: usize,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Mapping {
    pub id: String,
    pub kind: String,
    pub position: Position,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Diagnostic {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub node_id: Option<String>,
    pub edge_id: Option<String>,
    pub position: Option<Position>,
}
