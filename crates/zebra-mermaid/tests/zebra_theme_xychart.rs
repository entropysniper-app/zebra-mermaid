#![cfg(feature = "xychart")]

//! Zebra theme roles of the XY chart in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! Each series (its bars or line and points, and its legend swatch) takes the chart palette
//! color of its place in the source, the caller's series first. The grid is an inner stroke,
//! the axes and the baseline under bars going both ways are lines (`edge`), axis labels are
//! secondary text, and titles, values and legend text take the root text color.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const CHART: &str = "---
config:
  xyChart:
    showDataLabel: true
---
xychart-beta
  title \"Sales\"
  x-axis \"Month\" [jan, feb, mar, apr]
  y-axis \"Revenue\"
  bar \"A\" [10, -5, 20, 15]
  bar \"B\" [5, 8, -3, 12]
  line \"C\" [7 \"low\", 9, 11, 13]
";

const HORIZONTAL: &str = "xychart-beta horizontal
  x-axis [a, b, c]
  bar \"Up\" [3, 5, 2]
  line \"Trend\" [1, 4, 6]
";

/// The element that starts with `start`, through its matching `</g>` when it is a group.
fn element<'a>(svg: &'a str, start: &str) -> &'a str {
    let at = svg.find(start).unwrap_or_else(|| panic!("no {start} in {svg}"));
    if !start.starts_with("<g") {
        let end = svg[at..].find('>').unwrap() + at + 1;
        return &svg[at..end];
    }
    let mut depth = 0;
    let mut i = at;
    loop {
        let rest = &svg[i..];
        if rest.starts_with("<g") && rest[2..].starts_with([' ', '>']) {
            depth += 1;
            i += 2;
        } else if rest.starts_with("</g>") {
            depth -= 1;
            i += 4;
            if depth == 0 {
                return &svg[at..i];
            }
        } else {
            i += rest.chars().next().unwrap().len_utf8();
        }
    }
}

#[test]
fn xychart_consumes_its_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let r = render(CHART, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    // The root text color (titles, values, legend text inherit it) and the opaque canvas.
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("style=\"max-width:100%;height:auto;color:{}", p.text)));
    assert!(svg.contains(&format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        r.width, r.height, p.background
    )));
    assert!(svg.contains("class=\"pele-title\" font-weight=\"bold\""));

    // Grid, axes and axis labels.
    assert_eq!(t.inner_stroke, p.group_border.clone().unwrap());
    assert!(element(svg, "<path class=\"pele-grid\"").ends_with(&format!("stroke=\"{}\"/>", t.inner_stroke)));
    let axis = element(svg, "<path class=\"pele-axis\"");
    assert!(axis.ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", p.edge.clone().unwrap())), "{axis}");
    // Bars going both ways stand on a baseline drawn with the axes: the bottom axis and the
    // baseline are the two horizontal rules of the path.
    assert_eq!(axis.matches('H').count(), 2, "{axis}");
    assert!(svg.contains(&format!("<g class=\"pele-axis-labels\" font-size=\"14\" fill=\"{}\">", p.secondary_text)));
    assert!(svg.contains("<g class=\"pele-axis-titles\" font-size=\"14\">"));
    assert!(svg.contains("<g class=\"pele-data-labels\" font-size=\"12\">"));
    assert!(svg.contains("<g class=\"pele-point-labels\" font-size=\"12\">"));

    // Series in source order: two bar series and a line, each with its legend swatch.
    let series = p.series.clone().unwrap();
    assert!(element(svg, "<g class=\"pele-series pele-bars\" data-id=\"A\"")
        .starts_with(&format!("<g class=\"pele-series pele-bars\" data-id=\"A\" fill=\"{}\">", series[0])));
    assert!(element(svg, "<g class=\"pele-series pele-bars\" data-id=\"B\"")
        .starts_with(&format!("<g class=\"pele-series pele-bars\" data-id=\"B\" fill=\"{}\">", series[1])));
    let line = element(svg, "<g class=\"pele-series pele-line\" data-id=\"C\"");
    assert!(line.starts_with(&format!("<g class=\"pele-series pele-line\" data-id=\"C\" fill=\"{}\">", series[2])), "{line}");
    assert!(line.contains(&format!("fill=\"none\" stroke=\"{}\" stroke-width=\"2\"", series[2])), "{line}");
    assert!(line.contains("<circle "));
    let legend = element(svg, "<g class=\"pele-legend\"");
    assert!(element(legend, "<g class=\"pele-legend-item\" data-id=\"A\"").contains(&format!("rx=\"2\" fill=\"{}\"/>", series[0])));
    assert!(element(legend, "<g class=\"pele-legend-item\" data-id=\"B\"").contains(&format!("rx=\"2\" fill=\"{}\"/>", series[1])));
    assert!(
        element(legend, "<g class=\"pele-legend-item\" data-id=\"C\"").contains(&format!("stroke=\"{}\" stroke-width=\"2\"", series[2]))
    );

    // Roles an XY chart does not use must not leak in.
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
        &series[3],
        &series[4],
        &series[5],
    ] {
        assert!(!svg.contains(unused.as_str()), "xychart uses {unused}");
    }
}

