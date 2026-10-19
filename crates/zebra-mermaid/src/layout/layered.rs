//! Layered (Sugiyama-style) layout of one flat graph. Ported from Pele
//! `src/layout/layered.ts`.
//!
//! Ranks run down the y axis; callers rotate the result for other directions. The port keeps
//! Pele's data layout (flat arrays indexed by node and segment), loop orders, tie-breaks,
//! constants and floating point expressions, so that every coordinate comes out bit for bit as
//! Pele computes it.
//!
//! What the layout of edges aims for, most important first (Pele's own summary). Where two aims
//! pull apart, the one higher up decides, and each names where it is carried out.
//!
//! 1. Lines can be told apart. No two touch or run along each other, and none runs through a
//!    node or a label. Ends on one side of a node have places of their own (`spread`), and
//!    curves that would pass within a few pixels turn at levels of their own (`assign_tracks`).
//! 2. Lines cross as little as the order of the nodes allows (`reduce_crossings`, and the order
//!    of the levels in `assign_tracks`).
//! 3. An edge takes the shortest and simplest way: straight if it can, else one curve, else a
//!    curve with a level run. One that passes several ranks runs in line with one of its ends
//!    and turns once, at the other (`align_labels`, `straighten`). Ends slide along their
//!    sides, and nodes move a little, where that makes an edge straight (`settle`, `snap`).
//! 4. An edge meets the middle of a side. An end with a side to itself stays there, and ends
//!    that share a side sit around the middle in the order of where they lead. An end moves off
//!    the middle only for the aims above (`settle`).
//! 5. The drawing is balanced, and what is alike is drawn alike. A label sits midway along the
//!    line that shows, the curves either side of it have much the same room to turn in, and a
//!    gap whose curves mostly cross has them all as the same plain curve rather than each on a
//!    level of its own (`head_room` in `layered`, the crossed gaps in `assign_tracks`).
//! 6. An edge leaves and reaches a node square to it, and an arrowhead points straight at the
//!    node (`route_path` in svg/edges.rs, which draws the routes).
//! 7. Nodes keep the order they were written in, where nothing above says otherwise.
//! 8. The drawing stays compact. A gap between ranks grows only to hold the levels in it, or
//!    to let curves that cross do so at a clear angle.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::util::{js, math};

