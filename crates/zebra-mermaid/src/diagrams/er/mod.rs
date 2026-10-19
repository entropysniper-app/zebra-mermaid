//! Entity-relationship models and ordered semantic actions from Pele.
mod lexer;
mod parser;
mod render;
mod tokens;
#[cfg(test)]
#[path = "../../../tests/upstream/er_parser.rs"]
mod upstream_parser;
use super::ParseInput;
use crate::{util::js, Error};
use indexmap::{IndexMap, IndexSet};
pub(crate) use render::render;
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Attribute {
    pub kind: String,
    pub name: String,
    pub keys: Vec<String>,
    pub comment: String,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Entity {
    pub id: String,
    pub label: String,
    pub alias: String,
    pub attributes: Vec<Attribute>,
    pub classes: String,
    pub styles: Vec<String>,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Relationship {
    pub a: String,
    pub b: String,
    pub role: String,
    pub card_a: String,
    pub card_b: String,
    pub identifying: bool,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Class {
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Group {
    pub id: String,
    pub nodes: Vec<String>,
    pub title: String,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
    pub dir: Option<String>,
}
#[derive(Debug, Clone)]
pub enum DocItem {
    Node(String),
    Dir(String),
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ErModel {
    pub direction: String,
    pub entities: IndexMap<String, Entity>,
    pub relationships: Vec<Relationship>,
    pub classes: IndexMap<String, Class>,
    pub groups: Vec<Group>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    claimed: IndexSet<String>,
}
impl Default for ErModel {
    fn default() -> Self {
        Self {
            direction: "TB".into(),
            entities: IndexMap::new(),
            relationships: vec![],
            classes: IndexMap::new(),
            groups: vec![],
            title: None,
            acc_title: None,
            acc_descr: None,
            claimed: IndexSet::new(),
        }
    }
}
impl ErModel {
    fn entity(&mut self, name: &str, alias: &str) -> &mut Entity {
        let count = self.entities.len();
        let e = self.entities.entry(name.into()).or_insert_with(|| Entity {
            id: format!("entity-{name}-{count}"),
            label: name.into(),
            alias: alias.into(),
            classes: "default".into(),
            ..Default::default()
        });
        if e.alias.is_empty() && !alias.is_empty() {
            e.alias = alias.into();
        }
        e
    }
    fn set_class(&mut self, ids: &[String], classes: &[String]) {
        for id in ids {
            if let Some(e) = self.entities.get_mut(id) {
                for c in classes {
                    e.classes.push(' ');
                    e.classes.push_str(c);
                }
            }
            if let Some(g) = self.groups.iter_mut().rev().find(|g| &g.id == id) {
                g.classes.extend(classes.iter().cloned());
            }
        }
    }
    fn styles(&mut self, ids: &[String], styles: &[String]) {
        for id in ids {
            if let Some(e) = self.entities.get_mut(id) {
                e.styles.extend(styles.iter().cloned());
            }
            if let Some(g) = self.groups.iter_mut().rev().find(|g| &g.id == id) {
                g.styles.extend(styles.iter().cloned());
            }
        }
    }
    fn class(&mut self, ids: &[String], styles: &[String]) {
        for id in ids {
            let c = self.classes.entry(id.clone()).or_default();
            for s in styles {
                if s.contains("color") {
                    c.text_styles.push(s.replacen("fill", "bgFill", 1));
                }
                c.styles.push(s.clone());
            }
        }
    }
    fn relationship(&mut self, a: &str, b: &str, role: String, card_a: String, card_b: String, identifying: bool) {
        let a = if self.groups.iter().any(|g| g.id == a) { a.into() } else { self.entity(a, "").id.clone() };
        let b = if self.groups.iter().any(|g| g.id == b) { b.into() } else { self.entity(b, "").id.clone() };
        self.relationships.push(Relationship { a, b, role, card_a, card_b, identifying });
    }
    fn group(&mut self, id: &str, title: &str, items: Vec<DocItem>) -> String {
        let id = js::trim(id).to_string();
        let mut seen = IndexSet::new();
        let mut nodes = vec![];
        let mut dir = None;
        for item in items {
            match item {
                DocItem::Node(s) => {
                    let trimmed = js::trim(&s);
                    if trimmed.is_empty() || !seen.insert(trimmed.to_owned()) {
                        continue;
                    }
                    if !self.claimed.contains(&s) {
                        nodes.push(s);
                    }
                }
                DocItem::Dir(d) => dir = Some(d),
            }
        }
        self.claimed.extend(nodes.iter().cloned());
        self.groups.push(Group { id: id.clone(), nodes, title: js::trim(title).into(), dir, ..Default::default() });
        id
    }
}
pub(crate) fn parse(input: &ParseInput) -> Result<ErModel, Error> {
    let mut model = ErModel { title: input.title.map(str::to_owned), ..Default::default() };
    parser::parse(input.source, &mut model)?;
    Ok(model)
}
