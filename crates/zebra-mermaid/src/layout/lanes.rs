//! Layout for swimlanes. Ported from Pele `src/layout/lanes.ts`.
//!
//! Every top-level group is a lane: a band that runs the length of the diagram along the flow.
//! Ranks are shared by all lanes, so a step sits level with the steps that happen at the same
//! time in other lanes, and each node stays inside its own lane.

use super::compound::{CEdge, CNode, CompoundResult, Dir};
use super::layered::{ledge, lnode, orient, rank, Kind, LEdge, LNode, LayeredOptions};
use crate::util::js;

const LANE_HEAD: f64 = 34.0;
pub const LANE_PAD: f64 = 24.0;
const SWEEPS: usize = 8;
const MAX_DUMMIES: i64 = 20000;
const MAX_HOPS: usize = 10000;
const ROUTE_BUDGET: f64 = 2000000.0;
/// Parallel lines closer than this are drawn as one.
const NEAR: f64 = 4.0;
const ROUNDS: usize = 3;
const RELAY: f64 = 8.0;
const TURN: f64 = 20.0;
/// The room a lane keeps beside a handoff along its margin, and how much of it is between the
/// handoff and the lane's items.
const MARGIN: f64 = 20.0;
const LEG: f64 = 14.0;
const CLOSE: f64 = 3.0;

struct Placed {
    width: f64,
    height: f64,
    routed: Vec<u8>,
    margins: Vec<u8>,
    work: f64,
}

#[inline]
fn num_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    let d = a - b;
    if d < 0.0 {
        std::cmp::Ordering::Less
    } else if d > 0.0 {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

/// `laneLayout(nodes, edges, dir, opt, lanePad)`; pass [`LANE_PAD`] for Pele's default pad.
pub fn lane_layout(nodes: &mut [CNode], edges: &mut [CEdge], dir: Dir, opt: &LayeredOptions, lane_pad: f64) -> CompoundResult {
    let n = nodes.len();
    if n == 0 {
        return CompoundResult { width: 0.0, height: 0.0 };
    }
    let sideways = dir == Dir::LR || dir == Dir::RL;
    let flipped = dir == Dir::BT || dir == Dir::RL;

    // Lanes in the order they were written. The graph lists groups last-declared first.
    let mut lanes: Vec<usize> = Vec::new();
    for i in (0..n).rev() {
        if nodes[i].parent < 0 && nodes[i].is_group {
            lanes.push(i);
        }
    }
    let lane_count = lanes.len().max(1);
    let mut lane_slot = vec![-1isize; n];
    for (k, &id) in lanes.iter().enumerate() {
        lane_slot[id] = k as isize;
    }
    let mut has_child = vec![false; n];
    for i in 0..n {
        if nodes[i].parent >= 0 {
            has_child[nodes[i].parent as usize] = true;
        }
    }
    // Room a lane keeps around its items, so that the groups drawn around them stay inside it.
    let mut lane_pads = vec![lane_pad; lane_count];
    let mut end_pad = lane_pad;

    // What gets ranked and placed: nodes, and groups that are empty and so act as nodes.
    let mut item_of = vec![-1isize; n];
    let mut real: Vec<usize> = Vec::new();
    for i in 0..n {
        if lane_slot[i] >= 0 {
            continue;
        }
        if !nodes[i].is_group || !has_child[i] {
            item_of[i] = real.len() as isize;
            real.push(i);
        }
    }
    let along = |c: &CNode| if sideways { c.w } else { c.h };
    let cross = |c: &CNode| if sideways { c.h } else { c.w };

    let mut lnodes: Vec<LNode> = real.iter().map(|&i| lnode(cross(&nodes[i]), along(&nodes[i]), Kind::Node)).collect();
    let mut ledges: Vec<LEdge> = Vec::new();
    let mut edge_of: Vec<usize> = Vec::new();
    let mut step: i32 = 1;
    {
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (ei, e) in edges.iter().enumerate() {
            let a = item_of[e.src];
            let b = item_of[e.dst];
            if a < 0 || b < 0 || a == b {
                continue;
            }
            let (a, b) = (a as usize, b as usize);
            let key = a.min(b) * real.len() + a.max(b);
            if e.label_w > 0.0 || seen.contains(&key) {
                step = 2;
            }
            seen.insert(key);
            ledges.push(ledge(a, b, e.minlen, if sideways { e.label_h } else { e.label_w }, if sideways { e.label_w } else { e.label_h }));
            edge_of.push(ei);
        }
    }
    orient(&lnodes, &mut ledges);
    rank(&mut lnodes, &ledges, step);

    let m = ledges.len();
    let item_lane: Vec<usize> = real
        .iter()
        .map(|&i| {
            // laneOf: the lane of the top-level group above `i`, and the room the groups between
            // keep around it.
            let mut t = i;
            let mut room = lane_pad;
            let mut hops = 0;
            while nodes[t].parent >= 0 && hops < MAX_HOPS {
                t = nodes[t].parent as usize;
                if lane_slot[t] < 0 {
                    room += js::max(js::max(nodes[t].pad_x, nodes[t].pad_top), nodes[t].pad_bottom);
                }
                hops += 1;
            }
            let lane = lane_slot[t].max(0) as usize;
            if room > lane_pads[lane] {
                lane_pads[lane] = room;
            }
            if room > end_pad {
                end_pad = room;
            }
            lane
        })
        .collect();

    let ctx = Ctx {
        sideways,
        flipped,
        lanes: &lanes,
        lane_count,
        lane_slot: &lane_slot,
        item_of: &item_of,
        real: &real,
        lnodes: &lnodes,
        ledges: &ledges,
        edge_of: &edge_of,
        step,
        item_lane: &item_lane,
        lane_pads: &lane_pads,
        end_pad,
        opt,
    };

    let mut plain: Vec<u8> = vec![0; m];
    let mut margins: Vec<u8> = vec![0; lane_count];
    let mut placed = place(&ctx, nodes, edges, &plain, &margins);
    // A handoff that was given a simple route needs no room kept for it through the ranks it
    // passes, so the lanes are laid out again without it. A handoff that then loses its route
    // goes back to the ranked one. Routing is most of the work, so a large graph is laid out
    // once.
    let mut round = 0;
    while round < ROUNDS && placed.work <= ROUTE_BUDGET / RELAY {
        let next: Vec<u8> =
            if round == 0 { placed.routed.clone() } else { plain.iter().zip(&placed.routed).map(|(&on, &r)| on & r).collect() };
        if next == plain && placed.margins == margins {
            break;
        }
        plain = next;
        margins = placed.margins.clone();
        placed = place(&ctx, nodes, edges, &plain, &margins);
        round += 1;
    }
    if plain.iter().zip(&placed.routed).any(|(&on, &r)| on != 0 && r == 0) {
        placed = place(&ctx, nodes, edges, &vec![0; m], &vec![0; lane_count]);
    }
    CompoundResult { width: placed.width, height: placed.height }
}

/// What `place` shares with the rest of `lane_layout`.
struct Ctx<'a> {
    sideways: bool,
    flipped: bool,
    lanes: &'a [usize],
    lane_count: usize,
    lane_slot: &'a [isize],
    item_of: &'a [isize],
    real: &'a [usize],
    lnodes: &'a [LNode],
    ledges: &'a [LEdge],
    edge_of: &'a [usize],
    step: i32,
    item_lane: &'a [usize],
    lane_pads: &'a [f64],
    end_pad: f64,
    opt: &'a LayeredOptions,
}

