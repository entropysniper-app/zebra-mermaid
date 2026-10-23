//! Layout of a graph whose nodes may be nested in groups. Ported from Pele
//! `src/layout/compound.ts`.
//!
//! Each group is laid out on its own, in its own direction, and then placed in its parent as a
//! single box. An edge that crosses a group's border passes through a port on the side facing
//! its other end. The nesting is walked level by level from a queue, never by recursion, so a
//! deeply nested input cannot exhaust the stack.

pub use super::crossings::{route_crossings, style_of, Style};
use super::layered::{layered, ledge, lnode, orient, rank, Kind, LEdge, LNode, LayeredGroups, LayeredOptions, LayeredResult};
use crate::util::hash::IndexedMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    TB,
    BT,
    LR,
    RL,
}

impl Dir {
    pub fn as_str(self) -> &'static str {
        match self {
            Dir::TB => "TB",
            Dir::BT => "BT",
            Dir::LR => "LR",
            Dir::RL => "RL",
        }
    }
}

/// The direction a diagram names, or top to bottom for anything else.
pub fn direction(dir: Option<&str>) -> Dir {
    match dir {
        Some("BT") => Dir::BT,
        Some("LR") => Dir::LR,
        Some("RL") => Dir::RL,
        _ => Dir::TB,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CNode {
    pub w: f64,
    pub h: f64,
    /// The index of the group that holds the node, or -1.
    pub parent: isize,
    pub is_group: bool,
    pub dir: Option<Dir>,
    pub pad_x: f64,
    pub pad_top: f64,
    pub pad_bottom: f64,
    pub min_w: f64,
    /// Declaration order; a group takes the smallest value among its members.
    pub seq: f64,
    /// How much of a side its edges may spread along, measured across the flow; 0 keeps them
    /// at its middle.
    pub span: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CEdge {
    pub src: usize,
    pub dst: usize,
    pub minlen: f64,
    pub label_w: f64,
    pub label_h: f64,
    /// x, y, axis triples; axis is 0 where the edge runs vertically into the point, 1
    /// horizontally.
    pub route: Vec<f64>,
    pub label_x: f64,
    pub label_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompoundResult {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}

struct PortEdge {
    e: usize,
    inner: usize,
    out: bool,
    side: Side,
    hint: f64,
    px: f64,
    py: f64,
    route: Vec<f64>,
}

struct LevelEdge {
    e: usize,
    a: usize,
    b: usize,
}

struct Level {
    id: isize,
    dir: Dir,
    items: Vec<usize>,
    edges: Vec<LevelEdge>,
    ports: Vec<PortEdge>,
    /// Edge index * 2 (+1 for the head end) → index into `ports`.
    port_of: IndexedMap<usize, usize>,
    routes: IndexedMap<usize, Vec<f64>>,
    x: f64,
    y: f64,
}

const MAX_PORTS: usize = 40000;

pub fn shift_layout(nodes: &mut [CNode], edges: &mut [CEdge], ox: f64, oy: f64) {
    for c in nodes.iter_mut() {
        c.x += ox;
        c.y += oy;
    }
    for e in edges.iter_mut() {
        let mut k = 0;
        while k < e.route.len() {
            e.route[k] += ox;
            e.route[k + 1] += oy;
            k += 3;
        }
        e.label_x += ox;
        e.label_y += oy;
    }
}

/// `cnode(w, h, parent)`; Pele's default parent is -1.
pub fn cnode(w: f64, h: f64, parent: isize) -> CNode {
    CNode {
        w,
        h,
        parent,
        is_group: false,
        dir: None,
        pad_x: 0.0,
        pad_top: 0.0,
        pad_bottom: 0.0,
        min_w: 0.0,
        seq: 0.0,
        span: 0.0,
        x: 0.0,
        y: 0.0,
    }
}

fn start_side(dir: Dir) -> Side {
    match dir {
        Dir::TB => Side::Top,
        Dir::BT => Side::Bottom,
        Dir::LR => Side::Left,
        Dir::RL => Side::Right,
    }
}

fn opposite(side: Side) -> Side {
    match ((side as u8) + 2) & 3 {
        0 => Side::Top,
        1 => Side::Right,
        2 => Side::Bottom,
        _ => Side::Left,
    }
}

/// The level of group `id` (-1 for the root).
fn level_index(level_of: &[usize], id: isize) -> usize {
    level_of[(id + 1) as usize]
}

/// Lays out a graph whose nodes may be nested in groups. Each group is laid out on its own, in
/// its own direction, and then placed in its parent as a single box. An edge that crosses a
/// group's border passes through a port on the side facing its other end.
pub fn compound_layout(nodes: &mut [CNode], edges: &mut [CEdge], root_dir: Dir, opt: &LayeredOptions) -> CompoundResult {
    let n = nodes.len();
    // Index of each group's level in `order`, by group index + 1 (0 is the root).
    let mut level_of: Vec<usize> = vec![usize::MAX; n + 1];
    let mut order: Vec<Level> = Vec::new();
    let make_level = |id: isize, dir: Dir| Level {
        id,
        dir,
        items: Vec::new(),
        edges: Vec::new(),
        ports: Vec::new(),
        port_of: IndexedMap::default(),
        routes: IndexedMap::default(),
        x: 0.0,
        y: 0.0,
    };
    let mut root = make_level(-1, root_dir);

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut root_items: Vec<usize> = Vec::new();
    for i in 0..n {
        let p = nodes[i].parent;
        if p >= 0 && (p as usize) < n && p as usize != i && nodes[p as usize].is_group {
            children[p as usize].push(i);
        } else {
            nodes[i].parent = -1;
            root_items.push(i);
        }
    }
    root.items = root_items;
    level_of[0] = 0;
    order.push(root);
    let mut depth = vec![0i64; n];
    let mut placed = vec![false; n];
    let descend = |order: &mut Vec<Level>,
                   level_of: &mut Vec<usize>,
                   depth: &mut Vec<i64>,
                   placed: &mut Vec<bool>,
                   children: &mut Vec<Vec<usize>>,
                   nodes: &[CNode],
                   from: usize| {
        let mut q = from;
        while q < order.len() {
            let id = order[q].id;
            let dir = order[q].dir;
            // The items are read by index: a group among them adds a level to `order`.
            for k in 0..order[q].items.len() {
                let item = order[q].items[k];
                placed[item] = true;
                depth[item] = if id < 0 { 0 } else { depth[id as usize] + 1 };
                if nodes[item].is_group {
                    let mut child = make_level(item as isize, nodes[item].dir.unwrap_or(dir));
                    child.items = std::mem::take(&mut children[item]);
                    level_of[item + 1] = order.len();
                    order.push(child);
                }
            }
            q += 1;
        }
    };
    descend(&mut order, &mut level_of, &mut depth, &mut placed, &mut children, nodes, 0);
    // Groups that contain each other are unreachable from the root; lift them out.
    for i in 0..n {
        if placed[i] {
            continue;
        }
        let former = nodes[i].parent as usize;
        children[former].retain(|&c| c != i);
        nodes[i].parent = -1;
        order[0].items.push(i);
        placed[i] = true;
        depth[i] = 0;
        if nodes[i].is_group {
            let from = order.len();
            let mut child = make_level(i as isize, nodes[i].dir.unwrap_or(root_dir));
            child.items = std::mem::take(&mut children[i]);
            level_of[i + 1] = order.len();
            order.push(child);
            descend(&mut order, &mut level_of, &mut depth, &mut placed, &mut children, nodes, from);
        }
    }

    for q in (1..order.len()).rev() {
        let mut seq = f64::INFINITY;
        for &item in &order[q].items {
            if nodes[item].seq < seq {
                seq = nodes[item].seq;
            }
        }
        if seq != f64::INFINITY {
            nodes[order[q].id as usize].seq = seq;
        }
    }
    for level in order.iter_mut() {
        level.items.sort_by(|&a, &b| {
            let d = nodes[a].seq - nodes[b].seq;
            if d < 0.0 {
                std::cmp::Ordering::Less
            } else if d > 0.0 {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(&b)
            }
        });
    }

    let parent_of = |nodes: &[CNode], i: usize| nodes[i].parent;
    // Every group border an edge crosses costs a port. Past the budget, an edge is drawn
    // between the groups that hold its ends, at the level they share.
    let mut ports = 0usize;
    for e in edges.iter_mut() {
        let mut a = e.src;
        let mut b = e.dst;
        let mut crossed = 0usize;
        while depth[a] > depth[b] {
            a = parent_of(nodes, a) as usize;
            crossed += 1;
        }
        while depth[b] > depth[a] {
            b = parent_of(nodes, b) as usize;
            crossed += 1;
        }
        while a != b && nodes[a].parent != nodes[b].parent {
            a = parent_of(nodes, a) as usize;
            b = parent_of(nodes, b) as usize;
            crossed += 2;
        }
        if a == b {
            continue;
        }
        if ports + crossed > MAX_PORTS {
            e.src = a;
            e.dst = b;
        } else {
            ports += crossed;
        }
    }

    let mut loops: Vec<usize> = Vec::new();
    let mut stray: Vec<usize> = Vec::new();
    for ei in 0..edges.len() {
        let e = &mut edges[ei];
        e.route = Vec::new();
        if e.src == e.dst {
            loops.push(ei);
            continue;
        }
        let mut a = e.src;
        let mut b = e.dst;
        let mut a_path: Vec<usize> = Vec::new();
        let mut b_path: Vec<usize> = Vec::new();
        while depth[a] > depth[b] {
            a_path.push(a);
            a = nodes[a].parent as usize;
        }
        while depth[b] > depth[a] {
            b_path.push(b);
            b = nodes[b].parent as usize;
        }
        if a == b {
            stray.push(ei);
            continue;
        }
        while nodes[a].parent != nodes[b].parent {
            a_path.push(a);
            b_path.push(b);
            a = nodes[a].parent as usize;
            b = nodes[b].parent as usize;
        }
        let li = level_index(&level_of, nodes[a].parent);
        order[li].edges.push(LevelEdge { e: ei, a, b });
        for (path, out) in [(&a_path, true), (&b_path, false)] {
            for &inner in path {
                let li = level_index(&level_of, nodes[inner].parent);
                let level = &mut order[li];
                level.ports.push(PortEdge { e: ei, inner, out, side: Side::Top, hint: f64::NAN, px: 0.0, py: 0.0, route: Vec::new() });
                let at = level.ports.len() - 1;
                level.port_of.insert(ei * 2 + if out { 0 } else { 1 }, at);
            }
        }
    }

    let ordered = ports > 1 && ports <= 400 && edges.len() <= 500;
    let mut key = vec![f64::NAN; n];
    let flat = if order.len() > 1 && ordered && order.iter().all(|level| level.dir == root_dir) {
        let vertical = matches!(root_dir, Dir::TB | Dir::BT);
        let mut leaf = vec![-1isize; n];
        let mut group = vec![-1isize; n];
        let mut lnodes = Vec::new();
        let mut of = Vec::new();
        let mut parent = Vec::new();
        for i in 0..n {
            if nodes[i].is_group {
                group[i] = parent.len() as isize;
                parent.push(-1);
            }
        }
        for i in 0..n {
            let p = nodes[i].parent;
            if nodes[i].is_group {
                parent[group[i] as usize] = if p >= 0 { group[p as usize] } else { -1 };
            } else {
                leaf[i] = lnodes.len() as isize;
                lnodes.push(lnode(
                    if vertical { nodes[i].w } else { nodes[i].h },
                    if vertical { nodes[i].h } else { nodes[i].w },
                    Kind::Node,
                ));
                of.push(if p >= 0 { group[p as usize] } else { -1 });
            }
        }
        let mut ledges = Vec::new();
        let mut edge_of = vec![-1isize; edges.len()];
        for (ei, e) in edges.iter().enumerate() {
            if e.src == e.dst || leaf[e.src] < 0 || leaf[e.dst] < 0 {
                continue;
            }
            edge_of[ei] = ledges.len() as isize;
            ledges.push(ledge(
                leaf[e.src] as usize,
                leaf[e.dst] as usize,
                e.minlen,
                if vertical { e.label_w } else { e.label_h },
                if vertical { e.label_h } else { e.label_w },
            ));
        }
        let flat_layout = layered(
            &mut lnodes,
            &mut ledges,
            &LayeredOptions { groups: Some(LayeredGroups { of, parent }), order_only: true, ..opt.clone() },
        );
        let crossings = flat_layout.crossings.unwrap_or(0).min(flat_layout.declared.unwrap_or(0));
        let mut sum = vec![0.0; n];
        let mut count = vec![0; n];
        let mut last = vec![-1; n];
        for i in 0..n {
            if leaf[i] < 0 {
                continue;
            }
            let ln = &lnodes[leaf[i] as usize];
            key[i] = ln.x;
            let mut p = nodes[i].parent;
            while p >= 0 {
                let q = p as usize;
                sum[q] += key[i];
                count[q] += 1;
                last[q] = last[q].max(ln.rank);
                p = nodes[q].parent;
            }
        }
        for i in 0..n {
            if count[i] > 0 {
                key[i] = sum[i] / count[i] as f64;
            }
        }
        Some(Flat { edge_of, edges: ledges, last, crossings })
    } else {
        None
    };
    let mut laid: Vec<Vec<Laid>> = (0..order.len()).map(|_| Vec::new()).collect();
    let style = style_of(opt);
    let mut keyed = flat.is_some();
    let mut total = place(&mut order, &level_of, nodes, edges, opt, keyed.then_some(key.as_slice()), flat.as_ref(), ordered, &mut laid);
    let mut crossings = None;
    if let Some(flat) = &flat {
        let crossed = route_crossings(nodes, edges, style);
        crossings = Some(crossed);
        if crossed as f64 > flat.crossings as f64 * 1.5 + 4.0 {
            let first = snapshot(nodes, edges, &order, total);
            keyed = false;
            total = place(&mut order, &level_of, nodes, edges, opt, None, Some(flat), ordered, &mut laid);
            let own = route_crossings(nodes, edges, style);
            if own >= crossed {
                keyed = true;
                total = restore(first, nodes, edges, &mut order);
            } else {
                crossings = Some(own);
            }
        }
    }
    if keyed == flat.is_some() && ordered {
        let mut again = false;
        for level in &mut order {
            if level.id < 0 {
                continue;
            }
            let b = &nodes[level.id as usize];
            let vertical = matches!(level.dir, Dir::TB | Dir::BT);
            let end = opposite(start_side(level.dir));
            let mut seen = Vec::new();
            for port in &mut level.ports {
                if port.side != end {
                    continue;
                }
                let route = &edges[port.e].route;
                let (bx, by) = (level.x + port.px, level.y + port.py);
                let mut at = None;
                for k in (0..route.len()).step_by(3) {
                    if (route[k] - bx).abs() < 0.01 && (route[k + 1] - by).abs() < 0.01 {
                        at = Some(k);
                        if !port.out {
                            break;
                        }
                    }
                }
                let Some(at) = at else {
                    continue;
                };
                let a = if vertical { 0 } else { 1 };
                let step = if port.out { 3 } else { -3 };
                let mut next = at as isize + step;
                while next >= 0 && (next as usize) < route.len() && (route[next as usize + a] - route[at + a]).abs() < 0.5 {
                    next += step;
                }
                if next < 0 || next as usize >= route.len() {
                    continue;
                }
                port.hint = route[next as usize + a] - if vertical { b.x - b.w / 2.0 } else { b.y - b.h / 2.0 };
                seen.push((if vertical { port.px } else { port.py }, port.hint));
            }
            seen.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap_or(std::cmp::Ordering::Equal));
            if seen.windows(2).any(|w| w[0].1 > w[1].1) {
                again = true;
            }
        }
        if again {
            let before = crossings.unwrap_or_else(|| route_crossings(nodes, edges, style));
            if before > 0 {
                let first = snapshot(nodes, edges, &order, total);
                total = place(&mut order, &level_of, nodes, edges, opt, keyed.then_some(key.as_slice()), flat.as_ref(), ordered, &mut laid);
                if route_crossings(nodes, edges, style) >= before {
                    total = restore(first, nodes, edges, &mut order);
                }
            }
        }
    }

    for &ei in &stray {
        let (ax, ay) = (nodes[edges[ei].src].x, nodes[edges[ei].src].y);
        let (bx, by) = (nodes[edges[ei].dst].x, nodes[edges[ei].dst].y);
        let e = &mut edges[ei];
        e.route = vec![ax, ay, 0.0, bx, by, 0.0];
        e.label_x = (ax + bx) / 2.0;
        e.label_y = (ay + by) / 2.0;
    }
    for &ei in &loops {
        let node = &nodes[edges[ei].src];
        let (x, y, w) = (node.x, node.y, node.w);
        let e = &mut edges[ei];
        e.label_x = x + w / 2.0 + 28.0 + e.label_w / 2.0;
        e.label_y = y;
    }

    total
}

/// The flat graph of one level: its items, its edges, and a port node for every edge that
/// passes through the level's border at its start or end side. `sized` is false for the first,
/// top-down pass, which only ranks.
fn build(
    order: &[Level],
    q: usize,
    level_of: &[usize],
    nodes: &[CNode],
    edges: &[CEdge],
    sized: bool,
    key: Option<&[f64]>,
    far: Option<&Flat>,
) -> (Vec<LNode>, Vec<LEdge>, Vec<isize>) {
    let level = &order[q];
    let vertical = level.dir == Dir::TB || level.dir == Dir::BT;
    let start = start_side(level.dir);
    let mut local: IndexedMap<usize, usize> = IndexedMap::with_capacity_and_hasher(level.items.len(), Default::default());
    let mut lnodes: Vec<LNode> = Vec::with_capacity(level.items.len() + level.ports.len());
    let mut ledges: Vec<LEdge> = Vec::with_capacity(level.edges.len() + level.ports.len());
    for (i, &item) in level.items.iter().enumerate() {
        local.insert(item, i);
        let node = &nodes[item];
        let mut ln = if sized {
            lnode(if vertical { node.w } else { node.h }, if vertical { node.h } else { node.w }, Kind::Node)
        } else {
            lnode(0.0, 0.0, Kind::Node)
        };
        ln.span = if node.is_group { crate::util::js::min(node.span, ln.w - 2.0 * node.pad_x) } else { node.span };
        ln.key = key.map_or(f64::NAN, |key| key[item]);
        lnodes.push(ln);
    }
    let endpoint_below = |item: usize, e: &CEdge, out: bool| -> bool { (if out { e.src } else { e.dst }) != item };
    let held = |item: usize, ei: usize, out: bool| -> bool { nodes[item].is_group && endpoint_below(item, &edges[ei], out) };
    let offset = |item: usize, ei: usize, out: bool| -> f64 {
        if !sized || !nodes[item].is_group || !endpoint_below(item, &edges[ei], out) {
            return 0.0;
        }
        let inner = &order[level_index(level_of, item as isize)];
        let port = &inner.ports[inner.port_of[&(ei * 2 + if out { 0 } else { 1 })]];
        if vertical {
            port.px - nodes[item].w / 2.0
        } else {
            port.py - nodes[item].h / 2.0
        }
    };
    for le in &level.edges {
        let e = &edges[le.e];
        let mut edge = ledge(local[&le.a], local[&le.b], e.minlen, 0.0, 0.0);
        if sized {
            edge.label_w = if vertical { e.label_w } else { e.label_h };
            edge.label_h = if vertical { e.label_h } else { e.label_w };
            edge.tail_dx = offset(le.a, le.e, true);
            edge.head_dx = offset(le.b, le.e, false);
            edge.tail_fixed = held(le.a, le.e, true);
            edge.head_fixed = held(le.b, le.e, false);
        }
        ledges.push(edge);
    }
    let mut dummies = vec![-1isize; level.ports.len()];
    for (i, port) in level.ports.iter().enumerate() {
        if port.side == start || port.side == opposite(start) {
            let d = lnodes.len();
            let mut ln = lnode(0.0, 0.0, if port.side == start { Kind::StartPort } else { Kind::EndPort });
            ln.key = key.map_or(f64::NAN, |key| key[if port.out { edges[port.e].dst } else { edges[port.e].src }]);
            ln.hint = if port.hint.is_nan() && key.is_some() {
                far.map_or(f64::NAN, |flat| flat.hint(level, port, edges, key.unwrap()))
            } else {
                port.hint
            };
            lnodes.push(ln);
            dummies[i] = d as isize;
            let mut edge = if port.out { ledge(local[&port.inner], d, 1.0, 0.0, 0.0) } else { ledge(d, local[&port.inner], 1.0, 0.0, 0.0) };
            if port.out {
                edge.tail_dx = offset(port.inner, port.e, true);
                edge.tail_fixed = held(port.inner, port.e, true);
            } else {
                edge.head_dx = offset(port.inner, port.e, false);
                edge.head_fixed = held(port.inner, port.e, false);
            }
            ledges.push(edge);
        } else {
            lnodes[local[&port.inner]].pin = if port.side == Side::Top || port.side == Side::Left { -1.0 } else { 1.0 };
        }
    }
    (lnodes, ledges, dummies)
}

fn place(
    order: &mut [Level],
    level_of: &[usize],
    nodes: &mut [CNode],
    edges: &mut [CEdge],
    opt: &LayeredOptions,
    key: Option<&[f64]>,
    far: Option<&Flat>,
    ordered: bool,
    laid: &mut [Vec<Laid>],
) -> CompoundResult {
    let endpoint_below = |item: usize, e: &CEdge, out: bool| -> bool { (if out { e.src } else { e.dst }) != item };
    // Top-down: rank each level to learn which side of a group every crossing edge leaves from.
    for q in 0..order.len() {
        for pi in 0..order[q].ports.len() {
            let (inner, pe, out, side) = {
                let port = &order[q].ports[pi];
                (port.inner, port.e, port.out, port.side)
            };
            if nodes[inner].is_group && endpoint_below(inner, &edges[pe], out) {
                let li = level_index(&level_of, inner as isize);
                let at = order[li].port_of[&(pe * 2 + if out { 0 } else { 1 })];
                order[li].ports[at].side = side;
            }
        }
        if order[q].edges.is_empty() {
            continue;
        }
        let mut needed = false;
        for le in &order[q].edges {
            let e = &edges[le.e];
            if (nodes[le.a].is_group && e.src != le.a) || (nodes[le.b].is_group && e.dst != le.b) {
                needed = true;
            }
        }
        if !needed {
            continue;
        }
        let (mut lnodes, mut ledges, _) = build(order, q, level_of, nodes, edges, false, key, far);
        orient(&lnodes, &mut ledges);
        rank(&mut lnodes, &ledges, 1);
        let start = start_side(order[q].dir);
        for i in 0..order[q].edges.len() {
            let (le_e, le_a, le_b) = {
                let le = &order[q].edges[i];
                (le.e, le.a, le.b)
            };
            let a_above = !ledges[i].reversed;
            if nodes[le_a].is_group && edges[le_e].src != le_a {
                let li = level_index(&level_of, le_a as isize);
                let at = order[li].port_of[&(le_e * 2)];
                order[li].ports[at].side = if a_above { opposite(start) } else { start };
            }
            if nodes[le_b].is_group && edges[le_e].dst != le_b {
                let li = level_index(&level_of, le_b as isize);
                let at = order[li].port_of[&(le_e * 2 + 1)];
                order[li].ports[at].side = if a_above { start } else { opposite(start) };
            }
        }
    }

    // Bottom-up: lay out each level now that the sizes of its groups are known.
    let mut total = CompoundResult { width: 0.0, height: 0.0 };
    for q in (0..order.len()).rev() {
        let id = order[q].id;
        let dir = order[q].dir;
        let vertical = dir == Dir::TB || dir == Dir::BT;
        let flip = dir == Dir::BT || dir == Dir::RL;
        let (mut lnodes, mut ledges, dummies) = build(order, q, level_of, nodes, edges, true, key, far);
        let size = if ordered {
            let mut given = Vec::new();
            let mut add = |x: f64| given.push(if x.is_nan() { f64::NAN.to_bits() } else { x.to_bits() });
            for ln in &lnodes {
                for v in [ln.w, ln.h, ln.kind as u8 as f64, ln.pin, ln.hint, ln.key, ln.span] {
                    add(v);
                }
            }
            for le in &ledges {
                for v in [
                    le.tail as f64,
                    le.head as f64,
                    le.minlen,
                    le.label_w,
                    le.label_h,
                    le.tail_dx,
                    le.head_dx,
                    le.tail_fixed as u8 as f64,
                    le.head_fixed as u8 as f64,
                ] {
                    add(v);
                }
            }
            if let Some(past) = laid[q].iter().find(|p| p.given == given) {
                lnodes.clone_from(&past.nodes);
                ledges.clone_from(&past.edges);
                past.size
            } else {
                let size = layered(&mut lnodes, &mut ledges, opt);
                laid[q].push(Laid { given, nodes: lnodes.clone(), edges: ledges.clone(), size });
                size
            }
        } else {
            layered(&mut lnodes, &mut ledges, opt)
        };
        let content_w = if vertical { size.width } else { size.height };
        let content_h = if vertical { size.height } else { size.width };
        let group = if id >= 0 { Some(&nodes[id as usize]) } else { None };
        let pad_x = group.map_or(0.0, |g| g.pad_x);
        let pad_top = group.map_or(0.0, |g| g.pad_top);
        let box_w = crate::util::js::max(content_w + 2.0 * pad_x, group.map_or(0.0, |g| g.min_w));
        let box_h = content_h + pad_top + group.map_or(0.0, |g| g.pad_bottom);
        let off_x = (box_w - content_w) / 2.0;
        let off_y = pad_top;
        let tx = |lx: f64, ly: f64| -> f64 {
            off_x
                + if vertical {
                    lx
                } else if flip {
                    size.height - ly
                } else {
                    ly
                }
        };
        let ty = |lx: f64, ly: f64| -> f64 {
            off_y
                + if vertical {
                    if flip {
                        size.height - ly
                    } else {
                        ly
                    }
                } else {
                    lx
                }
        };
        let axis = if vertical { 0.0 } else { 1.0 };
        let convert = |points: &[f64]| -> Vec<f64> {
            let mut out = Vec::with_capacity(points.len() / 2 * 3);
            let mut i = 0;
            while i < points.len() {
                out.push(tx(points[i], points[i + 1]));
                out.push(ty(points[i], points[i + 1]));
                out.push(axis);
                i += 2;
            }
            out
        };

        for (i, &item) in order[q].items.iter().enumerate() {
            nodes[item].x = tx(lnodes[i].x, lnodes[i].y);
            nodes[item].y = ty(lnodes[i].x, lnodes[i].y);
        }
        for i in 0..order[q].edges.len() {
            let le_e = order[q].edges[i].e;
            let edge = &ledges[i];
            order[q].routes.insert(le_e, convert(&edge.points));
            let e = &mut edges[le_e];
            if e.label_w > 0.0 {
                e.label_x = tx(edge.label_x, edge.label_y);
                e.label_y = ty(edge.label_x, edge.label_y);
            }
        }
        let mut k = order[q].edges.len();
        for i in 0..order[q].ports.len() {
            let (inner_i, pe, out, side) = {
                let port = &order[q].ports[i];
                (port.inner, port.e, port.out, port.side)
            };
            if dummies[i] >= 0 {
                let mut route = convert(&ledges[k].points);
                k += 1;
                let at = if out { route.len() - 3 } else { 0 };
                let mut bx = route[at];
                let mut by = route[at + 1];
                match side {
                    Side::Top => by = 0.0,
                    Side::Bottom => by = box_h,
                    Side::Left => bx = 0.0,
                    Side::Right => bx = box_w,
                }
                if out {
                    route.extend_from_slice(&[bx, by, axis]);
                } else {
                    route.splice(0..0, [bx, by, axis]);
                }
                let port = &mut order[q].ports[i];
                port.px = bx;
                port.py = by;
                port.route = route;
                continue;
            }
            let (item_x, item_y, item_w, item_h, item_group) = {
                let item = &nodes[inner_i];
                (item.x, item.y, item.w, item.h, item.is_group)
            };
            let mut ix = item_x;
            let mut iy = item_y;
            if item_group && endpoint_below(inner_i, &edges[pe], out) {
                let li = level_index(&level_of, inner_i as isize);
                let at = order[li].port_of[&(pe * 2 + if out { 0 } else { 1 })];
                let inner = &order[li].ports[at];
                ix += inner.px - item_w / 2.0;
                iy += inner.py - item_h / 2.0;
            } else {
                match side {
                    Side::Top => iy -= item_h / 2.0,
                    Side::Bottom => iy += item_h / 2.0,
                    Side::Left => ix -= item_w / 2.0,
                    Side::Right => ix += item_w / 2.0,
                }
            }
            let mut bx = ix;
            let mut by = iy;
            match side {
                Side::Top => by = 0.0,
                Side::Bottom => by = box_h,
                Side::Left => bx = 0.0,
                Side::Right => bx = box_w,
            }
            let a = if side == Side::Top || side == Side::Bottom { 0.0 } else { 1.0 };
            let port = &mut order[q].ports[i];
            port.px = bx;
            port.py = by;
            port.route = if out { vec![ix, iy, a, bx, by, a] } else { vec![bx, by, a, ix, iy, a] };
        }

        if id >= 0 {
            let group = &mut nodes[id as usize];
            group.w = box_w;
            group.h = box_h;
        } else {
            total = CompoundResult { width: box_w, height: box_h };
        }
    }

    // Top-down again: turn positions inside each box into diagram coordinates.
    for q in 0..order.len() {
        let ox = order[q].x;
        let oy = order[q].y;
        for k in 0..order[q].items.len() {
            let item = order[q].items[k];
            let node = &mut nodes[item];
            node.x += ox;
            node.y += oy;
            if node.is_group {
                let (x, y) = (node.x - node.w / 2.0, node.y - node.h / 2.0);
                let li = level_index(&level_of, item as isize);
                order[li].x = x;
                order[li].y = y;
            }
        }
    }

    let append = |route: &mut Vec<f64>, part: &[f64], ox: f64, oy: f64| {
        let mut i = 0;
        while i < part.len() {
            let x = part[i] + ox;
            let y = part[i + 1] + oy;
            let len = route.len();
            if len >= 3 && (route[len - 3] - x).abs() < 0.01 && (route[len - 2] - y).abs() < 0.01 {
                i += 3;
                continue;
            }
            route.extend_from_slice(&[x, y, part[i + 2]]);
            i += 3;
        }
    };

    for level in order.iter() {
        for le in &level.edges {
            let (src, dst) = (edges[le.e].src, edges[le.e].dst);
            let mut route: Vec<f64> = Vec::new();
            let mut outward: Vec<usize> = Vec::new();
            let mut item = src;
            while nodes[item].parent != level.id {
                outward.push(level_index(&level_of, nodes[item].parent));
                item = nodes[item].parent as usize;
            }
            for &lv in &outward {
                let lv = &order[lv];
                append(&mut route, &lv.ports[lv.port_of[&(le.e * 2)]].route, lv.x, lv.y);
            }
            append(&mut route, &level.routes[&le.e], level.x, level.y);
            let mut inward: Vec<usize> = Vec::new();
            let mut item = dst;
            while nodes[item].parent != level.id {
                inward.push(level_index(&level_of, nodes[item].parent));
                item = nodes[item].parent as usize;
            }
            for &lv in inward.iter().rev() {
                let lv = &order[lv];
                append(&mut route, &lv.ports[lv.port_of[&(le.e * 2 + 1)]].route, lv.x, lv.y);
            }
            let e = &mut edges[le.e];
            e.route = route;
            if e.label_w > 0.0 {
                e.label_x += level.x;
                e.label_y += level.y;
            }
        }
    }

    total
}

struct Flat {
    edge_of: Vec<isize>,
    edges: Vec<LEdge>,
    last: Vec<i32>,
    crossings: i64,
}
impl Flat {
    fn hint(&self, level: &Level, port: &PortEdge, edges: &[CEdge], key: &[f64]) -> f64 {
        let le = self.edge_of[port.e];
        if le < 0 || level.id < 0 || !port.out || port.side != opposite(start_side(level.dir)) {
            return f64::NAN;
        }
        let points = &self.edges[le as usize].points;
        let mut inside = key[edges[port.e].src];
        let mut outside = 0.0;
        for k in (0..points.len()).step_by(2) {
            if points[k] == self.last[level.id as usize] as f64 {
                inside = points[k + 1];
            }
            if points[k] == (self.last[level.id as usize] + 1) as f64 {
                outside = points[k + 1];
            }
        }
        inside + outside * 1e-6
    }
}
struct Laid {
    given: Vec<u64>,
    nodes: Vec<LNode>,
    edges: Vec<LEdge>,
    size: LayeredResult,
}
struct Snapshot {
    nodes: Vec<[f64; 4]>,
    routes: Vec<Vec<f64>>,
    labels: Vec<[f64; 2]>,
    levels: Vec<Vec<f64>>,
    total: CompoundResult,
}
fn snapshot(nodes: &[CNode], edges: &[CEdge], levels: &[Level], total: CompoundResult) -> Snapshot {
    Snapshot {
        nodes: nodes.iter().map(|n| [n.x, n.y, n.w, n.h]).collect(),
        routes: edges.iter().map(|e| e.route.clone()).collect(),
        labels: edges.iter().map(|e| [e.label_x, e.label_y]).collect(),
        levels: levels
            .iter()
            .map(|l| {
                let mut v = vec![l.x, l.y];
                for p in &l.ports {
                    v.extend([p.px, p.py]);
                }
                v
            })
            .collect(),
        total,
    }
}
fn restore(s: Snapshot, nodes: &mut [CNode], edges: &mut [CEdge], levels: &mut [Level]) -> CompoundResult {
    for (n, at) in nodes.iter_mut().zip(s.nodes) {
        [n.x, n.y, n.w, n.h] = at;
    }
    for ((e, r), at) in edges.iter_mut().zip(s.routes).zip(s.labels) {
        e.route = r;
        [e.label_x, e.label_y] = at;
    }
    for (l, at) in levels.iter_mut().zip(s.levels) {
        l.x = at[0];
        l.y = at[1];
        for (i, p) in l.ports.iter_mut().enumerate() {
            p.px = at[2 + 2 * i];
            p.py = at[3 + 2 * i];
        }
    }
    s.total
}
