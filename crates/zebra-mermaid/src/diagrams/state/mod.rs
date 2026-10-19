//! State documents are stored in an arena so deeply nested composites do not recurse on drop.
mod graph;
mod lexer;
mod parser;
mod render;
mod tokens;
#[cfg(test)]
#[path = "../../../tests/upstream/state_stress_generated.rs"]
mod upstream_stress;
use super::ParseInput;
use crate::{util::js, Error};
use indexmap::IndexMap;
pub(crate) use render::render;
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Description {
    Text(String),
    Lines(Vec<String>),
}
impl Description {
    fn truthy(&self) -> bool {
        match self {
            Self::Text(s) => !s.is_empty(),
            Self::Lines(_) => true,
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Text(s) => js::len16(s),
            Self::Lines(v) => v.len(),
        }
    }
    fn values(&self) -> Vec<String> {
        match self {
            Self::Text(s) => vec![s.clone()],
            Self::Lines(v) => v.clone(),
        }
    }
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateNote {
    pub position: String,
    pub text: String,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct State {
    pub id: String,
    pub kind: Option<String>,
    pub description: Option<Description>,
    pub doc: Option<usize>,
    pub note: Option<StateNote>,
    pub start: Option<bool>,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Stmt {
    Text(String),
    State(State),
    Relation { first: State, second: State, label: Option<String> },
    ClassDef(String, String),
    Style(String, String),
    ApplyClass(String, String),
    Dir(String),
    Click { id: String, url: String, tooltip: String },
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct KnownState {
    pub classes: Vec<String>,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateModel {
    pub direction: String,
    pub docs: Vec<Vec<Stmt>>,
    pub known: IndexMap<String, KnownState>,
    pub classes: IndexMap<String, Vec<String>>,
    pub links: IndexMap<String, (String, String)>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub graph: graph::Graph,
    divider_count: usize,
}
impl StateModel {
    fn divider(&mut self) -> String {
        self.divider_count += 1;
        format!("divider-id-{}", self.divider_count)
    }
    fn translate(&mut self) {
        let mut pending = vec![(0usize, "root".to_string())];
        while let Some((doc, parent)) = pending.pop() {
            let old = std::mem::take(&mut self.docs[doc]);
            let mut current = vec![];
            let mut regions = vec![];
            for stmt in old {
                if let Stmt::State(mut state) = stmt {
                    if state.kind.as_deref() == Some("divider") && state.doc.is_none() {
                        let child = self.docs.len();
                        self.docs.push(current);
                        current = vec![];
                        state.doc = Some(child);
                        regions.push(Stmt::State(state));
                        continue;
                    }
                    current.push(Stmt::State(state));
                } else {
                    current.push(stmt);
                }
            }
            self.docs[doc] = if regions.is_empty() {
                current
            } else {
                if !current.is_empty() {
                    let id = self.divider();
                    let child = self.docs.len();
                    self.docs.push(current);
                    regions.push(Stmt::State(State { id, kind: Some("divider".into()), doc: Some(child), ..Default::default() }));
                }
                regions
            };
            let mut children = vec![];
            for stmt in &mut self.docs[doc] {
                let translate = |state: &mut State, first: bool| {
                    if state.id == "[*]" {
                        state.id = format!("{parent}_{}", if first { "start" } else { "end" });
                        state.start = Some(first);
                    } else {
                        state.id = js::trim(&state.id).into();
                    }
                };
                match stmt {
                    Stmt::State(s) => {
                        translate(s, true);
                        if let Some(child) = s.doc {
                            children.push((child, s.id.clone()));
                        }
                    }
                    Stmt::Relation { first, second, .. } => {
                        translate(first, true);
                        translate(second, false);
                    }
                    _ => {}
                }
            }
            pending.extend(children.into_iter().rev());
        }
    }
    fn extract(&mut self) {
        for item in &self.docs[0] {
            match item {
                Stmt::State(s) => {
                    self.known.entry(js::trim(&s.id).into()).or_default();
                }
                Stmt::Relation { first, second, .. } => {
                    for s in [first, second] {
                        let known = self.known.entry(js::trim(&s.id).into()).or_default();
                        known.classes.extend(s.classes.iter().map(|s| js::trim(s).to_owned()));
                    }
                }
                Stmt::ClassDef(id, styles) => {
                    let out = self.classes.entry(js::trim(id).into()).or_default();
                    if !styles.is_empty() {
                        out.extend(styles.split(',').map(|s| js::trim(&s.replacen(';', "", 1)).to_owned()));
                    }
                }
                Stmt::Style(ids, value) => {
                    let styles: Vec<_> = value.split(',').map(|s| js::trim(&s.replace(';', "")).to_owned()).collect();
                    for id in js::trim(ids).split(',') {
                        let key = if self.known.contains_key(id) { id } else { js::trim(id) };
                        self.known.entry(key.into()).or_default().styles = styles.clone();
                    }
                }
                Stmt::ApplyClass(ids, value) => {
                    for id in js::trim(ids).split(',') {
                        let key = if self.known.contains_key(id) { id } else { js::trim(id) };
                        self.known.entry(key.into()).or_default().classes.push(value.clone());
                    }
                }
                Stmt::Click { id, url, tooltip } => {
                    self.links.insert(id.clone(), (url.clone(), tooltip.clone()));
                }
                _ => {}
            }
        }
        self.direction = direction(&self.docs[0]);
    }
}
pub(crate) fn direction(doc: &[Stmt]) -> String {
    doc.iter().filter_map(|s| if let Stmt::Dir(d) = s { Some(d) } else { None }).last().cloned().unwrap_or_else(|| "TB".into())
}
pub(crate) fn parse(input: &ParseInput) -> Result<StateModel, Error> {
    let mut db = StateModel { title: input.title.map(str::to_owned), docs: vec![vec![]], ..Default::default() };
    parser::parse(input.source, &mut db)?;
    db.translate();
    db.extract();
    db.graph = graph::build(&db)?;
    Ok(db)
}
