//! The sequence model builder. Ported from Pele `src/diagrams/sequence/db.ts`.

use indexmap::IndexMap;
use std::borrow::Cow;

use super::colors::is_css_color;
use super::line_type as lt;
use super::{NoteActor, SeqActor, SeqBox, SeqContent, SeqMessage, SeqNote, SequenceModel};
use crate::config::json::{parse_json_limited, JsonError};
use crate::config::yaml::parse_yaml;
use crate::config::ConfigValue;
use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::chars::is_space;
use crate::util::js;

/// A text with its wrap marker (`SeqText`).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SeqText {
    pub text: String,
    pub wrap: Option<bool>,
}

/// What a `box` line gives (`SeqBoxData`).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SeqBoxData {
    pub text: Option<String>,
    pub color: String,
    pub wrap: Option<bool>,
}

/// The kind of a participant statement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Declare {
    Add,
    Create,
    Destroy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Menu {
    Links,
    ALink,
    Properties,
    Details,
}

/// One statement of the list the parser hands to `apply`, in the order Mermaid's grammar
/// builds it. Pele nests the statements of blocks in arrays and walks them depth first; this
/// list holds them in that same order already.
/// Actor references can borrow the parse source until `apply` stores owned model data.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Stmt<'a> {
    /// `addParticipant`, `createParticipant`, `destroyParticipant`. `draw` is None for a
    /// participant only mentioned in a message.
    Participant {
        kind: Declare,
        actor: Cow<'a, str>,
        draw: Option<&'static str>,
        description: Option<SeqText>,
        config: Option<String>,
    },
    /// `activeStart`, `activeEnd`, `centralConnection`, `centralConnectionReverse`.
    Signal {
        signal_type: i32,
        actor: Cow<'a, str>,
    },
    Message {
        from: Cow<'a, str>,
        to: Cow<'a, str>,
        signal_type: i32,
        msg: SeqText,
        activate: bool,
        central: Option<i32>,
    },
    Note {
        actor: NoteActor,
        placement: i32,
        text: SeqText,
    },
    Menu {
        kind: Menu,
        actor: Cow<'a, str>,
        text: SeqText,
    },
    SequenceIndex {
        start: Option<f64>,
        step: Option<f64>,
        visible: bool,
    },
    BoxStart(SeqBoxData),
    BoxEnd,
    /// The start of a block or a divider inside one, with its text.
    Section {
        signal_type: i32,
        text: SeqText,
    },
    /// The end of a block.
    BlockEnd {
        signal_type: i32,
    },
}

fn fail(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Sequence)
}

/// `/^:?wrap:/`, `/^:?nowrap:/`: the length of the prefix when there is one.
fn wrap_prefix(text: &str, word: &str) -> Option<usize> {
    let skip = usize::from(text.starts_with(':'));
    let rest = &text[skip..];
    if rest.starts_with(word) && rest[word.len()..].starts_with(':') {
        Some(skip + word.len() + 1)
    } else {
        None
    }
}

fn extract_wrap(raw: &str) -> SeqText {
    let text = js::trim(raw);
    let wrap = if wrap_prefix(text, "wrap").is_some() {
        Some(true)
    } else if wrap_prefix(text, "nowrap").is_some() {
        Some(false)
    } else {
        None
    };
    let text = match wrap {
        None => text,
        // `text.replace(/^:?(?:no)?wrap:/, '')`
        Some(_) => {
            let cut = wrap_prefix(text, "wrap").or_else(|| wrap_prefix(text, "nowrap")).unwrap_or(0);
            &text[cut..]
        }
    };
    SeqText { text: js::trim(text).to_string(), wrap }
}

/// `/[\r  ]/`
fn has_line_terminator(s: &str) -> bool {
    s.contains(['\r', '\u{2028}', '\u{2029}'])
}

/// Splits a box line into a leading color and the rest, as Mermaid's
/// `/^((?:rgba?|hsla?)\s*\(.*\)|\w*)(.*)$/` does. `.` stops at a line terminator, so a line
/// that holds one outside the gap before the parenthesis does not match at all.
fn split_box(s: &str) -> Option<(String, String)> {
    // `/^(?:rgba?|hsla?)\s*\(/`
    let head = ["rgba", "rgb", "hsla", "hsl"].iter().find(|name| s.starts_with(*name)).map(|name| name.len());
    if let Some(name) = head {
        let mut q = name;
        for c in s[name..].chars() {
            if !is_space(c) {
                break;
            }
            q += c.len_utf8();
        }
        if s[q..].starts_with('(') {
            let open_end = q + 1;
            if let Some(close) = s.rfind(')') {
                if close >= open_end {
                    if has_line_terminator(&s[open_end..]) {
                        return None;
                    }
                    return Some((s[..close + 1].to_string(), s[close + 1..].to_string()));
                }
            }
        }
    }
    if has_line_terminator(s) {
        return None;
    }
    let word = s.bytes().take_while(|&b| b.is_ascii_alphanumeric() || b == b'_').count();
    Some((s[..word].to_string(), s[word..].to_string()))
}

