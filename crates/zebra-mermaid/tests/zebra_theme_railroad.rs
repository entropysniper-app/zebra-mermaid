#![cfg(feature = "railroad")]

//! Zebra theme roles in railroad diagrams (docs/ZEBRA-THEME-COMPATIBILITY.md). Each part
//! takes its role's color in resolved mode:
//!
//! | part                                   | role           | Pele literal |
//! | -------------------------------------- | -------------- | ------------ |
//! | canvas                                 | Background     | `var(--_bg)` |
//! | rule names, title, box labels          | Text (root)    | `var(--_fg)` |
//! | track path                             | Line           | `var(--_l)`  |
//! | start and end markers of a track       | Line           | `var(--_l)`  |
//! | direction arrowheads (loops, folds)    | Arrow          | `var(--_l)`  |
//! | terminal fill / outline                | NodeFill / NodeStroke | `var(--_s)` / `var(--_b)` |
//! | non-terminal fill / outline            | GroupHeader / NodeStroke | `var(--_bg)` / `var(--_b)` |
//! | special sequence fill / dashed outline | Background / NodeStroke | `var(--_bg)` / `var(--_b)` |
//! | special sequence `?` marks             | SecondaryText  | `var(--_m)`  |
//! | repetition count                       | SecondaryText  | `var(--_m)`  |

mod common;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

/// Every kind of part: terminals, non-terminals, a special sequence, a counted loop, an
/// uncounted loop, a choice, an optional bypass, and (at a narrow width) a folded sequence.
const GRAMMAR: &str =
    "railroad-ebnf-beta\ntitle Parts\nexpr = term , { (\"+\" | \"-\") , term } , [ ? any text ? ] ;\nterm = \"a\" , \"b\"+ ;\n";
const COUNTED: &str = "railroad-abnf-beta\nr = 2*5\"x\" 1*name ;\n";
const LONG: &str = "railroad-ebnf-beta\nlong = \"one\" , \"two\" , \"three\" , \"four\" , \"five\" , \"six\" , \"seven\" , \"eight\" ;\n";

/// The text of every element that starts with `start`, up to its end `>`.
fn tags<'a>(svg: &'a str, start: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(start) {
        let at = from + k;
        let end = svg[at..].find('>').map(|e| at + e + 1).unwrap();
        out.push(&svg[at..end]);
        from = end;
    }
    out
}

/// The element that follows `marker` (the next `<tag ...>` after it).
fn next_tag<'a>(svg: &'a str, marker: &str, tag: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(k) = svg[from..].find(marker) {
        let at = from + k + marker.len();
        let start = svg[at..].find(tag).map(|s| at + s).unwrap();
        let end = svg[start..].find('>').map(|e| start + e + 1).unwrap();
        out.push(&svg[start..end]);
        from = end;
    }
    out
}

fn resolved_theme(p: &MermaidPalette) -> ResolvedTheme {
    ResolvedTheme::from_palette(p)
}

