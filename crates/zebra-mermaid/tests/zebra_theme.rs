#![cfg(all(feature = "pie", feature = "info"))]

//! Zebra theme compatibility (docs/ZEBRA-THEME-COMPATIBILITY.md, docs/TEST-PLAN.md §5) for the
//! diagram types this build renders.

mod common;

use common::svg_policy;
use common::theme::{sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::zebra::{self, render_abi_v2, render_for_zebra, ZebraRenderOptions, ZebraStatus};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

fn resolved(palette: Option<MermaidPalette>) -> RenderOptions {
    RenderOptions { color_mode: ColorMode::Resolved, palette, ..Default::default() }
}

const PIE: &str = "pie title Pets\n  \"Dogs\" : 40\n  \"Cats\" : 30\n  \"Rats\" : 20\n  \"Bats\" : 6\n  \"Ants\" : 3\n  \"Owls\" : 1\n";

fn assert_no_css_runtime(svg: &str) {
    for needle in ["var(", "color-mix(", "light-dark(", "@media", "@import", "--_", "--pele-"] {
        assert!(!svg.contains(needle), "resolved SVG contains {needle}: {svg}");
    }
}

/// The attribute value that follows `prefix`.
fn attr_after<'a>(svg: &'a str, prefix: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(prefix) {
        let start = from + k + prefix.len();
        let end = svg[start..].find('"').map(|e| start + e).unwrap();
        out.push(&svg[start..end]);
        from = end;
    }
    out
}

#[test]
fn pie_consumes_its_roles() {
    let p = sentinel();
    let r = render(PIE, &resolved(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    // Root text color and the opaque canvas.
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("color:{}", p.text)));
    assert!(svg.contains(&format!(
        "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
        r.width, r.height, p.background
    )));
    // Slices: the six series in order, stroked with the background.
    let series = p.series.clone().unwrap();
    let slice_fills = attr_after(svg, "class=\"pele-slice\" data-id=\"");
    assert_eq!(slice_fills.len(), 6);
    for (i, s) in series.iter().enumerate() {
        assert!(svg.contains(&format!("fill=\"{}\"/>", s)), "slice {i}");
    }
    assert!(svg.contains("class=\"pele-slices\" transform=\"translate("));
    assert!(svg.contains(&format!("stroke=\"{}\" stroke-width=\"1\" stroke-linejoin=\"round\"", p.background)));
    // Legend swatches take the same series; legend text the text color (series given).
    for s in &series {
        assert!(svg.contains(&format!("rx=\"2\" fill=\"{}\"", s)));
    }
    assert!(svg.contains(&format!("class=\"pele-legend\" font-size=\"14\" fill=\"{}\"", p.text)));
    // Labels inside slices read on their slice: dark text on light slices, light on dark.
    for fill in [&series[0], &series[1], &series[3]] {
        let expected = contrasting_text_on_fill(fill, &p.text, &p.background, 1.0);
        assert!(svg.contains(&format!(" fill=\"{}\">", expected)), "label on {fill}");
    }
    assert_eq!(contrasting_text_on_fill(&series[1], &p.text, &p.background, 1.0), p.background, "dark slice takes the canvas color");
    assert_eq!(contrasting_text_on_fill(&series[3], &p.text, &p.background, 1.0), p.text, "light slice takes the text color");
    // Roles a pie does not use must not leak in.
    for unused in [
        &p.surface,
        &p.surface_secondary,
        &p.secondary_text,
        &p.accent,
        &p.border,
        &p.selection,
        &p.warning_background,
        &p.warning_text,
        &p.danger_background,
        &p.danger_text,
        &p.success_background,
        &p.success_text,
    ] {
        assert!(!svg.contains(unused.as_str()), "pie uses {unused}");
    }
    for unused in [p.edge.as_ref().unwrap(), p.group_background.as_ref().unwrap(), p.group_border.as_ref().unwrap()] {
        assert!(!svg.contains(unused.as_str()), "pie uses {unused}");
    }
}

