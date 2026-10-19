//! Direct ports of all 31 executable tests in Pele flowDb.spec.ts (MIT).
//! Case 7 is an upstream-skipped JavaScript own-property reflection test, not a native pass.
use super::{
    graph::{self, FlowGraph, GraphNode},
    model::{DocItem, FlowText, Link, Subgraph},
    parser, FlowchartModel,
};
use std::collections::HashSet;
fn fresh() -> FlowchartModel {
    let mut m = FlowchartModel::default();
    m.max_edges = 500;
    m
}
fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).into()).collect()
}
fn sub(nodes: &[&str]) -> Subgraph {
    Subgraph { nodes: strings(nodes), ..Default::default() }
}
fn groups() -> Vec<Subgraph> {
    vec![sub(&["a", "b", "c", "e"]), sub(&["f", "g", "h"]), sub(&["i", "j"]), sub(&["k"])]
}
fn vertex(m: &mut FlowchartModel, id: &str) {
    m.vertex(id, Some(FlowText { text: id.into(), kind: "text".into() }), None, Some(vec![]), Some(Default::default()), None).unwrap();
    m.nodes.get_mut(id).unwrap().dir = Some("".into());
}
fn meta(m: &mut FlowchartModel, id: &str, metadata: &str, text: Option<&str>) {
    m.vertex(
        id,
        text.map(|t| FlowText { text: t.into(), kind: "text".into() }),
        None,
        Some(vec![]),
        Some(Default::default()),
        Some(metadata),
    )
    .unwrap();
}
fn edge(m: &mut FlowchartModel, a: &str, b: &str, id: Option<&str>) {
    m.add_link(&strings(&[a]), &strings(&[b]), &Link { id: id.map(str::to_owned), ..Default::default() }).unwrap();
}
fn group(m: &mut FlowchartModel, id: &str, nodes: &[&str], title: &str) {
    m.add_subgraph(
        Some(FlowText { text: id.into(), kind: String::new() }),
        nodes.iter().map(|id| DocItem::Text((*id).into())).collect(),
        Some(FlowText { text: title.into(), kind: "text".into() }),
        false,
    );
}
fn draw(m: &FlowchartModel) -> FlowGraph {
    graph::build(m, None)
}
fn node<'a>(g: &'a FlowGraph, id: &str) -> Option<&'a GraphNode> {
    g.nodes.iter().find(|n| n.id == id)
}
fn parse(source: &str) -> FlowchartModel {
    let mut m = fresh();
    parser::parse(source, &mut m).unwrap();
    m
}
fn sg<'a>(m: &'a FlowchartModel, id: &str) -> &'a Subgraph {
    m.subgraphs.iter().find(|s| s.id == id).unwrap()
}
fn ids(m: &FlowchartModel) -> Vec<&str> {
    m.subgraphs.iter().map(|s| s.id.as_str()).collect()
}
#[test]
fn case_00() {
    for id in ["a", "h", "j", "k"] {
        assert!(FlowchartModel::exists(&groups(), id));
    }
}
#[test]
fn case_01() {
    for id in ["a2", "l"] {
        assert!(!FlowchartModel::exists(&groups(), id));
    }
}
#[test]
fn case_02() {
    assert_eq!(FlowchartModel::make_unique(&sub(&["i", "j"]), &groups()), Vec::<String>::new());
}
#[test]
fn case_03() {
    assert_eq!(FlowchartModel::make_unique(&sub(&["i", "j", "o"]), &groups()), ["o"]);
}
#[test]
fn case_04() {
    assert_eq!(FlowchartModel::make_unique(&sub(&["q", "r", "s"]), &groups()), ["q", "r", "s"]);
}
#[test]
fn case_05() {
    let mut m = fresh();
    m.add_class("a,b", strings(&["stroke-width: 8px"]));
    assert!(m.classes.contains_key("a"));
    assert!(m.classes.contains_key("b"));
    assert_eq!(m.classes["a"].styles, ["stroke-width: 8px"]);
    assert_eq!(m.classes["b"].styles, ["stroke-width: 8px"]);
}
#[test]
fn case_06() {
    let mut m = fresh();
    m.add_class("a", strings(&["stroke-width: 8px"]));
    assert!(m.classes.contains_key("a"));
    assert_eq!(m.classes["a"].styles, ["stroke-width: 8px"]);
}
#[test]
fn case_08() {
    let mut m = fresh();
    vertex(&mut m, "A");
    vertex(&mut m, "B");
    edge(&mut m, "A", "B", None);
    m.interpolate(&strings(&["default"]), "stepBefore").unwrap();
    assert_eq!(draw(&m).edges[0].curve.as_deref(), Some("stepBefore"));
}
#[test]
fn case_09() {
    let mut m = fresh();
    vertex(&mut m, "A");
    vertex(&mut m, "B");
    edge(&mut m, "A", "B", None);
    m.interpolate(&strings(&["default"]), "stepBefore").unwrap();
    m.interpolate(&strings(&["0"]), "basis").unwrap();
    assert_eq!(draw(&m).edges[0].curve.as_deref(), Some("basis"));
}
#[test]
fn case_10() {
    let mut m = fresh();
    for id in ["A", "B", "C", "D"] {
        vertex(&mut m, id);
    }
    edge(&mut m, "A", "B", None);
    edge(&mut m, "A", "C", Some("e2"));
    edge(&mut m, "B", "D", Some("e3"));
    edge(&mut m, "C", "D", None);
    m.interpolate(&strings(&["default"]), "stepBefore").unwrap();
    m.interpolate(&strings(&["0"]), "basis").unwrap();
    meta(&mut m, "e2", " curve: monotoneX ", Some("Shouldnt be used"));
    meta(&mut m, "e3", " curve: catmullRom ", Some("Shouldnt be used"));
    let g = draw(&m);
    assert_eq!(g.edges[0].curve.as_deref(), Some("basis"));
    assert_eq!(g.edges[1].curve.as_deref(), Some("monotoneX"));
    assert_eq!(g.edges[2].curve.as_deref(), Some("catmullRom"));
    assert_eq!(g.edges[3].curve.as_deref(), Some("stepBefore"));
}
#[test]
fn case_11() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    edge(&mut m, "A", "B", None);
    edge(&mut m, "C", "A", None);
    group(&mut m, "sub1", &["A", "B"], "My Group");
    let g = draw(&m);
    assert!(node(&g, "sub1").unwrap().is_group);
    for id in ["A", "B", "C", "sub1"] {
        assert!(node(&g, id).is_some());
    }
    assert!(g.edges.iter().any(|e| e.start == "C" && e.end == "A"));
}
#[test]
fn case_12() {
    let mut m = fresh();
    vertex(&mut m, "A");
    vertex(&mut m, "B");
    group(&mut m, "sub1", &["A", "B"], "My Group");
    meta(&mut m, "sub1", " algorithm: elk.box ", None);
    let g = draw(&m);
    let sub = node(&g, "sub1").unwrap();
    assert!(sub.is_group);
    assert_eq!(sub.metadata.as_ref().unwrap().get("algorithm").unwrap().as_str(), Some("elk.box"));
}
#[test]
fn case_13() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    edge(&mut m, "A", "B", None);
    edge(&mut m, "C", "A", None);
    group(&mut m, "sub1", &["A", "B"], "My Group");
    meta(&mut m, "sub1", " view: collapsed ", None);
    let g = draw(&m);
    let sub = node(&g, "sub1").unwrap();
    assert_eq!(sub.shape, "collapsedGroup");
    assert!(!sub.is_group);
    assert_eq!(sub.label.as_deref(), Some("My Group"));
    assert!(node(&g, "A").is_none());
    assert!(node(&g, "B").is_none());
    assert!(node(&g, "C").is_some());
}
#[test]
fn case_14() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    edge(&mut m, "C", "A", None);
    group(&mut m, "sub1", &["A", "B"], "My Group");
    meta(&mut m, "sub1", " view: collapsed ", None);
    let g = draw(&m);
    assert!(g.edges.iter().any(|e| e.start == "C" && e.end == "sub1"));
    assert!(!g.edges.iter().any(|e| e.end == "A"));
}
#[test]
fn case_15() {
    let mut m = fresh();
    vertex(&mut m, "A");
    vertex(&mut m, "B");
    edge(&mut m, "A", "B", None);
    group(&mut m, "sub1", &["A", "B"], "My Group");
    meta(&mut m, "sub1", " view: collapsed ", None);
    assert!(draw(&m).edges.is_empty());
}
#[test]
fn case_16() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    edge(&mut m, "C", "A", None);
    group(&mut m, "inner", &["A", "B"], "Inner");
    group(&mut m, "outer", &["inner"], "Outer");
    meta(&mut m, "inner", " view: collapsed ", None);
    meta(&mut m, "outer", " view: collapsed ", None);
    let g = draw(&m);
    assert_eq!(node(&g, "outer").unwrap().shape, "collapsedGroup");
    assert!(node(&g, "inner").is_none());
    assert!(node(&g, "A").is_none());
    assert!(g.edges.iter().any(|e| e.start == "C" && e.end == "outer"));
}
#[test]
fn case_17() {
    let mut m = fresh();
    m.set_direction("TD");
    assert_eq!(m.direction.as_deref(), Some("TB"));
}
#[test]
fn case_18() {
    let mut m = fresh();
    m.set_direction(" TD");
    assert_eq!(m.direction.as_deref(), Some("TB"));
}
#[test]
fn case_19() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    group(&mut m, "first", &["A"], "First");
    group(&mut m, "second", &["B"], "Second");
    group(&mut m, "third", &["C"], "Third");
    let g = draw(&m);
    assert_eq!(
        [node(&g, "first").unwrap().color_index, node(&g, "second").unwrap().color_index, node(&g, "third").unwrap().color_index],
        [Some(0), Some(1), Some(2)]
    );
}
#[test]
fn case_20() {
    let mut m = fresh();
    for id in ["A", "B", "C", "D"] {
        vertex(&mut m, id);
    }
    group(&mut m, "InnerOne", &["A"], "InnerOne");
    group(&mut m, "InnerTwo", &["B"], "InnerTwo");
    group(&mut m, "Outer", &["InnerOne", "InnerTwo"], "Outer");
    group(&mut m, "Sibling", &["D"], "Sibling");
    let g = draw(&m);
    assert_eq!(
        [
            node(&g, "Outer").unwrap().color_index,
            node(&g, "InnerOne").unwrap().color_index,
            node(&g, "InnerTwo").unwrap().color_index,
            node(&g, "Sibling").unwrap().color_index
        ],
        [Some(0), Some(1), Some(2), Some(3)]
    );
}
#[test]
fn case_21() {
    let mut m = fresh();
    for id in ["A", "B", "C"] {
        vertex(&mut m, id);
    }
    group(&mut m, "first", &["A"], "First");
    group(&mut m, "second", &["B"], "Second");
    group(&mut m, "third", &["C"], "Third");
    meta(&mut m, "second", " view: collapsed ", None);
    let g = draw(&m);
    assert_eq!(node(&g, "first").unwrap().color_index, Some(0));
    assert_eq!(node(&g, "second").unwrap().color_index, Some(1));
    assert_eq!(node(&g, "third").unwrap().color_index, Some(2));
}
#[test]
fn case_22() {
    let mut m = fresh();
    vertex(&mut m, "A");
    group(&mut m, "only", &["A"], "Only");
    let g = draw(&m);
    assert!(node(&g, "A").unwrap().color_index.is_none());
    assert_eq!(node(&g, "only").unwrap().color_index, Some(0));
}
#[test]
fn case_23() {
    let m = parse("flowchart LR\nsubgraph S\n  x\nend\nsubgraph S\n  y\nend");
    assert_eq!(ids(&m), ["S"]);
    assert_eq!(sg(&m, "S").nodes, ["x", "y"]);
}
#[test]
fn case_24() {
    let m = parse("flowchart LR\nsubgraph S [First]\n  x\nend\nsubgraph S [Second]\n  y\nend");
    assert_eq!(ids(&m), ["S"]);
    assert_eq!(sg(&m, "S").title, "First");
}
#[test]
fn case_25() {
    let m = parse("flowchart LR\nsubgraph S\n  x\nend\nsubgraph S\n  y\nend");
    let g = draw(&m);
    assert_eq!(g.nodes.iter().filter(|n| n.id == "S").count(), 1);
    assert!(node(&g, "S").unwrap().is_group);
    assert_eq!(node(&g, "x").unwrap().parent.as_deref(), Some("S"));
    assert_eq!(node(&g, "y").unwrap().parent.as_deref(), Some("S"));
    assert_eq!(g.nodes.iter().map(|n| &n.id).collect::<HashSet<_>>().len(), g.nodes.len());
}
#[test]
fn case_26() {
    let m = parse("flowchart LR\nsubgraph S\n  x\nend\nsubgraph S\n  y\nend\nclassDef hot fill:#f00\nclass S hot");
    assert!(sg(&m, "S").classes.contains(&"hot".to_string()));
    assert!(node(&draw(&m), "S").unwrap().classes.contains("hot"));
}
fn collapse_repeated(source: &str) {
    let m = parse(source);
    assert_eq!(
        crate::ConfigValue::Object(sg(&m, "S").metadata.clone()),
        crate::config::json::parse_json(r#"{"view":"collapsed"}"#).unwrap()
    );
    let g = draw(&m);
    assert_eq!(g.nodes.iter().filter(|n| n.id == "S").count(), 1);
    assert_eq!(node(&g, "S").unwrap().shape, "collapsedGroup");
    assert!(node(&g, "x").is_none());
    assert!(node(&g, "y").is_none());
}
#[test]
fn case_27() {
    collapse_repeated("flowchart LR\nsubgraph S\n  x\nend\nsubgraph S\n  y\nend\nS@{ view: collapsed }");
}
#[test]
fn case_28() {
    collapse_repeated("flowchart LR\nsubgraph S\n  x\nend\nS@{ view: collapsed }\nsubgraph S\n  y\nend");
}
#[test]
fn case_29() {
    let m = parse("flowchart LR\nsubgraph S\n  subgraph S\n    x\n  end\n  y\nend");
    assert_eq!(ids(&m), ["S"]);
    let members = &sg(&m, "S").nodes;
    assert!(!members.contains(&"S".into()));
    for id in ["x", "y"] {
        assert!(members.contains(&id.into()));
    }
    let g = draw(&m);
    assert_eq!(g.nodes.iter().filter(|n| n.id == "S").count(), 1);
    assert!(!g.nodes.iter().any(|n| n.parent.as_deref() == Some(n.id.as_str())));
    assert_eq!(node(&g, "x").unwrap().parent.as_deref(), Some("S"));
    assert_eq!(node(&g, "y").unwrap().parent.as_deref(), Some("S"));
}
#[test]
fn case_30() {
    let m = parse("flowchart LR\nsubgraph S\nend\nsubgraph S\n  subgraph T\n    y\n  end\nend");
    let mut got = ids(&m);
    got.sort();
    assert_eq!(got, ["S", "T"]);
    assert_eq!(sg(&m, "S").nodes, ["T"]);
    assert_eq!(sg(&m, "T").nodes, ["y"]);
    let g = draw(&m);
    assert_eq!(g.nodes.iter().filter(|n| n.id == "S").count(), 1);
    assert_eq!(node(&g, "T").unwrap().parent.as_deref(), Some("S"));
    assert_eq!(node(&g, "y").unwrap().parent.as_deref(), Some("T"));
}
#[test]
fn case_31() {
    let m = parse(
        "flowchart LR\nsubgraph A\n  a\nend\nsubgraph B\n  b\nend\nsubgraph \"Same Title\"\n  c\nend\nsubgraph \"Same Title\"\n  d\nend",
    );
    assert_eq!(m.subgraphs.len(), 4);
    assert_eq!(sg(&m, "A").nodes, ["a"]);
    assert_eq!(sg(&m, "B").nodes, ["b"]);
    assert_eq!(sg(&m, "subGraph2").nodes, ["c"]);
    assert_eq!(sg(&m, "subGraph3").nodes, ["d"]);
}
