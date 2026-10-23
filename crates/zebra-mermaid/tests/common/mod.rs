//! Helpers for the differential tests: fixture reading and ordered JSON comparison.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use zebra_mermaid::util::js;
use zebra_mermaid::{ConfigMap, ConfigValue};

pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/pele-1.9.0")
}

pub fn read_jsonl(rel: &str) -> Vec<Value> {
    let current = fixtures().join(rel);
    let path = if current.exists() { current } else { fixtures().parent().unwrap().join(rel) };
    read_jsonl_path(&path)
}

/// Reads a `.jsonl` or gzip-compressed `.jsonl.gz` file.
pub fn read_jsonl_path(path: &std::path::Path) -> Vec<Value> {
    use std::io::Read;
    let text = if path.extension().is_some_and(|e| e == "gz") {
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut s = String::new();
        flate2::read::GzDecoder::new(&bytes[..]).read_to_string(&mut s).expect("gzip");
        s
    } else {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    };
    let mut excluded = 0;
    let rows = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            match serde_json::from_str(line) {
                Ok(value) => Some(value),
                Err(error) => {
                    // These exact upstream inputs contain isolated UTF-16 surrogates. The
                    // public Rust API takes UTF-8, so they are outside its input domain.
                    // Never discard an arbitrary malformed fixture or count an exclusion as a pass.
                    static EXCLUDED: std::sync::OnceLock<std::collections::HashSet<String>> = std::sync::OnceLock::new();
                    let known = EXCLUDED.get_or_init(|| {
                        let manifests = [
                            include_str!("../../../../tests/fixtures/unrepresentable-inputs.json"),
                            include_str!("../../../../tests/fixtures/pele-1.9.0/unrepresentable-inputs.json"),
                        ];
                        manifests
                            .into_iter()
                            .flat_map(|text| {
                                let manifest: Value = serde_json::from_str(text).unwrap();
                                manifest["entries"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|entry| entry["record"].as_str().unwrap().to_owned())
                                    .collect::<Vec<_>>()
                            })
                            .collect()
                    });
                    if known.contains(line.trim()) {
                        excluded += 1;
                        None
                    } else {
                        panic!("{}: malformed fixture: {error}", path.display());
                    }
                }
            }
        })
        .collect();
    if excluded != 0 {
        eprintln!("{}: excluded {excluded} exact non-UTF-8 input records (not passes)", path.display());
    }
    rows
}

pub fn json_to_config(v: &Value) -> ConfigValue {
    match v {
        Value::Null => ConfigValue::Null,
        Value::Bool(b) => ConfigValue::Bool(*b),
        Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap()),
        Value::String(s) => ConfigValue::String(s.clone()),
        Value::Array(a) => ConfigValue::Array(a.iter().map(json_to_config).collect()),
        Value::Object(o) => {
            if let (1, Some(n)) = (o.len(), o.get("$number")) {
                return ConfigValue::Number(number(&serde_json::json!({ "$number": n })));
            }
            let mut m = ConfigMap::new();
            for (k, v) in o {
                m.insert(k.clone(), json_to_config(v));
            }
            ConfigValue::Object(m)
        }
    }
}

/// A count option: a number, or `{"$number": "Infinity"}` for no limit.
fn count(v: &Value) -> Option<usize> {
    if v.is_null() {
        return None;
    }
    let n = number(v);
    Some(if n.is_infinite() && n > 0.0 { usize::MAX } else { n as usize })
}

/// Render options from a fixture's JavaScript option names.
pub fn render_options(o: &Value, icons: &Value) -> zebra_mermaid::RenderOptions {
    let strings = |v: &Value| v.as_array().map(|a| a.iter().map(|s| s.as_str().unwrap().to_string()).collect::<Vec<_>>());
    let mut r = zebra_mermaid::RenderOptions {
        font_family: o["fontFamily"].as_str().map(str::to_string),
        font_family_mono: o["fontFamilyMono"].as_str().map(str::to_string),
        font_size: o["fontSize"].as_f64(),
        id_prefix: o["idPrefix"].as_str().map(str::to_string),
        responsive: o["responsive"].as_bool(),
        max_width: o["maxWidth"].as_f64(),
        auto_direction: o["autoDirection"].as_bool(),
        direction_breakpoint: o["directionBreakpoint"].as_f64(),
        padding: o["padding"].as_f64(),
        limit: count(&o["limit"]),
        output_limit: count(&o["outputLimit"]),
        max_edges: count(&o["maxEdges"]),
        link_schemes: strings(&o["linkSchemes"]),
        image_schemes: strings(&o["imageSchemes"]),
        links: o["links"].as_bool(),
        images: o["images"].as_bool(),
        link_rel: o["linkRel"].as_str().map(str::to_string),
        // `clock(null)` in the JS boundary is epoch zero. Absence instead reads
        // the clock (normalized by the fixture generator); preserve that distinction.
        now: o.get("now").map(|v| if v.is_null() { 0.0 } else { number(v) }),
        info_text: None,
        ..Default::default()
    };
    if let ConfigValue::Object(c) = json_to_config(&o["config"]) {
        r.config = Some(c);
    }
    if let Some(table) = icons.as_object() {
        let mut t = zebra_mermaid::IconTable::new();
        for (name, markup) in table {
            if let Some(m) = markup.as_str() {
                t.insert(name.clone(), m);
            }
        }
        r.icons = Some(std::sync::Arc::new(t));
    }
    r
}

/// A number from a fixture: a JSON number, or `{"$number": "NaN" | "Infinity" | "-Infinity"}`.
pub fn number(v: &Value) -> f64 {
    if let Some(n) = v.as_f64() {
        return n;
    }
    match v.get("$number").and_then(Value::as_str) {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        other => panic!("not a number: {other:?}"),
    }
}

