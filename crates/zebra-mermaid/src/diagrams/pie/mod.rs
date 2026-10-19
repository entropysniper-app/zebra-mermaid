//! Pie charts. Ported from Pele `src/diagrams/pie/`.

mod parser;
mod render;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_pie, PieAst};
pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PieModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub show_data: bool,
    /// Each label with its value, in the order first given. A repeated label keeps its first value.
    pub sections: IndexMap<String, f64>,
}

pub fn build_pie(ast: PieAst, title: Option<&str>) -> Result<PieModel, Error> {
    let mut sections = IndexMap::new();
    for (label, value) in ast.sections {
        if value < 0.0 {
            return Err(Error::semantic(format!(
                "\"{}\" has invalid value: {}. Negative values are not allowed in pie charts. All slice values must be >= 0.",
                label,
                js::to_string(value)
            ))
            .with_type(crate::DiagramType::Pie));
        }
        sections.entry(label).or_insert(value);
    }
    Ok(PieModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        show_data: ast.show_data,
        sections,
    })
}

pub(crate) fn parse(input: &ParseInput) -> Result<PieModel, Error> {
    build_pie(parse_pie(input.source)?, input.title)
}
