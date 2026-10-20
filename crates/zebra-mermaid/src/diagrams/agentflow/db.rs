//! Agentflow semantic actions, metadata, diagnostics and source mappings.
use super::{graph::Graph, shapes, types::*};
use crate::diagrams::common::str_entry::or_insert_with as str_entry;
use crate::{
    config::yaml::parse_yaml,
    util::{
        chars::{is_space, normalize_accessibility_description},
        js,
        url::safe_url,
    },
    ConfigMap, ConfigValue, DiagramType, Error, ErrorCode,
};
use indexmap::{IndexMap, IndexSet};
#[derive(Clone, Debug)]
struct Membership {
    ordinal: usize,
    owner: usize,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AgentflowModel {
    pub direction: Option<String>,
    pub nodes: IndexMap<String, Node>,
    pub edges: Vec<Edge>,
    pub subgraphs: Vec<Subgraph>,
    pub connectors: IndexSet<String>,
    pub classes: IndexMap<String, Class>,
    pub tooltips: IndexMap<String, String>,
    pub default_edge_style: Vec<String>,
    pub default_interpolate: Option<String>,
    pub mappings: Vec<Mapping>,
    pub diagnostics: Vec<Diagnostic>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub line_offset: usize,
    pub graph: Graph,
    pub max_edges: Option<usize>,
    pub inherit_dir: bool,
    pub configured_direction: Option<String>,
    lookup: IndexMap<String, usize>,
    global_nodes: IndexSet<String>,
    // A member belongs to exactly one flow. The owner lets a global block update only
    // affected flows; ordinals retain serialized order despite swap_remove.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_members"))]
    members: IndexMap<String, Membership>,
    #[cfg_attr(feature = "serde", serde(skip))]
    next_member: usize,
    edge_by_id: IndexMap<String, usize>,
    pair_count: IndexMap<String, usize>,
    sub_count: usize,
    bare_mappings: IndexSet<usize>,
    first_mapping: IndexMap<String, usize>,
}
#[cfg(feature = "serde")]
fn serialize_members<S: serde::Serializer>(members: &IndexMap<String, Membership>, serializer: S) -> Result<S::Ok, S::Error> {
    let mut ordered: Vec<_> = members.iter().collect();
    ordered.sort_unstable_by_key(|(_, member)| member.ordinal);
    serializer.collect_seq(ordered.into_iter().map(|(id, _)| id))
}
fn unquote(s: &str) -> String {
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s).to_owned()
}
fn label_type(s: Option<&str>) -> String {
    s.filter(|s| matches!(*s, "text" | "string" | "markdown")).unwrap_or("markdown").into()
}
fn shape_error(offset: usize, loc: Option<Loc>, message: String) -> Error {
    let mut e = Error::new(message, ErrorCode::Semantic).with_type(DiagramType::Agentflow);
    e.line = loc.map_or(0, |l| l.first_line + offset);
    e.column = loc.map_or(0, |l| l.first_column + 1);
    e
}
fn strip_prototype(value: &mut ConfigValue) {
    match value {
        ConfigValue::Array(items) => {
            for item in items {
                strip_prototype(item);
            }
        }
        ConfigValue::Object(object) => {
            for key in ["__proto__", "constructor", "prototype"] {
                object.remove(key);
            }
            let keys: Vec<_> = object.keys().into_iter().map(str::to_owned).collect();
            for key in keys {
                strip_prototype(object.get_mut(&key).unwrap());
            }
        }
        _ => {}
    }
}
fn merge(into: &mut Option<ConfigMap>, doc: &ConfigValue) {
    let target = into.get_or_insert_with(ConfigMap::new);
    match doc {
        ConfigValue::Object(fields) => {
            for (k, v) in fields.iter() {
                target.insert(k, v.clone());
            }
        }
        ConfigValue::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                target.insert(i.to_string(), v.clone());
            }
        }
        ConfigValue::String(s) => {
            for (i, c) in s.encode_utf16().enumerate() {
                target.insert(i.to_string(), ConfigValue::String(String::from_utf16_lossy(&[c])));
            }
        }
        _ => {}
    }
}
fn strip_commas(body: &str) -> String {
    let mut single = false;
    let mut double = false;
    let mut depth = 0usize;
    let mut block = None;
    let mut out = vec![];
    for line in body.split('\n') {
        let indent = js::len16(line) - js::len16(js::trim_start(line));
        if let Some(level) = block {
            if js::trim(line).is_empty() || indent > level {
                out.push(line.to_owned());
                continue;
            }
            block = None;
        }
        let chars: Vec<_> = line.char_indices().collect();
        let mut comment = None;
        let mut i = 0;
        while i < chars.len() {
            let (at, c) = chars[i];
            if double {
                if c == '\\' {
                    i += 1;
                } else if c == '"' {
                    double = false;
                }
            } else if single {
                if c == '\'' {
                    single = false;
                }
            } else if c == '"' {
                double = true;
            } else if c == '\'' {
                single = true;
            } else if c == '#' && (i == 0 || matches!(chars[i - 1].1, ' ' | '\t')) {
                comment = Some(at);
                break;
            } else if matches!(c, '[' | '{') {
                depth += 1;
            } else if matches!(c, ']' | '}') {
                depth = depth.saturating_sub(1);
            }
            i += 1;
        }
        if single || double || depth > 0 {
            out.push(line.to_owned());
            continue;
        }
        let (code, comment) = comment.map_or((line, ""), |i| (&line[..i], &line[i..]));
        let block_scalar = code.rsplit_once(':').is_some_and(|(_, tail)| {
            let tail = js::trim(tail);
            let mut chars = tail.chars();
            matches!(chars.next(), Some('>' | '|')) && chars.all(|c| c.is_ascii_digit() || matches!(c, '+' | '-'))
        });
        if block_scalar {
            block = Some(indent);
            out.push(line.to_owned());
            continue;
        }
        let trimmed = code.trim_end_matches([' ', '\t']);
        if let Some(prefix) = trimmed.strip_suffix(',') {
            out.push(format!("{prefix}{}{comment}", &code[trimmed.len()..]));
        } else {
            out.push(line.to_owned());
        }
    }
    out.join("\n")
}
fn semantic_metadata(data: &Option<ConfigMap>) -> Option<ConfigMap> {
    let mut out = ConfigMap::new();
    if let Some(data) = data {
        for (k, v) in data.iter() {
            if !matches!(k, "shape" | "view" | "icon" | "img" | "form" | "pos" | "w" | "h" | "class" | "style" | "labelType") {
                out.insert(k, v.clone());
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}
impl AgentflowModel {
    fn metadata(&self, id: &str, source: &str, loc: Option<Loc>) -> Result<ConfigValue, Error> {
        let multiline = source.contains('\n');
        let input = if multiline { format!("{source}\n") } else { format!("{{\n{source}\n}}") };
        let mut parsed = match parse_yaml(&input) {
            Ok(v) => v,
            Err(error) => {
                let stripped = if multiline { strip_commas(source) } else { source.into() };
                let retried = if stripped != source { parse_yaml(&format!("{stripped}\n")).ok() } else { None };
                if let Some(value) = retried {
                    value
                } else {
                    let at = loc.map_or_else(String::new, |l| format!(" ({}:{})", l.first_line + self.line_offset, l.first_column + 1));
                    let mut e = Error::new(format!("Metadata of \"{id}\" is not valid YAML. {}{at}", error.message), ErrorCode::Syntax)
                        .with_type(DiagramType::Agentflow);
                    e.line = loc.map_or(0, |l| l.first_line + self.line_offset);
                    e.column = loc.map_or(0, |l| l.first_column + 1);
                    return Err(e);
                }
            }
        };
        if parsed.is_null() {
            parsed = ConfigValue::Object(ConfigMap::new());
        }
        strip_prototype(&mut parsed);
        Ok(parsed)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn vertex(
        &mut self,
        id: &str,
        text: Option<FlowText>,
        shape: Option<&str>,
        styles: Option<Vec<String>>,
        props: Option<IndexMap<String, String>>,
        data: Option<&str>,
        loc: Option<Loc>,
    ) -> Result<(), Error> {
        if id.is_empty() || js::trim(id).is_empty() {
            return Ok(());
        }
        let mut doc = data.map(|s| self.metadata(id, s, loc)).transpose()?;
        let mut authored = None;
        if let Some(ConfigValue::Object(fields)) = &mut doc {
            if let Some(s) = fields.get("shape").and_then(ConfigValue::as_str) {
                authored = Some(s.to_owned());
                fields.insert("shape", ConfigValue::String(shapes::alias(s).into()));
            }
        }
        if let Some(&i) = self.lookup.get(id) {
            if let Some(doc) = doc.as_ref().filter(|d| d.truthy()) {
                merge(&mut self.subgraphs[i].metadata, doc);
                return Ok(());
            }
        }
        if self.connectors.contains(id) {
            if let Some(doc) = doc.as_ref().filter(|d| d.truthy()) {
                merge(&mut self.nodes.get_mut(id).unwrap().metadata, doc);
                return Ok(());
            }
        }
        if id == "connectors" {
            return Ok(());
        }
        if let Some(&i) = self.edge_by_id.get(id) {
            let edge = &mut self.edges[i];
            if let Some(fields) = doc.as_ref().and_then(ConfigValue::as_object) {
                if let Some(v) = fields.get("animate") {
                    edge.animate = Some(v.clone());
                }
                if let Some(v) = fields.get("animation") {
                    edge.animation = Some(v.clone());
                }
                if let Some(v) = fields.get("curve") {
                    edge.interpolate = Some(v.clone());
                }
            }
            if let Some(doc) = doc.as_ref().filter(|d| d.truthy()) {
                merge(&mut edge.metadata, doc);
            }
            return Ok(());
        }
        let vertex = str_entry(&mut self.nodes, id, || Node { id: id.into(), label_type: "text".into(), ..Default::default() });
        if let Some(t) = text {
            vertex.label_type = t.kind;
            vertex.text = Some(unquote(js::trim(&t.text)));
        } else if vertex.text.is_none() {
            vertex.text = Some(id.into());
        }
        if let Some(shape) = shape {
            vertex.shape = Some(shape.into());
        }
        if let Some(styles) = styles {
            vertex.styles.extend(styles);
        }
        if let Some(props) = props {
            vertex.props.extend(props);
        }
        let Some(doc) = doc.filter(|d| d.truthy()) else {
            return Ok(());
        };
        merge(&mut vertex.metadata, &doc);
        let Some(fields) = doc.as_object() else {
            return Ok(());
        };
        if let Some(s) = fields.get("shape").filter(|s| s.truthy()) {
            let authored = authored.as_deref().map(str::to_owned).or_else(|| s.as_str().map(str::to_owned));
            let Some(raw) = authored else {
                return Err(shape_error(self.line_offset, loc, format!("No such shape: {}.", s.to_json_string())));
            };
            if raw != raw.to_lowercase() || raw.contains('_') {
                return Err(shape_error(self.line_offset, loc, format!("No such shape: {raw}. Shape names should be lowercase.")));
            }
            let shape = s.as_str().unwrap();
            if !shapes::valid(shape) {
                return Err(shape_error(self.line_offset, loc, format!("No such shape: {shape}.")));
            }
            vertex.shape = Some(shape.into());
        }
        if let Some(label) = fields.get("label").filter(|v| v.truthy()) {
            vertex.text = Some(label.to_js_string());
            vertex.label_type = label_type(fields.get("labelType").and_then(ConfigValue::as_str));
        }
        Ok(())
    }
    pub fn add_link(&mut self, starts: &[String], ends: &[String], link: &Link) -> Result<(), Error> {
        let id = link.id.as_ref().map(|s| s.replacen('@', "", 1));
        for start in starts {
            for end in ends {
                let text = link.text.as_ref();
                let pair = format!("{start}\0{end}");
                let existing = self.pair_count.get(&pair).copied().unwrap_or(0);
                let candidate = if Some(start) == starts.last() && Some(end) == ends.first() { id.as_deref() } else { None };
                let user = candidate.is_some_and(|id| !id.is_empty() && !self.edge_by_id.contains_key(id));
                let id = if user {
                    candidate.unwrap().to_owned()
                } else {
                    format!("L_{start}_{end}_{}", if existing == 0 { 0 } else { existing + 1 })
                };
                if self.max_edges.is_some_and(|max| self.edges.len() >= max) {
                    return Err(Error::limit(format!(
                        "Edge limit exceeded. {} edges found, but the limit is {}.",
                        self.edges.len(),
                        self.max_edges.unwrap()
                    ))
                    .with_type(DiagramType::Agentflow));
                }
                self.edges.push(Edge {
                    start: start.clone(),
                    end: end.clone(),
                    kind: Some(link.kind.clone()),
                    text: text.map_or_else(String::new, |t| unquote(js::trim(&t.text))),
                    label_type: text.map_or_else(|| "text".into(), |t| label_type(Some(&t.kind))),
                    stroke: Some(link.stroke.clone()),
                    semantic: link.semantic.clone(),
                    length: link.length.map(|n| if n > 10.0 { 10.0 } else { n }),
                    id: id.clone(),
                    user_id: user,
                    interpolate: self.default_interpolate.clone().map(ConfigValue::String),
                    ..Default::default()
                });
                self.pair_count.insert(pair, existing + 1);
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
            Err(Error::semantic(format!("The index {pos} for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and {}. (Help: Ensure that the index is within the range of existing edges.)",self.edges.len()as i64-1)).with_type(DiagramType::Agentflow))
        }
    }
    pub fn interpolate(&mut self, positions: &[String], value: &str) -> Result<(), Error> {
        for pos in positions {
            if pos == "default" {
                self.default_interpolate = Some(value.into());
            } else {
                let i = self.edge_index(pos)?;
                self.edges[i].interpolate = Some(ConfigValue::String(value.into()));
            }
        }
        Ok(())
    }
    pub fn update_link(&mut self, positions: &[String], mut styles: Vec<String>) -> Result<(), Error> {
        let mut touched = vec![];
        let mut default = false;
        for pos in positions {
            if pos == "default" {
                default = true;
                self.default_edge_style = styles.clone();
            } else {
                let i = self.edge_index(pos)?;
                if !styles.is_empty() && !styles.iter().any(|s| s.starts_with("fill")) {
                    styles.push("fill:none".into());
                    if default {
                        self.default_edge_style = styles.clone();
                    }
                    for &k in &touched {
                        let e: &mut Edge = &mut self.edges[k];
                        e.styles = styles.clone();
                    }
                }
                self.edges[i].styles = styles.clone();
                touched.push(i);
            }
        }
        Ok(())
    }
    pub fn add_class(&mut self, ids: &str, styles: Vec<String>) {
        let joined = styles.join(",").replace("\\,", "§§§").replace(',', ";").replace("§§§", ",");
        for id in ids.split(',') {
            let c = str_entry(&mut self.classes, id, Default::default);
            for s in joined.split(';') {
                if s.contains("color") {
                    c.text_styles.push(s.replacen("fill", "bgFill", 1));
                }
                c.styles.push(s.into());
            }
        }
    }
    pub fn set_direction(&mut self, dir: &str) {
        let mut d = js::trim(dir).to_owned();
        for (c, value) in [('<', "RL"), ('^', "BT"), ('>', "LR"), ('v', "TB")] {
            if d.contains(c) {
                d = value.into();
            }
        }
        if d == "TD" {
            d = "TB".into();
        }
        self.direction = Some(d);
    }
    pub fn set_class(&mut self, ids: &str, name: &str) {
        for id in ids.split(',') {
            if let Some(v) = self.nodes.get_mut(id) {
                v.classes.push(name.into());
            }
            if let Some(&i) = self.edge_by_id.get(id) {
                self.edges[i].classes.push(name.into());
            }
            if let Some(&i) = self.lookup.get(id) {
                self.subgraphs[i].classes.push(name.into());
            }
        }
    }
    pub fn tooltip(&mut self, ids: &str, text: &str) {
        for id in ids.split(',') {
            self.tooltips.insert(id.into(), text.into());
        }
    }
    pub fn set_link(&mut self, ids: &str, url: &str, target: Option<&str>) {
        let url = js::trim(url);
        for id in ids.split(',') {
            if let Some(v) = self.nodes.get_mut(id) {
                v.link = if url.is_empty() { None } else { Some(safe_url(url)) };
                v.link_target = target.map(str::to_owned);
            }
        }
        self.set_class(ids, "clickable");
    }
    pub fn click(&mut self, ids: &str) {
        self.set_class(ids, "clickable");
    }
    pub fn set_acc_title(&mut self, s: &str) {
        self.acc_title = Some(js::trim_start(s).into());
    }
    pub fn set_acc_descr(&mut self, s: &str) {
        self.acc_descr = Some(normalize_accessibility_description(s));
    }
    pub fn add_subgraph(&mut self, id: Option<&FlowText>, list: &[DocItem], title: Option<&FlowText>, same: bool) -> String {
        let mut id = id.map(|id| js::trim(&id.text).to_string());
        let title_text = title.map_or("", |t| t.text.as_str());
        if same && title_text.chars().any(is_space) {
            id = None;
        }
        let mut seen = IndexSet::new();
        let mut items = vec![];
        let mut dir = None;
        for item in list {
            let raw: Vec<&String> = match item {
                DocItem::Text(s) => vec![s],
                DocItem::Nodes(items) => items.iter().collect(),
                DocItem::Dir(d) => {
                    dir = Some(d.clone());
                    vec![]
                }
            };
            for item in raw {
                if !js::trim(item).is_empty() && seen.insert(item.clone()) {
                    items.push(item.clone());
                }
            }
        }
        if dir.is_none() && self.inherit_dir {
            dir = self.direction.clone().or_else(|| self.configured_direction.clone());
        }
        let id = id.unwrap_or_else(|| format!("subGraph{}", self.sub_count));
        self.sub_count += 1;
        let nodes: Vec<_> =
            items.into_iter().filter(|n| n != &id && !self.global_nodes.contains(n) && !self.members.contains_key(n)).collect();
        let owner = self.lookup.get(&id).copied().unwrap_or(self.subgraphs.len());
        self.members.reserve(nodes.len());
        for node in &nodes {
            if let indexmap::map::Entry::Vacant(entry) = self.members.entry(node.clone()) {
                entry.insert(Membership { ordinal: self.next_member, owner });
                self.next_member += 1;
            }
        }
        if let Some(&i) = self.lookup.get(&id) {
            let s = &mut self.subgraphs[i];
            s.nodes.extend(nodes);
            if !js::trim(title_text).is_empty() {
                s.title = js::trim(title_text).into();
            }
            if dir.as_ref().is_some_and(|s| !s.is_empty()) {
                s.dir = dir;
            }
            return id;
        }
        self.lookup.insert(id.clone(), self.subgraphs.len());
        self.subgraphs.push(Subgraph {
            id: id.clone(),
            nodes,
            title: js::trim(title_text).into(),
            classes: vec![],
            dir,
            label_type: label_type(title.map(|t| t.kind.as_str())),
            metadata: None,
        });
        id
    }
    pub fn add_global(&mut self, list: &[DocItem]) {
        let mut first = None;
        let mut owners = Vec::new();
        for entry in list {
            let nodes = match entry {
                DocItem::Text(text) => std::slice::from_ref(text),
                DocItem::Nodes(nodes) => nodes.as_slice(),
                DocItem::Dir(_) => &[],
            };
            for text in nodes {
                let id = js::trim(text);
                if id.is_empty() {
                    continue;
                }
                self.global_nodes.insert(id.into());
                if let Some(member) = self.members.swap_remove(id) {
                    if let Some((owner, _)) = first {
                        if owners.is_empty() {
                            owners.push(owner);
                        }
                        owners.push(member.owner);
                    } else {
                        first = Some((member.owner, id));
                    }
                }
            }
        }
        if let Some((owner, id)) = first {
            if owners.is_empty() {
                // The common one-node block needs neither a temporary allocation nor
                // a hash lookup for every surviving node. Vec::remove preserves order.
                let nodes = &mut self.subgraphs[owner].nodes;
                if let Some(index) = nodes.iter().position(|node| node == id) {
                    nodes.remove(index);
                }
            } else {
                // Batch removals filter each affected flow only once.
                owners.sort_unstable();
                owners.dedup();
                for owner in owners {
                    self.subgraphs[owner].nodes.retain(|id| !self.global_nodes.contains(id));
                }
            }
        }
    }
    pub fn add_connector(&mut self, id: Option<&FlowText>, title: Option<&FlowText>) -> String {
        let Some(id) = id.map(|t| js::trim(&t.text)).filter(|s| !s.is_empty()) else {
            return String::new();
        };
        let title = title.map_or("", |t| js::trim(&t.text));
        if self.connectors.insert(id.into()) {
            self.nodes.insert(
                id.into(),
                Node {
                    id: id.into(),
                    text: Some(if title.is_empty() { id } else { title }.into()),
                    label_type: "text".into(),
                    is_connector: true,
                    ..Default::default()
                },
            );
        } else if !title.is_empty() {
            self.nodes.get_mut(id).unwrap().text = Some(title.into());
        }
        id.into()
    }
    pub fn destruct_link(raw: &str, start: Option<&str>) -> Link {
        let s = js::trim(raw);
        let last = s.chars().last();
        let kind = match last {
            Some('x') => "arrow_cross",
            Some('>') => "arrow_point",
            _ => "arrow_open",
        };
        let line = if matches!(last, Some('-' | '.')) { s } else { &s[..s.len().saturating_sub(last.map_or(0, char::len_utf8))] };
        let dots = line.chars().filter(|&c| c == '.').count();
        let stroke = if dots > 0 { "dotted" } else { "normal" };
        let length = if dots > 0 { dots as f64 } else { js::len16(line) as f64 - 1.0 };
        if let Some(start) = start.filter(|s| !s.is_empty()) {
            let first = if start.contains('.') { "dotted" } else { "normal" };
            if first != stroke {
                return Link { kind: "INVALID".into(), stroke: "INVALID".into(), ..Default::default() };
            }
        }
        let semantic = match (kind, stroke) {
            ("arrow_point", "normal") => Some("sequence"),
            ("arrow_cross", "normal") => Some("failure"),
            ("arrow_open", "dotted") => Some("reference"),
            _ => None,
        };
        Link { kind: kind.into(), stroke: stroke.into(), length: Some(length), semantic: semantic.map(str::to_owned), ..Default::default() }
    }
    pub fn subgraph(&self, id: &str) -> Option<&Subgraph> {
        self.lookup.get(id).map(|&i| &self.subgraphs[i])
    }
    pub fn vertex_kind(vertex: &Node) -> &'static str {
        let raw = vertex.shape.as_deref().unwrap_or("");
        if matches!(raw, "subroutine" | "subprocess" | "subproc" | "framed-rectangle" | "tool") {
            return "tool";
        }
        match shapes::alias(raw) {
            "hexagon" | "hex" => "action",
            "lean-right" | "lean_right" => "input",
            "lin-doc" | "lined-document" => "refdoc",
            "diamond" => "decision",
            _ => "task",
        }
    }
    fn position(&self, loc: Option<Loc>) -> Position {
        let l = loc.unwrap_or_default();
        Position {
            start_line: l.first_line + self.line_offset,
            start_column: l.first_column,
            end_line: l.last_line + self.line_offset,
            end_column: l.last_column,
            start_index: 0,
            end_index: 0,
        }
    }
    fn push_mapping(&mut self, id: &str, kind: &str, position: Position) -> usize {
        let i = self.mappings.len();
        self.mappings.push(Mapping { id: id.into(), kind: kind.into(), position });
        str_entry(&mut self.first_mapping, id, || i);
        i
    }
    pub fn vertex_mapping(&mut self, id: &str, shape: Option<&str>, loc: Loc) {
        if id.is_empty() {
            return;
        }
        let i = self.push_mapping(id, "vertex", self.position(Some(loc)));
        if shape.is_none() {
            self.bare_mappings.insert(i);
        }
    }
    pub fn extend_mapping(&mut self, id: &str, loc: Loc) {
        if id.is_empty() {
            return;
        }
        let end = self.position(Some(loc));
        for i in (0..self.mappings.len()).rev() {
            let m = &mut self.mappings[i];
            if m.kind != "vertex" || m.id != id {
                continue;
            }
            if self.bare_mappings.shift_remove(&i) {
                m.kind = "attachment".into();
            }
            if end.end_line > m.position.end_line || (end.end_line == m.position.end_line && end.end_column > m.position.end_column) {
                m.position.end_line = end.end_line;
                m.position.end_column = end.end_column;
            }
            return;
        }
        self.push_mapping(id, "attachment", end);
    }
    pub fn edge_mapping(&mut self, to: &[String], loc: Loc) {
        let ids: Vec<_> = to.iter().filter(|s| !s.is_empty()).cloned().collect();
        self.push_mapping(&if ids.is_empty() { "edge".into() } else { ids.join(">") }, "edge", self.position(Some(loc)));
    }
    pub fn block_mapping(&mut self, id: &str, kind: &str, start: Loc, end: Loc) {
        if id.is_empty() {
            return;
        }
        let a = self.position(Some(start));
        let b = self.position(Some(end));
        self.push_mapping(
            id,
            kind,
            Position {
                start_line: a.start_line,
                start_column: a.start_column,
                end_line: b.end_line,
                end_column: b.end_column,
                start_index: 0,
                end_index: 0,
            },
        );
    }
    pub fn emit(&mut self, id: &str, severity: &str, message: String, node: Option<&str>, edge: Option<&str>) {
        let anchor = node.or(edge);
        let node = node.filter(|s| !s.is_empty()).map(str::to_owned);
        let edge = if node.is_none() { edge.filter(|s| !s.is_empty()).map(str::to_owned) } else { None };
        if self
            .diagnostics
            .iter()
            .any(|d| d.id == id && d.severity == severity && d.message == message && d.node_id == node && d.edge_id == edge)
        {
            return;
        }
        let position =
            anchor.filter(|s| !s.is_empty()).and_then(|id| self.first_mapping.get(id)).map(|&i| self.mappings[i].position.clone());
        self.diagnostics.push(Diagnostic { id: id.into(), severity: severity.into(), message, node_id: node, edge_id: edge, position });
    }
    pub fn semantic_model(&self) -> ConfigValue {
        let mut root = ConfigMap::new();
        let mut vertices = vec![];
        let mut connectors = vec![];
        for (id, v) in &self.nodes {
            if self.lookup.contains_key(id) {
                continue;
            }
            let mut n = ConfigMap::new();
            n.insert("id", ConfigValue::String(id.clone()));
            if v.is_connector {
                if let Some(t) = v.text.as_ref().filter(|t| *t != id) {
                    n.insert("title", ConfigValue::String(t.clone()));
                }
                if let Some(m) = semantic_metadata(&v.metadata) {
                    n.insert("metadata", ConfigValue::Object(m));
                }
                connectors.push(ConfigValue::Object(n));
                continue;
            }
            if let Some(t) = &v.text {
                n.insert("label", ConfigValue::String(t.clone()));
            }
            if let Some(s) = &v.shape {
                n.insert("shape", ConfigValue::String(shapes::alias(s).into()));
            }
            n.insert("vertexKind", ConfigValue::String(Self::vertex_kind(v).into()));
            if let Some(m) = semantic_metadata(&v.metadata) {
                n.insert("metadata", ConfigValue::Object(m));
            }
            vertices.push(ConfigValue::Object(n));
        }
        let mut edges = vec![];
        for e in &self.edges {
            let mut v = ConfigMap::new();
            v.insert("start", ConfigValue::String(e.start.clone()));
            v.insert("end", ConfigValue::String(e.end.clone()));
            v.insert("id", ConfigValue::String(e.id.clone()));
            if !e.text.is_empty() {
                v.insert("label", ConfigValue::String(e.text.clone()));
            }
            for (key, value) in [("type", &e.kind), ("stroke", &e.stroke), ("edgeSemantic", &e.semantic)] {
                if let Some(s) = value {
                    v.insert(key, ConfigValue::String(s.clone()));
                }
            }
            if let Some(n) = e.length {
                v.insert("length", ConfigValue::Number(n));
            }
            if let Some(m) = e.metadata.as_ref().filter(|m| !m.is_empty()) {
                v.insert("metadata", ConfigValue::Object(m.clone()));
            }
            edges.push(ConfigValue::Object(v));
        }
        let mut groups = vec![];
        for sg in &self.subgraphs {
            let mut v = ConfigMap::new();
            v.insert("id", ConfigValue::String(sg.id.clone()));
            v.insert("nodes", ConfigValue::Array(sg.nodes.iter().cloned().map(ConfigValue::String).collect()));
            v.insert("type", ConfigValue::String("flow".into()));
            v.insert("title", ConfigValue::String(sg.title.clone()));
            if let Some(s) = &sg.dir {
                v.insert("direction", ConfigValue::String(s.clone()));
            }
            if let Some(m) = semantic_metadata(&sg.metadata) {
                v.insert("metadata", ConfigValue::Object(m));
            }
            groups.push(ConfigValue::Object(v));
        }
        root.insert("vertices", ConfigValue::Array(vertices));
        root.insert("edges", ConfigValue::Array(edges));
        root.insert("subGraphs", ConfigValue::Array(groups));
        root.insert("connectors", ConfigValue::Array(connectors));
        let mut diagnostics = vec![];
        for d in &self.diagnostics {
            let mut v = ConfigMap::new();
            for (k, s) in [("id", &d.id), ("severity", &d.severity), ("message", &d.message)] {
                v.insert(k, ConfigValue::String(s.clone()));
            }
            if let Some(s) = &d.node_id {
                v.insert("nodeId", ConfigValue::String(s.clone()));
            }
            if let Some(s) = &d.edge_id {
                v.insert("edgeId", ConfigValue::String(s.clone()));
            }
            if let Some(p) = &d.position {
                let mut pos = ConfigMap::new();
                for (k, n) in [
                    ("startLine", p.start_line),
                    ("startColumn", p.start_column),
                    ("endLine", p.end_line),
                    ("endColumn", p.end_column),
                    ("startIndex", p.start_index),
                    ("endIndex", p.end_index),
                ] {
                    pos.insert(k, ConfigValue::Number(n as f64));
                }
                v.insert("position", ConfigValue::Object(pos));
            }
            diagnostics.push(ConfigValue::Object(v));
        }
        root.insert("diagnostics", ConfigValue::Array(diagnostics));
        if let Some(s) = &self.direction {
            root.insert("direction", ConfigValue::String(s.clone()));
        }
        ConfigValue::Object(root)
    }
}
