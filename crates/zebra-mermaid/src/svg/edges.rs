//! Edge paths, markers, labels and self-loops. Ported from Pele `src/svg/edges.ts`.

use super::builder::{num, push_num, write_esc, write_num};
use crate::svg::concat::svg_format;
use crate::text::render_label::{LabelRead, LabelView};
use crate::text::{Icons, Label, RenderLabel};
use crate::util::hash::IndexedMap;
use crate::util::{js, math};
use std::fmt;

pub const ARROW: f64 = 8.0;

#[derive(Debug, Clone, PartialEq)]
pub struct EdgePath {
    pub d: String,
    pub sx: f64,
    pub sy: f64,
    pub sdx: f64,
    pub sdy: f64,
    pub ex: f64,
    pub ey: f64,
    pub edx: f64,
    pub edy: f64,
}

// Marker types are Mermaid's: arrow_point, arrow_circle, arrow_cross; anything else draws nothing.

/// The most a corner is rounded by in a stepped route.
const CORNER: f64 = 12.0;
/// How far the handles of a wide turn reach along its two legs. A little more than a circle's,
/// which makes the turn fuller.
const HANDLE: f64 = 0.62;
/// How far an edge runs straight before a marker at its end, where it has the room.
pub const LEAD: f64 = 6.0;

/// How far along the flow the handles of a plain S reach, as a share of the way it travels
/// along it. Pele's `ease` from `layout/layered.ts`, which the layout also uses.
pub use crate::layout::layered::ease;

/// How far from each side of each node the edges that meet it start to turn: as far as the
/// largest marker there needs, for every edge on the side, so that none turns sooner than its
/// neighbour and crosses it.
#[derive(Debug, Default, Clone)]
pub struct SideReach {
    reach: IndexedMap<i64, f64>,
}

impl SideReach {
    pub fn new() -> Self {
        SideReach::default()
    }

    /// Takes a node, the side of it, and the marker of an end there.
    pub fn add(&mut self, node: i64, side: i64, kind: &str) {
        let trim = marker_trim(kind);
        let key = node * 4 + side;
        if side >= 0 && trim > 0.0 && trim + LEAD > self.reach.get(&key).copied().unwrap_or(0.0) {
            self.reach.insert(key, trim + LEAD);
        }
    }

