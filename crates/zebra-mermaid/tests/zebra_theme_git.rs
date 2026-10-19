#![cfg(feature = "gitGraph")]

//! Zebra theme roles of the git graph in resolved mode (docs/ZEBRA-THEME-COMPATIBILITY.md).
//!
//! Zebra's `apply_theme` gives each branch one color of its `chart_series` (lane, connectors,
//! commit symbols, name tag), the name in a color that reads on it, commit ids in the text
//! color on the node fill, and tags in the text color on the secondary surface with a faint
//! outline.

mod common;

use common::theme::{assert_no_css_runtime, resolved_options, sentinel, without_colors};
use zebra_mermaid::theme::color::contrasting_text_on_fill;
use zebra_mermaid::{render, CanvasBackground, ConfigMap, ConfigValue, MermaidPalette, MermaidThemeMode, RenderOptions, ResolvedTheme};

const GIT: &str = "gitGraph
  commit id: \"a\" tag: \"v1\"
  branch dev
  commit id: \"b\" type: REVERSE
  branch feat
  commit id: \"c\" type: HIGHLIGHT
  branch fix
  commit id: \"d\"
  checkout main
  merge dev id: \"m\"
  cherry-pick id: \"c\"
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

fn theme(p: &MermaidPalette) -> ResolvedTheme {
    ResolvedTheme::from_palette(p)
}

