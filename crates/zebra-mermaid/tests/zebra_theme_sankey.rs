#![cfg(feature = "sankey")]

//! Zebra theme compatibility of sankey diagrams (docs/ZEBRA-THEME-COMPATIBILITY.md): which
//! semantic role colors each part in resolved mode, and that CSS mode stays Pele's.
//!
//! | Part                              | Role             | Pele literal          |
//! |-----------------------------------|------------------|-----------------------|
//! | node bar `rect.pele-sankey-node`  | series(i)        | `seriesColor(i)`      |
//! | band `path.pele-sankey-link`      | series(end)      | `seriesColor(end)`    |
//! | value `tspan.pele-sankey-value`   | SecondaryText    | `var(--_m)`           |
//! | halo of an outlined label         | Background       | `var(--_bg)`          |
//! | names and title                   | Text (inherited) | root `var(--_fg)`     |
//! | opaque canvas                     | Background       | `var(--_bg)`          |
//!
//! Labels sit beside the bars, never on them, so they keep the text color.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions};

/// Zebra's own renderer test input.
const ZEBRA: &str = "sankey\n Capture, Edit, 10\n Edit, Publish, 8";

const OUTLINED: &str =
    "---\nconfig:\n  sankey:\n    labelStyle: outlined\n    linkColor: target\n---\nsankey\nA,B,3\nA,C,2\nB,D,1\nC,D,2\n";

const AUTHORED: &str =
    "---\nconfig:\n  sankey:\n    nodeColors:\n      A: \"#123456\"\n    linkColor: \"#654321\"\n---\nsankey\nA,B,3\nA,C,2\n";

/// The opening tag of the element that the last tag of `prefix` starts, where `prefix` first
/// occurs.
fn element<'a>(svg: &'a str, prefix: &str) -> &'a str {
    let at = svg.find(prefix).unwrap_or_else(|| panic!("no {prefix} in {svg}"));
    let start = at + prefix.rfind('<').unwrap();
    let end = svg[start..].find('>').unwrap() + start + 1;
    &svg[start..end]
}

fn series(p: &MermaidPalette) -> [String; 6] {
    p.series.clone().unwrap()
}

#[test]
fn zebra_reference_first_series_reaches_the_chart() {
    // `semantic_palette_reaches_chart_layouts_and_contrasting_pie_labels` in Zebra's renderer.
    let mut p = MermaidPalette::dark();
    p.background = "#101010".into();
    p.text = "#F0F0F0".into();
    p.edge = Some("#72B3C4".into());
    p.group_background = Some("#243546".into());
    p.group_border = Some("#8394A5".into());
    p.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let r = render(ZEBRA, &resolved_options(Some(p))).unwrap();
    assert!(r.svg.contains("#EFEFEF"), "sankey must use the supplied first series");
    assert!(!r.svg.contains("#131300"), "sankey must not retain fixed dark label text");
    assert_no_css_runtime(&r.svg);
}

#[test]
fn sankey_consumes_its_roles() {
    let p = sentinel();
    let s = series(&p);
    let r = render(ZEBRA, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    // Root text color and the opaque canvas.
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!(
        "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
        r.width, r.height, p.background
    )));
    // Bars take the series in node order.
    for (id, color) in [("Capture", &s[0]), ("Edit", &s[1]), ("Publish", &s[2])] {
        let bar = element(svg, &format!("<rect class=\"pele-sankey-node\" data-id=\"{id}\""));
        assert!(bar.contains(&format!("fill=\"{color}\"")), "{bar}");
    }
    // Bands take their source's series (the default `gradient`).
    let band = element(svg, "<path class=\"pele-sankey-link\" data-id=\"Capture-&gt;Edit\"");
    assert!(band.contains(&format!("fill=\"{}\"", s[0])), "{band}");
    let band = element(svg, "<path class=\"pele-sankey-link\" data-id=\"Edit-&gt;Publish\"");
    assert!(band.contains(&format!("fill=\"{}\"", s[1])), "{band}");
    // Values beside the names are secondary text; the names inherit the root text color.
    assert!(svg.contains(&format!("<tspan class=\"pele-sankey-value\" fill=\"{}\">10</tspan>", p.secondary_text)));
    assert!(svg.contains("<g class=\"pele-labels\" font-size=\"14\"><g class=\"pele-label\" data-id=\"Capture\"><text x="));
    // Roles a sankey does not use must not leak in.
    let unused = [
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
    ];
    for color in unused {
        assert!(!svg.contains(color.as_str()), "sankey uses {color}");
    }
    // Only three nodes: series beyond the third do not appear.
    for color in &s[3..] {
        assert!(!svg.contains(color.as_str()), "sankey uses {color}");
    }
}