#[test]
fn empty_pie_outlines_with_the_border() {
    let p = sentinel();
    let r = render("pie\n  \"Zero\" : 0\n", &resolved(Some(p.clone()))).unwrap();
    assert!(r.svg.contains(&format!("fill=\"none\" stroke=\"{}\"", p.border)));
    assert_no_css_runtime(&r.svg);
}

#[test]
fn info_consumes_its_roles() {
    let p = sentinel();
    let r = render("---\ntitle: About\n---\ninfo", &resolved(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    assert!(r.svg.contains(&format!("class=\"pele-version\" fill=\"{}\"", p.secondary_text)));
    assert!(r.svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(r.svg.contains(&format!("fill=\"{}\"/>", p.background)));
    // The title is bold through an attribute, not a custom property.
    assert!(r.svg.contains("font-weight=\"bold\""));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    for source in [PIE, "info"] {
        let p = sentinel();
        let opaque =
            render(source, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved(Some(p.clone())) }).unwrap();
        let clear =
            render(source, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved(Some(p.clone())) }).unwrap();
        assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
        let rect = format!(
            "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            opaque.width, opaque.height, p.background
        );
        assert!(opaque.svg.contains(&rect));
        assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
        if source == PIE {
            // The background still strokes the slices and colors labels on transparent output.
            assert!(clear.svg.contains(&format!("stroke=\"{}\"", p.background)));
        }
    }
}

#[test]
fn css_mode_without_palette_is_pele_and_opaque_adds_a_canvas() {
    let pele = render(PIE, &RenderOptions::default()).unwrap();
    assert!(!pele.svg.contains("pele-canvas"));
    assert!(pele.svg.contains("--_bg:var(--pele-bg,#fff)"));
    let opaque = render(PIE, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..Default::default() }).unwrap();
    assert!(opaque.svg.contains("fill=\"var(--_bg)\"/>"));
    // A palette in CSS mode becomes the fallback of each custom property.
    let p = sentinel();
    let themed = render(PIE, &RenderOptions { palette: Some(p.clone()), ..Default::default() }).unwrap();
    assert!(themed.svg.contains(&format!("--_bg:var(--pele-bg,{})", p.background)));
    assert!(themed.svg.contains(&format!("var(--pele-series-1,{})", p.series.as_ref().unwrap()[0])));
    assert_eq!((themed.width, themed.height), (pele.width, pele.height));
}

#[test]
fn light_and_dark_defaults_without_palette() {
    let light = render(PIE, &RenderOptions { color_mode: ColorMode::Resolved, ..Default::default() }).unwrap();
    let dark =
        render(PIE, &RenderOptions { color_mode: ColorMode::Resolved, theme: MermaidThemeMode::Dark, ..Default::default() }).unwrap();
    assert!(light.svg.contains("fill=\"#FFFFFF\"/>"));
    assert!(dark.svg.contains("fill=\"#18181B\"/>"));
    assert_ne!(light.svg, dark.svg);
    assert_eq!((light.width, light.height), (dark.width, dark.height));
    // Without series, slices take Zebra's derived chart colors, starting with the accent.
    let accent = MermaidPalette::beautiful_light().accent;
    assert!(light.svg.contains(&format!("fill=\"{}\"/>", accent)));
    assert_no_css_runtime(&light.svg);
    assert_no_css_runtime(&dark.svg);
}

