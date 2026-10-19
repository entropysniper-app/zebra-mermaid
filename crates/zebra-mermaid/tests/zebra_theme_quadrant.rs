#![cfg(feature = "quadrantChart")]

//! Zebra theme roles of the quadrant chart in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! Pele fills the four quadrants with one surface and outlines them with the border. They are
//! regions of the chart, so they take the group fill and group outline; the quadrant labels and
//! the point labels over them take the text color that reads on that fill. Points take the
//! first series color (Zebra's reference test expects the caller's first series in a quadrant
//! chart), axis labels are secondary text and the title takes the root text color. A point's
//! own `color`, `stroke-color` and `stroke-width` stay in its `style` attribute.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const CHART: &str = "quadrantChart
  title Reach and engagement
  x-axis Low Reach --> High Reach
  y-axis Low Engagement --> High Engagement
  quadrant-1 We should expand
  quadrant-2 Need to promote
  quadrant-3 Re-evaluate
  quadrant-4 May be improved
  Campaign A: [0.3, 0.6]
  Campaign B: [0.45, 0.23] color: #ff3300, radius: 9
  Campaign C:::warm: [0.57, 0.69]
  Campaign D: [0.78, 0.34] stroke-width: 3px
  classDef warm stroke-color: #00ff00, stroke-width: 2px
";

const EMPTY: &str = "quadrantChart
  quadrant-1 Expand
  quadrant-3 Drop
";

#[test]
fn quadrant_consumes_its_roles() {
    let p = sentinel();
    let r = render(CHART, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        r.width, r.height, p.background
    )));
    assert!(svg.contains("class=\"pele-title\" font-weight=\"bold\""));

    // The quadrants: group fill and outline, corner radius as a plain attribute.
    let group_fill = p.group_background.clone().unwrap();
    let group_stroke = p.group_border.clone().unwrap();
    assert!(svg.contains(&format!("<g class=\"pele-quadrants\" fill=\"{group_fill}\" stroke=\"{group_stroke}\">")));
    assert_eq!(svg.matches("rx=\"4\"/>").count(), 4);
    // Their labels read on the fill. The sentinel group fill is dark, so they take the canvas
    // color, not the text color.
    let on = contrasting_text_on_fill(&group_fill, &p.text, &p.background, 1.0);
    assert_eq!(on, p.background);
    assert_eq!(svg.matches(&format!("class=\"pele-label\" fill=\"{on}\" stroke=\"none\"")).count(), 4);
    assert!(svg.contains(&format!("<g class=\"pele-point-labels\" font-size=\"12\" fill=\"{on}\">")));

    // Axis labels.
    assert!(svg.contains(&format!("<g class=\"pele-axis-labels\" font-size=\"14\" fill=\"{}\">", p.secondary_text)));

    // Points: the first series, also as the stroke of a point with a width and no stroke color.
    let series = p.series.clone().unwrap();
    assert!(svg.contains(&format!("<g class=\"pele-points\" fill=\"{}\">", series[0])));
    assert!(svg.contains(&format!(
        "data-id=\"Campaign D\" cx=\"391.24\" cy=\"321.84\" r=\"5\" stroke=\"{}\" style=\"stroke-width:3px;\"/>",
        series[0]
    )));
    // Author styles keep their place and precedence. (Coordinates as Pele writes them.)
    assert!(svg.contains("data-id=\"Campaign B\" cx=\"240.1\" cy=\"368.48\" r=\"9\" style=\"fill:#ff3300;\"/>"));
    assert!(svg.contains(
        "class=\"pele-point warm\" data-id=\"Campaign C\" cx=\"295.06\" cy=\"173.44\" r=\"5\" style=\"stroke:#00ff00;stroke-width:2px;\"/>"
    ));

    // Roles a quadrant chart does not use must not leak in.
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
        &series[1],
        &series[2],
        &series[3],
        &series[4],
        &series[5],
    ] {
        assert!(!svg.contains(unused.as_str()), "quadrant chart uses {unused}");
    }
}