#[test]
fn railroad_consumes_its_roles() {
    let p = sentinel();
    let edge = p.edge.clone().unwrap();
    for source in [GRAMMAR, COUNTED] {
        let r = render(source, &resolved_options(Some(p.clone()))).unwrap();
        let svg = &r.svg;
        assert_no_css_runtime(svg);
        // Root text color, inherited by rule names, the title and box labels.
        assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
        assert!(svg.contains(&format!("color:{}", p.text)));
        // The opaque canvas.
        assert!(svg.contains(&format!(
            "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
            r.width, r.height, p.background
        )));
        // Tracks are lines; their start and end markers too.
        let tracks = tags(svg, "<path class=\"pele-edge pele-track\"");
        assert!(!tracks.is_empty());
        for t in &tracks {
            assert!(t.ends_with(&format!("fill=\"none\" stroke=\"{}\"/>", edge)), "{t}");
        }
        let markers = tags(svg, "<circle class=\"pele-marker");
        assert_eq!(markers.len(), 2 * tracks.len());
        for c in &markers {
            assert!(c.ends_with(&format!("fill=\"{}\"/>", edge)), "{c}");
        }
        // Arrowheads take the arrow color.
        let arrows = tags(svg, "<path class=\"pele-marker\"");
        assert!(!arrows.is_empty());
        for a in &arrows {
            assert!(a.ends_with(&format!("fill=\"{}\"/>", p.accent)), "{a}");
        }
        // Terminals are filled nodes.
        let terminals = next_tag(svg, "<g class=\"pele-node pele-terminal\">", "<rect");
        assert!(!terminals.is_empty());
        for t in &terminals {
            assert!(t.contains(&format!("fill=\"{}\" stroke=\"{}\"", p.surface, p.border)), "{t}");
        }
        // Non-terminals are set apart from terminals by the secondary surface.
        let nonterminals = next_tag(svg, "<g class=\"pele-node pele-nonterminal\"", "<rect");
        assert!(!nonterminals.is_empty());
        for t in &nonterminals {
            assert!(t.contains(&format!("rx=\"4\" fill=\"{}\" stroke=\"{}\"", p.surface_secondary, p.border)), "{t}");
        }
        // The box labels inherit the text color; terminals are monospace.
        assert!(svg.contains("font-family=\"monospace\">"));
        for t in tags(svg, "<text x=") {
            assert!(!t.contains(" fill="), "{t}");
        }
        // Rule names are bold text in the text color.
        for t in tags(svg, "<text class=\"pele-rule-name\"") {
            assert!(t.contains("font-weight=\"bold\"") && !t.contains(" fill="), "{t}");
        }
    }

    let r = render(GRAMMAR, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    // The special sequence is a dashed annotation box on the canvas, with muted marks.
    let special = next_tag(svg, "<g class=\"pele-node pele-special\">", "<rect");
    assert_eq!(special.len(), 1);
    assert!(special[0].contains(&format!("fill=\"{}\" stroke=\"{}\" stroke-dasharray=\"4 3\"", p.background, p.border)));
    assert!(svg.contains(&format!(
        "font-style=\"italic\"><tspan fill=\"{0}\">? </tspan>any text<tspan fill=\"{0}\"> ?</tspan>",
        p.secondary_text
    )));
    // The title is bold.
    assert!(svg.contains("class=\"pele-title\" font-weight=\"bold\""));

    // Repetition counts are secondary text.
    let counted = render(COUNTED, &resolved_options(Some(p.clone()))).unwrap();
    let counts = tags(&counted.svg, "<text class=\"pele-edge-label pele-count\"");
    assert_eq!(counts.len(), 1);
    assert!(counts[0].ends_with(&format!("font-size=\"12\" fill=\"{}\">", p.secondary_text)));
    assert!(counted.svg.contains("2–5×</text>"));
}

#[test]
fn folded_sequences_point_their_arrows_with_the_arrow_color() {
    let p = sentinel();
    let options = RenderOptions { max_width: Some(260.0), ..resolved_options(Some(p.clone())) };
    let r = render(LONG, &options).unwrap();
    assert_no_css_runtime(&r.svg);
    // The rows of a folded sequence are joined by a track with an arrowhead at each row.
    let arrows = tags(&r.svg, "<path class=\"pele-marker\"");
    assert_eq!(arrows.len(), 1);
    assert!(arrows[0].contains("l7,3.5l-7,3.5z"));
    assert!(arrows[0].ends_with(&format!("fill=\"{}\"/>", p.accent)));
    assert!(r.width <= 260.0);
}

#[test]
fn roles_a_railroad_does_not_use_stay_out() {
    let p = sentinel();
    let t = resolved_theme(&p);
    let mut svg = String::new();
    for source in [GRAMMAR, COUNTED, LONG] {
        svg.push_str(&render(source, &resolved_options(Some(p.clone()))).unwrap().svg);
    }
    let unused = [
        ("selection", p.selection.clone()),
        ("warningBackground", p.warning_background.clone()),
        ("warningText", p.warning_text.clone()),
        ("dangerBackground", p.danger_background.clone()),
        ("dangerText", p.danger_text.clone()),
        ("successBackground", p.success_background.clone()),
        ("successText", p.success_text.clone()),
        ("groupBackground", p.group_background.clone().unwrap()),
        ("groupBorder", p.group_border.clone().unwrap()),
        ("mutedText", t.muted_text.clone()),
        ("faintText", t.faint_text.clone()),
        ("innerStroke", t.inner_stroke.clone()),
        ("groupFill", t.group_fill.clone()),
        ("groupStroke", t.group_stroke.clone()),
    ];
    for (name, color) in unused {
        if [&p.border, &p.background].contains(&&color) {
            continue;
        }
        assert!(!svg.contains(color.as_str()), "railroad uses {name} {color}");
    }
    for (i, s) in p.series.as_ref().unwrap().iter().enumerate() {
        assert!(!svg.contains(s.as_str()), "railroad uses series {i}");
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [GRAMMAR, COUNTED, LONG] {
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
    // The background still fills special sequences on a transparent canvas.
    let clear =
        render(GRAMMAR, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    assert!(clear.svg.contains(&format!("fill=\"{}\" stroke=\"{}\" stroke-dasharray", p.background, p.border)));
}

#[test]
fn only_colors_change_between_palettes() {
    let a = sentinel();
    let mut b = MermaidPalette::dark().swift_completed();
    b.edge = Some("#123456".into());
    for source in [GRAMMAR, COUNTED, LONG] {
        for width in [None, Some(260.0)] {
            let ra = render(source, &RenderOptions { max_width: width, ..resolved_options(Some(a.clone())) }).unwrap();
            let rb = render(source, &RenderOptions { max_width: width, ..resolved_options(Some(b.clone())) }).unwrap();
            assert_ne!(ra.svg, rb.svg);
            assert_eq!((ra.width, ra.height), (rb.width, rb.height));
            assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
            assert!(rb.svg.contains("stroke=\"#123456\""));
        }
    }
}

#[test]
fn css_mode_keeps_pele_literals_and_geometry() {
    let pele = render(GRAMMAR, &RenderOptions::default()).unwrap();
    assert!(pele.svg.contains("fill=\"var(--_s)\" stroke=\"var(--_b)\""));
    assert!(pele.svg.contains("fill=\"var(--_bg)\" stroke=\"var(--_b)\" stroke-dasharray=\"4 3\""));
    assert!(pele.svg.contains("fill=\"none\" stroke=\"var(--_l)\""));
    assert!(pele.svg.contains("<tspan fill=\"var(--_m)\">? </tspan>"));
    assert!(!pele.svg.contains("pele-canvas"));
    let resolved = render(GRAMMAR, &resolved_options(Some(sentinel()))).unwrap();
    assert_eq!((pele.width, pele.height), (resolved.width, resolved.height));
    // A palette in CSS mode only changes the fallbacks of the custom properties.
    let themed = render(GRAMMAR, &RenderOptions { palette: Some(sentinel()), ..Default::default() }).unwrap();
    assert!(themed.svg.contains(&format!("--_s:var(--pele-surface,{})", sentinel().surface)));
    assert_eq!((themed.width, themed.height), (pele.width, pele.height));
}

#[test]
fn light_and_dark_defaults_resolve_without_css() {
    let light = render(GRAMMAR, &RenderOptions { color_mode: ColorMode::Resolved, ..Default::default() }).unwrap();
    let dark =
        render(GRAMMAR, &RenderOptions { color_mode: ColorMode::Resolved, theme: MermaidThemeMode::Dark, ..Default::default() }).unwrap();
    assert_no_css_runtime(&light.svg);
    assert_no_css_runtime(&dark.svg);
    assert_ne!(light.svg, dark.svg);
    assert_eq!(without_colors(&light.svg), without_colors(&dark.svg));
    let dark_theme = resolved_theme(&MermaidPalette::dark());
    assert!(dark.svg.contains(&format!("fill=\"none\" stroke=\"{}\"", dark_theme.line)));
    assert!(dark.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"", dark_theme.node_fill, dark_theme.node_stroke)));
}
