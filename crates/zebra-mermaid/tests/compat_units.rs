//! Differential tests of the core modules against fixtures exported from the pinned Pele
//! snapshot by `tools/compat/gen-fixtures.mjs`.

mod common;

use common::{config_to_json, json_eq, known_differences, map_to_json, number, read_jsonl, Failures};
use serde_json::{json, Value};
use zebra_mermaid::config::yaml::parse_yaml;
use zebra_mermaid::preprocess::{encode_entities, preprocess};
use zebra_mermaid::svg::builder::num;
use zebra_mermaid::svg::theme::resolve_style;
use zebra_mermaid::text::entities::decode_entities;
use zebra_mermaid::text::{layout_label, Icons, Measure, MetricsMeasurer};
use zebra_mermaid::util::math;
use zebra_mermaid::util::url::{safe_url, sanitize_url};

#[test]
fn yaml_matches_pele() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/yaml.jsonl") {
        let source = row["source"].as_str().unwrap();
        let expect = &row["expect"];
        let actual = match parse_yaml(source) {
            Ok(v) => json!({ "ok": true, "value": config_to_json(&v) }),
            Err(e) => json!({ "ok": false, "message": e.message }),
        };
        f.check(json_eq(&actual, expect, ""), || format!("{source:?}"));
    }
    f.assert_none("yaml");
}

#[test]
fn urls_match_pele() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/url.jsonl") {
        let source = row["source"].as_str().unwrap();
        let actual = json!({ "ok": true, "value": sanitize_url(source), "safe": safe_url(source) });
        f.check(json_eq(&actual, &row["expect"], ""), || format!("{source:?}"));
    }
    f.assert_none("url");
}

#[test]
fn entities_match_pele() {
    let mut f = Failures::default();
    let known = known_differences("entities");
    for row in read_jsonl("unit/entities.jsonl") {
        let source = row["source"].as_str().unwrap();
        if known.iter().any(|k| k == source) {
            continue;
        }
        let actual = Value::String(decode_entities(source));
        f.check(json_eq(&actual, &row["expect"], ""), || format!("{source:?}"));
    }
    f.assert_none("entities");
}

#[test]
fn coordinates_match_pele() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/num.jsonl") {
        let v = number(&row["value"]);
        let actual = num(v).to_string();
        let expect = row["expect"].as_str().unwrap();
        f.check(if actual == expect { Ok(()) } else { Err(format!("{actual} != {expect}")) }, || format!("{v:e}"));
    }
    f.assert_none("num");
}

#[test]
fn math_matches_v8_bit_for_bit() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/math.jsonl") {
        let name = row["fn"].as_str().unwrap();
        let args: Vec<f64> = row["args"].as_array().unwrap().iter().map(number).collect();
        let r = match name {
            "sin" => math::sin(args[0]),
            "cos" => math::cos(args[0]),
            "tan" => math::tan(args[0]),
            "atan" => math::atan(args[0]),
            "asin" => math::asin(args[0]),
            "acos" => math::acos(args[0]),
            "exp" => math::exp(args[0]),
            "log" => math::log(args[0]),
            "log10" => math::log10(args[0]),
            "log2" => math::log2(args[0]),
            "atan2" => math::atan2(args[0], args[1]),
            "pow" => math::pow(args[0], args[1]),
            "hypot" => math::hypot(args[0], args[1]),
            other => panic!("unknown {other}"),
        };
        let actual = format!("{:016x}", r.to_bits());
        let expect = row["expect"].as_str().unwrap();
        // Every NaN prints as NaN in SVG; only its presence matters.
        let same = actual == expect || (r.is_nan() && f64::from_bits(u64::from_str_radix(expect, 16).unwrap()).is_nan());
        f.check(if same { Ok(()) } else { Err(format!("{actual} != {expect}")) }, || format!("{name}{args:?}"));
    }
    f.assert_none("math");
}

#[test]
fn labels_match_pele() {
    let icons = Icons::none();
    let m = Measure::new(&MetricsMeasurer, &icons);
    let mut f = Failures::default();
    for row in read_jsonl("unit/label.jsonl") {
        let source = row["source"].as_str().unwrap();
        let markdown = row["markdown"].as_bool().unwrap();
        let size = row["size"].as_f64().unwrap();
        let max = row["max"].as_f64().unwrap();
        let base = row["base"].as_u64().unwrap() as u8;
        let l = layout_label(Some(source), markdown, &m, size, max, base);
        let lines: Vec<Value> = l
            .lines
            .iter()
            .map(|line| {
                Value::Array(
                    line.iter()
                        .map(|s| {
                            let mut o = serde_json::Map::new();
                            o.insert("text".into(), json!(s.text));
                            o.insert("style".into(), json!(s.style));
                            if let Some(icon) = &s.icon {
                                o.insert("icon".into(), json!(icon));
                            }
                            o.insert("width".into(), json!(s.width));
                            Value::Object(o)
                        })
                        .collect(),
                )
            })
            .collect();
        let actual = json!({ "lines": lines, "widths": l.widths, "width": l.width, "height": l.height, "size": l.size, "lineHeight": l.line_height });
        let mut expect = row["expect"].clone();
        // Pele writes each span's keys in the order they were set; compare without key order.
        normalize_spans(&mut expect);
        let mut actual = actual;
        normalize_spans(&mut actual);
        f.check(json_eq(&actual, &expect, ""), || format!("{source:?} md={markdown} max={max} base={base}"));
    }
    f.assert_none("label");
}

fn normalize_spans(v: &mut Value) {
    if let Some(lines) = v.get_mut("lines").and_then(Value::as_array_mut) {
        for line in lines {
            for span in line.as_array_mut().unwrap() {
                let o = span.as_object().unwrap();
                let mut sorted: Vec<(String, Value)> = o.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                *span = Value::Object(sorted.into_iter().collect());
            }
        }
    }
}

#[test]
fn preprocessing_matches_pele() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/preprocess.jsonl") {
        let source = row["source"].as_str().unwrap();
        let actual = match preprocess(source) {
            Ok(p) => json!({
                "ok": true,
                "text": p.text,
                "title": p.title,
                "config": map_to_json(&p.config),
                "withComments": p.with_comments,
                "lineOffset": p.line_offset,
                "encoded": encode_entities(&p.text).0,
            }),
            Err(e) => json!({ "ok": false, "message": e.message, "code": e.code.as_str() }),
        };
        f.check(json_eq(&actual, &row["expect"], ""), || format!("{source:?}"));
    }
    f.assert_none("preprocess");
}

#[test]
fn styles_match_pele() {
    let mut f = Failures::default();
    for row in read_jsonl("unit/style.jsonl") {
        let decls: Vec<String> = row["declarations"].as_array().unwrap().iter().map(|d| d.as_str().unwrap().to_string()).collect();
        let s = resolve_style(&decls);
        let mut o = serde_json::Map::new();
        o.insert("shape".into(), json!(s.shape));
        o.insert("line".into(), json!(s.line));
        o.insert("text".into(), json!(s.text));
        if let Some(stroke) = &s.stroke {
            o.insert("stroke".into(), json!(stroke));
        }
        o.insert("bold".into(), json!(s.bold));
        o.insert("italic".into(), json!(s.italic));
        if let Some(size) = s.font_size {
            o.insert("fontSize".into(), json!(size));
        }
        let mut expect = row["expect"].clone();
        // JSON.stringify leaves out undefined members.
        if let Some(e) = expect.as_object_mut() {
            e.retain(|_, v| !v.is_null());
        }
        f.check(json_eq(&Value::Object(o), &expect, ""), || format!("{decls:?}"));
    }
    f.assert_none("style");
}
