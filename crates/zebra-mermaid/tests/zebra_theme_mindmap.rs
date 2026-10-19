//! Zebra theme roles of mind maps in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! Zebra's renderer (`apply_layout_config`) gives a mind map: a root filled with 16% of the
//! accent over the background and labeled in the text color; links in the line (edge) color;
//! section fills from the chart palette when the caller gave series colors, with labels in a
//! color that reads on them, and otherwise a tonal scale of the accent with labels in the text
//! color. Pele draws the root with the surface and border colors and each branch in a series
//! color; CSS mode keeps that, and resolved mode follows Zebra.

#![cfg(feature = "mindmap")]

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::{contrasting_text_on_fill, mix_hex};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, ColorMode, IconTable, MermaidPalette, MermaidThemeMode, RenderOptions};

/// Zebra's reference fixture (`renders_every_supported_diagram_family_in_both_themes`).
const ZEBRA: &str = "mindmap\n root((Zebra))\n  Editor\n  Search";

/// A root, four branches (a box, a pill that branches on, a hexagon with an icon, a cloud), and
/// plain text leaves.
const MAP: &str = "mindmap\n  root((Zebra))\n    A[Alpha]\n      a1\n      a2(round)\n    B\n      b1\n    C{{Gamma}}\n      ::icon(star)\n    D)Delta(\n";

fn with_icons(mut options: RenderOptions) -> RenderOptions {
    let mut icons = IconTable::new();
    icons.insert("star", "<circle cx=\"12\" cy=\"12\" r=\"4\"/>");
    options.icons = Some(std::sync::Arc::new(icons));
    options
}

/// The group of the node with `data-id="id"`, from its opening tag to its end. Node groups
/// hold no other groups.
fn node<'a>(svg: &'a str, id: &str) -> &'a str {
    let at = svg.find(&format!("data-id=\"{id}\"")).unwrap_or_else(|| panic!("no node {id}"));
    let start = svg[..at].rfind("<g ").unwrap();
    let end = at + svg[at..].find("</g>").unwrap();
    &svg[start..end]
}

/// The opening tag of a node's group.
fn group_tag<'a>(svg: &'a str, id: &str) -> &'a str {
    let n = node(svg, id);
    &n[..n.find('>').unwrap() + 1]
}

/// Every value of the attribute that follows `prefix`.
fn attrs_after<'a>(svg: &'a str, prefix: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(prefix) {
        let start = from + k + prefix.len();
        let end = svg[start..].find('"').map(|e| start + e).unwrap();
        out.push(&svg[start..end]);
        from = end;
    }
    out
}

#[test]
fn mindmap_consumes_its_roles() {
    let p = sentinel();
    let series = p.series.clone().unwrap();
    let r = render(MAP, &with_icons(resolved_options(Some(p.clone())))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("fill=\"{}\"/>", p.background)), "opaque canvas");

    // Links: every branch in the line color, none in a series color.
    let strokes = attrs_after(svg, "class=\"pele-edges\" fill=\"none\" stroke=\"");
    assert_eq!(strokes.len(), 4);
    assert!(strokes.iter().all(|s| *s == p.edge.as_deref().unwrap()), "{strokes:?}");

    // The root: a light mix of the accent, outlined in the border color, labeled in the text color.
    let root = node(svg, "root");
    let root_fill = mix_hex(&p.accent, &p.background, 0.16);
    assert!(root.contains("<circle r=\""), "{root}");
    assert!(root.contains(&format!("fill=\"{}\" stroke=\"{}\"/>", root_fill, p.border)), "{root}");
    assert!(group_tag(svg, "root").ends_with(&format!(" fill=\"{0}\" color=\"{0}\">", p.text)));

    // Sections take the series in order, outlined in the line color, with labels that read on them.
    let edge = p.edge.clone().unwrap();
    for (id, k) in [("A", 0), ("a2", 0), ("B", 1), ("C", 2), ("D", 3)] {
        let n = node(svg, id);
        assert!(n.contains(&format!("fill=\"{}\" stroke=\"{}\"/>", series[k], edge)), "{id}: {n}");
        assert!(!n.contains("fill-opacity"), "{id} is filled opaquely");
        let label = contrasting_text_on_fill(&series[k], &p.text, &p.background, 1.0);
        assert!(group_tag(svg, id).ends_with(&format!(" fill=\"{0}\" color=\"{0}\">", label)), "{id}");
    }
    // A dark section takes the canvas color for its label, a light one the text color.
    assert!(group_tag(svg, "B").contains(&format!("fill=\"{}\"", p.background)));
    assert!(group_tag(svg, "D").contains(&format!("fill=\"{}\"", p.text)));
    // The icon of a section node draws in its label color.
    assert!(node(svg, "C").contains("class=\"pele-icon\" data-icon=\"star\""));
    // Plain text leaves sit on the canvas and keep the inherited text color.
    for id in ["a1", "b1"] {
        assert!(!group_tag(svg, id).contains("fill="), "{id}");
    }

    // Roles a mind map does not use.
    for unused in [
        &p.surface,
        &p.surface_secondary,
        &p.secondary_text,
        &p.accent,
        &p.selection,
        &p.warning_background,
        &p.warning_text,
        &p.danger_background,
        &p.danger_text,
        &p.success_background,
        &p.success_text,
        p.group_background.as_ref().unwrap(),
        p.group_border.as_ref().unwrap(),
        &series[4],
        &series[5],
    ] {
        assert!(!svg.contains(unused.as_str()), "mindmap uses {unused}");
    }
}

