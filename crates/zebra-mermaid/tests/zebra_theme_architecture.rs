#![cfg(feature = "architecture")]

//! Zebra theme compatibility for architecture diagrams: each colored part takes its semantic
//! role in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

/// Groups, a nested group, services with built-in icons and icon text, a junction, a titled
/// edge and arrowheads.
const RICH: &str = "architecture-beta
  title Shop
  group api(cloud)[API]
  group inner[Inner] in api
  service db(database)[Database] in api
  service server(server)[Server] in inner
  service cache\"C$\"[Cache]
  junction j
  db:L -[reads]- R:server
  server:B --> T:j
  j:R <-- L:cache
";

/// Zebra's own fixtures (`renderer.rs`).
const ZEBRA_GROUP: &str = "architecture-beta\n group app(cloud)[App]\n service editor(server)[Editor] in app";
const ZEBRA_FIXTURE: &str = "architecture-beta\n group app(cloud)[Zebra]\n service editor(server)[Editor] in app\n service store(database)[Store] in app\n editor:R --> L:store";

/// Every tag that starts with `prefix`, up to its closing `>`.
fn tags<'a>(svg: &'a str, prefix: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(prefix) {
        let start = from + k;
        let end = start + svg[start..].find('>').unwrap() + 1;
        out.push(&svg[start..end]);
        from = end;
    }
    out
}

/// The first tag after `marker`.
fn tag_after<'a>(svg: &'a str, marker: &str) -> &'a str {
    let at = svg.find(marker).unwrap_or_else(|| panic!("no {marker} in {svg}")) + marker.len();
    let start = at + svg[at..].find('<').unwrap();
    let end = start + svg[start..].find('>').unwrap() + 1;
    &svg[start..end]
}

#[test]
fn architecture_consumes_its_roles() {
    let p = sentinel();
    let t = ResolvedTheme::from_palette(&p);
    let r = render(RICH, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    // Root text color and the opaque canvas.
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("color:{}", p.text)));
    assert!(svg.contains(&format!(
        "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
        r.width, r.height, p.background
    )));

    // Groups: Zebra's group fill and border, the fill not thinned, titles in secondary text.
    let group_border = p.group_border.clone().unwrap();
    let group_fill = p.group_background.clone().unwrap();
    for id in ["api", "inner"] {
        let rect = tag_after(svg, &format!("<g class=\"pele-cluster\" data-id=\"{id}\">"));
        assert!(rect.starts_with("<rect "), "{rect}");
        assert!(rect.contains(&format!(" fill=\"{group_fill}\" stroke=\"{group_border}\"")), "{rect}");
        assert!(!rect.contains("fill-opacity"), "{rect}");
    }
    let titles = tags(svg, "<text class=\"pele-cluster-label\"");
    assert_eq!(titles.len(), 2);
    for title in titles {
        assert!(title.contains(&format!("fill=\"{}\"", p.secondary_text)), "{title}");
    }

    // Services: the node surface and border.
    for id in ["db", "server", "cache"] {
        let rect = tag_after(svg, &format!("data-id=\"{id}\" transform="));
        assert!(rect.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.surface, p.border)), "{rect}");
    }
    // Built-in icons, in services and in the group badge, take the first series color.
    let series = p.series.clone().unwrap();
    let glyphs = tags(svg, "<path d=\"M");
    let glyphs: Vec<_> = glyphs.iter().filter(|g| g.contains("stroke-width=\"1.5\"")).collect();
    assert_eq!(glyphs.len(), 3, "cloud, database and server");
    for g in glyphs {
        assert!(g.contains(&format!("stroke=\"{}\"", series[0])), "{g}");
    }
    // Icon text and service titles keep the inherited text color.
    assert!(svg.contains("<text class=\"pele-icon-text\" font-size=\"12\" x="));

    // Edges in the edge color, arrowheads in the arrow (accent) color, junctions as edges.
    assert!(svg.contains(&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"{}\"", t.line)));
    let markers = tags(svg, "<path class=\"pele-marker\"");
    assert_eq!(markers.len(), 2);
    for m in markers {
        assert!(m.contains(&format!("fill=\"{}\" stroke=\"none\"", p.accent)), "{m}");
    }
    let junction = tag_after(svg, "<g class=\"pele-node pele-junction\" data-id=\"j\"");
    assert!(junction.contains(&format!("fill=\"{}\"", t.line)), "{junction}");

    // Edge labels: on a patch of the canvas, in the secondary text color.
    assert!(svg.contains(&format!("<g class=\"pele-edge-labels\" font-size=\"14\" fill=\"{}\">", p.secondary_text)));
    let patch = tag_after(svg, "<g class=\"pele-edge-label\" data-id=\"db-server\">");
    assert!(patch.contains(&format!("rx=\"3\" fill=\"{}\"", p.background)), "{patch}");

    // Roles an architecture diagram does not use.
    for unused in [
        &p.surface_secondary,
        &p.selection,
        &p.warning_background,
        &p.warning_text,
        &p.danger_background,
        &p.danger_text,
        &p.success_background,
        &p.success_text,
        &t.muted_text,
        &t.faint_text,
    ] {
        assert!(!svg.contains(unused.as_str()), "architecture uses {unused}");
    }
    for s in &series[1..] {
        assert!(!svg.contains(s.as_str()), "architecture uses series {s}");
    }
}

