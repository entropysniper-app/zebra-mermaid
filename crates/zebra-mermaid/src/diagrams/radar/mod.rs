//! Radar charts. Ported from Pele `src/diagrams/radar/`.

mod parser;
mod render;

use indexmap::IndexMap;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_radar, AxisAst, CurveAst, EntryAst, OptionAst, OptionValue, RadarAst};
pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RadarAxis {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RadarCurve {
    pub name: String,
    pub label: String,
    pub entries: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Graticule {
    Circle,
    Polygon,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RadarOptions {
    pub show_legend: bool,
    pub ticks: f64,
    pub max: Option<f64>,
    pub min: f64,
    pub graticule: Graticule,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RadarModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub axes: Vec<RadarAxis>,
    pub curves: Vec<RadarCurve>,
    pub options: RadarOptions,
}

const MAX_TICKS: f64 = 32.0;

fn invalid(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Radar)
}

pub fn build_radar(ast: RadarAst, title: Option<&str>) -> Result<RadarModel, Error> {
    let axes: Vec<RadarAxis> = ast
        .axes
        .iter()
        .map(|axis| RadarAxis { name: axis.name.clone(), label: axis.label.clone().unwrap_or_else(|| axis.name.clone()) })
        .collect();

    let mut curves = Vec::with_capacity(ast.curves.len());
    for curve in &ast.curves {
        // The parser reads at least one entry, and all entries of a curve have the same form.
        let entries: Vec<f64> = if curve.entries.first().is_none_or(|e| e.axis.is_none()) {
            curve.entries.iter().map(|entry| entry.value).collect()
        } else {
            if axes.is_empty() {
                return Err(invalid("Axes must be populated before curves for reference entries".into()));
            }
            let mut by_axis: IndexMap<&str, f64> = IndexMap::new();
            for entry in &curve.entries {
                if let Some(axis) = &entry.axis {
                    by_axis.entry(axis.as_str()).or_insert(entry.value);
                }
            }
            let mut values = Vec::with_capacity(axes.len());
            for axis in &axes {
                match by_axis.get(axis.name.as_str()) {
                    Some(&value) => values.push(value),
                    None => return Err(invalid(format!("Missing entry for axis {}", axis.label))),
                }
            }
            values
        };
        curves.push(RadarCurve { name: curve.name.clone(), label: curve.label.clone().unwrap_or_else(|| curve.name.clone()), entries });
    }

    let mut options = RadarOptions { show_legend: true, ticks: 5.0, max: None, min: 0.0, graticule: Graticule::Circle };
    for option in &ast.options {
        match (option.name.as_str(), &option.value) {
            ("showLegend", parser::OptionValue::Bool(b)) => options.show_legend = *b,
            ("graticule", parser::OptionValue::Text(t)) => {
                options.graticule = if t == "polygon" { Graticule::Polygon } else { Graticule::Circle }
            }
            ("ticks", parser::OptionValue::Number(n)) => options.ticks = *n,
            ("max", parser::OptionValue::Number(n)) => options.max = Some(*n),
            ("min", parser::OptionValue::Number(n)) => options.min = *n,
            _ => {}
        }
    }
    if options.ticks > MAX_TICKS {
        options.ticks = MAX_TICKS;
    }

    Ok(RadarModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        axes,
        curves,
        options,
    })
}

pub(crate) fn parse(input: &ParseInput) -> Result<RadarModel, Error> {
    build_radar(parse_radar(input.source)?, input.title)
}
