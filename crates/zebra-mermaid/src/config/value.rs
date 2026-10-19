//! Configuration values: what YAML front matter and `%%{init}%%` directives produce.
//!
//! Pele keeps configuration as plain JavaScript objects. [`ConfigValue`] keeps the same kinds
//! of value, including the NaN and infinities that the YAML reader produces, and
//! [`ConfigMap`] iterates its keys in JavaScript's order: array-index keys first, ascending,
//! then the others in insertion order.

use indexmap::IndexMap;

use crate::util::js;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ConfigValue {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(ConfigMap),
}

impl ConfigValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&ConfigMap> {
        match self {
            ConfigValue::Object(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            ConfigValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }

    /// A property of an object value; `None` for anything else.
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.as_object().and_then(|m| m.get(key))
    }

    /// JavaScript truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            ConfigValue::Null => false,
            ConfigValue::Bool(b) => *b,
            ConfigValue::Number(n) => *n != 0.0 && !n.is_nan(),
            ConfigValue::String(s) => !s.is_empty(),
            ConfigValue::Array(_) | ConfigValue::Object(_) => true,
        }
    }

    /// `String(value)`.
    pub fn to_js_string(&self) -> String {
        match self {
            ConfigValue::Null => "null".into(),
            ConfigValue::Bool(b) => b.to_string(),
            ConfigValue::Number(n) => js::to_string(*n),
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Array(items) => items
                .iter()
                .map(|v| match v {
                    // Array.prototype.join writes null and undefined as nothing.
                    ConfigValue::Null => String::new(),
                    other => other.to_js_string(),
                })
                .collect::<Vec<_>>()
                .join(","),
            ConfigValue::Object(_) => "[object Object]".into(),
        }
    }

    /// `String(value)`, taking the text of a string value instead of copying it.
    pub fn into_js_string(self) -> String {
        match self {
            ConfigValue::String(s) => s,
            other => other.to_js_string(),
        }
    }

    /// Compact JSON with keys in JavaScript order; NaN and infinities as `{"$number":"..."}`.
    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        write_json(self, &mut out);
        out
    }

    /// `typeof value` for the kinds a configuration holds.
    pub fn type_of(&self) -> &'static str {
        match self {
            ConfigValue::Null | ConfigValue::Array(_) | ConfigValue::Object(_) => "object",
            ConfigValue::Bool(_) => "boolean",
            ConfigValue::Number(_) => "number",
            ConfigValue::String(_) => "string",
        }
    }
}

fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_json(v: &ConfigValue, out: &mut String) {
    match v {
        ConfigValue::Null => out.push_str("null"),
        ConfigValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        ConfigValue::Number(n) if n.is_finite() => out.push_str(&js::to_string(*n)),
        ConfigValue::Number(n) => {
            out.push_str("{\"$number\":");
            write_json_string(&js::to_string(*n), out);
            out.push('}');
        }
        ConfigValue::String(s) => write_json_string(s, out),
        ConfigValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json(item, out);
            }
            out.push(']');
        }
        ConfigValue::Object(m) => {
            out.push('{');
            for (i, (k, v)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_string(k, out);
                out.push(':');
                write_json(v, out);
            }
            out.push('}');
        }
    }
}

/// Whether a key is an array index, which JavaScript lists before other keys.
fn array_index(key: &str) -> Option<u32> {
    if key.is_empty() || key.len() > 10 {
        return None;
    }
    if key.len() > 1 && key.starts_with('0') {
        return None;
    }
    if !key.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = key.parse().ok()?;
    if n < 4_294_967_295 {
        Some(n as u32)
    } else {
        None
    }
}

/// An object of configuration values.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMap {
    entries: IndexMap<String, ConfigValue>,
}

