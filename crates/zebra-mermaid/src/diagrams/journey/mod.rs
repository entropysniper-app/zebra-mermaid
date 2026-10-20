//! User journeys. Ported from Pele `src/diagrams/journey/`.

mod lexer;
mod parser;
mod render;

use std::cmp::Ordering;

use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::parse_journey;
pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct JourneyTask {
    pub task: String,
    /// As written: any number, or NaN when the score is not one.
    pub score: f64,
    pub people: Vec<String>,
    /// Name of the section in force when the task was declared, empty before the first one.
    pub section: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct JourneyModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub sections: Vec<String>,
    pub tasks: Vec<JourneyTask>,
}

/// JavaScript's default sort order: by UTF-16 code units.
fn cmp_utf16(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// Everyone named on a task, once each, in code unit order.
pub fn journey_actors(tasks: &[JourneyTask]) -> Vec<String> {
    actor_names(tasks).into_iter().map(str::to_owned).collect()
}

pub(crate) fn actor_names(tasks: &[JourneyTask]) -> Vec<&str> {
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for task in tasks {
        for name in &task.people {
            if seen.insert(name.as_str()) {
                names.push(name.as_str());
            }
        }
    }
    names.sort_by(|a, b| cmp_utf16(a, b));
    names
}

/// The calls Pele's parser makes on its database (`JourneyDb`), under the same names.
#[derive(Debug, Default)]
pub struct JourneyDb {
    pub model: JourneyModel,
    current_section: String,
}

impl JourneyDb {
    pub fn set_diagram_title(&mut self, text: &str) {
        self.model.title = Some(text.to_string());
    }

    pub fn set_acc_title(&mut self, text: &str) {
        self.model.acc_title = Some(js::trim_start(text).to_string());
    }

    pub fn set_acc_description(&mut self, text: &str) {
        self.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(text));
    }

    pub fn add_section(&mut self, text: &str) {
        self.current_section = text.to_string();
        self.model.sections.push(text.to_string());
    }

    /// `data` is the rest of the line from its first colon: `:score` or `:score:actor, actor`.
    pub fn add_task(&mut self, task: &str, data: &str) {
        let mut pieces = data[1..].split(':');
        let score = pieces.next().unwrap();
        let people = pieces.next().map_or_else(Vec::new, |people| people.split(',').map(|name| js::trim(name).to_string()).collect());
        self.model.tasks.push(JourneyTask {
            task: task.to_string(),
            score: js::number(score),
            people,
            section: self.current_section.clone(),
        });
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<JourneyModel, Error> {
    let mut db = JourneyDb::default();
    db.model.title = input.title.map(str::to_string);
    parse_journey(input.source, &mut db)?;
    Ok(db.model)
}
