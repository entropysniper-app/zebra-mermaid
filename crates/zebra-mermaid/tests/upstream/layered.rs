//! 1:1 port: Pele 1.9.0 tests/layout/layered.test.ts (MIT, kepano 2026).
use zebra_mermaid::layout::layered::{layered, ledge, lnode, Kind, LayeredGroups, LayeredOptions};
#[test]
fn groups_stay_together_and_report_the_crossings_of_the_retained_order() {
    let mut nodes = (0..6).map(|_| lnode(40.0, 40.0, Kind::Node)).collect::<Vec<_>>();
    let of = vec![0, 1, 0, -1, 0, -1];
    let mut edges = [(0, 4), (1, 4), (2, 5), (3, 4), (3, 5)].into_iter().map(|(a, b)| ledge(a, b, 1.0, 0.0, 0.0)).collect::<Vec<_>>();
    let result = layered(
        &mut nodes,
        &mut edges,
        &LayeredOptions {
            node_sep: 40.0,
            edge_sep: 16.0,
            rank_sep: 48.0,
            port_sep: 20.0,
            groups: Some(LayeredGroups { of: of.clone(), parent: vec![-1, -1] }),
            order_only: true,
            ..Default::default()
        },
    );
    let mut pieces = Vec::new();
    for e in &edges {
        for k in (2..e.points.len()).step_by(2) {
            pieces.push([e.points[k - 2], e.points[k - 1], e.points[k + 1]]);
        }
    }
    let mut crossed = 0;
    for i in 0..pieces.len() {
        for j in i + 1..pieces.len() {
            let [r, a0, a1] = pieces[i];
            let [s, b0, b1] = pieces[j];
            if r == s && (a0 - b0) * (a1 - b1) < 0.0 {
                crossed += 1;
            }
        }
    }
    assert_eq!(result.crossings, Some(crossed));
    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if of[i] < 0 || of[j] != of[i] || nodes[i].rank != nodes[j].rank || nodes[i].x >= nodes[j].x {
                continue;
            }
            for k in 0..nodes.len() {
                let between = nodes[k].rank == nodes[i].rank && nodes[k].x > nodes[i].x && nodes[k].x < nodes[j].x;
                assert!(!(between && of[k] != of[i]), "node {k} between {i} and {j}");
            }
        }
    }
}
