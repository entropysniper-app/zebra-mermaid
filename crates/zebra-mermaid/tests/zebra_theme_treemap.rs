#![cfg(feature = "treemap")]

//! Zebra theme roles of treemaps in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! | Part | Role | Pele |
//! | --- | --- | --- |
//! | section and leaf fill, section outline | series of the top-level branch | `seriesColor(i)` |
//! | leaf name and value | text on the leaf's fill (contrast-picked) | `var(--_bg)` on the group |
//! | section value | SecondaryText | `var(--_m)` |
//! | section name, title | Text (root) | inherited |
//! | canvas | Background | `var(--_bg)` |

mod common;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

// Four branches take series 0 to 3: dark, dark, mid and light in the sentinel palette.
const TREEMAP: &str =
    "treemap-beta\n\"Alpha\"\n  \"a1\": 60\n  \"a2\": 40\n\"Beta\"\n  \"b1\": 30\n\"Gamma\"\n  \"c1\": 20\n\"Delta\"\n  \"d1\": 25\n";
const STYLED: &str = "treemap-beta\n\"Alpha\"\n  \"a1\": 60:::big\n  \"a2\": 40\nclassDef big fill:#123456,color:#654321\n";

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

/// The markup of the `<g>` whose start tag contains `marker`, through its end tag.
fn group<'a>(svg: &'a str, marker: &str) -> &'a str {
    let at = svg.find(marker).unwrap();
    let start = svg[..at].rfind('<').unwrap();
    let end = at + svg[at..].find("</g>").unwrap();
    &svg[start..end + 4]
}

#[test]
fn treemap_consumes_its_roles() {
    let p = sentinel();
    let series = p.series.clone().unwrap();
    let r = render(TREEMAP, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)), "root text");
    assert!(svg.contains(&format!(
        "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
        r.width, r.height, p.background
    )));

    // Sections: a tint of their branch's series, outlined with it.
    for (i, name) in ["Alpha", "Beta", "Gamma", "Delta"].iter().enumerate() {
        let g = group(svg, &format!("class=\"pele-node pele-section\" data-id=\"{name}\""));
        let rect = tags(g, "<rect")[0];
        assert_eq!(attr(rect, "fill"), Some(series[i].as_str()), "{name}");
        assert_eq!(attr(rect, "stroke"), Some(series[i].as_str()), "{name}");
        assert_eq!(attr(rect, "fill-opacity"), Some("0.12"), "{name}");
    }
    // A section's value is a caption.
    let values = tags(group(svg, "data-id=\"Alpha\""), "class=\"pele-value\"");
    assert_eq!(values.len(), 1);
    assert_eq!(attr(values[0], "fill"), Some(p.secondary_text.as_str()));

    // Leaves: filled with their branch's series, labels in a color that reads on it.
    assert_eq!(attr(tags(svg, "class=\"pele-leaves\"")[0], "fill"), Some(p.background.as_str()));
    for (name, i) in [("a1", 0), ("a2", 0), ("b1", 1), ("c1", 2), ("d1", 3)] {
        let g = group(svg, &format!("class=\"pele-node pele-leaf\" data-id=\"{name}\""));
        assert_eq!(attr(tags(g, "<rect")[0], "fill"), Some(series[i].as_str()), "{name}");
        let expected = contrasting_text_on_fill(&series[i], &p.text, &p.background, 1.0);
        let texts = tags(g, "<text");
        assert!(!texts.is_empty(), "{name} has a label");
        for t in texts {
            assert_eq!(attr(t, "fill"), Some(expected.as_str()), "{name}");
        }
    }
    // Dark series take the canvas color, the light one the text color.
    assert_eq!(contrasting_text_on_fill(&series[1], &p.text, &p.background, 1.0), p.background);
    assert_eq!(contrasting_text_on_fill(&series[3], &p.text, &p.background, 1.0), p.text);

    // Roles a treemap does not use.
    for unused in [
        &p.surface,
        &p.surface_secondary,
        &p.accent,
        &p.border,
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
        &series[4],
        &series[5],
    ] {
        assert!(!svg.contains(unused.as_str()), "treemap uses {unused}");
    }
}

#[test]
fn treemap_author_styles_keep_precedence() {
    let p = sentinel();
    let r = render(STYLED, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    let g = group(&r.svg, "data-id=\"a1\"");
    // The class's fill and text color are written in style attributes, which win over the
    // role colors written as presentation attributes.
    assert!(tags(g, "<rect")[0].contains("style=\"fill:#123456;\""));
    for t in tags(g, "<text") {
        assert!(t.contains("style=\"fill:#654321;\""));
    }
}

#[test]
fn treemap_defaults_without_palette() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(TREEMAP, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert_no_css_runtime(&r.svg);
        let accent = MermaidPalette::default_for(theme).accent;
        let g = group(&r.svg, "data-id=\"a1\"");
        assert_eq!(attr(tags(g, "<rect")[0], "fill"), Some(accent.as_str()));
    }
}

#[test]
fn treemap_transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(TREEMAP, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear =
        render(TREEMAP, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
    // Leaf labels still read on their fills without the canvas.
    assert!(clear.svg.contains(&format!("fill=\"{}\">", p.background)));
}

#[test]
fn treemap_palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::light().swift_completed();
    for source in [TREEMAP, STYLED] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        let css = render(source, &RenderOptions { palette: Some(a.clone()), ..Default::default() }).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
    }
}
