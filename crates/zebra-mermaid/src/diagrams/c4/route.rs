//! Relation routing: straight lines, elbows around shapes in the way, and detours along a
//! lane. Ported from Pele `src/diagrams/c4/route.ts`.

use std::collections::HashMap;

use crate::svg::builder::num;
use crate::util::{js, math};

/// A laid out shape or boundary: top-left corner, size, and the shared shape it is drawn as.
/// `id` stands for the object identity Pele compares boxes by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bx {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub form: &'static str,
}

#[derive(Debug, Clone)]
pub(super) struct Route {
    pub d: String,
    // Where the markers go: the tip, and the way it points.
    pub sx: f64,
    pub sy: f64,
    pub sdx: f64,
    pub sdy: f64,
    pub ex: f64,
    pub ey: f64,
    pub edx: f64,
    pub edy: f64,
    // The stretch the label may sit on.
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    /// The side of that stretch the label is pushed to, as a unit vector; zero centers it on
    /// the stretch.
    pub nx: f64,
    pub ny: f64,
    /// When set, the label stays over the line and moves that way by at most this much.
    pub lean: f64,
    /// The widest the label should be.
    pub room: f64,
    /// The corners of the line, as x, y pairs.
    pub points: Vec<f64>,
}

const LOOP: f64 = 28.0;
pub(super) const LANE: f64 = 20.0;
const LANE_STEP: f64 = 7.0;
const CORNER: f64 = 8.0;
/// A wide label on an upright lane leans this far out from it, to the middle of the gap
/// between columns.
const LEAN: f64 = 30.0;
/// Label widths that fit the gap between two columns, and where there is room to spare.
const NARROW: f64 = 88.0;
const WIDE: f64 = 160.0;
/// The shared person shape: a head of radius 13 centered 14 below the top, over a body that
/// starts 26 below it.
pub(super) const HEAD: f64 = 26.0;

/// The cap radii drawShape uses for the two cylinders.
pub(super) fn cyl_ry(w: f64) -> f64 {
    js::min(12.0, js::max(6.0, w / 14.0))
}

pub(super) fn queue_rx(h: f64) -> f64 {
    js::min(12.0, h / 4.0)
}

/// `Math.sign(v)` as a JavaScript condition sees it: NaN and zero are false.
fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

/// How far along the segment from (px, py) by (dx, dy) it first meets the rectangle;
/// Infinity when it misses.
#[allow(clippy::too_many_arguments)]
fn hit_rect(x0: f64, y0: f64, x1: f64, y1: f64, px: f64, py: f64, dx: f64, dy: f64) -> f64 {
    let mut enter = 0.0;
    let mut leave = 1.0;
    if dx == 0.0 {
        if px < x0 || px > x1 {
            return f64::INFINITY;
        }
    } else {
        let a = (x0 - px) / dx;
        let b = (x1 - px) / dx;
        enter = js::max(enter, js::min(a, b));
        leave = js::min(leave, js::max(a, b));
    }
    if dy == 0.0 {
        if py < y0 || py > y1 {
            return f64::INFINITY;
        }
    } else {
        let a = (y0 - py) / dy;
        let b = (y1 - py) / dy;
        enter = js::max(enter, js::min(a, b));
        leave = js::min(leave, js::max(a, b));
    }
    if enter <= leave {
        enter
    } else {
        f64::INFINITY
    }
}

#[allow(clippy::too_many_arguments)]
fn hit_ellipse(cx: f64, cy: f64, rx: f64, ry: f64, px: f64, py: f64, dx: f64, dy: f64) -> f64 {
    let ux = dx / rx;
    let uy = dy / ry;
    let vx = (px - cx) / rx;
    let vy = (py - cy) / ry;
    let a = ux * ux + uy * uy;
    let b = 2.0 * (ux * vx + uy * vy);
    let c = vx * vx + vy * vy - 1.0;
    let disc = b * b - 4.0 * a * c;
    if a == 0.0 || disc < 0.0 {
        return f64::INFINITY;
    }
    let t = (-b - math::sqrt(disc)) / (2.0 * a);
    if t >= 0.0 && t <= 1.0 {
        t
    } else if c <= 0.0 {
        0.0
    } else {
        f64::INFINITY
    }
}

