#![cfg(feature = "treeView")]

//! Zebra theme roles of treeView diagrams in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! | Part | Role | Pele |
//! | --- | --- | --- |
//! | guide lines from a folder to its entries | Line | `var(--_b)` |
//! | highlighted row (`::: highlight`) | Selection | `var(--_c)` at 14% |
//! | built-in icon outline | SecondaryText | `var(--_m)` |
//! | description (`## ...`) | SecondaryText | `var(--_m)` |
//! | names, title | Text (root) | inherited |
//! | canvas | Background | `var(--_bg)` |

mod common;

use std::sync::Arc;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::{render, CanvasBackground, ColorMode, IconTable, MermaidPalette, MermaidThemeMode, RenderOptions};

const TREE: &str = "treeView-beta\n  title Files\n  src/ ::: highlight\n    main.rs ## entry point\n    lib.rs\n  README.md icon(file)\n  docs/ icon(folder)\n    guide.md\n";
const BOXES: &str = "treeView-beta\nproject/\n├── src/\n│   └── a.txt ## first\n└── b.txt\n";

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
fn treeview_consumes_its_roles() {
    let p = sentinel();
    // Loose, and tight at a narrow width, where descriptions go under their names.
    for options in [resolved_options(Some(p.clone())), RenderOptions { max_width: Some(120.0), ..resolved_options(Some(p.clone())) }] {
        let r = render(TREE, &options).unwrap();
        let svg = &r.svg;
        assert_no_css_runtime(svg);
        assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)), "root text");
        assert!(svg.contains(&format!(
            "data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"",
            r.width, r.height, p.background
        )));
        let guides = tags(svg, "class=\"pele-tree-guides\"");
        assert_eq!(guides.len(), 1);
        assert_eq!(attr(guides[0], "stroke"), p.edge.as_deref());
        // The highlighted row is filled with the selection color, opaque.
        let highlight = tags(svg, "class=\"pele-tree-highlight\"");
        assert_eq!(highlight.len(), 1);
        assert_eq!(attr(highlight[0], "fill"), Some(p.selection.as_str()));
        assert_eq!(attr(highlight[0], "fill-opacity"), None);
        // Built-in icons are outlined, descriptions written, in the secondary text color.
        let outlines = tags(svg, "stroke-width=\"1.5\"");
        assert_eq!(outlines.len(), 2);
        for o in outlines {
            assert_eq!(attr(o, "stroke"), Some(p.secondary_text.as_str()));
        }
        // A description that wraps is written a line to an element.
        let descriptions = tags(svg, "class=\"pele-tree-description\"");
        assert!(!descriptions.is_empty());
        for d in descriptions {
            assert_eq!(attr(d, "fill"), Some(p.secondary_text.as_str()));
        }
        // Roles a tree does not use.
        for unused in [
            &p.surface,
            &p.surface_secondary,
            &p.accent,
            &p.border,
            &p.warning_background,
            &p.warning_text,
            &p.danger_background,
            &p.danger_text,
            &p.success_background,
            &p.success_text,
            p.group_background.as_ref().unwrap(),
            p.group_border.as_ref().unwrap(),
        ] {
            assert!(!svg.contains(unused.as_str()), "treeView uses {unused}");
        }
        for unused in p.series.as_ref().unwrap() {
            assert!(!svg.contains(unused.as_str()), "treeView uses {unused}");
        }
    }
}

#[test]
fn treeview_box_drawing_input_takes_the_same_roles() {
    let p = sentinel();
    let r = render(BOXES, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    assert_eq!(attr(tags(&r.svg, "class=\"pele-tree-guides\"")[0], "stroke"), p.edge.as_deref());
    assert_eq!(attr(tags(&r.svg, "class=\"pele-tree-description\"")[0], "fill"), Some(p.secondary_text.as_str()));
}

#[test]
fn treeview_host_icons_are_written_as_given() {
    let p = sentinel();
    let mut table = IconTable::new();
    table.insert("mermaid-treeview:file", "<circle r=\"3\" fill=\"#123456\"/>");
    let options = RenderOptions { icons: Some(Arc::new(table)), ..resolved_options(Some(p.clone())) };
    let r = render(TREE, &options).unwrap();
    assert!(r.svg.contains("<circle r=\"3\" fill=\"#123456\"/>"));
    // The folder icon has no host markup and keeps the built-in outline.
    assert_eq!(tags(&r.svg, "stroke-width=\"1.5\"").len(), 1);
}

#[test]
fn treeview_defaults_without_palette() {
    for theme in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let r = render(TREE, &RenderOptions { color_mode: ColorMode::Resolved, theme, ..Default::default() }).unwrap();
        assert_no_css_runtime(&r.svg);
        let palette = MermaidPalette::default_for(theme);
        assert_eq!(attr(tags(&r.svg, "class=\"pele-tree-highlight\"")[0], "fill"), Some(palette.selection.as_str()));
    }
}

#[test]
fn treeview_transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    for source in [TREE, BOXES] {
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
fn treeview_palettes_change_only_colors() {
    let a = sentinel();
    let b = MermaidPalette::dark().swift_completed();
    for source in [TREE, BOXES] {
        let ra = render(source, &resolved_options(Some(a.clone()))).unwrap();
        let rb = render(source, &resolved_options(Some(b.clone()))).unwrap();
        assert_ne!(ra.svg, rb.svg);
        assert_eq!((ra.width, ra.height), (rb.width, rb.height));
        assert_eq!(without_colors(&ra.svg), without_colors(&rb.svg));
        let css = render(source, &RenderOptions { palette: Some(a.clone()), ..Default::default() }).unwrap();
        assert_eq!((css.width, css.height), (ra.width, ra.height));
    }
}
