#![cfg(feature = "radar")]

//! Zebra theme roles of radar charts in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! | Part | Role | Pele |
//! | --- | --- | --- |
//! | curve fill and outline, legend swatch | series *i* | `seriesColor(i)` |
//! | graticule rings (grid) | InnerStroke | `var(--_a)` |
//! | spokes (axes) | Line | `var(--_b)` |
//! | axis names (captions) | SecondaryText | inherited |
//! | title, legend text | Text (root) | inherited |
//! | canvas | Background | `var(--_bg)` |

mod common;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

const RADAR: &str = "radar-beta\n  title Skills\n  axis A[\"Speed\"], B, C\n  curve x[\"One\"]{1, 2, 3}\n  curve y{3, 2, 1}\n  curve z{2, 2, 2}\n  curve w{1, 3, 1}\n";
const POLYGON: &str = "radar-beta\n  axis A, B, C, D\n  curve x{1, 2, 3, 4}\n  curve y{4, 3, 2, 1}\n  graticule polygon\n  ticks 3\n";

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
fn radar_consumes_its_roles() {
    let p = sentinel();
    let series = p.series.clone().unwrap();
    for source in [RADAR, POLYGON] {
        let r = render(source, &resolved_options(Some(p.clone()))).unwrap();
        let svg = &r.svg;
        assert_no_css_runtime(svg);
        assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)), "root text");
        assert!(svg.contains(&format!(
            "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
            r.width, r.height, p.background
        )));
        // The grid takes the inner stroke (the group border when one is given), the spokes the
        // connector color, axis names the secondary text color.
        let grid = tags(svg, "class=\"pele-graticule\"");
        assert_eq!(grid.len(), 1);
        assert_eq!(attr(grid[0], "stroke"), p.group_border.as_deref());
        let axes = tags(svg, "class=\"pele-axes\"");
        assert_eq!(axes.len(), 1);
        assert_eq!(attr(axes[0], "stroke"), p.edge.as_deref());
        let labels = tags(svg, "class=\"pele-axis-labels\"");
        assert_eq!(attr(labels[0], "fill"), Some(p.secondary_text.as_str()));
        // Curves and legend swatches take the series in order.
        let curves = tags(svg, "class=\"pele-curve\"");
        assert!(curves.len() >= 2);
        for (i, curve) in curves.iter().enumerate() {
            assert_eq!(attr(curve, "fill"), Some(series[i].as_str()), "curve {i}");
            assert_eq!(attr(curve, "stroke"), Some(series[i].as_str()), "curve {i}");
        }
        let swatches: Vec<&str> = tags(svg, "fill-opacity=\"0.4\"");
        assert_eq!(swatches.len(), curves.len());
        for (i, swatch) in swatches.iter().enumerate() {
            assert_eq!(attr(swatch, "fill"), Some(series[i].as_str()), "swatch {i}");
            assert_eq!(attr(swatch, "stroke"), Some(series[i].as_str()), "swatch {i}");
        }
        // Roles a radar chart does not use.
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
            p.group_background.as_ref().unwrap(),
        ] {
            assert!(!svg.contains(unused.as_str()), "radar uses {unused}");
        }
        for unused in &series[curves.len()..] {
            assert!(!svg.contains(unused.as_str()), "radar uses {unused}");
        }
    }
}

#[test]
fn radar_grid_without_group_border_is_a_faint_text_mix() {
    let mut p = sentinel();
    p.group_border = None;
    let r = render(RADAR, &resolved_options(Some(p.clone()))).unwrap();
    let expected = zebra_mermaid::theme::color::mix_hex(&p.text, &p.background, 0.12);
    assert_eq!(attr(tags(&r.svg, "class=\"pele-graticule\"")[0], "stroke"), Some(expected.as_str()));
    assert!(!r.svg.contains(&p.border), "the grid is not a node border");
}

#[test]
fn zebra_reference_radar_uses_the_first_supplied_series() {
    // Zebra's `semantic_palette_reaches_chart_layouts_and_contrasting_pie_labels`.
    let mut p = MermaidPalette::dark();
    p.background = "#101010".into();
    p.text = "#F0F0F0".into();
    p.edge = Some("#72B3C4".into());
    p.group_background = Some("#243546".into());
    p.group_border = Some("#8394A5".into());
    p.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let r = render("radar-beta\n axis Speed, Beauty, Safety\n curve Zebra {9,8,9}", &resolved_options(Some(p))).unwrap();
    assert!(r.svg.contains("#EFEFEF"), "radar must use the supplied first series");
    assert!(!r.svg.contains("#131300"));
    assert_no_css_runtime(&r.svg);
}

#[test]
fn radar_defaults_without_palette() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(RADAR, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert_no_css_runtime(&r.svg);
        // Without series, the first curve takes Zebra's first derived chart color, the accent.
        let accent = MermaidPalette::default_for(theme).accent;
        assert_eq!(attr(tags(&r.svg, "class=\"pele-curve\"")[0], "fill"), Some(accent.as_str()));
    }
}

#[test]
fn radar_transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [RADAR, POLYGON] {
        let opaque =
            render(source, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) })
                .unwrap();
        let clear =
            render(source, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
                .unwrap();
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
fn radar_palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    for source in [RADAR, POLYGON] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        // CSS output keeps Pele's geometry with any palette.
        let css = render(source, &RenderOptions { palette: Some(a.clone()), ..Default::default() }).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
        assert!(css.svg.contains(&format!("var(--pele-series-1,{})", a.series.as_ref().unwrap()[0])));
    }
}