/// Where a segment from outside first meets the outline of a shape, as a fraction of the
/// segment.
fn hit(b: &Bx, px: f64, py: f64, qx: f64, qy: f64) -> f64 {
    let Bx { x, y, w, h, .. } = *b;
    let dx = qx - px;
    let dy = qy - py;
    match b.form {
        "person" => {
            js::min(hit_rect(x, y + HEAD, x + w, y + h, px, py, dx, dy), hit_ellipse(x + w / 2.0, y + 14.0, 13.0, 13.0, px, py, dx, dy))
        }
        "cyl" => {
            let ry = cyl_ry(w);
            js::min(
                js::min(
                    hit_rect(x, y + ry, x + w, y + h - ry, px, py, dx, dy),
                    hit_ellipse(x + w / 2.0, y + ry, w / 2.0, ry, px, py, dx, dy),
                ),
                hit_ellipse(x + w / 2.0, y + h - ry, w / 2.0, ry, px, py, dx, dy),
            )
        }
        "h-cyl" => {
            let rx = queue_rx(h);
            js::min(
                js::min(
                    hit_rect(x + rx, y, x + w - rx, y + h, px, py, dx, dy),
                    hit_ellipse(x + rx, y + h / 2.0, rx, h / 2.0, px, py, dx, dy),
                ),
                hit_ellipse(x + w - rx, y + h / 2.0, rx, h / 2.0, px, py, dx, dy),
            )
        }
        _ => hit_rect(x, y, x + w, y + h, px, py, dx, dy),
    }
}

pub(super) fn crosses(points: &[f64], x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
    let mut i = 2;
    while i < points.len() {
        let px = points[i - 2];
        let py = points[i - 1];
        if hit_rect(x0, y0, x1, y1, px, py, points[i] - px, points[i + 1] - py) <= 1.0 {
            return true;
        }
        i += 2;
    }
    false
}

fn contains(outer: &Bx, inner: &Bx) -> bool {
    outer.x <= inner.x && outer.y <= inner.y && outer.x + outer.w >= inner.x + inner.w && outer.y + outer.h >= inner.y + inner.h
}

/// The straight line from a to b: x, y of its start, then of its end.
fn connect(a: &Bx, b: &Bx, shift: f64) -> [f64; 4] {
    let ax = a.x + a.w / 2.0;
    let ay = a.y + a.h / 2.0;
    let bx = b.x + b.w / 2.0;
    let by = b.y + b.h / 2.0;
    let down = contains(a, b);
    if down || contains(b, a) {
        // One holds the other: a short line from the nearest side of the outer box.
        let (outer, inner) = if down { (a, b) } else { (b, a) };
        let ix = inner.x + inner.w / 2.0;
        let iy = inner.y + inner.h / 2.0;
        let left = inner.x - outer.x;
        let right = outer.x + outer.w - inner.x - inner.w;
        let top = inner.y - outer.y;
        let bottom = outer.y + outer.h - inner.y - inner.h;
        let least = js::min(js::min(js::min(left, right), top), bottom);
        let ox = if least == left {
            outer.x
        } else if least == right {
            outer.x + outer.w
        } else {
            ix
        };
        let oy = if least == left || least == right {
            iy
        } else if least == bottom {
            outer.y + outer.h
        } else {
            outer.y
        };
        let t = js::min(hit(inner, ox, oy, ix, iy), 1.0);
        let mx = ox + (ix - ox) * t;
        let my = oy + (iy - oy) * t;
        return if down { [ox, oy, mx, my] } else { [mx, my, ox, oy] };
    }

    let mut px = ax;
    let mut py = ay;
    let mut qx = bx;
    let mut qy = by;
    // Shapes that share a column or a row are joined by an upright or level line through the
    // shared span.
    let x0 = js::max(a.x, b.x);
    let x1 = js::min(a.x + a.w, b.x + b.w);
    let y0 = js::max(a.y, b.y);
    let y1 = js::min(a.y + a.h, b.y + b.h);
    if x0 < x1 {
        px = (x0 + x1) / 2.0;
        qx = px;
    } else if y0 < y1 {
        py = (y0 + y1) / 2.0;
        qy = py;
    }
    let len = or_one(math::hypot(qx - px, qy - py));
    let nx = (-(qy - py) / len) * shift;
    let ny = ((qx - px) / len) * shift;
    let mut ta = hit(a, qx + nx, qy + ny, px + nx, py + ny);
    let mut tb = hit(b, px + nx, py + ny, qx + nx, qy + ny);
    if ta <= 1.0 && tb <= 1.0 {
        px += nx;
        py += ny;
        qx += nx;
        qy += ny;
    } else {
        px = ax;
        py = ay;
        qx = bx;
        qy = by;
        ta = js::min(hit(a, qx, qy, px, py), 1.0);
        tb = js::min(hit(b, px, py, qx, qy), 1.0);
    }
    [qx + (px - qx) * ta, qy + (py - qy) * ta, px + (qx - px) * tb, py + (qy - py) * tb]
}

