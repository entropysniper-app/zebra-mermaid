//! Independent upstream cases for declaration order, escaping and unsafe-value filtering.
mod common;
use serde_json::json;
use std::io::Read;
use zebra_mermaid::svg::theme::resolve_style;

#[test]
fn author_styles_preserve_upstream_values_order_and_filtering() {
    let mut text = String::new();
    flate2::read::GzDecoder::new(&include_bytes!("../../../tests/fixtures/style-regression.json.gz")[..])
        .read_to_string(&mut text)
        .unwrap();
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["commit"], zebra_mermaid::PELE_COMMIT);
    assert_eq!(data["seed"], "0x57a1e");
    assert_eq!(data["randomCases"], 2048);
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2902);
    for (index, case) in cases.iter().enumerate() {
        let declarations: Vec<_> = case["declarations"].as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect();
        let actual = resolve_style(&declarations);
        let mut actual = json!({"shape":actual.shape,"line":actual.line,"text":actual.text,"stroke":actual.stroke,
            "bold":actual.bold,"italic":actual.italic,"fontSize":actual.font_size});
        actual.as_object_mut().unwrap().retain(|_, v| !v.is_null());
        common::json_eq(&actual, &case["expected"], &format!("case {index}: {declarations:?}")).unwrap();
    }
}
