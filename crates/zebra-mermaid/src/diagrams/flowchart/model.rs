//! Flowchart semantic model, ported from Pele types.ts and db.ts.
use crate::config::yaml::parse_yaml;
use crate::util::{js, url::safe_url};
use crate::{ConfigMap, ConfigValue, DiagramType, Error};
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlowText {
    pub text: String,
    pub kind: String,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Node {
    pub id: String,
    pub text: Option<String>,
    pub label_type: String,
    pub shape: Option<String>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    pub dir: Option<String>,
    pub props: IndexMap<String, String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub icon: Option<String>,
    pub form: Option<String>,
    pub pos: Option<String>,
    pub img: Option<String>,
    pub constraint: Option<String>,
    pub asset_width: Option<f64>,
    pub asset_height: Option<f64>,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Edge {
    pub start: String,
    pub end: String,
    pub kind: String,
    pub text: String,
    pub label_type: String,
    pub stroke: String,
    pub length: Option<f64>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    pub id: String,
    pub user_id: bool,
    pub interpolate: Option<String>,
    pub animate: Option<bool>,
    pub animation: Option<String>,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Class {
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Subgraph {
    pub id: String,
    pub nodes: Vec<String>,
    pub title: String,
    pub classes: Vec<String>,
    pub dir: Option<String>,
    pub label_type: String,
    pub metadata: ConfigMap,
}
#[derive(Debug, Clone, Default)]
pub struct Link {
    pub kind: String,
    pub stroke: String,
    pub length: Option<f64>,
    pub text: Option<FlowText>,
    pub id: Option<String>,
}
#[derive(Debug, Clone)]
pub enum DocItem {
    Nodes(Vec<String>),
    Text(String),
    Dir(String),
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlowchartModel {
    pub direction: Option<String>,
    pub nodes: IndexMap<String, Node>,
    pub edges: Vec<Edge>,
    pub subgraphs: Vec<Subgraph>,
    pub classes: IndexMap<String, Class>,
    pub tooltips: IndexMap<String, String>,
    pub default_edge_style: Vec<String>,
    pub default_interpolate: Option<String>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub inherit_dir: bool,
    pub max_edges: usize,
    /// Collect warnings when a host requests diagnostics, equivalent to Pele's warn callback.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub collect_warnings: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub warnings: Vec<String>,
    #[cfg(test)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) calls: Option<Vec<(&'static str, Vec<ConfigValue>)>>,
    members: IndexSet<String>,
    edge_by_id: IndexMap<String, usize>,
    pair_count: IndexMap<(String, String), usize>,
    sub_count: usize,
}
fn unquote(text: &str) -> String {
    text.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(text).to_owned()
}
fn label_type(kind: &str) -> String {
    if matches!(kind, "text" | "string" | "markdown") { kind } else { "markdown" }.to_owned()
}
fn semantic(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Flowchart)
}
impl FlowchartModel {
    pub fn exists(all: &[Subgraph], id: &str) -> bool {
        all.iter().any(|sg| sg.nodes.iter().any(|node| node == id))
    }
    pub fn make_unique(sg: &Subgraph, all: &[Subgraph]) -> Vec<String> {
        let taken: IndexSet<&str> = all.iter().flat_map(|s| s.nodes.iter().map(String::as_str)).collect();
        sg.nodes.iter().filter(|id| !taken.contains(id.as_str())).cloned().collect()
    }
    pub fn vertex(
        &mut self,
        id: &str,
        text: Option<FlowText>,
        shape: Option<&str>,
        styles: Option<Vec<String>>,
        props: Option<IndexMap<String, String>>,
        metadata: Option<&str>,
    ) -> Result<(), Error> {
        if js::trim(id).is_empty() {
            return Ok(());
        }
        let doc = if let Some(raw) = metadata {
            let yaml = if raw.contains('\n') { format!("{raw}\n") } else { format!("{{\n{raw}\n}}") };
            match parse_yaml(&yaml).map_err(|e| {
                Error::new(format!("Metadata of \"{id}\" is not valid YAML. {}", e.message), crate::ErrorCode::Syntax)
                    .with_type(DiagramType::Flowchart)
            })? {
                ConfigValue::Object(map) => Some(map),
                _ => None,
            }
        } else {
            None
        };
        if let (Some(index), Some(doc)) = (self.subgraphs.iter().position(|s| s.id == id), doc.as_ref()) {
            for (k, v) in doc.iter() {
                self.subgraphs[index].metadata.insert(k, v.clone());
            }
            return Ok(());
        }
        if let Some(&i) = self.edge_by_id.get(id) {
            if let Some(doc) = doc {
                let e = &mut self.edges[i];
                if let Some(v) = doc.get("animate") {
                    e.animate = Some(v.truthy());
                }
                if let Some(v) = doc.get("animation") {
                    e.animation = Some(v.to_js_string());
                }
                if let Some(v) = doc.get("curve") {
                    e.interpolate = Some(v.to_js_string());
                }
            }
            return Ok(());
        }
        if self.collect_warnings && !self.nodes.contains_key(id) && text.is_none() && shape.is_none() && styles.is_some() {
            self.warnings
                .push(format!("Style applied to unknown node \"{id}\". This may indicate a typo. The node will be created automatically."));
        }
        let n = self.nodes.entry(id.into()).or_insert_with(|| Node { id: id.into(), label_type: "text".into(), ..Default::default() });
        if let Some(t) = text {
            n.text = Some(unquote(js::trim(&t.text)));
            n.label_type = t.kind;
        } else if n.text.is_none() {
            n.text = Some(id.into());
        }
        if let Some(s) = shape {
            n.shape = Some(s.into());
        }
        if let Some(s) = styles {
            n.styles.extend(s);
        }
        if let Some(p) = props {
            n.props.extend(p);
        }
        let Some(doc) = doc else { return Ok(()) };
        if let Some(v) = doc.get("shape").filter(|v| v.truthy()) {
            let name = v.to_js_string();
            if name != name.to_lowercase() || name.contains('_') {
                return Err(semantic(format!("No such shape: {name}. Shape names should be lowercase.")));
            }
            if !super::shapes::valid(&name) {
                return Err(semantic(format!("No such shape: {name}.")));
            }
            n.shape = Some(name);
        }
        let label = doc.get("label").and_then(ConfigValue::as_str);
        if let Some(v) = doc.get("label").filter(|v| v.truthy()) {
            n.text = Some(v.to_js_string());
            n.label_type = label_type(doc.get("labelType").and_then(ConfigValue::as_str).unwrap_or(""));
        }
        for (key, field) in
            [("icon", &mut n.icon), ("form", &mut n.form), ("pos", &mut n.pos), ("img", &mut n.img), ("constraint", &mut n.constraint)]
        {
            if let Some(v) = doc.get(key).filter(|v| v.truthy()) {
                *field = Some(v.to_js_string());
            }
        }
        if (n.icon.is_some() || n.img.is_some()) && label.is_none_or(|s| js::trim(s).is_empty()) && n.text.as_deref() == Some(id) {
            n.text = Some(String::new());
        }
        if let Some(v) = doc.get("w").filter(|v| v.truthy()) {
            n.asset_width = Some(js::number(&v.to_js_string()));
        }
        if let Some(v) = doc.get("h").filter(|v| v.truthy()) {
            n.asset_height = Some(js::number(&v.to_js_string()));
        }
        Ok(())
    }
    pub fn add_link(&mut self, starts: &[String], ends: &[String], link: &Link) -> Result<(), Error> {
        for (si, start) in starts.iter().enumerate() {
            for (ei, end) in ends.iter().enumerate() {
                if self.edges.len() >= self.max_edges {
                    return Err(Error::limit(format!(
                        "Edge limit exceeded. {} edges found, but the limit is {}.",
                        self.edges.len(),
                        self.max_edges
                    ))
                    .with_type(DiagramType::Flowchart));
                }
                let candidate = if si + 1 == starts.len() && ei == 0 { link.id.as_ref().map(|s| s.replacen('@', "", 1)) } else { None };
                let user_id = candidate.as_ref().is_some_and(|s| !s.is_empty() && !self.edge_by_id.contains_key(s));
                let pair = (start.clone(), end.clone());
                let count = self.pair_count.get(&pair).copied().unwrap_or(0);
                let id = if user_id { candidate.unwrap() } else { format!("L_{start}_{end}_{}", if count == 0 { 0 } else { count + 1 }) };
                let text = link.text.as_ref();
                self.edges.push(Edge {
                    start: start.clone(),
                    end: end.clone(),
                    kind: link.kind.clone(),
                    stroke: link.stroke.clone(),
                    length: link.length.map(|n| if n > 10.0 { 10.0 } else { n }),
                    text: text.map_or_else(String::new, |t| unquote(js::trim(&t.text))),
                    label_type: text.map_or_else(|| "text".into(), |t| label_type(&t.kind)),
                    id: id.clone(),
                    user_id,
                    interpolate: self.default_interpolate.clone(),
                    ..Default::default()
                });
                self.pair_count.insert(pair, count + 1);
                self.edge_by_id.entry(id).or_insert(self.edges.len() - 1);
            }
        }
        Ok(())
    }
    fn edge_index(&self, pos: &str) -> Result<usize, Error> {
        let n = js::number(pos);
        if n.is_finite() && n >= 0.0 && n.fract() == 0.0 && (n as usize) < self.edges.len() {
            Ok(n as usize)
        } else {
            Err(semantic(format!("The index {pos} for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and {}. (Help: Ensure that the index is within the range of existing edges.)",self.edges.len() as i64-1)))
        }
    }
    pub fn update_link(&mut self, positions: &[String], mut styles: Vec<String>) -> Result<(), Error> {
        for pos in positions {
            if pos == "default" {
                self.default_edge_style = styles.clone();
            } else {
                let i = self.edge_index(pos)?;
                if !styles.is_empty() && !styles.iter().any(|s| s.starts_with("fill")) {
                    styles.push("fill:none".into());
                }
                self.edges[i].styles = styles.clone();
            }
        }
        Ok(())
    }
    pub fn interpolate(&mut self, positions: &[String], value: &str) -> Result<(), Error> {
        for pos in positions {
            if pos == "default" {
                self.default_interpolate = Some(value.into());
            } else {
                let i = self.edge_index(pos)?;
                self.edges[i].interpolate = Some(value.into());
            }
        }
        Ok(())
    }
    pub fn add_class(&mut self, ids: &str, styles: Vec<String>) {
        let joined = styles.join(",").replace("\\,", "§§§").replace(',', ";").replace("§§§", ",");
        for id in ids.split(',') {
            let c = self.classes.entry(id.into()).or_default();
            for s in joined.split(';') {
                c.styles.push(s.into());
                if s.contains("color") {
                    c.text_styles.push(s.replacen("fill", "bgFill", 1));
                }
            }
        }
    }
    pub fn set_class(&mut self, ids: &str, name: &str) {
        for id in ids.split(',') {
            if let Some(n) = self.nodes.get_mut(id) {
                n.classes.push(name.into());
            }
            if let Some(&i) = self.edge_by_id.get(id) {
                self.edges[i].classes.push(name.into());
            }
            if let Some(s) = self.subgraphs.iter_mut().find(|s| s.id == id) {
                s.classes.push(name.into());
            }
        }
    }
    pub fn set_link(&mut self, ids: &str, url: &str, target: Option<String>) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            let mut args = vec![ConfigValue::String(ids.into()), ConfigValue::String(url.into())];
            if let Some(target) = &target {
                args.push(ConfigValue::String(target.clone()));
            }
            calls.push(("setLink", args));
        }
        let u = js::trim(url);
        for id in ids.split(',') {
            if let Some(n) = self.nodes.get_mut(id) {
                n.link = (!u.is_empty()).then(|| safe_url(u));
                n.link_target = target.clone();
            }
        }
        self.set_class(ids, "clickable");
    }
    pub fn tooltip(&mut self, ids: &str, value: String) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            calls.push(("setTooltip", vec![ConfigValue::String(ids.into()), ConfigValue::String(value.clone())]));
        }
        for id in ids.split(',') {
            self.tooltips.insert(id.into(), value.clone());
        }
    }
    /// Keep the clickable class without executing a callback, matching Pele's inert renderer.
    pub fn set_click_event(&mut self, ids: &str, _name: &str, _args: Option<&str>) {
        #[cfg(test)]
        if let Some(calls) = &mut self.calls {
            let mut args = vec![ConfigValue::String(ids.into()), ConfigValue::String(_name.into())];
            if let Some(value) = _args {
                args.push(ConfigValue::String(value.into()));
            }
            calls.push(("setClickEvent", args));
        }
        self.set_class(ids, "clickable");
    }
    pub fn set_direction(&mut self, dir: &str) {
        let mut d = js::trim(dir);
        if d.contains('<') {
            d = "RL";
        }
        if d.contains('^') {
            d = "BT";
        }
        if d.contains('>') {
            d = "LR";
        }
        if d.contains('v') {
            d = "TB";
        }
        if d == "TD" {
            d = "TB";
        }
        self.direction = Some(d.into());
    }
    pub fn add_subgraph(
        &mut self,
        id_text: Option<FlowText>,
        items: Vec<DocItem>,
        title_text: Option<FlowText>,
        same_title: bool,
    ) -> String {
        let mut id = id_text.as_ref().map(|t| js::trim(&t.text).to_owned());
        let title = title_text.as_ref().map_or("", |t| t.text.as_str());
        if same_title && title.chars().any(crate::util::chars::is_space) {
            id = None;
        }
        let mut names = IndexSet::new();
        let mut dir = None;
        for item in items {
            match item {
                DocItem::Text(t) => {
                    if !js::trim(&t).is_empty() {
                        names.insert(t);
                    }
                }
                DocItem::Nodes(v) => {
                    for t in v {
                        if !js::trim(&t).is_empty() {
                            names.insert(t);
                        }
                    }
                }
                DocItem::Dir(v) => dir = Some(v),
            }
        }
        if dir.is_none() && self.inherit_dir {
            dir = self.direction.clone();
        }
        let id = id.unwrap_or_else(|| format!("subGraph{}", self.sub_count));
        self.sub_count += 1;
        let nodes: Vec<_> = names.into_iter().filter(|n| n != &id && !self.members.contains(n)).collect();
        self.members.extend(nodes.iter().cloned());
        if let Some(s) = self.subgraphs.iter_mut().find(|s| s.id == id) {
            s.nodes.extend(nodes);
        } else {
            self.subgraphs.push(Subgraph {
                id: id.clone(),
                nodes,
                title: js::trim(title).into(),
                label_type: label_type(title_text.as_ref().map_or("", |t| &t.kind)),
                dir,
                ..Default::default()
            });
        }
        id
    }
}
pub fn destruct_link(end: &str, start: Option<&str>) -> Link {
    let s = js::trim(end);
    let mut line = s.get(..s.len().saturating_sub(1)).unwrap_or("");
    let mut kind = match s.as_bytes().last() {
        Some(b'x') => "arrow_cross",
        Some(b'>') => "arrow_point",
        Some(b'o') => "arrow_circle",
        _ => "arrow_open",
    }
    .to_owned();
    let first = match s.as_bytes().last() {
        Some(b'x') => Some('x'),
        Some(b'>') => Some('<'),
        Some(b'o') => Some('o'),
        _ => None,
    };
    if first.is_some_and(|c| s.starts_with(c)) {
        kind = format!("double_{kind}");
        line = &line[1..];
    }
    let dots = line.bytes().filter(|&c| c == b'.').count();
    let stroke = if dots > 0 {
        "dotted"
    } else if line.starts_with('=') {
        "thick"
    } else if line.starts_with('~') {
        "invisible"
    } else {
        "normal"
    };
    let length = Some(if dots > 0 { dots as f64 } else { js::len16(line) as f64 - 1.0 });
    if let Some(start) = start.filter(|s| !s.is_empty()) {
        let raw = js::trim(start);
        let head = match raw.chars().next() {
            Some('<') => "arrow_point",
            Some('x') => "arrow_cross",
            Some('o') => "arrow_circle",
            _ => "arrow_open",
        };
        let style = if raw.contains('.') {
            "dotted"
        } else if raw.contains('=') {
            "thick"
        } else {
            "normal"
        };
        if style != stroke || (head != "arrow_open" && head != kind) {
            return Link { kind: "INVALID".into(), stroke: "INVALID".into(), ..Default::default() };
        }
        if head != "arrow_open" {
            kind = format!("double_{head}");
        }
    }
    Link { kind, stroke: stroke.into(), length, ..Default::default() }
}