/// `x || 1`
fn or_one(x: f64) -> f64 {
    if truthy(x) {
        x
    } else {
        1.0
    }
}

/// Parallel stretches closer than this read as one line, and a line this close to a
/// boundary's frame reads as part of it.
const NEAR: f64 = 6.0;
const FRAME_NEAR: f64 = 3.0;
const PORT_STEP: f64 = 12.0;

/// `(k & 1 ? 1 : -1) * Math.ceil(k / 2)`: 0, 1, -1, 2, -2, ...
fn spread(k: usize) -> f64 {
    let s = if k & 1 == 1 { 1.0 } else { -1.0 };
    s * (k as f64 / 2.0).ceil()
}

/// What the routes drawn so far take up, so that the next keeps apart from them: upright
/// stretches as x, top, bottom; level ones as y, left, right; and where lines leave or reach
/// each side of a shape.
#[derive(Debug, Default)]
pub(super) struct Lanes {
    upright: Vec<f64>,
    level: Vec<f64>,
    frames: [Vec<f64>; 2],
    taken: HashMap<usize, Vec<f64>>,
    /// Routes that found no lane of their own: a line left straight behind a shape, or a lane
    /// that runs along another.
    pub squeezed: usize,
}

impl Lanes {
    pub fn free(&self, upright: bool, at: f64, a0: f64, a1: f64) -> bool {
        let lo = js::min(a0, a1);
        let hi = js::max(a0, a1);
        let near = |list: &[f64], gap: f64| -> bool {
            let mut i = 0;
            while i < list.len() {
                if (list[i] - at).abs() < gap && list[i + 1] < hi && list[i + 2] > lo {
                    return true;
                }
                i += 3;
            }
            false
        };
        !near(if upright { &self.upright } else { &self.level }, NEAR) && !near(&self.frames[if upright { 0 } else { 1 }], FRAME_NEAR)
    }

    pub fn crossings(&self, points: &[f64]) -> usize {
        let mut count = 0;
        let mut i = 2;
        while i < points.len() {
            let upright = points[i] == points[i - 2];
            let at = if upright { points[i] } else { points[i + 1] };
            let lo = js::min(if upright { points[i - 1] } else { points[i - 2] }, if upright { points[i + 1] } else { points[i] });
            let hi = js::max(if upright { points[i - 1] } else { points[i - 2] }, if upright { points[i + 1] } else { points[i] });
            let list = if upright { &self.level } else { &self.upright };
            let mut j = 0;
            while j < list.len() {
                if list[j] > lo && list[j] < hi && list[j + 1] < at && list[j + 2] > at {
                    count += 1;
                }
                j += 3;
            }
            i += 2;
        }
        count
    }

