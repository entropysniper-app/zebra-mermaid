//! Gantt syntax, calendar scheduling, axis layout and SVG rendering.
mod axis;
mod date_parse;
mod dates;
mod db;
mod js_date;
mod lexer;
mod parser;
mod render;
#[cfg(test)]
#[path = "../../../tests/upstream/gantt_dates.rs"]
mod upstream_dates;
use super::ParseInput;
use crate::{ConfigValue, Error};
pub use db::{GanttModel, GanttTask};
pub(crate) use render::render;
pub(crate) fn parse(input: &ParseInput) -> Result<GanttModel, Error> {
    let mut builder = db::Builder::default();
    builder.title = input.title.map(str::to_owned);
    if let Some(mode) = input.config.lookup("gantt", "displayMode").and_then(ConfigValue::as_str) {
        builder.display_mode = mode.into();
    }
    parser::parse_gantt(input.source, &mut builder)?;
    Ok(builder.finish())
}