    pub fn of(&self, node: i64, side: i64) -> f64 {
        if side >= 0 {
            self.reach.get(&(node * 4 + side)).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

/// `sideReach()`.
pub fn side_reach() -> SideReach {
    SideReach::new()
}

/// The default reach of an end trimmed by `trim`: far enough to clear its own marker.
pub fn default_reach(trim: f64) -> f64 {
    if trim > 0.0 {
        trim + LEAD
    } else {
        0.0
    }
}

/// Builds the path for a route of x, y, axis triples, trimming both ends to leave room for
/// markers. See Pele's comment on `routePath` for how bends and turns are drawn.
/// `start_reach` and `end_reach` are Pele's optional arguments; pass [`default_reach`] of the
/// trim for its defaults.
pub fn route_path(route: &[f64], curve: Option<&str>, start_trim: f64, end_trim: f64, start_reach: f64, end_reach: f64) -> EdgePath {
    let linear = curve == Some("linear");
    let stepped = matches!(curve, Some("step" | "stepBefore" | "stepAfter"));
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    let mut axes: Vec<f64> = Vec::new();
    let near = |ax: f64, ay: f64, bx: f64, by: f64| (ax - bx).abs() < 0.01 && (ay - by).abs() < 0.01;
    let mut i = 0;
    while i + 2 < route.len() {
        let x = route[i];
        let y = route[i + 1];
        let axis = route[i + 2];
        // A stepped route turns at right angles where a smooth one would curve.
        if stepped && !xs.is_empty() {
            let last = xs.len() - 1;
            if (x - xs[last]).abs() >= 0.01 && (y - ys[last]).abs() >= 0.01 {
                let px = xs[last];
                let py = ys[last];
                let my = match curve {
                    Some("stepBefore") => py,
                    Some("stepAfter") => y,
                    _ => (py + y) / 2.0,
                };
                let mx = match curve {
                    Some("stepBefore") => px,
                    Some("stepAfter") => x,
                    _ => (px + x) / 2.0,
                };
                let turns = if axis == 0.0 { [px, my, x, my] } else { [mx, py, mx, y] };
                let mut k = 0;
                while k < 4 {
                    if !(near(turns[k], turns[k + 1], px, py) || near(turns[k], turns[k + 1], x, y)) {
                        xs.push(turns[k]);
                        ys.push(turns[k + 1]);
                        axes.push(axis);
                    }
                    k += 2;
                }
            }
        }
        xs.push(x);
        ys.push(y);
        axes.push(axis);
        i += 3;
    }
    let mut last = xs.len() - 1;
    let aligned = |xs: &[f64], ys: &[f64], i: usize| (xs[i] - xs[i - 1]).abs() < 0.01 || (ys[i] - ys[i - 1]).abs() < 0.01;
    let straight = |xs: &[f64], ys: &[f64], i: usize| linear || aligned(xs, ys, i);
    // The way the route heads from point i to point j: along the flow where an S joins them.
    let unit = |xs: &[f64], ys: &[f64], i: usize, j: usize, axis: f64| -> (f64, f64) {
        if !straight(xs, ys, i.max(j)) {
            return if axis == 0.0 { (0.0, nonzero_sign(ys[j] - ys[i])) } else { (nonzero_sign(xs[j] - xs[i]), 0.0) };
        }
        let dx = xs[j] - xs[i];
        let dy = ys[j] - ys[i];
        let h = math::hypot(dx, dy);
        let len = if h == 0.0 || h.is_nan() { 1.0 } else { h };
        (dx / len, dy / len)
    };
    let (sdx, sdy) = unit(&xs, &ys, 0, 1, axes[1]);
    let (edx, edy) = unit(&xs, &ys, last - 1, last, axes[last]);
    let sx = xs[0];
    let sy = ys[0];
    let ex = xs[last];
    let ey = ys[last];
    xs[0] += sdx * start_trim;
    ys[0] += sdy * start_trim;
    xs[last] -= edx * end_trim;
    ys[last] -= edy * end_trim;
    let length = |xs: &[f64], ys: &[f64], i: usize| (xs[i] - xs[i - 1]).abs() + (ys[i] - ys[i - 1]).abs();
    // Whether the route turns a corner at point i, from running one way to running the other.
    let turns = |xs: &[f64], ys: &[f64], i: usize, last: usize| -> bool {
        !linear
            && i > 0
            && i < last
            && aligned(xs, ys, i)
            && aligned(xs, ys, i + 1)
            && length(xs, ys, i) >= 0.01
            && length(xs, ys, i + 1) >= 0.01
            && (((xs[i] - xs[i - 1]).abs() < 0.01) != ((xs[i + 1] - xs[i]).abs() < 0.01))
    };
    // An edge that curves into a marker runs straight for a little way first, where it has
    // the room, so that the marker does not sit on the bend.
    let along =
        |xs: &[f64], ys: &[f64], i: usize, ux: f64, uy: f64| (xs[i] - xs[i - 1]).abs() * ux.abs() + (ys[i] - ys[i - 1]).abs() * uy.abs();
    let lead = |reach: f64, trim: f64, run: f64| -> f64 {
        let room = js::min(js::max(0.0, reach - LEAD - trim), run / 4.0);
        room + js::min(reach - trim - room, (run - room) / 4.0)
    };
    let arrives = if end_reach > end_trim && (!straight(&xs, &ys, last) || turns(&xs, &ys, last - 1, last)) {
        lead(end_reach, end_trim, along(&xs, &ys, last, edx, edy))
    } else {
        0.0
    };
    let leaves = if start_reach > start_trim && (!straight(&xs, &ys, 1) || turns(&xs, &ys, 1, last)) {
        lead(start_reach, start_trim, along(&xs, &ys, 1, sdx, sdy))
    } else {
        0.0
    };
    if arrives >= 1.0 {
        let (x, y, a) = (xs[last] - edx * arrives, ys[last] - edy * arrives, axes[last]);
        xs.insert(last, x);
        ys.insert(last, y);
        axes.insert(last, a);
        last += 1;
    }
    if leaves >= 1.0 {
        let (x, y, a) = (xs[0] + sdx * leaves, ys[0] + sdy * leaves, axes[1]);
        xs.insert(1, x);
        ys.insert(1, y);
        axes.insert(1, a);
        last += 1;
    }

    let mut d = String::with_capacity(16 + 24 * last);
    d.push('M');
    push_num(&mut d, xs[0]);
    d.push(',');
    push_num(&mut d, ys[0]);
    let mut at_x = xs[0];
    let mut at_y = ys[0];
    let line_to = |d: &mut String, at_x: &mut f64, at_y: &mut f64, x: f64, y: f64| {
        if near(x, y, *at_x, *at_y) {
            return;
        }
        d.push('L');
        push_num(d, x);
        d.push(',');
        push_num(d, y);
        *at_x = x;
        *at_y = y;
    };
    // `C` and `Q` segments: pairs of coordinates, the first with the command letter.
    let curve_to = |d: &mut String, command: char, points: &[f64]| {
        d.push(command);
        for (i, &p) in points.iter().enumerate() {
            if i > 0 {
                d.push(if i % 2 == 0 { ' ' } else { ',' });
            }
            push_num(d, p);
        }
    };
    for i in 1..=last {
        let x = xs[i];
        let y = ys[i];
        if !straight(&xs, &ys, i) {
            let down = axes[i] == 0.0;
            let along_d = if down { y - at_y } else { x - at_x };
            let k = along_d.abs() * ease(if down { x - at_x } else { y - at_y }, along_d);
            let ux = if down { 0.0 } else { js::sign(along_d) * k };
            let uy = if down { js::sign(along_d) * k } else { 0.0 };
            curve_to(&mut d, 'C', &[at_x + ux, at_y + uy, x - ux, y - uy, x, y]);
            at_x = x;
            at_y = y;
            continue;
        }
        if !turns(&xs, &ys, i, last) {
            // A point part of the way along one straight run adds nothing to the path.
            if i < last
                && aligned(&xs, &ys, i)
                && aligned(&xs, &ys, i + 1)
                && (x - xs[i - 1]) * (xs[i + 1] - x) + (y - ys[i - 1]) * (ys[i + 1] - y) > 0.0
            {
                continue;
            }
            line_to(&mut d, &mut at_x, &mut at_y, x, y);
            continue;
        }
        // How much of the leg before the turn and of the leg after it the turn takes: all of
        // one that ends at a node, short of the straight run into a marker, and half of one it
        // shares.
        let mut before = if turns(&xs, &ys, i - 1, last) { length(&xs, &ys, i) / 2.0 } else { length(&xs, &ys, i) };
        let mut after = if turns(&xs, &ys, i + 1, last) { length(&xs, &ys, i + 1) / 2.0 } else { length(&xs, &ys, i + 1) };
        if stepped {
            let m = js::min(CORNER, js::min(before, after));
            before = m;
            after = m;
        }
        let (ux, uy) = unit(&xs, &ys, i - 1, i, axes[i]);
        let (vx, vy) = unit(&xs, &ys, i, i + 1, axes[i + 1]);
        line_to(&mut d, &mut at_x, &mut at_y, x - ux * before, y - uy * before);
        at_x = x + vx * after;
        at_y = y + vy * after;
        if stepped {
            curve_to(&mut d, 'Q', &[x, y, at_x, at_y]);
        } else {
            curve_to(
                &mut d,
                'C',
                &[
                    x - ux * before * (1.0 - HANDLE),
                    y - uy * before * (1.0 - HANDLE),
                    x + vx * after * (1.0 - HANDLE),
                    y + vy * after * (1.0 - HANDLE),
                    at_x,
                    at_y,
                ],
            );
        }
    }
    EdgePath { d, sx, sy, sdx: -sdx, sdy: -sdy, ex, ey, edx, edy }
}

/// `Math.sign(v) || 1`
fn nonzero_sign(v: f64) -> f64 {
    let s = js::sign(v);
    if s == 0.0 || s.is_nan() {
        1.0
    } else {
        s
    }
}

pub fn marker_trim(kind: &str) -> f64 {
    match kind {
        "arrow_point" => ARROW - 1.0,
        "arrow_circle" => ARROW,
        "arrow_cross" => 4.0,
        _ => 0.0,
    }
}

/// A marker whose tip is at (x, y), pointing along (dx, dy), written when it is formatted.
#[derive(Clone, Copy)]
pub struct Marker<'a> {
    kind: &'a str,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    color: &'a str,
}

impl fmt::Display for Marker<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_marker(f, self.kind, self.x, self.y, self.dx, self.dy, self.color)
    }
}

