//! Current-release output comparison; independent of the original-test 1:1 count.
mod common;

use serde_json::json;
use zebra_mermaid::{render, type_enabled, DiagramType};

#[test]
fn current_release_corpus_matches_svg_geometry_links_and_error_metadata() {
    let metadata: serde_json::Value = serde_json::from_str(include_str!("../../../tests/fixtures/pele-1.9.0.metadata.json")).unwrap();
    assert_eq!(metadata["version"], zebra_mermaid::PELE_VERSION);
    assert_eq!(metadata["commit"], zebra_mermaid::PELE_COMMIT);
    let mut checked = 0;
    for row in common::read_jsonl("pele-1.9.0.jsonl.gz") {
        let expected = &row["expect"];
        let diagram_type = expected["type"].as_str().or(expected["error"]["type"].as_str());
        if diagram_type.and_then(|t| t.parse::<DiagramType>().ok()).is_some_and(|t| !type_enabled(t)) {
            continue;
        }
        let mut options = common::render_options(&row["options"], &row["icons"]);
        options.info_text = None; // Exercise the release's default version, not the historical fixture override.
        let actual = match render(row["source"].as_str().unwrap(), &options) {
            Ok(r) => json!({"ok":true,"type":r.diagram_type,"mermaidType":r.mermaid_type,"width":r.width,"height":r.height,
                "svg":r.svg,"links":r.links.iter().map(|l| json!({"id":l.id,"href":l.href,"internal":l.internal})).collect::<Vec<_>>()}),
            Err(e) => json!({"ok":false,"error":{"code":e.code,"message":e.message,"line":e.line,"column":e.column,
                "snippet":e.snippet,"type":e.diagram_type,"mermaidType":e.mermaid_type}}),
        };
        common::json_eq(&actual, expected, row["id"].as_str().unwrap()).unwrap();
        checked += 1;
    }
    if DiagramType::ALL.into_iter().all(type_enabled) {
        assert_eq!(checked, metadata["cases"].as_u64().unwrap() as usize);
    }
    eprintln!("Pele {}: {checked} complete render responses identical", zebra_mermaid::PELE_VERSION);
}
