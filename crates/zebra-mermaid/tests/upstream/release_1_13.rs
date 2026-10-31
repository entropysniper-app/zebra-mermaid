//! Independent complete ports of Pele 1.13.0 render callbacks (MIT, kepano 2026).
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeMap;
fn re(p: &str) -> Regex {
    Regex::new(p).unwrap()
}
fn svg(src: &str) -> String {
    zebra_mermaid::render(src, &Default::default()).unwrap().svg
}
fn with(src: &str, options: Value) -> String {
    zebra_mermaid::render(src, &crate::common::render_options(&options, &Value::Null)).unwrap().svg
}
fn themed(vars: Value, body: &str) -> String {
    format!("%%{{init: {}}}%%\n{body}", serde_json::to_string(&json!({"themeVariables":vars})).unwrap())
}
fn series(s: &str) -> BTreeMap<usize, String> {
    let root = re(r#"^<svg[^>]*\bstyle="([^"]*)""#).captures(s).unwrap()[1].to_owned();
    re(r"--pele-series-(\d):([^;]*)").captures_iter(&root).map(|c| (c[1].parse().unwrap(), c[2].to_owned())).collect()
}
fn expected(items: &[(usize, &str)]) -> BTreeMap<usize, String> {
    items.iter().map(|(k, v)| (*k, (*v).into())).collect()
}
const PIE: &str = "pie\n  \"A\": 1\n  \"B\": 2";
const XY: &str = "xychart\n  bar [1, 2]\n  line [2, 1]";

#[cfg(feature = "xychart")]
#[test]
fn xy_palette_repeats() {
    let src="---\nconfig:\n  themeVariables:\n    xyChart:\n      plotColorPalette: '#000000, #0000FF, #00FF00, #FF0000'\n---\nxychart\ntitle \"Different Colors in xyChart\"\nx-axis \"categoriesX\" [\"Category 1\", \"Category 2\", \"Category 3\", \"Category 4\"]\ny-axis \"valuesY\" 0 --> 50\nline [10,20,30,40]\nbar [20,30,25,35]\nbar [15,25,20,30]\nline [5,15,25,35]";
    let s = svg(src);
    assert_eq!(
        series(&s),
        expected(&[
            (1, "#000000"),
            (2, "#0000FF"),
            (3, "#00FF00"),
            (4, "#FF0000"),
            (5, "#000000"),
            (6, "#0000FF"),
            (7, "#00FF00"),
            (8, "#FF0000")
        ])
    );
    assert!(s.contains("stroke=\"var(--pele-series-1,"));
}
#[cfg(feature = "xychart")]
#[test]
fn function_commas() {
    let s = svg(&themed(json!({"xyChart":{"plotColorPalette":"rgb(0, 0, 0), hsl(0 50% 50% / 0.5)"}}), XY));
    assert_eq!(series(&s).get(&1).map(String::as_str), Some("rgb(0, 0, 0)"));
    assert_eq!(series(&s).get(&2).map(String::as_str), Some("hsl(0 50% 50% / 0.5)"));
}
#[cfg(all(feature = "pie", feature = "gitGraph", feature = "timeline", feature = "radar", feature = "treemap", feature = "mindmap"))]
#[test]
fn palettes_in_series_order() {
    for (body, vars) in [
        (PIE, json!({"pie1":"red","pie2":"blue"})),
        ("gitGraph\n  commit", json!({"git0":"red","git1":"blue"})),
        ("timeline\n  2001 : a\n  2002 : b", json!({"cScale0":"red","cScale1":"blue"})),
        ("radar-beta\n  axis a, b, c\n  curve x{1,2,3}", json!({"cScale0":"red","cScale1":"blue"})),
        ("treemap\n\"a\": 1\n\"b\": 2", json!({"cScale0":"red","cScale1":"blue"})),
        ("mindmap\n  root\n    a\n    b", json!({"cScale0":"green","cScale1":"red","cScale2":"blue"})),
    ] {
        assert_eq!(series(&svg(&themed(vars, body))), expected(&[(1, "red"), (2, "blue")]), "{body}");
    }
}
#[cfg(feature = "venn")]
#[test]
fn venn_palette_repeats() {
    let s = svg(&themed(json!({"venn2":"red","venn4":"blue"}), "venn-beta\n  set A\n  set B"));
    assert_eq!(series(&s), expected(&[(1, "red"), (2, "blue"), (3, "red"), (4, "blue"), (5, "red"), (6, "blue"), (7, "red"), (8, "blue")]));
}
#[cfg(all(feature = "pie", feature = "journey", feature = "flowchart"))]
#[test]
fn unrelated_variables_ignored() {
    for (vars, body) in [
        (json!({"cScale0":"red","primaryColor":"red"}), PIE),
        (json!({"fillType0":"red","cScale0":"red"}), "journey\n  section S\n    Task: 5: Me"),
        (json!({"pie1":"red"}), "flowchart LR\n  a --> b"),
    ] {
        assert_eq!(series(&svg(&themed(vars, body))), BTreeMap::new());
    }
}
#[cfg(feature = "pie")]
#[test]
fn palette_config_sources() {
    let front = format!("---\nconfig:\n  themeVariables:\n    pie1: '#123456'\n---\n{PIE}");
    assert_eq!(series(&svg(&front)), expected(&[(1, "#123456")]));
    let option = with(PIE, json!({"config":{"themeVariables":{"pie2":"oklch(70% 0.1 200)"}}}));
    assert_eq!(series(&option), expected(&[(2, "oklch(70% 0.1 200)")]));
}
#[cfg(feature = "pie")]
#[test]
fn theme_aware_colors() {
    assert_eq!(
        series(&svg(&themed(
            json!({"pie1":"var(--pele-accent)","pie2":"light-dark(#000, #fff)","pie3":"color-mix(in srgb, red 40%, blue)"}),
            PIE
        ))),
        expected(&[(1, "var(--pele-accent)"), (2, "light-dark(#000, #fff)"), (3, "color-mix(in srgb, red 40%, blue)")])
    );
}
#[cfg(all(feature = "pie", feature = "xychart"))]
#[test]
fn invalid_palette_colors() {
    for value in [
        "url(https://example.com/a.svg)",
        "url(//example.com/a.svg)",
        "image-set(a 1x)",
        "red;stroke:blue",
        "\"><script>alert(1)</script>",
        "expression(alert(1))",
        "rgb(0, 0",
        "red)",
        "red}",
        "javascript:alert(1)",
        "/*",
        "reddish",
        "#ff00000",
        "var(--page-color)",
        "color-mix(in srgb, var(--page-color) 50%, red)",
    ] {
        let s = svg(&themed(json!({"pie1":value,"pie2":"blue"}), PIE));
        assert_eq!(series(&s), expected(&[(2, "blue")]), "{value}");
        assert!(!s.contains("<script"));
        assert!(!s.contains("url("));
    }
    assert_eq!(
        series(&svg(&themed(json!({"xyChart":{"plotColorPalette":"url(a), blue"}}), XY))),
        expected(&[(2, "blue"), (4, "blue"), (6, "blue"), (8, "blue")])
    );
}
#[cfg(all(feature = "pie", feature = "gitGraph", feature = "timeline", feature = "treemap"))]
#[test]
fn palette_label_contrast() {
    let light = "fill=\"var(--pele-fg-on-light,#1f1f1f)\"";
    let dark = "fill=\"var(--pele-fg-on-dark,#fff)\"";
    let pie = svg(&themed(json!({"pie1":"#ffeb3b","pie2":"navy","pie3":"var(--pele-accent)"}), "pie\n  \"A\": 1\n  \"B\": 1\n  \"C\": 1"));
    let labels = re(r"<text [^>]*>\d+%<").find_iter(&pie).map(|m| m.as_str()).collect::<Vec<_>>();
    assert!(labels[0].contains(light));
    assert!(labels[1].contains(dark));
    assert!(!labels[2].contains("fill="));
    assert!(pie.contains("class=\"pele-slice-labels\""));
    let git = svg(&themed(json!({"git0":"#ffeb3b"}), "gitGraph\n  commit\n  branch dev\n  commit"));
    assert!(re(&format!("class=\"pele-label\" {}", regex::escape(light))).is_match(&git));
    assert!(git.contains("class=\"pele-label\" fill=\"var(--_bg)\""));
    let timeline = svg(&themed(json!({"cScale0":"#ffeb3b"}), "timeline\n  section S\n    2001 : a"));
    assert_eq!(timeline.matches(light).count(), 2);
    let treemap = svg(&themed(json!({"cScale0":"#ffeb3b"}), "treemap\n\"a\": 1\n\"b\": 1"));
    assert_eq!(treemap.matches(light).count(), 2);
}
#[cfg(feature = "pie")]
#[test]
fn no_palette_unchanged() {
    assert_eq!(svg(PIE), svg(&themed(json!({"primaryColor":"red"}), PIE)));
    assert!(!svg(PIE).contains("--pele-series-1:"));
}

#[cfg(feature = "block")]
fn node_label(s: &str, id: &str) -> Vec<String> {
    let at = s.find(&format!("data-id=\"{id}\"")).unwrap();
    let body = &s[s[..at].rfind("<g class=\"pele-node").unwrap()..s[at..].find("</g>").unwrap() + at];
    let inner = re(r#"<text class="pele-label"[^>]*>([\s\S]*?)</text>"#).captures(body).map_or(String::new(), |c| c[1].into());
    let lines = re(r"<tspan(?:\s[^>]*)?>([\s\S]*?)</tspan>").captures_iter(&inner).map(|c| c[1].into()).collect::<Vec<_>>();
    if !lines.is_empty() {
        lines
    } else if inner.is_empty() {
        vec![]
    } else {
        vec![inner]
    }
}
#[cfg(feature = "block")]
#[test]
fn spanning_block_wraps() {
    let text = "Something long long long long long long";
    let s = svg(&format!("block\n  1[\"Something\"]\n  2[\"{text}\"]:2\n  3[\"{text}\"]:5\n  4[\"{text}\"]:2\n  5[\"Something\"]"));
    assert_eq!(node_label(&s, "3"), [text]);
    assert_eq!(node_label(&s, "2").len(), 2);
}
#[cfg(feature = "block")]
#[test]
fn fixed_shapes_and_breaks() {
    let text = "Something long long long long long long";
    let fixed = svg(&format!("block\n  a((\"{text}\")) c{{\"{text}\"}} d[\"{text}\"]:5"));
    assert_eq!(node_label(&fixed, "a").len(), 2);
    assert_eq!(node_label(&fixed, "c").len(), 2);
    assert_eq!(node_label(&fixed, "d"), [text]);
    let broken = svg(&format!("block\n  a[\"{text}<br>tail\"]:5 b c d e"));
    assert_eq!(node_label(&broken, "a"), ["Something long long long long long", "long", "tail"]);
}
#[cfg(feature = "flowchart")]
#[test]
fn entering_edge_keeps_row() {
    let s = svg("flowchart TB\n  subgraph S\n    a\n    b\n    c\n  end\n  x --> a");
    let y = |id: &str| {
        re(&format!(r#"data-id="{id}" transform="translate\([-\d.]+,([-\d.]+)"#)).captures(&s).unwrap()[1].parse::<f64>().unwrap()
    };
    assert_eq!(y("b"), y("a"));
    assert_eq!(y("c"), y("a"));
}
#[cfg(feature = "quadrantChart")]
#[test]
fn quadrant_font_settings() {
    let src="quadrantChart\n  title Garden plan\n  x-axis Low effort --> High effort\n  y-axis Low value --> High value\n  quadrant-1 Big bets\n  Herb spiral: [0.3, 0.7]";
    let config = |body: &str| format!("---\nconfig:\n  quadrantChart:\n{body}\n---\n{src}");
    let plain = with(src, json!({"fontSize":16}));
    for needle in [
        "class=\"pele-title\" font-weight",
        "<g class=\"pele-axis-labels\" font-size=\"14\"",
        "<g class=\"pele-point-labels\" font-size=\"12\"",
    ] {
        assert!(plain.contains(needle));
    }
    let sized=with(&config("    titleFontSize: 30\n    quadrantLabelFontSize: 20\n    xAxisLabelFontSize: 11\n    yAxisLabelFontSize: 13\n    pointLabelFontSize: 9"),json!({"fontSize":16}));
    for needle in [
        "class=\"pele-title\" font-size=\"30\"",
        "class=\"pele-label\" font-size=\"20\" fill=\"var(--_fg)\"",
        "<g class=\"pele-axis-labels\" font-size=\"11\"",
        "<g class=\"pele-point-labels\" font-size=\"9\"",
    ] {
        assert!(sized.contains(needle), "{needle}");
    }
    assert_eq!(re(r#"rotate\(-90\)" font-size="13""#).find_iter(&sized).count(), 2);
    assert_eq!(with(&config("    titleFontSize: \"big\""), json!({"fontSize":16})), plain);
    assert!(with(&config("    pointLabelFontSize: 0"), json!({"fontSize":16})).contains("<g class=\"pele-point-labels\" font-size=\"4\""));
}
#[cfg(feature = "quadrantChart")]
#[test]
fn quadrant_point_line() {
    let name = "Drip irrigation for the whole orchard and the vegetable beds";
    let s = with(&format!("quadrantChart\n {name}: [0.5, 0.5]"), json!({"fontSize":22}));
    let texts = re(r"<text[^>]*>([^<]*)</text>|<tspan[^>]*>([^<]*)</tspan>")
        .captures_iter(&s)
        .map(|c| c.get(1).or_else(|| c.get(2)).unwrap().as_str())
        .collect::<Vec<_>>();
    assert!(texts.contains(&name));
}