    pub fn frame(&mut self, b: &Bx) {
        self.frames[0].extend_from_slice(&[b.x, b.y, b.y + b.h, b.x + b.w, b.y, b.y + b.h]);
        self.frames[1].extend_from_slice(&[b.y, b.x, b.x + b.w, b.y + b.h, b.x, b.x + b.w]);
    }

    pub fn take(&mut self, points: &[f64]) {
        let mut i = 2;
        while i < points.len() {
            if points[i] == points[i - 2] {
                self.upright.extend_from_slice(&[points[i], js::min(points[i - 1], points[i + 1]), js::max(points[i - 1], points[i + 1])]);
            } else if points[i + 1] == points[i - 1] {
                self.level.extend_from_slice(&[points[i + 1], js::min(points[i - 2], points[i]), js::max(points[i - 2], points[i])]);
            }
            i += 2;
        }
    }

    /// Where on one side of a shape (0 left, 1 right, 2 top, 3 bottom) a line can leave or
    /// arrive, between lo and hi: the places clear of the lines already there, nearest `want`
    /// first.
    pub fn ports(&self, b: &Bx, side: usize, want: f64, lo: f64, hi: f64) -> Vec<f64> {
        let empty = Vec::new();
        let used = self.taken.get(&b.id).unwrap_or(&empty);
        let mut out = Vec::new();
        let mut k = 0;
        while k < 9 && out.len() < 4 {
            let at = want + spread(k) * PORT_STEP;
            k += 1;
            if at < lo || at > hi {
                continue;
            }
            let mut clear = true;
            let mut i = 0;
            while i < used.len() {
                if used[i] == side as f64 && (used[i + 1] - at).abs() < PORT_STEP {
                    clear = false;
                }
                i += 2;
            }
            if clear {
                out.push(at);
            }
        }
        if out.is_empty() {
            vec![want]
        } else {
            out
        }
    }

    pub fn claim(&mut self, b: &Bx, side: usize, at: f64) {
        self.taken.entry(b.id).or_default().extend_from_slice(&[side as f64, at]);
    }
}

fn path(points: &[f64], start_trim: f64, end_trim: f64) -> String {
    let n = points.len();
    let ux = js::sign(points[2] - points[0]);
    let uy = js::sign(points[3] - points[1]);
    let mut d = format!("M{},{}", num(points[0] + ux * start_trim), num(points[1] + uy * start_trim));
    let mut i = 2;
    while i < n - 2 {
        let x = points[i];
        let y = points[i + 1];
        let ix = js::sign(x - points[i - 2]);
        let iy = js::sign(y - points[i - 1]);
        let ox = js::sign(points[i + 2] - x);
        let oy = js::sign(points[i + 3] - y);
        let r = js::min(
            js::min(CORNER, (x - points[i - 2] + y - points[i - 1]).abs() / 2.0),
            (points[i + 2] - x + points[i + 3] - y).abs() / 2.0,
        );
        let start = if truthy(ix) { format!("H{}", num(x - ix * r)) } else { format!("V{}", num(y - iy * r)) };
        d.push_str(&format!("{}Q{},{} {},{}", start, num(x), num(y), num(x + ox * r), num(y + oy * r)));
        i += 2;
    }
    let lx = js::sign(points[n - 2] - points[n - 4]);
    let ly = js::sign(points[n - 1] - points[n - 3]);
    if truthy(lx) {
        d.push_str(&format!("H{}", num(points[n - 2] - lx * end_trim)));
    } else {
        d.push_str(&format!("V{}", num(points[n - 1] - ly * end_trim)));
    }
    d
}

fn clear_of(points: &[f64], others: &[Bx], a: &Bx, b: &Bx) -> bool {
    for other in others {
        if other.id != a.id && other.id != b.id && crosses(points, other.x, other.y, other.x + other.w, other.y + other.h) {
            return false;
        }
    }
    true
}

/// What a bent route sets about its label, over the defaults.
struct LabelPlace {
    nx: f64,
    lean: f64,
    room: f64,
}

