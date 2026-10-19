//! Direct reference assertions from Pele util/url.test.ts and util/yaml.test.ts (MIT).
use serde_json::Value;
use std::{sync::LazyLock, time::Instant};
use zebra_mermaid::config::{json::parse_json, yaml::parse_yaml, ConfigMap, ConfigValue};
static DATA: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/utilities.json")).unwrap());
fn check(id: &str) {
    let record = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    for assertion in record["assertions"].as_array().unwrap() {
        let source = assertion["source"].as_str().unwrap();
        match assertion["op"].as_str().unwrap() {
            "url" => assert_eq!(zebra_mermaid::util::url::sanitize_url(source), assertion["expected"].as_str().unwrap(), "{source:?}"),
            "yaml" => assert_eq!(parse_yaml(source).unwrap(), parse_json(&assertion["expected"].to_string()).unwrap(), "{source:?}"),
            other => panic!("Unsupported utility assertion {other}"),
        }
    }
}
include!("utilities_generated.rs");

#[test]
fn yaml_plain_object_does_not_touch_prototypes() {
    // ConfigMap is an owned, prototype-free map: the typed Object variant is the language
    // equivalent of Object.getPrototypeOf(value) === Object.prototype. No global prototype exists.
    let value = parse_yaml("{ __proto__: { polluted: true } }").unwrap();
    assert!(matches!(value, ConfigValue::Object(_)));
    assert_eq!(ConfigMap::new().get("polluted"), None);
}
#[test]
fn yaml_refuses_absurd_nesting() {
    let error = parse_yaml(&format!("{{ a: {} }}", "[".repeat(20000))).unwrap_err();
    assert!(error.message.contains("too deep"));
}
#[test]
fn yaml_unterminated_collection_is_linear() {
    let started = Instant::now();
    assert!(parse_yaml(&format!("a: {{\n{}", " b: 1,\n".repeat(20000))).is_err());
    assert!(started.elapsed().as_secs_f64() * 1000.0 < 500.0);
}
#[test]
fn yaml_rejects_unterminated_flow_mapping() {
    assert!(parse_yaml("{ a: 1").is_err());
    // The independent js-yaml assertion is executed by compile-utilities.mjs and frozen here.
    assert_eq!(DATA["referenceChecks"]["unterminatedYamlThrows"], true);
}
