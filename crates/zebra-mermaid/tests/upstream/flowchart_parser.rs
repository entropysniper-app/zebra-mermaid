//! 1:1 port of Pele tests/flowchart/parser.test.ts (MIT).
use super::{parser, FlowchartModel};
use std::time::Instant;
fn parse(source: &str) -> Result<FlowchartModel, crate::Error> {
    let mut model = FlowchartModel::default();
    // FlowDb() defaults maxEdges to Infinity; the public render pipeline supplies its own limit.
    model.max_edges = usize::MAX;
    parser::parse(source, &mut model)?;
    Ok(model)
}
#[test]
fn merges_repeated_subgraph_id() {
    let db = parse("flowchart LR\nsubgraph S\n  x\nend\nsubgraph S\n  y\nend").unwrap();
    assert_eq!(db.subgraphs.len(), 1);
    assert_eq!(db.subgraphs[0].nodes, ["x", "y"]);
}
#[test]
fn accepts_unnamed_subgraph() {
    let db = parse("flowchart TD\nsubgraph\n  a --> b\nend").unwrap();
    assert_eq!(db.subgraphs.len(), 1);
    assert_eq!(db.subgraphs[0].id, "subGraph0");
    assert_eq!(db.subgraphs[0].nodes, ["b", "a"]);
}
#[test]
fn reports_nonexistent_link_style() {
    assert!(parse("graph TD\nA-->B\nlinkStyle 1 stroke-width:1px;")
        .unwrap_err()
        .message
        .contains("The index 1 for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and 0."));
}
#[test]
fn reports_syntax_position() {
    let e: crate::Error = parse("graph TD\nA-->B\nB --> [oops]").unwrap_err();
    assert_eq!(e.line, 3);
    assert_eq!(e.column, 7);
    assert!(e.message.contains("got 'SQS'"));
}
#[test]
fn huge_diagram_within_original_budget() {
    let body = ("A-->B;B-->A;".repeat(415) + "A-->B;").repeat(57) + &"A-->B;B-->A;".repeat(275);
    let started = Instant::now();
    let db = parse(&format!("graph LR;{body}")).unwrap();
    assert_eq!(db.edges[0].kind, "arrow_point");
    assert_eq!(db.edges.len(), 47917);
    assert_eq!(db.nodes.len(), 2);
    assert!(started.elapsed().as_secs_f64() * 1000.0 < 2000.0);
}
#[test]
fn honours_explicit_edge_limit() {
    let mut db = FlowchartModel::default();
    db.max_edges = 2;
    assert!(parser::parse("graph TD\nA-->B-->C-->D", &mut db).unwrap_err().message.contains("Edge limit exceeded"));
}
