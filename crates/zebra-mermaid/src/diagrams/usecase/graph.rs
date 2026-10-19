//! Graph projection of the resolved use-case model, including ordered JSON rows.
use super::types::*;
use crate::util::js;
use indexmap::IndexMap;
#[derive(Clone, Debug)]
pub struct JsonRow {
    pub key: String,
    pub accessible_key: String,
    pub value: String,
}
#[derive(Clone, Debug, Default)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub label_type: String,
    pub shape: String,
    pub group: bool,
    pub padding: f64,
    pub classes: String,
    pub styles: Vec<String>,
    pub compiled: Vec<String>,
    pub actor_type: Option<String>,
    pub business: bool,
    pub icon: Option<String>,
    pub stereotype: Option<String>,
    pub parent: Option<String>,
    pub note_target: Option<String>,
    pub json_rows: Option<Vec<JsonRow>>,
    pub boundary_type: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Edge {
    pub id: String,
    pub start: String,
    pub end: String,
    pub kind: String,
    pub dotted: bool,
    pub head: String,
    pub tail: String,
    pub label: Option<String>,
    pub label_type: Option<String>,
    pub classes: String,
    pub styles: Vec<String>,
    pub compiled: Vec<String>,
    pub animate: bool,
    pub animation: Option<String>,
    pub minlen: f64,
    pub explicit_id: bool,
    pub internal: bool,
}
fn classes(names: impl IntoIterator<Item = String>) -> String {
    names.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
}
pub fn flatten_json_rows(json: &OrderedJson) -> Vec<JsonRow> {
    let mut rows = vec![];
    let mut stack = vec![(json.root, String::new(), String::new())];
    while let Some((id, path, pointer)) = stack.pop() {
        let mut children = vec![];
        match &json.values[id] {
            JsonValue::Array(items) => {
                if items.is_empty() {
                    rows.push(JsonRow { key: path.clone(), accessible_key: path, value: "[]".into() });
                } else if items.iter().all(|&i| matches!(json.values[i], JsonValue::Scalar(_))) {
                    for (i, &item) in items.iter().enumerate() {
                        let JsonValue::Scalar(v) = &json.values[item] else { unreachable!() };
                        rows.push(JsonRow {
                            key: if i == 0 { path.clone() } else { String::new() },
                            accessible_key: path.clone(),
                            value: v.to_js_string(),
                        });
                    }
                } else {
                    for (i, &item) in items.iter().enumerate() {
                        children.push((item, format!("{path}[{i}]"), format!("{pointer}/{i}")));
                    }
                }
            }
            JsonValue::Object(object) => {
                let keys: Vec<_> = json.property_order.get(&pointer).map_or_else(|| object.keys().cloned().collect(), Clone::clone);
                if keys.is_empty() {
                    rows.push(JsonRow { key: path.clone(), accessible_key: path.clone(), value: "{}".into() });
                }
                for key in keys {
                    if let Some(&id) = object.get(&key) {
                        children.push((
                            id,
                            if path.is_empty() { key.clone() } else { format!("{path}.{key}") },
                            format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                        ));
                    } else {
                        rows.push(JsonRow {
                            key: if path.is_empty() { key.clone() } else { format!("{path}.{key}") },
                            accessible_key: if path.is_empty() { key.clone() } else { format!("{path}.{key}") },
                            value: "undefined".into(),
                        });
                    }
                }
            }
            JsonValue::Scalar(value) => rows.push(JsonRow { key: path.clone(), accessible_key: path, value: value.to_js_string() }),
        }
        stack.extend(children.into_iter().rev());
    }
    rows
}
pub fn build(model: &UsecaseModel) -> (Vec<Node>, Vec<Edge>) {
    let compiled = |names: &[String]| {
        let mut out = IndexMap::new();
        for name in std::iter::once("default").chain(names.iter().map(String::as_str)) {
            if let Some(def) = model.class_defs.get(name) {
                for raw in def {
                    let style = js::trim(raw);
                    let property = js::trim(style.split_once(':').map_or(style, |(p, _)| p));
                    if !property.is_empty() {
                        out.insert(property.to_owned(), style.to_owned());
                    }
                }
            }
        }
        out.into_values().collect::<Vec<String>>()
    };
    let mut nodes = vec![];
    let mut edges = vec![];
    for actor in model.actors.values() {
        let shape = match actor.kind.as_str() {
            "hollow" => "usecaseActorHollow",
            "awesome" => "usecaseActorAwesome",
            "icon" => "usecaseActorIcon",
            _ => "usecaseActor",
        };
        let mut names = vec![
            "default".into(),
            "usecase-actor".into(),
            format!("usecase-actor-{}", actor.kind),
            if actor.business { "usecase-business".into() } else { String::new() },
        ];
        names.extend(actor.classes.iter().cloned());
        nodes.push(Node {
            id: actor.id.clone(),
            label: actor.label.clone(),
            label_type: actor.label_type.clone(),
            shape: shape.into(),
            padding: 10.0,
            classes: classes(names),
            styles: actor.styles.clone(),
            compiled: compiled(&actor.classes),
            actor_type: Some(actor.kind.clone()),
            business: actor.business,
            icon: actor.icon.clone().filter(|s| !s.is_empty()),
            stereotype: actor.stereotype.clone().filter(|s| !s.is_empty()),
            parent: actor.parent.clone().filter(|s| !s.is_empty()),
            ..Default::default()
        });
    }
    for case in model.use_cases.values() {
        let ellipse = case.shape == "ellipse";
        let mut names = vec![
            "default".into(),
            "usecase-element".into(),
            format!("usecase-{}", case.shape),
            if case.business { "usecase-business".into() } else { String::new() },
        ];
        names.extend(case.classes.iter().cloned());
        nodes.push(Node {
            id: case.id.clone(),
            label: case.label.clone(),
            label_type: case.label_type.clone(),
            shape: if !ellipse {
                "rect"
            } else if case.business {
                "usecaseBusiness"
            } else {
                "usecaseEllipse"
            }
            .into(),
            padding: if ellipse { 20.0 } else { 10.0 },
            classes: classes(names),
            styles: case.styles.clone(),
            compiled: compiled(&case.classes),
            business: case.business,
            stereotype: case.stereotype.clone().filter(|s| !s.is_empty()),
            parent: case.parent.clone().filter(|s| !s.is_empty()),
            ..Default::default()
        });
    }
    for note in model.notes.values() {
        nodes.push(Node {
            id: note.id.clone(),
            label: note.label.clone(),
            label_type: note.label_type.clone(),
            shape: "note".into(),
            padding: 10.0,
            classes: "default usecase-note".into(),
            compiled: compiled(&[]),
            note_target: Some(note.target.clone()),
            ..Default::default()
        });
    }
    for json in model.json_nodes.values() {
        nodes.push(Node {
            id: json.id.clone(),
            label: json.id.clone(),
            label_type: "text".into(),
            shape: "usecaseJsonTable".into(),
            padding: 10.0,
            classes: classes([vec!["default".into(), "usecase-json-table".into()], json.classes.clone()].concat()),
            styles: json.styles.clone(),
            compiled: compiled(&json.classes),
            json_rows: Some(flatten_json_rows(&json.value)),
            ..Default::default()
        });
    }
    for b in model.boundaries.values() {
        nodes.push(Node {
            id: b.id.clone(),
            label: b.label.clone(),
            label_type: b.label_type.clone(),
            shape: "usecaseSystemBoundary".into(),
            group: true,
            padding: 20.0,
            classes: classes(
                [vec!["default".into(), "system-boundary".into(), format!("system-boundary-{}", b.kind)], b.classes.clone()].concat(),
            ),
            styles: b.styles.clone(),
            compiled: compiled(&b.classes),
            boundary_type: Some(b.kind.clone()),
            ..Default::default()
        });
    }
    const MARKERS: [(&str, &str); 7] = [
        ("none", "arrow_point"),
        ("arrow_point", "none"),
        ("none", "none"),
        ("none", "arrow_circle"),
        ("none", "arrow_cross"),
        ("arrow_circle", "none"),
        ("arrow_cross", "none"),
    ];
    for r in &model.relationships {
        let semantic = matches!(r.kind.as_str(), "include" | "extend");
        let (tail, head) = if r.kind == "generalization" {
            ("none", "extension")
        } else if r.kind != "association" {
            MARKERS[0]
        } else {
            MARKERS.get(r.arrow_type as usize).copied().unwrap_or(MARKERS[2])
        };
        let animated = r.animate || r.animation.is_some();
        let mut names = vec!["default".into(), "relationship".into(), format!("relationship-{}", r.kind)];
        names.extend(r.classes.iter().cloned());
        if animated {
            names.push(format!("edge-animation-{}", r.animation.as_deref().unwrap_or("fast")));
        }
        edges.push(Edge {
            id: r.id.clone(),
            start: r.source.clone(),
            end: r.target.clone(),
            kind: r.kind.clone(),
            dotted: semantic,
            head: head.into(),
            tail: tail.into(),
            label: if semantic {
                Some(r.kind.clone())
            } else if r.kind == "association" {
                r.label.clone().filter(|s| !s.is_empty())
            } else {
                None
            },
            label_type: if semantic {
                Some("text".into())
            } else if r.kind == "association" {
                r.label_type.clone().filter(|s| !s.is_empty())
            } else {
                None
            },
            classes: classes(names),
            styles: r.styles.clone(),
            compiled: compiled(&r.classes),
            animate: r.animate,
            animation: r.animation.clone(),
            minlen: r.minlen,
            explicit_id: r.explicit_id,
            internal: false,
        });
    }
    for note in model.notes.values() {
        edges.push(Edge {
            id: format!("{}-edge", note.id),
            start: note.id.clone(),
            end: note.target.clone(),
            kind: "note".into(),
            dotted: true,
            head: "none".into(),
            tail: "none".into(),
            classes: "default relationship relationship-note".into(),
            compiled: compiled(&[]),
            minlen: 1.0,
            internal: true,
            ..Default::default()
        });
    }
    (nodes, edges)
}
