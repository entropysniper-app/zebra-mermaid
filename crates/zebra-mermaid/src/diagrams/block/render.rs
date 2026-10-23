//! Block diagram rendering. Ported from Pele `src/diagrams/block/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::{HashMap, HashSet};
use std::f64::consts::{FRAC_1_SQRT_2, PI};
use std::fmt::Write as _;

use super::arrows::{arrow_axis, arrow_mask, arrow_size, draw_arrow, Axis, OVERHANG};
use super::layout::{layout, Cell};
use super::{Block, BlockModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::edges::{marker, marker_trim, render_edge_label_svg as edge_label_svg, EdgePath};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::shapes::{draw_shape, shape_inset, shape_size};
use crate::svg::theme::{class_names, resolve_style, ResolvedStyle};
use crate::text::{layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math};

const GAP: f64 = 12.0;
const WRAP: f64 = 200.0;
const CORNER: f64 = 8.0;
// Checking every edge against every block is quadratic, so large diagrams route without it.
const OBSTACLE_BUDGET: usize = 2_000_000;

fn shape_of(kind: &str) -> &'static str {
    match kind {
        "square" => "rect",
        "round" => "rounded",
        "circle" => "circle",
        "doublecircle" => "dbl-circ",
        "diamond" => "diam",
        "hexagon" => "hex",
        "stadium" => "stadium",
        "subroutine" => "fr-rect",
        "cylinder" => "cyl",
        "rect_left_inv_arrow" => "odd",
        "lean_right" => "lean-r",
        "lean_left" => "lean-l",
        "trapezoid" => "trap-b",
        "inv_trapezoid" => "trap-t",
        _ => "rect",
    }
}

/// Shapes that keep their own size in a larger cell, where the others stretch to fill it.
fn fixed(shape: &str) -> bool {
    matches!(shape, "circle" | "dbl-circ" | "diam")
}

/// Shapes that only widen, because their slants and curves are set by their height.
fn wide(shape: &str) -> bool {
    matches!(shape, "stadium" | "hex" | "odd" | "lean-r" | "lean-l" | "trap-b" | "trap-t")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Leaf,
    Arrow,
    Composite,
    Space,
}

/// What Pele's `View` adds to its layout cell, which lives in a parallel list.
struct View {
    block: usize,
    kind: Kind,
    index: usize,
    shape: &'static str,
    label: Label,
    style: ResolvedStyle,
    mask: u32,
    dx: f64,
    dy: f64,
    // Size as drawn, which for a fixed shape is smaller than its cell.
    bw: f64,
    bh: f64,
}

/// The geometry of a view that routing reads: center, drawn size and outline.
#[derive(Clone, Copy)]
struct Shape {
    x: f64,
    y: f64,
    bw: f64,
    bh: f64,
    kind: Kind,
    shape: &'static str,
    mask: u32,
}

struct Route {
    edge: usize,
    // Two points for a straight line, three for a bend, none for a self-loop.
    points: Vec<f64>,
    loop_k: usize,
    label: Label,
    lx: f64,
    ly: f64,
}

fn is_round(v: &Shape) -> bool {
    v.kind == Kind::Leaf && (v.shape == "circle" || v.shape == "dbl-circ")
}

/// Where a vertical line at x meets the outline, below the center (sign 1) or above it (sign -1).
fn edge_y(v: &Shape, x: f64, sign: f64) -> f64 {
    let hw = v.bw / 2.0;
    let hh = v.bh / 2.0;
    let dx = (x - v.x).abs();
    let mut reach = hh;
    if is_round(v) {
        reach = math::sqrt(js::max(hw * hw - dx * dx, 0.0));
    } else if v.kind == Kind::Leaf && v.shape == "diam" {
        reach = hh * js::max(1.0 - dx / hw, 0.0);
    } else if v.kind == Kind::Leaf {
        reach = hh - shape_inset(v.shape, v.bw, v.bh, if sign > 0.0 { 2 } else { 0 }, 0.0);
    } else if v.kind == Kind::Arrow && arrow_axis(v.mask) == Axis::X {
        reach = hh - OVERHANG;
    }
    v.y + sign * reach
}