/// `Object.entries` of what `JSON.parse` gives, or of a string's characters.
fn entries(value: ConfigValue) -> Vec<(String, ConfigValue)> {
    match value {
        ConfigValue::String(s) => s.chars().enumerate().map(|(i, c)| (i.to_string(), ConfigValue::String(c.to_string()))).collect(),
        ConfigValue::Array(items) => items.into_iter().enumerate().map(|(i, v)| (i.to_string(), v)).collect(),
        ConfigValue::Object(map) => {
            let keys: Vec<String> = map.keys().into_iter().map(str::to_string).collect();
            keys.into_iter().map(|k| (k.clone(), map.get(&k).cloned().unwrap_or_default())).collect()
        }
        _ => Vec::new(),
    }
}

/// How deeply a menu's JSON nests before its deep values are kept as null. `JSON.parse` has
/// no limit; nested values here are held on the native stack. Only the top-level entries are
/// read (a string `class` property), so a deeper value changes nothing that is drawn.
const MENU_DEPTH: usize = 100;

/// `JSON.parse(text)` reduced to its top-level entries, or None where it throws. A value
/// nested deeper than [`MENU_DEPTH`] is kept as null.
fn json_entries(text: &str) -> Option<Vec<(String, ConfigValue)>> {
    match parse_json_limited(text, MENU_DEPTH) {
        Ok(v) => Some(entries(v)),
        Err(JsonError::Invalid) => None,
        Err(JsonError::TooDeep) => {
            let members = json_members(text)?;
            Some(members.into_iter().map(|(key, raw)| (key, parse_json_limited(raw, MENU_DEPTH).unwrap_or(ConfigValue::Null))).collect())
        }
    }
}

fn json_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }
    i
}

/// A JSON string at `i` (at its quote): the position after it.
fn json_string(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'"') {
        return None;
    }
    let mut i = i + 1;
    loop {
        match *b.get(i)? {
            b'"' => return Some(i + 1),
            b'\\' => match *b.get(i + 1)? {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => i += 2,
                b'u' => {
                    if b.len() < i + 6 || !b[i + 2..i + 6].iter().all(u8::is_ascii_hexdigit) {
                        return None;
                    }
                    i += 6;
                }
                _ => return None,
            },
            c if c < 0x20 => return None,
            _ => i += 1,
        }
    }
}

/// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?` at `i`: the position after it.
fn json_number(b: &[u8], mut i: usize) -> Option<usize> {
    let digits = |i: usize| i + b[i.min(b.len())..].iter().take_while(|c| c.is_ascii_digit()).count();
    if b.get(i) == Some(&b'-') {
        i += 1;
    }
    match b.get(i) {
        Some(b'0') => i += 1,
        Some(c) if c.is_ascii_digit() => i = digits(i),
        _ => return None,
    }
    if b.get(i) == Some(&b'.') {
        let end = digits(i + 1);
        if end == i + 1 {
            return None;
        }
        i = end;
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let end = digits(i);
        if end == i {
            return None;
        }
        i = end;
    }
    Some(i)
}

