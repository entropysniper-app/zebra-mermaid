//! C4 diagrams: C4Context, C4Container, C4Component, C4Dynamic and C4Deployment. Ported from
//! Pele `src/diagrams/c4/`.

mod db;
mod layout;
mod lexer;
mod parser;
mod render;
mod route;

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

/// A `$key="value"` argument.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct C4Named {
    pub key: String,
    pub value: String,
}

/// An argument of a statement: positional, or named.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum C4Attr {
    Str(String),
    Named(C4Named),
}

/// What shapes and boundaries share.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct C4Element {
    pub alias: String,
    pub label: String,
    /// A named argument written where the alias belongs. It is applied to the field it names.
    pub alias_attr: Option<C4Named>,
    /// A named argument written where the label belongs.
    pub label_attr: Option<C4Named>,
    /// The boundary it is in, as an index into [`C4Model::boundaries`]; none for the top level.
    pub parent: Option<usize>,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind_type: Option<String>,
    pub descr: Option<String>,
    pub techn: Option<String>,
    pub sprite: Option<String>,
    pub tags: Option<String>,
    pub link: Option<String>,
    pub bg_color: Option<String>,
    pub font_color: Option<String>,
    pub border_color: Option<String>,
    pub shadowing: Option<String>,
    pub shape: Option<String>,
    pub legend_text: Option<String>,
    pub legend_sprite: Option<String>,
    /// Named arguments that set no known field.
    pub extra: Option<IndexMap<String, String>>,
}

impl C4Element {
    /// Sets the field Mermaid's element properties call `name`; false when there is none.
    pub(crate) fn set_field(&mut self, name: &str, value: String) -> bool {
        let slot = match name {
            "label" => {
                self.label = value;
                return true;
            }
            "type" => &mut self.kind_type,
            "descr" => &mut self.descr,
            "techn" => &mut self.techn,
            "sprite" => &mut self.sprite,
            "tags" => &mut self.tags,
            "link" => &mut self.link,
            "bgColor" => &mut self.bg_color,
            "fontColor" => &mut self.font_color,
            "borderColor" => &mut self.border_color,
            "shadowing" => &mut self.shadowing,
            "shape" => &mut self.shape,
            "legendText" => &mut self.legend_text,
            "legendSprite" => &mut self.legend_sprite,
            _ => return false,
        };
        *slot = Some(value);
        true
    }
}

/// A person, system, container or component.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct C4Shape {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub element: C4Element,
    /// Mermaid's typeC4Shape: person, external_system_db, container_queue and so on.
    pub kind: String,
}

/// A boundary or a deployment node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct C4Boundary {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub element: C4Element,
    /// Set on deployment nodes: node, nodeL or nodeR.
    pub node_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct C4Rel {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub rel_type: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub from_attr: Option<C4Named>,
    pub to_attr: Option<C4Named>,
    pub label_attr: Option<C4Named>,
    pub techn: Option<String>,
    pub descr: Option<String>,
    pub sprite: Option<String>,
    pub tags: Option<String>,
    pub link: Option<String>,
    pub text_color: Option<String>,
    pub line_color: Option<String>,
    /// `parseInt` of the offset given, which may be NaN.
    pub offset_x: Option<f64>,
    pub offset_y: Option<f64>,
    pub extra: Option<IndexMap<String, String>>,
}

impl C4Rel {
    /// Sets the field a relation's named argument `name` sets; false when there is none.
    pub(crate) fn set_field(&mut self, name: &str, value: String) -> bool {
        let slot = match name {
            "label" => {
                self.label = value;
                return true;
            }
            "descr" => &mut self.descr,
            "techn" => &mut self.techn,
            "sprite" => &mut self.sprite,
            "tags" => &mut self.tags,
            "link" => &mut self.link,
            "textColor" => &mut self.text_color,
            "lineColor" => &mut self.line_color,
            _ => return false,
        };
        *slot = Some(value);
        true
    }
}

/// A shape or a boundary, as an alias names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ElementRef {
    Shape(usize),
    Boundary(usize),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct C4Model {
    /// The header keyword: C4Context, C4Container, C4Component, C4Dynamic or C4Deployment.
    pub c4_type: Option<String>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub shapes: Vec<C4Shape>,
    /// The first boundary is the unnamed one that holds everything at the top level.
    pub boundaries: Vec<C4Boundary>,
    pub rels: Vec<C4Rel>,
    pub shape_in_row: f64,
    pub boundary_in_row: f64,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) shape_index: HashMap<String, usize>,
    /// The top-level boundary is not listed, so a boundary a diagram names `global` is its own
    /// box. In Mermaid it replaces the top level and nothing is drawn.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) boundary_index: HashMap<String, usize>,
}

impl C4Model {
    /// Pele's `getC4Shape`: the shape with this alias, else the boundary.
    pub(crate) fn element(&self, alias: &str) -> Option<ElementRef> {
        if let Some(&i) = self.shape_index.get(alias) {
            return Some(ElementRef::Shape(i));
        }
        self.boundary_index.get(alias).map(|&i| ElementRef::Boundary(i))
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<C4Model, Error> {
    let mut db = db::C4Db::new();
    if let Some(title) = input.title.filter(|t| !t.is_empty()) {
        db.model.title = Some(title.to_string());
    }
    parser::parse_c4(input.source, &mut db)?;
    Ok(db.model)
}