/// Where a horizontal line at y meets the outline, right of the center (sign 1) or left of it (sign -1).
fn edge_x(v: &Shape, y: f64, sign: f64) -> f64 {
    let hw = v.bw / 2.0;
    let hh = v.bh / 2.0;
    let dy = (y - v.y).abs();
    let mut reach = hw;
    if is_round(v) {
        reach = math::sqrt(js::max(hh * hh - dy * dy, 0.0));
    } else if v.kind == Kind::Leaf && v.shape == "diam" {
        reach = hw * js::max(1.0 - dy / hh, 0.0);
    } else if v.kind == Kind::Leaf {
        reach = hw - shape_inset(v.shape, v.bw, v.bh, if sign > 0.0 { 1 } else { 3 }, 0.0);
    } else if v.kind == Kind::Arrow && arrow_axis(v.mask) == Axis::Y {
        reach = hw - OVERHANG;
    }
    v.x + sign * reach
}

/// `Math.hypot(a, b) || 1`
fn length_or_one(a: f64, b: f64) -> f64 {
    let h = math::hypot(a, b);
    if h == 0.0 || h.is_nan() {
        1.0
    } else {
        h
    }
}

/// Where the line from the center toward (tx, ty) leaves the outline.
fn border(v: &Shape, tx: f64, ty: f64, out: &mut Vec<f64>) {
    let dx = tx - v.x;
    let dy = ty - v.y;
    let hw = v.bw / 2.0;
    let hh = v.bh / 2.0;
    let mut k = if is_round(v) {
        hw / length_or_one(dx, dy)
    } else if v.kind == Kind::Leaf && v.shape == "diam" {
        let s = dx.abs() / hw + dy.abs() / hh;
        1.0 / if s == 0.0 || s.is_nan() { 1.0 } else { s }
    } else {
        js::min(if dx == 0.0 { f64::INFINITY } else { hw / dx.abs() }, if dy == 0.0 { f64::INFINITY } else { hh / dy.abs() })
    };
    if k == f64::INFINITY {
        k = 0.0;
    }
    out.push(v.x + dx * k);
    out.push(v.y + dy * k);
}

fn crosses_box(x1: f64, y1: f64, x2: f64, y2: f64, v: &Shape) -> bool {
    let l = v.x - v.bw / 2.0 + 1.0;
    let r = v.x + v.bw / 2.0 - 1.0;
    let t = v.y - v.bh / 2.0 + 1.0;
    let b = v.y + v.bh / 2.0 - 1.0;
    if (x1 < l && x2 < l) || (x1 > r && x2 > r) || (y1 < t && y2 < t) || (y1 > b && y2 > b) {
        return false;
    }
    if x1 == x2 || y1 == y2 {
        return true;
    }
    // The segment is diagonal and its bounding box overlaps: the box is hit unless all four
    // corners are on one side.
    let dx = x2 - x1;
    let dy = y2 - y1;
    let side = |x: f64, y: f64| js::sign(dx * (y - y1) - dy * (x - x1));
    let first = side(l, t);
    first == 0.0 || side(r, t) != first || side(r, b) != first || side(l, b) != first
}

fn hits(points: &[f64], s: usize, t: usize, obstacles: &[(usize, Shape)]) -> usize {
    let mut count = 0;
    for (k, v) in obstacles {
        if *k == s || *k == t {
            continue;
        }
        let mut i = 0;
        while i + 3 < points.len() {
            if crosses_box(points[i], points[i + 1], points[i + 2], points[i + 3], v) {
                count += 1;
                break;
            }
            i += 2;
        }
    }
    count
}

