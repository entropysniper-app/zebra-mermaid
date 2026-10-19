#![cfg(all(feature = "state", feature = "usecase", feature = "agentflow"))]
//! Theme roles and native host policy for the final three diagram families.
mod common;
use common::{
    svg_policy,
    theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors},
};
use zebra_mermaid::zebra::{render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, RenderOptions};
const STATE: &str = "stateDiagram-v2\nstate Outer {\n A --> B\n}\n[*] --> Outer\nnote right of B : Note\n";
const USECASE: &str = "usecase-beta\nactor A(User)\nsystemBoundary S[System]\n U(Login)\nend\nA --> U\nnote for U \"Note\"\n";
const AGENT: &str =
    "agentflow-beta TB\nflow scope[Pipeline]\n A[\"Input\"]@{ shape: input }\n B[\"Task\"]\n A --> B\nend\nB --> C[\"Finish\"]\n";
#[test]
fn host_groups_edges_and_arrows_remain_independent() {
    let p = sentinel();
    for source in [STATE, USECASE, AGENT] {
        let out = render(source, &resolved_options(Some(p.clone()))).unwrap();
        assert_no_css_runtime(&out.svg);
        assert!(
            out.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.group_background.as_ref().unwrap(), p.group_border.as_ref().unwrap())),
            "group roles: {source}"
        );
        assert!(
            out.svg.contains(&format!("class=\"pele-edges\" fill=\"none\" stroke=\"{}\"", p.edge.as_ref().unwrap())),
            "edge role: {source}"
        );
        assert!(out.svg.contains(&format!("fill=\"{}\"", p.accent)), "arrow role: {source}");
    }
}
#[test]
fn notes_and_agent_kinds_consume_the_existing_palette() {
    let p = sentinel();
    for source in [STATE, USECASE] {
        let out = render(source, &resolved_options(Some(p.clone()))).unwrap();
        assert!(out.svg.contains(&format!("fill=\"{}\"", p.warning_background)));
        assert!(out.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.surface, p.border)));
    }
    let agent = render(AGENT, &resolved_options(Some(p.clone()))).unwrap();
    assert!(agent.svg.contains("pele-kind-input"));
    assert!(agent.svg.contains("pele-kind-task"));
    assert!(agent.svg.contains(&format!("fill=\"{}\"", p.series.as_ref().unwrap()[1])));
    assert!(agent.svg.contains(&format!("fill=\"{}\"", p.series.as_ref().unwrap()[3])));
}
#[test]
fn palette_changes_preserve_geometry_and_transparent_canvas() {
    for source in [STATE, USECASE, AGENT] {
        let p = sentinel();
        let mut q = p.clone();
        q.surface = "#123456".into();
        q.group_background = Some("#345678".into());
        q.edge = Some("#456789".into());
        q.series.as_mut().unwrap()[1] = "#556677".into();
        let first = render(source, &resolved_options(Some(p.clone()))).unwrap();
        let second = render(source, &resolved_options(Some(q))).unwrap();
        assert_ne!(first.svg, second.svg);
        assert_eq!(without_colors(&first.svg), without_colors(&second.svg));
        let clear =
            render(source, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p)) }).unwrap();
        assert_eq!((clear.width, clear.height), (first.width, first.height));
        assert!(!clear.svg.contains("data-mermaid-canvas-background"));
        assert_no_css_runtime(&clear.svg);
    }
}
#[test]
fn each_family_passes_native_svg_policy() {
    for source in [STATE, USECASE, AGENT] {
        let out = render_for_zebra(source, &ZebraRenderOptions { palette: Some(sentinel()), ..Default::default() }).unwrap();
        svg_policy::validate(&out.svg, 4 * 1024 * 1024).unwrap();
        assert_no_css_runtime(&out.svg);
    }
}
