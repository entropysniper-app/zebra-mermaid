#![cfg(feature = "cynefin")]

//! Zebra theme roles of the Cynefin diagram in resolved mode
//! (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! The four domains are tinted with the chart palette in Pele's order (Complex, Complicated,
//! Clear, Chaotic take series 1, 2, 3 and 4; Confusion series 5), the caller's series first.
//! The dashed boundaries between domains and around Confusion divide one shape: inner strokes.
//! The cliff and the transitions are lines (`edge`), their heads arrows (`accent`). The patch
//! under Confusion is the canvas color. Item badges are nodes (`surface`, `border`). Domain
//! names, items and transition labels take the text color; the descriptions and the "+N more"
//! badge the secondary text color.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const WORK: &str = "cynefin-beta
  title Work
  complex
    \"Research\"
  clear
    \"Checklists\"
  confusion
    \"a\" \"b\" \"c\" \"d\"
  complex --> clear : \"Stabilise\"
  chaotic --> complex
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

#[test]
fn cynefin_consumes_its_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let series = p.series.clone().unwrap();
    let r = render(WORK, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("style=\"max-width:100%;height:auto;color:{}", p.text)));
    assert!(svg.contains(&format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        r.width, r.height, p.background
    )));

    // Domain tints, in Pele's series order.
    let regions = tags(svg, "<path class=\"pele-cynefin-region\"");
    assert_eq!(regions.len(), 5);
    for (region, (name, k)) in regions.iter().zip([("complex", 0), ("complicated", 1), ("chaotic", 3), ("clear", 2)]) {
        assert!(region.contains(&format!("data-id=\"{name}\"")));
        assert!(region.ends_with(&format!("fill=\"{}\" fill-opacity=\"0.14\"/>", series[k])), "{name}");
    }
    assert!(regions[4].contains("data-id=\"confusion\""));
    assert!(regions[4]
        .ends_with(&format!("fill=\"{}\" fill-opacity=\"0.18\" stroke=\"{}\" stroke-dasharray=\"4 2\"/>", series[4], t.inner_stroke)));
    // The patch under Confusion hides the boundaries with the canvas color.
    assert!(svg.contains(&format!("Z\" fill=\"{}\"/><path class=\"pele-cynefin-region\" data-id=\"confusion\"", p.background)));

    // Boundaries are inner strokes, the cliff a line.
    assert_eq!(t.inner_stroke, p.group_border.clone().unwrap());
    assert!(svg
        .contains(&format!("<g class=\"pele-cynefin-boundaries\" fill=\"none\" stroke=\"{}\" stroke-dasharray=\"6 3\">", t.inner_stroke)));
    assert!(tags(svg, "<path class=\"pele-cynefin-cliff\"")[0].contains(&format!("stroke=\"{}\" stroke-width=\"2\"", t.line)));

    // Descriptions and the overflow badge are secondary text; badges are nodes.
    let subtitles = tags(svg, "<text class=\"pele-cynefin-subtitle\"");
    assert_eq!(subtitles.len(), 9);
    for s in &subtitles {
        assert!(s.contains(&format!("fill=\"{}\"", p.secondary_text)), "{s}");
    }
    let badges = tags(svg, "<rect x=");
    assert_eq!(badges.len(), 6);
    for b in &badges {
        assert!(b.contains(&format!("rx=\"4\" fill=\"{}\" stroke=\"{}\"", p.surface, p.border)), "{b}");
    }
    assert!(svg.contains(&format!("font-size=\"14\" fill=\"{}\"", p.secondary_text)), "+1 more");
    assert!(svg.contains("<text class=\"pele-label\" font-size=\"14\" x="), "item text inherits the text color");

    // Transitions are lines with arrowheads; their labels inherit the text color.
    let edges = tags(svg, "<g class=\"pele-edge\"");
    assert_eq!(edges.len(), 2);
    assert_eq!(svg.matches(&format!("\" fill=\"none\" stroke=\"{}\"/>", t.line)).count(), 2);
    let markers = tags(svg, "<path class=\"pele-marker\"");
    assert_eq!(markers.len(), 2);
    for m in &markers {
        assert!(m.ends_with(&format!("fill=\"{}\" stroke=\"none\"/>", p.accent)), "{m}");
    }
    assert!(svg.contains("<text class=\"pele-edge-label\" font-size=\"13\" x="));

    for unused in [
        &p.surface_secondary,
        &p.selection,
        &p.warning_background,
        &p.warning_text,
        &p.danger_background,
        &p.danger_text,
        &p.success_background,
        &p.success_text,
        p.group_background.as_ref().unwrap(),
        &series[5],
    ] {
        assert!(!svg.contains(unused.as_str()), "cynefin uses {unused}");
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(WORK, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear =
        render(WORK, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
    // The patch under Confusion keeps the canvas color.
    assert!(clear.svg.contains(&format!("fill=\"{}\"/>", p.background)));
}

#[test]
fn palettes_change_colors_only() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    let c = MermaidPalette::light();
    let ra = render(WORK, &resolved_options(Some(a.clone()))).unwrap();
    let rb = render(WORK, &resolved_options(Some(b.clone()))).unwrap();
    let rc = render(WORK, &resolved_options(Some(c.clone()))).unwrap();
    assert_ne!(ra.svg, rb.svg);
    assert_ne!(ra.svg, rc.svg);
    assert_eq!((ra.width, ra.height), (rb.width, rb.height));
    assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
    assert_eq!(without_colors(&ra.svg), without_colors(&rc.svg));
    assert_no_css_runtime(&rb.svg);
    assert_no_css_runtime(&rc.svg);
    // Without series, the domains take Zebra's derived chart colors, starting with the accent.
    let t = ResolvedTheme::from_palette(&c);
    assert!(rc.svg.contains("data-id=\"complex\" d=\""));
    assert!(rc.svg.contains(&format!("fill=\"{}\" fill-opacity=\"0.14\"", t.chart_color(0))));
    // CSS mode stays Pele's.
    let css = render(WORK, &RenderOptions::default()).unwrap();
    assert!(css.svg.contains("fill=\"var(--pele-series-1,#4c78a8)\" fill-opacity=\"0.14\""));
    assert!(css.svg.contains("stroke=\"var(--_b)\" stroke-dasharray=\"6 3\""));
    assert!(css.svg.contains("fill=\"var(--_bg)\" stroke=\"var(--_b)\""));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for dark in [false, true] {
        let mode = if dark { MermaidThemeMode::Dark } else { MermaidThemeMode::Light };
        let r = render_for_zebra(WORK, &ZebraRenderOptions { theme_mode: mode, ..Default::default() }).unwrap();
        svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{e}"));
        assert_no_css_runtime(&r.svg);
    }
}