impl super::concat::Part for Marker<'_> {
    #[inline]
    fn append(&self, out: &mut String) {
        push_marker(out, self.kind, self.x, self.y, self.dx, self.dy, self.color);
    }
}

/// Draws a marker whose tip is at (x, y), pointing along (dx, dy). The markup is written when
/// the result is formatted; `to_string` gives it as text.
pub fn marker<'a>(kind: &'a str, x: f64, y: f64, dx: f64, dy: f64, color: &'a str) -> Marker<'a> {
    Marker { kind, x, y, dx, dy, color }
}

/// Writes a pair of coordinates separated by a comma.
#[inline]
fn write_point<W: fmt::Write + ?Sized>(out: &mut W, x: f64, y: f64) -> fmt::Result {
    write_num(out, x)?;
    out.write_char(',')?;
    write_num(out, y)
}

/// Appends [`marker`] to a buffer.
pub fn push_marker(out: &mut String, kind: &str, x: f64, y: f64, dx: f64, dy: f64, color: &str) {
    let _ = write_marker(out, kind, x, y, dx, dy, color);
}

/// Writes [`marker`].
pub fn write_marker<W: fmt::Write + ?Sized>(out: &mut W, kind: &str, x: f64, y: f64, dx: f64, dy: f64, color: &str) -> fmt::Result {
    let nx = -dy;
    let ny = dx;
    match kind {
        "arrow_point" => {
            let bx = x - dx * ARROW;
            let by = y - dy * ARROW;
            let half = ARROW * 0.42;
            out.write_str("<path class=\"pele-marker\" d=\"M")?;
            write_point(out, x, y)?;
            out.write_char('L')?;
            write_point(out, bx + nx * half, by + ny * half)?;
            out.write_char('L')?;
            write_point(out, bx - nx * half, by - ny * half)?;
            out.write_str("Z\" fill=\"")?;
            out.write_str(color)?;
            out.write_str("\" stroke=\"none\"/>")
        }
        "arrow_circle" => {
            out.write_str("<circle class=\"pele-marker\" cx=\"")?;
            write_num(out, x - dx * 4.0)?;
            out.write_str("\" cy=\"")?;
            write_num(out, y - dy * 4.0)?;
            out.write_str("\" r=\"3.5\" fill=\"")?;
            out.write_str(color)?;
            out.write_str("\" stroke=\"none\"/>")
        }
        "arrow_cross" => {
            let cx = x - dx * 4.0;
            let cy = y - dy * 4.0;
            let a = 3.5;
            let ux = (dx + nx) * a;
            let uy = (dy + ny) * a;
            let vx = (dx - nx) * a;
            let vy = (dy - ny) * a;
            out.write_str("<path class=\"pele-marker\" d=\"M")?;
            write_point(out, cx - ux, cy - uy)?;
            out.write_char('L')?;
            write_point(out, cx + ux, cy + uy)?;
            out.write_char('M')?;
            write_point(out, cx - vx, cy - vy)?;
            out.write_char('L')?;
            write_point(out, cx + vx, cy + vy)?;
            out.write_str("\"/>")
        }
        _ => Ok(()),
    }
}

