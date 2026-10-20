//! 1:1 ports of the first six cases in Pele tests/render/fonts.test.ts (MIT).
//! Byte recognizers below preserve the exact languages of the original SVG regexes.
use std::sync::Arc;
use zebra_mermaid::text::MetricsMeasurer;
use zebra_mermaid::{render, RenderOptions};

const FLOW: &str = "flowchart LR\n  A --> B";

fn drawn(source: &str, mut options: RenderOptions) -> String {
    options.measurer = Some(Arc::new(MetricsMeasurer));
    render(source, &options).unwrap().svg
}

// /^<svg[^>]*>/
fn root(options: RenderOptions) -> String {
    let svg = drawn(FLOW, options);
    assert!(svg.starts_with("<svg"));
    let end = svg.find('>').expect("root tag must close") + 1;
    svg[..end].to_owned()
}

// [\d.]+, where JavaScript \d is ASCII and a following quote ends the run.
fn digits_and_dots(value: &str) -> Option<&str> {
    let count = value.bytes().take_while(|b| b.is_ascii_digit() || *b == b'.').count();
    (count > 0).then(|| &value[count..])
}

// /<text class="pele-title" font-weight="var\(--_tw\)"[^>]*><tspan
// x="[\d.]+" y="[\d.]+">My title<\/tspan><\/text>/
fn has_title_markup(svg: &str) -> bool {
    let prefix = "<text class=\"pele-title\" font-weight=\"var(--_tw)\"";
    svg.match_indices(prefix).any(|(at, _)| {
        let tail = &svg[at + prefix.len()..];
        let Some(close) = tail.find('>') else { return false };
        let Some(tail) = tail[close + 1..].strip_prefix("<tspan x=\"") else { return false };
        let Some(tail) = digits_and_dots(tail) else { return false };
        let Some(tail) = tail.strip_prefix("\" y=\"") else { return false };
        let Some(tail) = digits_and_dots(tail) else { return false };
        tail.starts_with("\">My title</tspan></text>")
    })
}

// /<text[^>]* font-weight="var\(--_hw\)"/
fn has_heading_weight(svg: &str) -> bool {
    svg.match_indices("<text").any(|(at, _)| svg[at + "<text".len()..].split('>').next().unwrap().contains(" font-weight=\"var(--_hw)\""))
}

// /font-weight="(?!var\(--_hw\)")/
fn has_non_heading_weight(svg: &str) -> bool {
    let prefix = "font-weight=\"";
    svg.match_indices(prefix).any(|(at, _)| !svg[at + prefix.len()..].starts_with("var(--_hw)\""))
}

// /<rect[^>]* style="rx:var\(--_r\)"\/>/
fn has_radius_style_at_tag_end(svg: &str) -> bool {
    svg.match_indices("<rect").any(|(at, _)| {
        let tail = &svg[at..];
        tail.find('>').is_some_and(|end| tail[..=end].ends_with(" style=\"rx:var(--_r)\"/>"))
    })
}

#[test]
fn draws_in_generic_fonts_unless_told_what_was_measured() {
    assert!(root(RenderOptions::default()).contains(";--_fm:var(--pele-font-mono,monospace);font:16px sans-serif;"));
    // /;font-family:var\(--pele-font,sans-serif\)"/ is a literal substring.
    assert!(root(RenderOptions::default()).contains(";font-family:var(--pele-font,sans-serif)\""));
    assert!(!root(RenderOptions::default()).contains(" font-family="));
}

#[test]
fn draws_in_the_fonts_it_measured_with_where_the_page_names_no_others() {
    let options = RenderOptions {
        font_family: Some("\"Segoe UI\", system-ui, sans-serif".into()),
        font_family_mono: Some("'SF Mono', ui-monospace".into()),
        ..Default::default()
    };
    let svg = root(options);
    assert!(svg.contains("font-family:var(--pele-font,&quot;Segoe UI&quot;, system-ui, sans-serif)\""));
    assert!(svg.contains("--_fm:var(--pele-font-mono,'SF Mono', ui-monospace);"));
    assert!(root(RenderOptions { font_family: Some("思源黑体, sans-serif".into()), ..Default::default() })
        .contains("var(--pele-font,思源黑体, sans-serif)"));
    assert!(root(RenderOptions {
        font_family: Some("\"??\", \"??\", ui-sans-serif, -apple-system, system-ui".into()),
        ..Default::default()
    })
    .contains("var(--pele-font,&quot;??&quot;, &quot;??&quot;, ui-sans-serif, -apple-system, system-ui)"));
}

#[test]
fn pins_what_text_would_inherit_from_the_page_and_was_not_measured() {
    for pinned in ["letter-spacing:normal", "word-spacing:normal", "text-transform:none", "direction:ltr"] {
        assert!(root(RenderOptions::default()).contains(&format!(";{pinned};")));
    }
}

#[test]
fn takes_the_weight_of_bold_text_from_variables_each_defined_only_where_it_is_used() {
    let plain = drawn(FLOW, RenderOptions::default());
    assert!(!plain.contains("font-weight"));
    // /--_t?h?w:/ has exactly these four expansions.
    assert!(!["--_w:", "--_tw:", "--_hw:", "--_thw:"].iter().any(|p| plain.contains(p)));

    let titled = drawn(&format!("---\ntitle: My title\n---\n{FLOW}\n  B --> C[\"`**Bold** word`\"]"), RenderOptions::default());
    assert!(titled.contains(";--_tw:var(--pele-title-weight,bold);--_w:var(--pele-bold-weight,bold);"));
    assert!(!titled.contains("--_hw:"));
    assert!(has_title_markup(&titled));
    assert!(titled.contains("<tspan x=\"0\" y=\"5.6\" font-weight=\"var(--_w)\">Bold</tspan><tspan> word</tspan>"));
}

#[test]
fn draws_the_names_of_classes_columns_and_other_headings_in_the_heading_weight() {
    let sources = [
        "classDiagram\n  class Duck {\n    +swim()\n  }",
        "kanban\n  Todo\n    [Write docs]",
        "treeView-beta\n    src/\n        index.js\n",
        "sequenceDiagram\n  loop Daily\n    A->>B: x\n  end",
        "timeline\n  section Early\n    2001 : First",
    ];
    for source in sources {
        let svg = drawn(source, RenderOptions::default());
        assert!(svg.contains(";--_hw:var(--pele-heading-weight,bold);"), "{source}");
        assert!(has_heading_weight(&svg), "{source}");
        assert!(!has_non_heading_weight(&svg), "{source}");
    }
}

#[test]
fn writes_the_corner_radius_in_a_style_which_every_browser_reads_a_variable_from() {
    let svg = drawn("flowchart LR\n  A[One] --> B[Two]\n  style B rx:10,fill:#eee", RenderOptions::default());
    assert!(svg.contains(";--_r:var(--pele-radius,4px)"));
    assert!(!svg.contains("rx=\"var("));
    assert!(has_radius_style_at_tag_end(&svg));
    assert!(svg.contains("style=\"rx:var(--_r);rx:10;fill:#eee;\""));
    assert!(!drawn("pie\n  \"a\": 1", RenderOptions::default()).contains("--_r"));
}
