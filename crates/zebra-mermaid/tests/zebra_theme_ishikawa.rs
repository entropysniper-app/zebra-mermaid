#![cfg(feature = "ishikawa")]

//! Zebra theme roles of the Ishikawa (fishbone) diagram in resolved mode
//! (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! The spine and the bones are lines (`edge`) and their arrowheads arrows (`accent`). The head
//! (the effect) and the category boxes are nodes: `surface` filled, `border` outlined, as Pele
//! fills them with its surface color. Category and first-level cause text takes the text color;
//! deeper causes the secondary text color. Drawn down the page for a narrow host, the outline's
//! connectors are lines too.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const FISH: &str = "---
title: Fish
---
ishikawa
Blurry photo
  Process
    Out of focus
      Lens dirty
        Smudge
  User
    Shaky hands
  Light
";

/// Every element (start tag) that starts with `start`.
fn tags<'a>(svg: &'a str, start: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(start) {
        let at = from + k;
        let end = svg[at..].find('>').unwrap() + at + 1;
        out.push(&svg[at..end]);
        from = end;
    }
    out
}

fn narrow(p: &MermaidPalette) -> RenderOptions {
    RenderOptions { max_width: Some(300.0), ..resolved_options(Some(p.clone())) }
}

#[test]
fn ishikawa_consumes_its_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let r = render(FISH, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("style=\"max-width:100%;height:auto;color:{}", p.text)));
    assert!(svg.contains(&format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        r.width, r.height, p.background
    )));
    assert!(svg.contains("<text class=\"pele-title\" font-weight=\"bold\""));

    // Spine and bones are lines, every arrowhead an arrow.
    assert_eq!(t.line, p.edge.clone().unwrap());
    assert!(tags(svg, "<path class=\"pele-ishikawa-spine\"")[0].ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", t.line)));
    let bones = tags(svg, "<path class=\"pele-ishikawa-bones\"");
    assert_eq!(bones.len(), 3);
    for b in &bones {
        assert!(b.ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", t.line)), "{b}");
    }
    let markers = tags(svg, "<path class=\"pele-marker\"");
    assert_eq!(markers.len(), 4);
    for m in &markers {
        assert!(m.ends_with(&format!("fill=\"{}\" stroke=\"none\"/>", p.accent)), "{m}");
    }

    // Category boxes and the head are nodes.
    let boxes = tags(svg, "<rect x=");
    assert_eq!(boxes.len(), 3);
    for b in &boxes {
        assert!(b.ends_with(&format!("rx=\"4\" fill=\"{}\" stroke=\"{}\"/>", p.surface, p.border)), "{b}");
    }
    assert!(svg.contains(&format!("H0Z\" fill=\"{}\" stroke=\"{}\" stroke-linejoin=\"round\"/>", p.surface, p.border)));
    // Category and head labels inherit the text color; first-level causes too, deeper ones are
    // secondary.
    assert!(svg.contains("<text class=\"pele-label\" x="));
    assert!(svg.contains("<text x=\"") && svg.contains(">Out of focus</text>"));
    for deep in ["Lens dirty", "Smudge"] {
        let at = svg.find(&format!(">{deep}</text>")).unwrap();
        let open = svg[..at].rfind("<text").unwrap();
        assert!(svg[open..at].starts_with(&format!("<text fill=\"{}\"", p.secondary_text)), "{deep}");
    }
    let at = svg.find(">Shaky hands</text>").unwrap();
    assert!(!svg[svg[..at].rfind("<text").unwrap()..at].contains("fill="));

    // Roles an Ishikawa diagram does not use must not leak in.
    let series = p.series.clone().unwrap();
    let mut unused = vec![
        p.surface_secondary.clone(),
        p.selection.clone(),
        p.warning_background.clone(),
        p.warning_text.clone(),
        p.danger_background.clone(),
        p.danger_text.clone(),
        p.success_background.clone(),
        p.success_text.clone(),
        p.group_background.clone().unwrap(),
        p.group_border.clone().unwrap(),
    ];
    unused.extend(series);
    for color in &unused {
        assert!(!svg.contains(color.as_str()), "ishikawa uses {color}");
    }
}

#[test]
fn the_outline_down_the_page_keeps_the_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let r = render(FISH, &narrow(&p)).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains("pele-node pele-ishikawa-cause"), "drawn down the page");
    let edges = tags(svg, "<path class=\"pele-edge\"");
    assert_eq!(edges.len(), 7);
    for e in &edges {
        assert!(e.ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", t.line)), "{e}");
    }
    let boxes = tags(svg, "<rect x=");
    assert_eq!(boxes.len(), 4);
    assert!(boxes[0].ends_with(&format!("rx=\"12\" fill=\"{}\" stroke=\"{}\"/>", p.surface, p.border)));
    for b in &boxes[1..] {
        assert!(b.ends_with(&format!("rx=\"4\" fill=\"{}\" stroke=\"{}\"/>", p.surface, p.border)), "{b}");
    }
    for unused in [&p.accent, &p.secondary_text, &p.surface_secondary, &p.selection] {
        assert!(!svg.contains(unused.as_str()), "the outline uses {unused}");
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for options in [resolved_options(Some(p.clone())), narrow(&p)] {
        let opaque = render(FISH, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..options.clone() }).unwrap();
        let clear = render(FISH, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..options }).unwrap();
        assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
        let rect = format!(
            "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            opaque.width, opaque.height, p.background
        );
        assert!(opaque.svg.contains(&rect));
        assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
    }
}

#[test]
fn palettes_change_colors_only() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    let c = MermaidPalette::light();
    for wide in [true, false] {
        let options = |p: &MermaidPalette| if wide { resolved_options(Some(p.clone())) } else { narrow(p) };
        let ra = render(FISH, &options(&a)).unwrap();
        let rb = render(FISH, &options(&b)).unwrap();
        let rc = render(FISH, &options(&c)).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        assert_eq!(without_colors(&ra.svg), without_colors(&rc.svg));
        assert_no_css_runtime(&rb.svg);
        assert_no_css_runtime(&rc.svg);
    }
    // CSS mode stays Pele's.
    let css = render(FISH, &RenderOptions::default()).unwrap();
    assert!(css.svg.contains("fill=\"var(--_s)\" stroke=\"var(--_b)\""));
    assert!(css.svg.contains("stroke=\"var(--_l)\""));
    assert!(css.svg.contains("<text fill=\"var(--_m)\""));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for dark in [false, true] {
        let mode = if dark { MermaidThemeMode::Dark } else { MermaidThemeMode::Light };
        let r = render_for_zebra(FISH, &ZebraRenderOptions { theme_mode: mode, ..Default::default() }).unwrap();
        svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{e}"));
        assert_no_css_runtime(&r.svg);
    }
}