/// Lays the ranked items out and routes the relations. A plain relation is a handoff that takes
/// a simple route of its own, and so keeps no room through the ranks it passes.
fn place(cx: &Ctx, nodes: &mut [CNode], edges: &mut [CEdge], plain: &[u8], margins: &[u8]) -> Placed {
    let n = nodes.len();
    let sideways = cx.sideways;
    let lanes = cx.lanes;
    let lane_count = cx.lane_count;
    let lane_slot = cx.lane_slot;
    let real = cx.real;
    let lnodes = cx.lnodes;
    let ledges = cx.ledges;
    let opt = cx.opt;
    let step = cx.step;
    let stepf = step as f64;
    let end_pad = cx.end_pad;
    let m = ledges.len();

    let pad: Vec<f64> =
        cx.lane_pads.iter().enumerate().map(|(lane, &room)| if margins[lane] != 0 { js::max(room, MARGIN) } else { room }).collect();
    let mut used = vec![0u8; lane_count];
    // Items: the real ones, then a dummy for every rank an edge passes through.
    let mut dummies: i64 = 0;
    let mut span = vec![0i64; m];
    for k in 0..m {
        span[k] = (lnodes[ledges[k].head].rank as i64 - lnodes[ledges[k].tail].rank as i64).abs();
        if plain[k] == 0 {
            dummies += span[k] - 1;
        }
    }
    let direct = dummies > MAX_DUMMIES;
    let total = (real.len() as i64 + if direct { 0 } else { dummies }).max(real.len() as i64) as usize;
    let mut rank_of = vec![0usize; total];
    let mut lane_of = vec![0usize; total];
    let mut size = vec![0.0f64; total];
    let mut depth = vec![0.0f64; total];
    let mut dummy = vec![false; total];
    let mut max_rank = 0usize;
    for k in 0..real.len() {
        rank_of[k] = lnodes[k].rank as usize;
        lane_of[k] = cx.item_lane[k];
        size[k] = lnodes[k].w;
        depth[k] = lnodes[k].h;
        if rank_of[k] > max_rank {
            max_rank = rank_of[k];
        }
    }
    let mut first_dummy = vec![-1isize; m];
    let mut label_item = vec![-1isize; m];
    let mut links: Vec<usize> = Vec::new();
    {
        let mut next = real.len();
        for k in 0..m {
            let e = &ledges[k];
            let a = if e.reversed { e.head } else { e.tail };
            let b = if e.reversed { e.tail } else { e.head };
            if direct || span[k] < 2 || plain[k] != 0 {
                links.push(a);
                links.push(b);
                continue;
            }
            let ra = rank_of[a] as i64;
            let rb = rank_of[b] as i64;
            let mut label_rank: i64 = -1;
            if e.label_w > 0.0 {
                label_rank = ra + ((rb - ra) >> 1);
                if step == 2 && (label_rank & 1) == 0 {
                    label_rank -= 1;
                }
            }
            first_dummy[k] = next as isize;
            let mut prev = a;
            for r in ra + 1..rb {
                let d = next;
                next += 1;
                rank_of[d] = r as usize;
                // The edge crosses from its first lane to its last one evenly along the way.
                let la = lane_of[a] as f64;
                let lb = lane_of[b] as f64;
                lane_of[d] = (la + js::round(((lb - la) * (r - ra) as f64) / (rb - ra) as f64)) as usize;
                dummy[d] = true;
                if r == label_rank {
                    size[d] = e.label_w;
                    depth[d] = e.label_h;
                    label_item[k] = d as isize;
                }
                links.push(prev);
                links.push(d);
                prev = d;
            }
            links.push(prev);
            links.push(b);
        }
    }

    let mut up_start = vec![0usize; total + 1];
    let mut down_start = vec![0usize; total + 1];
    let mut k = 0;
    while k < links.len() {
        down_start[links[k] + 1] += 1;
        up_start[links[k + 1] + 1] += 1;
        k += 2;
    }
    for i in 0..total {
        up_start[i + 1] += up_start[i];
        down_start[i + 1] += down_start[i];
    }
    let mut up = vec![0usize; links.len() / 2];
    let mut down = vec![0usize; links.len() / 2];
    {
        let mut uf = up_start[..total].to_vec();
        let mut df = down_start[..total].to_vec();
        let mut k = 0;
        while k < links.len() {
            down[df[links[k]]] = links[k + 1];
            df[links[k]] += 1;
            up[uf[links[k + 1]]] = links[k];
            uf[links[k + 1]] += 1;
            k += 2;
        }
    }

    // Order within each rank: by lane first, then by where the item's neighbours sit.
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); max_rank + 1];
    for i in 0..total {
        rows[rank_of[i]].push(i);
    }
    let mut pos = vec![0.0f64; total];
    let mut key = vec![0.0f64; total];
    let settle = |row: &mut Vec<usize>, key: &[f64], pos: &mut [f64]| {
        row.sort_by(|&a, &b| lane_of[a].cmp(&lane_of[b]).then(num_cmp(key[a], key[b])).then(a.cmp(&b)));
        for (k, &v) in row.iter().enumerate() {
            pos[v] = k as f64;
        }
    };
    for row in rows.iter_mut() {
        for (k, &v) in row.iter().enumerate() {
            key[v] = k as f64;
        }
        settle(row, &key, &mut pos);
    }
    let mut sweep = 0;
    while sweep < SWEEPS && max_rank > 0 {
        let downward = (sweep & 1) == 0;
        for s in 1..=max_rank {
            let r = if downward { s } else { max_rank - s };
            let start = if downward { &up_start } else { &down_start };
            let list = if downward { &up } else { &down };
            for &i in &rows[r] {
                let mut sum = 0.0;
                let count = start[i + 1] - start[i];
                for &u in &list[start[i]..start[i + 1]] {
                    sum += pos[u];
                }
                key[i] = if count > 0 { sum / count as f64 } else { pos[i] };
            }
            settle(&mut rows[r], &key, &mut pos);
        }
        sweep += 1;
    }

    // The items of one rank that share a lane sit side by side as a run.
    let gap = |a: usize, b: usize| -> f64 {
        if dummy[a] || dummy[b] {
            opt.edge_sep
        } else {
            opt.node_sep
        }
    };
    let runs = |row: &[usize]| -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut from = 0;
        while from < row.len() {
            let mut to = from;
            while to + 1 < row.len() && lane_of[row[to + 1]] == lane_of[row[from]] {
                to += 1;
            }
            out.push((from, to));
            from = to + 1;
        }
        out
    };
    let run_size = |row: &[usize], from: usize, to: usize| -> f64 {
        let mut s = size[row[from]];
        for k in from..to {
            s += gap(row[k], row[k + 1]) + size[row[k + 1]];
        }
        s
    };

    // Lane breadth: enough for its fullest rank, and for its title.
    let mut breadth = vec![0.0f64; lane_count];
    let mut head = if !lanes.is_empty() { LANE_HEAD } else { 0.0 };
    for k in 0..lanes.len() {
        let lane = &nodes[lanes[k]];
        breadth[k] = js::max(0.0, lane.min_w - 2.0 * pad[k]);
        if lane.pad_top > head {
            head = lane.pad_top;
        }
    }
    for row in &rows {
        for (from, to) in runs(row) {
            let s = run_size(row, from, to);
            if s > breadth[lane_of[row[from]]] {
                breadth[lane_of[row[from]]] = s;
            }
        }
    }
    let mut lane_start = vec![0.0f64; lane_count + 1];
    for k in 0..lane_count {
        lane_start[k + 1] = lane_start[k] + breadth[k] + 2.0 * pad[k];
    }

    // Across the flow: each lane's items start centered, then line up with their neighbours in
    // the same lane as far as the lane and the items beside them allow.
    let mut c_of = vec![0.0f64; total];
    let mut want = vec![0.0f64; total];
    for row in &rows {
        for (from, to) in runs(row) {
            let lane = lane_of[row[from]];
            let mut at = (lane_start[lane] + pad[lane] + (lane_start[lane + 1] - pad[lane]) - run_size(row, from, to)) / 2.0;
            for k in from..=to {
                c_of[row[k]] = at + size[row[k]] / 2.0;
                if k < to {
                    at += size[row[k]] + gap(row[k], row[k + 1]);
                }
            }
        }
    }
    let mut pass = 0;
    while pass < 4 && max_rank > 0 {
        let downward = (pass & 1) == 0;
        for s in 1..=max_rank {
            let row = &rows[if downward { s } else { max_rank - s }];
            let start = if downward { &up_start } else { &down_start };
            let list = if downward { &up } else { &down };
            for &i in row {
                let mut sum = 0.0;
                let mut count = 0usize;
                for &u in &list[start[i]..start[i + 1]] {
                    if lane_of[u] != lane_of[i] {
                        continue;
                    }
                    sum += c_of[u];
                    count += 1;
                }
                want[i] = if count > 0 { sum / count as f64 } else { c_of[i] };
            }
            for (from, to) in runs(row) {
                let lane = lane_of[row[from]];
                let mut floor = lane_start[lane] + pad[lane];
                for k in from..=to {
                    let i = row[k];
                    c_of[i] = js::max(want[i], floor + size[i] / 2.0);
                    if k < to {
                        floor = c_of[i] + size[i] / 2.0 + gap(i, row[k + 1]);
                    }
                }
                let mut ceil = lane_start[lane + 1] - pad[lane];
                for k in (from..=to).rev() {
                    let i = row[k];
                    c_of[i] = js::min(c_of[i], ceil - size[i] / 2.0);
                    if k > from {
                        ceil = c_of[i] - size[i] / 2.0 - gap(row[k - 1], i);
                    }
                }
            }
        }
        pass += 1;
    }

    // Along the flow: one band per rank, after the lane titles.
    let mut band_start = vec![0.0f64; max_rank + 1];
    let mut band_end = vec![0.0f64; max_rank + 1];
    // The room after a rank is for the relations that turn there or are only as long as they
    // were asked to be. Where just plain handoffs and straight lines pass, the ranks close up,
    // as far as the label of a handoff between them leaves it a line to sit on.
    let mut sep = vec![opt.rank_sep / stepf; max_rank + 1];
    if plain.iter().any(|&on| on == 1) {
        let full = opt.rank_sep / stepf;
        sep.fill(full / CLOSE);
        for k in 0..m {
            let e = &ledges[k];
            let a = if e.reversed { e.head } else { e.tail };
            let b = if e.reversed { e.tail } else { e.head };
            if plain[k] != 0 {
                if e.label_h <= 0.0 || span[k] > step as i64 {
                    continue;
                }
                let room = js::min(full, (e.label_h + 2.0 * TURN - js::min(depth[a], depth[b]) / 2.0) / span[k] as f64);
                for r in rank_of[a]..rank_of[b] {
                    if room > sep[r] {
                        sep[r] = room;
                    }
                }
                continue;
            }
            let d0 = first_dummy[k];
            if d0 < 0 || span[k] as f64 <= stepf * e.minlen {
                for r in rank_of[a]..rank_of[b] {
                    sep[r] = full;
                }
                continue;
            }
            let d0 = d0 as usize;
            let last = d0 + span[k] as usize - 1;
            for d in d0 - 1..last {
                let u = if d < d0 { a } else { d };
                let v = if d + 1 < last { d + 1 } else { b };
                if lane_of[u] != lane_of[v] || (c_of[u] - c_of[v]).abs() >= 0.5 {
                    sep[rank_of[u]] = full;
                }
            }
        }
    }
    let mut at = head + end_pad;
    for r in 0..=max_rank {
        let mut d = 0.0f64;
        for &i in &rows[r] {
            if depth[i] > d {
                d = depth[i];
            }
        }
        band_start[r] = at;
        band_end[r] = at + d;
        at += d + if r == max_rank { 0.0 } else { sep[r] };
    }
    let length = at + end_pad;
    let breadth_total = lane_start[lane_count];
    let flipped = cx.flipped;
    let flip = |a: f64| -> f64 {
        if flipped {
            head + length - a
        } else {
            a
        }
    };
    let px = |c: f64, a: f64| -> f64 {
        if sideways {
            flip(a)
        } else {
            c
        }
    };
    let py = |c: f64, a: f64| -> f64 {
        if sideways {
            c
        } else {
            flip(a)
        }
    };
    let axis = if sideways { 1.0 } else { 0.0 };

    for k in 0..real.len() {
        let node = &mut nodes[real[k]];
        let a = (band_start[rank_of[k]] + band_end[rank_of[k]]) / 2.0;
        node.x = px(c_of[k], a);
        node.y = py(c_of[k], a);
    }
    for k in 0..lanes.len() {
        let lane = &mut nodes[lanes[k]];
        let c = (lane_start[k] + lane_start[k + 1]) / 2.0;
        let s = lane_start[k + 1] - lane_start[k];
        lane.x = if sideways { length / 2.0 } else { c };
        lane.y = if sideways { c } else { length / 2.0 };
        lane.w = if sideways { length } else { s };
        lane.h = if sideways { s } else { length };
        lane.pad_top = head;
    }

    // Groups inside a lane are drawn around their members, deepest first.
    let mut depth_of = vec![0usize; n];
    let mut inner: Vec<usize> = Vec::new();
    for i in 0..n {
        if !nodes[i].is_group || lane_slot[i] >= 0 || cx.item_of[i] >= 0 {
            continue;
        }
        let mut d = 0usize;
        let mut p = nodes[i].parent;
        while p >= 0 && d < MAX_HOPS {
            d += 1;
            p = nodes[p as usize].parent;
        }
        depth_of[i] = d;
        inner.push(i);
    }
    inner.sort_by(|&a, &b| depth_of[b].cmp(&depth_of[a]).then(a.cmp(&b)));
    let mut bx = vec![0.0f64; n * 4];
    for i in 0..n {
        bx[i * 4] = f64::INFINITY;
        bx[i * 4 + 1] = f64::INFINITY;
        bx[i * 4 + 2] = f64::NEG_INFINITY;
        bx[i * 4 + 3] = f64::NEG_INFINITY;
    }
    let grow = |bx: &mut [f64], g: usize, c: &CNode| {
        if c.x - c.w / 2.0 < bx[g * 4] {
            bx[g * 4] = c.x - c.w / 2.0;
        }
        if c.y - c.h / 2.0 < bx[g * 4 + 1] {
            bx[g * 4 + 1] = c.y - c.h / 2.0;
        }
        if c.x + c.w / 2.0 > bx[g * 4 + 2] {
            bx[g * 4 + 2] = c.x + c.w / 2.0;
        }
        if c.y + c.h / 2.0 > bx[g * 4 + 3] {
            bx[g * 4 + 3] = c.y + c.h / 2.0;
        }
    };
    for &i in real {
        let p = nodes[i].parent;
        if p >= 0 && lane_slot[p as usize] < 0 {
            grow(&mut bx, p as usize, &nodes[i]);
        }
    }
    for &g in &inner {
        if bx[g * 4] == f64::INFINITY {
            continue;
        }
        let c = &mut nodes[g];
        let w = js::max(bx[g * 4 + 2] - bx[g * 4] + 2.0 * c.pad_x, c.min_w);
        c.x = (bx[g * 4] + bx[g * 4 + 2]) / 2.0;
        c.w = w;
        c.y = (bx[g * 4 + 1] - c.pad_top + bx[g * 4 + 3] + c.pad_bottom) / 2.0;
        c.h = bx[g * 4 + 3] - bx[g * 4 + 1] + c.pad_top + c.pad_bottom;
        let p = c.parent;
        if p >= 0 && lane_slot[p as usize] < 0 {
            let c = nodes[g].clone();
            grow(&mut bx, p as usize, &c);
        }
    }

    for e in edges.iter_mut() {
        e.route = Vec::new();
    }
    for k in 0..m {
        let e = &ledges[k];
        let out = &mut edges[cx.edge_of[k]];
        let a = if e.reversed { e.head } else { e.tail };
        let b = if e.reversed { e.tail } else { e.head };
        let mut pts: Vec<f64> = vec![c_of[a], band_end[rank_of[a]] - (band_end[rank_of[a]] - band_start[rank_of[a]] - depth[a]) / 2.0];
        let d0 = first_dummy[k];
        if d0 >= 0 {
            let d0 = d0 as usize;
            for d in d0..d0 + span[k] as usize - 1 {
                pts.extend_from_slice(&[c_of[d], band_start[rank_of[d]]]);
                if band_end[rank_of[d]] > band_start[rank_of[d]] {
                    pts.extend_from_slice(&[c_of[d], band_end[rank_of[d]]]);
                }
            }
        }
        pts.extend_from_slice(&[c_of[b], band_start[rank_of[b]] + (band_end[rank_of[b]] - band_start[rank_of[b]] - depth[b]) / 2.0]);
        let mut route: Vec<f64> = Vec::with_capacity(pts.len() / 2 * 3);
        let mut p = 0;
        while p < pts.len() {
            route.extend_from_slice(&[px(pts[p], pts[p + 1]), py(pts[p], pts[p + 1]), axis]);
            p += 2;
        }
        if e.reversed {
            let mut i = 0usize;
            let mut j = route.len() - 3;
            while i < j {
                for t in 0..2 {
                    route.swap(i + t, j + t);
                }
                i += 3;
                j -= 3;
            }
        }
        let label = label_item[k];
        if label >= 0 {
            let label = label as usize;
            let mid = (band_start[rank_of[label]] + band_end[rank_of[label]]) / 2.0;
            out.label_x = px(c_of[label], mid);
            out.label_y = py(c_of[label], mid);
        } else {
            out.label_x = (route[0] + route[route.len() - 3]) / 2.0;
            out.label_y = (route[1] + route[route.len() - 2]) / 2.0;
        }
        out.route = route;
    }

    // An edge that starts or ends at a lane or a group runs straight to its border.
    for ei in 0..edges.len() {
        if !edges[ei].route.is_empty() || edges[ei].src == edges[ei].dst {
            continue;
        }
        let a = &nodes[edges[ei].src];
        let b = &nodes[edges[ei].dst];
        let (ax, ay) = border(a, b);
        let (bx2, by2) = border(b, a);
        let flat = if (bx2 - ax).abs() > (by2 - ay).abs() { 1.0 } else { 0.0 };
        let e = &mut edges[ei];
        e.route = vec![ax, ay, flat, bx2, by2, flat];
        e.label_x = (ax + bx2) / 2.0;
        e.label_y = (ay + by2) / 2.0;
    }

    // A handoff can leave towards the other lane, then arrive along the flow (or vice versa).
    // In particular, a decision's return branch must not leave through its incoming side and
    // retrace the path past the preceding step. Try the two single-corner routes, keeping the
    // ranked route when a node or another label blocks them.
    let mut routed = vec![0u8; m];
    let mut route_points = 0.0f64;
    for edge in edges.iter() {
        route_points += edge.route.len() as f64 / 3.0;
    }
    let work = m as f64 * (real.len() as f64 + edges.len() as f64 + route_points);
    if work <= ROUTE_BUDGET {
        let amid = |i: usize| -> f64 { (band_start[rank_of[i]] + band_end[rank_of[i]]) / 2.0 };
        let mut labels: Vec<[f64; 2]> = edges
            .iter()
            .map(|e| [if sideways { e.label_y } else { e.label_x }, flip(if sideways { e.label_x } else { e.label_y })])
            .collect();
        let overlaps = |c0: f64, a0: f64, c1: f64, a1: f64, lc: f64, la: f64, cw: f64, ah: f64| -> bool {
            c1 > lc - cw / 2.0 && c0 < lc + cw / 2.0 && a1 > la - ah / 2.0 && a0 < la + ah / 2.0
        };
        let clear = |edges: &[CEdge], labels: &[[f64; 2]], c0: f64, a0: f64, c1: f64, a1: f64, src: isize, dst: isize, ei: usize| -> bool {
            for i in 0..real.len() {
                if i as isize != src && i as isize != dst && overlaps(c0, a0, c1, a1, c_of[i], amid(i), size[i] + 8.0, depth[i] + 8.0) {
                    return false;
                }
            }
            for (i, e) in edges.iter().enumerate() {
                // A step's own loop keeps its label inside the step's box, and has no place here.
                if i != ei
                    && e.label_w > 0.0
                    && e.src != e.dst
                    && overlaps(
                        c0,
                        a0,
                        c1,
                        a1,
                        labels[i][0],
                        labels[i][1],
                        if sideways { e.label_h } else { e.label_w },
                        if sideways { e.label_w } else { e.label_h },
                    )
                {
                    return false;
                }
            }
            true
        };
        // How many other relations have a stretch inside a box. A stretch's whole box counts,
        // including its curve.
        let crossings = |edges: &[CEdge], x0: f64, y0: f64, x1: f64, y1: f64, ei: usize| -> f64 {
            let mut count = 0.0;
            for (i, edge) in edges.iter().enumerate() {
                if i == ei {
                    continue;
                }
                let route = &edge.route;
                let mut j = 3;
                while j < route.len() {
                    if js::max(route[j - 3], route[j]) > x0
                        && js::min(route[j - 3], route[j]) < x1
                        && js::max(route[j - 2], route[j + 1]) > y0
                        && js::min(route[j - 2], route[j + 1]) < y1
                    {
                        count += 1.0;
                        break;
                    }
                    j += 3;
                }
            }
            count
        };
        let meets = |edges: &[CEdge], x: f64, y: f64, ei: usize| -> f64 {
            let mut count = 0.0;
            for (i, edge) in edges.iter().enumerate() {
                let route = &edge.route;
                if i == ei || route.is_empty() {
                    continue;
                }
                let last = route.len() - 3;
                if ((route[0] - x).abs() < 1.0 && (route[1] - y).abs() < 1.0)
                    || ((route[last] - x).abs() < 1.0 && (route[last + 1] - y).abs() < 1.0)
                {
                    count += 1.0;
                }
            }
            count
        };
        // Two relations drawn along one line read as a single path. Those that leave the same
        // step or arrive at the same step may share a trunk; any other pair must not.
        let retraces = |edges: &[CEdge], x0: f64, y0: f64, x1: f64, y1: f64, ei: usize| -> bool {
            let upright = (x1 - x0).abs() < 0.5;
            let lo = if upright { js::min(y0, y1) } else { js::min(x0, x1) };
            let hi = if upright { js::max(y0, y1) } else { js::max(x0, x1) };
            let line = if upright { x0 } else { y0 };
            for (i, other) in edges.iter().enumerate() {
                if i == ei || other.src == edges[ei].src || other.dst == edges[ei].dst {
                    continue;
                }
                let route = &other.route;
                let mut j = 3;
                while j < route.len() {
                    let (o0, o1) = if upright { (0, 1) } else { (1, 0) };
                    let u0 = route[j - 3 + o0];
                    let u1 = route[j + o0];
                    if (u0 - line).abs() >= NEAR || (u1 - line).abs() >= NEAR {
                        j += 3;
                        continue;
                    }
                    let v0 = route[j - 3 + o1];
                    let v1 = route[j + o1];
                    if js::min(hi, js::max(v0, v1)) - js::max(lo, js::min(v0, v1)) > 1.0 {
                        return true;
                    }
                    j += 3;
                }
            }
            false
        };
        for k in 0..m {
            let e = &ledges[k];
            let ei = cx.edge_of[k];
            let a = e.tail;
            let b = e.head;
            if lane_of[a] == lane_of[b] || (c_of[b] - c_of[a]).abs() < 1.0 {
                continue;
            }
            let ac = c_of[a];
            let bc = c_of[b];
            let aa = amid(a);
            let ba = amid(b);
            let dc = js::sign(bc - ac);
            let da = js::sign(ba - aa);
            let mut attempt = |edges: &mut [CEdge], labels: &mut Vec<[f64; 2]>, pts: &[f64], margin: f64| -> bool {
                let mut p = 2;
                while p < pts.len() {
                    if !clear(
                        edges,
                        labels,
                        js::min(pts[p - 2], pts[p]) - margin,
                        js::min(pts[p - 1], pts[p + 1]) - margin,
                        js::max(pts[p - 2], pts[p]) + margin,
                        js::max(pts[p - 1], pts[p + 1]) + margin,
                        a as isize,
                        b as isize,
                        ei,
                    ) {
                        return false;
                    }
                    if retraces(
                        edges,
                        px(pts[p - 2], pts[p - 1]),
                        py(pts[p - 2], pts[p - 1]),
                        px(pts[p], pts[p + 1]),
                        py(pts[p], pts[p + 1]),
                        ei,
                    ) {
                        return false;
                    }
                    p += 2;
                }
                let mut lc = 0.0;
                let mut la = 0.0;
                let mut longest = -1.0;
                let (out_label_w, out_label_h) = (edges[ei].label_w, edges[ei].label_h);
                if out_label_w > 0.0 {
                    let cw = e.label_w;
                    let ah = e.label_h;
                    let mut p = 2;
                    while p < pts.len() {
                        let cc = (pts[p - 2] + pts[p]) / 2.0;
                        let ca = (pts[p - 1] + pts[p + 1]) / 2.0;
                        let across = (pts[p] - pts[p - 2]).abs();
                        let along = (pts[p + 1] - pts[p - 1]).abs();
                        let run = across + along;
                        p += 2;
                        if run <= longest || (if across > 0.0 { across < cw + 8.0 } else { along < ah + 8.0 }) {
                            continue;
                        }
                        if cc - cw / 2.0 < 0.0 || cc + cw / 2.0 > breadth_total || ca - ah / 2.0 < 0.0 || ca + ah / 2.0 > length {
                            continue;
                        }
                        // Include the ends here: the label must clear both shapes it connects.
                        if !clear(edges, labels, cc - cw / 2.0, ca - ah / 2.0, cc + cw / 2.0, ca + ah / 2.0, -1, -1, ei) {
                            continue;
                        }
                        // A label moved onto a handoff must not cover a return path that keeps
                        // its ranked route. A segment's whole box is kept clear, including its
                        // curve.
                        let lx = px(cc, ca);
                        let ly = py(cc, ca);
                        if crossings(
                            edges,
                            lx - out_label_w / 2.0,
                            ly - out_label_h / 2.0,
                            lx + out_label_w / 2.0,
                            ly + out_label_h / 2.0,
                            ei,
                        ) > 0.0
                        {
                            continue;
                        }
                        lc = cc;
                        la = ca;
                        longest = run;
                    }
                    if longest < 0.0 {
                        return false;
                    }
                }
                // A point short of each corner and one past it keep the turn close to the
                // corner, so that the line stays under its label and the handoff takes little
                // room.
                let mut way: Vec<f64> = vec![pts[0], pts[1]];
                let mut p = 2;
                while p < pts.len() {
                    let dx = pts[p] - pts[p - 2];
                    let dy = pts[p + 1] - pts[p - 1];
                    let run = dx.abs() + dy.abs();
                    let first = if p > 2 { TURN } else { 0.0 };
                    let last = if p + 2 < pts.len() { TURN } else { 0.0 };
                    if run > first + last + 1.0 {
                        if first != 0.0 {
                            way.extend_from_slice(&[pts[p - 2] + (dx / run) * first, pts[p - 1] + (dy / run) * first]);
                        }
                        if last != 0.0 {
                            way.extend_from_slice(&[pts[p] - (dx / run) * last, pts[p + 1] - (dy / run) * last]);
                        }
                    }
                    way.extend_from_slice(&[pts[p], pts[p + 1]]);
                    p += 2;
                }
                let mut route: Vec<f64> = Vec::with_capacity(way.len() / 2 * 3);
                let mut p = 0;
                while p < way.len() {
                    let crosswise = if p == 0 { way[0] != way[2] } else { way[p] != way[p - 2] };
                    route.extend_from_slice(&[px(way[p], way[p + 1]), py(way[p], way[p + 1]), if crosswise { 1.0 - axis } else { axis }]);
                    p += 2;
                }
                let out = &mut edges[ei];
                out.route = route;
                if out.label_w > 0.0 {
                    out.label_x = px(lc, la);
                    out.label_y = py(lc, la);
                    labels[ei] = [lc, la];
                }
                routed[k] = 1;
                true
            };
            if attempt(edges, &mut labels, &[ac + dc * size[a] / 2.0, aa, bc, aa, bc, ba - da * depth[b] / 2.0], 2.0) {
                continue;
            }
            if attempt(edges, &mut labels, &[ac, aa + da * depth[a] / 2.0, ac, ba, bc - dc * size[b] / 2.0, ba], 2.0) {
                continue;
            }
            // With both corners blocked, as for a return past the steps between, the handoff
            // goes around: out to the margin of a lane, along it, and back in level with its
            // end. It takes the margin that crosses the fewest other relations and shares the
            // fewest ends with them, then the shortest way.
            if work > ROUTE_BUDGET / RELAY {
                continue;
            }
            struct Around {
                pts: [f64; 8],
                cost: f64,
                lane: usize,
            }
            let mut around: Vec<Around> = Vec::new();
            let pair = if lane_of[a] < lane_of[b] { [lane_of[b], lane_of[a]] } else { [lane_of[a], lane_of[b]] };
            for lane in pair {
                // A lane that keeps no margin yet has less room there, and the handoff stays
                // inside it.
                let inset = js::max(pad[lane] - LEG, js::min(pad[lane] / 2.0, MARGIN - LEG));
                for ch in [lane_start[lane] + inset, lane_start[lane + 1] - inset] {
                    if (ch - ac).abs() < size[a] / 2.0 + 4.0 || (ch - bc).abs() < size[b] / 2.0 + 4.0 {
                        continue;
                    }
                    let pts = [ac + js::sign(ch - ac) * size[a] / 2.0, aa, ch, aa, ch, ba, bc + js::sign(ch - bc) * size[b] / 2.0, ba];
                    let mut cost = (meets(edges, px(pts[0], pts[1]), py(pts[0], pts[1]), ei)
                        + meets(edges, px(pts[6], pts[7]), py(pts[6], pts[7]), ei))
                        * 1e6;
                    let mut p = 2;
                    while p < pts.len() {
                        let x0 = px(pts[p - 2], pts[p - 1]);
                        let y0 = py(pts[p - 2], pts[p - 1]);
                        let x1 = px(pts[p], pts[p + 1]);
                        let y1 = py(pts[p], pts[p + 1]);
                        cost += crossings(edges, js::min(x0, x1), js::min(y0, y1), js::max(x0, x1), js::max(y0, y1), ei) * 1e6
                            + (x1 - x0).abs()
                            + (y1 - y0).abs();
                        p += 2;
                    }
                    around.push(Around { pts, cost, lane });
                }
            }
            around.sort_by(|p, q| num_cmp(p.cost, q.cost));
            for way in &around {
                if !attempt(edges, &mut labels, &way.pts, 1.0) {
                    continue;
                }
                used[way.lane] = 1;
                break;
            }
        }
    }

    Placed {
        width: if sideways { length } else { breadth_total },
        height: if sideways { breadth_total } else { length },
        routed,
        margins: used,
        work,
    }
}

/// The point where the line from one box's center to another's leaves the first box. When the
/// other center is inside the box, the point is on the nearest side instead.
fn border(from: &CNode, to: &CNode) -> (f64, f64) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let hw = js::max(from.w / 2.0, 0.5);
    let hh = js::max(from.h / 2.0, 0.5);
    if dx.abs() < hw && dy.abs() < hh {
        let to_side = hw - dx.abs();
        let to_end = hh - dy.abs();
        return if to_side < to_end {
            (from.x + if dx < 0.0 { -hw } else { hw }, to.y)
        } else {
            (to.x, from.y + if dy < 0.0 { -hh } else { hh })
        };
    }
    let scale = js::min(if dx != 0.0 { hw / dx.abs() } else { f64::INFINITY }, if dy != 0.0 { hh / dy.abs() } else { f64::INFINITY });
    (from.x + dx * scale, from.y + dy * scale)
}
