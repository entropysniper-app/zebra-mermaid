//! Classes, namespaces, notes and lollipop interfaces.
mod graph;
mod lexer;
mod markers;
mod members;
mod parser;
mod render;
mod tokens;

#[cfg(test)]
#[path = "../../../tests/upstream/class_graph.rs"]
mod upstream_graph;
#[cfg(test)]
#[path = "../../../tests/upstream/class_members.rs"]
mod upstream_members;
#[cfg(test)]
#[path = "../../../tests/upstream/class_parser.rs"]
mod upstream_parser;
#[cfg(test)]
#[path = "../../../tests/upstream/class_stress_generated.rs"]
mod upstream_stress;
use crate::diagrams::common::str_entry::or_insert_with as str_entry;
use super::ParseInput;
use crate::{
    text::entities::decode_entities,
    util::{js, url::sanitize_url},
    Error,
};
use indexmap::{IndexMap, IndexSet};
pub use members::Member;
pub(crate) use render::render;
use std::sync::Arc;
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub css_classes: String,
    pub methods: Vec<Member>,
    pub members: Vec<Member>,
    pub annotations: Vec<String>,
    /// Immutable declarations shared by classes styled by the same classDef.
    /// Serialization remains an ordered array of strings, including duplicates.
    /// A sized String keeps each Arc handle thin in high-fan-out declaration lists.
    pub styles: Vec<Arc<String>>,
    pub parent: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub callback: bool,
    pub tooltip: Option<String>,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Relation {
    pub id1: String,
    pub id2: String,
    pub title1: String,
    pub title2: String,
    pub title: Option<String>,
    pub end1: Option<usize>,
    pub end2: Option<usize>,
    pub dotted: bool,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Note {
    pub id: String,
    pub class: Option<String>,
    pub text: String,
    pub index: usize,
    pub parent: Option<String>,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Interface {
    pub id: String,
    pub label: String,
    pub class_id: String,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Namespace {
    pub id: String,
    pub label: String,
    pub parent: Option<String>,
    pub explicit: bool,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassModel {
    pub direction: String,
    pub classes: IndexMap<String, ClassNode>,
    pub relations: Vec<Relation>,
    pub notes: IndexMap<String, Note>,
    pub namespaces: IndexMap<String, Namespace>,
    pub interfaces: Vec<Interface>,
    pub style_classes: IndexMap<String, Vec<String>>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    namespace_stack: Vec<String>,
    tags: IndexMap<String, IndexSet<String>>,
    #[cfg(test)]
    #[cfg_attr(feature = "serde", serde(skip))]
    calls: Option<Vec<(&'static str, Vec<String>)>>,
}
impl Default for ClassModel {
    fn default() -> Self {
        Self {
            direction: "TB".into(),
            classes: IndexMap::new(),
            relations: vec![],
            notes: IndexMap::new(),
            namespaces: IndexMap::new(),
            interfaces: vec![],
            style_classes: IndexMap::new(),
            title: None,
            acc_title: None,
            acc_descr: None,
            namespace_stack: vec![],
            tags: IndexMap::new(),
            #[cfg(test)]
            calls: None,
        }
    }
}
fn split_name(id: &str) -> (&str, &str) {
    if let Some(at) = id.find('~').filter(|&i| i > 0) {
        let end = id[at + 1..].find('~').map_or(id.len(), |i| at + 1 + i);
        (&id[..at], &id[at + 1..end])
    } else {
        (id, "")
    }
}
fn lookup_id(id: &str) -> String {
    if id.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        format!("classId-{}", split_name(id).0)
    } else {
        split_name(id).0.into()
    }
}
impl ClassModel {
    fn class(&mut self, id: &str) {
        let (name, kind) = split_name(id);
        if !self.classes.contains_key(name) {
            self.classes.insert(
                name.into(),
                ClassNode { id: name.into(), kind: kind.into(), label: name.into(), css_classes: "default".into(), ..Default::default() },
            );
            str_entry(&mut self.tags, "default", Default::default).insert(name.into());
        }
    }
    fn ensure(&mut self, id: &str) -> &mut ClassNode {
        self.class(id);
        self.classes.get_mut(split_name(id).0).unwrap()
    }
    fn member(&mut self, id: &str, raw: &str) {
        let node = self.ensure(id);
        let text = js::trim(raw);
        if text.starts_with("<<") && text.ends_with(">>") {
            node.annotations.push(text[2..text.len() - 2].into());
        } else if text.find(')').is_some_and(|i| i > 0) {
            let m = Member::new(text, true);
            if m.method {
                node.methods.push(m);
            } else {
                node.members.push(m);
            }
        } else if !text.is_empty() {
            node.members.push(Member::new(text, false));
        }
    }
    fn interface(&mut self, label: &str, class_id: &str) -> String {
        let id = format!("interface{}", self.interfaces.len());
        self.interfaces.push(Interface { id: id.clone(), label: label.into(), class_id: class_id.into() });
        id
    }
    fn relation(&mut self, mut r: Relation) {
        if r.end1 == Some(4) && r.end2.is_none() {
            self.class(&r.id2);
            r.id1 = self.interface(&r.id1, &r.id2);
        } else if r.end2 == Some(4) && r.end1.is_none() {
            self.class(&r.id1);
            r.id2 = self.interface(&r.id2, &r.id1);
        } else {
            self.class(&r.id1);
            self.class(&r.id2);
        }
        r.id1 = split_name(&r.id1).0.into();
        r.id2 = split_name(&r.id2).0.into();
        r.title1 = js::trim(&r.title1).into();
        r.title2 = js::trim(&r.title2).into();
        self.relations.push(r);
    }
    fn note(&mut self, text: String, class: Option<String>) -> String {
        let index = self.notes.len();
        let id = format!("note{index}");
        self.notes.insert(id.clone(), Note { id: id.clone(), class, text, index, parent: None });
        id
    }
    fn css_class(&mut self, ids: &str, name: &str) {
        for id in ids.split(',') {
            let key = lookup_id(id);
            if let Some(node) = self.classes.get_mut(&key) {
                node.css_classes.push(' ');
                node.css_classes.push_str(name);
                for word in name.split(' ') {
                    str_entry(&mut self.tags, word, Default::default).insert(key.clone());
                }
            }
        }
    }
    fn define_class(&mut self, ids: &[String], styles: &[String]) {
        for id in ids {
            str_entry(&mut self.style_classes, id, Default::default).extend(styles.iter().cloned());
            if let Some(tagged) = self.tags.get(id) {
                let parts: Vec<Arc<String>> = styles.iter().flat_map(|s| s.split(',').map(|part| Arc::new(part.to_owned()))).collect();
                for name in tagged {
                    if let Some(node) = self.classes.get_mut(name) {
                        node.styles.extend(parts.iter().cloned());
                    }
                }
            }
        }
    }
    fn tooltip(&mut self, ids: &str, tip: String) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            calls.push(("setTooltip", vec![ids.into(), tip.clone()]));
        }
        for id in ids.split(',') {
            if let Some(node) = self.classes.get_mut(split_name(id).0) {
                node.tooltip = Some(tip.clone());
            }
        }
    }
    fn link(&mut self, ids: &str, link: &str, target: Option<String>) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            let mut args = vec![ids.into(), link.into()];
            args.extend(target.iter().cloned());
            calls.push(("setLink", args));
        }
        let url = decode_entities(link);
        let url = js::trim(&url);
        for id in ids.split(',') {
            if let Some(node) = self.classes.get_mut(&lookup_id(id)) {
                node.link = (!url.is_empty()).then(|| sanitize_url(url));
                node.link_target = Some(target.clone().unwrap_or_else(|| "_blank".into()));
            }
        }
        self.css_class(ids, "clickable");
    }
    fn callback(&mut self, ids: &str, _name: &str, _arguments: Option<&str>) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            let mut args = vec![ids.into(), _name.into()];
            args.extend(_arguments.map(str::to_owned));
            calls.push(("setClickEvent", args));
        }
        for id in ids.split(',') {
            if let Some(node) = self.classes.get_mut(split_name(id).0) {
                node.callback = true;
            }
        }
        self.css_class(ids, "clickable");
    }
    fn namespace(&mut self, id: &str, label: Option<String>) -> String {
        let prefix = self.namespace_stack.last().cloned().filter(|s| !s.is_empty());
        let qualified = prefix.as_ref().map_or_else(|| id.into(), |p| format!("{p}.{id}"));
        self.namespace_stack.push(qualified.clone());
        if let Some(ns) = self.namespaces.get_mut(&qualified) {
            ns.explicit = true;
            if let Some(label) = label.filter(|s| !s.is_empty()) {
                ns.label = label;
            }
            return qualified;
        }
        let mut parent = prefix.clone();
        let mut from = prefix.as_ref().map_or(0, |p| p.len() + 1);
        loop {
            let dot = qualified[from..].find('.').map(|i| from + i);
            let end = dot.unwrap_or(qualified.len());
            let leaf = dot.is_none();
            let current = &qualified[..end];
            let node = str_entry(&mut self.namespaces, current, || Namespace {
                id: current.into(),
                label: if leaf {
                    label.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| qualified[from..end].into())
                } else {
                    qualified[from..end].into()
                },
                parent: parent.clone(),
                explicit: leaf,
            });
            if leaf {
                node.explicit = true;
            }
            if node.parent.is_none() {
                node.parent = parent.clone();
            }
            if leaf {
                return qualified;
            }
            parent = Some(current.into());
            from = end + 1;
        }
    }
    fn assign_namespace(&mut self, id: &str, classes: &[String], notes: &[String]) {
        for name in classes {
            if let Some(node) = self.classes.get_mut(split_name(name).0) {
                node.parent = Some(id.into());
            }
        }
        for name in notes {
            if let Some(note) = self.notes.get_mut(name) {
                note.parent = Some(id.into());
            }
        }
        self.namespace_stack.pop();
    }
}
pub(crate) fn parse(input: &ParseInput) -> Result<ClassModel, Error> {
    let mut db = ClassModel { title: input.title.map(str::to_owned), ..Default::default() };
    parser::parse(input.source, &mut db)?;
    Ok(db)
}
