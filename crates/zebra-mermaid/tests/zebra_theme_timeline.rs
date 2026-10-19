#![cfg(feature = "timeline")]

//! Zebra theme roles of the timeline in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! Each section, and its periods, events and dots, takes the chart palette color of its place
//! (each period its own when there are no sections), the caller's series first; text on a
//! section or period chip takes a color that reads on it. Periods that come before the first
//! section are neutral: the secondary text color. Event text, the title and the root take the
//! text color. The axis and the connectors are lines (`edge`), the axis arrowhead is an arrow
//! (`accent`), and dots are ringed with the canvas color.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const SECTIONS: &str = "timeline
  title History
  1990 : Before
  section Early
    2000 : A : B
  section Middle
    2005 : C
  section Late
    2010 : D
  section Light
    2020 : E
";

const PLAIN: &str = "timeline\n title Zebra\n 2025 : Prototype\n 2026 : Native renderer\n";

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

/// The `<g>` element that starts with `start`, through its matching `</g>`.
fn group<'a>(svg: &'a str, start: &str) -> &'a str {
    let at = svg.find(start).unwrap_or_else(|| panic!("no {start} in {svg}"));
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
fn timeline_consumes_its_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let series = p.series.clone().unwrap();
    let r = render(SECTIONS, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    // The root text color, which the title and event text inherit, and the opaque canvas.
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("style=\"max-width:100%;height:auto;color:{}", p.text)));
    assert!(svg.contains(&format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        r.width, r.height, p.background
    )));
    assert!(svg.contains("<text class=\"pele-title\" font-weight=\"bold\""));

    // The axis is a line, its head an arrow.
    assert!(tags(svg, "<path class=\"pele-axis\"")[0].ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", t.line)));
    assert_eq!(t.line, p.edge.clone().unwrap());
    assert!(tags(svg, "<path class=\"pele-marker\"")[0].ends_with(&format!("fill=\"{}\" stroke=\"none\"/>", p.accent)));
    // Connectors are lines.
    let edges = tags(svg, "<path class=\"pele-edge\"");
    assert_eq!(edges.len(), 5);
    for e in &edges {
        assert!(e.ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", t.line)), "{e}");
    }

    // Sections take the series in order: header chip, period chip, dot and event cards.
    for (k, name) in ["Early", "Middle", "Late", "Light"].iter().enumerate() {
        let fill = &series[k];
        let g = group(svg, &format!("<g class=\"pele-cluster pele-section\" data-id=\"{name}\">"));
        let on = contrasting_text_on_fill(fill, &p.text, &p.background, 1.0);
        let rects = tags(g, "<rect ");
        assert!(rects[0].ends_with(&format!("rx=\"4\" fill=\"{fill}\"/>")), "header of {name}: {}", rects[0]);
        assert!(g.contains(&format!("<text font-weight=\"bold\" fill=\"{on}\"")), "header text of {name}");
        assert!(rects[1].ends_with(&format!("rx=\"4\" fill=\"{fill}\"/>")), "period of {name}");
        assert!(g.contains(&format!("<text fill=\"{on}\" x=")), "period text of {name}");
        assert!(g.contains(&format!("r=\"4\" fill=\"{fill}\" stroke=\"{}\" stroke-width=\"2\"/>", p.background)), "dot of {name}");
        for card in &rects[2..] {
            assert!(card.ends_with(&format!("fill=\"{fill}\" fill-opacity=\"0.16\"/>")), "event of {name}: {card}");
        }
        // Event text inherits the text color.
        assert!(g.contains("<text font-size=\"14\" x="));
    }
    // Labels on a dark series read in the canvas color, on a light one in the text color.
    assert_eq!(contrasting_text_on_fill(&series[1], &p.text, &p.background, 1.0), p.background);
    assert_eq!(contrasting_text_on_fill(&series[3], &p.text, &p.background, 1.0), p.text);

    // The period before the first section is neutral.
    let leading = group(svg, "<g class=\"pele-node pele-period\" data-id=\"1990\">");
    assert!(leading.contains(&format!("rx=\"4\" fill=\"{}\"/>", p.secondary_text)));
    assert!(leading.contains(&format!("fill=\"{}\" fill-opacity=\"0.16\"/>", p.secondary_text)));
    let on = contrasting_text_on_fill(&p.secondary_text, &p.text, &p.background, 1.0);
    assert!(leading.contains(&format!("<text fill=\"{on}\" x=")));

    // Roles a timeline does not use must not leak in.
    for unused in [
        &p.surface,
        &p.surface_secondary,
        &p.border,
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
        assert!(!svg.contains(unused.as_str()), "timeline uses {unused}");
    }
}

#[test]
fn periods_without_sections_take_their_own_series() {
    let p = sentinel();
    let series = p.series.clone().unwrap();
    let r = render(PLAIN, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    for (k, year) in ["2025", "2026"].iter().enumerate() {
        let g = group(&r.svg, &format!("<g class=\"pele-node pele-period\" data-id=\"{year}\">"));
        assert!(g.contains(&format!("rx=\"4\" fill=\"{}\"/>", series[k])), "{year}");
        let on = contrasting_text_on_fill(&series[k], &p.text, &p.background, 1.0);
        assert!(g.contains(&format!("<text fill=\"{on}\" x=")), "{year}");
    }
    // `disableMulticolor` gives every period the first series.
    let config = "---\nconfig:\n  timeline:\n    disableMulticolor: true\n---\n";
    let r = render(&format!("{config}{PLAIN}"), &resolved_options(Some(p.clone()))).unwrap();
    assert!(!r.svg.contains(&series[1]));
    assert_eq!(r.svg.matches(&format!("rx=\"4\" fill=\"{}\"/>", series[0])).count(), 2);
    // Zebra's renderer test: a timeline uses the supplied first series and no fixed label color.
    let mut zebra = MermaidPalette::dark();
    zebra.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let r = render("timeline\n 2026 : Native renderer", &resolved_options(Some(zebra))).unwrap();
    assert!(r.svg.contains("#EFEFEF"));
    assert!(!r.svg.contains("#131300"));
}

#[test]
fn running_down_keeps_the_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let down = SECTIONS.replacen("timeline", "timeline TD", 1);
    let r = render(&down, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    assert!(r.svg.contains("<path class=\"pele-axis\" d=\"M"));
    assert!(tags(&r.svg, "<path class=\"pele-axis\"")[0].ends_with(&format!("stroke=\"{}\"/>", t.line)));
    assert!(tags(&r.svg, "<path class=\"pele-marker\"")[0].contains(&format!("fill=\"{}\"", p.accent)));
    let g = group(&r.svg, "<g class=\"pele-cluster pele-section\" data-id=\"Early\">");
    assert!(g.contains(&format!("fill=\"{}\" fill-opacity=\"0.16\"/>", p.series.as_ref().unwrap()[0])));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [SECTIONS, PLAIN] {
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
        // The canvas color still rings the dots.
        assert!(clear.svg.contains(&format!("stroke=\"{}\" stroke-width=\"2\"", p.background)));
    }
}

#[test]
fn palettes_change_colors_only() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    let c = MermaidPalette::light();
    for source in [SECTIONS, PLAIN] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        let rc = render(source, &resolved_options(Some(c.clone()))).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_ne!(ra.svg, rc.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        assert_eq!(without_colors(&ra.svg), without_colors(&rc.svg));
        assert_no_css_runtime(&rb.svg);
        assert_no_css_runtime(&rc.svg);
    }
    // CSS mode stays Pele's: series as custom properties, text on chips in the canvas color.
    let css = render(SECTIONS, &RenderOptions::default()).unwrap();
    assert!(css.svg.contains("fill=\"var(--pele-series-1,#4c78a8)\""));
    assert!(css.svg.contains("<text fill=\"var(--_bg)\""));
    assert!(css.svg.contains("rx:var(--_r)"));
    assert!(css.svg.contains("fill=\"var(--_m)\""));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [SECTIONS, PLAIN] {
        for dark in [false, true] {
            let mode = if dark { MermaidThemeMode::Dark } else { MermaidThemeMode::Light };
            let r = render_for_zebra(source, &ZebraRenderOptions { theme_mode: mode, ..Default::default() }).unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
        }
    }
}