/// An edge label on a patch of the background, written when it is formatted.
#[derive(Clone, Copy)]
pub struct EdgeLabelSvg<'a> {
    id: &'a str,
    label: LabelView<'a>,
    x: f64,
    y: f64,
    attrs: &'a str,
    icons: Option<&'a Icons>,
    patch: &'a str,
}

impl fmt::Display for EdgeLabelSvg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_edge_label_view(f, self.id, self.label, self.x, self.y, self.attrs, self.icons, self.patch)
    }
}

/// An edge label on a patch of the background, so the line does not run through the text.
/// `patch` is the patch's fill: `ctx.paint(Role::Background, "var(--_bg)")`. The markup is
/// written when the result is formatted.
pub fn edge_label_svg<'a>(
    id: &'a str,
    label: &'a Label,
    x: f64,
    y: f64,
    attrs: &'a str,
    icons: Option<&'a Icons>,
    patch: &'a str,
) -> EdgeLabelSvg<'a> {
    EdgeLabelSvg { id, label: label.view(), x, y, attrs, icons, patch }
}

pub(crate) fn render_edge_label_svg<'a>(
    id: &'a str,
    label: &'a RenderLabel,
    x: f64,
    y: f64,
    attrs: &'a str,
    icons: Option<&'a Icons>,
    patch: &'a str,
) -> EdgeLabelSvg<'a> {
    EdgeLabelSvg { id, label: label.view(), x, y, attrs, icons, patch }
}