/// One JSON value at `i` (after white space), checked as `JSON.parse` checks it but with an
/// explicit stack, so that nesting takes no native stack: the position after it.
fn json_value(b: &[u8], mut i: usize) -> Option<usize> {
    let mut open: Vec<u8> = Vec::new();
    loop {
        // A value starts here.
        match *b.get(i)? {
            b'{' => {
                i = json_ws(b, i + 1);
                if b.get(i) == Some(&b'}') {
                    i += 1;
                } else {
                    open.push(b'{');
                    i = json_ws(b, json_string(b, i)?);
                    if b.get(i) != Some(&b':') {
                        return None;
                    }
                    i = json_ws(b, i + 1);
                    continue;
                }
            }
            b'[' => {
                i = json_ws(b, i + 1);
                if b.get(i) == Some(&b']') {
                    i += 1;
                } else {
                    open.push(b'[');
                    continue;
                }
            }
            b'"' => i = json_string(b, i)?,
            b't' if b[i..].starts_with(b"true") => i += 4,
            b'f' if b[i..].starts_with(b"false") => i += 5,
            b'n' if b[i..].starts_with(b"null") => i += 4,
            _ => i = json_number(b, i)?,
        }
        // A value ended: close what it ends, up to the next value.
        loop {
            let Some(&kind) = open.last() else { return Some(i) };
            i = json_ws(b, i);
            match (kind, *b.get(i)?) {
                (b'{', b',') => {
                    i = json_ws(b, json_string(b, json_ws(b, i + 1))?);
                    if b.get(i) != Some(&b':') {
                        return None;
                    }
                    i = json_ws(b, i + 1);
                    break;
                }
                (b'[', b',') => {
                    i = json_ws(b, i + 1);
                    break;
                }
                (b'{', b'}') | (b'[', b']') => {
                    open.pop();
                    i += 1;
                }
                _ => return None,
            }
        }
    }
}

/// The members of a top-level JSON object or array that `JSON.parse` accepts, each key (or
/// index) with the text of its value, in `Object.entries` order. Repeated keys keep their
/// first place and their last value.
fn json_members(text: &str) -> Option<Vec<(String, &str)>> {
    let b = text.as_bytes();
    let start = json_ws(b, 0);
    let end = json_value(b, start)?;
    if json_ws(b, end) != b.len() {
        return None;
    }
    let mut members: Vec<(String, &str)> = Vec::new();
    let array = b[start] == b'[';
    if !array && b[start] != b'{' {
        return None;
    }
    let mut i = json_ws(b, start + 1);
    let close = if array { b']' } else { b'}' };
    let mut index = 0usize;
    while b[i] != close {
        let key = if array {
            index += 1;
            (index - 1).to_string()
        } else {
            let k_end = json_string(b, i)?;
            let key = match parse_json_limited(&text[i..k_end], 1) {
                Ok(ConfigValue::String(k)) => k,
                _ => return None,
            };
            i = json_ws(b, json_ws(b, k_end) + 1);
            key
        };
        let v_end = json_value(b, i)?;
        members.push((key, &text[i..v_end]));
        i = json_ws(b, v_end);
        if b[i] == b',' {
            i = json_ws(b, i + 1);
        }
    }
    // `Object.entries` order: array-index keys first, ascending, then the rest as written;
    // a repeated key keeps its first place and takes the last value.
    let mut map: crate::config::ConfigMap = crate::config::ConfigMap::new();
    let mut raw: IndexMap<String, &str> = IndexMap::new();
    for (key, value) in members {
        map.insert(key.clone(), ConfigValue::Null);
        raw.insert(key, value);
    }
    Some(map.keys().into_iter().map(|k| (k.to_string(), raw[k])).collect())
}

/// The model builder (`SeqDb`).
pub(super) struct SeqDb {
    model: SequenceModel,
    prev_actor: Option<String>,
    current_box: Option<usize>,
    last_created: Option<String>,
    last_destroyed: Option<String>,
    wrap_enabled: Option<bool>,
    sequence_wrap: bool,
    active: IndexMap<String, i64>,
}

impl SeqDb {
    pub fn new(wrap: Option<bool>, sequence_wrap: bool) -> Self {
        SeqDb {
            model: SequenceModel::default(),
            prev_actor: None,
            current_box: None,
            last_created: None,
            last_destroyed: None,
            wrap_enabled: wrap,
            sequence_wrap,
            active: IndexMap::new(),
        }
    }

    pub fn finish(self) -> SequenceModel {
        self.model
    }

    fn auto_wrap(&self) -> bool {
        self.wrap_enabled.unwrap_or(self.sequence_wrap)
    }

    pub fn set_acc_title(&mut self, text: &str) {
        self.model.acc_title = Some(js::trim_start(text).to_string());
    }

