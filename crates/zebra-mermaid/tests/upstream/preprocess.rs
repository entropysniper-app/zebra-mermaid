//! 1:1 port of Pele tests/util/preprocess.test.ts (MIT).
use super::*;
use serde_json::Value;
use std::{sync::LazyLock, time::Instant};
static DATA: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/utilities.json")).unwrap());
fn assertions(index: usize) -> &'static [Value] {
    DATA["records"].as_array().unwrap().iter().find(|r| r["file"] == "tests/util/preprocess.test.ts" && r["index"] == index).unwrap()
        ["assertions"]
        .as_array()
        .unwrap()
}
#[test]
fn entities_0() {
    let items = assertions(0);
    assert_eq!(items.len(), 10);
    for a in items {
        assert_eq!(encode_entities(a["source"].as_str().unwrap()).0, a["expected"].as_str().unwrap());
    }
}
#[test]
fn entities_1() {
    let mut state = 11_u32;
    let mut rnd = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state as f64 / 4294967296.0
    };
    let parts = ["style", "classDef", ":", "#", ";", " ", "\n", "a", "fill", "9", "\t", ":#", "#f9f;", "\r"];
    let items = assertions(1);
    assert_eq!(items.len(), 20000);
    for a in items {
        let mut text = String::new();
        let count = (rnd() * 14.0).floor() as usize;
        for _ in 0..count {
            text.push_str(parts[(rnd() * parts.len() as f64).floor() as usize]);
        }
        assert_eq!(text, a["source"].as_str().unwrap(), "seed and call order must match the original test");
        assert_eq!(encode_entities(&text).0, a["expected"].as_str().unwrap(), "{text:?}");
    }
}
#[test]
fn entities_stays_fast() {
    let started = Instant::now();
    encode_entities(&format!("graph TD\nstyle A {}", "style:x#".repeat(20000)));
    encode_entities(&format!("classDef {}", ":#".repeat(50000)));
    assert!(started.elapsed().as_secs_f64() * 1000.0 < 500.0);
}
#[test]
fn removes_comments_keeps_directives() {
    assert_eq!(cleanup_comments("%% one\ngraph TD\n  %% two\n  A --> B\n").0, "graph TD\n  A --> B\n");
    assert_eq!(cleanup_comments("%%{init: {}}%%\ngraph TD").0, "%%{init: {}}%%\ngraph TD");
}
#[test]
fn front_matter_title_config() {
    let result = preprocess("---\ntitle: My chart\nconfig:\n  flowchart:\n    curve: linear\n---\ngraph TD\n  A").unwrap();
    assert_eq!(result.title.as_deref(), Some("My chart"));
    assert_eq!(ConfigValue::Object(result.config), json::parse_json(r#"{"flowchart":{"curve":"linear"}}"#).unwrap());
    assert_eq!(result.text, "graph TD\n  A");
}
#[test]
fn directives_override_front_matter() {
    let result=preprocess("---\nconfig:\n  flowchart:\n    curve: linear\n    rankSpacing: 10\n---\n%%{init: {'flowchart': {'rankSpacing': 99}}}%%\ngraph TD\n  A").unwrap();
    assert_eq!(ConfigValue::Object(result.config), json::parse_json(r#"{"flowchart":{"curve":"linear","rankSpacing":99}}"#).unwrap());
    assert_eq!(result.text, "graph TD\n  A");
}
#[test]
fn config_does_not_reach_prototypes() {
    preprocess("%%{init: {\"__proto__\": {\"polluted\": true}, \"constructor\": {\"prototype\": {\"polluted\": true}}}}%%\ngraph TD\nA")
        .unwrap();
    preprocess("---\nconfig:\n  __proto__:\n    polluted: true\n---\ngraph TD\nA").unwrap();
    // Rust maps have no shared mutable prototype. A fresh object's lookup remains absent.
    assert_eq!(ConfigMap::new().get("polluted"), None);
}
#[test]
fn normalizes_endings_and_attribute_quotes() {
    assert_eq!(
        preprocess("graph TD\r\nA[\"<span class=\"x\">hi</span>\"]\r").unwrap().text,
        "graph TD\nA[\"<span class='x'>hi</span>\"]\n"
    );
}
#[test]
fn unclosed_tags_linear_time() {
    let started = Instant::now();
    preprocess(&format!("graph TD\nA[\"{}\"]", "<a".repeat(50000))).unwrap();
    assert!(started.elapsed().as_secs_f64() * 1000.0 < 200.0);
}
