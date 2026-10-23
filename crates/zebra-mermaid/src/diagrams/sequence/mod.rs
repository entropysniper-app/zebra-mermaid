//! Sequence diagrams. Ported from Pele `src/diagrams/sequence/`.

mod actors;
mod colors;
mod db;
mod lexer;
mod parser;
mod render;
mod tokens;
#[cfg(test)]
#[path = "../../../tests/upstream/sequence_parser.rs"]
mod upstream_parser;
#[cfg(test)]
#[path = "../../../tests/upstream/sequence_stress_generated.rs"]
mod upstream_stress;

use indexmap::IndexMap;

use crate::config::ConfigValue;
use crate::diagrams::ParseInput;
use crate::error::Error;

pub(crate) use render::render;

/// The message kinds, numbered as in Mermaid (`LINETYPE`).
pub mod line_type {
    pub const SOLID: i32 = 0;
    pub const DOTTED: i32 = 1;
    pub const NOTE: i32 = 2;
    pub const SOLID_CROSS: i32 = 3;
    pub const DOTTED_CROSS: i32 = 4;
    pub const SOLID_OPEN: i32 = 5;
    pub const DOTTED_OPEN: i32 = 6;
    pub const LOOP_START: i32 = 10;
    pub const LOOP_END: i32 = 11;
    pub const ALT_START: i32 = 12;
    pub const ALT_ELSE: i32 = 13;
    pub const ALT_END: i32 = 14;
    pub const OPT_START: i32 = 15;
    pub const OPT_END: i32 = 16;
    pub const ACTIVE_START: i32 = 17;
    pub const ACTIVE_END: i32 = 18;
    pub const PAR_START: i32 = 19;
    pub const PAR_AND: i32 = 20;
    pub const PAR_END: i32 = 21;
    pub const RECT_START: i32 = 22;
    pub const RECT_END: i32 = 23;
    pub const SOLID_POINT: i32 = 24;
    pub const DOTTED_POINT: i32 = 25;
    pub const AUTONUMBER: i32 = 26;
    pub const CRITICAL_START: i32 = 27;
    pub const CRITICAL_OPTION: i32 = 28;
    pub const CRITICAL_END: i32 = 29;
    pub const BREAK_START: i32 = 30;
    pub const BREAK_END: i32 = 31;
    pub const PAR_OVER_START: i32 = 32;
    pub const BIDIRECTIONAL_SOLID: i32 = 33;
    pub const BIDIRECTIONAL_DOTTED: i32 = 34;
    pub const SOLID_TOP: i32 = 41;
    pub const SOLID_BOTTOM: i32 = 42;
    pub const STICK_TOP: i32 = 43;
    pub const STICK_BOTTOM: i32 = 44;
    pub const SOLID_ARROW_TOP_REVERSE: i32 = 45;
    pub const SOLID_ARROW_BOTTOM_REVERSE: i32 = 46;
    pub const STICK_ARROW_TOP_REVERSE: i32 = 47;
    pub const STICK_ARROW_BOTTOM_REVERSE: i32 = 48;
    pub const SOLID_TOP_DOTTED: i32 = 51;
    pub const SOLID_BOTTOM_DOTTED: i32 = 52;
    pub const STICK_TOP_DOTTED: i32 = 53;
    pub const STICK_BOTTOM_DOTTED: i32 = 54;
    pub const SOLID_ARROW_TOP_REVERSE_DOTTED: i32 = 55;
    pub const SOLID_ARROW_BOTTOM_REVERSE_DOTTED: i32 = 56;
    pub const STICK_ARROW_TOP_REVERSE_DOTTED: i32 = 57;
    pub const STICK_ARROW_BOTTOM_REVERSE_DOTTED: i32 = 58;
    pub const CENTRAL_CONNECTION: i32 = 59;
    pub const CENTRAL_CONNECTION_REVERSE: i32 = 60;
    pub const CENTRAL_CONNECTION_DUAL: i32 = 61;
}

/// Where a note sits (`PLACEMENT`).
pub mod placement {
    pub const LEFT_OF: i32 = 0;
    pub const RIGHT_OF: i32 = 1;
    pub const OVER: i32 = 2;
}

/// A box around participants.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SeqBox {
    pub name: Option<String>,
    pub wrap: bool,
    pub fill: String,
    pub actor_keys: Vec<String>,
}

/// A participant.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SeqActor {
    /// The index in [`SequenceModel::boxes`] of the box that holds the participant.
    #[cfg_attr(feature = "serde", serde(rename = "box"))]
    pub box_index: Option<usize>,
    pub name: String,
    pub description: String,
    pub wrap: bool,
    pub prev_actor: Option<String>,
    pub next_actor: Option<String>,
    pub links: IndexMap<String, ConfigValue>,
    pub properties: IndexMap<String, ConfigValue>,
    /// The id of the page element named by a `details` statement. Pele has no page to read it
    /// from.
    pub details: Option<String>,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: String,
}

/// The text of a message, or the numbering an `autonumber` statement sets.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SeqContent {
    Text(String),
    Numbering { start: Option<f64>, step: Option<f64>, visible: bool },
}

impl SeqContent {
    pub fn text(&self) -> Option<&str> {
        match self {
            SeqContent::Text(t) => Some(t),
            SeqContent::Numbering { .. } => None,
        }
    }
}

/// One entry of the message list: a message line, a note, an activation, a block boundary or
/// a numbering change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SeqMessage {
    /// The position in the list, which Pele keeps as a string.
    pub id: usize,
    pub from: Option<String>,
    pub to: Option<String>,
    pub message: SeqContent,
    pub wrap: bool,
    /// One of [`line_type`].
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: i32,
    pub activate: Option<bool>,
    pub placement: Option<i32>,
    pub central_connection: Option<i32>,
}

/// The participant a note is on, or the two it spans.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum NoteActor {
    One(String),
    Two([String; 2]),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SeqNote {
    pub actor: NoteActor,
    pub placement: i32,
    pub message: String,
    pub wrap: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SequenceModel {
    pub actors: IndexMap<String, SeqActor>,
    /// Each created participant with the number of messages before its creation.
    pub created_actors: IndexMap<String, usize>,
    /// Each destroyed participant with the number of messages before its destruction.
    pub destroyed_actors: IndexMap<String, usize>,
    pub boxes: Vec<SeqBox>,
    pub messages: Vec<SeqMessage>,
    pub notes: Vec<SeqNote>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
}

pub(crate) fn parse(input: &ParseInput) -> Result<SequenceModel, Error> {
    let seq = input.config.section("sequence");
    let wrap = match input.config.get("wrap") {
        Some(ConfigValue::Bool(b)) => Some(*b),
        _ => None,
    };
    let sequence_wrap = matches!(seq.and_then(|s| s.get("wrap")), Some(ConfigValue::Bool(true)));
    let mut db = db::SeqDb::new(wrap, sequence_wrap);
    if let Some(title) = input.title.filter(|t| !t.is_empty()) {
        db.set_diagram_title(title.to_string());
    }
    parser::parse_sequence(input.source, &mut db)?;
    Ok(db.finish())
}
