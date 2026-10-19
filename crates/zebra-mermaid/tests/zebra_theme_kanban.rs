//! Zebra theme roles of kanban boards in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! A column is a group (group fill and outline), a card a node (node fill and outline), the
//! column's count, a ticket and an assignee secondary text, and a linked ticket the accent.
//! Pele marks the two high priorities in its critical color and the two low ones in the muted
//! color, the nearer to Medium the fainter, and Medium not at all; resolved mode keeps those
//! marks and paints them with the danger text and secondary text colors.

#![cfg(feature = "kanban")]

mod common;

use common::svg_policy;
use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::zebra::{self, render_for_zebra, ZebraRenderOptions};
use zebra_mermaid::{render, CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

/// Zebra's reference fixture (`renders_every_supported_diagram_family_in_both_themes`).
const ZEBRA: &str = "kanban\n todo[Draft]\n  t1[Write]\n done[Ready]\n  t2[Publish]";

const BOARD: &str = "kanban\n  todo[Draft]\n    t1[Write]@{ priority: 'Very High', ticket: T-1, assigned: bob }\n    t2[Edit]@{ priority: High }\n    t3[Read]@{ priority: Low }\n    t4[Nap]@{ priority: 'Very Low' }\n    t5[Plan]@{ priority: Medium }\n  done[Ready]\n    t6[Publish]\n";

const LINKED: &str = "---\nconfig:\n  kanban:\n    ticketBaseUrl: 'https://tickets.example/#TICKET#'\n---\nkanban\n  todo[Draft]\n    t1[Write]@{ ticket: T-1 }\n";

/// The element with `data-id="id"`, from its opening tag to the first `</g>` after it.
fn group<'a>(svg: &'a str, id: &str) -> &'a str {
    let at = svg.find(&format!("data-id=\"{id}\"")).unwrap_or_else(|| panic!("no element {id}"));
    let start = svg[..at].rfind("<g ").unwrap();
    let end = at + svg[at..].find("</g>").unwrap();
    &svg[start..end]
}

/// The priority mark of a card, if it has one.
fn mark<'a>(svg: &'a str, id: &str) -> Option<&'a str> {
    let card = group(svg, id);
    let at = card.find("<path class=\"pele-priority\"")?;
    Some(&card[at..at + card[at..].find("/>").unwrap()])
}

#[test]
fn kanban_consumes_its_roles() {
    let p = sentinel();
    let r = render(BOARD, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("fill=\"{}\"/>", p.background)), "opaque canvas");

    // Columns: group fill and outline; their count in the secondary text color.
    for id in ["todo", "done"] {
        let column = group(svg, id);
        assert!(column.starts_with("<g class=\"pele-cluster pele-column\""), "{column}");
        assert!(
            column.contains(&format!(
                "rx=\"4\" fill=\"{}\" stroke=\"{}\"/>",
                p.group_background.as_ref().unwrap(),
                p.group_border.as_ref().unwrap()
            )),
            "{column}"
        );
        assert!(column.contains("class=\"pele-count\""));
        assert!(column.contains(&format!("font-size=\"14\" fill=\"{}\">", p.secondary_text)));
    }

    // Cards: node fill and outline.
    for id in ["t1", "t2", "t3", "t4", "t5", "t6"] {
        assert!(group(svg, id).contains(&format!("rx=\"4\" fill=\"{}\" stroke=\"{}\"/>", p.surface, p.border)), "{id}");
    }

    // Priorities: the high ones in the danger text color, the low ones in the secondary text color.
    assert_eq!(mark(svg, "t1").map(|m| m.contains(&format!("fill=\"none\" stroke=\"{}\" stroke-width", p.danger_text))), Some(true));
    assert_eq!(mark(svg, "t2").map(|m| m.contains(&format!("stroke=\"{}\" opacity=\"0.5\"", p.danger_text))), Some(true));
    assert_eq!(mark(svg, "t3").map(|m| m.contains(&format!("stroke=\"{}\" opacity=\"0.5\"", p.secondary_text))), Some(true));
    assert_eq!(mark(svg, "t4").map(|m| m.contains(&format!("fill=\"none\" stroke=\"{}\" stroke-width", p.secondary_text))), Some(true));
    assert_eq!(mark(svg, "t5"), None, "Medium has no mark, as in Mermaid and Pele");
    assert_eq!(mark(svg, "t6"), None);

    // Ticket and assignee in the secondary text color.
    let t1 = group(svg, "t1");
    assert!(t1.contains(&format!("class=\"pele-ticket\" fill=\"{}\"", p.secondary_text)));
    assert!(t1.contains(&format!("class=\"pele-assigned\" fill=\"{}\"", p.secondary_text)));

    // Roles a board does not use (the accent only colors a linked ticket).
    let mut unused = vec![
        &p.surface_secondary,
        &p.accent,
        &p.selection,
        &p.warning_background,
        &p.warning_text,
        &p.danger_background,
        &p.success_background,
        &p.success_text,
        p.edge.as_ref().unwrap(),
    ];
    unused.extend(p.series.as_ref().unwrap().iter());
    for color in unused {
        assert!(!svg.contains(color.as_str()), "kanban uses {color}");
    }
}

