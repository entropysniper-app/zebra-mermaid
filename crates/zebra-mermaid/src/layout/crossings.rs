//! Crossings of the rendered routes, including curves, shared endpoints and group ports.
//! Ported from Pele 1.9.0 `src/layout/compound.ts`.
use super::compound::{CEdge, CNode};
use super::layered::{ease, LayeredOptions};
use crate::util::{js, math};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Curved,
    Stepped,
    Linear,
}
pub fn style_of(opt: &LayeredOptions) -> Style {
    if opt.tracks {
        Style::Stepped
    } else if opt.straight {
        Style::Linear
    } else {
        Style::Curved
    }
}
const IN_LINE: f64 = 0.5;
const FAN: f64 = 24.0;
const HANDLE: f64 = 0.62;
#[derive(Clone, Copy)]
struct Segment {
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    low: f64,
    high: f64,
    owner: usize,
    ends: u8,
}
#[derive(Default)]
struct Segments {
    all: Vec<Segment>,
    from: (f64, f64),
    to: (f64, f64),
}
impl Segments {
    fn add(&mut self, owner: usize, ends: u8, mut x0: f64, mut y0: f64, x1: f64, y1: f64) {
        let (fx, fy) = self.from;
        let on = x0 == self.to.0 && y0 == self.to.1 && ((y1 == y0 && fy == y0) || (x1 == x0 && fx == x0));
        if on && self.all.last().is_some_and(|p| p.owner == owner && p.ends == ends) && (x0 - fx) * (x1 - x0) + (y0 - fy) * (y1 - y0) > 0.0
        {
            self.all.pop();
            x0 = fx;
            y0 = fy;
        }
        self.from = (x0, y0);
        self.to = (x1, y1);
        self.all.push(Segment {
            ax: js::min(x0, x1),
            bx: js::max(x0, x1),
            ay: if x0 <= x1 { y0 } else { y1 },
            by: if x0 <= x1 { y1 } else { y0 },
            low: js::min(y0, y1),
            high: js::max(y0, y1),
            owner,
            ends,
        });
    }
}