/// A straight line where the two blocks share a row or a column, otherwise one bend between
/// facing sides.
fn route_edge(si: usize, ti: usize, s: &Shape, t: &Shape, shift: f64, obstacles: Option<&[(usize, Shape)]>) -> Vec<f64> {
    let sl = s.x - s.bw / 2.0;
    let sr = s.x + s.bw / 2.0;
    let st = s.y - s.bh / 2.0;
    let sb = s.y + s.bh / 2.0;
    let tl = t.x - t.bw / 2.0;
    let tr = t.x + t.bw / 2.0;
    let tt = t.y - t.bh / 2.0;
    let tb = t.y + t.bh / 2.0;
    let x1 = js::max(sl, tl);
    let x2 = js::min(sr, tr);
    let y1 = js::max(st, tt);
    let y2 = js::min(sb, tb);
    let over_x = x2 - x1 > 6.0;
    let over_y = y2 - y1 > 6.0;

    if over_x && over_y {
        // One block is inside the other: join the inner one to the nearest side of the outer one.
        let inner_is_s = s.bw * s.bh <= t.bw * t.bh;
        let (inner, outer) = if inner_is_s { (s, t) } else { (t, s) };
        let up = inner.y - inner.bh / 2.0 - (outer.y - outer.bh / 2.0);
        let down = outer.y + outer.bh / 2.0 - (inner.y + inner.bh / 2.0);
        let left = inner.x - inner.bw / 2.0 - (outer.x - outer.bw / 2.0);
        let right = outer.x + outer.bw / 2.0 - (inner.x + inner.bw / 2.0);
        let least = js::min(js::min(js::min(up, down), left), right);
        let points = if least == down {
            vec![inner.x, edge_y(inner, inner.x, 1.0), inner.x, outer.y + outer.bh / 2.0]
        } else if least == up {
            vec![inner.x, edge_y(inner, inner.x, -1.0), inner.x, outer.y - outer.bh / 2.0]
        } else if least == right {
            vec![edge_x(inner, inner.y, 1.0), inner.y, outer.x + outer.bw / 2.0, inner.y]
        } else {
            vec![edge_x(inner, inner.y, -1.0), inner.y, outer.x - outer.bw / 2.0, inner.y]
        };
        return if inner_is_s { points } else { vec![points[2], points[3], points[0], points[1]] };
    }
    if over_x {
        let margin = js::min(6.0, (x2 - x1) / 2.0);
        let x = js::min(js::max((x1 + x2) / 2.0 + shift, x1 + margin), x2 - margin);
        let down = if t.y > s.y { 1.0 } else { -1.0 };
        return vec![x, edge_y(s, x, down), x, edge_y(t, x, -down)];
    }
    if over_y {
        let margin = js::min(6.0, (y2 - y1) / 2.0);
        let y = js::min(js::max((y1 + y2) / 2.0 + shift, y1 + margin), y2 - margin);
        let right = if t.x > s.x { 1.0 } else { -1.0 };
        return vec![edge_x(s, y, right), y, edge_x(t, y, -right), y];
    }

    let right = if t.x > s.x { 1.0 } else { -1.0 };
    let down = if t.y > s.y { 1.0 } else { -1.0 };
    let across = vec![edge_x(s, s.y, right), s.y, t.x, s.y, t.x, edge_y(t, t.x, -down)];
    let along = vec![s.x, edge_y(s, s.x, down), s.x, t.y, edge_x(t, t.y, -right), t.y];
    let wide = (t.x - s.x).abs() >= (t.y - s.y).abs();
    let (mut best, other) = if wide { (across, along) } else { (along, across) };
    let Some(obstacles) = obstacles else { return best };
    let mut least = hits(&best, si, ti, obstacles);
    if least == 0 {
        return best;
    }
    let count = hits(&other, si, ti, obstacles);
    if count < least {
        best = other;
        least = count;
    }
    if least == 0 {
        return best;
    }
    let mut direct: Vec<f64> = Vec::new();
    border(s, t.x, t.y, &mut direct);
    border(t, s.x, s.y, &mut direct);
    if hits(&direct, si, ti, obstacles) < least {
        direct
    } else {
        best
    }
}