#[test]
fn switching_palettes_never_reuses_colors_and_keeps_geometry() {
    let a = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    let mut b = a.clone();
    b.series.as_mut().unwrap()[0] = "#123456".into();
    let mut c = a.clone();
    c.background = "#FEFEFE".into();
    let dark = MermaidPalette::dark().swift_completed();
    let renders: Vec<_> = [&a, &b, &a, &c, &dark, &a].iter().map(|p| render(PIE, &resolved(Some((*p).clone()))).unwrap()).collect();
    assert_eq!(renders[0].svg, renders[2].svg);
    assert_eq!(renders[0].svg, renders[5].svg);
    assert_ne!(renders[0].svg, renders[1].svg);
    assert!(renders[1].svg.contains("#123456"));
    assert_ne!(renders[0].svg, renders[3].svg);
    assert_ne!(renders[0].svg, renders[4].svg);
    for r in &renders[1..] {
        assert_eq!((r.width, r.height), (renders[0].width, renders[0].height));
    }
    // Only colors differ (labels may change color, never place).
    assert_eq!(without_colors(&renders[0].svg), without_colors(&renders[1].svg));
    assert_eq!(without_colors(&renders[0].svg), without_colors(&renders[3].svg));
    // The identity follows every color, the font size and the canvas.
    let ids: Vec<String> = [&a, &b, &c].iter().map(|p| resolved(Some((*p).clone())).render_identity()).collect();
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
    let base = resolved(Some(a.clone()));
    assert_eq!(base.render_identity(), resolved(Some(a.clone())).render_identity());
    assert_ne!(base.render_identity(), RenderOptions { font_size: Some(17.0), ..base.clone() }.render_identity());
    assert_ne!(
        base.render_identity(),
        RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..base.clone() }.render_identity()
    );
    assert_ne!(base.render_identity(), RenderOptions { theme: MermaidThemeMode::Dark, ..base.clone() }.render_identity());
}

#[test]
fn font_size_changes_measurement() {
    let small = render(PIE, &resolved(None)).unwrap();
    let large = render(PIE, &RenderOptions { font_size: Some(24.0), ..resolved(None) }).unwrap();
    assert!(large.width > small.width);
}

#[test]
fn zebra_pie_contrast_cases() {
    // Zebra's renderer tests, with slices drawn opaque as Pele draws them. Zebra's renderer
    // drew slices at 0.92 opacity; the contrast choice is computed on what is drawn.
    let mut p = MermaidPalette::dark();
    p.background = "#101010".into();
    p.text = "#F0F0F0".into();
    p.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let r = render("pie\n \"Bright\" : 50\n \"Dark\" : 50", &resolved(Some(p))).unwrap();
    assert!(r.svg.contains("fill=\"#101010\">50%</text>"), "bright slice takes the canvas color");
    assert!(r.svg.contains("fill=\"#F0F0F0\">50%</text>"), "dark slice takes the text color");

    let mut p = MermaidPalette::light();
    p.background = "#FAE7C9".into();
    p.text = "#322A20".into();
    p.series = Some(["#5E7E51"; 6].map(String::from));
    let r = render("pie\n \"Parchment green\" : 100", &resolved(Some(p.clone()))).unwrap();
    assert!(r.svg.contains("fill=\"#5E7E51\""));
    let at_full = contrasting_text_on_fill("#5E7E51", &p.text, &p.background, 1.0);
    let at_092 = contrasting_text_on_fill("#5E7E51", &p.text, &p.background, 0.92);
    assert_eq!(at_092, "#000000", "Zebra's expectation at its 0.92 opacity");
    // Pele draws slices opaque. On the color actually drawn the same rule picks white: both
    // black (4.6:1) and white pass 4.5:1 on this mid-tone and white is slightly ahead. This
    // visual-policy change is recorded in docs/IMPLEMENTATION-STATUS.md.
    assert_eq!(at_full, "#FFFFFF");
    assert!(r.svg.contains(&format!("fill=\"{at_full}\">100%</text>")));
    assert!(!r.svg.contains("opacity=\"0.92"));
}