/// The route for a line with corners through `points`, from a's outline to b's. `along` is
/// the stretch its label sits on.
fn bent(points: Vec<f64>, along: usize, start_trim: f64, end_trim: f64, label: LabelPlace) -> Route {
    let n = points.len();
    Route {
        d: path(&points, start_trim, end_trim),
        sx: points[0],
        sy: points[1],
        sdx: -js::sign(points[2] - points[0]),
        sdy: -js::sign(points[3] - points[1]),
        ex: points[n - 2],
        ey: points[n - 1],
        edx: js::sign(points[n - 2] - points[n - 4]),
        edy: js::sign(points[n - 1] - points[n - 3]),
        x0: points[along],
        y0: points[along + 1],
        x1: points[along + 2],
        y1: points[along + 3],
        nx: label.nx,
        ny: 0.0,
        lean: label.lean,
        room: label.room,
        points,
    }
}

fn across(lo: f64, hi: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let mid = (lo + hi) / 2.0;
    let step = js::max(LANE_STEP, (hi - lo) / 4.0);
    for k in 0..9 {
        let at = mid + spread(k) * step;
        if at >= lo + NEAR && at <= hi - NEAR {
            out.push(at);
        }
    }
    out
}

struct Best {
    points: Vec<f64>,
    cost: f64,
    upright: bool,
    sides: [usize; 2],
}

fn elbow(a: &Bx, b: &Bx, others: &[Bx], lanes: &mut Lanes, start_trim: f64, end_trim: f64) -> Option<Route> {
    let ax = a.x + a.w / 2.0;
    let ay = a.y + a.h / 2.0;
    let bx = b.x + b.w / 2.0;
    let by = b.y + b.h / 2.0;
    let mut best: Option<Best> = None;
    // Returns true when the route became the best.
    let consider = |best: &mut Option<Best>, lanes: &Lanes, points: Vec<f64>, upright: bool, sides: [usize; 2]| -> bool {
        if !clear_of(&points, others, a, b) {
            return false;
        }
        let mut i = 2;
        while i < points.len() {
            let up = points[i] == points[i - 2];
            if !lanes.free(
                up,
                if up { points[i] } else { points[i + 1] },
                if up { points[i - 1] } else { points[i - 2] },
                if up { points[i + 1] } else { points[i] },
            ) {
                return false;
            }
            i += 2;
        }
        let mut cost = lanes.crossings(&points) as f64 * 1e6;
        let mut i = 2;
        while i < points.len() {
            cost += (points[i] - points[i - 2]).abs() + (points[i + 1] - points[i - 1]).abs();
            i += 2;
        }
        if best.as_ref().is_none_or(|b| cost < b.cost) {
            *best = Some(Best { points, cost, upright, sides });
            return true;
        }
        false
    };
    if a.x + a.w <= b.x || b.x + b.w <= a.x {
        let right = bx > ax;
        let down = js::sign(by - ay);
        let sides = if right { [1, 0] } else { [0, 1] };
        let lanes_across = across(if right { a.x + a.w } else { b.x + b.w }, if right { b.x } else { a.x });
        'search: for y0 in lanes.ports(a, sides[0], ay + (down * a.h) / 4.0, a.y + 8.0, a.y + a.h - 8.0) {
            for y1 in lanes.ports(b, sides[1], by - (down * b.h) / 4.0, b.y + 8.0, b.y + b.h - 8.0) {
                for &lane in &lanes_across {
                    let x0 = lane + (ax - lane) * js::min(hit(a, lane, y0, ax, y0), 1.0);
                    let x1 = lane + (bx - lane) * js::min(hit(b, lane, y1, bx, y1), 1.0);
                    if consider(&mut best, lanes, vec![x0, y0, lane, y0, lane, y1, x1, y1], true, sides) {
                        break 'search;
                    }
                }
            }
        }
    }
    if a.y + a.h <= b.y || b.y + b.h <= a.y {
        let below = by > ay;
        let sideways = js::sign(bx - ax);
        let sides = if below { [3, 2] } else { [2, 3] };
        let lanes_across = across(if below { a.y + a.h } else { b.y + b.h }, if below { b.y } else { a.y });
        'search: for x0 in lanes.ports(a, sides[0], ax + (sideways * a.w) / 4.0, a.x + 8.0, a.x + a.w - 8.0) {
            for x1 in lanes.ports(b, sides[1], bx - (sideways * b.w) / 4.0, b.x + 8.0, b.x + b.w - 8.0) {
                for &lane in &lanes_across {
                    let y0 = lane + (ay - lane) * js::min(hit(a, x0, lane, x0, ay), 1.0);
                    let y1 = lane + (by - lane) * js::min(hit(b, x1, lane, x1, by), 1.0);
                    if consider(&mut best, lanes, vec![x0, y0, x0, lane, x1, lane, x1, y1], false, sides) {
                        break 'search;
                    }
                }
            }
        }
    }
    let Best { points, upright, sides, .. } = best?;
    lanes.take(&points);
    lanes.claim(a, sides[0], if upright { points[1] } else { points[0] });
    lanes.claim(b, sides[1], if upright { points[7] } else { points[6] });
    // The label sits on the middle stretch, across the gap.
    Some(bent(points, 2, start_trim, end_trim, LabelPlace { nx: 0.0, lean: 0.0, room: if upright { NARROW } else { WIDE } }))
}