/// Builds the path through two or three points, rounding the bend and leaving room for markers.
fn trace(p: &[f64], mut start_trim: f64, mut end_trim: f64) -> EdgePath {
    let n = p.len();
    let sx = p[0];
    let sy = p[1];
    let ex = p[n - 2];
    let ey = p[n - 1];
    let mut ax = p[2] - sx;
    let mut ay = p[3] - sy;
    let first = math::hypot(ax, ay);
    let first_or_one = if first == 0.0 || first.is_nan() { 1.0 } else { first };
    ax /= first_or_one;
    ay /= first_or_one;
    let mut bx = ex - p[n - 4];
    let mut by = ey - p[n - 3];
    let last = math::hypot(bx, by);
    let last_or_one = if last == 0.0 || last.is_nan() { 1.0 } else { last };
    bx /= last_or_one;
    by /= last_or_one;
    if n == 4 && start_trim + end_trim > first {
        let scale = first / (start_trim + end_trim);
        start_trim *= scale;
        end_trim *= scale;
    }
    start_trim = js::min(start_trim, first);
    end_trim = js::min(end_trim, last);
    let mut d = svg_format!("M", num(sx + ax * start_trim), ",", num(sy + ay * start_trim));
    if n == 6 {
        let r = js::max(0.0, js::min(js::min(CORNER, first - start_trim), last - end_trim));
        write!(
            d,
            "L{},{}Q{},{} {},{}",
            num(p[2] - ax * r),
            num(p[3] - ay * r),
            num(p[2]),
            num(p[3]),
            num(p[2] + bx * r),
            num(p[3] + by * r)
        )
        .unwrap();
    }
    write!(d, "L{},{}", num(ex - bx * end_trim), num(ey - by * end_trim)).unwrap();
    EdgePath { d, sx, sy, sdx: -ax, sdy: -ay, ex, ey, edx: bx, edy: by }
}

/// A self-loop is an arc around the top right corner of the block, or around the matching point
/// on a round or diamond outline. Gives the arc's center and radius and the angles where it
/// leaves and returns.
fn loop_arc(v: &Shape, k: usize) -> (f64, f64, f64, f64, f64) {
    let hw = v.bw / 2.0;
    let hh = v.bh / 2.0;
    let r = js::max(1.0, js::min(js::min(10.0 + 5.0 * (k as f64 - 1.0), hw), hh));
    if is_round(v) {
        let px = v.x + hw * FRAC_1_SQRT_2;
        let py = v.y - hw * FRAC_1_SQRT_2;
        let half = 2.0 * math::asin(js::min(1.0, r / (2.0 * hw)));
        let angle = |at: f64| math::atan2(v.y - hw * math::sin(at) - py, v.x + hw * math::cos(at) - px);
        return (px, py, r, angle(PI / 4.0 + half), angle(PI / 4.0 - half));
    }
    if v.kind == Kind::Leaf && v.shape == "diam" {
        let slope = math::atan2(hh, hw);
        return (v.x + hw / 2.0, v.y - hh / 2.0, r, slope + PI, slope);
    }
    (v.x + hw, v.y - hh, r, PI, PI / 2.0)
}

fn loop_path(v: &Shape, k: usize, start_trim: f64, end_trim: f64) -> EdgePath {
    let (px, py, r, a1, a2) = loop_arc(v, k);
    let from = a1 + start_trim / r;
    let to = a2 - end_trim / r;
    let mut sweep = js::rem(to - from, 2.0 * PI);
    if sweep <= 0.0 {
        sweep += 2.0 * PI;
    }
    let x1 = px + r * math::cos(from);
    let y1 = py + r * math::sin(from);
    let x2 = px + r * math::cos(to);
    let y2 = py + r * math::sin(to);
    let sx = px + r * math::cos(a1);
    let sy = py + r * math::sin(a1);
    let ex = px + r * math::cos(a2);
    let ey = py + r * math::sin(a2);
    // Each marker lies along the chord it covers, so it follows the curve instead of the
    // tangent at its tip.
    let sl = math::hypot(sx - x1, sy - y1);
    let el = math::hypot(ex - x2, ey - y2);
    EdgePath {
        d: svg_format!(
            "M",
            num(x1),
            ",",
            num(y1),
            "A",
            num(r),
            ",",
            num(r),
            " 0 ",
            if sweep > PI { 1 } else { 0 },
            " 1 ",
            num(x2),
            ",",
            num(y2)
        ),
        sx,
        sy,
        sdx: if sl > 0.0 { (sx - x1) / sl } else { math::sin(a1) },
        sdy: if sl > 0.0 { (sy - y1) / sl } else { -math::cos(a1) },
        ex,
        ey,
        edx: if el > 0.0 { (ex - x2) / el } else { -math::sin(a2) },
        edy: if el > 0.0 { (ey - y2) / el } else { math::cos(a2) },
    }
}

