//! The C4 model builder: Mermaid's c4Db semantics. Ported from Pele `src/diagrams/c4/db.ts`.

use std::collections::HashMap;

use indexmap::IndexMap;

use super::{C4Attr, C4Boundary, C4Element, C4Model, C4Named, C4Rel, C4Shape, ElementRef};
use crate::util::js;

/// Pele's `named` for an element. Mermaid assigns a named argument to whatever property it
/// names, structural ones included. Here it reaches only the known fields; any other name is
/// kept aside in `extra`.
fn named_element(el: &mut C4Element, attr: &C4Named) {
    if !el.set_field(&attr.key, attr.value.clone()) {
        el.extra.get_or_insert_with(IndexMap::new).insert(attr.key.clone(), attr.value.clone());
    }
}

/// Pele's `named` for a relation: offsets are read as integers.
fn named_rel(rel: &mut C4Rel, attr: &C4Named) {
    if attr.key == "offsetX" {
        rel.offset_x = Some(js::parse_int(&attr.value, 0));
    } else if attr.key == "offsetY" {
        rel.offset_y = Some(js::parse_int(&attr.value, 0));
    } else if !rel.set_field(&attr.key, attr.value.clone()) {
        rel.extra.get_or_insert_with(IndexMap::new).insert(attr.key.clone(), attr.value.clone());
    }
}

/// A positional argument fills the field of its slot. A named one fills the field it names,
/// whichever slot it is in.
fn put_element(el: &mut C4Element, field: &str, attr: Option<&C4Attr>) {
    match attr {
        None => {}
        Some(C4Attr::Str(s)) => {
            el.set_field(field, s.clone());
        }
        Some(C4Attr::Named(n)) => named_element(el, n),
    }
}

fn put_rel(rel: &mut C4Rel, field: &str, attr: Option<&C4Attr>) {
    match attr {
        None => {}
        Some(C4Attr::Str(s)) => {
            rel.set_field(field, s.clone());
        }
        Some(C4Attr::Named(n)) => named_rel(rel, n),
    }
}

/// Pele's `text`: a missing description or technology becomes empty, unless it is already set.
fn text_element(el: &mut C4Element, field: &str, attr: Option<&C4Attr>) {
    if attr.is_none() {
        let slot = if field == "descr" { &mut el.descr } else { &mut el.techn };
        slot.get_or_insert_with(String::new);
    } else {
        put_element(el, field, attr);
    }
}

fn text_rel(rel: &mut C4Rel, field: &str, attr: Option<&C4Attr>) {
    if attr.is_none() {
        let slot = if field == "descr" { &mut rel.descr } else { &mut rel.techn };
        slot.get_or_insert_with(String::new);
    } else {
        put_rel(rel, field, attr);
    }
}

fn label_element(el: &mut C4Element, attr: Option<&C4Attr>) {
    el.label_attr = None;
    match attr {
        Some(C4Attr::Named(n)) => {
            el.label = String::new();
            el.label_attr = Some(n.clone());
            named_element(el, n);
        }
        Some(C4Attr::Str(s)) => el.label = s.clone(),
        None => el.label = String::new(),
    }
}

fn label_rel(rel: &mut C4Rel, attr: Option<&C4Attr>) {
    rel.label_attr = None;
    match attr {
        Some(C4Attr::Named(n)) => {
            rel.label = String::new();
            rel.label_attr = Some(n.clone());
            named_rel(rel, n);
        }
        Some(C4Attr::Str(s)) => rel.label = s.clone(),
        None => rel.label = String::new(),
    }
}

/// `attrs[i]`, undefined past the end.
fn at(attrs: &[C4Attr], i: usize) -> Option<&C4Attr> {
    attrs.get(i)
}

pub(crate) struct C4Db {
    pub model: C4Model,
    rel_index: HashMap<(String, String), usize>,
    current: usize,
    stack: Vec<usize>,
}

impl C4Db {
    pub fn new() -> Self {
        let global = C4Element { alias: "global".into(), label: "global".into(), kind_type: Some("global".into()), ..Default::default() };
        C4Db {
            model: C4Model {
                c4_type: None,
                title: None,
                acc_title: None,
                acc_descr: None,
                shapes: Vec::new(),
                boundaries: vec![C4Boundary { element: global, node_type: None }],
                rels: Vec::new(),
                shape_in_row: 4.0,
                boundary_in_row: 2.0,
                shape_index: HashMap::new(),
                boundary_index: HashMap::new(),
            },
            rel_index: HashMap::new(),
            current: 0,
            stack: Vec::new(),
        }
    }

    pub fn set_c4_type(&mut self, kind: &str) {
        self.model.c4_type = Some(kind.to_string());
    }

    pub fn set_title(&mut self, title: &str) {
        self.model.title = Some(title.to_string());
    }