/// Appends [`edge_label_svg`] to a buffer.
#[allow(clippy::too_many_arguments)]
pub fn push_edge_label_svg(out: &mut String, id: &str, label: &Label, x: f64, y: f64, attrs: &str, icons: Option<&Icons>, patch: &str) {
    let _ = write_edge_label_svg(out, id, label, x, y, attrs, icons, patch);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_push_edge_label_svg(
    out: &mut String,
    id: &str,
    label: &RenderLabel,
    x: f64,
    y: f64,
    attrs: &str,
    icons: Option<&Icons>,
    patch: &str,
) {
    let _ = write_edge_label_view(out, id, label.view(), x, y, attrs, icons, patch);
}

/// Writes [`edge_label_svg`].
#[allow(clippy::too_many_arguments)]
pub fn write_edge_label_svg<W: fmt::Write + ?Sized>(
    out: &mut W,
    id: &str,
    label: &Label,
    x: f64,
    y: f64,
    attrs: &str,
    icons: Option<&Icons>,
    patch: &str,
) -> fmt::Result {
    write_edge_label_view(out, id, label.view(), x, y, attrs, icons, patch)
}

#[allow(clippy::too_many_arguments)]
fn write_edge_label_view<W: fmt::Write + ?Sized>(
    out: &mut W,
    id: &str,
    label: LabelView<'_>,
    x: f64,
    y: f64,
    attrs: &str,
    icons: Option<&Icons>,
    patch: &str,
) -> fmt::Result {
    let w = label.width + 8.0;
    let h = label.height;
    out.write_str("<g class=\"pele-edge-label\" data-id=\"")?;
    match crate::text::entities::decode_entities_cow(id) {
        std::borrow::Cow::Borrowed(t) => write_esc(out, t)?,
        std::borrow::Cow::Owned(t) => write_esc(out, &t)?,
    }
    out.write_str("\"><rect x=\"")?;
    write_num(out, x - w / 2.0)?;
    out.write_str("\" y=\"")?;
    write_num(out, y - h / 2.0)?;
    out.write_str("\" width=\"")?;
    write_num(out, w)?;
    out.write_str("\" height=\"")?;
    write_num(out, h)?;
    out.write_str("\" rx=\"3\" fill=\"")?;
    out.write_str(patch)?;
    out.write_str("\"/>")?;
    super::builder::write_label_view(out, label, x, y, attrs, icons)?;
    out.write_str("</g>")
}

/// The label position of a self-loop, which `loop_path` sets.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoopLabel {
    pub label_w: f64,
    pub label_h: f64,
    pub label_x: f64,
    pub label_y: f64,
}

