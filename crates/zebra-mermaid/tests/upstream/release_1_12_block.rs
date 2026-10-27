use super::*;
#[derive(Clone, Debug)]
struct Box {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
fn boxes(s: &str) -> BTreeMap<String, Box> {
    let mut out = BTreeMap::new();
    for m in matches(s, r#"class="pele-node[^"]*" data-id="([^"]*)" transform="translate\(([-\d.]+),([-\d.]+)\)"><(\w+)([^>]*)>"#) {
        let attr = |n: &str| re(&format!(r#"{n}="([\d.]+)""#)).captures(&m[5]).map_or(0., |c| number(&c[1]));
        out.insert(m[1].clone(), Box { x: number(&m[2]), y: number(&m[3]), w: attr("width"), h: attr("height") });
    }
    for m in
        matches(s, r#"class="pele-cluster[^"]*" data-id="([^"]*)"><rect x="([-\d.]+)" y="([-\d.]+)" width="([\d.]+)" height="([\d.]+)""#)
    {
        let w = number(&m[4]);
        let h = number(&m[5]);
        out.insert(m[1].clone(), Box { x: number(&m[2]) + w / 2., y: number(&m[3]) + h / 2., w, h });
    }
    out
}
fn layout(s: &str) -> BTreeMap<String, Box> {
    boxes(&svg(s))
}
fn paths(s: &str) -> Vec<String> {
    matches(s, r#"class="pele-edge" data-id="[^"]*"><path d="([^"]*)""#).into_iter().map(|m| m[1].clone()).collect()
}
fn label_boxes(s: &str) -> Vec<Box> {
    matches(s, r#"class="pele-edge-label" data-id="[^"]*"><rect x="([-\d.]+)" y="([-\d.]+)" width="([\d.]+)" height="([\d.]+)""#)
        .into_iter()
        .map(|m| {
            let w = number(&m[3]);
            let h = number(&m[4]);
            Box { x: number(&m[1]) + w / 2., y: number(&m[2]) + h / 2., w, h }
        })
        .collect()
}
fn separate(src: &str, count: usize) {
    let r = draw(src);
    let p = model(src);
    let DiagramModel::Block(m) = p.model() else { panic!("not a block model") };
    let el = elements(&r.svg);
    let mut labels = Vec::new();
    for (i, e) in el.iter().enumerate() {
        if e.attr("class") != "pele-edge-label" {
            continue;
        }
        let rect = &el[i + 1];
        assert_eq!(rect.name, "rect");
        labels.push((
            e.attr("data-id"),
            Box { x: rect.n("x") + rect.n("width") / 2., y: rect.n("y") + rect.n("height") / 2., w: rect.n("width"), h: rect.n("height") },
        ));
    }
    assert_eq!(labels.len(), count);
    assert_eq!(
        labels.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        m.edges
            .iter()
            .map(|&i| &m.blocks[i])
            .filter(|e| e.label.as_ref().is_some_and(|l| !l.is_empty()))
            .map(|e| e.id.as_deref().unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(paths(&r.svg).len(), m.edges.len());
    for (i, (id, a)) in labels.iter().enumerate() {
        assert!(a.w > 0.);
        assert!(a.h > 0.);
        assert!(a.x - a.w / 2. >= 0.);
        assert!(a.y - a.h / 2. >= 0.);
        assert!(a.x + a.w / 2. <= r.width);
        assert!(a.y + a.h / 2. <= r.height);
        for (bid, b) in labels.iter().skip(i + 1) {
            let ox = (a.w + b.w) / 2. - (a.x - b.x).abs();
            let oy = (a.h + b.h) / 2. - (a.y - b.y).abs();
            assert!(ox.min(oy) <= 0.01, "{id} overlaps {bid}");
        }
        let hits = labels
            .iter()
            .filter(|(_, b)| (a.x - b.x).abs() < b.w / 2. && (a.y - b.y).abs() < b.h / 2.)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(hits, [*id]);
    }
    assert_eq!(svg(src), r.svg);
}
fn straight(src: &str) {
    for path in paths(&svg(src)) {
        assert!(has(&path, r"^M[\d.,]+L[\d.,]+$"));
    }
}
#[test]
fn vertical_opposite_labels() {
    separate("block\n  columns 1\n  a[\"Upper layer\"]\n  space:3\n  b[\"Lower layer\"]\n  a -- \"12 production conforming\" --> b\n  b -- \"12 production deviation\" --> a",2);
}
#[test]
fn horizontal_opposite_labels() {
    separate("block\n  a[\"Left layer\"] space:3 b[\"Right layer\"]\n  a -- \"12 production<br>conforming\" --> b\n  b -- \"12 production<br>deviation\" --> a",2);
}
#[test]
fn disjoint_straight_routes() {
    let src = "block\n  a space:3 b\n  a -- \"forward\" --> b\n  b -- \"reverse\" --> a";
    separate(src, 2);
    straight(src);
}
#[test]
fn narrow_endpoints() {
    separate("block\n  columns 1\n  a[\"A\"]\n  space\n  b[\"B\"]\n  a -- \"144 production conforming\" --> b\n  b -- \"144 production deviation\" --> a",2);
}
#[test]
fn all_parallel_labels() {
    separate("block\n  columns 1\n  a[\"A\"]\n  space:2\n  b[\"B\"]\n  a -- \"7 production policy\" --> b\n  a -- \"7 test policy\" --> b\n  a -- \"7 production exception\" --> b\n  b -- \"7 production deviation\" --> a",4);
}
#[test]
fn stacked_labels_straight() {
    let src =
        "block\n  columns 3\n  l a r\n  space:3\n  m b n\n  a -- \"production conforming\" --> b\n  b -- \"production deviation\" --> a";
    separate(src, 2);
    straight(src);
}
#[test]
fn avoid_other_blocks() {
    let clear = |src: &str, ids: &[&str]| {
        let s = svg(src);
        let bs = boxes(&s);
        for id in ids {
            let b = &bs[*id];
            for l in label_boxes(&s) {
                assert!((l.x - b.x).abs() >= (l.w + b.w) / 2. || (l.y - b.y).abs() >= (l.h + b.h) / 2.);
            }
        }
    };
    clear(
        "block\n  columns 1\n  a\n  space\n  c[\"Middle\"]\n  space\n  b\n  a -- \"forward label\" --> b\n  a -- \"second label\" --> b",
        &["c"],
    );
    clear(
        "block\n  columns 1\n  a c d e b\n  a -- \"one<br>two<br>three<br>four\" --> b\n  a -- \"one<br>two<br>three<br>four\" --> b",
        &["c", "e"],
    );
}
#[test]
fn avoid_composite_title() {
    let s=svg("block\n  columns 1\n  b\n  space\n  block:g[\"Group\"]\n    x\n  end\n  a\n  a -- \"a long label\" --> b\n  a -- \"a long label\" --> b\n  a -- \"a long label\" --> b");
    let c = captures(&s, r#"class="pele-cluster-label"[^>]* x="([\d.]+)" y="([\d.]+)""#);
    let x = number(&c[1]);
    let baseline = number(&c[2]);
    for l in label_boxes(&s) {
        assert!((l.x - x).abs() >= l.w / 2. || (l.y - (baseline - 6.)).abs() >= l.h / 2.);
    }
}
#[test]
fn avoid_arrowheads() {
    let s = svg(
        "block\n  columns 1\n  a\n  space\n  b\n  a -- \"first label\" --> b\n  a -- \"second label\" --> b\n  b -- \"third label\" --> a",
    );
    let tips = matches(&s, r#"class="pele-marker" d="([^"]*)""#)
        .into_iter()
        .flat_map(|m| matches(&m[1], r"(-?[\d.]+),(-?[\d.]+)").into_iter().map(|p| (number(&p[1]), number(&p[2]))).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(!tips.is_empty());
    for l in label_boxes(&s) {
        for (x, y) in &tips {
            assert!((x - l.x).abs() >= l.w / 2. || (y - l.y).abs() >= l.h / 2.);
        }
    }
}
#[test]
fn widen_parallel_gap() {
    let rows = "block\n  columns 1\n  a\n  b\n  a -- \"conforming\" --> b\n  b -- \"deviation\" --> a";
    separate(rows, 2);
    let near = layout("block\n  columns 1\n  a\n  b\n  a --> b\n  b --> a");
    let far = layout(rows);
    assert!(far["b"].y - far["a"].y > near["b"].y - near["a"].y);
    separate(
        "block\n  a[\"Service\"] space b[\"Database\"]\n  a -- \"read\" --> b\n  a -- \"write rows\" --> b\n  b -- \"notify\" --> a",
        3,
    );
    separate("block\n  columns 1\n  a\n  space\n  b\n  a -- \"one\" --> b\n  a -- \"two\" --> b\n  a -- \"three\" --> b\n  a -- \"four\" --> b\n  a -- \"five\" --> b",5);
    separate("block\n  columns 1\n  block:up[\"Up\"]\n    a b\n  end\n  c\n  up -- \"hi there\" --> c\n  c -- \"back again\" --> up", 2);
    straight(rows);
    separate("block\n  columns 2\n  block:g\n    columns 1\n    a{\"a\"} b\n  end\n  q[\"Q<br>Q<br>Q<br>Q<br>Q<br>Q\"]\n  a -- \"long label\" --> b\n  a -- \"long label\" --> b",2);
}
#[test]
fn no_gap_growth_without_gain() {
    let src="block\n  columns 3\n  t1 t2 t3\n  a space b\n  b1 b2 b3\n  a -- \"line one<br>line two<br>line three\" --> b\n  b -- \"back one<br>back two<br>back three\" --> a";
    assert_eq!(draw(src).width, draw(&re(r#" -- "[^"]*" --> "#).replace_all(src, " --> ")).width);
}
#[test]
fn columns_max_width() {
    let src = "block\n  a[\"Service\"] space b[\"Database\"]\n  a -- \"read\" --> b\n  a -- \"write rows\" --> b\n  b -- \"notify\" --> a";
    for width in [380., 397.5] {
        assert!(draw_with(src, json!({"maxWidth":width})).width <= width);
    }
}
#[test]
fn stacked_max_width() {
    let src="block\n  columns 1\n  a\n  space:2\n  b\n  a -- \"a fairly long forward label\" --> b\n  b -- \"a fairly long backward label\" --> a";
    separate(src, 2);
    assert!(draw_with(src, json!({"maxWidth":240})).width <= 240.);
}
#[test]
fn opaque_composite() {
    let s = svg("block\n  block:g\n    a\n  end\n  block:h\n    b\n  end\n  style g fill:#fee");
    assert!(has(&s, r#"data-id="g"><rect [^>]*fill="var\(--_a\)" stroke="#));
    assert!(has(&s, r#"data-id="h"><rect [^>]*fill-opacity="0.5""#));
}
#[test]
fn contrasting_labels() {
    let s=svg("block\n  block:g[\"Group\"]\n    a[\"Light\"]\n  end\n  b[\"Dark\"] c[\"Own\"] d[\"Plain\"]\n  e<[\"Go\"]>(right)\n  style g fill:#FFF9C4\n  style a fill:#BBDEFB\n  style b fill:#1f3a5f\n  style c fill:#1f3a5f,color:#eee\n  style e fill:black");
    for p in [
        r#"data-id="g">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>Group<"#,
        r#"data-id="a"[^>]*>.*?<text class="pele-label" fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>Light<"#,
        r#"data-id="b"[^>]*>.*?<text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>Dark<"#,
        r#"data-id="c"[^>]*>.*?<text class="pele-label" style="fill:#eee;"[^>]*>Own<"#,
        r#"data-id="d"[^>]*>.*?<text class="pele-label" x=[^>]*>Plain<"#,
        r#"data-id="e"[^>]*>.*?<text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>Go<"#,
    ] {
        assert!(has(&s, p), "{p}");
    }
}
#[test]
fn filled_edges() {
    let src = "block\n  block:g\n    a b\n  end\n  c\n  a --> b\n  b --> c\n  c --> c\n  style g fill:#BBDEFB";
    let s = svg(src);
    let c = captures(&s, r#"<g class="pele-casings"[^>]*>(.*?)</g>"#)[1].clone();
    let es = matches(&s, r#"<g class="pele-edge" data-id="[^"]+"([^>]*)><path d="([^"]+)""#);
    assert_eq!(es.len(), 3);
    assert_eq!(es.iter().map(|e| c.contains(&format!("<path d=\"{}\"/>", e[2]))).collect::<Vec<_>>(), [false, true, false]);
    assert_eq!(es.iter().map(|e| e[1].as_str()).collect::<Vec<_>>(), [" stroke=\"var(--pele-fg-on-light,#1f1f1f)\"", "", ""]);
    assert!(!has(&svg(&src.replace("\n  style g fill:#BBDEFB", "")), r"pele-casings|--pele-fg-on"));
    let loops = svg("block\n  block:g\n    a\n  end\n  a --> a\n  a --> a\n  style g fill:white");
    assert!(has(&loops, r#"data-id="1-a-a" stroke="var\(--pele-fg-on-light,#1f1f1f\)""#));
    assert!(has(&loops, r#"<g class="pele-edge" data-id="2-a-a"><path d="([^"]+)""#));
    let c = captures(&loops, r#"<g class="pele-casings"[^>]*>(.*?)</g>"#)[1].clone();
    let d = captures(&loops, r#"data-id="2-a-a"><path d="([^"]+)""#)[1].clone();
    assert!(c.contains(&format!("<path d=\"{d}\"/>")));
}
