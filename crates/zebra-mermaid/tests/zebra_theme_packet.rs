#![cfg(feature = "packet")]

//! Zebra theme roles of packet diagrams in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! | Part | Role | Pele |
//! | --- | --- | --- |
//! | field box fill | NodeFill | `var(--_s)` |
//! | field box outline | NodeStroke | `var(--_b)` |
//! | bit numbers (captions) | SecondaryText | `var(--_m)` |
//! | field names, title | Text (root) | inherited |
//! | canvas | Background | `var(--_bg)` |

mod common;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

const PACKET: &str =
    "packet\n  title Header\n  0-15: \"Source Port\"\n  16-31: \"Destination Port\"\n  32-63: \"Sequence Number\"\n  +1: \"F\"\n";

/// The start tag of every element whose tag contains `marker`.
fn tags<'a>(svg: &'a str, marker: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(marker) {
        let at = from + k;
        // A marker may be the start of the tag itself.
        let start = svg[..=at].rfind('<').unwrap();
        let end = at + svg[at..].find('>').unwrap();
        out.push(&svg[start..=end]);
        from = end;
    }
    out
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let at = tag.find(&key)? + key.len();
    Some(&tag[at..at + tag[at..].find('"')?])
}

#[test]
fn packet_consumes_its_roles() {
    let p = sentinel();
    // Across, and down at a narrow width, where labels wrap.
    for options in [resolved_options(Some(p.clone())), RenderOptions { max_width: Some(300.0), ..resolved_options(Some(p.clone())) }] {
        let r = render(PACKET, &options).unwrap();
        let svg = &r.svg;
        assert_no_css_runtime(svg);
        assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)), "root text");
        assert!(svg.contains(&format!(
            "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
            r.width, r.height, p.background
        )));
        let fields = tags(svg, "class=\"pele-node\"");
        assert!(fields.len() >= 4);
        let boxes: Vec<&str> = tags(svg, "<rect x=");
        assert_eq!(boxes.len(), fields.len());
        for b in boxes {
            assert_eq!(attr(b, "fill"), Some(p.surface.as_str()));
            assert_eq!(attr(b, "stroke"), Some(p.border.as_str()));
        }
        let bits = tags(svg, "class=\"pele-packet-bits\"");
        assert_eq!(bits.len(), 1);
        assert_eq!(attr(bits[0], "fill"), Some(p.secondary_text.as_str()));
        // Roles a packet diagram does not use.
        for unused in [
            &p.surface_secondary,
            &p.accent,
            &p.selection,
            &p.warning_background,
            &p.warning_text,
            &p.danger_background,
            &p.danger_text,
            &p.success_background,
            &p.success_text,
            p.edge.as_ref().unwrap(),
            p.group_background.as_ref().unwrap(),
            p.group_border.as_ref().unwrap(),
        ] {
            assert!(!svg.contains(unused.as_str()), "packet uses {unused}");
        }
        for unused in p.series.as_ref().unwrap() {
            assert!(!svg.contains(unused.as_str()), "packet uses {unused}");
        }
    }
}

#[test]
fn packet_defaults_without_palette() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(PACKET, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert_no_css_runtime(&r.svg);
        let palette = MermaidPalette::default_for(theme);
        assert_eq!(attr(tags(&r.svg, "<rect x=")[0], "fill"), Some(palette.surface.as_str()));
    }
}

#[test]
fn packet_transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(PACKET, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear =
        render(PACKET, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
}

#[test]
fn packet_palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    let ra = render(PACKET, &resolved_options(Some(a.clone()))).unwrap();
    let rb = render(PACKET, &resolved_options(Some(b))).unwrap();
    assert_ne!(ra.svg, rb.svg);
    assert_eq!((ra.width, ra.height), (rb.width, rb.height));
    assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
    let css = render(PACKET, &RenderOptions { palette: Some(a), ..Default::default() }).unwrap();
    assert_eq!((css.width, css.height), (ra.width, ra.height));
}
