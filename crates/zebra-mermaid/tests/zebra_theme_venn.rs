#![cfg(feature = "venn")]

//! Zebra theme compatibility of venn diagrams (docs/ZEBRA-THEME-COMPATIBILITY.md): which
//! semantic role colors each part in resolved mode, and that CSS mode stays Pele's.
//!
//! | Part                                  | Role                         | Pele literal        |
//! |---------------------------------------|------------------------------|---------------------|
//! | set `circle.pele-venn-set` fill       | series(i), at 0.2 opacity    | `seriesColor(i)`    |
//! | set `circle.pele-venn-set` outline    | series(i)                    | `seriesColor(i)`    |
//! | region label `text.pele-label`        | Text, contrast-checked       | inherited `--_fg`   |
//! | text line `text.pele-venn-text`       | SecondaryText                | `var(--_m)`         |
//! | styled overlap `path.pele-venn-overlap` | none (`fill="none"`, author style) | —          |
//! | title                                 | Text (inherited)             | root `var(--_fg)`   |
//! | opaque canvas                         | Background                   | `var(--_bg)`        |
//!
//! A region label is drawn over the translucent fills of its circles. In resolved mode it is
//! given the color Zebra's contrast rule picks on the canvas tinted by those fills, which is
//! the text color unless the tint is strong.

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, MermaidPalette, MermaidThemeMode, RenderOptions};

const SETS: &str = "venn-beta\n  title Pets\n  set A[\"Cats\"]:20\n  set B[\"Dogs\"]:15\n    text b1[\"Rex\"]\n  union A, B[\"Both\"]:4\n";

const STYLED: &str = "venn-beta\n  set A\n  set B\n  set C\n  union A, B, C[\"All\"]\n  style A, B, C fill: #123456, stroke: #654321\n  style A color: #0A0B0C\n";

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
fn venn_consumes_its_roles() {
    let p = sentinel();
    let s = series(&p);
    let r = render(SETS, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!(
        "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
        r.width, r.height, p.background
    )));
    // Each set's circle is filled and outlined with its series color.
    for (id, color) in [("A", &s[0]), ("B", &s[1])] {
        let circle = element(svg, &format!("<circle class=\"pele-venn-set\" data-id=\"{id}\""));
        assert!(circle.contains(&format!("fill=\"{color}\" fill-opacity=\"0.2\" stroke=\"{color}\"/>")), "{circle}");
    }
    // Labels of sets and overlaps read on the tinted canvas: the text color here.
    let label = element(svg, "<g class=\"pele-venn-region\" data-id=\"A\"><text");
    assert!(label.contains(&format!("class=\"pele-label\" fill=\"{}\"", p.text)), "{label}");
    let label = element(svg, "<g class=\"pele-venn-region\" data-id=\"A|B\"><text");
    assert!(label.contains(&format!("class=\"pele-label\" font-size=\"14\" fill=\"{}\"", p.text)), "{label}");
    // A text line is secondary text.
    let text = element(svg, "<text class=\"pele-venn-text\" data-id=\"b1\"");
    assert!(text.contains(&format!("font-size=\"13\" fill=\"{}\"", p.secondary_text)), "{text}");
    // Roles a venn diagram does not use must not leak in.
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
    for color in unused.into_iter().chain(&s[2..]) {
        assert!(!svg.contains(color.as_str()), "venn uses {color}");
    }
    // The background colors only the canvas.
    assert_eq!(svg.matches(p.background.as_str()).count(), 1);
}

#[test]
fn a_strong_tint_turns_the_label_to_the_canvas_color() {
    // Light sets on a dark canvas: three tints under the middle label make it light enough
    // that Zebra's contrast rule prefers the canvas color there, while a single tint keeps the
    // text color.
    let mut p = MermaidPalette::dark();
    p.background = "#101010".into();
    p.text = "#F0F0F0".into();
    p.series = Some(["#FFFFFF"; 6].map(String::from));
    let r = render(STYLED, &resolved_options(Some(p.clone()))).unwrap();
    let all = element(&r.svg, "<g class=\"pele-venn-region\" data-id=\"A|B|C\"><text");
    assert!(all.contains(&format!("class=\"pele-label\" font-size=\"14\" fill=\"{}\"", p.background)), "{all}");
    let one = element(&r.svg, "<g class=\"pele-venn-region\" data-id=\"B\"><text");
    assert!(one.contains(&format!("class=\"pele-label\" fill=\"{}\"", p.text)), "{one}");
    // The composite is the canvas tinted three times at 0.2.
    let tint = zebra_mermaid::theme::color::mix_hex;
    let under = tint("#FFFFFF", &tint("#FFFFFF", &tint("#FFFFFF", &p.background, 0.2), 0.2), 0.2);
    assert_eq!(contrasting_text_on_fill(&under, &p.text, &p.background, 1.0), p.background);
    assert_no_css_runtime(&r.svg);
}

#[test]
fn author_styles_keep_precedence() {
    let p = sentinel();
    for options in [resolved_options(Some(p.clone())), RenderOptions::default()] {
        let r = render(STYLED, &options).unwrap();
        // A styled overlap is outlined, unfilled, in the author's colors only.
        let overlap = element(&r.svg, "<path class=\"pele-venn-overlap\" data-id=\"A|B|C\"");
        assert!(overlap.ends_with(" fill=\"none\" style=\"fill:#123456;stroke:#654321;\"/>"), "{overlap}");
        // A set's text color is a style, which wins over the fill attribute.
        let label = element(&r.svg, "<g class=\"pele-venn-region\" data-id=\"A\"><text");
        assert!(label.contains(" style=\"fill:#0A0B0C;\""), "{label}");
    }
}

#[test]
fn css_mode_is_pele() {
    let r = render(SETS, &RenderOptions::default()).unwrap();
    let circle = element(&r.svg, "<circle class=\"pele-venn-set\" data-id=\"A\"");
    assert!(circle.contains("fill=\"var(--pele-series-1,#4c78a8)\" fill-opacity=\"0.2\" stroke=\"var(--pele-series-1,#4c78a8)\""));
    assert!(r.svg.contains("fill=\"var(--_m)\""));
    // Region labels inherit their color, as in Pele.
    assert!(r.svg.contains("<g class=\"pele-venn-region\" data-id=\"A\"><text class=\"pele-label\" x="));
    assert!(!r.svg.contains("pele-canvas"));
    let p = sentinel();
    let themed = render(SETS, &RenderOptions { palette: Some(p.clone()), ..Default::default() }).unwrap();
    assert!(themed.svg.contains(&format!("var(--pele-series-1,{})", series(&p)[0])));
    assert_eq!((themed.width, themed.height), (r.width, r.height));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [SETS, STYLED] {
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
fn palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    for source in [SETS, STYLED] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        let rn = render(source, &resolved_options(None)).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        assert_eq!(without_colors(&ra.svg), without_colors(&rn.svg));
        assert_no_css_runtime(&rb.svg);
        assert_no_css_runtime(&rn.svg);
        let css = render(source, &RenderOptions::default()).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
    }
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [SETS, STYLED] {
        for theme_mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
            let r = render_for_zebra(source, &ZebraRenderOptions { theme_mode, ..Default::default() }).unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains("data-mermaid-canvas-background=\"opaque\""));
        }
    }
}