/// What a layout node is: a real node, a bend or label of an edge, or a port on a group's
/// border where an edge passes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Node = 0,
    Dummy = 1,
    Label = 2,
    StartPort = 3,
    EndPort = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LNode {
    pub w: f64,
    pub h: f64,
    pub kind: Kind,
    /// -1 keeps the node first in its rank, 1 last, 0 lets it move.
    pub pin: f64,
    /// How much of the side that edges meet may be shared out between them; 0 keeps them all
    /// at its middle.
    pub span: f64,
    pub x: f64,
    pub y: f64,
    pub rank: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LEdge {
    pub tail: usize,
    pub head: usize,
    pub minlen: f64,
    pub label_w: f64,
    pub label_h: f64,
    pub tail_dx: f64,
    pub head_dx: f64,
    /// An end that is held where its offset puts it, and takes no part in sharing out a node's
    /// side.
    pub tail_fixed: bool,
    pub head_fixed: bool,
    pub reversed: bool,
    pub points: Vec<f64>,
    pub label_x: f64,
    pub label_y: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayeredOptions {
    pub node_sep: f64,
    pub edge_sep: f64,
    pub rank_sep: f64,
    pub port_sep: f64,
    /// Route edges at right angles, each on a track of its own across the gap between two ranks.
    pub tracks: bool,
    /// Draw every edge straight from point to point: none is given a level to turn onto.
    pub straight: bool,
    /// The caller sets the ends of the edges on one side of a node apart itself, so two edges
    /// between the same two nodes need no bend to keep them from being drawn as one.
    pub apart: bool,
    /// The room that the marker at the head of an edge takes. A rank of labels whose edges all
    /// head the same way sits half of this nearer their tails, so that each label is midway
    /// along the line that shows, and the curves either side of it have the same room to turn
    /// in. `None` is Pele's `undefined`.
    pub head_room: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayeredResult {
    pub width: f64,
    pub height: f64,
}

/// `lnode(w, h, kind)`.
pub fn lnode(w: f64, h: f64, kind: Kind) -> LNode {
    LNode { w, h, kind, pin: 0.0, span: 0.0, x: 0.0, y: 0.0, rank: 0 }
}

/// `ledge(tail, head, minlen, labelW, labelH)`; Pele's defaults are 1, 0 and 0.
pub fn ledge(tail: usize, head: usize, minlen: f64, label_w: f64, label_h: f64) -> LEdge {
    LEdge {
        tail,
        head,
        minlen,
        label_w,
        label_h,
        tail_dx: 0.0,
        head_dx: 0.0,
        tail_fixed: false,
        head_fixed: false,
        reversed: false,
        points: Vec::new(),
        label_x: 0.0,
        label_y: 0.0,
    }
}

const MAX_SWEEPS: usize = 24;

/// An edge gets a bend point in every rank it skips. Past this many, the longest edges are
/// drawn as one direct curve instead, which keeps huge graphs from exhausting time and memory.
pub const MAX_DUMMIES: usize = 20000;

/// The room between two edges that meet one side of a node, and the least that keeps their
/// arrowheads apart.
const PORT_PITCH: f64 = 16.0;
const MIN_PITCH: f64 = 8.0;

/// Where edges are not given levels, a gap between ranks grows to keep the edges that cross it
/// at least this steep, up to this many times its usual height. Shallower than that, the edges
/// of a wide rank are hard to tell apart.
const MIN_SLOPE: f64 = 0.125;
const MAX_GAP: f64 = 3.0;

/// Tracks: the room kept clear between a rank and the nearest track, the room between two
/// tracks when the gap has to grow to hold them, and the most they spread to when it has room
/// to spare.
const TRACK_MARGIN: f64 = 18.0;
const TRACK_PITCH: f64 = 10.0;
const TRACK_SPREAD: f64 = 14.0;
/// A curve is one plain S unless it runs more than this many times as far across as the gap is
/// high, and so does another that it would run along. Then each turns onto a level of its own
/// to run across on. The levels of a gap keep this far from its ranks, to leave room to turn
/// in, and from each other.
const LEVEL_WIDE: f64 = 2.0;
const LEVEL_MARGIN: f64 = 28.0;
const LEVEL_PITCH: f64 = 12.0;
/// Two plain curves that would pass too near each other turn at levels of their own as well:
/// nearer than LEVEL_CLEAR where one of them runs far across, since it stays near the other
/// for a long way, and otherwise nearer than LEVEL_TOUCH. LEVEL_LEAD is the room a plain curve
/// is taken to keep straight at each end of its gap.
const LEVEL_LEAD: f64 = 12.0;
const LEVEL_CLEAR: f64 = 8.0;
const LEVEL_TOUCH: f64 = 4.0;
/// Levels tell apart curves that run alongside each other. Where the curves of a gap cross each
/// other more often than that, runs along levels only make the crossings long and shallow, so
/// each curve is one plain S instead, and the gap is made this many times as tall as its widest
/// curve runs across (and no taller than MAX_GAP allows), for them to cross at a clear angle.
const CROSSED_SLOPE: f64 = 0.5;
/// How far out of line two points may be and still count as in line.
const IN_LINE: f64 = 0.5;
/// A node, with whatever it is already in line with, moves this far at most to bring one more
/// of its edges into line.
const SNAP: f64 = 12.0;
/// An end that cannot be brought into line leans towards where its edge goes, but keeps this
/// far from the ends of the part of its side that edges spread along. On a narrow node that
/// leaves it in the middle.
const LEAN_CLEAR: f64 = 40.0;
/// Past this many edges across one gap, tracks are given out in a plain order: weighing every
/// pair of edges against each other would take too long.
const MAX_WEIGHED: usize = 400;

/// What a segment's end is to the node it meets, for the ends that may move along the node's
/// side (Pele's `End` enum).
const HELD: u8 = 0;
const LEAVES: u8 = 1;
const ARRIVES: u8 = 2;

#[inline]
fn is_dummy(kind: Kind) -> bool {
    kind == Kind::Dummy || kind == Kind::Label
}

/// `a - b || ...` on numbers: a difference that is zero or NaN falls through to the next key.
#[inline]
fn num_cmp(a: f64, b: f64) -> Ordering {
    let d = a - b;
    if d < 0.0 {
        Ordering::Less
    } else if d > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// Insertion sort of `list[from..to]` by `key`, then by the item itself.
fn sort_range(list: &mut [usize], from: usize, to: usize, key: &[i64]) {
    for i in from + 1..to {
        let item = list[i];
        let k = key[item];
        let mut j = i as isize - 1;
        while j >= from as isize && (key[list[j as usize]] > k || (key[list[j as usize]] == k && list[j as usize] > item)) {
            list[(j + 1) as usize] = list[j as usize];
            j -= 1;
        }
        list[(j + 1) as usize] = item;
    }
}

/// The graph that gets ordered and positioned: every edge cut into rank-to-rank segments.
struct Graph {
    n: usize,
    real: usize,
    max_rank: usize,
    w: Vec<f64>,
    kind: Vec<Kind>,
    rank: Vec<usize>,
    // Segment endpoints, and where each segment attaches on them.
    ef: Vec<usize>,
    et: Vec<usize>,
    efd: Vec<f64>,
    etd: Vec<f64>,
    succ_start: Vec<usize>,
    succ_e: Vec<usize>,
    pred_start: Vec<usize>,
    pred_e: Vec<usize>,
    layer_start: Vec<usize>,
    order: Vec<usize>,
    pos: Vec<usize>,
    offsets: bool,
}

fn set_positions(order: &[usize], pos: &mut [usize], layer_start: &[usize], max_rank: usize) {
    for r in 0..=max_rank {
        let s = layer_start[r];
        for (i, &v) in order.iter().enumerate().take(layer_start[r + 1]).skip(s) {
            pos[v] = i - s;
        }
    }
}

/// The pieces of edges that each cross one gap: where they leave the rank above and reach the
/// one below, the rank above, the bend they end at (or -1 at the edge's last node), and the
/// track they run across on (or -1 when they have none). Where only some pieces are given a
/// level to turn onto, `wide` marks those that run far across, `levels` is how many tracks the
/// group a piece belongs to takes, or 0 for a piece that needs none, and each piece's track
/// counts from its group's first. A piece that is not `wide` curves from the level above its own.
#[derive(Default)]
struct Pieces {
    from: Vec<f64>,
    to: Vec<f64>,
    rank: Vec<usize>,
    lower: Vec<isize>,
    track: Vec<i64>,
    levels: Vec<i64>,
    wide: Vec<u8>,
}

pub fn layered(nodes: &mut [LNode], edges: &mut [LEdge], opt: &LayeredOptions) -> LayeredResult {
    let real = nodes.len();
    if real == 0 {
        return LayeredResult { width: 0.0, height: 0.0 };
    }
    let m = edges.len();

    // Every edge gets a middle rank of its own when labels or parallel edges need the room.
    let mut step: i32 = 1;
    if m > 0 {
        let mut seen: HashSet<usize> = HashSet::new();
        for e in edges.iter() {
            let a = e.tail.min(e.head);
            let b = e.tail.max(e.head);
            let key = a * real + b;
            if e.label_w > 0.0 || seen.contains(&key) {
                step = 2;
                break;
            }
            seen.insert(key);
        }
    }

    orient(nodes, edges);
    rank(nodes, edges, step);

    let mut span = vec![0usize; m];
    let mut dummies: usize = 0;
    // An edge whose ends share a rank has no segment to cut it into. Pele only meets one with a
    // length that is not a number (its typed arrays then ignore the writes past their end); no
    // caller here gives such a length, and such an edge is drawn direct rather than panic.
    let mut direct = vec![false; m];
    for ei in 0..m {
        let e = &edges[ei];
        span[ei] = (nodes[e.head].rank as i64 - nodes[e.tail].rank as i64).unsigned_abs() as usize;
        if span[ei] > 1 {
            dummies += span[ei] - 1;
        }
        if span[ei] == 0 {
            direct[ei] = true;
        }
    }
    if dummies > MAX_DUMMIES {
        let mut longest: Vec<usize> = (0..m).filter(|&ei| span[ei] > 1).collect();
        longest.sort_by(|&a, &b| span[b].cmp(&span[a]).then(a.cmp(&b)));
        for ei in longest {
            if dummies <= MAX_DUMMIES {
                break;
            }
            direct[ei] = true;
            dummies -= span[ei] - 1;
        }
    }

    let n = real + dummies;
    let mut w = vec![0.0f64; n];
    let mut h = vec![0.0f64; n];
    let mut kind = vec![Kind::Node; n];
    let mut rank_of = vec![0usize; n];
    let mut max_rank: usize = 0;
    for i in 0..real {
        let node = &nodes[i];
        w[i] = node.w;
        h[i] = node.h;
        kind[i] = node.kind;
        rank_of[i] = node.rank as usize;
        if node.rank as usize > max_rank {
            max_rank = node.rank as usize;
        }
    }

    // The graph that gets ordered and positioned: every edge cut into rank-to-rank segments.
    let mut segments = 0usize;
    for ei in 0..m {
        if !direct[ei] {
            segments += span[ei];
        }
    }
    let mut ef = vec![0usize; segments];
    let mut et = vec![0usize; segments];
    let mut efd = vec![0.0f64; segments];
    let mut etd = vec![0.0f64; segments];
    let mut first_dummy = vec![-1isize; m];
    // Which way the edges with a label in each rank head: 1 down, 2 up, 3 both.
    let mut heads = vec![0u8; max_rank + 1];
    let mut label_node = vec![-1isize; m];
    let mut spans = false;
    for node in nodes.iter() {
        if node.span > 0.0 && node.kind == Kind::Node {
            spans = true;
        }
    }
    // What each segment's end is to the node it meets, for the ends that may move along the
    // node's side.
    let mut efk: Option<Vec<u8>> = if spans { Some(vec![HELD; segments]) } else { None };
    let mut etk: Option<Vec<u8>> = if spans { Some(vec![HELD; segments]) } else { None };
    let mut first_seg = vec![-1isize; m];
    let mut last_seg = vec![-1isize; m];
    let mut offsets = false;
    {
        let mut next = real;
        let mut ce = 0usize;
        for ei in 0..m {
            if direct[ei] {
                continue;
            }
            let e = &edges[ei];
            let a = if e.reversed { e.head } else { e.tail };
            let b = if e.reversed { e.tail } else { e.head };
            let a_dx = if e.reversed { e.head_dx } else { e.tail_dx };
            let b_dx = if e.reversed { e.tail_dx } else { e.head_dx };
            if a_dx != 0.0 || b_dx != 0.0 {
                offsets = true;
            }
            let ra = rank_of[a] as i64;
            let rb = rank_of[b] as i64;
            let mut label_rank: i64 = -1;
            if e.label_w > 0.0 {
                // The label goes halfway along. An edge that passes ranks of nodes on its way has
                // its label in one of those, beside the nodes, where it takes no height of its
                // own: the one halfway, or of the two either side of halfway, the one nearer the
                // node the edge starts from, which is the lower when the edge runs against the
                // flow. An edge between two neighbouring ranks has its label in the rank between
                // them.
                label_rank = ra + ((rb - ra) >> 1);
                if step == 2 && rb - ra >= 4 {
                    if (label_rank & 1) == 1 {
                        label_rank += if e.reversed { 1 } else { -1 };
                    }
                } else if step == 2 && (label_rank & 1) == 0 {
                    label_rank += if e.reversed { 1 } else { -1 };
                }
            }
            let mut prev = a;
            if rb - ra > 1 {
                first_dummy[ei] = next as isize;
            }
            first_seg[ei] = ce as isize;
            if let Some(efk) = efk.as_mut() {
                if !(if e.reversed { e.head_fixed } else { e.tail_fixed }) {
                    efk[ce] = if e.reversed { ARRIVES } else { LEAVES };
                }
            }
            for r in ra + 1..rb {
                let d = next;
                next += 1;
                rank_of[d] = r as usize;
                if r == label_rank {
                    w[d] = e.label_w;
                    h[d] = e.label_h;
                    kind[d] = Kind::Label;
                    label_node[ei] = d as isize;
                    heads[r as usize] |= if e.reversed { 2 } else { 1 };
                } else {
                    kind[d] = Kind::Dummy;
                }
                ef[ce] = prev;
                et[ce] = d;
                efd[ce] = if prev == a { a_dx } else { 0.0 };
                ce += 1;
                prev = d;
            }
            ef[ce] = prev;
            et[ce] = b;
            efd[ce] = if prev == a { a_dx } else { 0.0 };
            etd[ce] = b_dx;
            if let Some(etk) = etk.as_mut() {
                if !(if e.reversed { e.tail_fixed } else { e.head_fixed }) {
                    etk[ce] = if e.reversed { LEAVES } else { ARRIVES };
                }
            }
            last_seg[ei] = ce as isize;
            ce += 1;
        }
    }

    let mut succ_start = vec![0usize; n + 1];
    let mut pred_start = vec![0usize; n + 1];
    for e in 0..segments {
        succ_start[ef[e] + 1] += 1;
        pred_start[et[e] + 1] += 1;
    }
    for i in 0..n {
        succ_start[i + 1] += succ_start[i];
        pred_start[i + 1] += pred_start[i];
    }
    let mut succ_e = vec![0usize; segments];
    let mut pred_e = vec![0usize; segments];
    {
        let mut sf = succ_start[..n].to_vec();
        let mut pf = pred_start[..n].to_vec();
        for e in 0..segments {
            succ_e[sf[ef[e]]] = e;
            sf[ef[e]] += 1;
            pred_e[pf[et[e]]] = e;
            pf[et[e]] += 1;
        }
    }

    let mut layer_start = vec![0usize; max_rank + 2];
    for &r in rank_of.iter() {
        layer_start[r + 1] += 1;
    }
    for r in 0..=max_rank {
        layer_start[r + 1] += layer_start[r];
    }
    let mut order = vec![0usize; n];
    let mut pos = vec![0usize; n];

    // Initial order: depth-first from the top so that subtrees start out contiguous.
    {
        let mut by_rank = vec![0usize; n];
        let mut fill = layer_start[..=max_rank].to_vec();
        for i in 0..n {
            by_rank[fill[rank_of[i]]] = i;
            fill[rank_of[i]] += 1;
        }
        fill.copy_from_slice(&layer_start[..=max_rank]);
        let mut visited = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        for k in 0..n {
            if visited[by_rank[k]] {
                continue;
            }
            stack.push(by_rank[k]);
            while let Some(v) = stack.pop() {
                if visited[v] {
                    continue;
                }
                visited[v] = true;
                order[fill[rank_of[v]]] = v;
                fill[rank_of[v]] += 1;
                let mut j = succ_start[v + 1];
                while j > succ_start[v] {
                    j -= 1;
                    let w = et[succ_e[j]];
                    if !visited[w] {
                        stack.push(w);
                    }
                }
            }
        }
    }

    let pinned = nodes.iter().any(|node| node.pin != 0.0);
    if pinned {
        for r in 0..=max_rank {
            let layer = &mut order[layer_start[r]..layer_start[r + 1]];
            let pin_of = |v: usize| if v < real { nodes[v].pin } else { 0.0 };
            // A stable sort by pin keeps the rest in the order they had.
            layer.sort_by(|&a, &b| num_cmp(pin_of(a), pin_of(b)));
        }
    }
    set_positions(&order, &mut pos, &layer_start, max_rank);

    let mut g = Graph {
        n,
        real,
        max_rank,
        w,
        kind,
        rank: rank_of,
        ef,
        et,
        efd,
        etd,
        succ_start,
        succ_e,
        pred_start,
        pred_e,
        layer_start,
        order,
        pos,
        offsets,
    };

    if segments > 0 && max_rank > 0 {
        reduce_crossings(&mut g, nodes);
        set_positions(&g.order, &mut g.pos, &g.layer_start, g.max_rank);
    }

    if let (Some(efk), Some(etk)) = (efk.as_ref(), etk.as_ref()) {
        spread(&mut g, nodes, efk, etk);
    }

    let mut xs = position(&mut g, opt);

    let empty = empty_ranks(&g);
    if let (Some(efk), Some(etk)) = (efk.as_ref(), etk.as_ref()) {
        let free = free_ends(&g, efk, etk);
        align_labels(&g, nodes, &mut xs, &free, opt, empty.as_deref());
        settle(&mut g, nodes, &xs, efk, etk, &free, empty.as_deref());
        snap(&g, &mut xs, empty.as_deref(), opt);
        for ei in 0..m {
            if direct[ei] {
                continue;
            }
            let e = &mut edges[ei];
            let a_dx = g.efd[first_seg[ei] as usize];
            let b_dx = g.etd[last_seg[ei] as usize];
            e.tail_dx = if e.reversed { b_dx } else { a_dx };
            e.head_dx = if e.reversed { a_dx } else { b_dx };
        }
    }
    let spreads = spans;

    // A rank that holds nothing but the bends of edges is only there because some other edge
    // has a label or a twin. Its bends move onto the straight line between the ends of their
    // edges, and one that gets there is left out of the route, so that the edge is drawn as a
    // single curve.
    let mut on_line = vec![0u8; n];
    if let Some(empty) = empty.as_deref() {
        if !opt.tracks {
            straighten(&g, &mut xs, empty, &first_dummy, &span, if opt.apart { 0.0 } else { opt.edge_sep }, &mut on_line);
        }
    }
    if !opt.tracks {
        past_labels(&g, &mut xs, &first_dummy, &span, opt.edge_sep, &mut on_line);
    }
    // An edge is cut where it passes a rank that holds something, and each piece crosses one
    // gap. With `tracks`, every piece that does not run straight gets a track to run across on
    // at right angles. Otherwise, where edges spread along their nodes, a piece gets one only
    // if it needs a level to turn onto. `piece_start` gives every edge's run of pieces.
    let mut piece_start = vec![0usize; m + 1];
    let mut pieces: Option<Pieces> = if opt.tracks || (spreads && !opt.straight) { Some(Pieces::default()) } else { None };
    let mut track_count = vec![0i64; max_rank + 1];
    let rank_of = &g.rank;
    if let Some(pieces) = pieces.as_mut() {
        for ei in 0..m {
            piece_start[ei] = pieces.from.len();
            if direct[ei] {
                continue;
            }
            let d0 = first_dummy[ei];
            let end = if d0 < 0 { d0 } else { d0 + span[ei] as isize - 1 };
            // An edge that keeps a bend to stay clear of its twin is drawn through the bend, as
            // it is.
            let mut twin = false;
            let mut d = d0;
            while d < end {
                // So is one that has a bend left out in a rank of labels: its curve crosses that
                // rank.
                let du = d as usize;
                let bare = empty.as_ref().is_some_and(|e| e[rank_of[du]] == 1);
                if !opt.tracks && bare != (on_line[du] == 1) {
                    twin = true;
                }
                d += 1;
            }
            if twin {
                continue;
            }
            let fs = first_seg[ei] as usize;
            let ls = last_seg[ei] as usize;
            let mut x = xs[g.ef[fs]] + g.efd[fs];
            let mut rank = rank_of[g.ef[fs]];
            let mut d = d0;
            while d < end {
                let du = d as usize;
                d += 1;
                if empty.as_ref().is_some_and(|e| e[rank_of[du]] != 0) {
                    continue;
                }
                pieces.from.push(x);
                pieces.to.push(xs[du]);
                pieces.rank.push(rank);
                pieces.lower.push(du as isize);
                x = xs[du];
                rank = rank_of[du];
            }
            pieces.from.push(x);
            pieces.to.push(xs[g.et[ls]] + g.etd[ls]);
            pieces.rank.push(rank);
            pieces.lower.push(-1);
        }
        piece_start[m] = pieces.from.len();
    }

    let layer_start = &g.layer_start;
    let order = &g.order;
    let kind = &g.kind;
    // Ranks that hold only border ports or their pass-through dummies need little room of their
    // own.
    let port_only = |r: usize, k: Kind| -> bool {
        let mut found = false;
        for &v in &order[layer_start[r]..layer_start[r + 1]] {
            let kv = kind[v];
            if kv == k {
                found = true;
            } else if kv != Kind::Dummy {
                return false;
            }
        }
        found
    };
    let start_ranks: i64 = if port_only(0, Kind::StartPort) { step as i64 } else { 0 };
    let end_ranks: i64 = if port_only(max_rank, Kind::EndPort) { step as i64 } else { 0 };
    // The longest way across that any edge travels in the gap below each rank.
    let mut run = vec![0.0f64; max_rank + 1];
    for e in 0..segments {
        let across = (xs[g.ef[e]] + g.efd[e] - xs[g.et[e]] - g.etd[e]).abs();
        if across > run[rank_of[g.ef[e]]] {
            run[rank_of[g.ef[e]]] = across;
        }
    }
    let mut band_top = vec![0.0f64; max_rank + 1];
    let mut band_bottom = vec![0.0f64; max_rank + 1];
    let stepf = step as f64;
    let usual = |r: usize| -> f64 {
        let r = r as i64;
        (if r < start_ranks || r >= max_rank as i64 - end_ranks { opt.port_sep } else { opt.rank_sep }) / stepf
    };
    // The room under each rank. Edges that are not cut into pieces need more where they run far
    // across.
    let mut gap_under = vec![0.0f64; max_rank + 1];
    for r in 0..max_rank {
        gap_under[r] =
            if pieces.is_some() { usual(r) } else { js::max(usual(r), js::min((MAX_GAP * opt.rank_sep) / stepf, run[r] * MIN_SLOPE)) };
    }
    // A rank between two ranks of nodes, which holds labels and bends only.
    let mut between = vec![1u8; max_rank + 1];
    for i in 0..real {
        between[rank_of[i]] = 0;
    }
    for r in 1..max_rank {
        if opt.tracks || between[r] == 0 || (heads[r] != 1 && heads[r] != 2) {
            continue;
        }
        let shift = js::min(opt.head_room.unwrap_or(0.0) / 2.0, gap_under[if heads[r] == 1 { r - 1 } else { r }] / 2.0);
        gap_under[r - 1] += if heads[r] == 1 { -shift } else { shift };
        gap_under[r] += if heads[r] == 1 { shift } else { -shift };
    }
    // The rank that the pieces leaving each rank arrive at, past any that hold only bends, and
    // the room between the two, which grows to hold the gap's tracks or levels: these keep
    // `margin` from the ranks, and between `least_pitch` and `most_pitch` from each other.
    let margin = if opt.tracks { TRACK_MARGIN } else { LEVEL_MARGIN };
    let least_pitch = if opt.tracks { TRACK_PITCH } else { LEVEL_PITCH };
    let most_pitch = if opt.tracks { TRACK_SPREAD } else { LEVEL_PITCH };
    let mut below = vec![0usize; max_rank + 1];
    if let Some(pieces) = pieces.as_mut() {
        let mut room = vec![0.0f64; max_rank + 1];
        for r in 0..max_rank {
            if empty.as_ref().is_some_and(|e| e[r] != 0) {
                continue;
            }
            let mut next = r + 1;
            room[r] = gap_under[r];
            while next < max_rank && empty.as_ref().is_some_and(|e| e[next] != 0) {
                room[r] += gap_under[next];
                next += 1;
            }
            below[r] = next;
        }
        for k in 0..pieces.from.len() {
            let wide = (pieces.from[k] - pieces.to[k]).abs() > LEVEL_WIDE * room[pieces.rank[k]];
            pieces.wide.push(if wide { 1 } else { 0 });
        }
        let mut tall = vec![0.0f64; max_rank + 1];
        assign_tracks(pieces, &mut track_count, !opt.tracks, &room, &mut tall);
        for r in 0..max_rank {
            if track_count[r] > 0 {
                gap_under[r] += js::max(0.0, 2.0 * margin + (track_count[r] - 1) as f64 * least_pitch - room[r]);
            } else if tall[r] > 0.0 {
                gap_under[r] += js::max(0.0, js::min(MAX_GAP * opt.rank_sep, tall[r]) - room[r]);
            }
        }
    }
    let mut y = 0.0f64;
    for r in 0..=max_rank {
        let mut hh = 0.0f64;
        for &v in &order[layer_start[r]..layer_start[r + 1]] {
            if h[v] > hh {
                hh = h[v];
            }
        }
        band_top[r] = y;
        band_bottom[r] = y + hh;
        y += hh + gap_under[r];
    }
    let height = y;
    // Where a piece runs across its gap, or would on another track of its group: its tracks sit
    // in the middle of the gap, the first at the top.
    let track_y = |pieces: &Pieces, piece: usize, on_track: i64| -> f64 {
        let r = pieces.rank[piece];
        let top = band_bottom[r];
        let bottom = band_top[below[r]];
        let count = if opt.tracks { track_count[r] } else { pieces.levels[piece] } as f64;
        let pitch = if count > 1.0 { js::min(most_pitch, (bottom - top - 2.0 * margin) / (count - 1.0)) } else { 0.0 };
        (top + bottom) / 2.0 + (on_track as f64 - (count - 1.0) / 2.0) * pitch
    };
    // The part of its rank that an edge runs straight through at a bend or label. A bare bend
    // beside labels keeps to the middle half, which leaves the curves either side of it more
    // room and still keeps them clear of the labels' corners.
    let inset = |d: usize| -> f64 {
        if kind[d] == Kind::Dummy && between[rank_of[d]] != 0 && !opt.tracks {
            (band_bottom[rank_of[d]] - band_top[rank_of[d]]) / 4.0
        } else {
            0.0
        }
    };

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for i in 0..n {
        let half = g.w[i] / 2.0;
        if xs[i] - half < min_x {
            min_x = xs[i] - half;
        }
        if xs[i] + half > max_x {
            max_x = xs[i] + half;
        }
    }
    // Where edges spread along their nodes, a rank whose nodes only send edges on stands on one
    // line, and a rank whose nodes only receive them hangs from one, so that the edges of nodes
    // of different heights set out from, or arrive at, the same level. In any other rank the
    // nodes are centered.
    let mut arrives = vec![0u8; max_rank + 1];
    let mut leaves = vec![0u8; max_rank + 1];
    for i in 0..real {
        if kind[i] != Kind::Node || !spreads {
            continue;
        }
        if g.pred_start[i + 1] > g.pred_start[i] {
            arrives[rank_of[i]] = 1;
        }
        if g.succ_start[i + 1] > g.succ_start[i] {
            leaves[rank_of[i]] = 1;
        }
    }
    for i in 0..real {
        let node = &mut nodes[i];
        let r = node.rank as usize;
        node.x = xs[i] - min_x;
        node.y = if kind[i] == Kind::Node && leaves[r] != 0 && arrives[r] == 0 {
            band_bottom[r] - node.h / 2.0
        } else if kind[i] == Kind::Node && arrives[r] != 0 && leaves[r] == 0 {
            band_top[r] + node.h / 2.0
        } else {
            (band_top[r] + band_bottom[r]) / 2.0
        };
    }

    for ei in 0..m {
        let (first_x, first_y, first_h, last_x, last_y, last_h) = {
            let e = &edges[ei];
            let first = &nodes[if e.reversed { e.head } else { e.tail }];
            let last = &nodes[if e.reversed { e.tail } else { e.head }];
            (first.x, first.y, first.h, last.x, last.y, last.h)
        };
        let e = &mut edges[ei];
        let mut pts: Vec<f64> = vec![first_x + if e.reversed { e.head_dx } else { e.tail_dx }, first_y + first_h / 2.0];
        let d0 = first_dummy[ei];
        let has_pieces = pieces.as_ref().is_some_and(|_| piece_start[ei + 1] > piece_start[ei]);
        if has_pieces {
            let pieces = pieces.as_ref().expect("pieces");
            for k in piece_start[ei]..piece_start[ei + 1] {
                if pieces.track[k] >= 0 && (opt.tracks || pieces.wide[k] == 1) {
                    let across = track_y(pieces, k, pieces.track[k]);
                    pts.extend_from_slice(&[pieces.from[k] - min_x, across, pieces.to[k] - min_x, across]);
                } else if pieces.track[k] > 0 {
                    // A short piece among levels runs straight until it is past the level before
                    // its own, and turns in one S from there, so that it crosses none of them.
                    pts.extend_from_slice(&[pieces.from[k] - min_x, track_y(pieces, k, pieces.track[k] - 1)]);
                }
                let d = pieces.lower[k];
                if d < 0 {
                    continue;
                }
                let d = d as usize;
                pts.extend_from_slice(&[xs[d] - min_x, band_top[rank_of[d]] + inset(d)]);
                if band_bottom[rank_of[d]] > band_top[rank_of[d]] {
                    pts.extend_from_slice(&[xs[d] - min_x, band_bottom[rank_of[d]] - inset(d)]);
                }
            }
        } else if d0 >= 0 {
            let d0 = d0 as usize;
            for d in d0..d0 + span[ei] - 1 {
                if on_line[d] != 0 {
                    continue;
                }
                let x = xs[d] - min_x;
                let top = band_top[rank_of[d]];
                let bottom = band_bottom[rank_of[d]];
                pts.extend_from_slice(&[x, top + inset(d)]);
                if bottom > top {
                    pts.extend_from_slice(&[x, bottom - inset(d)]);
                }
            }
        }
        pts.extend_from_slice(&[last_x + if e.reversed { e.tail_dx } else { e.head_dx }, last_y - last_h / 2.0]);
        if e.reversed {
            let mut i = 0usize;
            let mut j = pts.len() - 2;
            while i < j {
                pts.swap(i, j);
                pts.swap(i + 1, j + 1);
                i += 2;
                j -= 2;
            }
        }
        let ln = label_node[ei];
        if ln >= 0 {
            let ln = ln as usize;
            e.label_x = xs[ln] - min_x;
            e.label_y = (band_top[rank_of[ln]] + band_bottom[rank_of[ln]]) / 2.0;
        } else if e.label_w > 0.0 {
            e.label_x = (pts[0] + pts[pts.len() - 2]) / 2.0;
            e.label_y = (pts[1] + pts[pts.len() - 1]) / 2.0;
        }
        e.points = pts;
    }

    LayeredResult { width: max_x - min_x, height }
}

/// Reverses the edges that close a cycle, found by depth-first search in declaration order.
pub fn orient(nodes: &[LNode], edges: &mut [LEdge]) {
    let n = nodes.len();
    let mut start = vec![0usize; n + 1];
    for e in edges.iter_mut() {
        e.reversed = nodes[e.head].kind == Kind::StartPort || nodes[e.tail].kind == Kind::EndPort;
        if !e.reversed {
            start[e.tail + 1] += 1;
        }
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut out = vec![0usize; start[n]];
    {
        let mut fill = start[..n].to_vec();
        for (i, e) in edges.iter().enumerate() {
            if !e.reversed {
                out[fill[e.tail]] = i;
                fill[e.tail] += 1;
            }
        }
    }
    let mut state = vec![0u8; n];
    let mut cursor = start[..n].to_vec();
    let mut stack: Vec<usize> = Vec::new();
    for root in 0..n {
        if state[root] != 0 {
            continue;
        }
        stack.push(root);
        state[root] = 1;
        while let Some(&v) = stack.last() {
            if cursor[v] < start[v + 1] {
                let ei = out[cursor[v]];
                cursor[v] += 1;
                let w = edges[ei].head;
                if state[w] == 1 {
                    edges[ei].reversed = true;
                } else if state[w] == 0 {
                    state[w] = 1;
                    stack.push(w);
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
}

pub fn rank(nodes: &mut [LNode], edges: &[LEdge], step: i32) {
    let n = nodes.len();
    let m = edges.len();
    let mut from = vec![0usize; m];
    let mut to = vec![0usize; m];
    // Int32Array: the length is `Math.max(1, minlen) * step` as ToInt32 stores it.
    let mut len = vec![0i64; m];
    let mut out_start = vec![0usize; n + 1];
    let mut in_start = vec![0usize; n + 1];
    for (i, e) in edges.iter().enumerate() {
        let a = if e.reversed { e.head } else { e.tail };
        let b = if e.reversed { e.tail } else { e.head };
        from[i] = a;
        to[i] = b;
        len[i] = js::to_int32(js::max(1.0, e.minlen) * step as f64) as i64;
        out_start[a + 1] += 1;
        in_start[b + 1] += 1;
    }
    for i in 0..n {
        out_start[i + 1] += out_start[i];
        in_start[i + 1] += in_start[i];
    }
    let mut out_e = vec![0usize; m];
    let mut in_e = vec![0usize; m];
    {
        let mut of = out_start[..n].to_vec();
        let mut inf = in_start[..n].to_vec();
        for i in 0..m {
            out_e[of[from[i]]] = i;
            of[from[i]] += 1;
            in_e[inf[to[i]]] = i;
            inf[to[i]] += 1;
        }
    }

    // Int32Array: sums are compared as numbers and stored with ToInt32.
    let mut r = vec![0i64; n];
    let store = |x: i64| x as i32 as i64;
    let mut indeg = vec![0usize; n];
    for i in 0..n {
        indeg[i] = in_start[i + 1] - in_start[i];
    }
    let mut topo = vec![0usize; n];
    let mut count = 0usize;
    for i in 0..n {
        if indeg[i] == 0 {
            topo[count] = i;
            count += 1;
        }
    }
    let mut q = 0;
    while q < count {
        let v = topo[q];
        for k in out_start[v]..out_start[v + 1] {
            let ei = out_e[k];
            let w = to[ei];
            if r[v] + len[ei] > r[w] {
                r[w] = store(r[v] + len[ei]);
            }
            indeg[w] -= 1;
            if indeg[w] == 0 {
                topo[count] = w;
                count += 1;
            }
        }
        q += 1;
    }

    // Longest-path ranking leaves loosely attached nodes far from their neighbours.
    // Move each node toward the side that has more edges.
    for _pass in 0..8 {
        let mut changed = false;
        for q in (0..count).rev() {
            let v = topo[q];
            let ins = in_start[v + 1] - in_start[v];
            let outs = out_start[v + 1] - out_start[v];
            if ins == 0 && outs == 0 {
                continue;
            }
            let mut lo = f64::NEG_INFINITY;
            let mut hi = f64::INFINITY;
            for k in in_start[v]..in_start[v + 1] {
                let ei = in_e[k];
                lo = js::max(lo, (r[from[ei]] + len[ei]) as f64);
            }
            for k in out_start[v]..out_start[v + 1] {
                let ei = out_e[k];
                hi = js::min(hi, (r[to[ei]] - len[ei]) as f64);
            }
            let mut next = r[v] as f64;
            if outs > ins && hi != f64::INFINITY {
                next = hi;
            } else if ins > outs && lo != f64::NEG_INFINITY {
                next = lo;
            }
            if next != r[v] as f64 {
                r[v] = store(next as i64);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut start_ports = false;
    for i in 0..n {
        let k = nodes[i].kind;
        if k == Kind::StartPort {
            start_ports = true;
        }
        if k == Kind::StartPort || k == Kind::EndPort {
            continue;
        }
        if (r[i] as f64) < min {
            min = r[i] as f64;
        }
        if (r[i] as f64) > max {
            max = r[i] as f64;
        }
    }
    if min == f64::INFINITY {
        min = 0.0;
        max = 0.0;
    }
    for i in 0..n {
        if nodes[i].kind == Kind::StartPort {
            r[i] = store((min - step as f64) as i64);
        } else if nodes[i].kind == Kind::EndPort {
            r[i] = store((max + step as f64) as i64);
        }
    }
    if start_ports {
        min -= step as f64;
    }
    for i in 0..n {
        nodes[i].rank = (r[i] as f64 - min) as i32;
    }
}

/// The ends of edges that may move along one side of a node (0 the side its edges arrive at, 1
/// the other), in the order of the nodes at their other ends, so that no two cross. `room`
/// receives the least distance between each end and the one before it, and the result is their
/// sum. When there is too little room for a full pitch between each pair, the edges that leave
/// side by side move closer together, since they have no arrowheads to keep apart. With less
/// room still, neighbours that run the same way share a place, which keeps an arrowhead apart
/// from an edge that leaves beside it.
fn side_ends(g: &Graph, v: usize, side: usize, span: f64, kinds: &[u8], ids: &mut Vec<usize>, room: &mut Vec<f64>) -> f64 {
    let start = if side == 0 { &g.pred_start } else { &g.succ_start };
    let list = if side == 0 { &g.pred_e } else { &g.succ_e };
    let other = if side == 0 { &g.ef } else { &g.et };
    let pos = &g.pos;
    ids.clear();
    for &e in &list[start[v]..start[v + 1]] {
        if kinds[e] != HELD {
            ids.push(e);
        }
    }
    if ids.len() < 2 {
        return 0.0;
    }
    ids.sort_by(|&a, &b| pos[other[a]].cmp(&pos[other[b]]).then(a.cmp(&b)));
    let fill = |room: &mut Vec<f64>, share: &dyn Fn(bool, bool) -> f64| -> f64 {
        let mut total = 0.0;
        room.resize(ids.len(), 0.0);
        room[0] = 0.0;
        for k in 1..ids.len() {
            let same = kinds[ids[k]] == kinds[ids[k - 1]];
            room[k] = share(same && kinds[ids[k]] == LEAVES, same);
            total += room[k];
        }
        total
    };
    let mut total = fill(room, &|_, _| 1.0);
    if span / total < MIN_PITCH {
        total = fill(room, &|leaves, _| if leaves { 0.5 } else { 1.0 });
    }
    if span / total < MIN_PITCH {
        total = fill(room, &|_, same| if same { 0.0 } else { 1.0 });
    }
    if total == 0.0 {
        return 0.0;
    }
    let pitch = js::min(PORT_PITCH, span / total);
    for r in room.iter_mut().take(ids.len()).skip(1) {
        *r *= pitch;
    }
    total * pitch
}

/// Gives the edges that meet one side of a node places of their own along it, around its
/// middle.
fn spread(g: &mut Graph, nodes: &[LNode], efk: &[u8], etk: &[u8]) {
    let mut ids: Vec<usize> = Vec::new();
    let mut room: Vec<f64> = Vec::new();
    for v in 0..g.real {
        let span = js::min(nodes[v].span, g.w[v]);
        if span <= 0.0 || nodes[v].kind != Kind::Node {
            continue;
        }
        for side in 0..2 {
            let total = side_ends(g, v, side, span, if side == 0 { etk } else { efk }, &mut ids, &mut room);
            if total == 0.0 {
                continue;
            }
            let dx = if side == 0 { &mut g.etd } else { &mut g.efd };
            let mut at = -total / 2.0;
            for k in 0..ids.len() {
                at += room[k];
                dx[ids[k]] = at;
            }
        }
    }
}

/// The segment of an edge that reaches the next point held in place, on the way up from
/// segment `e` (side 0) or down: past the bends in ranks that hold nothing else.
fn follow(g: &Graph, empty: Option<&[u8]>, mut e: usize, side: usize) -> usize {
    let Some(empty) = empty else { return e };
    for _hops in 0..100000 {
        let u = if side == 0 { g.ef[e] } else { g.et[e] };
        if g.kind[u] != Kind::Dummy || empty[g.rank[u]] == 0 {
            break;
        }
        e = if side == 0 { g.pred_e[g.pred_start[u]] } else { g.succ_e[g.succ_start[u]] };
    }
    e
}

/// How many ends that may move along it each node has on the side its edges arrive at, and on
/// the other. An end that has a side to itself keeps to the middle; ends that share one cannot
/// all be there, so those may move.
fn free_ends(g: &Graph, efk: &[u8], etk: &[u8]) -> [Vec<i32>; 2] {
    let mut free = [vec![0i32; g.n], vec![0i32; g.n]];
    for e in 0..g.ef.len() {
        if etk[e] != HELD {
            free[0][g.et[e]] += 1;
        }
        if efk[e] != HELD {
            free[1][g.ef[e]] += 1;
        }
    }
    free
}

/// How far along the flow the handles of a plain S reach, as a share of the way it travels
/// along it. Half gives an even S. One that runs more than eight times as far across as along
/// turns sooner and crosses in a straighter line, down to a quarter at sixteen times. The
/// layout uses this to tell how near two curves will pass, and the drawing to draw them.
pub fn ease(across: f64, along: f64) -> f64 {
    // `Math.abs(across) || 1`: zero and NaN count as one.
    let a = across.abs();
    js::max(0.25, js::min(0.5, (4.0 * along.abs()) / if a == 0.0 || a.is_nan() { 1.0 } else { a }))
}

/// Brings the labels and bends of each edge into line with an end of the edge, so that it runs
/// straight from that end and turns once, at the other: with the end that has fewer edges beside
/// it on its node, which is the one nearer the middle of its side, or else with the other, and
/// as near to that as what is beside them in their ranks leaves room. The other end may then
/// move along its node to meet the line. Bends in ranks that hold nothing else are left to
/// `straighten`. An edge with no one line that all of its labels and bends have room on has
/// each brought into line with the nearest point above or below that is held in place.
fn align_labels(g: &Graph, nodes: &[LNode], xs: &mut [f64], free: &[Vec<i32>; 2], opt: &LayeredOptions, empty: Option<&[u8]>) {
    let sep = opt.edge_sep;
    let placed = |v: usize| -> bool { g.kind[v] == Kind::Label || (g.kind[v] == Kind::Dummy && !empty.is_some_and(|e| e[g.rank[v]] == 1)) };
    // Where segment `e` ends on the way up (side 0) or down, and how many ends share that side
    // of the node there: none for a point that is not on a node.
    let end = |xs: &[f64], e: usize, side: usize| -> (f64, i32) {
        let u = if side == 0 { g.ef[e] } else { g.et[e] };
        let x = if side == 0 { xs[u] + g.efd[e] } else { xs[u] + g.etd[e] };
        (x, if u < g.real && nodes[u].kind == Kind::Node { free[1 - side][u] } else { 0 })
    };
    // The room the neighbours in its rank leave a label or bend: a full gap from a node or
    // between two labels, half of one between a label and a line that passes it.
    let room = |xs: &[f64], v: usize| -> (f64, f64) {
        let s = g.layer_start[g.rank[v]];
        let i = s + g.pos[v];
        let clear = |other: usize| -> f64 {
            g.w[other] / 2.0
                + g.w[v] / 2.0
                + if g.kind[other] == Kind::Node {
                    opt.node_sep / 2.0 + sep / 2.0
                } else if g.kind[other] == g.kind[v] {
                    sep
                } else {
                    sep / 2.0
                }
        };
        (
            if i > s { xs[g.order[i - 1]] + clear(g.order[i - 1]) } else { f64::NEG_INFINITY },
            if i + 1 < g.layer_start[g.rank[v] + 1] { xs[g.order[i + 1]] - clear(g.order[i + 1]) } else { f64::INFINITY },
        )
    };
    // The place between `lo` and `hi` that is in line with one of two ends, or else nearest the
    // first choice.
    let choose = |x: f64, up: (f64, i32), down: (f64, i32), lo: f64, hi: f64| -> f64 {
        let up_first = if up.1 != down.1 { up.1 < down.1 } else { (up.0 - x).abs() <= (down.0 - x).abs() };
        let choices = if up_first { [up.0, down.0] } else { [down.0, up.0] };
        match choices.iter().find(|&&c| c >= lo && c <= hi) {
            Some(&c) => c,
            None => js::max(lo, js::min(hi, choices[0])),
        }
    };
    let mut done = vec![false; g.n];
    let mut chain: Vec<usize> = Vec::new();
    for r in 1..g.max_rank {
        for i in g.layer_start[r]..g.layer_start[r + 1] {
            if !placed(g.order[i]) || done[g.order[i]] {
                continue;
            }
            // The edge's labels and bends from its first node to its last, and the room they
            // share.
            let mut top = g.order[i];
            while is_dummy(g.kind[g.ef[g.pred_e[g.pred_start[top]]]]) {
                top = g.ef[g.pred_e[g.pred_start[top]]];
            }
            let mut bottom = top;
            let mut lo = f64::NEG_INFINITY;
            let mut hi = f64::INFINITY;
            chain.clear();
            let mut d = top;
            while is_dummy(g.kind[d]) {
                bottom = d;
                if placed(d) {
                    done[d] = true;
                    chain.push(d);
                    let (least, most) = room(xs, d);
                    if least > lo {
                        lo = least;
                    }
                    if most < hi {
                        hi = most;
                    }
                }
                d = g.et[g.succ_e[g.succ_start[d]]];
            }
            if lo <= hi {
                let x = choose(xs[chain[0]], end(xs, g.pred_e[g.pred_start[top]], 0), end(xs, g.succ_e[g.succ_start[bottom]], 1), lo, hi);
                for &d in &chain {
                    xs[d] = x;
                }
                continue;
            }
            for &d in &chain {
                let (least, most) = room(xs, d);
                if least > most {
                    continue;
                }
                let up = end(xs, follow(g, empty, g.pred_e[g.pred_start[d]], 0), 0);
                let down = end(xs, follow(g, empty, g.succ_e[g.succ_start[d]], 1), 1);
                xs[d] = choose(xs[d], up, down, least, most);
            }
        }
    }
}

/// Once the nodes have their places, moves an end along its side when that lets its edge run
/// straight: when the point the edge runs to, or some part of the side it runs to, is in line
/// with the end's own side. Any other end leans towards the point its edge runs to, where its
/// side is long enough, and an end with a side to itself stays in the middle. On a group, the
/// ends keep clear of the points where other edges pass through its border.
#[allow(clippy::too_many_arguments)]
fn settle(g: &mut Graph, nodes: &[LNode], xs: &[f64], efk: &[u8], etk: &[u8], free: &[Vec<i32>; 2], empty: Option<&[u8]>) {
    let real = g.real;
    let mut ids: Vec<usize> = Vec::new();
    let mut room: Vec<f64> = Vec::new();
    let mut all: Vec<usize> = Vec::new();
    let mut sum: Vec<f64> = Vec::new();
    let mut count: Vec<usize> = Vec::new();
    // Where the end of segment `e` on node `v` should be, from the middle of its side. The edge
    // is followed up from the segment (side 0) or down, past the bends in ranks that hold
    // nothing else, to where it next comes to a point that is held in place or to another
    // node's side.
    let wanted = |g: &Graph, e: usize, side: usize, v: usize, span: f64, again: bool| -> f64 {
        if free[side][v] < 2 {
            return 0.0;
        }
        let e = follow(g, empty, e, side);
        let u = if side == 0 { g.ef[e] } else { g.et[e] };
        let point = if side == 0 { xs[u] + g.efd[e] } else { xs[u] + g.etd[e] };
        // Once the other end has its place, this one comes into line with it if it can.
        if again && (point - xs[v]).abs() <= span / 2.0 + IN_LINE {
            return js::max(-span / 2.0, js::min(span / 2.0, point - xs[v]));
        }
        // The part of the other side that its end may move along, if it may move.
        let mut middle = point;
        let mut width = 0.0;
        if u < real && nodes[u].kind == Kind::Node && (if side == 0 { efk } else { etk })[e] != HELD && free[1 - side][u] >= 2 {
            width = js::max(0.0, js::min(nodes[u].span, g.w[u]));
            if width > 0.0 {
                middle = xs[u];
            }
        }
        let lo = js::max(xs[v] - span / 2.0, middle - width / 2.0);
        let hi = js::min(xs[v] + span / 2.0, middle + width / 2.0);
        // Two sides with no part in line: the end leans towards the other, if its side is long
        // enough.
        if lo > hi + IN_LINE {
            let lean = js::max(0.0, span / 2.0 - LEAN_CLEAR);
            return js::max(-lean, js::min(lean, point - xs[v]));
        }
        // Both ends settle on one place in the part the two sides share: nearer the middle of
        // the narrower.
        let place = (xs[v] * width + middle * span) / (width + span);
        js::max(lo, js::min(hi, place)) - xs[v]
    };
    // The ends of an edge follow each other, so the nodes are gone over until none moves: a few
    // times at most, in practice.
    let mut pass = 0;
    let mut moved = true;
    while moved && pass < 6 {
        moved = false;
        for v in 0..real {
            let span = js::min(nodes[v].span, g.w[v]);
            if span <= 0.0 || nodes[v].kind != Kind::Node {
                continue;
            }
            for side in 0..2 {
                let kinds = if side == 0 { etk } else { efk };
                side_ends(g, v, side, span, kinds, &mut ids, &mut room);
                if ids.is_empty() {
                    continue;
                }
                if ids.len() == 1 {
                    if room.is_empty() {
                        room.push(0.0);
                    } else {
                        room[0] = 0.0;
                    }
                }
                // Every end on the side in order, the held ones among them, which divide the
                // rest into runs.
                all.clear();
                {
                    let start = if side == 0 { &g.pred_start } else { &g.succ_start };
                    let list = if side == 0 { &g.pred_e } else { &g.succ_e };
                    let other = if side == 0 { &g.ef } else { &g.et };
                    all.extend_from_slice(&list[start[v]..start[v + 1]]);
                    let pos = &g.pos;
                    all.sort_by(|&a, &b| pos[other[a]].cmp(&pos[other[b]]).then(a.cmp(&b)));
                }
                let mut k = 0usize;
                let mut lo = -span / 2.0;
                for i in 0..=all.len() {
                    let held = i < all.len() && kinds[all[i]] == HELD;
                    if i < all.len() && !held {
                        continue;
                    }
                    // The run of free ends before this point is ids[k..], as many as were passed
                    // since the last held end.
                    let mut size = 0usize;
                    let mut j = i as isize - 1;
                    while j >= 0 && kinds[all[j as usize]] != HELD {
                        size += 1;
                        j -= 1;
                    }
                    let dx_of = |g: &Graph, e: usize| if side == 0 { g.etd[e] } else { g.efd[e] };
                    let hi = if held { dx_of(g, all[i]) - PORT_PITCH } else { span / 2.0 };
                    if size > 0 {
                        // The places nearest the wanted ones that keep the order and the room
                        // between neighbours: an isotonic fit, by pooling neighbours that would
                        // otherwise be too close.
                        sum.clear();
                        count.clear();
                        let mut before = 0.0;
                        for j in 0..size {
                            if j > 0 {
                                before += room[k + j];
                            }
                            sum.push(wanted(g, ids[k + j], side, v, span, pass > 0) - before);
                            count.push(1);
                            // Ends that share a place for want of room stay together.
                            let mut together = j > 0 && room[k + j] == 0.0;
                            while sum.len() > 1
                                && (together
                                    || sum[sum.len() - 2] / count[count.len() - 2] as f64
                                        > sum[sum.len() - 1] / count[count.len() - 1] as f64)
                            {
                                together = false;
                                let top = sum.pop().expect("pooled sum");
                                let pooled = count.pop().expect("pooled count");
                                let last = sum.len() - 1;
                                sum[last] += top;
                                count[last] += pooled;
                            }
                        }
                        let total = before;
                        let mut j = 0usize;
                        before = 0.0;
                        for b in 0..sum.len() {
                            // With too little room between two held ends, the run sits in the
                            // middle of what there is.
                            let level = if hi - total < lo {
                                (lo + hi - total) / 2.0
                            } else {
                                js::max(lo, js::min(hi - total, sum[b] / count[b] as f64))
                            };
                            for _c in 0..count[b] {
                                if j > 0 {
                                    before += room[k + j];
                                }
                                let id = ids[k + j];
                                let dx = if side == 0 { &mut g.etd } else { &mut g.efd };
                                if (dx[id] - level - before).abs() > 0.01 {
                                    moved = true;
                                }
                                dx[id] = level + before;
                                j += 1;
                            }
                        }
                        k += size;
                    }
                    if held {
                        lo = dx_of(g, all[i]) + PORT_PITCH;
                    }
                }
            }
        }
        pass += 1;
    }
}

/// Brings edges that are nearly straight into line, by moving a node a little across the flow
/// along with everything it is in line with already, where the nodes beside them leave room.
/// What is in line stays in line, so each move leaves one more edge straight.
fn snap(g: &Graph, xs: &mut [f64], empty: Option<&[u8]>, opt: &LayeredOptions) {
    let passed = |u: usize| -> bool { empty.is_some_and(|e| g.kind[u] == Kind::Dummy && e[g.rank[u]] == 1) };
    // Each edge from one point that is held in place to the next, past the bends in ranks that
    // hold nothing else.
    let mut a: Vec<usize> = Vec::new();
    let mut b: Vec<usize> = Vec::new();
    let mut da: Vec<f64> = Vec::new();
    let mut db: Vec<f64> = Vec::new();
    for e in 0..g.ef.len() {
        if passed(g.ef[e]) {
            continue;
        }
        let f = follow(g, empty, e, 1);
        a.push(g.ef[e]);
        b.push(g.et[f]);
        da.push(g.efd[e]);
        db.push(g.etd[f]);
    }
    let off = |xs: &[f64], k: usize| -> f64 { xs[b[k]] + db[k] - xs[a[k]] - da[k] };
    // The nodes in line with each other move as one.
    let n = g.n;
    let mut block: Vec<usize> = (0..n).collect();
    let mut members: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    let join = |block: &mut Vec<usize>, members: &mut Vec<Vec<usize>>, x: usize, y: usize| {
        let mut keep = block[x];
        let mut drop = block[y];
        if keep == drop {
            return;
        }
        if members[keep].len() < members[drop].len() {
            std::mem::swap(&mut keep, &mut drop);
        }
        let moving = std::mem::take(&mut members[drop]);
        for &v in &moving {
            block[v] = keep;
        }
        members[keep].extend_from_slice(&moving);
    };
    let mut near: Vec<usize> = Vec::new();
    for k in 0..a.len() {
        let d = off(xs, k).abs();
        if d < 0.01 {
            join(&mut block, &mut members, a[k], b[k]);
        } else if d <= SNAP {
            near.push(k);
        }
    }
    if near.is_empty() {
        return;
    }
    near.sort_by(|&x, &y| num_cmp(off(xs, x).abs(), off(xs, y).abs()).then(x.cmp(&y)));
    let gap = |v: usize| half_room(g, opt, v);
    // Whether every node of a block can move by `shift` and keep its room from those beside it.
    let free = |xs: &[f64], block: &[usize], members: &[Vec<usize>], id: usize, shift: f64| -> bool {
        for &v in &members[id] {
            let s = g.layer_start[g.rank[v]];
            let end = g.layer_start[g.rank[v] + 1];
            let i = s + g.pos[v];
            if i > s && block[g.order[i - 1]] != id && xs[v] + shift - xs[g.order[i - 1]] < gap(v) + gap(g.order[i - 1]) - 0.01 {
                return false;
            }
            if i + 1 < end && block[g.order[i + 1]] != id && xs[g.order[i + 1]] - xs[v] - shift < gap(v) + gap(g.order[i + 1]) - 0.01 {
                return false;
            }
        }
        true
    };
    for &k in &near {
        let d = off(xs, k);
        if d.abs() < 0.01 {
            join(&mut block, &mut members, a[k], b[k]);
            continue;
        }
        if d.abs() > SNAP || block[a[k]] == block[b[k]] {
            continue;
        }
        // The smaller of the two blocks moves, if it can; else the other.
        let upper = block[a[k]];
        let lower = block[b[k]];
        let first = if members[upper].len() <= members[lower].len() { upper } else { lower };
        for id in [first, if first == upper { lower } else { upper }] {
            let shift = if id == upper { d } else { -d };
            if !free(xs, &block, &members, id, shift) {
                continue;
            }
            for &v in &members[id] {
                xs[v] += shift;
            }
            join(&mut block, &mut members, a[k], b[k]);
            break;
        }
    }
}

/// Marks the ranks that hold nothing but the bends of edges, if there are any.
fn empty_ranks(g: &Graph) -> Option<Vec<u8>> {
    let mut empty = vec![0u8; g.max_rank + 1];
    let mut any = false;
    for r in 1..g.max_rank {
        let mut only = g.layer_start[r + 1] > g.layer_start[r];
        let mut i = g.layer_start[r];
        while only && i < g.layer_start[r + 1] {
            if g.kind[g.order[i]] != Kind::Dummy {
                only = false;
            }
            i += 1;
        }
        if only {
            empty[r] = 1;
            any = true;
        }
    }
    if any {
        Some(empty)
    } else {
        None
    }
}

/// Gives every piece that does not run straight down a track across its gap, and counts the
/// tracks of each gap. Two pieces in one gap share a track only where they cannot be mistaken
/// for each other: where they do not overlap, or where they leave from or arrive at the same
/// point, as the branches of a tree do. Which of two goes above the other is whichever makes
/// them cross less. With `curved`, the tracks are levels for curves to turn onto, and only the
/// pieces that would run along or too near another get one: each group of those counts its
/// tracks from its own first. `room` is the height of the gap under each rank, and `tall`
/// receives the height that a gap whose curves mostly cross each other should have instead of
/// levels.
fn assign_tracks(pieces: &mut Pieces, track_count: &mut [i64], curved: bool, room: &[f64], tall: &mut [f64]) {
    let total = pieces.from.len();
    let mut by_rank: IndexMap<usize, Vec<usize>> = IndexMap::new();
    for k in 0..total {
        pieces.track.push(-1);
        pieces.levels.push(0);
        if (pieces.from[k] - pieces.to[k]).abs() < IN_LINE {
            continue;
        }
        by_rank.entry(pieces.rank[k]).or_default().push(k);
    }
    let from = &pieces.from;
    let to = &pieces.to;
    let wide = &pieces.wide;
    let track = &mut pieces.track;
    let levels = &mut pieces.levels;
    // The pieces are compared pair by pair, so where each begins and ends across the flow is
    // worked out once.
    let mut lo_of = vec![0.0f64; total];
    let mut hi_of = vec![0.0f64; total];
    for k in 0..total {
        lo_of[k] = js::min(from[k], to[k]);
        hi_of[k] = js::max(from[k], to[k]);
    }
    let lo_of = &lo_of;
    let hi_of = &hi_of;
    let within = |x: f64, k: usize| -> i32 { (x >= lo_of[k] - IN_LINE && x <= hi_of[k] + IN_LINE) as i32 };
    let mates = |a: usize, b: usize| -> bool { (from[a] - from[b]).abs() < IN_LINE || (to[a] - to[b]).abs() < IN_LINE };
    let overlap = |a: usize, b: usize| -> bool { lo_of[a] < hi_of[b] - IN_LINE && lo_of[b] < hi_of[a] - IN_LINE };
    // Whether a and b need tracks of their own.
    let clash = |a: usize, b: usize| -> bool { overlap(a, b) && !mates(a, b) };
    // Whether one of a and b could cross the other, depending on which is above.
    let meet =
        |a: usize, b: usize| -> bool { overlap(a, b) || within(to[a], b) + within(from[b], a) + within(to[b], a) + within(from[a], b) > 0 };
    // How often a and b cross with a on the higher track: a's way down through b, and b's way
    // down to itself through a.
    let crossings = |a: usize, b: usize| -> i32 { within(to[a], b) + within(from[b], a) };

    // Gives the pieces of one gap in `list` their tracks, and each the number of tracks that the
    // pieces it has to do with take between them, counting its own from the first of those.
    // Returns the number of tracks. The pieces come without tracks.
    let lay = |list: &mut Vec<usize>, track: &mut Vec<i64>, levels: &mut Vec<i64>| -> i64 {
        // A plain order to start from, and to keep where nothing says otherwise: the pieces that
        // reach furthest go first, so that those of one fan nest.
        list.sort_by(|&a, &b| num_cmp(hi_of[b] - lo_of[b], hi_of[a] - lo_of[a]).then(num_cmp(lo_of[a], lo_of[b])).then(a.cmp(&b)));
        let count = list.len();
        let mut above: Vec<Vec<usize>> = Vec::new();
        let mut group: Vec<usize> = (0..count).collect();
        let find = |group: &mut Vec<usize>, mut i: usize| -> usize {
            while group[i] != i {
                group[i] = group[group[i]];
                i = group[i];
            }
            i
        };
        let mut sequence: Vec<usize> = list.clone();
        if count <= MAX_WEIGHED {
            // above[j] lists the pieces that should be above list[j]. The order follows as many
            // of these as it can: a piece is placed once all that should be above it are, and
            // when none is free, the one with the fewest still waiting.
            let mut waiting = vec![0i32; count];
            let mut after: Vec<Vec<usize>> = Vec::with_capacity(count);
            for _ in 0..count {
                above.push(Vec::new());
                after.push(Vec::new());
            }
            for i in 0..count {
                for j in i + 1..count {
                    let a = list[i];
                    let b = list[j];
                    if !meet(a, b) {
                        continue;
                    }
                    let fi = find(&mut group, i);
                    let fj = find(&mut group, j);
                    group[fi] = fj;
                    let ab = crossings(a, b);
                    let ba = crossings(b, a);
                    if ab == ba {
                        continue;
                    }
                    let first = if ab < ba { i } else { j };
                    let second = if ab < ba { j } else { i };
                    above[second].push(first);
                    after[first].push(second);
                    waiting[second] += 1;
                }
            }
            let mut placed = vec![false; count];
            let mut result: Vec<usize> = Vec::with_capacity(count);
            for _done in 0..count {
                let mut pick: isize = -1;
                for i in 0..count {
                    if placed[i] {
                        continue;
                    }
                    if pick < 0 || waiting[i] < waiting[pick as usize] {
                        pick = i as isize;
                    }
                    if waiting[i] == 0 {
                        break;
                    }
                }
                let pick = pick as usize;
                placed[pick] = true;
                result.push(pick);
                for &j in &after[pick] {
                    waiting[j] -= 1;
                }
            }
            sequence = result.iter().map(|&i| list[i]).collect();
        }
        // Each piece takes the first track below those it should be under that has room for it.
        let place: HashMap<usize, usize> = list.iter().enumerate().map(|(i, &k)| (k, i)).collect();
        let mut rows: Vec<Vec<usize>> = Vec::new();
        for &k in &sequence {
            let mut t: i64 = 0;
            if let Some(list_above) = above.get(place[&k]) {
                for &i in list_above {
                    if track[list[i]] >= t {
                        t = track[list[i]] + 1;
                    }
                }
            }
            loop {
                let tu = t as usize;
                while rows.len() <= tu {
                    rows.push(Vec::new());
                }
                let row = &mut rows[tu];
                let mut free = true;
                for &other in row.iter() {
                    if clash(k, other) {
                        free = false;
                    }
                }
                if free {
                    row.push(k);
                    break;
                }
                t += 1;
            }
            track[k] = t;
        }
        let mut first: Vec<Option<i64>> = vec![None; count];
        let mut last_of: Vec<Option<i64>> = vec![None; count];
        for (i, &k) in list.iter().enumerate() {
            let root = find(&mut group, i);
            first[root] = Some(first[root].map_or(track[k], |f| f.min(track[k])));
            last_of[root] = Some(last_of[root].map_or(track[k], |l| l.max(track[k])));
        }
        for (i, &k) in list.iter().enumerate() {
            let root = find(&mut group, i);
            let f = first[root].expect("group first");
            levels[k] = last_of[root].expect("group last") - f + 1;
            if curved {
                track[k] -= f;
            }
        }
        rows.len() as i64
    };

    // The least distance between a and b as plain curves across a gap that leaves them
    // `height` to turn in: between points along each and the other drawn as a run of short
    // lines.
    const SAMPLES: usize = 16;
    let sample = |k: usize, h: f64, out: &mut [f64; 2 * SAMPLES + 2]| {
        let c = ease(to[k] - from[k], h);
        for i in 0..=SAMPLES {
            let t = i as f64 / SAMPLES as f64;
            let u = 1.0 - t;
            out[2 * i] = from[k] + (to[k] - from[k]) * t * t * (3.0 - 2.0 * t);
            out[2 * i + 1] = h * (3.0 * c * t * u * u + 3.0 * (1.0 - c) * t * t * u + t * t * t);
        }
    };
    let reach = |points: &[f64; 2 * SAMPLES + 2], line: &[f64; 2 * SAMPLES + 2]| -> f64 {
        let mut least = f64::INFINITY;
        for i in 0..=SAMPLES {
            let px = points[2 * i];
            let py = points[2 * i + 1];
            for j in 0..SAMPLES {
                let dx = line[2 * j + 2] - line[2 * j];
                let dy = line[2 * j + 3] - line[2 * j + 1];
                let denom = dx * dx + dy * dy;
                let denom = if denom == 0.0 || denom.is_nan() { 1.0 } else { denom };
                let t = js::max(0.0, js::min(1.0, ((px - line[2 * j]) * dx + (py - line[2 * j + 1]) * dy) / denom));
                let d = math::hypot(px - line[2 * j] - t * dx, py - line[2 * j + 1] - t * dy);
                if d < least {
                    least = d;
                }
            }
        }
        least
    };
    let mut line_a = [0.0f64; 2 * SAMPLES + 2];
    let mut line_b = [0.0f64; 2 * SAMPLES + 2];
    let mut nearest = |a: usize, b: usize, height: f64| -> f64 {
        let h = js::max(height / 2.0, height - 2.0 * LEVEL_LEAD);
        sample(a, h, &mut line_a);
        sample(b, h, &mut line_b);
        js::min(reach(&line_a, &line_b), reach(&line_b, &line_a))
    };

    for (&r, all) in by_rank.iter_mut() {
        if !curved {
            track_count[r] = lay(all, track, levels);
            continue;
        }
        if all.len() > MAX_WEIGHED {
            continue;
        }
        // The pieces that turn onto levels: first those that run far across and have another
        // such piece to be told apart from, then any two that would pass too near each other as
        // plain curves.
        let mut far: Vec<usize> = all.iter().copied().filter(|&k| wide[k] == 1).collect();
        lay(&mut far, track, levels);
        let mut level: HashSet<usize> = far.iter().copied().filter(|&k| levels[k] > 1).collect();
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                let a = all[i];
                let b = all[j];
                if (level.contains(&a) && level.contains(&b)) || !clash(a, b) || (from[a] < to[a]) != (from[b] < to[b]) {
                    continue;
                }
                if nearest(a, b, room[r]) < if wide[a] == 1 || wide[b] == 1 { LEVEL_CLEAR } else { LEVEL_TOUCH } {
                    level.insert(a);
                    level.insert(b);
                }
            }
        }
        for &k in &far {
            track[k] = -1;
            levels[k] = 0;
        }
        if level.is_empty() {
            continue;
        }
        let mut crossing = 0;
        let mut alongside = 0;
        // Only the number of pairs counts here, so the order of the set does not matter.
        let turning: Vec<usize> = all.iter().copied().filter(|k| level.contains(k)).collect();
        for i in 0..turning.len() {
            for j in i + 1..turning.len() {
                let a = turning[i];
                let b = turning[j];
                if !overlap(a, b) {
                    continue;
                }
                if (from[a] < to[a]) != (from[b] < to[b]) {
                    crossing += 1;
                } else if !mates(a, b) {
                    alongside += 1;
                }
            }
        }
        if crossing > alongside {
            for &k in all.iter() {
                if (hi_of[k] - lo_of[k]) * CROSSED_SLOPE > tall[r] {
                    tall[r] = (hi_of[k] - lo_of[k]) * CROSSED_SLOPE;
                }
            }
            continue;
        }
        // With them go the short pieces that could cross one of them, so that each has its
        // place among the levels and crosses none.
        let beside = |k: usize| -> bool { turning.iter().any(|&j| meet(k, j)) };
        let mut members: Vec<usize> = all.iter().copied().filter(|&k| level.contains(&k) || (wide[k] != 1 && beside(k))).collect();
        lay(&mut members, track, levels);
        let mut most = 0;
        for &k in &members {
            if levels[k] > most {
                most = levels[k];
            }
        }
        track_count[r] = most;
    }
}

/// Moves the bends in ranks that hold only bends onto the straight line between the nearest
/// points of their edges that are held in place, and marks the ones that can be left out of the
/// route. `first_dummy` and `span` give each edge's run of bends, one in every rank. `sep` is
/// the room kept between the bends of twin edges, or 0 to leave those out as well.
fn straighten(g: &Graph, xs: &mut [f64], empty: &[u8], first_dummy: &[isize], span: &[usize], sep: f64, on_line: &mut [u8]) {
    let len = xs.len();
    let mut want = vec![0.0f64; len];
    // The points that each bend's edge runs between, across the ranks that hold only bends.
    let mut from = vec![0.0f64; len];
    let mut to = vec![0.0f64; len];
    for ei in 0..first_dummy.len() {
        let d0 = first_dummy[ei];
        if d0 < 0 {
            continue;
        }
        let d0 = d0 as usize;
        let end = d0 + span[ei] - 1;
        // A bend's rank is its place in the run, counted from the rank after the edge's first
        // node.
        let base = g.rank[d0];
        let mut d = d0;
        while d < end {
            if empty[base + d - d0] == 0 {
                d += 1;
                continue;
            }
            let mut last = d;
            while last + 1 < end && empty[base + last + 1 - d0] != 0 {
                last += 1;
            }
            let above = g.pred_e[g.pred_start[d]];
            let below = g.succ_e[g.succ_start[last]];
            let x0 = xs[g.ef[above]] + g.efd[above];
            let x1 = xs[g.et[below]] + g.etd[below];
            let steps = (last - d + 2) as f64;
            for k in d..=last {
                want[k] = x0 + ((x1 - x0) * (k - d + 1) as f64) / steps;
                from[k] = x0;
                to[k] = x1;
            }
            d = last + 1;
        }
    }

    // Nothing in such a rank is in a bend's way, so each one goes onto its line and out of the
    // route. Only twins stay: bends of edges that run between the same two points, which would
    // otherwise be drawn as one. They take places a little apart, around the line.
    let mut twins: Vec<usize> = Vec::new();
    for r in 1..g.max_rank {
        if empty[r] == 0 {
            continue;
        }
        let s = g.layer_start[r];
        let end = g.layer_start[r + 1];
        let mut row: Vec<usize> = g.order[s..end].to_vec();
        row.sort_by(|&a, &b| num_cmp(from[a], from[b]).then(num_cmp(to[a], to[b])).then(a.cmp(&b)));
        let mut i = 0;
        while i < row.len() {
            twins.clear();
            let mut j = i;
            while j < row.len() && (from[row[j]] - from[row[i]]).abs() < 1.0 && (to[row[j]] - to[row[i]]).abs() < 1.0 {
                twins.push(row[j]);
                j += 1;
            }
            // The twins keep the order the rank gave them, so that two edges do not swap sides
            // on the way.
            twins.sort_by(|&a, &b| g.pos[a].cmp(&g.pos[b]));
            for k in 0..twins.len() {
                let d = twins[k];
                xs[d] = want[d] + (k as f64 - (twins.len() as f64 - 1.0) / 2.0) * sep;
                if twins.len() == 1 || sep == 0.0 {
                    on_line[d] = 1;
                }
            }
            i = j;
        }
    }
}

/// In a rank that holds labels and bends but no nodes, a bend is there to take its edge
/// straight through between the labels. Where the edge would pass clear of every label anyway,
/// the bend is left out, and the edge turns in the whole height between the ranks on either
/// side.
fn past_labels(g: &Graph, xs: &mut [f64], first_dummy: &[isize], span: &[usize], clear: f64, on_line: &mut [u8]) {
    let max_rank = g.max_rank;
    let mut labelled = vec![0u8; max_rank + 1];
    let mut any = false;
    for r in 1..max_rank {
        let mut labels = false;
        let mut others = false;
        for &v in &g.order[g.layer_start[r]..g.layer_start[r + 1]] {
            let k = g.kind[v];
            if k == Kind::Label {
                labels = true;
            } else if k != Kind::Dummy {
                others = true;
            }
        }
        if labels && !others {
            labelled[r] = 1;
            any = true;
        }
    }
    if !any {
        return;
    }
    for ei in 0..first_dummy.len() {
        let d0 = first_dummy[ei];
        if d0 < 0 {
            continue;
        }
        let d0 = d0 as usize;
        let end = d0 + span[ei] - 1;
        for d in d0..end {
            if g.kind[d] != Kind::Dummy || labelled[g.rank[d]] == 0 {
                continue;
            }
            // The points on either side that the edge is held at, past any bends already left
            // out.
            let mut first = d;
            let mut last = d;
            while first > d0 && on_line[first - 1] != 0 {
                first -= 1;
            }
            while last + 1 < end && on_line[last + 1] != 0 {
                last += 1;
            }
            let above = g.pred_e[g.pred_start[first]];
            let below = g.succ_e[g.succ_start[last]];
            let from = xs[g.ef[above]] + g.efd[above];
            let to = xs[g.et[below]] + g.etd[below];
            let lo = js::min(from, to) - clear;
            let hi = js::max(from, to) + clear;
            let mut free = true;
            let r = g.rank[d];
            let mut i = g.layer_start[r];
            while free && i < g.layer_start[r + 1] {
                let label = g.order[i];
                if g.kind[label] == Kind::Label && xs[label] + g.w[label] / 2.0 > lo && xs[label] - g.w[label] / 2.0 < hi {
                    free = false;
                }
                i += 1;
            }
            if !free {
                continue;
            }
            on_line[d] = 1;
            // The bends left out between the two points lie on the line between them.
            let steps = (last - first + 2) as f64;
            for k in first..=last {
                xs[k] = from + ((to - from) * (k - first + 1) as f64) / steps;
            }
        }
    }
}

fn reduce_crossings(g: &mut Graph, nodes: &[LNode]) {
    let n = g.n;
    let real = g.real;
    let max_rank = g.max_rank;
    let segments = g.ef.len();
    let mut widest = 0usize;
    for r in 0..=max_rank {
        widest = widest.max(g.layer_start[r + 1] - g.layer_start[r]);
    }
    // With offsets, edges that reach one node at different offsets take separate places along
    // it.
    let mut place: Option<Vec<usize>> = if g.offsets { Some(vec![0usize; segments]) } else { None };
    let mut size = 1usize;
    while size < if place.is_some() { widest + segments } else { widest } {
        size <<= 1;
    }
    let mut tree = vec![0i64; 2 * size];
    let mut targets = vec![0usize; segments];

    let count = |g: &Graph, place: &mut Option<Vec<usize>>, tree: &mut [i64], targets: &mut [usize]| -> i64 {
        let ls = &g.layer_start;
        let order = &g.order;
        let mut total: i64 = 0;
        for r in 0..max_rank {
            let mut south_len = ls[r + 2] - ls[r + 1];
            if let Some(place) = place.as_mut() {
                let mut places = 0usize;
                for i in ls[r + 1]..ls[r + 2] {
                    let from = g.pred_start[order[i]];
                    let to = g.pred_start[order[i] + 1];
                    let mut mixed = false;
                    for k in from + 1..to {
                        if g.etd[g.pred_e[k]] != g.etd[g.pred_e[from]] {
                            mixed = true;
                        }
                    }
                    if mixed {
                        let mut ids: Vec<usize> = g.pred_e[from..to].to_vec();
                        ids.sort_by(|&a, &b| num_cmp(g.etd[a], g.etd[b]));
                        for k in 0..ids.len() {
                            if k > 0 && g.etd[ids[k]] != g.etd[ids[k - 1]] {
                                places += 1;
                            }
                            place[ids[k]] = places;
                        }
                    } else {
                        for k in from..to {
                            place[g.pred_e[k]] = places;
                        }
                    }
                    places += 1;
                }
                south_len = places;
            }
            if south_len < 2 {
                continue;
            }
            let mut t = 0usize;
            for i in ls[r]..ls[r + 1] {
                let v = order[i];
                let from = g.succ_start[v];
                let to = g.succ_start[v + 1];
                let t0 = t;
                for k in from..to {
                    targets[t] = match place.as_ref() {
                        Some(place) => place[g.succ_e[k]],
                        None => g.pos[g.et[g.succ_e[k]]],
                    };
                    t += 1;
                }
                let d = t - t0;
                if d < 2 {
                    continue;
                }
                let mut mixed = false;
                if place.is_some() {
                    for k in from + 1..to {
                        if g.efd[g.succ_e[k]] != g.efd[g.succ_e[from]] {
                            mixed = true;
                        }
                    }
                }
                if let (Some(place), true) = (place.as_ref(), mixed) {
                    let mut ids: Vec<usize> = g.succ_e[from..to].to_vec();
                    ids.sort_by(|&a, &b| num_cmp(g.efd[a], g.efd[b]).then(place[a].cmp(&place[b])));
                    for k in 0..d {
                        targets[t0 + k] = place[ids[k]];
                    }
                } else if d <= 12 {
                    for a in t0 + 1..t {
                        let item = targets[a];
                        let mut b = a as isize - 1;
                        while b >= t0 as isize && targets[b as usize] > item {
                            targets[(b + 1) as usize] = targets[b as usize];
                            b -= 1;
                        }
                        targets[(b + 1) as usize] = item;
                    }
                } else {
                    targets[t0..t].sort();
                }
            }
            if t < 2 {
                continue;
            }
            let mut leaves = 1usize;
            while leaves < south_len {
                leaves <<= 1;
            }
            tree[..2 * leaves].fill(0);
            for &target in targets.iter().take(t) {
                let mut i = target + leaves;
                tree[i] += 1;
                while i > 1 {
                    if (i & 1) == 0 {
                        total += tree[i + 1];
                    }
                    i >>= 1;
                    tree[i] += 1;
                }
            }
        }
        total
    };

    let mut best = count(g, &mut place, &mut tree, &mut targets);
    if best == 0 {
        return;
    }
    let mut best_order = g.order.clone();
    let mut bary = vec![0.0f64; n];
    let mut moving = vec![0usize; widest];

    let sweep = |g: &mut Graph, bary: &mut [f64], moving: &mut [usize], down: bool, bias_right: bool| {
        for step in 1..=max_rank {
            let r = if down { step } else { max_rank - step };
            let s = g.layer_start[r];
            let e = g.layer_start[r + 1];
            let mut movable = 0usize;
            {
                let start = if down { &g.pred_start } else { &g.succ_start };
                let list = if down { &g.pred_e } else { &g.succ_e };
                let other = if down { &g.ef } else { &g.et };
                let other_dx = if down { &g.efd } else { &g.etd };
                for i in s..e {
                    let v = g.order[i];
                    let pin = if v < real { nodes[v].pin } else { 0.0 };
                    if pin != 0.0 {
                        bary[v] = pin * 1e9;
                        moving[movable] = v;
                        movable += 1;
                        continue;
                    }
                    let from = start[v];
                    let to = start[v + 1];
                    if from == to {
                        bary[v] = -1.0;
                        continue;
                    }
                    let mut sum = 0.0f64;
                    for k in from..to {
                        let u = other[list[k]];
                        sum += g.pos[u] as f64;
                        if g.offsets && g.w[u] > 0.0 {
                            sum += other_dx[list[k]] / g.w[u];
                        }
                    }
                    bary[v] = sum / (to - from) as f64;
                    moving[movable] = v;
                    movable += 1;
                }
            }
            if movable < 2 {
                continue;
            }
            {
                let pos = &g.pos;
                let bary: &[f64] = bary;
                if bias_right {
                    moving[..movable].sort_by(|&a, &b| num_cmp(bary[a], bary[b]).then(pos[b].cmp(&pos[a])));
                } else {
                    moving[..movable].sort_by(|&a, &b| num_cmp(bary[a], bary[b]).then(pos[a].cmp(&pos[b])));
                }
            }
            let mut k = 0;
            for i in s..e {
                if bary[g.order[i]] != -1.0 {
                    g.order[i] = moving[k];
                    k += 1;
                }
            }
            for i in s..e {
                let v = g.order[i];
                g.pos[v] = i - s;
            }
        }
    };

    // How many pairs of nodes in one rank are not in the order they were given.
    let out_of_order = |g: &Graph| -> u64 {
        let mut total = 0u64;
        for r in 0..=max_rank {
            for i in g.layer_start[r]..g.layer_start[r + 1] {
                if g.order[i] >= real {
                    continue;
                }
                for j in i + 1..g.layer_start[r + 1] {
                    if g.order[j] < g.order[i] {
                        total += 1;
                    }
                }
            }
        }
        total
    };
    let settle_positions = |g: &mut Graph| set_positions(&g.order, &mut g.pos, &g.layer_start, max_rank);

    // The sweeps are run twice from the same start, going down first and going up first. They
    // often end with as few crossings either way but a different order, and then the one that
    // keeps more of the nodes in the order they were given is the less surprising.
    // Only for graphs small enough to be read node by node: a second run costs as much as the
    // first.
    let initial = g.order.clone();
    let start = best;
    let small = n <= 600;
    let mut chosen = best_order.clone();
    let mut chosen_out = u64::MAX;
    for attempt in 0..(if small { 2 } else { 1 }) {
        if attempt > 0 {
            g.order.copy_from_slice(&initial);
            settle_positions(g);
        }
        let mut found = start;
        let mut found_order = initial.clone();
        let mut i = 0usize;
        let mut stale = 0usize;
        while stale < 4 && i < MAX_SWEEPS {
            sweep(g, &mut bary, &mut moving, (i + attempt) % 2 == 0, i % 4 >= 2);
            let c = count(g, &mut place, &mut tree, &mut targets);
            if c < found {
                found = c;
                found_order.copy_from_slice(&g.order);
                stale = 0;
                if c == 0 {
                    break;
                }
            }
            i += 1;
            stale += 1;
        }
        g.order.copy_from_slice(&found_order);
        let out = if small { out_of_order(g) } else { 0 };
        if attempt == 0 || found < best || (found == best && out < chosen_out) {
            best = found;
            chosen = found_order;
            chosen_out = out;
        }
    }
    g.order.copy_from_slice(&chosen);
    settle_positions(g);
    best_order.copy_from_slice(&chosen);

    // A drawing and its mirror image cross as often too. Of those two, likewise.
    if !small {
        return;
    }
    if nodes.iter().take(real).any(|node| node.pin != 0.0) {
        return;
    }
    if chosen_out == 0 {
        return;
    }
    for r in 0..=max_rank {
        let (s, e) = (g.layer_start[r], g.layer_start[r + 1]);
        g.order[s..e].reverse();
    }
    settle_positions(g);
    // Like Pele, the positions are left as they are here; the caller settles them again.
    if count(g, &mut place, &mut tree, &mut targets) > best || out_of_order(g) >= chosen_out {
        g.order.copy_from_slice(&best_order);
    }
}

/// Half the room a node, bend or port takes across the flow, with its share of the gap beside
/// it. A port on a group's border is a point that an edge passes through, and needs no more
/// than a bend.
fn half_room(g: &Graph, opt: &LayeredOptions, v: usize) -> f64 {
    g.w[v] / 2.0 + (if g.kind[v] == Kind::Node { opt.node_sep } else { opt.edge_sep }) / 2.0
}

/// Brandes-Köpf horizontal coordinate assignment: four aligned layouts, balanced.
fn position(g: &mut Graph, opt: &LayeredOptions) -> Vec<f64> {
    let n = g.n;
    let max_rank = g.max_rank;
    let segments = g.ef.len();
    let mut gap = vec![0.0f64; n];
    for (i, gi) in gap.iter_mut().enumerate() {
        *gi = half_room(g, opt, i);
    }

    // Neighbours in left-to-right order, so the medians can be read off directly.
    let mut key = vec![0i64; segments];
    for e in 0..segments {
        key[e] = g.pos[g.et[e]] as i64;
    }
    for v in 0..n {
        if g.succ_start[v + 1] - g.succ_start[v] > 1 {
            let (from, to) = (g.succ_start[v], g.succ_start[v + 1]);
            sort_range(&mut g.succ_e, from, to, &key);
        }
    }
    for e in 0..segments {
        key[e] = g.pos[g.ef[e]] as i64;
    }
    for v in 0..n {
        if g.pred_start[v + 1] - g.pred_start[v] > 1 {
            let (from, to) = (g.pred_start[v], g.pred_start[v + 1]);
            sort_range(&mut g.pred_e, from, to, &key);
        }
    }
    let g: &Graph = g;
    let ls = &g.layer_start;
    let order = &g.order;
    let pos = &g.pos;

    // An edge between two real nodes may not cross a dummy-to-dummy segment; mark those that
    // would.
    let mut conflict = vec![0u8; segments];
    for r in 1..=max_rank {
        let prev_len = (ls[r] - ls[r - 1]) as i64;
        let s = ls[r];
        let end = ls[r + 1];
        let mut k0: i64 = 0;
        let mut scan = s;
        for i in s..end {
            let v = order[i];
            let mut inner: isize = -1;
            if is_dummy(g.kind[v]) {
                let u = g.ef[g.pred_e[g.pred_start[v]]];
                if is_dummy(g.kind[u]) {
                    inner = u as isize;
                }
            }
            let k1 = if inner >= 0 { pos[inner as usize] as i64 } else { prev_len };
            if inner >= 0 || i == end - 1 {
                for &w in &order[scan..=i] {
                    let w_dummy = is_dummy(g.kind[w]);
                    for k in g.pred_start[w]..g.pred_start[w + 1] {
                        let e = g.pred_e[k];
                        let up = pos[g.ef[e]] as i64;
                        if (up < k0 || k1 < up) && !(w_dummy && is_dummy(g.kind[g.ef[e]])) {
                            conflict[e] = 1;
                        }
                    }
                }
                scan = i + 1;
                k0 = k1;
            }
        }
    }

    let mut results: Vec<Vec<f64>> = Vec::with_capacity(4);
    let mut root = vec![0usize; n];
    let mut align = vec![0usize; n];
    // Offset of each node from its block's root, so that aligned edges meet at their attachment
    // points.
    let mut shift = vec![0.0f64; n];
    let mut e_from = vec![0usize; n];
    let mut e_to = vec![0usize; n];
    let mut e_w = vec![0.0f64; n];
    let mut indeg = vec![0i64; n];
    let mut out_start = vec![0usize; n + 1];
    let mut out_e = vec![0usize; n];
    let mut fill = vec![0usize; n];
    let mut topo = vec![0usize; n];

    for variant in 0..4 {
        let up = variant < 2;
        let right = (variant & 1) == 1;
        let start = if up { &g.pred_start } else { &g.succ_start };
        let list = if up { &g.pred_e } else { &g.succ_e };
        let other = if up { &g.ef } else { &g.et };
        let own_dx = if up { &g.etd } else { &g.efd };
        let other_dx = if up { &g.efd } else { &g.etd };
        for i in 0..n {
            root[i] = i;
            align[i] = i;
            shift[i] = 0.0;
        }

        for step in 0..=max_rank {
            let r = if up { step } else { max_rank - step };
            let s = ls[r];
            let end = ls[r + 1];
            let adjacent: i64 = if up { r as i64 - 1 } else { r as i64 + 1 };
            let adjacent_last: i64 = if adjacent >= 0 && adjacent <= max_rank as i64 {
                ls[adjacent as usize + 1] as i64 - ls[adjacent as usize] as i64 - 1
            } else {
                0
            };
            let mut prev_idx: i64 = -1;
            let count = end - s;
            for c in 0..count {
                let i = if right { end - 1 - c } else { s + c };
                let v = order[i];
                let from = start[v];
                let d = start[v + 1] - from;
                if d == 0 {
                    continue;
                }
                let lo = from + ((d - 1) >> 1);
                let hi = from + (d >> 1);
                for pass in 0..2 {
                    if pass == 1 && lo == hi {
                        break;
                    }
                    let e = list[if (pass == 0) != right { lo } else { hi }];
                    let w = other[e];
                    let idx = if right { adjacent_last - pos[w] as i64 } else { pos[w] as i64 };
                    if align[v] == v && prev_idx < idx && conflict[e] == 0 {
                        align[w] = v;
                        root[v] = root[w];
                        align[v] = root[v];
                        shift[v] = shift[w] + other_dx[e] - own_dx[e];
                        prev_idx = idx;
                    }
                }
            }
        }

        let mut block_edges = 0usize;
        indeg.fill(0);
        out_start.fill(0);
        for r in 0..=max_rank {
            let s = ls[r];
            let end = ls[r + 1];
            for i in s + 1..end {
                let a = if right { order[i] } else { order[i - 1] };
                let b = if right { order[i - 1] } else { order[i] };
                let sep = gap[a] + gap[b];
                e_from[block_edges] = root[a];
                e_to[block_edges] = root[b];
                e_w[block_edges] = if right { sep - shift[a] + shift[b] } else { sep + shift[a] - shift[b] };
                indeg[root[b]] += 1;
                out_start[root[a] + 1] += 1;
                block_edges += 1;
            }
        }
        for i in 0..n {
            out_start[i + 1] += out_start[i];
        }
        fill.copy_from_slice(&out_start[..n]);
        for e in 0..block_edges {
            out_e[fill[e_from[e]]] = e;
            fill[e_from[e]] += 1;
        }

        let mut x = vec![0.0f64; n];
        let mut count = 0usize;
        for i in 0..n {
            if root[i] == i && indeg[i] == 0 {
                topo[count] = i;
                count += 1;
            }
        }
        let mut q = 0;
        while q < count {
            let v = topo[q];
            for k in out_start[v]..out_start[v + 1] {
                let e = out_e[k];
                let w = e_to[e];
                if x[v] + e_w[e] > x[w] {
                    x[w] = x[v] + e_w[e];
                }
                indeg[w] -= 1;
                if indeg[w] == 0 {
                    topo[count] = w;
                    count += 1;
                }
            }
            q += 1;
        }
        for q in (0..count).rev() {
            let v = topo[q];
            let mut min = f64::INFINITY;
            for k in out_start[v]..out_start[v + 1] {
                let e = out_e[k];
                let cand = x[e_to[e]] - e_w[e];
                if cand < min {
                    min = cand;
                }
            }
            if min != f64::INFINITY && min > x[v] {
                x[v] = min;
            }
        }
        let mut xs = vec![0.0f64; n];
        for i in 0..n {
            xs[i] = (if right { -x[root[i]] } else { x[root[i]] }) + shift[i];
        }
        results.push(xs);
    }

    let mut narrowest = 0usize;
    let mut best_width = f64::INFINITY;
    let mut mins: Vec<f64> = Vec::with_capacity(4);
    let mut maxs: Vec<f64> = Vec::with_capacity(4);
    for (k, xs) in results.iter().enumerate() {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for i in 0..n {
            let half = g.w[i] / 2.0;
            if xs[i] - half < lo {
                lo = xs[i] - half;
            }
            if xs[i] + half > hi {
                hi = xs[i] + half;
            }
        }
        mins.push(lo);
        maxs.push(hi);
        if hi - lo < best_width {
            best_width = hi - lo;
            narrowest = k;
        }
    }
    for (k, xs) in results.iter_mut().enumerate() {
        let delta = if (k & 1) == 0 { mins[narrowest] - mins[k] } else { maxs[narrowest] - maxs[k] };
        if delta != 0.0 {
            for x in xs.iter_mut() {
                *x += delta;
            }
        }
    }

    let mut out = vec![0.0f64; n];
    for i in 0..n {
        let mut a = results[0][i];
        let mut b = results[1][i];
        let mut c = results[2][i];
        let mut d = results[3][i];
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        if c > d {
            std::mem::swap(&mut c, &mut d);
        }
        if a > c {
            std::mem::swap(&mut a, &mut c);
        }
        if b > d {
            std::mem::swap(&mut b, &mut d);
        }
        if b > c {
            std::mem::swap(&mut b, &mut c);
        }
        out[i] = (b + c) / 2.0;
    }
    out
}
