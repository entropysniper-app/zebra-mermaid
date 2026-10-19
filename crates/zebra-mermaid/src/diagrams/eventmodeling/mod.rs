//! Event modeling diagrams. Ported from Pele `src/diagrams/eventmodeling/`.
//!
//! This type keeps `%%` comment lines in the text it parses, so that the line it reports for
//! a duplicate frame is the line in the original text; [`ParseInput::source`] is that text.

mod parser;
mod render;

use std::collections::{HashMap, HashSet};

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{acc_descr_value, acc_title_value, title_value};
use crate::diagrams::ParseInput;
use crate::error::{locate, Error};
use crate::util::js;

pub use parser::{parse_event_model, EventModelAst};
pub(crate) use render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum EntityKind {
    Ui,
    Processor,
    Command,
    Readmodel,
    Event,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Ui => "ui",
            EntityKind::Processor => "processor",
            EntityKind::Command => "command",
            EntityKind::Readmodel => "readmodel",
            EntityKind::Event => "event",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum LaneGroup {
    Trigger,
    State,
    Stream,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EmLane {
    /// Mermaid's ordering key: 0 for UI/Automation, 100 for Command/Read Model, 200 for
    /// Events, and the next free number after those for each namespace.
    pub index: f64,
    pub label: String,
    pub group: LaneGroup,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EmBox {
    /// The frame number as written, such as "01".
    pub id: String,
    pub kind: EntityKind,
    pub entity: String,
    pub name: String,
    pub namespace: Option<String>,
    pub reset: bool,
    /// Index into [`EventModelingModel::lanes`].
    pub lane: usize,
    /// Example data from the frame or from the data block it refers to.
    pub data: Option<String>,
    pub data_type: Option<String>,
}

/// A relation between two boxes, by index into [`EventModelingModel::boxes`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EmRelation {
    pub source: usize,
    pub target: usize,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EmNote {
    pub frame: String,
    pub text: String,
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EmSpecification {
    pub frame: String,
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EventModelingModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub boxes: Vec<EmBox>,
    /// Sorted from top to bottom.
    pub lanes: Vec<EmLane>,
    pub relations: Vec<EmRelation>,
    pub notes: Vec<EmNote>,
    pub specifications: Vec<EmSpecification>,
    pub entities: Vec<String>,
}

fn kind_of(entity_type: &str) -> EntityKind {
    match entity_type {
        "ui" => EntityKind::Ui,
        "pcr" | "processor" => EntityKind::Processor,
        "rmo" | "readmodel" => EntityKind::Readmodel,
        "cmd" | "command" => EntityKind::Command,
        _ => EntityKind::Event,
    }
}

/// [first index, label, label prefix for a namespace]
fn group_info(group: LaneGroup) -> (f64, &'static str, &'static str) {
    match group {
        LaneGroup::Trigger => (0.0, "UI/Automation", "UI/A: "),
        LaneGroup::State => (100.0, "Command/Read Model", "C/RM: "),
        LaneGroup::Stream => (200.0, "Events", "Stream: "),
    }
}

fn group_of(kind: EntityKind) -> LaneGroup {
    match kind {
        EntityKind::Ui | EntityKind::Processor => LaneGroup::Trigger,
        EntityKind::Event => LaneGroup::Stream,
        _ => LaneGroup::State,
    }
}

/// `line.length - line.trimStart().length`: the leading white space, one code unit per
/// character as all JavaScript white space lies in the Basic Multilingual Plane.
fn indent_of(line: &str) -> usize {
    line.chars().count() - js::trim_start(line).chars().count()
}

/// The text between the outer braces, without the indentation its lines share.
fn block_text(raw: &str) -> String {
    let open = raw.find('{');
    let close = raw.rfind('}');
    let inner = match (open, close) {
        (Some(o), Some(c)) if c > o => &raw[o + 1..c],
        // `raw.slice(1, -1)`: the first and last characters of an inline value are its quotes.
        (None, _) if js::len16(raw) >= 2 => js::slice16(raw, 1, js::len16(raw) - 1),
        _ => "",
    };
    let cleaned = inner.replace('\r', "");
    let mut lines: Vec<&str> = cleaned.split('\n').collect();
    while lines.first().is_some_and(|l| js::trim(l).is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| js::trim(l).is_empty()) {
        lines.pop();
    }
    let mut indent = usize::MAX;
    for line in &lines {
        if js::trim(line).is_empty() {
            continue;
        }
        indent = indent.min(indent_of(line));
    }
    lines
        .iter()
        .map(|line| {
            let skip = line.char_indices().nth(indent).map_or(line.len(), |(i, _)| i);
            js::trim_end(&line[skip..])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_event_model(ast: EventModelAst, source: &str, title: Option<&str>) -> Result<EventModelingModel, Error> {
    let mut ids: HashSet<&str> = HashSet::new();
    for frame in &ast.frames {
        if !ids.insert(frame.name.as_str()) {
            let line = locate(source, frame.offset).line;
            let mut e = Error::semantic(format!("Duplicate event modeling frame ID \"{}\" on line {}", frame.name, line))
                .with_type(DiagramType::Eventmodeling);
            e.line = line;
            return Err(e);
        }
    }

    let mut data: HashMap<&str, &str> = HashMap::new();
    for entity in &ast.data_entities {
        data.entry(entity.name.as_str()).or_insert(entity.data_block_value.as_str());
    }

    // One lane per group, and one more per namespace within a group. Lanes are kept in the
    // order they are made; `lane_of` gives each key's place in that list.
    let mut lanes: Vec<EmLane> = Vec::new();
    let mut lane_of: HashMap<(LaneGroup, Option<String>), usize> = HashMap::new();
    let mut next: HashMap<LaneGroup, f64> = HashMap::new();
    let mut lane = |kind: EntityKind, namespace: Option<&str>| -> usize {
        let group = group_of(kind);
        let key = (group, namespace.map(str::to_string));
        if let Some(&found) = lane_of.get(&key) {
            return found;
        }
        let (first, label, prefix) = group_info(group);
        let mut index = first;
        if namespace.is_some() {
            index = next.get(&group).copied().unwrap_or(first + 1.0);
            next.insert(group, index + 1.0);
        }
        lanes.push(EmLane {
            index,
            label: match namespace {
                None => label.to_string(),
                Some(ns) => format!("{}{}", prefix, ns),
            },
            group,
            namespace: namespace.map(str::to_string),
        });
        lane_of.insert(key, lanes.len() - 1);
        lanes.len() - 1
    };

    let mut boxes: Vec<EmBox> = Vec::new();
    let mut relations: Vec<EmRelation> = Vec::new();
    for (index, frame) in ast.frames.iter().enumerate() {
        // Only a two-part name has a namespace.
        let parts: Vec<&str> = frame.entity_identifier.split('.').collect();
        let namespace = if parts.len() == 2 { Some(parts[0]) } else { None };
        let kind = kind_of(&frame.model_entity_type);
        let mut text = frame.data_inline_value.as_deref().map(block_text);
        if let Some(reference) = &frame.data_reference {
            if let Some(block) = data.get(reference.as_str()) {
                text = Some(block_text(block));
            }
        }
        let b = EmBox {
            id: frame.name.clone(),
            kind,
            entity: frame.entity_identifier.clone(),
            name: if parts.len() == 2 { parts[1].to_string() } else { frame.entity_identifier.clone() },
            namespace: namespace.map(str::to_string),
            reset: frame.reset,
            lane: lane(kind, namespace),
            data: text,
            data_type: frame.data_type.clone(),
        };
        boxes.push(b);
        let target = boxes.len() - 1;

        if !frame.source_frames.is_empty() {
            // Sources connect in frame order, and only frames that already have a box.
            let wanted: HashSet<&str> = frame.source_frames.iter().map(String::as_str).collect();
            for (source, candidate) in boxes.iter().enumerate() {
                if wanted.contains(candidate.id.as_str()) {
                    relations.push(EmRelation { source, target });
                }
            }
        } else if index > 0 && !boxes[target].reset {
            // Otherwise the nearest earlier frame in another lane feeds this one.
            for i in (0..index).rev() {
                if boxes[i].lane != boxes[target].lane {
                    relations.push(EmRelation { source: i, target });
                    break;
                }
            }
        }
    }

    // `[...laneOf.values()].sort((a, b) => a.index - b.index)`, a stable sort; boxes then
    // refer to the sorted list.
    let mut sorted: Vec<usize> = (0..lanes.len()).collect();
    sorted.sort_by(|&a, &b| lanes[a].index.partial_cmp(&lanes[b].index).unwrap_or(std::cmp::Ordering::Equal));
    let mut place = vec![0usize; lanes.len()];
    for (at, &k) in sorted.iter().enumerate() {
        place[k] = at;
    }
    for b in &mut boxes {
        b.lane = place[b.lane];
    }
    let lanes: Vec<EmLane> = sorted.iter().map(|&k| lanes[k].clone()).collect();

    let title = match ast.title.as_deref().filter(|t| !t.is_empty()).map(title_value) {
        Some(t) if !t.is_empty() => Some(t),
        _ => title.map(str::to_string),
    };
    Ok(EventModelingModel {
        title,
        acc_title: ast.acc_title.as_deref().filter(|t| !t.is_empty()).map(|t| js::trim_start(&acc_title_value(t)).to_string()),
        acc_descr: ast.acc_descr.as_deref().filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&acc_descr_value(t))),
        boxes,
        lanes,
        relations,
        notes: ast
            .note_entities
            .iter()
            .map(|note| EmNote {
                frame: note.source_frame.clone(),
                text: block_text(&note.data_block_value),
                data_type: note.data_type.clone(),
            })
            .collect(),
        specifications: ast
            .gwt_entities
            .iter()
            .map(|gwt| EmSpecification {
                frame: gwt.source_frame.clone(),
                given: gwt.given.clone(),
                when: gwt.when.clone(),
                then: gwt.then.clone(),
            })
            .collect(),
        entities: ast.model_entities,
    })
}

pub(crate) fn parse(input: &ParseInput) -> Result<EventModelingModel, Error> {
    build_event_model(parse_event_model(input.source)?, input.source, input.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_texts() {
        assert_eq!(block_text("{\n    a\n      b\n}\n"), "a\n  b");
        assert_eq!(block_text("\"x{y}z\""), "y");
        assert_eq!(block_text("'plain'"), "plain");
        assert_eq!(block_text("\"a{b\""), "");
        assert_eq!(block_text("{\n \n}"), "");
    }

    #[test]
    fn lanes_and_relations() {
        let src = "eventmodeling\ntf 01 ui A\ntf 02 cmd Shop.B\ntf 03 evt C\ntf 04 cmd Shop.D ->> 01 ->> 03\n";
        let m = build_event_model(parse_event_model(src).unwrap(), src, None).unwrap();
        let labels: Vec<&str> = m.lanes.iter().map(|l| l.label.as_str()).collect();
        assert_eq!(labels, ["UI/Automation", "C/RM: Shop", "Events"]);
        let pairs: Vec<(usize, usize)> = m.relations.iter().map(|r| (r.source, r.target)).collect();
        assert_eq!(pairs, [(0, 1), (1, 2), (0, 3), (2, 3)]);
        let src = "eventmodeling\n%% c\ntf 01 ui A\ntf 01 ui B\n";
        let e = build_event_model(parse_event_model(src).unwrap(), src, None).unwrap_err();
        assert_eq!(e.message, "Duplicate event modeling frame ID \"01\" on line 4");
        assert_eq!(e.line, 4);
    }
}
