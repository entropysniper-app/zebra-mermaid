//! Supplementary expectations generated only by pinned Pele's public fontStyle function.
#[test]
fn font_lists_and_styles_match_independent_reference() {
    use zebra_mermaid::svg::theme::{families, font_style, is_font_list};
    let fixture: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../tests/fixtures/unit/font-list-reference.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(fixture["upstreamCommit"], zebra_mermaid::PELE_COMMIT);
    assert_eq!(fixture["seed"], "0x16f017");
    assert_eq!(fixture["generatedCases"], 4096);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 5000);
    for (index, case) in cases.iter().enumerate() {
        let input = case["input"].as_str().unwrap();
        let accepted = case["accepted"].as_bool().unwrap();
        assert_eq!(is_font_list(input), accepted, "case {index}");
        assert_eq!(families(Some(input), "_fallback_"), if accepted { input } else { "_fallback_" }, "case {index}");
        assert_eq!(font_style(16.0, Some(input), Some(input)), case["style"].as_str().unwrap(), "case {index}");
    }
}