pub fn config_to_json(v: &ConfigValue) -> Value {
    match v {
        ConfigValue::Null => Value::Null,
        ConfigValue::Bool(b) => Value::Bool(*b),
        ConfigValue::Number(n) if n.is_finite() => serde_json::Number::from_f64(*n).map(Value::Number).unwrap(),
        ConfigValue::Number(n) => serde_json::json!({ "$number": js::to_string(*n) }),
        ConfigValue::String(s) => Value::String(s.clone()),
        ConfigValue::Array(a) => Value::Array(a.iter().map(config_to_json).collect()),
        ConfigValue::Object(m) => map_to_json(m),
    }
}

pub fn map_to_json(m: &ConfigMap) -> Value {
    let mut out = serde_json::Map::new();
    for (k, v) in m.iter() {
        out.insert(k.to_string(), config_to_json(v));
    }
    Value::Object(out)
}

/// Compares two JSON values, keys in order and numbers by value.
pub fn json_eq(actual: &Value, expected: &Value, path: &str) -> Result<(), String> {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            if a == b {
                Ok(())
            } else {
                Err(format!("{path}: {a} != {b}"))
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            let ka: Vec<&String> = a.keys().collect();
            let kb: Vec<&String> = b.keys().collect();
            if ka != kb {
                return Err(format!("{path}: keys {ka:?} != {kb:?}"));
            }
            for k in ka {
                json_eq(&a[k], &b[k], &format!("{path}.{k}"))?;
            }
            Ok(())
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Err(format!("{path}: length {} != {}", a.len(), b.len()));
            }
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                json_eq(x, y, &format!("{path}[{i}]"))?;
            }
            Ok(())
        }
        _ if actual == expected => Ok(()),
        _ => Err(format!("{path}: {actual} != {expected}")),
    }
}

/// Inputs of `suite` that the port intentionally answers differently, from
/// `tests/fixtures/known-differences.json`.
pub fn known_differences(suite: &str) -> Vec<String> {
    let path = fixtures().join("known-differences.json");
    let v: Value = serde_json::from_str(&fs::read_to_string(path).expect("known differences")).expect("json");
    v["entries"].as_array().unwrap().iter().filter(|e| e["suite"] == suite).map(|e| e["input"].as_str().unwrap().to_string()).collect()
}

/// Collects failures and reports them together, so one run shows every difference.
#[derive(Default)]
pub struct Failures {
    pub items: Vec<String>,
    pub checked: usize,
}

impl Failures {
    pub fn check(&mut self, ok: Result<(), String>, what: impl FnOnce() -> String) {
        self.checked += 1;
        if let Err(e) = ok {
            self.items.push(format!("{}: {}", what(), e));
        }
    }

    pub fn assert_none(&self, name: &str) {
        if !self.items.is_empty() {
            let shown: Vec<&String> = self.items.iter().take(25).collect();
            panic!(
                "{name}: {} of {} differ:\n{}",
                self.items.len(),
                self.checked,
                shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
            );
        }
        eprintln!("{name}: {} cases match", self.checked);
    }
}

/// Renders a fixture row and compares the result with what Pele gave.
pub fn compare_render(row: &Value) -> Result<(), String> {
    let source = row["source"].as_str().unwrap();
    let expect = &row["expect"];
    let actual = zebra_mermaid::render(source, &render_options(&row["options"], &row["icons"]));
    match (actual, expect["ok"].as_bool().unwrap()) {
        (Ok(r), true) => {
            if r.diagram_type.as_str() != expect["type"].as_str().unwrap() {
                return Err(format!("type {} != {}", r.diagram_type, expect["type"]));
            }
            if r.width != expect["width"].as_f64().unwrap() || r.height != expect["height"].as_f64().unwrap() {
                return Err(format!("size {}x{} != {}x{}", r.width, r.height, expect["width"], expect["height"]));
            }
            let svg = expect["svg"].as_str().unwrap();
            if r.svg != svg {
                let at = r.svg.bytes().zip(svg.bytes()).position(|(a, b)| a != b).unwrap_or(r.svg.len().min(svg.len()));
                let from = at.saturating_sub(60);
                return Err(format!(
                    "svg differs at byte {at}:\n  rust: {}\n  pele: {}",
                    &r.svg[from..(at + 80).min(r.svg.len())],
                    &svg[from..(at + 80).min(svg.len())]
                ));
            }
            let links: Vec<(String, String, bool)> = r.links.iter().map(|l| (l.id.clone(), l.href.clone(), l.internal)).collect();
            let expected: Vec<(String, String, bool)> = expect["links"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| (l["id"].as_str().unwrap().into(), l["href"].as_str().unwrap().into(), l["internal"].as_bool().unwrap()))
                .collect();
            if links != expected {
                return Err(format!("links {links:?} != {expected:?}"));
            }
            Ok(())
        }
        (Err(e), false) => {
            let x = &expect["error"];
            let same = e.code.as_str() == x["code"].as_str().unwrap_or("")
                && e.message == x["message"].as_str().unwrap_or("")
                && e.line as u64 == x["line"].as_u64().unwrap_or(0)
                && e.column as u64 == x["column"].as_u64().unwrap_or(0);
            if same {
                Ok(())
            } else {
                Err(format!("error {:?} {:?} {}:{} != {}", e.code, e.message, e.line, e.column, x))
            }
        }
        (Ok(_), false) => Err(format!("rendered, Pele failed with {}", expect["error"])),
        (Err(e), true) => Err(format!("failed with {:?}: {}", e.code, e.message)),
    }
}

pub mod svg_policy;
pub mod theme;