const SPOTS: [f64; 5] = [0.5, 0.3, 0.7, 0.15, 0.85];

/// Picks the point on a leg where a label of the given size covers no block other than the two
/// it joins.
#[allow(clippy::too_many_arguments)]
fn label_spot(p: &[f64], at: usize, w: f64, h: f64, s: usize, t: usize, obstacles: Option<&[(usize, Shape)]>) -> f64 {
    let Some(obstacles) = obstacles else { return 0.5 };
    for spot in SPOTS {
        let x = p[at] + (p[at + 2] - p[at]) * spot;
        let y = p[at + 1] + (p[at + 3] - p[at + 1]) * spot;
        let mut clear = true;
        for (k, v) in obstacles {
            if *k != s && *k != t && (x - v.x).abs() < (w + v.bw) / 2.0 && (y - v.y).abs() < (h + v.bh) / 2.0 {
                clear = false;
                break;
            }
        }
        if clear {
            return spot;
        }
    }
    0.5
}

pub(crate) fn render(model: &BlockModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| draw(model, config, ctx, tight))
}

fn draw(model: &BlockModel, config: &Config, ctx: &RenderCtx, tight: bool) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let edge_size = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let icons = ctx.icons();
    let configured = match config.lookup("block", "padding") {
        Some(ConfigValue::Number(n)) if *n >= 0.0 => js::min(*n, 200.0),
        _ => GAP,
    };
    let gap = js::min(configured, if tight { 6.0 } else { f64::INFINITY });
    let wrap = if tight { 80.0 } else { WRAP };

    // Zebra's roles: blocks are nodes, composites are groups, edges are lines with accent heads.
    let shape_attrs =
        svg_format!(" fill=\"", ctx.paint(Role::NodeFill, "var(--_s)"), "\" stroke=\"", ctx.paint(Role::NodeStroke, "var(--_b)"), "\"");
    let line_attrs = svg_format!(" stroke=\"", ctx.paint(Role::NodeStroke, "var(--_b)"), "\"");

    let style_of = |block: &Block| -> ResolvedStyle {
        let mut declarations: Vec<&str> = Vec::new();
        // A style list is split on commas only, so one entry can still hold several declarations.
        fn add<'s>(declarations: &mut Vec<&'s str>, styles: &'s [String]) {
            for style in styles {
                for part in style.split(';') {
                    declarations.push(part);
                }
            }
        }
        if let Some(names) = model.classes_of(block).filter(|names| !names.is_empty()) {
            for name in names {
                if let Some(def) = model.classes.get(name) {
                    add(&mut declarations, &def.styles);
                }
            }
        } else if let Some(def) = model.classes.get("default") {
            add(&mut declarations, &def.styles);
        }
        if let Some(styles) = &block.styles {
            add(&mut declarations, styles);
        }
        resolve_style(&declarations)
    };

    let view = |bi: usize, index: usize| -> (Cell, View) {
        let block = &model.blocks[bi];
        let kind_str = block.kind.as_str();
        let st = if kind_str == "space" { resolve_style::<&str>(&[]) } else { style_of(block) };
        let text_style = (if st.bold { style::BOLD } else { 0 }) | (if st.italic { style::ITALIC } else { 0 });
        let font_size = st.font_size.unwrap_or(size);
        let mut kind = Kind::Leaf;
        let mut shape = "";
        let label: Label;
        let mut nw = 0.0;
        let mut nh = 0.0;
        let mut head = 0.0;
        let mut mask = 0;
        let mut dx = 0.0;
        let mut dy = 0.0;
        if kind_str == "space" {
            kind = Kind::Space;
            label = layout_label(None, false, &m, font_size, wrap, 0);
        } else if kind_str == "composite" {
            kind = Kind::Composite;
            label =
                layout_label(block.label.as_deref(), false, &m, font_size, if tight { title_room(options) } else { 4000.0 }, text_style);
            head = if label.height > 0.0 { label.height + 4.0 } else { 0.0 };
            nw = label.width + 2.0 * gap;
            nh = head + 2.0 * gap;
        } else if kind_str == "block_arrow" {
            kind = Kind::Arrow;
            label = layout_label(block.label.as_deref(), false, &m, font_size, wrap, text_style);
            let blank = label.lines().iter().all(|line| line.iter().all(|part| part.icon.is_none() && js::trim(&part.text).is_empty()));
            mask = arrow_mask(block.directions.as_deref());
            let s = if blank { arrow_size(mask, 0.0, 0.0) } else { arrow_size(mask, label.width, label.height) };
            nw = s.w;
            nh = s.h;
            dx = s.dx;
            dy = s.dy;
        } else {
            shape = shape_of(kind_str);
            label = layout_label(block.label.as_deref(), false, &m, font_size, wrap, text_style);
            let s = shape_size(shape, label.width, label.height);
            nw = s.w;
            // A cylinder's caps deepen as it widens, so it is sized for the deepest ones.
            nh = if shape == "cyl" { js::max(s.h, label.height + 40.0) } else { s.h };
            dy = s.dy;
        }
        let cell = Cell {
            children: Vec::new(),
            columns: if kind == Kind::Composite { block.columns.unwrap_or(-1.0) } else { -1.0 },
            span: if kind == Kind::Space { 1.0 } else { block.width_in_columns.unwrap_or(1.0) },
            space: kind == Kind::Space,
            head,
            nw,
            nh,
            ..Default::default()
        };
        (cell, View { block: bi, kind, index, shape, label, style: st, mask, dx, dy, bw: 0.0, bh: 0.0 })
    };

    let (mut root_cell, root_view) = view(BlockModel::ROOT, 0);
    root_cell.nw = 0.0;
    root_cell.nh = 0.0;
    let mut cells = vec![root_cell];
    let mut order = vec![root_view];
    let mut by_id: HashMap<&str, usize> = HashMap::new();
    let mut i = 0;
    while i < order.len() {
        if order[i].kind == Kind::Composite {
            for &child in &model.blocks[order[i].block].children {
                let index = order.len();
                let (cell, v) = view(child, index);
                cells[i].children.push(index);
                if v.kind != Kind::Space {
                    by_id.insert(model.blocks[child].id.as_deref().unwrap_or(""), index);
                }
                cells.push(cell);
                order.push(v);
            }
        }
        i += 1;
    }

    let (content_width, content_height) = layout(&mut cells, gap)?;

    let mut obstacles: Vec<(usize, Shape)> = Vec::new();
    for (k, v) in order.iter_mut().enumerate() {
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
        if v.kind == Kind::Leaf || v.kind == Kind::Arrow {
            obstacles.push((k, Shape { x: c.x, y: c.y, bw: v.bw, bh: v.bh, kind: v.kind, shape: v.shape, mask: v.mask }));
        }
    }
    let shape = |cells: &[Cell], k: usize| Shape {
        x: cells[k].x,
        y: cells[k].y,
        bw: order[k].bw,
        bh: order[k].bh,
        kind: order[k].kind,
        shape: order[k].shape,
        mask: order[k].mask,
    };

    // Edges between the same two blocks are spread apart instead of drawn on top of each other.
    let n_views = order.len();
    let mut pairs: HashMap<usize, usize> = HashMap::new();
    let mut labelled: HashSet<usize> = HashSet::new();
    let mut ends: Vec<(usize, usize)> = Vec::new();
    let mut drawn: Vec<usize> = Vec::new();
    for &edge in &model.edges {
        let e = &model.blocks[edge];
        let (Some(&s), Some(&t)) = (by_id.get(e.start.as_deref().unwrap_or("")), by_id.get(e.end.as_deref().unwrap_or(""))) else {
            continue;
        };
        let key = order[s].index.min(order[t].index) * n_views + order[s].index.max(order[t].index);
        *pairs.entry(key).or_insert(0) += 1;
        if e.label.as_deref().is_some_and(|l| !l.is_empty()) {
            labelled.insert(key);
        }
        ends.push((s, t));
        drawn.push(edge);
    }
    let check: Option<&[(usize, Shape)]> = if drawn.len() * obstacles.len() <= OBSTACLE_BUDGET { Some(&obstacles) } else { None };
    let mut seen: HashMap<usize, usize> = HashMap::new();
    let mut routes: Vec<Route> = Vec::new();
    let mut min_x = 0.0;
    let mut min_y = 0.0;
    let mut max_x = content_width;
    let mut max_y = content_height;
    let mut grow = |x: f64, y: f64, w: f64, h: f64| {
        if x - w / 2.0 < min_x {
            min_x = x - w / 2.0;
        }
        if x + w / 2.0 > max_x {
            max_x = x + w / 2.0;
        }
        if y - h / 2.0 < min_y {
            min_y = y - h / 2.0;
        }
        if y + h / 2.0 > max_y {
            max_y = y + h / 2.0;
        }
    };
    for (i, &edge) in drawn.iter().enumerate() {
        let (s, t) = ends[i];
        let key = order[s].index.min(order[t].index) * n_views + order[s].index.max(order[t].index);
        let total = pairs.get(&key).copied().unwrap_or(1);
        let k = seen.get(&key).copied().unwrap_or(0);
        seen.insert(key, k + 1);
        let label = layout_label(model.blocks[edge].label.as_deref(), false, &m, edge_size, WRAP, 0);
        let points: Vec<f64>;
        let lx: f64;
        let ly: f64;
        if s == t {
            let (px, py, r, _, _) = loop_arc(&shape(&cells, s), k + 1);
            points = Vec::new();
            grow(px, py, 2.0 * r + 2.0, 2.0 * r + 2.0);
            lx = px + r + 6.0 + label.width / 2.0;
            ly = py - r / 2.0;
        } else {
            let step = if labelled.contains(&key) { js::round(edge_size * 1.5) + 2.0 } else { 14.0 };
            points = route_edge(s, t, &shape(&cells, s), &shape(&cells, t), (k as f64 - (total as f64 - 1.0) / 2.0) * step, check);
            let n = points.len();
            // The label goes on the longer leg.
            let first_leg = (points[2] - points[0]).abs() + (points[3] - points[1]).abs();
            let last_leg = (points[n - 2] - points[n - 4]).abs() + (points[n - 1] - points[n - 3]).abs();
            let at = if n == 6 && last_leg > first_leg { 2 } else { 0 };
            let spot = if label.width > 0.0 { label_spot(&points, at, label.width + 8.0, label.height, s, t, check) } else { 0.5 };
            lx = points[at] + (points[at + 2] - points[at]) * spot;
            ly = points[at + 1] + (points[at + 3] - points[at + 1]) * spot;
        }
        if label.width > 0.0 {
            grow(lx, ly, label.width + 8.0, label.height);
        }
        routes.push(Route { edge, points, loop_k: if s == t { k + 1 } else { 0 }, label, lx, ly });
    }

    let title =
        layout_label(model.title.as_deref(), false, &m.without_math(), size, if tight { title_room(options) } else { 4000.0 }, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(max_x - min_x, title.width);
    let ox = pad - min_x + (inner - (max_x - min_x)) / 2.0;
    let oy = pad + title_height - min_y;
    for c in cells.iter_mut() {
        c.x += ox;
        c.y += oy;
    }
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();

    let mut clusters = String::new();
    let mut nodes = String::new();
    for i in 1..order.len() {
        let v = &order[i];
        let c = &cells[i];
        if v.kind == Kind::Space {
            continue;
        }
        let block = &model.blocks[v.block];
        let classes = class_names(&model.classes_of(block).unwrap_or(&[]).join(" "));
        let id = esc_text(block.id.as_deref().unwrap_or(""));
        if v.kind == Kind::Composite {
            let top = c.y - c.h / 2.0;
            svg_write!(
                clusters,
                "<g class=\"pele-cluster",
                classes,
                "\" data-id=\"",
                id,
                "\"><rect x=\"",
                num(c.x - c.w / 2.0),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(c.w),
                "\" height=\"",
                num(c.h),
                "\"",
                crate::svg::theme::rounded(&svg_format!(
                    " fill=\"",
                    ctx.paint(Role::GroupFill, "var(--_a)"),
                    "\" fill-opacity=\"0.5\" stroke=\"",
                    ctx.paint(Role::GroupStroke, "var(--_b)"),
                    "\"",
                    v.style.shape
                )),
                "/>",
                label_svg(
                    &v.label,
                    c.x,
                    top + (c.head + gap) / 2.0 + 2.0,
                    &svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", v.style.text),
                    icons
                ),
                "</g>"
            )
            .unwrap();
            continue;
        }
        let arrow = v.kind == Kind::Arrow;
        let drawn_shape = if arrow {
            draw_arrow(v.mask, v.bw, v.bh, &format!("{}{}", shape_attrs, v.style.shape))
        } else {
            draw_shape(
                v.shape,
                v.bw,
                v.bh,
                &format!("{}{}", shape_attrs, v.style.shape),
                &format!("{}{}", line_attrs, v.style.line),
                ctx.theming,
            )
        };
        svg_write!(
            nodes,
            "<g class=\"pele-node pele-shape-",
            if arrow { "block-arrow" } else { v.shape },
            classes,
            "\" data-id=\"",
            id,
            "\" transform=\"translate(",
            num(c.x),
            ",",
            num(c.y),
            ")\">",
            drawn_shape,
            label_svg(&v.label, v.dx, v.dy, &svg_format!(" class=\"pele-label\"", v.style.text), icons),
            "</g>"
        )
        .unwrap();
    }

    let arrow_color = ctx.paint(Role::Arrow, "var(--_l)");
    let patch = ctx.paint(Role::Background, "var(--_bg)");
    let mut edges = String::new();
    let mut labels = String::new();
    for (i, route) in routes.iter_mut().enumerate() {
        let edge = &model.blocks[route.edge];
        let start_type = edge.arrow_type_start.as_deref().unwrap_or("");
        let end_type = edge.arrow_type_end.as_deref().unwrap_or("");
        let path = if route.loop_k > 0 {
            loop_path(&shape(&cells, ends[i].0), route.loop_k, marker_trim(start_type), marker_trim(end_type))
        } else {
            let mut k = 0;
            while k + 1 < route.points.len() {
                route.points[k] += ox;
                route.points[k + 1] += oy;
                k += 2;
            }
            trace(&route.points, marker_trim(start_type), marker_trim(end_type))
        };
        let id = esc_text(edge.id.as_deref().unwrap_or(""));
        svg_write!(
            edges,
            "<g class=\"pele-edge\" data-id=\"",
            id,
            "\"><path d=\"",
            path.d,
            "\"",
            if edge.thickness.as_deref() == Some("thick") { " stroke-width=\"2.5\"" } else { "" },
            if edge.pattern.as_deref() == Some("dotted") { " stroke-dasharray=\"3 4\"" } else { "" },
            "/>",
            marker(start_type, path.sx, path.sy, path.sdx, path.sdy, arrow_color),
            marker(end_type, path.ex, path.ey, path.edx, path.edy, arrow_color),
            "</g>"
        )
        .unwrap();
        if route.label.width > 0.0 {
            let x = route.lx + ox;
            let y = route.ly + oy;
            svg_write!(labels, edge_label_svg(edge.id.as_deref().unwrap_or(""), &route.label, x, y, "", icons, patch)).unwrap();
        }
    }

    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
    if !clusters.is_empty() {
        svg_write!(content, "<g class=\"pele-clusters\">", clusters, "</g>").unwrap();
    }
    if !edges.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-edges\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\" stroke-linecap=\"round\">",
            edges,
            "</g>"
        )
        .unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", nodes, "</g>").unwrap();
    if !labels.is_empty() {
        // Pele lets edge labels take the text color. Zebra writes them in the secondary text
        // color, which resolved output sets on their group.
        let fill = if ctx.resolved() { svg_format!(" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"") } else { String::new() };
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", js::number_text(edge_size), "\"", fill, ">", labels, "</g>")
            .unwrap();
    }

    Ok(Rendered { svg: svg_document("block", width, height, size, ctx, Accessible::default(), &content), width, height, links: Vec::new() })
}