#[test]
fn without_group_colors_the_quadrants_take_the_canvas_and_border() {
    // A palette of the fourteen required fields: groups fall back as Zebra's renderer does.
    let mut p = sentinel();
    p.edge = None;
    p.group_background = None;
    p.group_border = None;
    p.series = None;
    let t = ResolvedTheme::from_palette(&p);
    let r = render(CHART, &resolved_options(Some(p.clone()))).unwrap();
    assert_eq!(t.group_fill, p.background);
    assert_eq!(t.group_stroke, p.border);
    assert!(r.svg.contains(&format!("<g class=\"pele-quadrants\" fill=\"{}\" stroke=\"{}\">", p.background, p.border)));
    // On the canvas color the labels take the text color.
    assert!(r.svg.contains(&format!("class=\"pele-label\" fill=\"{}\" stroke=\"none\"", p.text)));
    // Without series the points take Zebra's first derived chart color, the accent.
    assert!(r.svg.contains(&format!("<g class=\"pele-points\" fill=\"{}\">", p.accent)));
    assert_no_css_runtime(&r.svg);
}

#[test]
fn a_chart_without_points_has_no_point_groups() {
    let p = sentinel();
    let r = render(EMPTY, &resolved_options(Some(p.clone()))).unwrap();
    assert!(!r.svg.contains("pele-points"));
    assert!(!r.svg.contains("pele-point-labels"));
    assert!(!r.svg.contains(&p.series.as_ref().unwrap()[0]));
    assert_eq!(r.svg.matches("class=\"pele-label\"").count(), 2);
    assert_no_css_runtime(&r.svg);
}

#[test]
fn zebra_reference_first_series_reaches_the_chart() {
    // Zebra's `semantic_palette_reaches_chart_layouts_and_contrasting_pie_labels`.
    let mut palette = MermaidPalette::dark();
    palette.background = "#101010".into();
    palette.text = "#F0F0F0".into();
    palette.edge = Some("#72B3C4".into());
    palette.group_background = Some("#243546".into());
    palette.group_border = Some("#8394A5".into());
    palette.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let rendered = render("quadrantChart\n Quick : [0.8, 0.9]", &resolved_options(Some(palette))).unwrap();
    assert!(rendered.svg.contains("#EFEFEF"), "quadrant must use the supplied first series");
    assert!(rendered.svg.contains("<g class=\"pele-points\" fill=\"#EFEFEF\">"));
    assert!(!rendered.svg.contains("#131300"));
    // Light text reads on the dark group fill.
    assert!(rendered.svg.contains("<g class=\"pele-point-labels\" font-size=\"12\" fill=\"#F0F0F0\">"));
    assert_no_css_runtime(&rendered.svg);
}

#[test]
fn light_and_dark_defaults_without_palette() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(CHART, &RenderOptions { theme, ..resolved_options(None) }).unwrap();
        let p = MermaidPalette::default_for(theme);
        let t = ResolvedTheme::from_palette(&p);
        assert!(r.svg.contains(&format!("<g class=\"pele-quadrants\" fill=\"{}\" stroke=\"{}\">", t.group_fill, t.group_stroke)));
        assert!(r.svg.contains(&format!("<g class=\"pele-points\" fill=\"{}\">", t.chart_color(0))));
        assert!(r.svg.contains(&format!("fill=\"{}\" stroke=\"none\"", t.text_on(&t.group_fill))));
        assert_no_css_runtime(&r.svg);
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [CHART, EMPTY] {
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
    // CSS mode keeps Pele's literals and the same size.
    let css = render(CHART, &RenderOptions::default()).unwrap();
    assert_eq!((css.width, css.height), (ra.width, ra.height));
    assert!(css.svg.contains("<g class=\"pele-quadrants\" fill=\"var(--_s)\" stroke=\"var(--_b)\">"));
    assert!(css.svg.contains("class=\"pele-label\" fill=\"var(--_fg)\" stroke=\"none\""));
    assert!(css.svg.contains("<g class=\"pele-point-labels\" font-size=\"12\">"));
    assert!(css.svg.contains("<g class=\"pele-points\" fill=\"var(--pele-series-1,#4c78a8)\">"));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [CHART, EMPTY] {
        for theme_mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
            let r = render_for_zebra(source, &ZebraRenderOptions { theme_mode, ..Default::default() }).unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains("data-mermaid-canvas-background=\"opaque\""));
        }
    }
}