#[test]
fn horizontal_charts_take_the_same_roles() {
    let p = sentinel();
    let r = render(HORIZONTAL, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    let series = p.series.clone().unwrap();
    assert!(svg.contains(&format!("<g class=\"pele-series pele-bars\" data-id=\"Up\" fill=\"{}\">", series[0])));
    assert!(svg.contains(&format!("<g class=\"pele-series pele-line\" data-id=\"Trend\" fill=\"{}\">", series[1])));
    assert!(element(svg, "<path class=\"pele-axis\"").contains(&format!("stroke=\"{}\"", p.edge.clone().unwrap())));
    assert!(element(svg, "<path class=\"pele-grid\"").contains(&format!("stroke=\"{}\"", p.group_border.clone().unwrap())));
}

#[test]
fn series_past_the_sixth_take_derived_chart_colors() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let bars: String = (0..9).map(|i| format!("  bar \"s{i}\" [{}]\n", i + 1)).collect();
    let r = render(&format!("xychart-beta\n  x-axis [a]\n{bars}"), &resolved_options(Some(p.clone()))).unwrap();
    for i in 0..9 {
        let color = t.chart_color(i);
        assert!(r.svg.contains(&format!("data-id=\"s{i}\" fill=\"{color}\">")), "series {i}");
    }
    assert_eq!(t.chart_color(6), zebra_mermaid::theme::color::mix_hex(&p.series.as_ref().unwrap()[0], &p.background, 0.70));
    assert_no_css_runtime(&r.svg);
}

#[test]
fn zebra_reference_first_series_reaches_the_chart() {
    // Zebra's `semantic_palette_reaches_chart_layouts_and_contrasting_pie_labels`. Its XY input
    // (`x-axis Jan, Feb` without brackets) is a syntax error for Mermaid's grammar and Pele, so
    // the categories are written in brackets here.
    let mut palette = MermaidPalette::dark();
    palette.background = "#101010".into();
    palette.text = "#F0F0F0".into();
    palette.edge = Some("#72B3C4".into());
    palette.group_background = Some("#243546".into());
    palette.group_border = Some("#8394A5".into());
    palette.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let options = resolved_options(Some(palette));
    assert!(render("xychart-beta\n x-axis Jan, Feb\n y-axis Notes\n bar [10, 18]", &options).is_err());
    let rendered = render("xychart-beta\n x-axis [Jan, Feb]\n y-axis Notes\n bar [10, 18]", &options).unwrap();
    assert!(rendered.svg.contains("fill=\"#EFEFEF\""), "XY must use the supplied first series");
    assert!(!rendered.svg.contains("#131300"));
    assert!(rendered.svg.contains("stroke=\"#72B3C4\""), "axes take the edge color");
    assert_no_css_runtime(&rendered.svg);
}

#[test]
fn defaults_without_palette_derive_the_chart_colors() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(CHART, &RenderOptions { theme, ..resolved_options(None) }).unwrap();
        let p = MermaidPalette::default_for(theme);
        let t = ResolvedTheme::from_palette(&p);
        assert!(r.svg.contains(&format!("data-id=\"A\" fill=\"{}\">", t.chart_color(0))));
        assert_eq!(t.chart_color(0), p.accent);
        assert!(r.svg.contains("class=\"pele-grid\""));
        assert!(element(&r.svg, "<path class=\"pele-axis\"").contains(&format!("stroke=\"{}\"", t.line)));
        assert_no_css_runtime(&r.svg);
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [CHART, HORIZONTAL] {
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
fn only_colors_differ_between_palettes() {
    let a = sentinel();
    let b = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    let dark = MermaidPalette::dark().swift_completed();
    let ra = render(CHART, &resolved_options(Some(a))).unwrap();
    let rb = render(CHART, &resolved_options(Some(b))).unwrap();
    let rd = render(CHART, &resolved_options(Some(dark))).unwrap();
    assert_ne!(ra.svg, rb.svg);
    assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
    assert_eq!(without_colors(&ra.svg), without_colors(&rd.svg));
    // CSS mode draws the same geometry with Pele's custom properties.
    let css = render(CHART, &RenderOptions::default()).unwrap();
    assert_eq!((css.width, css.height), (ra.width, ra.height));
    assert!(css.svg.contains("stroke=\"var(--_a)\"") && css.svg.contains("stroke=\"var(--_b)\"") && css.svg.contains("fill=\"var(--_m)\""));
    assert!(css.svg.contains("fill=\"var(--pele-series-1,#4c78a8)\""));
    // A palette in CSS mode becomes the fallback of the series properties.
    let themed = render(CHART, &RenderOptions { palette: Some(sentinel()), ..Default::default() }).unwrap();
    assert!(themed.svg.contains("fill=\"var(--pele-series-1,#C10001)\""));
    assert!(themed.svg.contains("stroke=\"var(--_a)\""));
    assert_eq!((themed.width, themed.height), (css.width, css.height));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [CHART, HORIZONTAL] {
        for theme_mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
            let r = render_for_zebra(source, &ZebraRenderOptions { theme_mode, ..Default::default() }).unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains("data-mermaid-canvas-background=\"opaque\""));
        }
    }
}
