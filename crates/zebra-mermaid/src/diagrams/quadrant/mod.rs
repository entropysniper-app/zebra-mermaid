//! Quadrant charts: points on a two by two grid. Ported from Pele `src/diagrams/quadrant/`.

mod db;
mod lexer;
mod parser;
mod render;
mod tokens;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct QuadrantText {
    pub text: String,
    /// Written as a markdown string (`` "`...`" ``).
    pub markdown: bool,
}

/// The look of a point, from its own styles or its class.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PointStyle {
    pub radius: Option<f64>,
    pub color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct QuadrantPoint {
    /// `Number` of the coordinate as written; NaN when it does not read as a number.
    pub x: f64,
    pub y: f64,
    pub text: QuadrantText,
    pub class_name: String,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub style: PointStyle,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct QuadrantModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// Quadrants 1 to 4: top right, top left, bottom left, bottom right.
    pub quadrants: [Option<QuadrantText>; 4],
    pub x_axis_left: Option<QuadrantText>,
    pub x_axis_right: Option<QuadrantText>,
    pub y_axis_bottom: Option<QuadrantText>,
    pub y_axis_top: Option<QuadrantText>,
    pub points: Vec<QuadrantPoint>,
    pub classes: IndexMap<String, PointStyle>,
}

pub(crate) fn parse(input: &ParseInput) -> Result<QuadrantModel, Error> {
    let mut db = db::QuadrantDb::new(input.title);
    parser::parse_quadrant(input.source, &mut db)?;
    Ok(db.model)
}
