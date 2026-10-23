//! Root scans and escaping against independently generated upstream SVG output.
mod common;

use std::io::Read;
use zebra_mermaid::diagrams::RenderCtx;
use zebra_mermaid::svg::root::{svg_document, Accessible};
use zebra_mermaid::text::Icons;
use zebra_mermaid::theme::Theming;
use zebra_mermaid::MetricsMeasurer;

#[test]
fn root_output_matches_upstream_for_markup_fonts_and_accessibility() {
    let mut text = String::new();
    flate2::read::GzDecoder::new(&include_bytes!("../../../tests/fixtures/pele-1.9.0/svg-root.json.gz")[..])
        .read_to_string(&mut text)
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(fixture["commit"], zebra_mermaid::PELE_COMMIT);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 512);
    for (i, case) in cases.iter().enumerate() {
        let options = common::render_options(&case["options"], &serde_json::Value::Null);
        let icons = Icons::new(None);
        let theme = Theming::pele();
        let ctx = RenderCtx::new(&options, options.responsive.unwrap_or(true), &MetricsMeasurer, &icons, &theme);
        let acc = Accessible { acc_title: case["acc"]["accTitle"].as_str(), acc_descr: case["acc"]["accDescr"].as_str() };
        let actual = svg_document("flowchart", 641.25, 480.5, case["size"].as_f64().unwrap(), &ctx, acc, case["content"].as_str().unwrap());
        assert_eq!(actual, case["expected"].as_str().unwrap(), "case {i}");
    }
}
