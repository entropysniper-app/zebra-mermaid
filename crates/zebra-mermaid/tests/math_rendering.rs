//! Default RaTeX integration is an intentional host-API difference from upstream Pele.
//! These tests do not stand in for Pele's host callback tests.
mod common;
use zebra_mermaid::{render, RenderOptions};
fn well_formed(svg: &str) {
    let mut r = quick_xml::Reader::from_str(svg);
    loop {
        match r.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => panic!("invalid SVG: {e}"),
            _ => {}
        }
    }
}
fn visible_text(svg: &str) -> String {
    let mut r = quick_xml::Reader::from_str(svg);
    let mut out = String::new();
    loop {
        match r.read_event().unwrap() {
            quick_xml::events::Event::Eof => break,
            quick_xml::events::Event::Text(t) => out.push_str(&t.xml10_content()),
            _ => {}
        }
    }
    out
}
#[test]
#[cfg(all(
    feature = "flowchart",
    feature = "sequence",
    feature = "state",
    feature = "class",
    feature = "er",
    feature = "mindmap",
    feature = "kanban",
    feature = "block",
    feature = "requirement",
    feature = "usecase",
    feature = "swimlane"
))]
fn math_is_built_in_for_all_eleven_supported_types() {
    let cases=[
        ("flowchart LR\n  A[\"$$\\frac{a}{b}$$\"] -->|\"$$x^2$$\"| B",2),
        ("sequenceDiagram\n  participant A as $$\\alpha$$\n  A->>B: Solve $$x^2$$\n  Note over A: $$\\int x$$",4),
        ("swimlane-beta LR\n  subgraph L[\"$$\\lambda$$\"]\n    A[\"$$a$$\"]\n  end",2),
        ("stateDiagram-v2\n  state \"$$a$$\" as s1\n  s1 : $$b$$\n  [*] --> s1 : $$c$$\n  note right of s1 : $$d$$\n  state \"$$e$$\" as C {\n    x --> y\n  }",5),
        ("classDiagram\n  class A {\n    <<$$a$$>>\n  }\n  note for A \"$$b$$\"\n  A \"$$c$$\" --> \"1\" B : $$d$$\n  class C[\"$$e$$\"]",5),
        ("erDiagram\n  p[\"$$a$$\"] {\n    string name \"$$b$$\"\n  }\n  p ||--o{ q : \"$$c$$\"",3),
        ("mindmap\n  root((\"$$a$$\"))\n    $$b$$",2),
        ("kanban\n  todo[\"$$a$$\"]\n    t1[\"$$b$$\"]",2),
        ("block-beta\n  a[\"$$a$$\"] b[\"$$b$$\"]\n  a -- \"$$c$$\" --> b",3),
        ("requirementDiagram\n  requirement \"$$a$$\" {\n    id: \"$$b$$\"\n    text: \"$$c$$\"\n  }\n  element e1 {\n    type: \"$$d$$\"\n    docref: \"$$e$$\"\n  }",5),
        ("usecase-beta\n  actor C(\"$$a$$\")\n  systemBoundary \"$$b$$\"\n    U(\"$$c$$\")\n  end\n  C --> U",3),
    ];
    for (source, count) in cases {
        for native in [false, true] {
            let options = if native {
                RenderOptions { color_mode: zebra_mermaid::ColorMode::Resolved, ..Default::default() }
            } else {
                RenderOptions::default()
            };
            let drawn = render(source, &options).unwrap();
            assert_eq!(drawn.svg.matches("class=\"pele-math\"").count(), count, "{source}");
            assert!(!visible_text(&drawn.svg).contains("$$"), "{source}");
            if !native {
                assert!(drawn.svg.contains("currentColor"));
            }
            assert!(!drawn.svg.contains("KaTeX_"));
            if native {
                assert!(!drawn.svg.contains("var(--"));
                assert!(!drawn.svg.contains("currentColor"));
            }
            well_formed(&drawn.svg);
        }
    }
}
#[cfg(feature = "flowchart")]
#[test]
fn formulas_are_atomic_and_invalid_tex_falls_back_to_source() {
    let source = "flowchart TD\n  A[\"Words $$\\frac{1}{1+\\frac{1}{x}}$$<br/>below\"] --> B[\"$$\\notACommand$$\"]";
    let drawn = render(source, &RenderOptions { max_width: Some(180.0), ..Default::default() }).unwrap();
    assert_eq!(drawn.svg.matches("class=\"pele-math\"").count(), 1);
    assert!(visible_text(&drawn.svg).contains("$$\\notACommand$$"));
    assert!(visible_text(&drawn.svg).contains("below"));
    well_formed(&drawn.svg);
    assert_eq!(drawn, render(source, &RenderOptions { max_width: Some(180.0), ..Default::default() }).unwrap());
}
#[cfg(all(feature = "flowchart", feature = "sequence", feature = "class", feature = "pie"))]
#[test]
fn titles_class_members_and_other_types_stay_text() {
    for body in ["flowchart LR\n  A", "sequenceDiagram\n  A->>B: hi", "classDiagram\n  class A"] {
        let svg = render(&format!("---\ntitle: $$t$$\n---\n{body}"), &RenderOptions::default()).unwrap().svg;
        assert!(visible_text(&svg).contains("$$t$$"));
        assert!(!svg.contains("pele-math"));
    }
    let svg = render("classDiagram\n  class A {\n    +$$m$$ int\n  }\n  A : $$n$$ text", &RenderOptions::default()).unwrap().svg;
    let text = visible_text(&svg);
    assert!(text.contains("$$m$$ int"));
    assert!(text.contains("$$n$$ text"));
    assert!(!svg.contains("pele-math"));
    let svg = render("pie\n  \"$$x$$\": 3", &RenderOptions::default()).unwrap().svg;
    assert!(visible_text(&svg).contains("$$x$$"));
    assert!(!svg.contains("pele-math"));
}
#[cfg(feature = "flowchart")]
#[test]
fn math_follows_author_text_color_and_opacity_without_spacing_styles() {
    let svg = render(
        "flowchart LR\n  A[\"Hide $$x+\\color{black}{y}$$\"]\n  style A color:#c00,opacity:0,letter-spacing:6px,word-spacing:4px",
        &RenderOptions::default(),
    )
    .unwrap()
    .svg;
    let start = svg.find("<svg class=\"pele-math\"").unwrap();
    let end = start + svg[start..].find('>').unwrap();
    let outer = &svg[start..end];
    assert!(outer.contains(" color=\"#c00\""));
    assert!(outer.contains(" opacity=\"0\""));
    assert!(!outer.contains("spacing"));
    assert!(svg.contains("rgb(0,0,0)"));
    well_formed(&svg);
}

