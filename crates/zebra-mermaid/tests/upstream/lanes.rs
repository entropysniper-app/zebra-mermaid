//! 1:1 port: Pele tests/layout/lanes.test.ts (MIT, kepano 2026).
use std::collections::BTreeSet;
use zebra_mermaid::layout::{
    compound::{cnode, CEdge, CNode, Dir},
    lanes::{lane_layout, LANE_PAD},
    layered::LayeredOptions,
};
use zebra_mermaid::svg::shapes::end_side;
const DIRS: [Dir; 4] = [Dir::TB, Dir::BT, Dir::LR, Dir::RL];
fn options() -> LayeredOptions {
    LayeredOptions {
        node_sep: 40.0,
        edge_sep: 16.0,
        rank_sep: 48.0,
        port_sep: 20.0,
        tracks: false,
        straight: false,
        apart: false,
        head_room: None,
    }
}
fn node(w: f64, h: f64) -> CNode {
    cnode(w, h, -1)
}
fn group(parent: isize) -> CNode {
    let mut n = cnode(0.0, 0.0, parent);
    n.is_group = true;
    n.pad_x = 20.0;
    n.pad_top = 36.0;
    n.pad_bottom = 20.0;
    n.min_w = 60.0;
    n
}
fn edge(a: usize, b: usize) -> CEdge {
    labelled(a, b, 0.0, 0.0)
}
fn labelled(src: usize, dst: usize, w: f64, h: f64) -> CEdge {
    CEdge { src, dst, minlen: 1.0, label_w: w, label_h: h, route: vec![], label_x: 0.0, label_y: 0.0 }
}
fn layout(n: &mut [CNode], e: &mut [CEdge], d: Dir) -> zebra_mermaid::layout::compound::CompoundResult {
    lane_layout(n, e, d, &options(), LANE_PAD)
}
fn side(r: &[f64], at: usize, n: &CNode) -> i32 {
    end_side(r, at, n.x, n.y, n.w, n.h)
}
fn box_of(c: &CNode) -> [f64; 4] {
    [c.x - c.w / 2.0, c.y - c.h / 2.0, c.x + c.w / 2.0, c.y + c.h / 2.0]
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 0.5e-6, "{a} != {b}");
}
struct Random(u32);
impl Random {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0 as f64 / 4294967296.0
    }
    fn int(&mut self, n: usize) -> usize {
        (self.next() * n as f64).floor() as usize
    }
}
struct Sample {
    nodes: Vec<CNode>,
    edges: Vec<CEdge>,
    lanes: Vec<usize>,
    leaves: Vec<usize>,
    lane_of: Vec<usize>,
    inner: Vec<usize>,
    inner_lane: Vec<usize>,
    dir: Dir,
}
fn random_graph(seed: u32) -> Sample {
    let mut rnd = Random(seed);
    let mut nodes = vec![];
    let mut leaves = vec![];
    let mut lane_of = vec![];
    let mut lanes = vec![];
    let leaf_count = 1 + rnd.int(30);
    let lane_count = 1 + rnd.int(5);
    let mut inner = vec![];
    let mut inner_lane = vec![];
    for i in 0..leaf_count {
        nodes.push(node((30 + rnd.int(120)) as f64, (24 + rnd.int(60)) as f64));
        leaves.push(i);
    }
    for _ in 0..lane_count {
        lanes.push(nodes.len());
        nodes.push(group(-1));
    }
    // Upstream redraws int(4) in each loop condition; it is NOT a sampled constant.
    let mut k = 0;
    while k < rnd.int(4) {
        let lane = lanes[rnd.int(lane_count)];
        inner.push(nodes.len());
        inner_lane.push(lane);
        nodes.push(group(lane as isize));
        k += 1;
    }
    for &i in &leaves {
        if !inner.is_empty() && rnd.next() < 0.25 {
            let g = rnd.int(inner.len());
            nodes[i].parent = inner[g] as isize;
            lane_of.push(inner_lane[g]);
        } else {
            let lane = lanes[rnd.int(lane_count)];
            lane_of.push(lane);
            nodes[i].parent = lane as isize;
        }
    }
    let mut edges = vec![];
    let count = rnd.int(leaf_count * 2);
    for _ in 0..count {
        let has = rnd.next() < 0.3;
        edges.push(labelled(
            rnd.int(leaf_count),
            rnd.int(leaf_count),
            if has { (20 + rnd.int(80)) as f64 } else { 0.0 },
            if has { 18.0 } else { 0.0 },
        ));
    }
    if rnd.next() < 0.3 {
        edges.push(edge(lanes[0], leaves[0]));
    }
    Sample { nodes, edges, lanes, leaves, lane_of, inner, inner_lane, dir: DIRS[rnd.int(4)] }
}
#[test]
fn lays_out_nothing() {
    let r = layout(&mut [], &mut [], Dir::TB);
    assert_eq!(r.width, 0.0);
    assert_eq!(r.height, 0.0);
}
#[test]
fn lanes_follow_written_order() {
    for dir in DIRS {
        let mut n = [node(60.0, 30.0), node(60.0, 30.0), group(-1), group(-1)];
        n[0].parent = 3;
        n[1].parent = 2;
        let r = layout(&mut n, &mut [edge(0, 1)], dir);
        let (first, second) = (&n[3], &n[2]);
        if matches!(dir, Dir::TB | Dir::BT) {
            assert!(first.x < second.x);
            assert_eq!(first.h, r.height);
            close(first.w + second.w, r.width);
        } else {
            assert!(first.y < second.y);
            assert_eq!(first.w, r.width);
            close(first.h + second.h, r.height);
        }
    }
}
#[test]
fn follows_requested_direction() {
    let at = |d| {
        let mut n = [node(60.0, 30.0), node(60.0, 30.0), group(-1)];
        n[0].parent = 2;
        n[1].parent = 2;
        layout(&mut n, &mut [edge(0, 1)], d);
        n
    };
    assert!(at(Dir::TB)[0].y < at(Dir::TB)[1].y);
    assert!(at(Dir::BT)[0].y > at(Dir::BT)[1].y);
    assert!(at(Dir::LR)[0].x < at(Dir::LR)[1].x);
    assert!(at(Dir::RL)[0].x > at(Dir::RL)[1].x);
}
#[test]
fn chain_stays_straight() {
    let mut n = [node(50.0, 30.0), node(120.0, 30.0), node(70.0, 30.0), group(-1)];
    for v in &mut n[..3] {
        v.parent = 3;
    }
    layout(&mut n, &mut [edge(0, 1), edge(1, 2)], Dir::TB);
    close(n[0].x, n[1].x);
    close(n[1].x, n[2].x);
}
fn approval(dir: Dir) {
    let mut n = [
        node(60.0, 40.0),
        node(80.0, 40.0),
        node(70.0, 70.0),
        node(80.0, 40.0),
        node(80.0, 40.0),
        node(60.0, 40.0),
        group(-1),
        group(-1),
        group(-1),
    ];
    n[0].parent = 8;
    n[5].parent = 8;
    n[1].parent = 7;
    n[2].parent = 7;
    n[3].parent = 6;
    n[4].parent = 6;
    let mut e = [labelled(0, 1, 70.0, 20.0), edge(1, 2), labelled(2, 3, 50.0, 20.0), edge(3, 4), edge(4, 5), labelled(2, 0, 70.0, 20.0)];
    layout(&mut n, &mut e, dir);
    let sides = [side(&e[1].route, e[1].route.len() - 3, &n[2]), side(&e[2].route, 0, &n[2]), side(&e[5].route, 0, &n[2])];
    assert_eq!(sides.into_iter().collect::<BTreeSet<_>>().len(), 3);
    assert!(!sides.contains(&-1));
    let back = &e[5];
    let r = &back.route;
    let corners = (6..r.len()).step_by(3).filter(|&p| (r[p] == r[p - 3]) != (r[p - 3] == r[p - 6])).count();
    assert_eq!(corners, 1);
    for p in (3..r.len()).step_by(3) {
        assert!(r[p] == r[p - 3] || r[p + 1] == r[p - 2]);
    }
    for n in &n[..6] {
        let [l, t, r, b] = box_of(n);
        assert!(
            back.label_x + back.label_w / 2.0 <= l
                || back.label_x - back.label_w / 2.0 >= r
                || back.label_y + back.label_h / 2.0 <= t
                || back.label_y - back.label_h / 2.0 >= b
        );
    }
}
#[test]
fn approval_tb() {
    approval(Dir::TB)
}
#[test]
fn approval_bt() {
    approval(Dir::BT)
}
#[test]
fn approval_lr() {
    approval(Dir::LR)
}
#[test]
fn approval_rl() {
    approval(Dir::RL)
}
fn handoffs(dir: Dir) {
    let mut n = [node(110.0, 38.0), node(110.0, 38.0), node(100.0, 38.0), node(110.0, 38.0), group(-1), group(-1)];
    n[0].parent = 5;
    n[1].parent = 5;
    n[2].parent = 4;
    n[3].parent = 4;
    let mut e = [edge(2, 3), edge(0, 3), edge(3, 1)];
    layout(&mut n, &mut e, dir);
    let into = &e[1].route;
    let out = &e[2].route;
    assert_ne!(side(into, into.len() - 3, &n[3]), side(out, 0, &n[3]));
    for p in (3..into.len()).step_by(3) {
        for q in (3..out.len()).step_by(3) {
            let upright = into[p] == into[p - 3] && out[q] == out[q - 3] && into[p] == out[q];
            let level = into[p + 1] == into[p - 2] && out[q + 1] == out[q - 2] && into[p + 1] == out[q + 1];
            let k = usize::from(upright);
            let shared = into[p + k].max(into[p - 3 + k]).min(out[q + k].max(out[q - 3 + k]))
                - into[p + k].min(into[p - 3 + k]).max(out[q + k].min(out[q - 3 + k]));
            if upright || level {
                assert!(shared <= 0.0);
            }
        }
    }
}
#[test]
fn handoffs_tb() {
    handoffs(Dir::TB)
}
#[test]
fn handoffs_bt() {
    handoffs(Dir::BT)
}
#[test]
fn handoffs_lr() {
    handoffs(Dir::LR)
}
#[test]
fn handoffs_rl() {
    handoffs(Dir::RL)
}
fn return_path(dir: Dir) {
    let mut n = [
        node(140.0, 44.0),
        node(140.0, 44.0),
        node(140.0, 44.0),
        node(120.0, 76.0),
        node(140.0, 44.0),
        node(140.0, 44.0),
        group(-1),
        group(-1),
        group(-1),
    ];
    n[0].parent = 8;
    n[1].parent = 8;
    n[2].parent = 7;
    n[3].parent = 7;
    n[4].parent = 6;
    n[5].parent = 6;
    let mut e = [edge(0, 1), edge(1, 2), edge(2, 3), labelled(3, 4, 30.0, 20.0), edge(4, 5), labelled(3, 0, 30.0, 20.0)];
    layout(&mut n, &mut e, dir);
    let sideways = matches!(dir, Dir::LR | Dir::RL);
    let across = |n: &CNode| if sideways { n.y } else { n.x };
    let along = |n: &CNode| if sideways { n.x } else { n.y };
    let depth = |n: &CNode| if sideways { n.w } else { n.h };
    close(across(&n[0]), across(&n[1]));
    close(across(&n[4]), across(&n[5]));
    let back = &e[5].route;
    let sides = [side(&e[2].route, e[2].route.len() - 3, &n[3]), side(&e[3].route, 0, &n[3]), side(back, 0, &n[3])];
    assert_eq!(sides.into_iter().collect::<BTreeSet<_>>().len(), 3);
    assert_ne!(side(back, back.len() - 3, &n[0]), side(&e[0].route, 0, &n[0]));
    for p in (3..back.len()).step_by(3) {
        assert!(back[p] == back[p - 3] || back[p + 1] == back[p - 2]);
        for (i, n) in n[..6].iter().enumerate() {
            if i == 0 || i == 3 {
                continue;
            }
            let [l, t, r, b] = box_of(n);
            assert!(
                back[p].max(back[p - 3]) <= l
                    || back[p].min(back[p - 3]) >= r
                    || back[p + 1].max(back[p - 2]) <= t
                    || back[p + 1].min(back[p - 2]) >= b
            );
        }
    }
    let gap = |a: usize, b: usize| (along(&n[b]) - along(&n[a])).abs() - (depth(&n[a]) + depth(&n[b])) / 2.0;
    assert!(gap(1, 2) < gap(0, 1));
    close(gap(0, 1), options().rank_sep);
}
#[test]
fn return_tb() {
    return_path(Dir::TB)
}
#[test]
fn return_bt() {
    return_path(Dir::BT)
}
#[test]
fn return_lr() {
    return_path(Dir::LR)
}
#[test]
fn return_rl() {
    return_path(Dir::RL)
}
fn longer(dir: Dir) {
    let mut n = [node(80.0, 40.0), node(80.0, 40.0), node(80.0, 40.0), node(80.0, 40.0), group(-1), group(-1)];
    n[0].parent = 5;
    n[1].parent = 5;
    n[2].parent = 5;
    n[3].parent = 4;
    let mut e = [edge(0, 1), edge(1, 2), edge(2, 3)];
    e[0].minlen = 2.0;
    layout(&mut n, &mut e, dir);
    let sideways = matches!(dir, Dir::LR | Dir::RL);
    let gap = |a: usize, b: usize| if sideways { (n[b].x - n[a].x).abs() - 80.0 } else { (n[b].y - n[a].y).abs() - 40.0 };
    close(gap(0, 1), 2.0 * options().rank_sep);
    close(gap(1, 2), options().rank_sep);
}
#[test]
fn longer_tb() {
    longer(Dir::TB)
}
#[test]
fn longer_bt() {
    longer(Dir::BT)
}
#[test]
fn longer_lr() {
    longer(Dir::LR)
}
#[test]
fn longer_rl() {
    longer(Dir::RL)
}
#[test]
fn handoff_avoids_lane_relation_label() {
    let mut n = vec![node(110.0, 38.0); 6];
    n.extend([group(-1), group(-1)]);
    n[0].parent = 7;
    for n in &mut n[1..6] {
        n.parent = 6;
    }
    let mut e = [edge(0, 1), edge(1, 2), edge(2, 3), edge(3, 4), edge(4, 5), labelled(6, 5, 400.0, 200.0)];
    layout(&mut n, &mut e, Dir::TB);
    let r = &e[0].route;
    assert!(r.len() > 6);
    for p in (3..r.len()).step_by(3) {
        assert!(r[p] == r[p - 3] || r[p + 1] == r[p - 2]);
    }
    for p in (1..r.len()).step_by(3) {
        assert!((r[p] - e[5].label_y).abs() > 100.0);
    }
}
fn random_graphs(pad: f64) {
    for seed in 1..=300 {
        let Sample { mut nodes, mut edges, lanes, leaves, lane_of, inner, inner_lane, dir } = random_graph(seed);
        let result = lane_layout(&mut nodes, &mut edges, dir, &options(), pad);
        assert!(result.width.is_finite() && result.height.is_finite(), "seed {seed}");
        for c in &nodes {
            assert!(c.x.is_finite() && c.y.is_finite() && c.w.is_finite() && c.h.is_finite());
        }
        for &i in &leaves {
            let [l, t, r, b] = box_of(&nodes[i]);
            let [ll, lt, lr, lb] = box_of(&nodes[lane_of[i]]);
            assert!(l >= ll - 0.01 && r <= lr + 0.01 && t >= lt - 0.01 && b <= lb + 0.01, "seed {seed} node {i} in lane");
        }
        for (k, &g) in inner.iter().enumerate() {
            if !leaves.iter().any(|&i| nodes[i].parent == g as isize) {
                continue;
            }
            let [l, t, r, b] = box_of(&nodes[g]);
            let [ll, lt, lr, lb] = box_of(&nodes[inner_lane[k]]);
            assert!(l >= ll - 0.01 && r <= lr + 0.01 && t >= lt - 0.01 && b <= lb + 0.01, "seed {seed} group {g}");
            for &i in &leaves {
                if nodes[i].parent != g as isize {
                    continue;
                }
                let [nl, nt, nr, nb] = box_of(&nodes[i]);
                assert!(nl >= l - 0.01 && nr <= r + 0.01 && nt >= t - 0.01 && nb <= b + 0.01, "seed {seed} node {i} in group");
            }
        }
        for a in 0..leaves.len() {
            for b in a + 1..leaves.len() {
                let [al, at, ar, ab] = box_of(&nodes[leaves[a]]);
                let [bl, bt, br, bb] = box_of(&nodes[leaves[b]]);
                assert!(ar <= bl + 0.01 || br <= al + 0.01 || ab <= bt + 0.01 || bb <= at + 0.01, "seed {seed} nodes {a},{b}");
            }
        }
        for a in 0..lanes.len() {
            let [l, t, r, b] = box_of(&nodes[lanes[a]]);
            assert!(l >= -0.01 && t >= -0.01 && r <= result.width + 0.01 && b <= result.height + 0.01, "seed {seed} lane inside");
            for k in a + 1..lanes.len() {
                let [ol, ot, or, ob] = box_of(&nodes[lanes[k]]);
                assert!(r <= ol + 0.01 || or <= l + 0.01 || b <= ot + 0.01 || ob <= t + 0.01, "seed {seed} lanes overlap");
            }
        }
        for e in &edges {
            if e.src == e.dst {
                continue;
            }
            assert!(e.route.len() >= 6 && e.route.len() % 3 == 0);
            for v in &e.route {
                assert!(v.is_finite());
            }
            for p in (0..e.route.len()).step_by(3) {
                assert!(
                    e.route[p] >= -0.01
                        && e.route[p] <= result.width + 0.01
                        && e.route[p + 1] >= -0.01
                        && e.route[p + 1] <= result.height + 0.01,
                    "seed {seed} route inside"
                );
            }
            assert!(e.label_x.is_finite() && e.label_y.is_finite());
        }
    }
}
#[test]
fn random_graphs_pad_24() {
    random_graphs(24.0)
}
#[test]
fn random_graphs_pad_12() {
    random_graphs(12.0)
}
#[test]
fn deterministic() {
    let mut a = random_graph(42);
    let mut b = random_graph(42);
    layout(&mut a.nodes, &mut a.edges, a.dir);
    layout(&mut b.nodes, &mut b.edges, b.dir);
    assert_eq!(
        a.nodes.iter().map(|c| [c.x, c.y, c.w, c.h]).collect::<Vec<_>>(),
        b.nodes.iter().map(|c| [c.x, c.y, c.w, c.h]).collect::<Vec<_>>()
    );
    assert_eq!(a.edges.iter().map(|e| &e.route).collect::<Vec<_>>(), b.edges.iter().map(|e| &e.route).collect::<Vec<_>>());
}
