//! Parallel labels and grid expansion from Pele 1.12.0 block/render.ts (MIT).
use super::*;
use crate::text::Measure;
use indexmap::IndexMap;

pub(super) struct Placement {
    pub width: f64,
    pub height: f64,
    pub obstacles: Vec<(usize, Shape)>,
    pub titles: Vec<(usize, Shape)>,
}
pub(super) fn shape_at(cells: &[Cell], views: &[View], k: usize) -> Shape {
    Shape {
        x: cells[k].x,
        y: cells[k].y,
        bw: views[k].bw,
        bh: views[k].bh,
        kind: views[k].kind,
        shape: views[k].shape,
        mask: views[k].mask,
    }
}
pub(super) fn place(cells: &mut [Cell], views: &mut [View], widths: &[f64], gap: f64) -> Result<Placement, Error> {
    for (i, c) in cells.iter_mut().enumerate() {
        c.nw = widths[i];
    }
    let (width, height) = layout(cells, gap)?;
    let (mut obstacles, mut titles) = (Vec::new(), Vec::new());
    for (k, v) in views.iter_mut().enumerate() {
        let c = &cells[k];
        if v.kind == Kind::Leaf && fixed(v.shape) {
            v.bw = c.nw;
            v.bh = c.nh;
        } else if v.kind == Kind::Arrow {
            v.bw = if c.span > 1.0 { c.w } else { c.nw };
            v.bh = c.nh;
        } else {
            v.bw = c.w;
            v.bh = if v.kind == Kind::Leaf && wide(v.shape) { c.nh } else { c.h };
        }
        if v.kind == Kind::Leaf && v.shape == "cyl" {
            v.dy = js::min(12.0, js::max(6.0, v.bw / 14.0)) / 2.0;
        }
        if matches!(v.kind, Kind::Leaf | Kind::Arrow) {
            obstacles.push((k, Shape { x: c.x, y: c.y, bw: v.bw, bh: v.bh, kind: v.kind, shape: v.shape, mask: v.mask }));
        } else if c.head > 0.0 {
            titles.push((
                usize::MAX,
                Shape {
                    x: c.x,
                    y: c.y - c.h / 2.0 + (c.head + gap) / 2.0 + 2.0,
                    bw: v.label.width,
                    bh: v.label.height,
                    kind: Kind::Composite,
                    shape: "",
                    mask: 0,
                },
            ));
        }
    }
    Ok(Placement { width, height, obstacles, titles })
}
fn covers(x: f64, y: f64, w: f64, h: f64, s: usize, t: usize, boxes: Option<&[(usize, Shape)]>) -> bool {
    boxes.is_some_and(|boxes| {
        boxes.iter().any(|(i, v)| *i != s && *i != t && (x - v.x).abs() < (w + v.bw) / 2.0 && (y - v.y).abs() < (h + v.bh) / 2.0)
    })
}
fn stack_labels(
    stack: &[usize],
    routes: &mut [Route],
    legs: &[Option<[f64; 4]>],
    s: usize,
    t: usize,
    boxes: Option<&[(usize, Shape)]>,
) -> f64 {
    let first = legs[stack[0]].unwrap();
    let vertical = first[0] == first[2];
    let (mut lo, mut hi, mut need, mut crowded) = (f64::NEG_INFINITY, f64::INFINITY, -4.0, false);
    for (a, &i) in stack.iter().enumerate() {
        let leg = legs[i].unwrap();
        if if vertical { leg[0] != leg[2] } else { leg[1] != leg[3] } {
            return 0.0;
        }
        lo = js::max(lo, if vertical { js::min(leg[1], leg[3]) } else { js::min(leg[0], leg[2]) });
        hi = js::min(hi, if vertical { js::max(leg[1], leg[3]) } else { js::max(leg[0], leg[2]) });
        let p = &routes[i];
        need += (if vertical { p.label.height } else { p.label.width + 8.0 }) + 4.0;
        for &j in &stack[..a] {
            let q = &routes[j];
            if (p.lx - q.lx).abs() < (p.label.width + q.label.width) / 2.0 + 8.0
                && (p.ly - q.ly).abs() < (p.label.height + q.label.height) / 2.0
            {
                crowded = true;
            }
        }
    }
    if !crowded {
        return 0.0;
    }
    if need > hi - lo - 16.0 {
        return need - (hi - lo - 16.0);
    }
    let mut along = Vec::new();
    let mut next = (lo + hi - need) / 2.0;
    for &i in stack {
        let p = &routes[i];
        let size = if vertical { p.label.height } else { p.label.width + 8.0 };
        let at = next + size / 2.0;
        if covers(if vertical { p.lx } else { at }, if vertical { at } else { p.ly }, p.label.width + 8.0, p.label.height, s, t, boxes) {
            return 0.0;
        }
        along.push(at);
        next += size + 4.0;
    }
    for (&i, at) in stack.iter().zip(along) {
        if vertical {
            routes[i].ly = at;
        } else {
            routes[i].lx = at;
        }
    }
    0.0
}
fn widen(
    cells: &mut [Cell],
    parents: &[Option<usize>],
    s: usize,
    t: usize,
    vertical: bool,
    short: f64,
    base: &mut HashMap<(usize, bool), Vec<f64>>,
) -> bool {
    let mut chain = HashSet::new();
    let mut v = Some(s);
    while let Some(i) = v {
        chain.insert(i);
        v = parents[i];
    }
    let (mut tb, mut bx) = (None, Some(t));
    while let Some(i) = bx {
        if chain.contains(&i) {
            break;
        }
        tb = bx;
        bx = parents[i];
    }
    let (Some(bx), Some(tb)) = (bx, tb) else { return false };
    if bx == s {
        return false;
    }
    let mut sb = s;
    while parents[sb] != Some(bx) {
        sb = parents[sb].unwrap();
    }
    let at = if vertical {
        if cells[sb].row == cells[tb].row {
            return false;
        }
        (js::max(cells[sb].row, cells[tb].row) - 1.0) as usize
    } else {
        let (l, r) = if cells[sb].col < cells[tb].col { (sb, tb) } else { (tb, sb) };
        if cells[r].col < cells[l].col + cells[l].span {
            return false;
        }
        (cells[r].col - 1.0) as usize
    };
    let gaps = if vertical { &mut cells[bx].row_gaps } else { &mut cells[bx].col_gaps };
    let before = base.entry((bx, vertical)).or_insert_with(|| gaps.clone()).get(at).copied().unwrap_or(0.0);
    if gaps.len() <= at {
        gaps.resize(at + 1, 0.0);
    }
    gaps[at] = js::max(gaps[at], before + short);
    true
}
pub(super) struct Routing<'a, 'm> {
    pub model: &'a BlockModel,
    pub measure: &'a Measure<'m>,
    pub drawn: &'a [usize],
    pub ends: &'a [(usize, usize)],
    pub pairs: &'a HashMap<usize, usize>,
    pub labelled: &'a HashSet<usize>,
    pub parents: &'a [Option<usize>],
    pub edge_size: f64,
    pub tight: bool,
    pub check: bool,
}
impl Routing<'_, '_> {
    pub fn settle(&self, cells: &mut [Cell], views: &[View], placement: &Placement, grow: bool) -> (Vec<Route>, bool) {
        let check = self.check.then_some(placement.obstacles.as_slice());
        let (mut seen, mut stacks) = (HashMap::new(), IndexMap::<usize, Vec<usize>>::new());
        let mut routes = Vec::new();
        let mut legs = vec![None; self.drawn.len()];
        for (i, &edge) in self.drawn.iter().enumerate() {
            let (s, t) = self.ends[i];
            let key = s.min(t) * views.len() + s.max(t);
            let total = self.pairs.get(&key).copied().unwrap_or(1);
            let k = seen.get(&key).copied().unwrap_or(0usize);
            seen.insert(key, k + 1);
            let label = layout_label(self.model.blocks[edge].label.as_deref(), false, self.measure, self.edge_size, WRAP, 0);
            let (points, lx, ly);
            if s == t {
                let (px, py, r, _, _) = loop_arc(&shape_at(cells, views, s), k + 1);
                points = Vec::new();
                lx = px + r + 6.0 + label.width / 2.0;
                ly = py - r / 2.0;
            } else {
                let step = if self.labelled.contains(&key) { js::round(self.edge_size * 1.5) + 2.0 } else { 14.0 };
                points = route_edge(
                    s,
                    t,
                    &shape_at(cells, views, s),
                    &shape_at(cells, views, t),
                    (k as f64 - (total as f64 - 1.0) / 2.0) * step,
                    check,
                );
                let n = points.len();
                let first = (points[2] - points[0]).abs() + (points[3] - points[1]).abs();
                let last = (points[n - 2] - points[n - 4]).abs() + (points[n - 1] - points[n - 3]).abs();
                let at = if n == 6 && last > first { 2 } else { 0 };
                let spot = if label.width > 0.0 { label_spot(&points, at, label.width + 8.0, label.height, s, t, check) } else { 0.5 };
                lx = points[at] + (points[at + 2] - points[at]) * spot;
                ly = points[at + 1] + (points[at + 3] - points[at + 1]) * spot;
                if label.width > 0.0 && total > 1 {
                    legs[i] = Some(points[at..at + 4].try_into().unwrap());
                    stacks.entry(key).or_default().push(i);
                }
            }
            routes.push(Route { edge, points, loop_k: if s == t { k + 1 } else { 0 }, label, lx, ly });
        }
        let blocked: Vec<_> = if self.check { placement.obstacles.iter().chain(&placement.titles).copied().collect() } else { Vec::new() };
        let boxes = self.check.then_some(blocked.as_slice());
        let mut base = HashMap::new();
        let mut grown = false;
        for stack in stacks.values() {
            if stack.len() < 2 {
                continue;
            }
            let (s, t) = self.ends[stack[0]];
            let short = stack_labels(stack, &mut routes, &legs, s, t, boxes);
            let leg = legs[stack[0]].unwrap();
            let vertical = leg[0] == leg[2];
            if grow && short > 0.0 && (vertical || !self.tight) && widen(cells, self.parents, s, t, vertical, short.ceil(), &mut base) {
                grown = true;
            }
        }
        (routes, grown)
    }
}
pub(super) fn overlap(routes: &[Route]) -> f64 {
    let mut area = 0.0;
    for (i, p) in routes.iter().enumerate() {
        for q in &routes[i + 1..] {
            let pw = (p.label.width + 8.0) / 2.0;
            let qw = (q.label.width + 8.0) / 2.0;
            let w = js::min(p.lx + pw, q.lx + qw) - js::max(p.lx - pw, q.lx - qw);
            let h = js::min(p.ly + p.label.height / 2.0, q.ly + q.label.height / 2.0)
                - js::max(p.ly - p.label.height / 2.0, q.ly - q.label.height / 2.0);
            if p.label.width > 0.0 && q.label.width > 0.0 && w > 0.0 && h > 0.0 {
                area += w * h;
            }
        }
    }
    area
}
pub(super) fn bound(placement: &Placement, cells: &[Cell], views: &[View], routes: &[Route], ends: &[(usize, usize)]) -> [f64; 4] {
    let mut b = [0.0, 0.0, placement.width, placement.height];
    let mut grow = |x: f64, y: f64, w: f64, h: f64| {
        if x - w / 2.0 < b[0] {
            b[0] = x - w / 2.0;
        }
        if y - h / 2.0 < b[1] {
            b[1] = y - h / 2.0;
        }
        if x + w / 2.0 > b[2] {
            b[2] = x + w / 2.0;
        }
        if y + h / 2.0 > b[3] {
            b[3] = y + h / 2.0;
        }
    };
    for (i, r) in routes.iter().enumerate() {
        if r.loop_k > 0 {
            let (x, y, k, _, _) = loop_arc(&shape_at(cells, views, ends[i].0), r.loop_k);
            grow(x, y, 2.0 * k + 2.0, 2.0 * k + 2.0);
        }
        if r.label.width > 0.0 {
            grow(r.lx, r.ly, r.label.width + 8.0, r.label.height);
        }
    }
    b
}