/// One side of a detour: its points, the side of the shapes it leaves from, where on them, and
/// how many lanes out it went (5 when none was free).
struct Way {
    points: Vec<f64>,
    side: usize,
    p0: f64,
    p1: f64,
    step: usize,
}

/// Routes a relation from a to b. `shift` moves the line sideways when another runs the
/// opposite way. `others` are the shapes to steer clear of; `None` skips that check. `lanes`
/// holds what the routes before it take up, so that this one keeps apart from them.
pub(super) fn route(a: &Bx, b: &Bx, shift: f64, others: Option<&[Bx]>, lanes: &mut Lanes, start_trim: f64, end_trim: f64) -> Route {
    if a.id == b.id {
        let spread = js::min(a.h / 2.0 - 4.0, 10.0);
        let len = math::hypot(LOOP, spread);
        let x = a.x + a.w;
        let cy = a.y + a.h / 2.0;
        let ux = LOOP / len;
        let uy = spread / len;
        return Route {
            d: format!(
                "M{},{}C{},{} {},{} {},{}",
                num(x + ux * start_trim),
                num(cy - spread - uy * start_trim),
                num(x + LOOP),
                num(cy - spread * 2.0),
                num(x + LOOP),
                num(cy + spread * 2.0),
                num(x + ux * end_trim),
                num(cy + spread + uy * end_trim)
            ),
            sx: x,
            sy: cy - spread,
            sdx: -ux,
            sdy: uy,
            ex: x,
            ey: cy + spread,
            edx: -ux,
            edy: -uy,
            x0: x + LOOP + 4.0,
            y0: cy,
            x1: x + LOOP + 4.0,
            y1: cy,
            nx: 1.0,
            ny: 0.0,
            lean: 0.0,
            room: 64.0,
            points: Vec::new(),
        };
    }

    let [sx, sy, ex, ey] = connect(a, b, shift);
    let upright = sx == ex;
    let mut blocked = false;
    if let Some(others) = others {
        if !contains(a, b) && !contains(b, a) {
            for other in others {
                if other.id != a.id
                    && other.id != b.id
                    && hit_rect(other.x, other.y, other.x + other.w, other.y + other.h, sx, sy, ex - sx, ey - sy) <= 1.0
                {
                    blocked = true;
                }
            }
        }
    }
    if let (Some(others), true) = (others, blocked) {
        if !upright && sy != ey {
            if let Some(around) = elbow(a, b, others, lanes, start_trim, end_trim) {
                return around;
            }
            lanes.squeezed += 1;
        }
    }
    if let (Some(others), true) = (others, blocked && (upright || sy == ey)) {
        return detour(a, b, others, lanes, [sx, sy, ex, ey], upright, start_trim, end_trim);
    }

    let len = or_one(math::hypot(ex - sx, ey - sy));
    let dx = (ex - sx) / len;
    let dy = (ey - sy) / len;
    if upright || sy == ey {
        lanes.take(&[sx, sy, ex, ey]);
    }
    let shifted = truthy(shift);
    Route {
        d: format!("M{},{}L{},{}", num(sx + dx * start_trim), num(sy + dy * start_trim), num(ex - dx * end_trim), num(ey - dy * end_trim)),
        sx,
        sy,
        sdx: -dx,
        sdy: -dy,
        ex,
        ey,
        edx: dx,
        edy: dy,
        x0: sx,
        y0: sy,
        x1: ex,
        y1: ey,
        nx: if shifted { -dy } else { 0.0 },
        ny: if shifted { dx } else { 0.0 },
        lean: 0.0,
        room: if dx.abs() > dy.abs() { NARROW } else { WIDE },
        points: vec![sx, sy, ex, ey],
    }
}