#[test]
fn git_consumes_its_roles() {
    let p = sentinel();
    let t = theme(&p);
    let r = render(GIT, &resolved_options(Some(p.clone()))).unwrap();
    let svg = &r.svg;
    assert_no_css_runtime(svg);
    assert!(svg.contains(&format!("fill=\"{}\" role=", p.text)));
    assert!(svg.contains(&format!("fill=\"{}\"/>", p.background)), "opaque canvas");

    let series = p.series.clone().unwrap();
    // Each branch: lane, name tag and name in the branch color, the name readable on it.
    for (i, name) in ["main", "dev", "feat", "fix"].iter().enumerate() {
        let color = &series[i];
        assert_eq!(&t.lines[i], color, "branch colors are the caller's series");
        let branch = element(svg, &format!("<g class=\"pele-branch\" data-id=\"{name}\">"));
        assert!(branch.contains(&format!("stroke=\"{color}\" stroke-dasharray=\"2 4\"")), "lane {name}: {branch}");
        assert!(branch.contains(&format!("rx=\"4\" fill=\"{color}\"/>")), "name tag {name}: {branch}");
        let on = contrasting_text_on_fill(color, &p.text, &p.background, 1.0);
        assert!(branch.contains(&format!("class=\"pele-label\" fill=\"{on}\"")), "name {name}: {branch}");
    }
    // A dark series takes the canvas color for its name, a light one the text color.
    assert_eq!(contrasting_text_on_fill(&series[1], &p.text, &p.background, 1.0), p.background);
    assert_eq!(contrasting_text_on_fill(&series[3], &p.text, &p.background, 1.0), p.text);

    // Commit symbols in their branch color; marks on them read on that color.
    let a = element(svg, "<g class=\"pele-commit pele-commit-normal\" data-id=\"a\">");
    assert!(a.contains(&format!("r=\"7\" fill=\"{}\"/>", series[0])), "{a}");
    let b = element(svg, "<g class=\"pele-commit pele-commit-reverse\" data-id=\"b\">");
    assert!(b.contains(&format!("r=\"7\" fill=\"{}\"/>", series[1])), "{b}");
    let mark = contrasting_text_on_fill(&series[1], &p.text, &p.background, 1.0);
    assert!(b.contains(&format!("stroke=\"{mark}\" stroke-width=\"1.5\"")), "{b}");
    let c = element(svg, "<g class=\"pele-commit pele-commit-highlight\" data-id=\"c\">");
    assert!(c.contains(&format!("rx=\"2\" fill=\"{}\"/>", series[2])), "{c}");
    let d = element(svg, "<g class=\"pele-commit pele-commit-normal\" data-id=\"d\">");
    assert!(d.contains(&format!("r=\"7\" fill=\"{}\"/>", series[3])), "{d}");
    let m = element(svg, "<g class=\"pele-commit pele-commit-merge\" data-id=\"m\">");
    assert!(m.contains(&format!("r=\"6\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"", p.background, series[0])), "{m}");
    assert!(m.contains(&format!("r=\"2.5\" fill=\"{}\"/>", series[0])), "{m}");
    let picked = element(svg, "<g class=\"pele-commit pele-commit-cherry-pick\"");
    let mark = contrasting_text_on_fill(&series[0], &p.text, &p.background, 1.0);
    assert!(picked.contains(&format!("r=\"7\" fill=\"{}\"/>", series[0])), "{picked}");
    assert!(picked.contains(&format!("fill=\"none\" stroke=\"{mark}\" stroke-linejoin=\"round\"")), "{picked}");
    assert!(picked.contains(&format!("r=\"1.8\" fill=\"{mark}\"/>")), "{picked}");

    // Commit ids: text color, on a patch of the node fill.
    assert!(svg.contains(&format!("<g class=\"pele-commits\" font-size=\"12\" fill=\"{}\">", p.text)));
    let label = element(a, "<g class=\"pele-commit-label\"");
    assert!(label.contains(&format!("rx=\"3\" fill=\"{}\"/>", p.surface)), "{label}");
    assert!(!label.contains("<text fill="), "the id inherits the group's text color: {label}");

    // Tags: text on the secondary surface, outlined (and pierced) in the faint text color.
    for tag in [element(a, "<g class=\"pele-tag\">"), element(picked, "<g class=\"pele-tag\">")] {
        assert!(tag.contains(&format!("fill=\"{}\" stroke=\"{}\" stroke-linejoin=\"round\"", p.surface_secondary, t.faint_text)), "{tag}");
        assert!(tag.contains(&format!("r=\"1.5\" fill=\"{}\"/>", t.faint_text)), "{tag}");
        assert!(tag.contains(&format!("<text fill=\"{}\"", p.text)), "{tag}");
    }

    // Connectors in the color of the branch they run on; a merge in the merged branch's color,
    // a cherry-pick dashed in the color of the branch it lands on.
    let to_dev = element(svg, "<path class=\"pele-edge\" d=\"M0,");
    assert!(to_dev.contains(&format!("stroke=\"{}\"", series[1])), "{to_dev}");
    let merge = element(svg, "<path class=\"pele-edge pele-edge-merge\"");
    assert!(merge.contains(&format!("stroke=\"{}\"", series[1])), "{merge}");
    let pick = element(svg, "<path class=\"pele-edge pele-edge-cherry-pick\"");
    assert!(pick.contains(&format!("stroke=\"{}\" stroke-dasharray=\"4 3\"", series[0])), "{pick}");

    // Roles a git graph does not use.
    let unused = [
        ("secondaryText", p.secondary_text.clone()),
        ("accent", p.accent.clone()),
        ("border", p.border.clone()),
        ("selection", p.selection.clone()),
        ("warningBackground", p.warning_background.clone()),
        ("warningText", p.warning_text.clone()),
        ("dangerBackground", p.danger_background.clone()),
        ("dangerText", p.danger_text.clone()),
        ("successBackground", p.success_background.clone()),
        ("successText", p.success_text.clone()),
        ("edge", p.edge.clone().unwrap()),
        ("groupBackground", p.group_background.clone().unwrap()),
        ("groupBorder", p.group_border.clone().unwrap()),
        ("series[4]", series[4].clone()),
        ("series[5]", series[5].clone()),
        ("mutedText", t.muted_text.clone()),
    ];
    for (role, color) in unused {
        assert!(!svg.contains(color.as_str()), "git graph uses {role} {color}");
    }
}

#[test]
fn branches_past_six_take_zebra_series_lines() {
    let p = sentinel();
    let t = theme(&p);
    let mut source = String::from("gitGraph\n  commit\n");
    for i in 1..8 {
        source.push_str(&format!("  branch b{i}\n  commit\n"));
    }
    let r = render(&source, &resolved_options(Some(p.clone()))).unwrap();
    assert_no_css_runtime(&r.svg);
    // Lanes 6 and 7 mix the first series toward the text color (`chart_series`), not toward
    // the canvas as chart fills do.
    for (i, name) in [(6, "b6"), (7, "b7")] {
        assert_ne!(t.lines[i], t.chart[i]);
        let branch = element(&r.svg, &format!("<g class=\"pele-branch\" data-id=\"{name}\">"));
        assert!(branch.contains(&format!("stroke=\"{}\" stroke-dasharray", t.lines[i])), "{branch}");
        assert!(!r.svg.contains(&t.chart[i]), "lane {i} must not use the chart fill");
    }
}

#[test]
fn default_palettes_without_series() {
    for mode in [MermaidThemeMode::Light, MermaidThemeMode::Dark] {
        let options = RenderOptions { theme: mode, ..resolved_options(None) };
        let r = render(GIT, &options).unwrap();
        assert_no_css_runtime(&r.svg);
        let t = theme(&MermaidPalette::default_for(mode));
        let main = element(&r.svg, "<g class=\"pele-branch\" data-id=\"main\">");
        assert!(main.contains(&format!("stroke=\"{}\"", t.lines[0])), "{main}");
        let fix = element(&r.svg, "<g class=\"pele-branch\" data-id=\"fix\">");
        assert!(fix.contains(&format!("stroke=\"{}\"", t.lines[3])), "{fix}");
        assert!(r.svg.contains(&format!("fill=\"{}\"/>", t.background)));
    }
}

#[test]
fn every_direction_and_option_resolves() {
    let p = sentinel();
    let mut flags = ConfigMap::new();
    flags.insert("rotateCommitLabel", ConfigValue::Bool(false));
    flags.insert("parallelCommits", ConfigValue::Bool(true));
    let mut config = ConfigMap::new();
    config.insert("gitGraph", ConfigValue::Object(flags));
    for source in [GIT.to_string(), GIT.replace("gitGraph\n", "gitGraph TB:\n"), GIT.replace("gitGraph\n", "gitGraph BT:\n")] {
        for options in [
            resolved_options(Some(p.clone())),
            RenderOptions { max_width: Some(200.0), ..resolved_options(Some(p.clone())) },
            RenderOptions { config: Some(config.clone()), ..resolved_options(Some(p.clone())) },
        ] {
            let r = render(&source, &options).unwrap();
            assert_no_css_runtime(&r.svg);
            assert!(r.svg.contains(&format!("<g class=\"pele-commits\" font-size=\"12\" fill=\"{}\">", p.text)));
        }
    }
}

#[test]
fn transparent_canvas_drops_only_the_root_rectangle() {
    let p = sentinel();
    let opaque =
        render(GIT, &RenderOptions { canvas_background: Some(CanvasBackground::Opaque), ..resolved_options(Some(p.clone())) }).unwrap();
    let clear = render(GIT, &RenderOptions { canvas_background: Some(CanvasBackground::Transparent), ..resolved_options(Some(p.clone())) })
        .unwrap();
    assert_eq!((opaque.width, opaque.height), (clear.width, clear.height));
    let rect = format!(
        "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
        opaque.width, opaque.height, p.background
    );
    assert!(opaque.svg.contains(&rect));
    assert_eq!(opaque.svg.replace(&rect, ""), clear.svg);
    // The canvas color still fills the middle of merge rings.
    assert!(clear.svg.contains(&format!("r=\"6\" fill=\"{}\"", p.background)));
}

#[test]
fn palettes_change_colors_only() {
    let a = sentinel();
    let b = MermaidPalette::from_json(include_str!("../../../docs/research/zebra-theme-palette.example.json")).unwrap();
    let c = MermaidPalette::dark().swift_completed();
    let ra = render(GIT, &resolved_options(Some(a))).unwrap();
    for p in [b, c] {
        let r = render(GIT, &resolved_options(Some(p))).unwrap();
        assert_ne!(r.svg, ra.svg);
        assert_eq!((r.width, r.height), (ra.width, ra.height));
        assert_eq!(without_colors(&r.svg), without_colors(&ra.svg));
    }
}

#[test]
fn css_mode_is_unchanged() {
    let r = render(GIT, &RenderOptions::default()).unwrap();
    assert!(r.svg.contains("stroke=\"var(--pele-series-2,#f58518)\" stroke-dasharray=\"2 4\""));
    assert!(r.svg.contains("class=\"pele-label\" fill=\"var(--_bg)\""));
    assert!(r.svg.contains("<g class=\"pele-commits\" font-size=\"12\" fill=\"var(--_m)\">"));
    assert!(r.svg.contains("fill=\"var(--_s)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\""));
}