/// A self-loop beside a node of drawn size w by h: out of the side that lies across the flow
/// and back in. `k` counts the loops already on the node, so each one reaches a little
/// further. Also places the edge's label beyond the loop.
#[allow(clippy::too_many_arguments)]
pub fn loop_path(cx: f64, cy: f64, w: f64, h: f64, k: f64, sideways: bool, base: f64, trim: f64, e: &mut LoopLabel) -> EdgePath {
    let half = (if sideways { w } else { h }) / 2.0;
    let spread = js::min(half - 4.0, 8.0 + k * 6.0);
    let reach = base + k * 8.0;
    let hy = math::hypot(reach, spread);
    let len = if hy == 0.0 || hy.is_nan() { 1.0 } else { hy };
    // Local frame: `out` points away from the node, `along` runs along its side.
    let at = |out: f64, along: f64| {
        if sideways {
            format!("{},{}", num(cx + along), num(cy + h / 2.0 + out))
        } else {
            format!("{},{}", num(cx + w / 2.0 + out), num(cy + along))
        }
    };
    let tx = (reach / len) * trim;
    let ty = (spread / len) * trim;
    if sideways {
        e.label_x = cx;
        e.label_y = cy + h / 2.0 + reach + 4.0 + e.label_h / 2.0;
    } else {
        e.label_x = cx + w / 2.0 + reach + 4.0 + e.label_w / 2.0;
        e.label_y = cy;
    }
    EdgePath {
        d: svg_format!("M", at(0.0, -spread), "C", at(reach, -spread * 2.0), " ", at(reach, spread * 2.0), " ", at(tx, spread + ty)),
        sx: if sideways { cx - spread } else { cx + w / 2.0 },
        sy: if sideways { cy + h / 2.0 } else { cy - spread },
        sdx: if sideways { 0.0 } else { -1.0 },
        sdy: if sideways { -1.0 } else { 0.0 },
        ex: if sideways { cx + spread } else { cx + w / 2.0 },
        ey: if sideways { cy + h / 2.0 } else { cy + spread },
        edx: if sideways { -spread / len } else { -reach / len },
        edy: if sideways { -reach / len } else { -spread / len },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_and_curved_routes() {
        let p = route_path(&[0.0, 0.0, 0.0, 0.0, 100.0, 0.0], None, 0.0, 7.0, 0.0, default_reach(7.0));
        assert_eq!(p.d, "M0,0L0,93");
        let p = route_path(&[0.0, 0.0, 0.0, 50.0, 100.0, 0.0], None, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(p.d, "M0,0C0,50 50,50 50,100");
        let p = route_path(&[0.0, 0.0, 0.0, 0.0, 40.0, 0.0, 60.0, 40.0, 1.0], Some("step"), 0.0, 0.0, 0.0, 0.0);
        assert!(p.d.contains('Q'), "{}", p.d);
    }

    #[test]
    fn markers() {
        assert_eq!(marker("arrow_point", 10.0, 20.0, 0.0, 1.0, "red").to_string(), "<path class=\"pele-marker\" d=\"M10,20L6.64,12L13.36,12Z\" fill=\"red\" stroke=\"none\"/>");
        assert_eq!(marker("arrow_circle", 10.0, 20.0, 0.0, 1.0, "red").to_string(), "<circle class=\"pele-marker\" cx=\"10\" cy=\"16\" r=\"3.5\" fill=\"red\" stroke=\"none\"/>");
        assert_eq!(marker("arrow_cross", 10.0, 20.0, 0.0, 1.0, "red").to_string(), "<path class=\"pele-marker\" d=\"M13.5,12.5L6.5,19.5M6.5,12.5L13.5,19.5\"/>");
        assert_eq!(marker("none", 10.0, 20.0, 0.0, 1.0, "red").to_string(), "");
    }
}
