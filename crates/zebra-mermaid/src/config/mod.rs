//! Diagram configuration: values, the YAML reader for front matter, and JSON for directives.

pub mod json;
pub mod value;
pub mod yaml;

pub use value::{Config, ConfigMap, ConfigValue};

const UNSAFE_KEYS: [&str; 3] = ["__proto__", "constructor", "prototype"];

/// Deep-merges `source` into `target`, as Pele's `merge` does: objects merge key by key, and
/// anything else replaces what was there. Keys that could reach a JavaScript prototype are
/// skipped, so that configuration means the same in both implementations.
pub fn merge(target: &mut ConfigMap, source: &ConfigMap) {
    for (key, value) in source.iter() {
        if UNSAFE_KEYS.contains(&key) {
            continue;
        }
        match value {
            ConfigValue::Object(src) => {
                if let Some(ConfigValue::Object(current)) = target.get_mut(key) {
                    merge(current, src);
                } else {
                    let mut fresh = ConfigMap::new();
                    merge(&mut fresh, src);
                    target.insert(key, ConfigValue::Object(fresh));
                }
            }
            other => target.insert(key, other.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_deeply_and_skips_prototype_keys() {
        let a = json::parse_json(r#"{"pie":{"a":1,"b":{"c":2}},"x":1}"#).unwrap();
        let b = json::parse_json(r#"{"pie":{"b":{"d":3}},"x":{"y":1},"__proto__":{"z":1}}"#).unwrap();
        let mut target = a.as_object().unwrap().clone();
        merge(&mut target, b.as_object().unwrap());
        let pie = target.section("pie").unwrap();
        assert_eq!(pie.get("a").unwrap().as_f64(), Some(1.0));
        let inner = pie.section("b").unwrap();
        assert_eq!(inner.keys(), vec!["c", "d"]);
        assert!(target.section("x").is_some());
        assert!(!target.contains_key("__proto__"));
    }
}
