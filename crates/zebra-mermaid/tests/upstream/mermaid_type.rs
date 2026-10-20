//! 1:1 port: Pele 1.7.0 tests/render/mermaid-type.test.ts (MIT, kepano 2026).
use zebra_mermaid::{detect_type, parse, render, render_parsed, ErrorCode, ParseOptions, RenderOptions};

fn reports(source: &str, diagram_type: &str, mermaid_type: &str) {
    assert_eq!(detect_type(source).unwrap().unwrap().as_str(), diagram_type);
    let result = render(source, &RenderOptions::default()).unwrap();
    assert_eq!(result.diagram_type.as_str(), diagram_type);
    assert_eq!(result.mermaid_type.as_str(), mermaid_type);
    let model = parse(source, &ParseOptions::default()).unwrap();
    assert_eq!(model.mermaid_type().as_str(), mermaid_type);
    let result = render_parsed(&model, &RenderOptions::default()).unwrap();
    assert_eq!(result.diagram_type.as_str(), diagram_type);
    assert_eq!(result.mermaid_type.as_str(), mermaid_type);
}

fn railroad_with(header: &str) -> String {
    let corpus: Vec<String> =
        serde_json::from_str(include_str!("../../../../tests/upstream/source/tests/corpus/railroad-docs.json")).unwrap();
    corpus.into_iter().find(|source| source.starts_with(&format!("{header}\n"))).unwrap()
}

#[test]
fn reports_case_00() {
    reports("graph TD\n  A --> B", "flowchart", "flowchart-v2");
}
#[test]
fn reports_case_01() {
    reports("flowchart LR\n  A --> B", "flowchart", "flowchart-v2");
}
#[test]
fn reports_case_02() {
    reports("flowchart-elk TD\n  A --> B", "flowchart", "flowchart-elk");
}
#[test]
fn reports_case_03() {
    reports("classDiagram\n  class A", "class", "classDiagram");
}
#[test]
fn reports_case_04() {
    reports("classDiagram-v2\n  class A", "class", "classDiagram");
}
#[test]
fn reports_case_05() {
    reports("stateDiagram\n  [*] --> A", "state", "stateDiagram");
}
#[test]
fn reports_case_06() {
    reports("stateDiagram-v2\n  [*] --> A", "state", "stateDiagram");
}
#[test]
fn reports_case_07() {
    reports(&railroad_with("railroad-beta"), "railroad", "railroad");
}
#[test]
fn reports_case_08() {
    reports(&railroad_with("railroad-ebnf-beta"), "railroad", "railroadEbnf");
}
#[test]
fn reports_case_09() {
    reports(&railroad_with("railroad-abnf-beta"), "railroad", "railroadAbnf");
}
#[test]
fn reports_case_10() {
    reports(&railroad_with("railroad-peg-beta"), "railroad", "railroadPeg");
}
#[test]
fn reports_case_11() {
    reports("sequenceDiagram\n  A->>B: Hi", "sequence", "sequence");
}
#[test]
fn reports_case_12() {
    reports("pie\n  \"a\": 1", "pie", "pie");
}

#[test]
fn finds_the_type_after_front_matter_directives_and_comments() {
    for (source, expected) in [
        ("---\ntitle: T\n---\nclassDiagram\n  class A", "classDiagram"),
        ("%%{init: {\"theme\": \"dark\"}}%%\ngraph TD\n  A --> B", "flowchart-v2"),
        ("%% note\nstateDiagram-v2\n  [*] --> A", "stateDiagram"),
    ] {
        assert_eq!(render(source, &RenderOptions::default()).unwrap().mermaid_type.as_str(), expected);
    }
}

#[test]
fn leaves_the_svg_named_by_peles_type() {
    let svg = render("graph TD\n  A --> B", &RenderOptions::default()).unwrap().svg;
    assert!(svg.contains("class=\"pele pele-flowchart\""));
    assert!(svg.contains("aria-roledescription=\"flowchart\""));
}

#[test]
fn is_on_errors_once_the_type_is_known() {
    let error = render("flowchart LR\n  A -->", &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, ErrorCode::Syntax);
    assert_eq!(error.diagram_type.unwrap().as_str(), "flowchart");
    assert_eq!(error.mermaid_type.unwrap().as_str(), "flowchart-v2");
    let error = render("classDiagram\n  class A", &RenderOptions { output_limit: Some(10), ..Default::default() }).unwrap_err();
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!(error.diagram_type.unwrap().as_str(), "class");
    assert_eq!(error.mermaid_type.unwrap().as_str(), "classDiagram");
    let error = render("not a diagram", &RenderOptions::default()).unwrap_err();
    assert_eq!(error.diagram_type, None);
    assert_eq!(error.mermaid_type, None);
    let error = parse("graph TD\n  A", &ParseOptions { limit: Some(5), ..Default::default() }).unwrap_err();
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!(error.mermaid_type, None);
}
