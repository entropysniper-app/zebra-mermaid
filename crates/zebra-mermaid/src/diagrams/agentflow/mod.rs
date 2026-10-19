//! Agentflow diagrams with flow scopes, connectors and semantic diagnostics.
mod db;
mod graph;
mod lexer;
#[cfg(test)]
#[path = "../../../tests/upstream/agentflow_stress_generated.rs"]
mod upstream_stress;
mod parser;
mod render;
mod shapes;
mod tokens;
mod types;
use super::ParseInput;
use crate::{ConfigValue, Error};
pub use db::AgentflowModel;
pub(crate) use render::render;
pub(crate) fn parse(input: &ParseInput) -> Result<AgentflowModel, Error> {
    let mut db = AgentflowModel::default();
    db.title = input.title.map(str::to_owned);
    db.line_offset = input.line_offset;
    db.max_edges = Some(input.limits.max_edges);
    db.inherit_dir = input.config.lookup("flowchart", "inheritDir").and_then(ConfigValue::as_bool) == Some(true);
    db.configured_direction = input.config.get("direction").and_then(ConfigValue::as_str).map(str::to_owned);
    parser::parse(input.source, &mut db, false)?;
    let curve = input.config.lookup("flowchart", "curve").and_then(ConfigValue::as_str);
    db.graph = graph::build(&mut db, 8, curve);
    Ok(db)
}