#[test]
fn without_series_sections_take_a_tonal_scale_of_the_accent() {
    let mut p = sentinel();
    p.series = None;
    let r = render(MAP, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    let edge = p.edge.clone().unwrap();
    for (id, k) in [("A", 0.0f32), ("B", 1.0), ("C", 2.0), ("D", 3.0)] {
        let fill = mix_hex(&p.accent, &p.background, 0.04 + (0.13 - 0.04) * (k / 11.0));
        let n = node(&r.svg, id);
        assert!(n.contains(&format!("fill=\"{}\" stroke=\"{}\"/>", fill, edge)), "{id}: {n}");
        assert!(group_tag(&r.svg, id).ends_with(&format!(" fill=\"{0}\" color=\"{0}\">", p.text)), "{id}");
    }
}

#[test]
fn an_outline_running_down_uses_the_same_roles() {
    let p = sentinel();
    let across = render(MAP, &resolved_options(Some(p.clone()))).unwrap();
    let down = render(MAP, &RenderOptions { max_width: Some(260.0), ..resolved_options(Some(p.clone())) }).unwrap();
    assert!(down.width < across.width);
    assert!(down.svg.contains("Q"), "links turn in to their node");
    assert_no_css_runtime(&down.svg);
    let strokes = attrs_after(&down.svg, "class=\"pele-edges\" fill=\"none\" stroke=\"");
    assert!(!strokes.is_empty() && strokes.iter().all(|s| *s == p.edge.as_deref().unwrap()));
    assert!(node(&down.svg, "root").contains(&format!("stroke=\"{}\"", p.border)));
}

#[test]
fn only_colors_change_with_the_palette() {
    let a = sentinel();
    let b = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    let mut c = sentinel();
    c.series = None;
    let renders: Vec<_> = [&a, &b, &c].iter().map(|p| render(MAP, &resolved_options(Some((*p).clone()))).unwrap()).collect();
    for r in &renders[1..] {
        assert_eq!((r.width, r.height), (renders[0].width, renders[0].height));
        assert_eq!(without_colors(&r.svg), without_colors(&renders[0].svg));
    }
    assert_ne!(renders[0].svg, renders[1].svg);
    // CSS output has the same size.
    let css = render(MAP, &RenderOptions::default()).unwrap();
    assert_eq!((css.width, css.height), (renders[0].width, renders[0].height));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(MAP, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear = render(MAP, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
        .unwrap();
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
    // The background still serves as the label color on dark sections.
    assert!(group_tag(&clear.svg, "B").contains(&format!("fill=\"{}\"", p.background)));
}

#[test]
fn zebra_reference_fixture_in_light_and_dark() {
    let mut sizes = Vec::new();
    let mut svgs = Vec::new();
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(ZEBRA, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert!(r.width > 0.0 && r.height > 0.0);
        assert!(r.svg.contains("<svg"));
        assert!(!r.svg.contains("color-mix("));
        assert!(!r.svg.contains("@import"));
        assert_no_css_runtime(&r.svg);
        let defaults = MermaidPalette::default_for(theme);
        assert!(r.svg.contains(&mix_hex(&defaults.accent, &defaults.background, 0.16)), "root fill");
        sizes.push((r.width, r.height));
        svgs.push(r.svg);

        let z = render_for_zebra(ZEBRA, &ZebraRenderOptions { theme_mode: theme, ..Default::default() }).unwrap();
        svg_policy::validate(&z.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap();
        assert_no_css_runtime(&z.svg);
    }
    assert_ne!(svgs[0], svgs[1]);
    assert_eq!(sizes[0], sizes[1]);
}