#[test]
fn a_linked_ticket_takes_the_accent() {
    let p = sentinel();
    let r = render(LINKED, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    assert!(r.svg.contains("<a href=\"https://tickets.example/T-1\""));
    assert!(r.svg.contains(&format!("class=\"pele-ticket\" fill=\"{}\"", p.accent)));
    assert_eq!(r.links.len(), 1);

    // Zebra's hosts take no links: the ticket is plain secondary text there.
    for theme_mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let z = render_for_zebra(LINKED, &ZebraRenderOptions { theme_mode, palette: Some(p.clone()), ..Default::default() }).unwrap();
        svg_policy::validate(&z.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap();
        assert!(!z.svg.contains("<a "));
        assert!(z.svg.contains(&format!("class=\"pele-ticket\" fill=\"{}\"", p.secondary_text)));
    }
}

#[test]
fn without_group_colors_columns_take_the_canvas_and_border() {
    let mut p = sentinel();
    p.group_background = None;
    p.group_border = None;
    let r = render(BOARD, &resolved_options(Some(p.clone()))).unwrap();
    assert!(group(&r.svg, "todo").contains(&format!("fill=\"{}\" stroke=\"{}\"/>", p.background, p.border)));
}

#[test]
fn only_colors_change_with_the_palette() {
    let a = sentinel();
    let b = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    let ra = render(BOARD, &resolved_options(Some(a))).unwrap();
    let rb = render(BOARD, &resolved_options(Some(b))).unwrap();
    assert_ne!(ra.svg, rb.svg);
    assert_eq!((ra.width, ra.height), (rb.width, rb.height));
    assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
    let css = render(BOARD, &RenderOptions::default()).unwrap();
    assert_eq!((css.width, css.height), (ra.width, ra.height));
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(BOARD, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear =
        render(BOARD, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
            .unwrap();
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
}

#[test]
fn zebra_reference_fixture_in_light_and_dark() {
    let mut sizes = Vec::new();
    let mut svgs = Vec::new();
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(ZEBRA, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert!(r.width > 0.0 && r.height > 0.0);
        assert!(r.svg.contains("<svg"));
        assert!(!r.svg.contains("color-mix("));
        assert!(!r.svg.contains("@import"));
        assert_no_css_runtime(&r.svg);
        let defaults = MermaidPalette::default_for(theme);
        assert!(r.svg.contains(&format!("fill=\"{}\" stroke=\"{}\"/>", defaults.surface, defaults.border)), "cards");
        sizes.push((r.width, r.height));
        svgs.push(r.svg);

        let z = render_for_zebra(ZEBRA, &ZebraRenderOptions { theme_mode: theme, ..Default::default() }).unwrap();
        svg_policy::validate(&z.svg, zebra::MAX_OUTPUT_UTF8_BYTES).unwrap();
        assert_no_css_runtime(&z.svg);
    }
    assert_ne!(svgs[0], svgs[1]);
    assert_eq!(sizes[0], sizes[1]);
}