    pub fn set_acc_description(&mut self, descr: &str) {
        self.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(descr));
    }

    /// Pele's `element` for shapes: the shape with this alias, or a new one, labelled.
    fn shape(&mut self, alias: Option<&C4Attr>, title: Option<&C4Attr>) -> usize {
        let found = match alias {
            Some(C4Attr::Str(s)) => self.model.shape_index.get(s).copied(),
            _ => None,
        };
        let i = match found {
            Some(i) => i,
            None => {
                let i = self.model.shapes.len();
                self.model.shapes.push(C4Shape { element: C4Element::default(), kind: String::new() });
                Self::name(&mut self.model.shapes[i].element, &mut self.model.shape_index, i, alias);
                i
            }
        };
        label_element(&mut self.model.shapes[i].element, title);
        i
    }

    /// Pele's `element` for boundaries.
    fn boundary_element(&mut self, alias: Option<&C4Attr>, title: Option<&C4Attr>) -> usize {
        let found = match alias {
            Some(C4Attr::Str(s)) => self.model.boundary_index.get(s).copied(),
            _ => None,
        };
        let i = match found {
            Some(i) => i,
            None => {
                let i = self.model.boundaries.len();
                self.model.boundaries.push(C4Boundary { element: C4Element::default(), node_type: None });
                Self::name(&mut self.model.boundaries[i].element, &mut self.model.boundary_index, i, alias);
                i
            }
        };
        label_element(&mut self.model.boundaries[i].element, title);
        i
    }

    /// Gives a new element its alias. `attrs[0]` is always present: a statement has at least
    /// one argument.
    fn name(el: &mut C4Element, index: &mut HashMap<String, usize>, i: usize, alias: Option<&C4Attr>) {
        match alias {
            Some(C4Attr::Str(s)) => {
                el.alias = s.clone();
                index.insert(s.clone(), i);
            }
            Some(C4Attr::Named(n)) => {
                el.alias_attr = Some(n.clone());
                named_element(el, n);
            }
            None => {}
        }
    }

    /// alias, label, ?descr, ?sprite, ?tags, $link
    pub fn add_person_or_system(&mut self, kind: &str, attrs: &[C4Attr]) {
        let i = self.shape(at(attrs, 0), at(attrs, 1));
        text_element(&mut self.model.shapes[i].element, "descr", at(attrs, 2));
        self.finish_shape(i, kind, attrs, 3);
    }

    /// alias, label, ?techn, ?descr, ?sprite, ?tags, $link
    pub fn add_container(&mut self, kind: &str, attrs: &[C4Attr]) {
        let i = self.shape(at(attrs, 0), at(attrs, 1));
        let el = &mut self.model.shapes[i].element;
        text_element(el, "techn", at(attrs, 2));
        text_element(el, "descr", at(attrs, 3));
        self.finish_shape(i, kind, attrs, 4);
    }

    pub fn add_component(&mut self, kind: &str, attrs: &[C4Attr]) {
        self.add_container(kind, attrs);
    }

    fn finish_shape(&mut self, i: usize, kind: &str, attrs: &[C4Attr], from: usize) {
        let current = self.current;
        let shape = &mut self.model.shapes[i];
        put_element(&mut shape.element, "sprite", at(attrs, from));
        put_element(&mut shape.element, "tags", at(attrs, from + 1));
        put_element(&mut shape.element, "link", at(attrs, from + 2));
        shape.kind = kind.to_string();
        shape.element.parent = Some(current);
    }

    fn boundary(&mut self, attrs: &[C4Attr], fallback: &str) -> usize {
        let i = self.boundary_element(at(attrs, 0), at(attrs, 1));
        let el = &mut self.model.boundaries[i].element;
        if at(attrs, 2).is_none() {
            el.kind_type = Some(fallback.to_string());
        } else {
            put_element(el, "type", at(attrs, 2));
        }
        i
    }

    fn open(&mut self, i: usize) {
        self.model.boundaries[i].element.parent = Some(self.current);
        self.stack.push(self.current);
        self.current = i;
    }

    /// alias, label, ?type, ?tags, $link
    pub fn add_person_or_system_boundary(&mut self, attrs: &[C4Attr], fallback: &str) {
        let i = self.boundary(attrs, fallback);
        let el = &mut self.model.boundaries[i].element;
        put_element(el, "tags", at(attrs, 3));
        put_element(el, "link", at(attrs, 4));
        self.open(i);
    }

    pub fn add_container_boundary(&mut self, attrs: &[C4Attr]) {
        self.add_person_or_system_boundary(attrs, "container");
    }

    /// alias, label, ?type, ?descr, ?sprite, ?tags, $link
    pub fn add_deployment_node(&mut self, node_type: &str, attrs: &[C4Attr]) {
        let i = self.boundary(attrs, "node");
        let b = &mut self.model.boundaries[i];
        text_element(&mut b.element, "descr", at(attrs, 3));
        // Mermaid drops whatever is in the sprite slot, a named argument included.
        put_element(&mut b.element, "sprite", at(attrs, 4));
        put_element(&mut b.element, "tags", at(attrs, 5));
        put_element(&mut b.element, "link", at(attrs, 6));
        b.node_type = Some(node_type.to_string());
        self.open(i);
    }

    pub fn pop_boundary_parse_stack(&mut self) {
        self.current = self.stack.pop().unwrap_or(0);
    }

    /// from, to, label, ?techn, ?descr, ?sprite, ?tags, $link
    pub fn add_rel(&mut self, rel_type: &str, attrs: &[C4Attr]) {
        let (Some(from), Some(to), Some(_)) = (at(attrs, 0), at(attrs, 1), at(attrs, 2)) else { return };
        let known = match (from, to) {
            (C4Attr::Str(f), C4Attr::Str(t)) => Some((f.clone(), t.clone())),
            _ => None,
        };
        let found = known.as_ref().and_then(|k| self.rel_index.get(k).copied());
        let i = match found {
            Some(i) => i,
            None => {
                let i = self.model.rels.len();
                let mut rel = C4Rel { rel_type: rel_type.to_string(), ..Default::default() };
                if let Some(k) = known {
                    self.rel_index.insert(k, i);
                }
                match from {
                    C4Attr::Str(s) => rel.from = s.clone(),
                    C4Attr::Named(n) => rel.from_attr = Some(n.clone()),
                }
                match to {
                    C4Attr::Str(s) => rel.to = s.clone(),
                    C4Attr::Named(n) => rel.to_attr = Some(n.clone()),
                }
                self.model.rels.push(rel);
                i
            }
        };
        let rel = &mut self.model.rels[i];
        rel.rel_type = rel_type.to_string();
        label_rel(rel, at(attrs, 2));
        text_rel(rel, "techn", at(attrs, 3));
        text_rel(rel, "descr", at(attrs, 4));
        put_rel(rel, "sprite", at(attrs, 5));
        put_rel(rel, "tags", at(attrs, 6));
        put_rel(rel, "link", at(attrs, 7));
    }

    /// elementName, ?bgColor, ?fontColor, ?borderColor, ?shadowing, ?shape, ?sprite, ?techn,
    /// ?legendText, ?legendSprite
    pub fn update_el_style(&mut self, attrs: &[C4Attr]) {
        let Some(C4Attr::Str(alias)) = at(attrs, 0) else { return };
        let el = match self.model.element(alias) {
            Some(ElementRef::Shape(i)) => &mut self.model.shapes[i].element,
            Some(ElementRef::Boundary(i)) => &mut self.model.boundaries[i].element,
            None => return,
        };
        for (slot, field) in ["bgColor", "fontColor", "borderColor", "shadowing", "shape", "sprite", "techn", "legendText", "legendSprite"]
            .iter()
            .enumerate()
        {
            put_element(el, field, at(attrs, slot + 1));
        }
    }

    /// from, to, ?textColor, ?lineColor, ?offsetX, ?offsetY
    pub fn update_rel_style(&mut self, attrs: &[C4Attr]) {
        let (Some(C4Attr::Str(from)), Some(C4Attr::Str(to))) = (at(attrs, 0), at(attrs, 1)) else { return };
        let Some(&i) = self.rel_index.get(&(from.clone(), to.clone())) else { return };
        let rel = &mut self.model.rels[i];
        put_rel(rel, "textColor", at(attrs, 2));
        put_rel(rel, "lineColor", at(attrs, 3));
        match at(attrs, 4) {
            Some(C4Attr::Str(x)) => rel.offset_x = Some(js::parse_int(x, 0)),
            Some(C4Attr::Named(n)) => named_rel(rel, n),
            None => {}
        }
        match at(attrs, 5) {
            Some(C4Attr::Str(y)) => rel.offset_y = Some(js::parse_int(y, 0)),
            Some(C4Attr::Named(n)) => named_rel(rel, n),
            None => {}
        }
    }

    /// ?c4ShapeInRow, ?c4BoundaryInRow. As in Mermaid, the slot decides which one a value
    /// sets, not its name.
    pub fn update_layout_config(&mut self, attrs: &[C4Attr]) {
        let count = |attr: Option<&C4Attr>| match attr {
            None => f64::NAN,
            Some(C4Attr::Str(s)) => js::parse_int(s, 0),
            Some(C4Attr::Named(n)) => js::parse_int(&n.value, 0),
        };
        let shapes = count(at(attrs, 0));
        let boundaries = count(at(attrs, 1));
        if shapes >= 1.0 {
            self.model.shape_in_row = shapes;
        }
        if boundaries >= 1.0 {
            self.model.boundary_in_row = boundaries;
        }
    }
}