#[test]
fn zebra_requests_pass_zebra_svg_validation() {
    for source in [PIE, "info", "pie showData\n title Links\n \"a\" : 1"] {
        for dark in [false, true] {
            let r = render_for_zebra(
                source,
                &ZebraRenderOptions {
                    theme_mode: if dark { MermaidThemeMode::Dark } else { MermaidThemeMode::Light },
                    ..Default::default()
                },
            )
            .unwrap();
            svg_policy::validate(&r.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains("data-mermaid-canvas-background=\"opaque\""));
        }
    }
}

#[test]
fn abi_v2_ignores_target_width_and_keeps_its_errors() {
    let palette = include_bytes!("../../../docs/research/zebra-theme-palette.example.json");
    let narrow = render_abi_v2(PIE.as_bytes(), false, Some(palette), 16.0, 120.0, 0).unwrap();
    let wide = render_abi_v2(PIE.as_bytes(), false, Some(palette), 16.0, 4000.0, 0).unwrap();
    assert_eq!(narrow, wide, "geometry must not depend on target_width");
    let transparent = render_abi_v2(PIE.as_bytes(), false, Some(palette), 16.0, 640.0, 1).unwrap();
    assert!(!transparent.svg.contains("data-mermaid-canvas-background=\"opaque\""));
    assert_eq!((transparent.width, transparent.height), (wide.width, wide.height));

    let err = |r: Result<zebra::ZebraRendered, (ZebraStatus, String)>| r.unwrap_err();
    assert_eq!(
        err(render_abi_v2(PIE.as_bytes(), false, None, 0.5, 640.0, 0)),
        (ZebraStatus::InvalidArgument, "Invalid Mermaid layout metrics.".into())
    );
    assert_eq!(err(render_abi_v2(PIE.as_bytes(), false, None, 16.0, 0.0, 0)).0, ZebraStatus::InvalidArgument);
    assert_eq!(err(render_abi_v2(PIE.as_bytes(), false, None, 16.0, f64::NAN, 0)).0, ZebraStatus::InvalidArgument);
    assert_eq!(err(render_abi_v2(b"", false, None, 16.0, 640.0, 0)), (ZebraStatus::InvalidArgument, "Invalid Mermaid source.".into()));
    assert_eq!(
        err(render_abi_v2(&[0xff, 0xfe], false, None, 16.0, 640.0, 0)),
        (ZebraStatus::InvalidUtf8, "Invalid Mermaid source.".into())
    );
    assert_eq!(
        err(render_abi_v2(PIE.as_bytes(), false, Some(b"{}"), 16.0, 640.0, 0)),
        (ZebraStatus::InvalidArgument, "Invalid Mermaid palette.".into())
    );
    let lower = String::from_utf8_lossy(palette).replace("#2563EB", "#2563eb");
    assert_eq!(
        err(render_abi_v2(PIE.as_bytes(), false, Some(lower.as_bytes()), 16.0, 640.0, 0)),
        (ZebraStatus::InvalidArgument, "Invalid Mermaid palette.".into())
    );
    assert_eq!(
        err(render_abi_v2(PIE.as_bytes(), false, None, 16.0, 640.0, 2)),
        (ZebraStatus::InvalidArgument, "Invalid Mermaid canvas background policy.".into())
    );
    assert_eq!(err(render_abi_v2(b"pie\n \"a\" 1", false, None, 16.0, 640.0, 0)).0, ZebraStatus::ParseFailure);
    assert_eq!(err(render_abi_v2(b"notadiagram", false, None, 16.0, 640.0, 0)).0, ZebraStatus::ParseFailure);
    // An empty palette buffer means no palette, as in the ABI.
    assert!(render_abi_v2(PIE.as_bytes(), true, Some(b""), 16.0, 640.0, 0).unwrap().svg.contains("#18181B"));
}

#[test]
fn invalid_palette_in_library_options_is_an_options_error() {
    let mut p = sentinel();
    p.accent = "#abcdef".into();
    let e = render(PIE, &resolved(Some(p))).unwrap_err();
    assert_eq!(e.code, zebra_mermaid::ErrorCode::InvalidOptions);
}
