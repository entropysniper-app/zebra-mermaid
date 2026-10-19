//! XY charts: bars and lines over a category or numeric axis. Ported from Pele
//! `src/diagrams/xychart/`.

mod db;
mod lexer;
mod parser;
mod render;
mod ticks;
mod tokens;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Orientation {
    Vertical,
    Horizontal,
}

/// The category axis: named categories, or a numeric range.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum XAxis {
    Band { title: String, categories: Vec<String> },
    Linear { title: String, min: f64, max: f64 },
}

/// The value axis. Before any value is known its range is empty (`Infinity` to `-Infinity`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LinearAxis {
    pub title: String,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum PlotType {
    Line,
    Bar,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Plot {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: PlotType,
    pub title: String,
    /// Each value with its place on the x axis: a category, or a number written as text. A
    /// category with no value has `None`.
    pub data: Vec<(String, Option<f64>)>,
    /// The labels written beside the points of a line, when any was given.
    pub point_labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct XyChartModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub orientation: Option<Orientation>,
    pub x_axis: XAxis,
    pub y_axis: LinearAxis,
    /// False while the y range comes from the data and not from a `y-axis` statement.
    pub has_set_y_axis: bool,
    pub plots: Vec<Plot>,
}

pub(crate) fn parse(input: &ParseInput) -> Result<XyChartModel, Error> {
    let mut db = db::XyChartDb::new(input.title);
    parser::parse_xychart(input.source, &mut db)?;
    Ok(db.model)
}
