//! 1:1 port: Pele tests/layout/compound.test.ts (MIT, kepano 2026).
use std::collections::BTreeSet;
use zebra_mermaid::layout::{
    compound::{cnode, compound_layout, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use zebra_mermaid::util::js;
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
fn edge(src: usize, dst: usize) -> CEdge {
    labelled(src, dst, 0.0, 0.0)
}
fn labelled(src: usize, dst: usize, w: f64, h: f64) -> CEdge {
    CEdge { src, dst, minlen: 1.0, label_w: w, label_h: h, route: vec![], label_x: 0.0, label_y: 0.0 }
}
fn node(w: f64, h: f64) -> CNode {
    cnode(w, h, -1)
}
fn seq(nodes: &mut [CNode]) {
    for (i, n) in nodes.iter_mut().enumerate() {
        n.seq = i as f64;
    }
}
fn span(nodes: &mut [CNode], s: f64) {
    seq(nodes);
    for n in nodes {
        n.span = s;
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 0.5e-5, "{a} != {b}");
}
struct Random(u32);
impl Random {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0 as f64 / 4294967296.0
    }
    fn int(&mut self, n: f64) -> usize {
        (self.next() * n).floor() as usize
    }
}
fn random_graph(seed: u32) -> (Vec<CNode>, Vec<CEdge>, Dir) {
    let mut rnd = Random(seed);
    let mut nodes = vec![];
    let mut groups = vec![];
    let count = rnd.int(5.0);
    for g in 0..count {
        let parent = if !groups.is_empty() && rnd.next() < 0.4 { groups[rnd.int(groups.len() as f64)] as isize } else { -1 };
        let mut n = cnode(0.0, 0.0, parent);
        n.is_group = true;
        n.pad_x = 20.0;
        n.pad_top = 40.0;
        n.pad_bottom = 20.0;
        n.min_w = 60.0;
        n.dir = if rnd.next() < 0.3 { Some(DIRS[rnd.int(4.0)]) } else { None };
        n.seq = (1000 + g) as f64;
        if seed % 2 == 0 {
            n.span = f64::INFINITY;
        }
        groups.push(nodes.len());
        nodes.push(n);
    }
    let leaves = 3 + rnd.int(30.0);
    for i in 0..leaves {
        let parent = if !groups.is_empty() && rnd.next() < 0.5 { groups[rnd.int(groups.len() as f64)] as isize } else { -1 };
        let mut n = cnode((30 + rnd.int(120.0)) as f64, (30 + rnd.int(50.0)) as f64, parent);
        n.seq = i as f64;
        if seed % 2 == 0 {
            n.span = n.w.min(n.h) - 24.0;
        }
        nodes.push(n);
    }
    let mut edges = vec![];
    let count = rnd.int((leaves * 2) as f64);
    for _ in 0..count {
        let has = rnd.next() < 0.3;
        edges.push(labelled(
            rnd.int(nodes.len() as f64),
            rnd.int(nodes.len() as f64),
            if has { (20 + rnd.int(60.0)) as f64 } else { 0.0 },
            if has { 24.0 } else { 0.0 },
        ));
    }
    (nodes, edges, DIRS[rnd.int(4.0)])
}
fn ancestor(nodes: &[CNode], a: usize, b: usize) -> bool {
    let mut p = nodes[b].parent;
    while p >= 0 {
        if p as usize == a {
            return true;
        }
        p = nodes[p as usize].parent;
    }
    false
}
fn border(n: &CNode, x: f64, y: f64) -> bool {
    let dx = (x - n.x).abs();
    let dy = (y - n.y).abs();
    dx <= n.w / 2.0 + 0.5 && dy <= n.h / 2.0 + 0.5 && ((dx - n.w / 2.0).abs() <= 0.5 || (dy - n.h / 2.0).abs() <= 0.5)
}
#[test]
fn random_graphs_well_formed() {
    for seed in 1..=300 {
        let (mut nodes, mut edges, dir) = random_graph(seed);
        let size = compound_layout(&mut nodes, &mut edges, dir, &options());
        assert!(size.width.is_finite() && size.height.is_finite(), "seed {seed}");
        for (i, n) in nodes.iter().enumerate() {
            assert!(n.x.is_finite() && n.y.is_finite(), "seed {seed} node {i}");
            assert!(n.x - n.w / 2.0 >= -0.5);
            assert!(n.y - n.h / 2.0 >= -0.5);
            assert!(n.x + n.w / 2.0 <= size.width + 0.5);
            assert!(n.y + n.h / 2.0 <= size.height + 0.5);
            if n.parent >= 0 {
                let p = &nodes[n.parent as usize];
                assert!((n.x - p.x).abs() + n.w / 2.0 <= p.w / 2.0 + 0.5);
                assert!((n.y - p.y).abs() + n.h / 2.0 <= p.h / 2.0 + 0.5);
            }
        }
        for a in 0..nodes.len() {
            for b in a + 1..nodes.len() {
                if nodes[a].parent != nodes[b].parent {
                    continue;
                }
                let ox = (nodes[a].w + nodes[b].w) / 2.0 - (nodes[a].x - nodes[b].x).abs();
                let oy = (nodes[a].h + nodes[b].h) / 2.0 - (nodes[a].y - nodes[b].y).abs();
                assert!(!(ox > 0.5 && oy > 0.5), "seed {seed} nodes {a},{b}");
            }
        }
        for (i, e) in edges.iter().enumerate() {
            if e.src == e.dst {
                continue;
            }
            assert_eq!(e.route.len() % 3, 0);
            assert!(e.route.len() >= 6);
            assert!(e.route.iter().all(|x| x.is_finite()));
            if ancestor(&nodes, e.src, e.dst) || ancestor(&nodes, e.dst, e.src) {
                continue;
            }
            let last = e.route.len() - 3;
            assert!(border(&nodes[e.src], e.route[0], e.route[1]), "seed {seed} edge {i} start");
            assert!(border(&nodes[e.dst], e.route[last], e.route[last + 1]), "seed {seed} edge {i} end");
        }
    }
}
#[test]
fn deterministic() {
    for seed in 1..=40 {
        let (mut a, mut ea, da) = random_graph(seed);
        let (mut b, mut eb, db) = random_graph(seed);
        compound_layout(&mut a, &mut ea, da, &options());
        compound_layout(&mut b, &mut eb, db, &options());
        assert_eq!(b.iter().map(|n| [n.x, n.y, n.w, n.h]).collect::<Vec<_>>(), a.iter().map(|n| [n.x, n.y, n.w, n.h]).collect::<Vec<_>>());
        assert_eq!(eb.iter().map(|e| &e.route).collect::<Vec<_>>(), ea.iter().map(|e| &e.route).collect::<Vec<_>>());
    }
}
#[test]
fn ranks_follow_direction() {
    for dir in DIRS {
        let mut nodes = vec![node(60.0, 40.0); 3];
        seq(&mut nodes);
        compound_layout(&mut nodes, &mut [edge(0, 1), edge(1, 2)], dir, &options());
        let axis = |n: &CNode| if matches!(dir, Dir::TB | Dir::BT) { n.y } else { n.x };
        let sign = if matches!(dir, Dir::TB | Dir::LR) { 1.0 } else { -1.0 };
        assert!(sign * (axis(&nodes[1]) - axis(&nodes[0])) > 40.0);
        assert!(sign * (axis(&nodes[2]) - axis(&nodes[1])) > 40.0);
    }
}
#[test]
fn long_edge_straight_through_skipped_ranks() {
    let mut nodes = vec![node(60.0, 40.0); 3];
    seq(&mut nodes);
    let mut edges = [edge(0, 1), edge(1, 2), edge(0, 2)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let r = &edges[2].route;
    let xs: BTreeSet<_> = (3..r.len() - 3).step_by(3).map(|i| js::round(r[i]) as i64).collect();
    assert_eq!(xs.len(), 1);
}
#[test]
fn skipping_edge_turns_at_one_end() {
    let mut nodes = vec![node(80.0, 40.0); 8];
    span(&mut nodes, 56.0);
    let mut edges = vec![
        edge(0, 1),
        edge(1, 2),
        edge(2, 3),
        edge(3, 4),
        edge(4, 5),
        edge(5, 6),
        edge(6, 7),
        edge(0, 3),
        edge(0, 7),
        edge(1, 5),
        edge(2, 6),
        edge(6, 1),
        edge(7, 0),
    ];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    for e in &edges[7..] {
        let turns = (3..e.route.len()).step_by(3).filter(|&i| (e.route[i] - e.route[i - 3]).abs() > 0.01).count();
        assert_eq!(turns, 1, "{} to {}", e.src, e.dst);
    }
    let down = &edges[9].route;
    let up = &edges[11].route;
    close(down[3], down[0]);
    close(up[up.len() - 6], up[up.len() - 3]);
}
#[test]
fn centered_without_span() {
    let mut nodes = vec![node(120.0, 40.0), node(60.0, 40.0), node(60.0, 40.0), node(60.0, 40.0)];
    seq(&mut nodes);
    let mut edges = [edge(0, 1), edge(0, 2), edge(0, 3)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    for e in edges {
        close(e.route[0], nodes[0].x);
    }
}
#[test]
fn spreads_in_target_order() {
    for dir in DIRS {
        let mut nodes = vec![node(120.0, 120.0), node(60.0, 60.0), node(60.0, 60.0), node(60.0, 60.0)];
        seq(&mut nodes);
        nodes[0].span = 96.0;
        let mut edges = [edge(0, 1), edge(0, 2), edge(0, 3)];
        compound_layout(&mut nodes, &mut edges, dir, &options());
        let across = if matches!(dir, Dir::TB | Dir::BT) { 0 } else { 1 };
        let starts: Vec<_> = edges.iter().map(|e| e.route[across] - if across == 0 { nodes[0].x } else { nodes[0].y }).collect();
        let targets: Vec<_> = edges.iter().map(|e| if across == 0 { nodes[e.dst].x } else { nodes[e.dst].y }).collect();
        let mut by_start = [0, 1, 2];
        let mut by_target = [0, 1, 2];
        by_start.sort_by(|&a, &b| starts[a].total_cmp(&starts[b]));
        by_target.sort_by(|&a, &b| targets[a].total_cmp(&targets[b]));
        assert_eq!(by_start, by_target);
        close(starts[by_start[0]], -16.0);
        close(starts[by_start[1]], 0.0);
        close(starts[by_start[2]], 16.0);
        for e in edges {
            assert!(border(&nodes[0], e.route[0], e.route[1]));
        }
    }
}
#[test]
fn arriving_and_leaving_edges_apart() {
    let mut nodes = vec![node(60.0, 40.0); 2];
    span(&mut nodes, 36.0);
    let mut edges = [edge(0, 1), edge(1, 0)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let down = &edges[0].route;
    let up = &edges[1].route;
    close((down[down.len() - 3] - up[0]).abs(), 16.0);
    close((down[0] - up[up.len() - 3]).abs(), 16.0);
}
#[test]
fn same_direction_edges_share_when_crowded() {
    let mut nodes = vec![node(60.0, 40.0); 13];
    span(&mut nodes, 36.0);
    let mut edges: Vec<_> = (0..12).map(|i| edge(i, 12)).collect();
    edges.push(edge(12, 0));
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let into: Vec<_> = edges[..12].iter().map(|e| e.route[e.route.len() - 3]).collect();
    let unique: BTreeSet<_> = into.iter().map(|x| js::round(x * 100.0) as i64).collect();
    assert!(unique.len() <= 2);
    let back = &edges[12].route;
    close(back[1], nodes[12].y - 20.0);
    for x in into {
        assert!((x - back[0]).abs() >= 8.0);
    }
}
#[test]
fn sending_and_receiving_ranks_align() {
    let lay = |s| {
        let mut n = vec![node(60.0, 40.0), node(60.0, 120.0), node(60.0, 30.0), node(60.0, 90.0)];
        span(&mut n, s);
        compound_layout(&mut n, &mut [edge(0, 2), edge(1, 2), edge(0, 3), edge(1, 3)], Dir::TB, &options());
        n
    };
    let spread = lay(36.0);
    close(spread[0].y + 20.0, spread[1].y + 60.0);
    close(spread[2].y - 15.0, spread[3].y - 45.0);
    let plain = lay(0.0);
    close(plain[0].y, plain[1].y);
    close(plain[2].y, plain[3].y);
}
fn fan(children: usize, s: f64) -> (Vec<CNode>, Vec<CEdge>, f64) {
    let mut nodes = vec![node(100.0, 40.0); children + 1];
    let mut edges: Vec<_> = (1..=children).map(|i| edge(0, i)).collect();
    span(&mut nodes, s);
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let gap = nodes[1].y - nodes[0].y - 40.0;
    (nodes, edges, gap)
}
#[test]
fn widens_rank_gap_for_far_centered_edges() {
    close(fan(3, 0.0).2, options().rank_sep);
    close(fan(7, 0.0).2, 420.0 / 8.0);
    close(fan(40, 0.0).2, 3.0 * options().rank_sep);
}
#[test]
fn far_side_by_side_edges_get_levels() {
    let (_, few, gap) = fan(3, 76.0);
    close(gap, options().rank_sep);
    for e in few {
        assert_eq!(e.route.len(), 6);
    }
    let (nodes, edges, gap) = fan(7, 76.0);
    let level = |e: &CEdge| {
        assert_eq!(e.route.len(), 12);
        close(e.route[3], e.route[0]);
        close(e.route[7], e.route[4]);
        close(e.route[9], e.route[6]);
        e.route[4]
    };
    close(level(&edges[1]) - level(&edges[0]), 12.0);
    close(level(&edges[2]) - level(&edges[1]), 12.0);
    for i in 0..3 {
        close(level(&edges[6 - i]), level(&edges[i]));
    }
    assert_eq!(edges[3].route.len(), 6);
    close(edges[3].route[3], edges[3].route[0]);
    close(level(&edges[0]) - nodes[0].y - 20.0, 28.0);
    close(gap, 2.0 * 28.0 + 2.0 * 12.0);
}
#[test]
fn crossing_edges_use_plain_curves() {
    let mut nodes = vec![node(80.0, 40.0); 8];
    span(&mut nodes, 56.0);
    let mut edges = vec![];
    for i in 0..4 {
        for j in 4..8 {
            edges.push(edge(i, j));
        }
    }
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    for e in &edges {
        assert_eq!(e.route.len(), 6);
    }
    let widest = edges.iter().map(|e| (e.route[0] - e.route[3]).abs()).fold(f64::NEG_INFINITY, f64::max);
    assert!(widest > 6.0 * options().rank_sep);
    close(nodes[4].y - nodes[0].y - 40.0, 3.0 * options().rank_sep);
}
#[test]
fn labels_move_half_an_arrowhead_toward_tail() {
    let lay = |room| {
        let mut nodes = vec![node(60.0, 40.0); 2];
        span(&mut nodes, 36.0);
        let mut edges = vec![labelled(0, 1, 40.0, 20.0), labelled(0, 1, 50.0, 20.0), edge(0, 1)];
        compound_layout(&mut nodes, &mut edges, Dir::TB, &LayeredOptions { head_room: Some(room), ..options() });
        edges
    };
    let plain = lay(0.0);
    close(plain[0].label_y, (40.0 + 108.0) / 2.0);
    let arrows = lay(8.0);
    let (first, second, bare) = (&arrows[0], &arrows[1], &arrows[2]);
    close(first.label_y, (40.0 + 108.0 - 8.0) / 2.0);
    close(second.label_y, first.label_y);
    close(first.route[7] - first.route[4], 20.0);
    close(bare.route[7] - bare.route[4], 10.0);
    close((bare.route[4] + bare.route[7]) / 2.0, first.label_y);
}
#[test]
fn far_edge_gets_its_own_level() {
    let mut nodes = vec![node(80.0, 40.0); 5];
    span(&mut nodes, 56.0);
    let mut edges = [edge(0, 1), edge(0, 2), edge(0, 3), edge(1, 4), edge(2, 4), edge(3, 4), edge(4, 0)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let back = &edges[6].route;
    let n = back.len();
    let level = back[n - 5];
    close(back[n - 8], level);
    close(back[n - 6], back[n - 3]);
    assert!((back[n - 9] - back[n - 6]).abs() > 2.0 * options().rank_sep);
    let down = &edges[2].route;
    assert!((down[0] - back[n - 3]).abs() < 20.0);
    assert_eq!(down.len(), 9);
    close(down[3], down[0]);
    assert!(down[4] > level - 0.01);
}
#[test]
fn moves_end_to_make_edge_straight() {
    let mut nodes = vec![node(400.0, 40.0), node(40.0, 40.0), node(40.0, 40.0), node(40.0, 40.0), node(40.0, 40.0)];
    seq(&mut nodes);
    for n in &mut nodes {
        n.span = n.w - 24.0;
    }
    let mut edges = [edge(0, 1), edge(0, 2), edge(0, 3), edge(0, 4)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    for e in edges {
        close(e.route[0], e.route[e.route.len() - 3]);
    }
    let mut two = vec![node(200.0, 40.0), node(40.0, 40.0)];
    seq(&mut two);
    for n in &mut two {
        n.span = n.w - 24.0;
    }
    let mut pair = [edge(0, 1), edge(1, 0)];
    compound_layout(&mut two, &mut pair, Dir::TB, &options());
    close((pair[0].route[0] - pair[1].route[pair[1].route.len() - 3]).abs(), 16.0);
}
#[test]
fn compresses_before_sharing_positions() {
    let mut nodes = vec![node(60.0, 40.0); 8];
    let mut edges: Vec<_> = (1..=7).map(|i| edge(0, i)).collect();
    seq(&mut nodes);
    nodes[0].span = 36.0;
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let mut starts: Vec<_> = edges.iter().map(|e| e.route[0]).collect();
    starts.sort_by(f64::total_cmp);
    for i in 1..starts.len() {
        close(starts[i] - starts[i - 1], 6.0);
    }
}
#[test]
fn curves_across_ranks_created_only_by_twins() {
    let lay = |twin| {
        let mut nodes = vec![node(60.0, 40.0); 4];
        span(&mut nodes, 36.0);
        let mut edges = vec![edge(0, 1), edge(0, 2), edge(0, 3)];
        if twin {
            edges.push(edge(0, 3));
        }
        compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
        edges
    };
    let plain = lay(false);
    let twins = lay(true);
    for e in &twins[..2] {
        assert_eq!(e.route.len(), 6);
    }
    close(twins[0].route[1], plain[0].route[1]);
    close(twins[0].route[4], plain[0].route[4] + 2.0 * 28.0 + 12.0 - options().rank_sep);
    let (one, other) = (&twins[2].route, &twins[3].route);
    assert!((one[0] - other[0]).abs() >= 8.0);
    assert!((one[one.len() - 3] - other[other.len() - 3]).abs() >= 8.0);
    let at = |r: &[f64]| if (r[3] - r[0]).abs() < 0.01 { r[4] } else { r[1] };
    assert!((at(one) - at(other)).abs() >= 10.0);
}
#[test]
fn separates_twins_with_fixed_ends() {
    let mut nodes = vec![node(60.0, 40.0); 2];
    seq(&mut nodes);
    let mut edges = [edge(0, 1), edge(0, 1)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    let (a, b) = (&edges[0].route, &edges[1].route);
    assert_eq!(a.len(), 9);
    assert_eq!(b.len(), 9);
    close(a[0], b[0]);
    close((a[3] - b[3]).abs(), options().edge_sep);
}
#[test]
fn routes_on_distinct_right_angle_tracks() {
    #[derive(Clone, Copy)]
    struct Run {
        y: f64,
        lo: f64,
        hi: f64,
        from: f64,
        to: f64,
    }
    for seed in 1..=120 {
        let mut rnd = Random(seed * 7919);
        let mut nodes = vec![];
        let count = 3 + rnd.int(14.0);
        for i in 0..count {
            let mut n = node((40 + rnd.int(120.0)) as f64, (30 + rnd.int(60.0)) as f64);
            n.seq = i as f64;
            n.span = n.w - 24.0;
            nodes.push(n);
        }
        let mut edges = vec![];
        let wanted = rnd.int(count as f64 * 2.5);
        for _ in 0..wanted {
            let src = rnd.int(count as f64);
            let dst = rnd.int(count as f64);
            if src != dst {
                edges.push(labelled(src, dst, if rnd.next() < 0.2 { (30 + rnd.int(40.0)) as f64 } else { 0.0 }, 20.0));
            }
        }
        compound_layout(&mut nodes, &mut edges, Dir::TB, &LayeredOptions { tracks: true, ..options() });
        let mut runs = vec![];
        for (ei, e) in edges.iter().enumerate() {
            let r = &e.route;
            assert!(border(&nodes[e.src], r[0], r[1]), "seed {seed} edge {ei} start");
            assert!(border(&nodes[e.dst], r[r.len() - 3], r[r.len() - 2]), "seed {seed} edge {ei} end");
            for k in (3..r.len()).step_by(3) {
                let across = (r[k + 1] - r[k - 2]).abs() < 0.01;
                let down = (r[k] - r[k - 3]).abs() < 0.5;
                assert!(across || down, "seed {seed} edge {ei} right angles");
                if !across || down {
                    continue;
                }
                let before = if k >= 6 { r[k - 5] } else { r[k - 2] };
                let first = before < r[k - 2] - 0.01 || (k + 4 < r.len() && r[k + 4] > r[k + 1] + 0.01);
                runs.push(Run {
                    y: r[k + 1],
                    lo: r[k].min(r[k - 3]),
                    hi: r[k].max(r[k - 3]),
                    from: if first { r[k - 3] } else { r[k] },
                    to: if first { r[k] } else { r[k - 3] },
                });
            }
        }
        for a in 0..runs.len() {
            for b in a + 1..runs.len() {
                if (runs[a].y - runs[b].y).abs() > 0.01 {
                    continue;
                }
                let overlap = runs[a].lo < runs[b].hi - 0.5 && runs[b].lo < runs[a].hi - 0.5;
                let mates = (runs[a].from - runs[b].from).abs() < 0.5 || (runs[a].to - runs[b].to).abs() < 0.5;
                assert!(!(overlap && !mates), "seed {seed} runs {a},{b} overlap");
            }
        }
        for r in runs {
            for n in &nodes {
                let inside = (r.y - n.y).abs() < n.h / 2.0 - 0.5 && r.lo < n.x + n.w / 2.0 - 0.5 && r.hi > n.x - n.w / 2.0 + 0.5;
                assert!(!inside, "seed {seed} run crosses node");
            }
        }
    }
}
#[test]
fn nests_fan_tracks_without_crossing() {
    let mut nodes = vec![node(120.0, 40.0), node(60.0, 40.0), node(60.0, 40.0), node(60.0, 40.0), node(60.0, 40.0)];
    seq(&mut nodes);
    nodes[0].span = 96.0;
    let mut edges = [edge(0, 1), edge(0, 2), edge(0, 3), edge(0, 4)];
    compound_layout(&mut nodes, &mut edges, Dir::TB, &LayeredOptions { tracks: true, ..options() });
    let mut left: Vec<_> = edges.iter().filter(|e| nodes[e.dst].x < nodes[0].x).collect();
    left.sort_by(|a, b| nodes[a.dst].x.total_cmp(&nodes[b.dst].x));
    let mut right: Vec<_> = edges.iter().filter(|e| nodes[e.dst].x > nodes[0].x).collect();
    right.sort_by(|a, b| nodes[b.dst].x.total_cmp(&nodes[a.dst].x));
    for side in [left, right] {
        for i in 1..side.len() {
            assert!(side[i - 1].route[4] < side[i].route[4]);
        }
    }
}
#[test]
fn survives_mutually_containing_groups() {
    let mut a = cnode(0.0, 0.0, 1);
    let mut b = cnode(0.0, 0.0, 0);
    a.is_group = true;
    b.is_group = true;
    let mut nodes = [a, b, cnode(40.0, 40.0, 0), node(40.0, 40.0)];
    let mut edges = [edge(2, 3)];
    let size = compound_layout(&mut nodes, &mut edges, Dir::TB, &options());
    assert!(size.width.is_finite());
    assert!(edges[0].route.len() >= 6);
}
