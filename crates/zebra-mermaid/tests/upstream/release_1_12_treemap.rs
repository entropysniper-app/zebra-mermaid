use super::*;
const BASIC: &str =
    "treemap-beta\n\"Category A\"\n    \"Item A1\": 10\n    \"Item A2\": 20\n\"Category B\"\n    \"Item B1\": 15\n    \"Item B2\": 25\n";
const BUDGET:&str="treemap-beta\n\"Budget\"\n    \"Operations\"\n        \"Salaries\": 700000\n        \"Equipment\": 200000\n        \"Supplies\": 100000\n    \"Marketing\"\n        \"Advertising\": 400000\n        \"Events\": 100000";
fn config(config: &str, body: &str) -> String {
    format!("---\nconfig:\n  treemap:\n{config}\n---\n{body}")
}
#[derive(Clone)]
struct Rect {
    id: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    attrs: BTreeMap<String, String>,
}
fn rects(s: &str) -> Vec<Rect> {
    let mut id = String::new();
    let mut out = Vec::new();
    for e in elements(s) {
        if e.name == "g" && e.attrs.contains_key("data-id") {
            id = e.attr("data-id").into();
        }
        if e.name == "rect" {
            out.push(Rect { id: id.clone(), x: e.n("x"), y: e.n("y"), w: e.n("width"), h: e.n("height"), attrs: e.attrs });
        }
    }
    out
}
fn smallest_gap(boxes: &[Rect]) -> f64 {
    let mut least = f64::INFINITY;
    for a in boxes {
        for b in boxes {
            if a.w.min(a.h).min(b.w).min(b.h) < 8. {
                continue;
            }
            if a.y < b.y + b.h && b.y < a.y + a.h && b.x >= a.x + a.w {
                least = least.min(b.x - a.x - a.w);
            }
            if a.x < b.x + b.w && b.x < a.x + a.w && b.y >= a.y + a.h {
                least = least.min(b.y - a.y - a.h);
            }
        }
    }
    least
}
fn deep_tree() -> String {
    let mut s = "treemap\n".to_owned();
    for i in 0..6 {
        s += &format!("\"top {i}\"\n");
        for j in 0..8 {
            s += &format!("  \"mid {i}.{j}\"\n    \"inner {i}.{j}\"\n");
            for k in 0..8 {
                s += &format!("      \"leaf {i}.{j}.{k}\": {}\n", 1 + ((i * 64 + j * 8 + k) % 37_usize).pow(3));
            }
        }
    }
    s
}
fn select(boxes: &[Rect], prefix: &str) -> Vec<Rect> {
    boxes.iter().filter(|r| r.id.starts_with(prefix)).cloned().collect()
}
#[test]
fn contrasting_text() {
    let s=svg_with("treemap\n\"Section\":::frame\n  \"a\": 10:::dark\n  \"b\": 20:::own\n  \"c\": 30\nclassDef frame fill:#FFF9C4\nclassDef dark fill:navy\nclassDef own fill:navy,color:#ddd",json!({"config":{"treemap":{"showValues":true}}}));
    let group = |id: &str| captures(&s, &format!(r#"data-id="{id}">(.*?)</g>"#))[1].clone();
    assert!(has(&group("Section"), r#"<text[^>]* font-weight="var\(--_hw\)" fill="var\(--pele-fg-on-light,#1f1f1f\)">Section<"#));
    assert!(has(&group("Section"), r#"<text class="pele-value"[^>]* fill="var\(--pele-fg-on-light,#1f1f1f\)">"#));
    assert!(has(&group("a"), r#"<text[^>]* fill="var\(--pele-fg-on-dark,#fff\)">a<"#));
    for id in ["b", "c"] {
        assert!(!group(id).contains("--pele-fg-on"));
    }
}
#[test]
fn title_classes() {
    let s = svg("treemap\n\"S\":::titles\n  \"a\": 1:::title_2\nclassDef title_2 fill:red");
    assert!(s.contains("class=\"pele-node pele-section titles\""));
    assert!(s.contains("class=\"pele-node pele-leaf title_2\" data-id=\"a\""));
}
#[test]
fn headerless_tooltips() {
    let s = svg(&config("    nodeHeight: 3", "treemap\n\"A & B\"\n  \"Inner\"\n    \"Leaf\": 1"));
    assert!(s.contains("<title>A &amp; B › Inner</title>"));
    assert!(s.contains("<title>A &amp; B › Inner › Leaf</title>"));
    assert!(!svg(BASIC).contains("<title>"));
    assert!(!svg(&config("    nodeHeight: 3", "treemap\n\"#1;\"\n  \"Leaf\": 1")).contains("<title>"));
}
#[test]
fn large_deep_tree() {
    let s = svg(&deep_tree());
    let leaves = 6 * 8 * 8;
    let sections = 6 + 6 * 8 * 2;
    assert_eq!(matches(&s, r#"class="pele-node pele-leaf""#).len(), leaves);
    assert_eq!(matches(&s, r#"class="pele-node pele-section""#).len(), sections);
}
#[test]
fn crowded_gaps() {
    let bs = rects(&svg(&deep_tree()));
    let top = bs.iter().find(|r| r.id == "top 0").unwrap();
    let min = select(&bs, "mid 0.").iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
    assert_eq!(min - top.x, 3.);
    let mut gaps = vec![smallest_gap(&select(&bs, "top "))];
    for i in 0..6 {
        gaps.push(smallest_gap(&select(&bs, &format!("mid {i}."))));
        for j in 0..8 {
            gaps.push(smallest_gap(&select(&bs, &format!("leaf {i}.{j}."))));
        }
    }
    gaps.retain(|g| *g != f64::INFINITY);
    let mut unique = Vec::new();
    for g in gaps {
        if !unique.contains(&g) {
            unique.push(g);
        }
    }
    assert_eq!(unique, [2.]);
}
#[test]
fn roomy_gaps() {
    let bs = rects(&svg(BUDGET));
    let id = |name: &str| bs.iter().find(|r| r.id == name).unwrap();
    assert_eq!(id("Operations").x - id("Budget").x, 6.);
    assert_eq!(id("Operations").y - id("Budget").y, 28.);
    assert_eq!(smallest_gap(&[id("Salaries").clone(), id("Equipment").clone(), id("Supplies").clone()]), 4.);
    assert_eq!(smallest_gap(&[id("Operations").clone(), id("Marketing").clone()]), 4.);
}
#[test]
fn author_padding() {
    let crowded = rects(&svg(&config("    padding: 6", &deep_tree())));
    assert_eq!(smallest_gap(&select(&crowded, "top ")), 6.);
    let roomy = rects(&svg(&config("    padding: 1", BUDGET)));
    assert_eq!(
        smallest_gap(&roomy.into_iter().filter(|r| ["Salaries", "Equipment", "Supplies"].contains(&r.id.as_str())).collect::<Vec<_>>()),
        1.
    );
}
#[test]
fn no_header_box() {
    assert!(has(
        &svg(&config("    nodeHeight: 3", "treemap\n\"Section\"\n  \"Leaf\": 1")),
        r#"data-id="Section"><title>Section</title><rect [^>]*fill="none"/></g>"#
    ));
}
#[test]
fn short_header() {
    let s = svg(&config("    nodeHeight: 4", "treemap\n\"Section\"\n  \"Leaf\": 1"));
    let bs = rects(&s);
    assert!(has(&s, r#"font-size="12">Section<"#));
    assert_eq!(bs.iter().find(|r| r.id == "Leaf").unwrap().y - bs.iter().find(|r| r.id == "Section").unwrap().y, 18.);
    assert!(!has(
        &svg(&config("    nodeHeight: 4", "treemap\n\"Section\":::big\n  \"Leaf\": 1\nclassDef big font-size:14.4px")),
        r"<text[^>]*>Section<"
    ));
}
#[test]
fn small_corners() {
    let bs = rects(&svg("treemap\n\"big\": 1000\n\"small\": 4"));
    let small = bs.iter().find(|r| r.id == "small").unwrap();
    assert_eq!(small.attrs.get("style"), Some(&format!("rx:min(var(--_r),{}px)", small.w.min(small.h) / 4.)));
    assert_eq!(bs.iter().find(|r| r.id == "big").unwrap().attrs.get("style").map(String::as_str), Some("rx:var(--_r)"));
}
#[test]
fn class_font_fits() {
    let s=svg("treemap\n\"A section with a long name\":::head\n  \"A leaf with a fairly long name\": 30:::big\n  \"Another leaf with a long name\": 20:::big\n  \"rest\": 400\nclassDef big font-size:20px\nclassDef head font-size:18px");
    let groups = matches(&s, r#"<g class="pele-node pele-(leaf|section) (big|head)"[^>]*>(.*?)</g>"#);
    assert_eq!(groups.len(), 3);
    for g in groups {
        let w = number(&captures(&g[3], r#"<rect[^>]* width="([\d.]+)""#)[1]);
        let texts = matches(&g[3], r"<text[^>]*>([^<]*)</text>");
        assert!(!texts.is_empty());
        for t in texts {
            let width = if g[1] == "leaf" {
                MetricsMeasurer.width(&t[1], 20., 0)
            } else {
                MetricsMeasurer.width(&t[1], 18., zebra_mermaid::text::style::BOLD)
            };
            assert!(width <= w - 8., "{}", t[1]);
        }
    }
}
#[test]
fn large_section_names() {
    let tree = "treemap\n\"Section\"\n  \"Leaf\": 1";
    assert!(has(&svg_with(tree, json!({"fontSize":24})), r"<text[^>]*>Section<"));
    assert!(has(&svg(&format!("{tree}\nclassDef big font-size:20px").replace("\"Section\"", "\"Section\":::big")), r"<text[^>]*>Section<"));
    let tall = svg(&format!("{tree}\nclassDef big font-size:32px").replace("\"Section\"", "\"Section\":::big"));
    assert!(!has(&tall, r"<text[^>]*>Section<"));
    assert!(tall.contains("<title>Section</title>"));
}
#[test]
fn fractional_class_font() {
    assert!(has(&svg(&config("    nodeHeight: 2", "treemap\n\"Label\": 1:::frac\nclassDef frac font-size:12.5px")), r"<text[^>]*>Label<"));
}
