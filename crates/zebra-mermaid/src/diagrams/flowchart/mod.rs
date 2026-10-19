//! Mermaid flowcharts with Pele's lexer, semantic model and compound layout.
mod graph;
mod lexer;
mod model;
mod parser;
mod render;
pub(crate) mod shapes;
mod tokens;
use super::ParseInput;
use crate::{ConfigValue, Error};
pub use model::FlowchartModel;
pub(crate) use render::render;
#[cfg(feature = "swimlane")]
pub(crate) use render::render_variant;

pub(crate) fn parse(input: &ParseInput) -> Result<FlowchartModel, Error> {
    let mut model = FlowchartModel::default();
    model.max_edges = input.limits.max_edges;
    model.inherit_dir = input.config.lookup("flowchart", "inheritDir").and_then(ConfigValue::as_bool) == Some(true);
    model.title = input.title.map(str::to_owned);
    parser::parse(input.source, &mut model)?;
    Ok(model)
}
