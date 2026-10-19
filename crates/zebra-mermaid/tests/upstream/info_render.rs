//! 1:1 port: Pele tests/render/info.test.ts (MIT, kepano 2026).
use zebra_mermaid::diagrams::info::InfoModel;
use zebra_mermaid::{parse, render, supports, DiagramModel, DiagramType, ParseOptions, RenderOptions};

#[test]
fn draws_peles_version() {
    let result = render("info", &RenderOptions::default()).unwrap();
    assert_eq!(result.diagram_type, DiagramType::Info);
    assert!(supports("info showInfo"));
    assert!(result.svg.contains(&format!(">Pele v{}<", zebra_mermaid::PELE_VERSION)));
    assert!(result.svg.contains("class=\"pele-version\""));
    assert!(result.width < 200.0);
    assert!(result.height < 60.0);
}
#[test]
fn draws_the_same_with_and_without_show_info() {
    assert_eq!(render("info\nshowInfo\n", &RenderOptions::default()).unwrap().svg, render("info", &RenderOptions::default()).unwrap().svg);
}
#[test]
fn draws_a_title_from_the_diagram_or_front_matter() {
    let titled = render("---\ntitle: From front matter\n---\ninfo", &RenderOptions::default()).unwrap();
    assert!(titled.svg.contains(">From front matter<"));
    assert!(titled.height > render("info", &RenderOptions::default()).unwrap().height);
    let own = render("---\ntitle: From front matter\n---\ninfo\ntitle Own", &RenderOptions::default()).unwrap().svg;
    assert!(own.contains(">Own<"));
    assert!(!own.contains("From front matter"));
}
#[test]
fn writes_accessible_names_and_exposes_the_model() {
    let src = "info showInfo\naccTitle: Name\naccDescr: Text";
    let svg = render(src, &RenderOptions::default()).unwrap().svg;
    assert!(svg.contains("<title id=\"pele-title\">Name</title>"));
    assert!(svg.contains("<desc id=\"pele-desc\">Text</desc>"));
    let parsed = parse(src, &ParseOptions::default()).unwrap();
    assert_eq!(parsed.diagram_type(), DiagramType::Info);
    let DiagramModel::Info(model) = parsed.model() else { panic!("expected Info model") };
    assert_eq!(model, &InfoModel { title: None, acc_title: Some("Name".into()), acc_descr: Some("Text".into()) });
}
#[test]
fn rejects_anything_else_after_the_keyword() {
    assert!(render("info unsupported", &RenderOptions::default()).unwrap_err().message.contains("Lexical error on line 1"));
    assert!(render("info\ntitle a\nshowInfo", &RenderOptions::default()).unwrap_err().message.contains("Parse error on line 3"));
}
