#![cfg(all(feature = "flowchart", feature = "swimlane"))]

//! Theme and native-output regression tests for the compound/lanes integration.
mod common;
use common::{
    svg_policy,
    theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors},
};
use zebra_mermaid::zebra::{render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, RenderOptions};

const FLOW: &str = "flowchart LR\nsubgraph group[编辑流程]\n A[草稿] -->|审核| B{发布?}\nend\n B --> C[完成]\n";

#[test]
fn independently_routes_node_edge_arrow_and_group_colors() {
    let p = sentinel();
    let r = render(FLOW, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    assert!(r.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.surface, p.border)));
    assert!(r.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.group_background.as_ref().unwrap(), p.group_border.as_ref().unwrap())));
    assert!(r.svg.contains(&format!("class=\"pele-edges\" fill=\"none\" stroke=\"{}\"", p.edge.as_ref().unwrap())));
    assert!(r.svg.contains(&format!("fill=\"{}\"", p.accent)), "arrowheads use the accent");
    assert!(r.svg.contains(&format!("class=\"pele-cluster-label\" fill=\"{}\"", p.secondary_text)));
    for color in [&p.warning_text, &p.danger_text, &p.success_text, &p.selection] {
        assert!(!r.svg.contains(color));
    }
}

#[test]
fn palette_switch_and_canvas_do_not_change_geometry() {
    for src in [FLOW, "swimlane-beta LR\nsubgraph one[One]\n A --> B\nend\nsubgraph two[Two]\n C\nend\nB --> C\n"] {
        let p = sentinel();
        let mut q = p.clone();
        q.surface = "#123456".into();
        q.group_background = Some("#345678".into());
        q.edge = Some("#456789".into());
        let options = resolved_options(Some(p));
        let first = render(src, &options).unwrap();
        let second = render(src, &resolved_options(Some(q))).unwrap();
        assert_ne!(first.svg, second.svg);
        assert_eq!(without_colors(&first.svg), without_colors(&second.svg));
        let clear = render(src, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..options.clone() }).unwrap();
        assert_eq!((first.width, first.height), (clear.width, clear.height));
        assert!(!clear.svg.contains("data-mermaid-canvas-background"));
        assert!(clear.svg.contains("pele-label"));
        assert_eq!(render(src, &options).unwrap(), first);
        assert_no_css_runtime(&clear.svg);
    }
}

#[test]
fn author_styles_override_the_corresponding_role() {
    let src = "flowchart LR\nA --> B\nstyle A fill:#112233,stroke:#223344,color:#334455\nlinkStyle 0 stroke:#445566,stroke-width:4px\n";
    let r = render(src, &resolved_options(Some(sentinel()))).unwrap();
    assert!(r.svg.contains("fill:#112233"));
    assert!(r.svg.contains("stroke:#223344"));
    assert!(r.svg.contains("fill:#334455"));
    assert!(r.svg.contains("stroke:#445566"));
    assert_no_css_runtime(&r.svg);
}

#[test]
fn native_profile_suppresses_external_links_and_images() {
    let source = "flowchart LR\nA --> B\nclick A \"https://example.com\"\n";
    let r = render_for_zebra(source, &ZebraRenderOptions { palette: Some(sentinel()), ..Default::default() }).unwrap();
    svg_policy::validate(&r.svg, 4 * 1024 * 1024).unwrap();
    assert!(!r.svg.contains("<a "));
    assert_no_css_runtime(&r.svg);
}
