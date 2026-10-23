//! 1:1 port: Pele 1.9.0 tests/render/break-hints.test.ts (MIT, kepano 2026).
use zebra_mermaid::text::{layout_label, Icons, Measure, MetricsMeasurer, TextMeasurer};
const ZWSP: char = '\u{200b}';
const SHY: char = '\u{ad}';
fn label(raw: &str, max: f64) -> zebra_mermaid::text::Label {
    let icons = Icons::none();
    layout_label(Some(raw), false, &Measure::new(&MetricsMeasurer, &icons), 16.0, max, 0)
}
fn lines(raw: &str, max: f64) -> Vec<String> {
    label(raw, max).lines.iter().map(|line| line.iter().map(|s| s.text.as_str()).collect()).collect()
}
#[test]
fn breaks_at_zero_width_spaces() {
    let out = lines(&format!("RRCConnection{ZWSP}Reconfiguration{ZWSP}Complete"), 140.0);
    assert_eq!(out.iter().map(|s| s.replace(ZWSP, "")).collect::<Vec<_>>(), ["RRCConnection", "Reconfiguration", "Complete"]);
}
#[test]
fn soft_hyphen_is_visible_only_at_breaks() {
    let source = format!("RRCConnection{SHY}Reconfiguration{SHY}Complete");
    assert_eq!(lines(&source, 140.0), ["RRCConnection-", "Reconfiguration-", "Complete"]);
    for width in label(&source, 140.0).widths {
        assert!(width <= 140.0);
    }
    assert_eq!(lines(&format!("Re{SHY}con{SHY}fig"), 200.0), [format!("Re{SHY}con{SHY}fig")]);
}
#[test]
fn keeps_the_rest_of_a_word_when_it_fits() {
    assert_eq!(lines(&format!("a uni{SHY}t test"), 39.2), [format!("a uni{SHY}t"), "test".to_owned()]);
}
#[test]
fn several_soft_hyphens_stay_within_width() {
    for width in label(&format!("a uni{SHY}t{SHY}ed"), 39.2).widths {
        assert!(width <= 39.2);
    }
}
#[test]
fn word_continuing_in_bold_stays_within_width() {
    for width in label(&format!("a uni{SHY}<b>t</b> test"), 39.2).widths {
        assert!(width <= 39.2);
    }
}
#[test]
fn zero_width_pieces_wrap_in_linear_time() {
    for hint in [SHY, ZWSP] {
        let start = std::time::Instant::now();
        label(&("a".repeat(22) + &format!("\u{300}{hint}").repeat(24000) + "i xx"), 200.0);
        assert!(start.elapsed().as_secs_f64() * 1000.0 < 1000.0);
    }
}
#[test]
fn breaks_at_space_after_soft_hyphen_without_showing_it() {
    let source = format!("a uni{SHY} t test");
    for width in label(&source, 39.2).widths {
        assert!(width <= 39.2);
    }
    assert!(!lines(&source, 39.2).join(" ").contains('-'));
}
#[test]
fn space_in_the_next_styled_span_leaves_hyphen_hidden() {
    assert_eq!(lines(&format!("ab{SHY}<i> cd</i>"), 25.0), [format!("ab{SHY}"), "cd".to_owned()]);
}
#[test]
fn decodes_shy_entity() {
    assert_eq!(lines("RRCConnection&shy;Reconfiguration&shy;Complete", 140.0), ["RRCConnection-", "Reconfiguration-", "Complete"]);
}
#[test]
fn hidden_soft_hyphen_has_no_width() {
    assert_eq!(MetricsMeasurer.width(&format!("ab{SHY}c"), 16.0, 0), MetricsMeasurer.width("abc", 16.0, 0));
}
#[cfg(feature = "sequence")]
#[test]
fn narrow_sequence_message_splits_at_hints() {
    let source=format!("sequenceDiagram\n  participant UE\n  participant gNB\n  participant AMF\n  gNB->>UE: RRCConnection{ZWSP}Reconfiguration{ZWSP}Complete\n  UE->>gNB: ok");
    let svg = zebra_mermaid::render(&source, &zebra_mermaid::RenderOptions { max_width: Some(360.0), ..Default::default() }).unwrap().svg;
    // The original <tspan\b[^>]*>([^<]*)</tspan> projection, without altering text.
    let mut labels = Vec::new();
    let mut from = 0;
    while let Some(at) = svg[from..].find("<tspan") {
        let at = from + at;
        let Some(end) = svg[at..].find('>') else {
            break;
        };
        let start = at + end + 1;
        let next = svg[start..].find('<').unwrap() + start;
        if svg[next..].starts_with("</tspan>") {
            labels.push(svg[start..next].replace(ZWSP, ""));
        }
        from = next + 1;
    }
    assert_eq!(labels, ["RRCConnection", "Reconfiguration", "Complete"]);
}