#[test]
fn outlined_labels_and_target_colored_bands() {
    let p = sentinel();
    let s = series(&p);
    let r = render(OUTLINED, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    // The halo behind an outlined label is the canvas color.
    let halo = element(&r.svg, "<g class=\"pele-label\" data-id=\"A\"><text");
    assert!(halo.contains(&format!("fill=\"none\" stroke=\"{}\" stroke-width=\"3\"", p.background)), "{halo}");
    // `linkColor: target` colors each band as the node it enters.
    let band = element(&r.svg, "<path class=\"pele-sankey-link\" data-id=\"A-&gt;C\"");
    assert!(band.contains(&format!("fill=\"{}\"", s[2])), "{band}");
    let band = element(&r.svg, "<path class=\"pele-sankey-link\" data-id=\"C-&gt;D\"");
    assert!(band.contains(&format!("fill=\"{}\"", s[3])), "{band}");
}

#[test]
fn author_colors_keep_precedence() {
    let p = sentinel();
    for options in [resolved_options(Some(p.clone())), RenderOptions::default()] {
        let r = render(AUTHORED, &options).unwrap();
        let bar = element(&r.svg, "<rect class=\"pele-sankey-node\" data-id=\"A\"");
        assert!(bar.ends_with(" style=\"fill:#123456;\"/>"), "{bar}");
        let band = element(&r.svg, "<path class=\"pele-sankey-link\" data-id=\"A-&gt;B\"");
        assert!(band.ends_with(" style=\"fill:#654321;\">"), "{band}");
    }
}

#[test]
fn css_mode_is_pele() {
    let r = render(OUTLINED, &RenderOptions::default()).unwrap();
    assert!(r.svg.contains("fill=\"var(--pele-series-1,#4c78a8)\""));
    assert!(r.svg.contains("<tspan class=\"pele-sankey-value\" fill=\"var(--_m)\">"));
    assert!(r.svg.contains("stroke=\"var(--_bg)\" stroke-width=\"3\""));
    assert!(!r.svg.contains("pele-canvas"));
    // A palette in CSS mode only changes the fallbacks of the custom properties.
    let p = sentinel();
    let themed = render(OUTLINED, &RenderOptions { palette: Some(p.clone()), ..Default::default() }).unwrap();
    assert!(themed.svg.contains(&format!("var(--pele-series-1,{})", series(&p)[0])));
    assert_eq!((themed.width, themed.height), (r.width, r.height));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [ZEBRA, OUTLINED] {
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
    // The halo still takes the background on a transparent canvas.
    let clear =
        render(OUTLINED, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert!(clear.svg.contains(&format!("stroke=\"{}\"", p.background)));
}

#[test]
fn palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    for source in [ZEBRA, OUTLINED, AUTHORED] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        let rn = render(source, &resolved_options(None)).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        assert_eq!(without_colors(&ra.svg), without_colors(&rn.svg));
        assert_no_css_runtime(&rb.svg);
        assert_no_css_runtime(&rn.svg);
        // And the geometry is Pele's.
        let css = render(source, &RenderOptions::default()).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
    }
}

#[test]
fn many_nodes_cycle_through_the_chart_palette() {
    let p = sentinel();
    let source = format!("sankey\n{}", (1..14).map(|i| format!("Hub,N{i},{i}\n")).collect::<String>());
    let r = render(&source, &resolved_options(Some(p.clone()))).unwrap();
    // Node 12 wraps to the first series again, as Zebra's twelve-color chart palette does.
    let bar = element(&r.svg, "<rect class=\"pele-sankey-node\" data-id=\"N12\"");
    assert!(bar.contains(&format!("fill=\"{}\"", series(&p)[0])), "{bar}");
    assert_no_css_runtime(&r.svg);
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [ZEBRA, OUTLINED, AUTHORED] {
        for theme_mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
            let r = render_for_zebra(source, &ZebraRenderOptions { theme_mode, ..Default::default() }).unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains("data-mermaid-canvas-background=\"opaque\""));
        }
    }
}