#[test]
fn public_label_layout_uses_builtin_math_and_can_keep_titles_plain() {
    use zebra_mermaid::text::{layout_label, Icons, Measure, MetricsMeasurer};
    let icons = Icons::none();
    let measure = Measure::new(&MetricsMeasurer, &icons);
    let label = layout_label(Some("Value $$x^2$$"), false, &measure, 16.0, 4000.0, 0);
    assert!(label.lines[0].iter().any(|span| span.math.is_some()));
    let plain = layout_label(Some("Value $$x^2$$"), false, &measure.without_math(), 16.0, 4000.0, 0);
    assert!(plain.lines[0].iter().all(|span| span.math.is_none()));
    assert_eq!(plain.lines[0].iter().map(|s| s.text.as_str()).collect::<String>(), "Value $$x^2$$");
}

#[cfg(feature = "flowchart")]
#[test]
fn math_respects_the_native_image_policy() {
    let options = RenderOptions { images: Some(false), color_mode: zebra_mermaid::ColorMode::Resolved, ..Default::default() };
    let svg = render("flowchart LR\n  A[\"$$\\text{🙂}$$\"]", &options).unwrap().svg;
    assert!(!svg.contains("<image"));
    assert!(!svg.contains("data:image"));
    well_formed(&svg);
}
