//! Original label-ink assertions, including the complete 148-name table and call sequence.
use serde_json::Value;
use std::sync::LazyLock;
use zebra_mermaid::svg::{ink::ink, theme::resolve_style};
static DATA: LazyLock<Value> = LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/ink.json")).unwrap());
fn check(id: &str) {
    let record = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    let mut slots = Vec::new();
    for action in record["actions"].as_array().unwrap() {
        match action["op"].as_str().unwrap() {
            "ink" => {
                let decls: Vec<_> = action["declarations"].as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect();
                let style = resolve_style(&decls);
                assert_eq!(action["slot"].as_u64().unwrap() as usize, slots.len());
                slots.push(ink(&style.shape, &style.text));
            }
            "length" => assert_eq!(action["actual"].as_array().unwrap().len(), action["expected"].as_u64().unwrap() as usize),
            "equal" => {
                let value = |v: &Value| {
                    if let Some(slot) = v["$ink"].as_u64() {
                        slots[slot as usize].map(str::to_owned)
                    } else if v["$undefined"] == true {
                        None
                    } else {
                        v.as_str().map(str::to_owned)
                    }
                };
                assert_eq!(value(&action["actual"]), value(&action["expected"]), "{}", record["name"]);
            }
            other => panic!("Unknown ink operation {other}"),
        }
    }
}
include!("ink_generated.rs");
