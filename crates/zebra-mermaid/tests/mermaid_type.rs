use zebra_mermaid::detect::detect_diagram;
use zebra_mermaid::{DiagramType, MermaidType};

#[test]
fn type_names_preserve_detector_precedence_and_railroad_case_rules() {
    for (source, diagram_type, mermaid_type) in [
        ("\u{3000}FLOWCHART TD", None, None),
        ("flowchart TD\n A[C4Container]", Some(DiagramType::C4), Some(MermaidType::C4)),
        ("flowchart-elk TD\n A[C4Container]", Some(DiagramType::Flowchart), Some(MermaidType::FlowchartElk)),
        ("\u{feff}Railroad-EBNF-beta", Some(DiagramType::Railroad), Some(MermaidType::RailroadEbnf)),
        ("RAILROAD-abnf-BETA", Some(DiagramType::Railroad), Some(MermaidType::RailroadAbnf)),
        ("railroad-PEG-beta", Some(DiagramType::Railroad), Some(MermaidType::RailroadPeg)),
    ] {
        let found = detect_diagram(source);
        assert_eq!(found.map(|d| d.diagram_type), diagram_type, "{source}");
        assert_eq!(found.map(|d| d.mermaid_type), mermaid_type, "{source}");
    }
    for name in MermaidType::ALL {
        assert_eq!(name.as_str().parse::<MermaidType>(), Ok(name));
        assert_eq!(serde_json::to_value(name).unwrap(), name.as_str());
    }
}

#[cfg(not(feature = "flowchart"))]
#[test]
fn disabled_diagram_error_retains_both_detected_names() {
    let error = zebra_mermaid::parse("flowchart-elk TD\n A --> B", &Default::default()).unwrap_err();
    assert_eq!(error.code, zebra_mermaid::ErrorCode::UnsupportedDiagram);
    assert_eq!(error.diagram_type, Some(DiagramType::Flowchart));
    assert_eq!(error.mermaid_type, Some(MermaidType::FlowchartElk));
}

#[cfg(feature = "flowchart")]
#[test]
fn cached_parses_renders_and_errors_keep_mermaid_type() {
    let mut cache = zebra_mermaid::RenderCache::new(8, 1024 * 1024);
    for _ in 0..2 {
        let model = cache.parse("flowchart-elk TD\n A --> B", &Default::default());
        assert_eq!(model.as_ref().as_ref().unwrap().mermaid_type(), MermaidType::FlowchartElk);
        let result = cache.render("flowchart-elk TD\n A --> B", &Default::default()).unwrap();
        assert_eq!(result.mermaid_type, MermaidType::FlowchartElk);
        let error = cache.parse("flowchart LR\n A -->", &Default::default());
        assert_eq!(error.as_ref().as_ref().unwrap_err().mermaid_type, Some(MermaidType::FlowchartV2));
    }
    assert!(cache.stats().parse_hits > 0);
    assert!(cache.stats().render_hits > 0);
}