/// A straight upright or level line that a shape blocks takes a lane beside the shapes.
#[allow(clippy::too_many_arguments)]
fn detour(a: &Bx, b: &Bx, others: &[Bx], lanes: &mut Lanes, line: [f64; 4], upright: bool, start_trim: f64, end_trim: f64) -> Route {
    let [sx, sy, ex, ey] = line;
    let lane_on = |forward: bool| -> f64 {
        let mut lane = if forward { f64::NEG_INFINITY } else { f64::INFINITY };
        let mut widen = |bx: &Bx| {
            let edge = if upright { bx.x } else { bx.y };
            lane = if forward { js::max(lane, edge + if upright { bx.w } else { bx.h }) } else { js::min(lane, edge) };
        };
        widen(a);
        widen(b);
        for other in others {
            if other.id != a.id
                && other.id != b.id
                && hit_rect(other.x, other.y, other.x + other.w, other.y + other.h, sx, sy, ex - sx, ey - sy) <= 1.0
            {
                widen(other);
            }
        }
        lane
    };
    let lo = if upright { js::min(a.y, b.y) } else { js::min(a.x, b.x) };
    let hi = if upright { js::max(a.y + a.h, b.y + b.h) } else { js::max(a.x + a.w, b.x + b.w) };
    let outside = |forward: bool, lane: f64| -> bool {
        for other in others {
            if other.id == a.id || other.id == b.id {
                continue;
            }
            let near = if upright { other.y < hi && other.y + other.h > lo } else { other.x < hi && other.x + other.w > lo };
            let start = if upright { other.x } else { other.y };
            let end = start + if upright { other.w } else { other.h };
            if near && (if forward { end > lane } else { start < lane }) {
                return false;
            }
        }
        true
    };
    let ax = a.x + a.w / 2.0;
    let ay = a.y + a.h / 2.0;
    let bx = b.x + b.w / 2.0;
    let by = b.y + b.h / 2.0;
    let towards = if upright { js::sign(by - ay) } else { js::sign(bx - ax) };
    // The route on one side: each end leaves its shape a quarter of the way from the middle
    // towards the other, clear of the straight lines that meet the middle of that side and of
    // other routes there. The lane runs a little further out for each route already along
    // that gap, and never behind a shape. `step` is how many lanes out it went; 5 when none
    // was free.
    let along = |lanes: &Lanes, forward: bool| -> Way {
        let lane = lane_on(forward);
        let sign = if forward { 1.0 } else { -1.0 };
        let side = if upright {
            if forward {
                1
            } else {
                0
            }
        } else if forward {
            3
        } else {
            2
        };
        let p0 = if upright {
            lanes.ports(a, side, ay + (towards * a.h) / 4.0, a.y + 8.0, a.y + a.h - 8.0)[0]
        } else {
            lanes.ports(a, side, ax + (towards * a.w) / 4.0, a.x + 8.0, a.x + a.w - 8.0)[0]
        };
        let p1 = if upright {
            lanes.ports(b, side, by - (towards * b.h) / 4.0, b.y + 8.0, b.y + b.h - 8.0)[0]
        } else {
            lanes.ports(b, side, bx - (towards * b.w) / 4.0, b.x + 8.0, b.x + b.w - 8.0)[0]
        };
        let points_at = |at: f64| -> Vec<f64> {
            if upright {
                let x0 = at + (ax - at) * js::min(hit(a, at, p0, ax, p0), 1.0);
                let x1 = at + (bx - at) * js::min(hit(b, at, p1, bx, p1), 1.0);
                return vec![x0, p0, at, p0, at, p1, x1, p1];
            }
            let y0 = at + (ay - at) * js::min(hit(a, p0, at, p0, ay), 1.0);
            let y1 = at + (by - at) * js::min(hit(b, p1, at, p1, by), 1.0);
            vec![p0, y0, p0, at, p1, at, p1, y1]
        };
        let mut points = points_at(lane + sign * LANE);
        for k in 0..=4usize {
            let at = lane + sign * (LANE + k as f64 * LANE_STEP);
            let tried = points_at(at);
            if !clear_of(&tried, others, a, b) {
                continue;
            }
            points = tried;
            if lanes.free(upright, at, p0, p1) {
                return Way { points, side, p0, p1, step: k };
            }
        }
        Way { points, side, p0, p1, step: 5 }
    };
    // A detour takes the outside of the drawing when only one side is outside, leaving the
    // gaps between columns and rows to the relations that cross them. Otherwise it takes the
    // side where it crosses fewer lines, then the one where it runs nearer its shapes, so that
    // detours spread over both sides.
    let natural = if upright { ey > sy } else { ex < sx };
    let out = outside(natural, lane_on(natural));
    let back = outside(!natural, lane_on(!natural));
    let first = along(lanes, natural);
    let other = if out == back { Some(along(lanes, !natural)) } else { None };
    let cost = |way: &Way| -> usize { lanes.crossings(&way.points) * 8 + way.step };
    let forward = if back && !out {
        !natural
    } else if other.as_ref().is_some_and(|o| cost(o) < cost(&first)) {
        !natural
    } else {
        natural
    };
    let Way { points, side, p0, p1, step } = if forward == natural {
        first
    } else {
        match other {
            Some(o) => o,
            None => along(lanes, !natural),
        }
    };
    let sign = if forward { 1.0 } else { -1.0 };
    if step > 4 {
        lanes.squeezed += 1;
    }
    lanes.take(&points);
    lanes.claim(a, side, p0);
    lanes.claim(b, side, p1);
    let label = if upright { LabelPlace { nx: sign, lean: LEAN, room: NARROW } } else { LabelPlace { nx: 0.0, lean: 0.0, room: WIDE } };
    bent(points, 2, start_trim, end_trim, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(id: usize, x: f64, y: f64, w: f64, h: f64) -> Bx {
        Bx { id, x, y, w, h, form: "rect" }
    }

    #[test]
    fn hits_and_crossings() {
        assert_eq!(hit_rect(0.0, 0.0, 10.0, 10.0, -10.0, 5.0, 20.0, 0.0), 0.5);
        assert_eq!(hit_rect(0.0, 0.0, 10.0, 10.0, -10.0, 20.0, 20.0, 0.0), f64::INFINITY);
        assert!(crosses(&[-5.0, 5.0, 15.0, 5.0], 0.0, 0.0, 10.0, 10.0));
        assert!(!crosses(&[-5.0, 15.0, 15.0, 15.0], 0.0, 0.0, 10.0, 10.0));
    }

    #[test]
    fn straight_and_self_routes() {
        let mut lanes = Lanes::default();
        let a = bx(0, 0.0, 0.0, 100.0, 50.0);
        let b = bx(1, 200.0, 0.0, 100.0, 50.0);
        let r = route(&a, &b, 0.0, Some(&[a, b]), &mut lanes, 0.0, 7.0);
        assert_eq!(r.d, "M100,25L193,25");
        let r = route(&a, &a, 0.0, None, &mut lanes, 7.0, 7.0);
        assert!(r.d.starts_with("M106.59,"), "{}", r.d);
    }
}
