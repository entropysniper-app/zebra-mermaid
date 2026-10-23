//! Architecture rendering. Ported from Pele `src/diagrams/architecture/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::HashMap;

use super::icons::builtin_icon;
use super::layout::{place_on_grid, side_index};
use super::{ArchNodeKind, ArchitectureModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::edges::{marker, marker_trim, render_edge_label_svg as edge_label_svg};
use crate::svg::root::{svg_document, Accessible};
use crate::text::{layout_render_label as layout_label, style, Icons, Measure, RenderLabel as Label, Span};
use crate::theme::Role;
use crate::util::{js, math};
use std::fmt::Write as _;

const STUB: f64 = 12.0;
const CORNER: f64 = 6.0;
const LABEL_GAP: f64 = 4.0;
const DOT: f64 = 3.0;
const DX: [f64; 4] = [-1.0, 1.0, 0.0, 0.0];
const DY: [f64; 4] = [0.0, 0.0, -1.0, 1.0];

fn setting(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("architecture", key) {
        Some(ConfigValue::Number(v)) if v.is_finite() => js::min(max, js::max(min, *v)),
        _ => fallback,
    }
}

/// Maps the grid lines in use to consecutive indexes, so empty rows and columns take no room.
fn rank(mut values: Vec<i64>) -> HashMap<i64, usize> {
    values.sort();
    let mut out = HashMap::new();
    for value in values {
        let next = out.len();
        out.entry(value).or_insert(next);
    }
    out
}

/// The host's drawing of an icon, else the built-in one: Pele's `icons?.(name) ?? builtinIcon(name)`.
///
/// `Icons::markup` answers an empty string both for a name the host does not know and for a
/// host that draws the name as nothing, so the built-in drawing is used in both cases. Pele
/// uses it only in the first.
fn icon_slot(name: &str, x: f64, y: f64, side: f64, icons: Option<&Icons>, glyph_color: &str) -> String {
    let hosted = icons.map(|i| i.markup(name)).filter(|m| !m.is_empty());
    let inner = match hosted {
        Some(markup) => markup.to_string(),
        None => builtin_icon(name, glyph_color),
    };
    svg_format!("<svg class=\"pele-icon\" data-icon=\"", esc_text(name), "\" x=\"", num(x), "\" y=\"", num(y), "\" width=\"", num(side), "\" height=\"", num(side), "\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">", inner, "</svg>")
}

/// The first `units` UTF-16 code units of `text`, for measuring. Where that cuts a surrogate
/// pair, JavaScript measures a lone high surrogate, which Pele's measurer counts as the whole
/// wide character (the low surrogate has no width); the whole character is taken here.
fn measured_prefix(text: &str, units: usize) -> &str {
    let mut count = 0;
    for (i, c) in text.char_indices() {
        if count >= units {
            return &text[..i];
        }
        count += c.len_utf16();
    }
    text
}

/// Whether UTF-16 code unit `unit` of `text` is a high surrogate: where a character outside
/// the Basic Multilingual Plane starts.
fn high_surrogate_at(text: &str, unit: usize) -> bool {
    let mut count = 0;
    for c in text.chars() {
        if count >= unit {
            return count == unit && c.len_utf16() == 2;
        }
        count += c.len_utf16();
    }
    false
}

/// Cuts a label down to a box, ending what is left with an ellipsis.
fn fit(label: &Label, max_width: f64, max_lines: usize, m: &Measure) -> Label {
    let cut = label.lines().len() > max_lines;
    if !cut && label.width <= max_width {
        return label.clone();
    }
    let dots = m.width("…", label.size, 0);
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();
    let mut widest: f64 = 0.0;
    for i in 0..max_lines.min(label.lines().len()) {
        let mut line = label.lines()[i].to_vec();
        let mut width = label.widths()[i];
        if width > max_width || (cut && i + 1 == max_lines) {
            loop {
                let Some(span) = line.last_mut() else {
                    line.push(Span { text: "…".to_string(), style: 0, icon: None, math: None, width: dots });
                    width = dots;
                    break;
                };
                let rest = width - span.width;
                if span.icon.is_none() && rest + dots < max_width {
                    let mut lo = 0usize;
                    let mut hi = js::len16(&span.text);
                    while lo < hi {
                        let mid = (lo + hi + 1) >> 1;
                        if rest + m.width(measured_prefix(&span.text, mid), label.size, span.style) + dots <= max_width {
                            lo = mid;
                        } else {
                            hi = mid - 1;
                        }
                    }
                    // A cut after a high surrogate moves back before it.
                    if lo > 0 && high_surrogate_at(&span.text, lo - 1) {
                        lo -= 1;
                    }
                    span.text = format!("{}…", js::trim_end(js::slice16(&span.text, 0, lo)));
                    span.width = m.width(&span.text, label.size, span.style);
                    width = rest + span.width;
                    break;
                }
                line.pop();
                width = rest;
            }
        }
        lines.push(line);
        widths.push(width);
        widest = js::max(widest, width);
    }
    let height = lines.len() as f64 * label.line_height;
    Label::from_lines(lines, widths, widest, height, label.size, label.line_height)
}

#[derive(Debug, Clone, Copy)]
struct Port {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    /// The box the route has to stay out of when it cannot leave straight.
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    /// How far short of the port an arrowhead stops: the radius of a junction's dot.
    gap: f64,
    /// Set where a junction is reached through the border of its group: the line carries on
    /// to the junction.
    wire: bool,
}

/// An orthogonal route from one port to another, each left along its side's normal.
fn route(s: &Port, e: &Port) -> Vec<f64> {
    let (sx, sy, sdx, sdy) = (s.x, s.y, s.dx, s.dy);
    let (ex, ey, edx, edy) = (e.x, e.y, e.dx, e.dy);
    let s1x = sx + sdx * STUB;
    let s1y = sy + sdy * STUB;
    let e1x = ex + edx * STUB;
    let e1y = ey + edy * STUB;
    if sdx * edx + sdy * edy == 0.0 {
        let cx = if sdx != 0.0 { ex } else { sx };
        let cy = if sdx != 0.0 { sy } else { ey };
        if (cx - sx) * sdx + (cy - sy) * sdy > 0.0 && (cx - ex) * edx + (cy - ey) * edy > 0.0 {
            return vec![sx, sy, cx, cy, ex, ey];
        }
        return if sdx != 0.0 {
            vec![sx, sy, s1x, s1y, s1x, e1y, e1x, e1y, ex, ey]
        } else {
            vec![sx, sy, s1x, s1y, e1x, s1y, e1x, e1y, ex, ey]
        };
    }
    let sideways = sdx != 0.0;
    let offset = if sideways { ey - sy } else { ex - sx };
    if sdx == -edx && sdy == -edy {
        let ahead = (ex - sx) * sdx + (ey - sy) * sdy;
        if ahead > 0.0 {
            if offset.abs() < 0.5 {
                return vec![sx, sy, ex, ey];
            }
            // Turn midway between the two boxes, which is clear of both titles, when there is
            // such a place.
            if sideways {
                let lo = if sdx > 0.0 { s.x1 } else { e.x1 };
                let hi = if sdx > 0.0 { e.x0 } else { s.x0 };
                let mx = if lo < hi { (lo + hi) / 2.0 } else { (sx + ex) / 2.0 };
                return vec![sx, sy, mx, sy, mx, ey, ex, ey];
            }
            let lo = if sdy > 0.0 { s.y1 } else { e.y1 };
            let hi = if sdy > 0.0 { e.y0 } else { s.y0 };
            let my = if lo < hi { (lo + hi) / 2.0 } else { (sy + ey) / 2.0 };
            return vec![sx, sy, sx, my, ex, my, ex, ey];
        }
    } else if offset.abs() >= 1.0 {
        if sideways {
            let x = if sdx > 0.0 { js::max(sx, ex) + STUB } else { js::min(sx, ex) - STUB };
            return vec![sx, sy, x, sy, x, ey, ex, ey];
        }
        let y = if sdy > 0.0 { js::max(sy, ey) + STUB } else { js::min(sy, ey) - STUB };
        return vec![sx, sy, sx, y, ex, y, ex, ey];
    }
    // The straight way is blocked by one of the two nodes: go between them, or around both.
    if sideways {
        let lane = if s.y1 + STUB <= e.y0 {
            (s.y1 + e.y0) / 2.0
        } else if e.y1 + STUB <= s.y0 {
            (e.y1 + s.y0) / 2.0
        } else {
            js::max(s.y1, e.y1) + STUB
        };
        return vec![sx, sy, s1x, sy, s1x, lane, e1x, lane, e1x, ey, ex, ey];
    }
    let lane = if s.x1 + STUB <= e.x0 {
        (s.x1 + e.x0) / 2.0
    } else if e.x1 + STUB <= s.x0 {
        (e.x1 + s.x0) / 2.0
    } else {
        js::max(s.x1, e.x1) + STUB
    };
    vec![sx, sy, sx, s1y, lane, s1y, lane, e1y, ex, e1y, ex, ey]
}

/// Drops repeated points and joins runs that continue in a straight line.
fn simplify(p: &[f64]) -> Vec<f64> {
    let mut out = vec![p[0], p[1]];
    let mut i = 2;
    while i < p.len() {
        let x = p[i];
        let y = p[i + 1];
        i += 2;
        let n = out.len();
        let same_x = (x - out[n - 2]).abs() < 0.01;
        let same_y = (y - out[n - 1]).abs() < 0.01;
        if same_x && same_y {
            continue;
        }
        if n >= 4 && ((same_x && (out[n - 4] - out[n - 2]).abs() < 0.01) || (same_y && (out[n - 3] - out[n - 1]).abs() < 0.01)) {
            out[n - 2] = x;
            out[n - 1] = y;
        } else {
            out.push(x);
            out.push(y);
        }
    }
    out
}

fn path_data(p: &[f64], start_trim: f64, end_trim: f64) -> String {
    let n = p.len() / 2;
    let mut xs: Vec<f64> = (0..n).map(|i| p[i * 2]).collect();
    let mut ys: Vec<f64> = (0..n).map(|i| p[i * 2 + 1]).collect();
    let trim = |xs: &mut [f64], ys: &mut [f64], at: usize, toward: usize, amount: f64| {
        let len = math::hypot(xs[toward] - xs[at], ys[toward] - ys[at]);
        let t = js::min(amount, js::max(0.0, len - 1.0));
        if len > 0.0 && t > 0.0 {
            xs[at] += ((xs[toward] - xs[at]) / len) * t;
            ys[at] += ((ys[toward] - ys[at]) / len) * t;
        }
    };
    trim(&mut xs, &mut ys, 0, 1, start_trim);
    trim(&mut xs, &mut ys, n - 1, n - 2, end_trim);
    let mut d = svg_format!("M", num(xs[0]), ",", num(ys[0]));
    for i in 1..n - 1 {
        let before = math::hypot(xs[i] - xs[i - 1], ys[i] - ys[i - 1]);
        let after = math::hypot(xs[i + 1] - xs[i], ys[i + 1] - ys[i]);
        let r = js::min(js::min(CORNER, before / 2.0), after / 2.0);
        if r < 0.5 {
            write!(d, "L{},{}", num(xs[i]), num(ys[i])).unwrap();
            continue;
        }
        let in_x = xs[i] - ((xs[i] - xs[i - 1]) / before) * r;
        let in_y = ys[i] - ((ys[i] - ys[i - 1]) / before) * r;
        let out_x = xs[i] + ((xs[i + 1] - xs[i]) / after) * r;
        let out_y = ys[i] + ((ys[i + 1] - ys[i]) / after) * r;
        write!(d, "L{},{}Q{},{} {},{}", num(in_x), num(in_y), num(xs[i]), num(ys[i]), num(out_x), num(out_y)).unwrap();
    }
    write!(d, "L{},{}", num(xs[n - 1]), num(ys[n - 1])).unwrap();
    d
}

pub(crate) fn render(model: &ArchitectureModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| Ok(draw(model, config, ctx, tight)))
}

fn draw(model: &ArchitectureModel, config: &Config, ctx: &RenderCtx, tight: bool) -> Rendered {
    let options = ctx.options;
    let icon_size = setting(config, "iconSize", 48.0, 8.0, 512.0);
    let size = setting(config, "fontSize", options.font_size.unwrap_or(16.0), 4.0, 128.0);
    let group_pad = js::min(setting(config, "padding", 20.0, 0.0, 256.0), if tight { 12.0 } else { f64::INFINITY });
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let icons = ctx.icons();
    let half = icon_size / 2.0;
    // Pele draws the built-in icons in the text color. Zebra gives the icons of a diagram its
    // first chart color, which is the accent unless the caller set series colors.
    let glyph_color = if ctx.resolved() { ctx.series(0) } else { std::borrow::Cow::Borrowed("var(--_fg)") };

    let nodes: Vec<&super::ArchNode> = model.nodes.values().collect();
    let groups: Vec<&super::ArchGroup> = model.groups.values().collect();
    let n = nodes.len();
    let g_count = groups.len();
    let grid = place_on_grid(&nodes, &groups, &model.edges, &model.layout_hints);
    let mut index: HashMap<&str, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        index.insert(node.id.as_str(), i);
    }

    // A service is its icon square with the title beneath; a junction takes the same room.
    let mut labels: Vec<Label> = Vec::with_capacity(n);
    let mut box_w = vec![0.0f64; n];
    let mut box_h = vec![0.0f64; n];
    let wrap = if tight { js::max(icon_size, 64.0) } else { js::max(icon_size * 2.25, 96.0) };
    for (i, node) in nodes.iter().enumerate() {
        let title = if node.kind == ArchNodeKind::Service { node.title.as_deref() } else { None };
        let label = layout_label(title, true, &m, size, wrap, 0);
        box_w[i] = js::max(icon_size, 2.0 * (label.width / 2.0).ceil());
        box_h[i] = icon_size + if label.height > 0.0 { LABEL_GAP + label.height } else { 0.0 };
        labels.push(label);
    }

    let mut col_values: Vec<i64> = Vec::new();
    let mut row_values: Vec<i64> = Vec::new();
    for i in 0..n {
        col_values.push(grid.col[i]);
        row_values.push(grid.row[i]);
    }
    for g in 0..g_count {
        if !grid.group_empty[g] {
            continue;
        }
        col_values.push(grid.group_col[g]);
        row_values.push(grid.group_row[g]);
    }
    let col_of = rank(col_values);
    let row_of = rank(row_values);
    // An index Pele reads from the map and stores in an Int32Array: a missing one becomes 0.
    let col_at = |v: i64| col_of.get(&v).copied().unwrap_or(0);
    let row_at = |v: i64| row_of.get(&v).copied().unwrap_or(0);
    let cols = col_of.len();
    let rows = row_of.len();
    let mut col_w = vec![0.0f64; cols];
    let mut row_h = vec![0.0f64; rows];
    let gap = if tight { js::max(24.0, 2.0 * STUB) } else { js::max(40.0, js::round(icon_size * 0.75)) };
    let mut gap_x = vec![gap; cols];
    let mut gap_y = vec![gap; rows];
    let mut node_col = vec![0usize; n];
    let mut node_row = vec![0usize; n];
    for i in 0..n {
        let c = col_at(grid.col[i]);
        let r = row_at(grid.row[i]);
        node_col[i] = c;
        node_row[i] = r;
        col_w[c] = js::max(col_w[c], box_w[i]);
        row_h[r] = js::max(row_h[r], box_h[i]);
    }
    for g in 0..g_count {
        if !grid.group_empty[g] {
            continue;
        }
        let c = col_at(grid.group_col[g]);
        let r = row_at(grid.group_row[g]);
        col_w[c] = js::max(col_w[c], icon_size);
        row_h[r] = js::max(row_h[r], half);
    }

    // An edge label needs the gap its edge crosses to be wide enough to hold it.
    let e_count = model.edges.len();
    let mut edge_labels: Vec<Label> = Vec::with_capacity(e_count);
    let mut ends: Vec<Option<(usize, usize)>> = Vec::with_capacity(e_count);
    for edge in &model.edges {
        let a = index.get(edge.lhs_id.as_str()).copied();
        let b = index.get(edge.rhs_id.as_str()).copied();
        let label = layout_label(edge.title.as_deref(), true, &m, small, 160.0, 0);
        ends.push(a.zip(b));
        let width = label.width;
        let height = label.height;
        edge_labels.push(label);
        let (Some(a), Some(b)) = (a, b) else { continue };
        if width == 0.0 {
            continue;
        }
        if node_col[a] != node_col[b] {
            let c = node_col[a].min(node_col[b]);
            gap_x[c] = js::max(gap_x[c], width.ceil() + 28.0);
        } else {
            col_w[node_col[a]] = js::max(col_w[node_col[a]], 2.0 * (width / 2.0).ceil() + 8.0);
            if node_row[a] != node_row[b] {
                let r = node_row[a].min(node_row[b]);
                gap_y[r] = js::max(gap_y[r], height.ceil() + 24.0);
            }
        }
    }

    // Room around each group, kept on the rows and columns at its edges. A group that shares an
    // edge with a group inside it stands that much further out. Children come after their
    // parents, so walking backwards settles every group before the one that holds it.
    let mut group_labels: Vec<Label> = Vec::with_capacity(g_count);
    let mut c0 = vec![0usize; g_count];
    let mut c1 = vec![0usize; g_count];
    let mut r0 = vec![0usize; g_count];
    let mut r1 = vec![0usize; g_count];
    let mut ins_l = vec![0.0f64; g_count];
    let mut ins_r = vec![0.0f64; g_count];
    let mut ins_t = vec![0.0f64; g_count];
    let mut ins_b = vec![0.0f64; g_count];
    let mut head_w = vec![0.0f64; g_count];
    let mut head_h = vec![0.0f64; g_count];
    let mut col_l = vec![0.0f64; cols];
    let mut col_r = vec![0.0f64; cols];
    let mut row_t = vec![0.0f64; rows];
    let mut row_b = vec![0.0f64; rows];
    let badge = js::round(size * 1.25);
    for g in 0..g_count {
        c0[g] = col_at(grid.group_col[g]);
        c1[g] = col_at(grid.group_col[g] + grid.group_cols[g] - 1);
        r0[g] = row_at(grid.group_row[g]);
        r1[g] = row_at(grid.group_row[g] + grid.group_rows[g] - 1);
        let room = if tight { js::max(80.0, title_room(options) / 2.0) } else { 320.0 };
        let label = layout_label(groups[g].title.as_deref(), true, &m, size, room, 0);
        let icon = if groups[g].icon.as_deref().is_some_and(|i| !i.is_empty()) { badge } else { 0.0 };
        head_w[g] = label.width + icon + if label.width > 0.0 && icon > 0.0 { 6.0 } else { 0.0 };
        head_h[g] = js::max(label.height, icon);
        group_labels.push(label);
    }
    for g in (0..g_count).rev() {
        let left = group_pad + ins_l[g];
        let mut right = group_pad + ins_r[g];
        let top = (if head_h[g] > 0.0 { js::max(group_pad, head_h[g] + 12.0) } else { group_pad }) + ins_t[g];
        let bottom = group_pad + ins_b[g];
        if head_w[g] > 0.0 {
            let mut inner = left + right;
            let mut c = c0[g];
            while c <= c1[g] {
                inner += col_w[c] + if c < c1[g] { col_r[c] + gap_x[c] + col_l[c + 1] } else { 0.0 };
                c += 1;
            }
            if head_w[g] + 20.0 > inner {
                right += (head_w[g] + 20.0 - inner).ceil();
            }
        }
        ins_l[g] = left;
        ins_r[g] = right;
        ins_t[g] = top;
        ins_b[g] = bottom;
        col_l[c0[g]] = js::max(col_l[c0[g]], left);
        col_r[c1[g]] = js::max(col_r[c1[g]], right);
        row_t[r0[g]] = js::max(row_t[r0[g]], top);
        row_b[r1[g]] = js::max(row_b[r1[g]], bottom);
        let p = grid.group_parent[g];
        if p < 0 {
            continue;
        }
        let p = p as usize;
        if c0[p] == c0[g] {
            ins_l[p] = js::max(ins_l[p], left);
        }
        if c1[p] == c1[g] {
            ins_r[p] = js::max(ins_r[p], right);
        }
        if r0[p] == r0[g] {
            ins_t[p] = js::max(ins_t[p], top);
        }
        if r1[p] == r1[g] {
            ins_b[p] = js::max(ins_b[p], bottom);
        }
    }

    let mut xs = vec![0.0f64; cols];
    let mut ys = vec![0.0f64; rows];
    let mut inner_w = 0.0;
    for c in 0..cols {
        xs[c] = inner_w + col_l[c];
        inner_w = xs[c] + col_w[c] + col_r[c] + if c + 1 < cols { gap_x[c] } else { 0.0 };
    }
    let mut inner_h = 0.0;
    for r in 0..rows {
        ys[r] = inner_h + row_t[r];
        inner_h = ys[r] + row_h[r] + row_b[r] + if r + 1 < rows { gap_y[r] } else { 0.0 };
    }
    // What is drawn, which an edge that has to go around a node can push past the grid.
    let mut min_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_x: f64 = inner_w;
    let mut max_y: f64 = inner_h;

    let left = |g: usize| xs[c0[g]] - ins_l[g];
    let right = |g: usize| xs[c1[g]] + col_w[c1[g]] + ins_r[g];
    let top = |g: usize| ys[r0[g]] - ins_t[g];
    let bottom = |g: usize| ys[r1[g]] + row_h[r1[g]] + ins_b[g];
    let center_x = |i: usize| xs[node_col[i]] + col_w[node_col[i]] / 2.0;

    let port = |i: usize, side: usize, via_group: bool| -> Port {
        let dx = DX[side];
        let dy = DY[side];
        let cx = center_x(i);
        let y = ys[node_row[i]];
        let mut p = Port {
            x: cx,
            y: y + half,
            dx,
            dy,
            x0: cx - box_w[i] / 2.0,
            y0: y,
            x1: cx + box_w[i] / 2.0,
            y1: y + box_h[i],
            gap: 0.0,
            wire: false,
        };
        if nodes[i].kind == ArchNodeKind::Junction {
            p.x0 = cx;
            p.x1 = cx;
            p.y0 = p.y;
            p.y1 = p.y;
            p.gap = DOT;
        } else if dx != 0.0 {
            p.x = cx + dx * half;
        } else {
            p.y = if dy < 0.0 { y } else { y + box_h[i] + if labels[i].height > 0.0 { 2.0 } else { 0.0 } };
        }
        let g = if via_group { grid.node_parent[i] } else { -1 };
        if g >= 0 {
            let g = g as usize;
            p.x0 = left(g);
            p.x1 = right(g);
            p.y0 = top(g);
            p.y1 = bottom(g);
            p.gap = 0.0;
            p.wire = nodes[i].kind == ArchNodeKind::Junction;
            if dx != 0.0 {
                p.x = if dx < 0.0 { p.x0 } else { p.x1 };
            } else {
                p.y = if dy < 0.0 { p.y0 } else { p.y1 };
            }
        }
        p
    };

    let arrow = ctx.paint(Role::Arrow, "var(--_l)");
    let patch = ctx.paint(Role::Background, "var(--_bg)");
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    // Vertical runs of the edges, to keep group titles out from under them.
    let mut runs: Vec<f64> = Vec::new();
    let trim_length = marker_trim("arrow_point");
    for (k, edge) in model.edges.iter().enumerate() {
        let Some((a, b)) = ends[k] else { continue };
        let mut s = port(a, side_index(edge.lhs_dir), edge.lhs_group);
        let mut e = port(b, side_index(edge.rhs_dir), edge.rhs_group);
        if s.x == e.x && s.y == e.y && s.dx == e.dx {
            // An edge from a side back to the same side: part the two ends so it shows as a loop.
            s.x -= 8.0 * s.dy.abs();
            s.y -= 8.0 * s.dx.abs();
            e.x += 8.0 * s.dy.abs();
            e.y += 8.0 * s.dx.abs();
        }
        let mut routed = route(&s, &e);
        if s.wire {
            routed.splice(0..0, [center_x(a), ys[node_row[a]] + half]);
        }
        if e.wire {
            routed.push(center_x(b));
            routed.push(ys[node_row[b]] + half);
        }
        let points = simplify(&routed);
        if points.len() < 4 {
            continue;
        }
        let id = esc_text(&format!("{}-{}", edge.lhs_id, edge.rhs_id)).into_owned();
        let start_trim = if edge.lhs_into && !s.wire { trim_length + s.gap } else { 0.0 };
        let end_trim = if edge.rhs_into && !e.wire { trim_length + e.gap } else { 0.0 };
        svg_write!(edges_out, "<g class=\"pele-edge\" data-id=\"", id, "\"><path d=\"", path_data(&points, start_trim, end_trim), "\"/>")
            .unwrap();
        if edge.lhs_into {
            svg_write!(edges_out, marker("arrow_point", s.x + s.dx * s.gap, s.y + s.dy * s.gap, -s.dx, -s.dy, arrow)).unwrap();
        }
        if edge.rhs_into {
            svg_write!(edges_out, marker("arrow_point", e.x + e.dx * e.gap, e.y + e.dy * e.gap, -e.dx, -e.dy, arrow)).unwrap();
        }
        edges_out.push_str("</g>");
        let mut best = 0usize;
        let mut best_length = -1.0;
        let mut flat = 0usize;
        let mut flat_length = -1.0;
        let label = &edge_labels[k];
        let mut i = 0;
        while i < points.len() {
            min_x = js::min(min_x, points[i] - 4.0);
            max_x = js::max(max_x, points[i] + 4.0);
            min_y = js::min(min_y, points[i + 1] - 4.0);
            max_y = js::max(max_y, points[i + 1] + 4.0);
            i += 2;
        }
        let mut i = 0;
        while i + 3 < points.len() {
            let horizontal = (points[i + 1] - points[i + 3]).abs() < 0.01;
            let length = (points[i] - points[i + 2]).abs() + (points[i + 1] - points[i + 3]).abs();
            if !horizontal {
                runs.push(points[i]);
                runs.push(js::min(points[i + 1], points[i + 3]));
                runs.push(js::max(points[i + 1], points[i + 3]));
            }
            if length > best_length {
                best = i;
                best_length = length;
            }
            if horizontal && length > flat_length {
                flat = i;
                flat_length = length;
            }
            i += 2;
        }
        if label.width > 0.0 {
            let at = if flat_length >= label.width + 8.0 { flat } else { best };
            let mut x = (points[at] + points[at + 2]) / 2.0;
            let mut y = (points[at + 1] + points[at + 3]) / 2.0;
            let w = label.width + 8.0;
            let upright = (points[at] - points[at + 2]).abs() < 0.01;
            // On a short run next to a node the label would land on the node, so it moves off
            // to the side.
            for p in [&s, &e] {
                if x + w / 2.0 <= p.x0 || x - w / 2.0 >= p.x1 || y + label.height / 2.0 <= p.y0 || y - label.height / 2.0 >= p.y1 {
                    continue;
                }
                if upright {
                    x = if x >= (p.x0 + p.x1) / 2.0 { p.x1 + w / 2.0 + 4.0 } else { p.x0 - w / 2.0 - 4.0 };
                } else {
                    y = if y >= (p.y0 + p.y1) / 2.0 { p.y1 + label.height / 2.0 + 4.0 } else { p.y0 - label.height / 2.0 - 4.0 };
                }
            }
            min_x = js::min(min_x, x - w / 2.0);
            max_x = js::max(max_x, x + w / 2.0);
            min_y = js::min(min_y, y - label.height / 2.0);
            max_y = js::max(max_y, y + label.height / 2.0);
            svg_write!(labels_out, edge_label_svg(&format!("{}-{}", edge.lhs_id, edge.rhs_id), label, x, y, "", icons, patch)).unwrap();
        }
    }

    let mut clusters = String::new();
    // Zebra's group fill is a finished color, so it is not thinned as Pele thins its surface.
    let group_fill = if ctx.resolved() {
        svg_format!("fill=\"", ctx.paint(Role::GroupFill, "var(--_a)"), "\"")
    } else {
        "fill=\"var(--_a)\" fill-opacity=\"0.5\"".to_string()
    };
    for g in 0..g_count {
        let group = groups[g];
        let x = left(g);
        let y = top(g);
        let w = right(g) - x;
        let label = &group_labels[g];
        let mut head = String::new();
        if head_w[g] > 0.0 {
            // The title starts at the left, or after the edges that come down through that
            // corner.
            let band = y + head_h[g] + 12.0;
            let mut crossings: Vec<f64> = Vec::new();
            let mut i = 0;
            while i < runs.len() {
                if runs[i + 1] < band && runs[i + 2] > y && runs[i] > x && runs[i] < x + w {
                    crossings.push(runs[i]);
                }
                i += 3;
            }
            crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            crossings.push(x + w - 4.0);
            let mut start = x + 10.0;
            for &crossing in &crossings {
                if crossing - 6.0 - start >= head_w[g] {
                    break;
                }
                start = js::max(start, crossing + 6.0);
            }
            if start + head_w[g] > x + w - 4.0 {
                start = x + 10.0;
            }
            let cy = y + 6.0 + head_h[g] / 2.0;
            let mut text_x = start;
            if let Some(icon) = group.icon.as_deref().filter(|i| !i.is_empty()) {
                head.push_str(&icon_slot(icon, start, cy - badge / 2.0, badge, icons, &glyph_color));
                text_x += badge + 6.0;
            }
            svg_write!(
                head,
                label_svg(
                    label,
                    text_x + label.width / 2.0,
                    cy,
                    &svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\""),
                    icons,
                )
            )
            .unwrap();
        }
        svg_write!(
            clusters,
            "<g class=\"pele-cluster\" data-id=\"",
            esc_text(&group.id),
            "\"><rect x=\"",
            num(x),
            "\" y=\"",
            num(y),
            "\" width=\"",
            num(w),
            "\" height=\"",
            num(bottom(g) - y),
            "\"",
            crate::svg::theme::rounded(&svg_format!(" ", group_fill, " stroke=\"", ctx.paint(Role::GroupStroke, "var(--_b)"), "\"")),
            "/>",
            head,
            "</g>"
        )
        .unwrap();
    }

    let mut nodes_out = String::new();
    let glyph = icon_size * 0.5;
    let text_size = js::max(6.0, js::round(size * 0.75));
    for (i, node) in nodes.iter().enumerate() {
        let at =
            svg_format!("data-id=\"", esc_text(&node.id), "\" transform=\"translate(", num(center_x(i)), ",", num(ys[node_row[i]]), ")\"");
        if node.kind == ArchNodeKind::Junction {
            svg_write!(
                nodes_out,
                "<g class=\"pele-node pele-junction\" ",
                at,
                "><circle cy=\"",
                num(half),
                "\" r=\"",
                js::number_text(DOT),
                "\" fill=\"",
                ctx.paint(Role::Line, "var(--_l)"),
                "\"/></g>"
            )
            .unwrap();
            continue;
        }
        let mut inner = String::new();
        if let Some(icon) = node.icon.as_deref().filter(|i| !i.is_empty()) {
            inner = icon_slot(icon, -glyph / 2.0, (icon_size - glyph) / 2.0, glyph, icons, &glyph_color);
        } else if let Some(text) = node.icon_text.as_deref().filter(|t| !t.is_empty()) {
            let text = layout_label(Some(text), false, &m, text_size, icon_size - 8.0, 0);
            let line_height = if text.line_height == 0.0 || text.line_height.is_nan() { 1.0 } else { text.line_height };
            let lines = js::max(1.0, ((icon_size - 4.0) / line_height).floor());
            // `lines` is at least 1 and at most the icon size, so it is a small whole number.
            let fitted = fit(&text, icon_size - 8.0, lines as usize, &m);
            inner = svg_format!(label_svg(
                &fitted,
                0.0,
                half,
                &svg_format!(" class=\"pele-icon-text\" font-size=\"", js::number_text(text_size), "\""),
                icons,
            ));
        }
        svg_write!(
            nodes_out,
            "<g class=\"pele-node pele-service\" ",
            at,
            "><rect x=\"",
            num(-half),
            "\" width=\"",
            num(icon_size),
            "\" height=\"",
            num(icon_size),
            "\"",
            crate::svg::theme::rounded(&svg_format!(
                " fill=\"",
                ctx.paint(Role::NodeFill, "var(--_s)"),
                "\" stroke=\"",
                ctx.paint(Role::NodeStroke, "var(--_b)"),
                "\""
            )),
            "/>",
            inner,
            label_svg(&labels[i], 0.0, icon_size + LABEL_GAP + labels[i].height / 2.0, " class=\"pele-label\"", icons),
            "</g>"
        )
        .unwrap();
    }

    let title_max = if tight { title_room(options) } else { 4000.0 };
    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, title_max, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let width = (js::max(max_x - min_x, title.width) + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();
    let ox = js::round((width - (max_x - min_x)) / 2.0 - min_x);
    let oy = js::round(pad + title_height - min_y);
    let mut body = String::new();
    if !clusters.is_empty() {
        svg_write!(body, "<g class=\"pele-clusters\">", clusters, "</g>").unwrap();
    }
    if !edges_out.is_empty() {
        svg_write!(
            body,
            "<g class=\"pele-edges\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            edges_out,
            "</g>"
        )
        .unwrap();
    }
    if !labels_out.is_empty() {
        // Pele lets edge labels take the text color. Zebra writes them in the secondary text
        // color, as its edge labels are.
        let fill = if ctx.resolved() { svg_format!(" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"") } else { String::new() };
        svg_write!(body, "<g class=\"pele-edge-labels\" font-size=\"", js::number_text(small), "\"", fill, ">", labels_out, "</g>")
            .unwrap();
    }
    if !nodes_out.is_empty() {
        svg_write!(body, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    }
    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
    if !body.is_empty() {
        svg_write!(content, "<g transform=\"translate(", js::number_text(ox), ",", js::number_text(oy), ")\">", body, "</g>").unwrap();
    }
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered { svg: svg_document("architecture", width, height, size, ctx, acc, &content), width, height, links: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplifies_routes() {
        assert_eq!(simplify(&[0.0, 0.0, 0.0, 0.0, 10.0, 0.0, 20.0, 0.0, 20.0, 5.0]), vec![0.0, 0.0, 20.0, 0.0, 20.0, 5.0]);
    }

    #[test]
    fn prefixes_keep_whole_characters() {
        assert_eq!(measured_prefix("a😀b", 2), "a😀");
        assert_eq!(measured_prefix("a😀b", 3), "a😀");
        assert_eq!(measured_prefix("a😀b", 1), "a");
        assert_eq!(measured_prefix("ab", 5), "ab");
    }
}
