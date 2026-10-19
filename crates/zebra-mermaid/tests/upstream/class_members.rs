//! 1:1 projection of the original ClassMember display-detail assertions.
use super::Member;
use serde_json::Value;
use std::sync::LazyLock;
fn check(id: &str) {
    static DATA: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/class-members.json")).unwrap());
    let r = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    for a in r["assertions"].as_array().unwrap() {
        let member = Member::new(a["input"].as_str().unwrap(), a["kind"] == "method");
        let actual = match a["key"].as_str().unwrap() {
            "displayText" => member.display(),
            // The native member keeps the parsed classifier; Pele's adapter exposes its
            // CSS spelling. This is a representation projection, not an expected value.
            "cssStyle" => match member.classifier.as_str() {
                "*" => "font-style:italic;",
                "$" => "text-decoration:underline;",
                _ => "",
            }
            .to_owned(),
            other => panic!("Unsupported member observation {other}"),
        };
        assert_eq!(actual, a["expected"].as_str().unwrap(), "{}", r["name"]);
    }
}
include!("class_members_generated.rs");