    pub fn set_acc_description(&mut self, text: &str) {
        self.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(text));
    }

    pub fn set_diagram_title(&mut self, text: String) {
        self.model.title = Some(text);
    }

    pub fn parse_message(&self, s: &str) -> SeqText {
        extract_wrap(s)
    }

    pub fn parse_box_data(&self, s: &str) -> SeqBoxData {
        let found = split_box(s);
        let mut color = match &found {
            Some((c, _)) if !c.is_empty() => js::trim(c).to_string(),
            _ => "transparent".to_string(),
        };
        let mut title = match &found {
            Some((_, t)) if !t.is_empty() => Some(js::trim(t).to_string()),
            _ => None,
        };
        if !is_css_color(&color) {
            color = "transparent".to_string();
            title = Some(js::trim(s).to_string());
        }
        let Some(title) = title else {
            return SeqBoxData { text: None, color, wrap: None };
        };
        let SeqText { text, wrap } = extract_wrap(&title);
        SeqBoxData { text: if text.is_empty() { None } else { Some(text) }, color, wrap }
    }

    fn add_box(&mut self, data: SeqBoxData) {
        let wrap = data.wrap.unwrap_or(self.auto_wrap());
        self.model.boxes.push(SeqBox { name: data.text, wrap, fill: data.color, actor_keys: Vec::new() });
        self.current_box = Some(self.model.boxes.len() - 1);
    }

    fn box_name(&self, index: usize) -> String {
        self.model.boxes[index].name.clone().unwrap_or_else(|| "undefined".to_string())
    }

    fn add_actor(
        &mut self,
        id: &str,
        name: &str,
        description: Option<SeqText>,
        kind: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<(), Error> {
        let mut assigned_box = self.current_box;
        let mut kind: Option<String> = kind.map(str::to_string);
        let mut description = description;
        let mut doc: Option<crate::config::ConfigMap> = None;
        if let Some(metadata) = metadata {
            let text = if metadata.contains('\n') { format!("{}\n", metadata) } else { format!("{{\n{}\n}}", metadata) };
            let parsed = parse_yaml(&text).map_err(|e| {
                Error::syntax(format!("Config of participant \"{}\" is not valid YAML. {}", id, e.message)).with_type(DiagramType::Sequence)
            })?;
            if let ConfigValue::Object(map) = parsed {
                doc = Some(map);
            }
        }
        let doc_type = doc.as_ref().and_then(|d| d.get("type"));
        let doc_alias = doc.as_ref().and_then(|d| d.get("alias"));
        if let Some(t) = doc_type.filter(|t| !t.is_null()) {
            kind = Some(t.to_js_string());
        }
        if let Some(alias) = doc_alias.filter(|a| a.truthy()) {
            if description.as_ref().is_none_or(|d| d.text == name) {
                description = Some(SeqText { text: alias.to_js_string(), wrap: description.as_ref().and_then(|d| d.wrap) });
            }
        }

        if let Some(old) = self.model.actors.get(id) {
            if let (Some(current), Some(old_box)) = (self.current_box, old.box_index) {
                if current != old_box {
                    return Err(fail(format!(
                        "A same participant should only be defined in one Box: {} can't be in '{}' and in '{}' at the same time.",
                        old.name,
                        self.box_name(old_box),
                        self.box_name(current)
                    )));
                }
            }
            assigned_box = if old.box_index.is_some() { old.box_index } else { self.current_box };
            let same = name == old.name && description.is_none();
            if let Some(old) = self.model.actors.get_mut(id) {
                old.box_index = assigned_box;
            }
            if same {
                return Ok(());
            }
        }
        // A participant that is only mentioned in a message has no kind and takes its name as
        // its label.
        if description.is_none() || kind.is_none() {
            description = Some(SeqText { text: name.to_string(), wrap: None });
        }
        let description = description.unwrap_or(SeqText { text: name.to_string(), wrap: None });
        let wrap = description.wrap.unwrap_or(self.auto_wrap());
        self.model.actors.insert(
            id.to_string(),
            SeqActor {
                box_index: assigned_box,
                name: name.to_string(),
                description: description.text,
                wrap,
                prev_actor: self.prev_actor.clone(),
                next_actor: None,
                links: IndexMap::new(),
                properties: IndexMap::new(),
                details: None,
                kind: kind.unwrap_or_else(|| "participant".to_string()),
            },
        );
        if let Some(prev) = self.prev_actor.as_deref().filter(|p| !p.is_empty()) {
            if let Some(prev) = self.model.actors.get_mut(prev) {
                prev.next_actor = Some(id.to_string());
            }
        }
        if let Some(current) = self.current_box {
            self.model.boxes[current].actor_keys.push(id.to_string());
        }
        self.prev_actor = Some(id.to_string());
        Ok(())
    }

    fn push_message(&mut self, from: Option<String>, to: Option<String>, message: SeqContent, wrap: bool, kind: i32) -> &mut SeqMessage {
        let id = self.model.messages.len();
        self.model.messages.push(SeqMessage {
            id,
            from,
            to,
            message,
            wrap,
            kind,
            activate: None,
            placement: None,
            central_connection: None,
        });
        self.model.messages.last_mut().expect("just pushed")
    }

    fn add_signal(
        &mut self,
        from: Option<&str>,
        to: Option<&str>,
        message: Option<&SeqText>,
        kind: i32,
        activate: bool,
        central: Option<i32>,
    ) -> Result<(), Error> {
        // Mermaid recounts every earlier message here. A running count per participant says
        // the same.
        if kind == lt::ACTIVE_END {
            let count = match from.filter(|f| !f.is_empty()) {
                Some(f) => self.active.get(f).copied().unwrap_or(0),
                None => 0,
            };
            if count < 1 {
                return Err(fail(format!("Trying to inactivate an inactive participant ({})", from.unwrap_or("undefined"))));
            }
            self.active.insert(from.unwrap_or_default().to_string(), count - 1);
        } else if kind == lt::ACTIVE_START {
            if let Some(f) = from.filter(|f| !f.is_empty()) {
                let count = self.active.get(f).copied().unwrap_or(0);
                self.active.insert(f.to_string(), count + 1);
            }
        }
        let text = message.map(|m| m.text.clone()).unwrap_or_default();
        let wrap = message.and_then(|m| m.wrap).unwrap_or(self.auto_wrap());
        let m = self.push_message(from.map(str::to_string), to.map(str::to_string), SeqContent::Text(text), wrap, kind);
        m.activate = Some(activate);
        m.central_connection = Some(central.unwrap_or(0));
        Ok(())
    }

    fn add_note(&mut self, actor: NoteActor, placement: i32, message: &SeqText) {
        let wrap = message.wrap.unwrap_or(self.auto_wrap());
        let (from, to) = match &actor {
            NoteActor::One(a) => (a.clone(), a.clone()),
            NoteActor::Two([a, b]) => (a.clone(), b.clone()),
        };
        self.model.notes.push(SeqNote { actor, placement, message: message.text.clone(), wrap });
        let m = self.push_message(Some(from), Some(to), SeqContent::Text(message.text.clone()), wrap, lt::NOTE);
        m.placement = Some(placement);
    }

    // Mermaid logs and ignores menu text that is not valid JSON. It also undoes the `&amp;`
    // and `&equals;` its sanitizer writes; Pele has no sanitizer, and a `;` cannot occur in
    // the text.
    fn add_links(&mut self, actor_id: &str, text: &SeqText) {
        let Some(actor) = self.model.actors.get_mut(actor_id) else { return };
        if let Some(found) = json_entries(&text.text) {
            for (key, value) in found {
                actor.links.insert(key, value);
            }
        }
    }

    fn add_a_link(&mut self, actor_id: &str, text: &SeqText) {
        let Some(actor) = self.model.actors.get_mut(actor_id) else { return };
        let t = &text.text;
        let len = js::len16(t) as i64;
        // `text.indexOf('@')`, in UTF-16 units.
        let sep = t.find('@').map_or(-1, |b| js::len16(&t[..b]) as i64);
        // `text.slice(0, sep - 1)` and `text.slice(sep + 1)`, with JavaScript's negative ends.
        let clamp = |v: i64| -> usize {
            if v < 0 {
                (len + v).max(0) as usize
            } else {
                v.min(len) as usize
            }
        };
        let key = js::trim(js::slice16(t, 0, clamp(sep - 1))).to_string();
        let value = js::trim(js::slice16(t, clamp(sep + 1), len as usize)).to_string();
        actor.links.insert(key, ConfigValue::String(value));
    }

    fn add_properties(&mut self, actor_id: &str, text: &SeqText) {
        let Some(actor) = self.model.actors.get_mut(actor_id) else { return };
        if let Some(found) = json_entries(&text.text) {
            for (key, value) in found {
                actor.properties.insert(key, value);
            }
        }
    }

    // Mermaid reads the properties and links from a page element with this id. Pele keeps
    // the id.
    fn add_details(&mut self, actor_id: &str, text: &SeqText) {
        if let Some(actor) = self.model.actors.get_mut(actor_id) {
            actor.details = Some(text.text.clone());
        }
    }

    /// Applies the statements in order (`apply`). Pele walks nested lists with an explicit
    /// stack; the list here is flat already.
    pub fn apply(&mut self, statements: Vec<Stmt<'_>>) -> Result<(), Error> {
        for statement in statements {
            self.apply_one(statement)?;
        }
        Ok(())
    }

    fn apply_one(&mut self, param: Stmt<'_>) -> Result<(), Error> {
        match param {
            Stmt::SequenceIndex { start, step, visible } => {
                self.push_message(None, None, SeqContent::Numbering { start, step, visible }, false, lt::AUTONUMBER);
            }
            Stmt::Participant { kind: Declare::Add, actor, draw, description, config } => {
                self.add_actor(&actor, &actor, description, draw, config.as_deref())?;
            }
            Stmt::Participant { kind: Declare::Create, actor, draw, description, config } => {
                if self.model.actors.contains_key(actor.as_ref()) {
                    return Err(fail(
                        "It is not possible to have actors with the same id, even if one is destroyed before the next is created. Use 'AS' aliases to simulate the behavior"
                            .to_string(),
                    ));
                }
                self.last_created = Some(actor.to_string());
                self.add_actor(&actor, &actor, description, draw, config.as_deref())?;
                let at = self.model.messages.len();
                self.model.created_actors.insert(actor.into_owned(), at);
            }
            Stmt::Participant { kind: Declare::Destroy, actor, .. } => {
                self.last_destroyed = Some(actor.to_string());
                let at = self.model.messages.len();
                self.model.destroyed_actors.insert(actor.into_owned(), at);
            }
            Stmt::Signal { signal_type, actor } => {
                self.add_signal(Some(&actor), None, None, signal_type, false, None)?;
            }
            Stmt::Note { actor, placement, text } => self.add_note(actor, placement, &text),
            Stmt::Menu { kind, actor, text } => match kind {
                Menu::Links => self.add_links(&actor, &text),
                Menu::ALink => self.add_a_link(&actor, &text),
                Menu::Properties => self.add_properties(&actor, &text),
                Menu::Details => self.add_details(&actor, &text),
            },
            Stmt::Message { from, to, signal_type, msg, activate, central } => {
                if let Some(created) = self.last_created.as_deref().filter(|c| !c.is_empty()) {
                    if to != created {
                        return Err(fail(format!(
                            "The created participant {} does not have an associated creating message after its declaration. Please check the sequence diagram.",
                            created
                        )));
                    }
                    self.last_created = None;
                } else if let Some(destroyed) = self.last_destroyed.as_deref().filter(|d| !d.is_empty()) {
                    if to != destroyed && from != destroyed {
                        return Err(fail(format!(
                            "The destroyed participant {} does not have an associated destroying message after its declaration. Please check the sequence diagram.",
                            destroyed
                        )));
                    }
                    self.last_destroyed = None;
                }
                self.add_signal(Some(&from), Some(&to), Some(&msg), signal_type, activate, central)?;
            }
            Stmt::BoxStart(data) => self.add_box(data),
            Stmt::BoxEnd => self.current_box = None,
            Stmt::Section { signal_type, text } => {
                self.add_signal(None, None, Some(&text), signal_type, false, None)?;
            }
            Stmt::BlockEnd { signal_type } => {
                self.add_signal(None, None, None, signal_type, false, None)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_markers() {
        assert_eq!(extract_wrap("  :wrap: hi "), SeqText { text: "hi".into(), wrap: Some(true) });
        assert_eq!(extract_wrap("nowrap:x"), SeqText { text: "x".into(), wrap: Some(false) });
        assert_eq!(extract_wrap("wrapx: y"), SeqText { text: "wrapx: y".into(), wrap: None });
        assert_eq!(extract_wrap("::wrap:"), SeqText { text: "::wrap:".into(), wrap: None });
    }

    #[test]
    fn box_lines() {
        let db = SeqDb::new(None, false);
        assert_eq!(db.parse_box_data("Aqua Group"), SeqBoxData { text: Some("Group".into()), color: "Aqua".into(), wrap: None });
        assert_eq!(
            db.parse_box_data("rgb(1, 2, 3) The box"),
            SeqBoxData { text: Some("The box".into()), color: "rgb(1, 2, 3)".into(), wrap: None }
        );
        assert_eq!(db.parse_box_data("Group one"), SeqBoxData { text: Some("Group one".into()), color: "transparent".into(), wrap: None });
        assert_eq!(db.parse_box_data("transparent"), SeqBoxData { text: None, color: "transparent".into(), wrap: None });
        assert_eq!(
            db.parse_box_data("rgb(1,2) x"),
            SeqBoxData { text: Some("rgb(1,2) x".into()), color: "transparent".into(), wrap: None }
        );
    }
}