pub fn route_crossings(nodes: &[CNode], edges: &[CEdge], style: Style) -> usize {
    let capacity: usize = edges.iter().filter(|e| e.src != e.dst && e.route.len() > 3).map(|e| (e.route.len() / 3 - 1) * 3).sum();
    let mut segs = Segments { all: Vec::with_capacity(capacity), from: (f64::NAN, f64::NAN), to: (f64::NAN, f64::NAN) };
    for (ei, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let r = &e.route;
        let aligned = |k: usize| (r[k] - r[k - 3]).abs() < 0.01 || (r[k + 1] - r[k - 2]).abs() < 0.01;
        let length = |k: usize| (r[k] - r[k - 3]).abs() + (r[k + 1] - r[k - 2]).abs();
        let turns = |k: usize| {
            style == Style::Curved
                && k > 0
                && k < r.len() - 3
                && aligned(k)
                && aligned(k + 3)
                && length(k) >= 0.01
                && length(k + 3) >= 0.01
                && ((r[k] - r[k - 3]).abs() < 0.01) != ((r[k + 3] - r[k]).abs() < 0.01)
        };
        let (mut at_x, mut at_y) = (r[0], r[1]);
        for k in (3..r.len()).step_by(3) {
            let part = u8::from(k == 3) | (u8::from(k == r.len() - 3) << 1);
            let (x0, y0, x1, y1) = (at_x, at_y, r[k], r[k + 1]);
            at_x = x1;
            at_y = y1;
            if aligned(k) || style == Style::Linear {
                if !turns(k) {
                    segs.add(ei, part, x0, y0, x1, y1);
                    continue;
                }
                let before = if turns(k - 3) { length(k) / 2.0 } else { length(k) };
                let after = if turns(k + 3) { length(k + 3) / 2.0 } else { length(k + 3) };
                let (ux, uy) = ((x1 - r[k - 3]) / length(k), (y1 - r[k - 2]) / length(k));
                let (vx, vy) = ((r[k + 3] - x1) / length(k + 3), (r[k + 4] - y1) / length(k + 3));
                let (sx, sy) = (x1 - ux * before, y1 - uy * before);
                segs.add(ei, part, x0, y0, sx, sy);
                let (c1x, c1y) = (x1 - ux * before * (1.0 - HANDLE), y1 - uy * before * (1.0 - HANDLE));
                let (c2x, c2y) = (x1 + vx * after * (1.0 - HANDLE), y1 + vy * after * (1.0 - HANDLE));
                at_x = x1 + vx * after;
                at_y = y1 + vy * after;
                let turn = part | (u8::from(k + 3 == r.len() - 3) << 1);
                let (mut px, mut py) = (sx, sy);
                for i in 1..=2 {
                    let t = i as f64 / 2.0;
                    let u = 1.0 - t;
                    let x = u * u * u * sx + 3.0 * u * u * t * c1x + 3.0 * u * t * t * c2x + t * t * t * at_x;
                    let y = u * u * u * sy + 3.0 * u * u * t * c1y + 3.0 * u * t * t * c2y + t * t * t * at_y;
                    segs.add(ei, turn, px, py, x, y);
                    px = x;
                    py = y;
                }
                continue;
            }
            let down = r[k + 2] == 0.0;
            if style == Style::Stepped {
                let mx = if down { x0 } else { (x0 + x1) / 2.0 };
                let my = if down { (y0 + y1) / 2.0 } else { y0 };
                segs.add(ei, part, x0, y0, mx, my);
                segs.add(ei, part, mx, my, if down { x1 } else { mx }, if down { my } else { y1 });
                segs.add(ei, part, if down { x1 } else { mx }, if down { my } else { y1 }, x1, y1);
                continue;
            }
            let c = if down { ease(x1 - x0, y1 - y0) } else { ease(y1 - y0, x1 - x0) };
            let (mut px, mut py) = (x0, y0);
            for i in 1..=3 {
                let t = i as f64 / 3.0;
                let u = 1.0 - t;
                let across = t * t * (3.0 - 2.0 * t);
                let along = 3.0 * c * t * u * u + 3.0 * (1.0 - c) * t * t * u + t * t * t;
                let x = x0 + (x1 - x0) * if down { across } else { along };
                let y = y0 + (y1 - y0) * if down { along } else { across };
                segs.add(ei, part, px, py, x, y);
                px = x;
                py = y;
            }
        }
    }
    let all = &segs.all;
    if all.is_empty() {
        return 0;
    }
    let top = all.iter().map(|p| p.low).fold(f64::INFINITY, js::min);
    let bottom = all.iter().map(|p| p.high).fold(f64::NEG_INFINITY, js::max);
    let bands = if bottom > top { js::round((all.len() as f64).sqrt()) as usize } else { 1 };
    let scale = if bottom > top { bands as f64 / (bottom - top) } else { 0.0 };
    let band_of = |y: f64| bands.saturating_sub(1).min(((y - top) * scale).floor() as usize);
    let mut first = vec![0; all.len()];
    let mut in_band = vec![Vec::new(); bands];
    for (p, s) in all.iter().enumerate() {
        first[p] = band_of(s.low);
        for b in first[p]..=band_of(s.high) {
            in_band[b].push(p);
        }
    }
    let mut runs = vec![[0.0; 8]; edges.len()];
    let in_line = |r: &[f64], a: usize, b: usize, c: usize| {
        ((r[a] - r[b]).abs() < IN_LINE && (r[a] - r[c]).abs() < IN_LINE)
            || ((r[a + 1] - r[b + 1]).abs() < IN_LINE && (r[a + 1] - r[c + 1]).abs() < IN_LINE)
    };
    for (ei, e) in edges.iter().enumerate() {
        let r = &e.route;
        if r.len() < 6 {
            continue;
        }
        let mut k = 3;
        while k + 3 < r.len() && in_line(r, 0, k, k + 3) {
            k += 3;
        }
        let last = r.len() - 3;
        let mut j = last - 3;
        while j >= 3 && in_line(r, last, j, j - 3) {
            j -= 3;
        }
        runs[ei] = [r[0], r[1], r[k], r[k + 1], r[j], r[j + 1], r[last], r[last + 1]];
    }
    let on_run = |ei: usize, end: u8, x: f64, y: f64| {
        let o = if end == 1 { 0 } else { 4 };
        let r = runs[ei];
        let (x0, y0, dx, dy) = (r[o], r[o + 1], r[o + 2] - r[o], r[o + 3] - r[o + 1]);
        let length = dx * dx + dy * dy;
        let t = if length > 0.0 { js::max(0.0, js::min(1.0, ((x - x0) * dx + (y - y0) * dy) / length)) } else { 0.0 };
        math::hypot(x - x0 - t * dx, y - y0 - t * dy) < IN_LINE
    };
    let mut total = 0;
    for (band, list) in in_band.iter_mut().enumerate() {
        list.sort_by(|&p, &q| all[p].ax.partial_cmp(&all[q].ax).unwrap_or(std::cmp::Ordering::Equal).then(p.cmp(&q)));
        for s in 0..list.len() {
            let pi = list[s];
            let p = all[pi];
            let (dx, dy) = (p.bx - p.ax, p.by - p.ay);
            let here = first[pi] == band;
            for &qi in &list[s + 1..] {
                let q = all[qi];
                if q.ax > p.bx {
                    break;
                }
                if q.high < p.low || q.low > p.high || p.owner == q.owner || (!here && first[qi] != band) {
                    continue;
                }
                let d1 = dx * (q.ay - p.ay) - dy * (q.ax - p.ax);
                let d2 = dx * (q.by - p.ay) - dy * (q.bx - p.ax);
                if (d1 > 0.0) == (d2 > 0.0) {
                    continue;
                }
                let (ex, ey) = (q.bx - q.ax, q.by - q.ay);
                let d3 = ex * (p.ay - q.ay) - ey * (p.ax - q.ax);
                let d4 = ex * (p.by - q.ay) - ey * (p.bx - q.ax);
                if (d3 > 0.0) == (d4 > 0.0) {
                    continue;
                }
                let (ep, eq) = (&edges[p.owner], &edges[q.owner]);
                let at = d3 / (d3 - d4);
                let (x, y) = (p.ax + at * dx, p.ay + at * dy);
                let fan = |v: usize, pe: u8, qe: u8| {
                    (p.ends & pe != 0 && q.ends & qe != 0)
                        || ((x - nodes[v].x).abs() < nodes[v].w / 2.0 + FAN && (y - nodes[v].y).abs() < nodes[v].h / 2.0 + FAN)
                        || (on_run(p.owner, pe, x, y) && on_run(q.owner, qe, x, y))
                };
                if ep.src == eq.src && fan(ep.src, 1, 1) {
                    continue;
                }
                if ep.src == eq.dst && fan(ep.src, 1, 2) {
                    continue;
                }
                if ep.dst == eq.src && fan(ep.dst, 2, 1) {
                    continue;
                }
                if ep.dst == eq.dst && fan(ep.dst, 2, 2) {
                    continue;
                }
                total += 1;
            }
        }
    }
    total
}