#[test]
fn css_mode_keeps_pele_colors() {
    let r = render(ZEBRA_FIXTURE, &RenderOptions::default()).unwrap();
    assert!(r.svg.contains("fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\""));
    assert!(r.svg.contains("fill=\"var(--_s)\" stroke=\"var(--_b)\""));
    assert!(r.svg.contains("stroke=\"var(--_fg)\" stroke-width=\"1.5\""));
    assert!(r.svg.contains("fill=\"var(--_l)\" stroke=\"none\""));
    assert!(r.svg.contains("<text class=\"pele-cluster-label\" fill=\"var(--_m)\""));
}

#[test]
fn zebra_semantic_palette_reaches_architecture() {
    // Zebra's `semantic_palette_reaches_chart_layouts_and_contrasting_pie_labels`.
    let mut palette = MermaidPalette::dark();
    palette.background = "#101010".into();
    palette.text = "#F0F0F0".into();
    palette.edge = Some("#72B3C4".into());
    palette.group_background = Some("#243546".into());
    palette.group_border = Some("#8394A5".into());
    palette.series = Some(["#EFEFEF", "#252525", "#AB3456", "#3478AB", "#56AB78", "#AB8934"].map(String::from));
    let options = resolved_options(Some(palette));
    let single = render("architecture-beta\n service editor(server)[Editor]", &options).unwrap();
    assert!(single.svg.contains("#EFEFEF"), "architecture must use the supplied first series");
    assert!(!single.svg.contains("#131300"), "architecture must not retain fixed dark label text");
    let grouped = render(ZEBRA_GROUP, &options).unwrap();
    assert!(grouped.svg.contains("stroke=\"#8394A5\""), "architecture groups must use semantic borders");
    assert!(grouped.svg.contains("fill=\"#243546\""), "architecture groups must use the group background");
    assert_no_css_runtime(&grouped.svg);
}

#[test]
fn zebra_fixture_renders_in_both_themes() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        for color_mode in [ColorMode::Css, ColorMode::Resolved] {
            let r = render(ZEBRA_FIXTURE, &RenderOptions { theme, color_mode, ..Default::default() }).unwrap();
            assert!(r.width > 0.0 && r.height > 0.0);
            assert!(r.svg.contains("<svg"));
            assert!(!r.svg.contains("color-mix("));
            assert!(!r.svg.contains("@import"));
            if color_mode == ColorMode::Resolved {
                assert_no_css_runtime(&r.svg);
            }
        }
        let z = render_for_zebra(ZEBRA_FIXTURE, &ZebraRenderOptions { theme_mode: theme, ..Default::default() }).unwrap();
        svg_policy::validate(&z.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap_or_else(|e| panic!("{theme:?}: {e}"));
        assert_no_css_runtime(&z.svg);
    }
}

#[test]
fn without_series_icons_take_the_accent() {
    let mut p = sentinel();
    p.series = None;
    let r = render(ZEBRA_FIXTURE, &resolved_options(Some(p.clone()))).unwrap();
    assert!(r.svg.contains(&format!("stroke=\"{}\" stroke-width=\"1.5\"", p.accent)));
}

#[test]
fn host_icons_keep_their_own_colors() {
    let mut table = zebra_mermaid::IconTable::new();
    table.insert("logos:aws-s3", "<circle cx=\"12\" cy=\"12\" r=\"4\"/>");
    let options = RenderOptions { icons: Some(std::sync::Arc::new(table)), ..resolved_options(Some(sentinel())) };
    let r = render("architecture-beta\n service s3(logos:aws-s3)[S3]", &options).unwrap();
    // Drawn in currentColor, which the root sets to the text color.
    assert!(r.svg.contains("stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"12\" cy=\"12\" r=\"4\"/></svg>"));
    assert!(!r.svg.contains(&sentinel().series.unwrap()[0]));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    for source in [RICH, ZEBRA_FIXTURE] {
        let p = sentinel();
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
    // The edge label patch keeps the canvas color on a transparent canvas.
    let p = sentinel();
    let clear =
        render(RICH, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert!(clear.svg.contains(&format!("rx=\"3\" fill=\"{}\"", p.background)));
}

#[test]
fn palettes_change_colors_only() {
    let a = sentinel();
    let b = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    for source in [RICH, ZEBRA_FIXTURE, ZEBRA_GROUP] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        let rd = render(source, &RenderOptions { theme: MermaidThemeMode::Dark, ..resolved_options(None) }).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        assert_eq!(without_colors(&ra.svg), without_colors(&rd.svg));
        // The CSS output has the same geometry as the resolved one.
        let css = render(source, &RenderOptions::default()).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
    }
}