impl ConfigMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.entries.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut ConfigValue> {
        self.entries.get_mut(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Sets a property. An existing key keeps its place, as with a JavaScript object.
    pub fn insert(&mut self, key: impl Into<String>, value: ConfigValue) {
        self.entries.insert(key.into(), value);
    }

    pub fn remove(&mut self, key: &str) -> Option<ConfigValue> {
        self.entries.shift_remove(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The keys in the order `Object.keys` gives them.
    pub fn keys(&self) -> Vec<&str> {
        let mut indexed: Vec<(u32, &str)> = Vec::new();
        let mut named: Vec<&str> = Vec::new();
        for key in self.entries.keys() {
            match array_index(key) {
                Some(n) => indexed.push((n, key.as_str())),
                None => named.push(key.as_str()),
            }
        }
        if indexed.is_empty() {
            return named;
        }
        indexed.sort_by_key(|&(n, _)| n);
        let mut out: Vec<&str> = indexed.into_iter().map(|(_, k)| k).collect();
        out.extend(named);
        out
    }

    /// Entries in the order `Object.entries` gives them.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ConfigValue)> {
        self.keys().into_iter().map(move |k| (k, &self.entries[k]))
    }

    /// The object stored under `key`, if that value is an object.
    pub fn section(&self, key: &str) -> Option<&ConfigMap> {
        self.get(key).and_then(ConfigValue::as_object)
    }

    /// `config[section]?.[key]` for an object section.
    pub fn lookup(&self, section: &str, key: &str) -> Option<&ConfigValue> {
        self.section(section).and_then(|s| s.get(key))
    }
}

impl FromIterator<(String, ConfigValue)> for ConfigMap {
    fn from_iter<I: IntoIterator<Item = (String, ConfigValue)>>(iter: I) -> Self {
        let mut map = ConfigMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

/// The configuration of a diagram: front matter `config`, init directives and host values.
pub type Config = ConfigMap;

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{ConfigMap, ConfigValue};
    use serde::ser::{SerializeMap, SerializeSeq};
    use serde::{Serialize, Serializer};

    impl Serialize for ConfigValue {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            match self {
                ConfigValue::Null => s.serialize_unit(),
                ConfigValue::Bool(b) => s.serialize_bool(*b),
                // JSON has no NaN or infinity; those are written as a tagged string.
                ConfigValue::Number(n) if n.is_finite() => s.serialize_f64(*n),
                ConfigValue::Number(n) => {
                    let mut m = s.serialize_map(Some(1))?;
                    m.serialize_entry("$number", &crate::util::js::to_string(*n))?;
                    m.end()
                }
                ConfigValue::String(v) => s.serialize_str(v),
                ConfigValue::Array(items) => {
                    let mut seq = s.serialize_seq(Some(items.len()))?;
                    for item in items {
                        seq.serialize_element(item)?;
                    }
                    seq.end()
                }
                ConfigValue::Object(map) => map.serialize(s),
            }
        }
    }

    impl Serialize for ConfigMap {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(Some(self.len()))?;
            for (k, v) in self.iter() {
                m.serialize_entry(k, v)?;
            }
            m.end()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn javascript_key_order() {
        let mut m = ConfigMap::new();
        m.insert("b", ConfigValue::Null);
        m.insert("10", ConfigValue::Null);
        m.insert("a", ConfigValue::Null);
        m.insert("2", ConfigValue::Null);
        m.insert("01", ConfigValue::Null);
        m.insert("4294967295", ConfigValue::Null);
        assert_eq!(m.keys(), vec!["2", "10", "b", "a", "01", "4294967295"]);
        m.insert("b", ConfigValue::Bool(true));
        assert_eq!(m.keys()[2], "b");
    }

    #[test]
    fn strings_and_truth() {
        assert_eq!(ConfigValue::Number(1.5).to_js_string(), "1.5");
        assert_eq!(
            ConfigValue::Array(vec![ConfigValue::Number(1.0), ConfigValue::Null, ConfigValue::String("x".into())]).to_js_string(),
            "1,,x"
        );
        assert!(!ConfigValue::Number(f64::NAN).truthy());
        assert!(ConfigValue::Object(ConfigMap::new()).truthy());
    }
}
