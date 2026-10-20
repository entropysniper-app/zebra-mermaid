//! Wardley map rendering. Ported from Pele `src/diagrams/wardley/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::{HashMap, HashSet};
use std::f64::consts::{PI, SQRT_2};

use super::{Flow, WardleyModel, WardleyNode};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::fit_width;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg, render_shared_style as shared_style, span_style, text_style};
use crate::svg::edges::{marker, marker_trim};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::{layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math};
use std::fmt::Write as _;

const DEFAULT_STAGES: [&str; 4] = ["Genesis", "Custom Built", "Product", "Commodity"];
/// Mermaid's canvas has this much room around the plot on every side.
const CANVAS_MARGIN: f64 = 48.0;
const R: f64 = 6.0;
const SQUARE: f64 = 10.0;
const LABEL_OFFSET: f64 = 8.0;
const PIPELINE_PAD: f64 = 15.0;
const FORCE_WIDTH: f64 = 60.0;
const FORCE_HEIGHT: f64 = 30.0;
const FORCE_HEAD: f64 = 20.0;
const ACCENT: &str = "var(--_c)";
const LINE: &str = "var(--_l)";

/// A node where it is drawn. A pipeline moves its parent onto the pipeline's box.
struct Placed<'a> {
    x: f64,
    y: f64,
    node: &'a WardleyNode,
}

fn setting(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("wardley-beta", key) {
        Some(ConfigValue::Number(v)) if *v >= min && *v <= max => *v,
        _ => fallback,
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Writes a label with its lines starting at x and its first baseline at y.
fn start_label(label: &Label, x: f64, y: f64, attrs: &str) -> String {
    let shared = shared_style(label);
    let mut body = String::new();
    let mut baseline = y;
    for line in &label.lines() {
        let mut first = true;
        for span in line {
            if span.icon.is_some() {
                continue;
            }
            let pos = if first { svg_format!(" x=\"", num(x), "\" y=\"", num(baseline), "\"") } else { String::new() };
            svg_write!(body, "<tspan", pos, span_style(span.style & !shared), ">", esc(&span.text), "</tspan>").unwrap();
            first = false;
        }
        baseline += label.line_height;
    }
    if body.is_empty() {
        String::new()
    } else {
        svg_format!("<text", text_style(attrs, shared), " xml:space=\"preserve\">", body, "</text>")
    }
}

fn dimension(config: &Config, given: Option<f64>, key: &str, fallback: f64) -> f64 {
    let value = match given {
        Some(g) if g.is_finite() => g,
        _ => setting(config, key, fallback, 1.0, 1e6),
    };
    clamp(value, 200.0, 10000.0) - 2.0 * CANVAS_MARGIN
}

pub(crate) fn render(model: &WardleyModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let natural = dimension(config, model.size.map(|s| s.0), "width", 900.0);
    fit_width(ctx.options, natural, 160.0, |plot_w, tight| Ok(draw(model, config, ctx, plot_w, natural, tight)))
}

/// The box drawn so far, which the plot starts.
struct Extent {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Extent {
    fn grow(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        if x0 < self.min_x {
            self.min_x = x0;
        }
        if y0 < self.min_y {
            self.min_y = y0;
        }
        if x1 > self.max_x {
            self.max_x = x1;
        }
        if y1 > self.max_y {
            self.max_y = y1;
        }
    }
}

fn draw(model: &WardleyModel, config: &Config, ctx: &RenderCtx, plot_w: f64, natural_w: f64, tight: bool) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.8125);
    let tiny = js::round(size * 0.6875);
    let small_s = js::to_string(small);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let icons = ctx.icons();
    let resolved = ctx.resolved();
    // A fill Pele leaves to inheritance, written in resolved mode only.
    let own_fill = |role: Role| if resolved { svg_format!(" fill=\"", ctx.theming.resolved.color(role), "\"") } else { String::new() };
    let line = ctx.paint(Role::Line, LINE);
    let accent = ctx.paint(Role::Accent, ACCENT);
    let node_stroke = ctx.paint(Role::NodeStroke, LINE);

    // A narrower map is not as much shorter, so that components keep some room between them.
    let plot_h = dimension(config, model.size.map(|s| s.1), "height", 600.0) * js::max(0.75, plot_w / natural_w);
    let show_grid = matches!(config.lookup("wardley-beta", "showGrid"), Some(ConfigValue::Bool(true)));

    let mut ext = Extent { min_x: 0.0, min_y: 0.0, max_x: plot_w, max_y: plot_h };
    let centered = |ext: &mut Extent, label: &Label, cx: f64, cy: f64, attrs: &str| -> String {
        ext.grow(cx - label.width / 2.0, cy - label.height / 2.0, cx + label.width / 2.0, cy + label.height / 2.0);
        label_svg(label, cx, cy, attrs, icons).to_string()
    };
    let starting = |ext: &mut Extent, label: &Label, x: f64, y: f64, attrs: &str| -> String {
        ext.grow(x, y - label.size, x + label.width, y - label.size + label.height + 4.0);
        start_label(label, x, y, attrs)
    };
    let default_width = if tight { js::max(64.0, js::min(120.0, plot_w / 2.0)) } else { 4000.0 };
    let text =
        |raw: &str, font_size: f64, max_width: f64, st: u8| -> Label { layout_label(Some(raw), false, &m, font_size, max_width, st) };
    let project_x = |value: f64| (value / 100.0) * plot_w;
    let project_y = |value: f64| plot_h - (value / 100.0) * plot_h;

    let stages: Vec<&str> = match &model.axes.stages {
        Some(s) if !s.is_empty() => s.iter().map(String::as_str).collect(),
        _ => DEFAULT_STAGES.to_vec(),
    };
    let boundaries = model.axes.stage_boundaries.as_deref();
    let custom = boundaries.is_some_and(|b| b.len() == stages.len());
    let mut frame = String::new();
    let mut stage_labels = String::new();
    let mut stage_bottom = plot_h;
    let mut previous = 0.0;
    let count = stages.len() as f64;
    for (index, stage) in stages.iter().enumerate() {
        let start = if custom { previous } else { index as f64 / count };
        let end = match boundaries {
            Some(b) if custom => clamp(b[index], 0.0, 1.0),
            _ => (index as f64 + 1.0) / count,
        };
        previous = end;
        if index > 0 {
            svg_write!(
                frame,
                "<path class=\"pele-wardley-stage\" d=\"M",
                num(start * plot_w),
                ",0V",
                num(plot_h),
                "\" stroke=\"",
                ctx.paint(Role::InnerStroke, "var(--_b)"),
                "\" stroke-dasharray=\"4 4\"/>"
            )
            .unwrap();
        }
        let label = text(stage, small, js::max(48.0, (end - start).abs() * plot_w - 12.0), 0);
        stage_labels.push_str(&centered(
            &mut ext,
            &label,
            ((start + end) / 2.0) * plot_w,
            plot_h + 8.0 + label.height / 2.0,
            &svg_format!(
                " class=\"pele-wardley-stage-label\" font-size=\"",
                small_s,
                "\" fill=\"",
                ctx.paint(Role::SecondaryText, "var(--_m)"),
                "\""
            ),
        ));
        stage_bottom = js::max(stage_bottom, plot_h + 8.0 + label.height);
    }
    if show_grid {
        for i in 1..4 {
            let i = i as f64;
            svg_write!(
                frame,
                "<path class=\"pele-wardley-grid\" d=\"M",
                num((plot_w * i) / 4.0),
                ",0V",
                num(plot_h),
                "M0,",
                num((plot_h * i) / 4.0),
                "H",
                num(plot_w),
                "\" stroke=\"",
                ctx.paint(Role::InnerStroke, "var(--_b)"),
                "\" stroke-dasharray=\"2 6\"/>"
            )
            .unwrap();
        }
    }
    svg_write!(frame, "<path class=\"pele-wardley-axis\" d=\"M0,0V", num(plot_h), "H", num(plot_w), "\" stroke=\"", line, "\"/>").unwrap();
    let x_title = text(model.axes.x_label.as_deref().unwrap_or("Evolution"), small, default_width, 0);
    let y_title = text(model.axes.y_label.as_deref().unwrap_or("Visibility"), small, default_width, 0);
    // Pele lets the axis titles take the text color; Zebra writes axis text in the secondary
    // text color.
    let axis_attrs = svg_format!(
        " class=\"pele-wardley-axis-label\" font-size=\"",
        small_s,
        "\" font-weight=\"var(--_hw)\"",
        own_fill(Role::SecondaryText)
    );
    let mut axis_labels = centered(&mut ext, &x_title, plot_w / 2.0, stage_bottom + 6.0 + x_title.height / 2.0, &axis_attrs);
    write!(
        axis_labels,
        "{}",
        label_svg(&y_title, -16.0, plot_h / 2.0, &svg_format!(axis_attrs, " transform=\"rotate(-90 -16 ", num(plot_h / 2.0), ")\""), None,)
    )
    .unwrap();
    ext.grow(-16.0 - y_title.height / 2.0, 0.0, 0.0, 0.0);
    let legend_top = stage_bottom + 6.0 + x_title.height + 10.0;

    let mut positions: Vec<Placed> = Vec::with_capacity(model.nodes.len());
    let mut position_of: HashMap<&str, usize> = HashMap::new();
    for node in &model.nodes {
        let placed = Placed { x: project_x(node.x.unwrap_or(f64::NAN)), y: project_y(node.y.unwrap_or(f64::NAN)), node };
        match position_of.get(node.id.as_str()) {
            Some(&k) => positions[k] = placed,
            None => {
                position_of.insert(node.id.as_str(), positions.len());
                positions.push(placed);
            }
        }
    }

    // Pipelines: a box around the components, with the parent as a square on its top edge.
    let mut pipelines = String::new();
    let mut members: HashMap<&str, HashSet<&str>> = HashMap::new();
    for pipeline in &model.pipelines {
        members.insert(pipeline.node_id.as_str(), pipeline.component_ids.iter().map(String::as_str).collect());
        let mut parts: Vec<usize> = pipeline.component_ids.iter().filter_map(|id| position_of.get(id.as_str()).copied()).collect();
        if parts.is_empty() {
            continue;
        }
        parts.sort_by(|&a, &b| (positions[a].x - positions[b].x).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal));
        let left = positions[parts[0]].x;
        let right = positions[parts[parts.len() - 1]].x;
        let y = positions[parts[0]].y;
        let top = y - 2.0 * R;
        if let Some(&p) = position_of.get(pipeline.node_id.as_str()) {
            positions[p].x = (left + right) / 2.0;
            positions[p].y = top - SQUARE / 6.0;
        }
        ext.grow(left - PIPELINE_PAD, top, right + PIPELINE_PAD, top + 4.0 * R);
        svg_write!(
            pipelines,
            "<g class=\"pele-wardley-pipeline\" data-id=\"",
            esc_text(&pipeline.node_id),
            "\"><rect x=\"",
            num(left - PIPELINE_PAD),
            "\" y=\"",
            num(top),
            "\" width=\"",
            num(right - left + 2.0 * PIPELINE_PAD),
            "\" height=\"",
            num(4.0 * R),
            "\" rx=\"",
            RADIUS,
            "\" fill=\"",
            ctx.paint(Role::GroupFill, "var(--_s)"),
            "\" stroke=\"",
            ctx.paint(Role::GroupStroke, "var(--_b)"),
            "\"/>"
        )
        .unwrap();
        if parts.len() > 1 {
            svg_write!(
                pipelines,
                "<path d=\"M",
                num(left),
                ",",
                num(y),
                "H",
                num(right),
                "\" stroke=\"",
                line,
                "\" stroke-dasharray=\"4 4\"/>"
            )
            .unwrap();
        }
        pipelines.push_str("</g>");
    }

    // How far from a node's center a line to it stops.
    let anchor_room = if tight { js::max(64.0, plot_w / 2.0) } else { 4000.0 };
    let mut anchor_labels: HashMap<usize, Label> = HashMap::new();
    let mut reach = |at: usize, node: &WardleyNode, ux: f64, uy: f64| -> f64 {
        if node.class_name.as_deref() == Some("anchor") {
            let label = anchor_labels.entry(at).or_insert_with(|| text(&node.label, small, anchor_room, style::BOLD));
            return 1.0 / math::hypot(ux / (label.width / 2.0 + 5.0), uy / (small * 0.8));
        }
        if node.is_pipeline_parent {
            return SQUARE / SQRT_2;
        }
        if node.source_strategy.as_deref().is_some_and(|s| !s.is_empty()) {
            2.0 * R
        } else {
            R
        }
    };

    // Quadrants around a node that a link runs through: 1 up and right, 2 down and right, 4 up
    // and left, 8 down and left.
    let mut taken: HashMap<usize, u8> = HashMap::new();
    let mut occupy = |at: usize, ux: f64, uy: f64| {
        if ux.abs() < 0.25 || uy.abs() < 0.25 {
            return;
        }
        let bit = if ux > 0.0 {
            if uy < 0.0 {
                1
            } else {
                2
            }
        } else if uy < 0.0 {
            4
        } else {
            8
        };
        *taken.entry(at).or_insert(0) |= bit;
    };

    let arrow = ctx.paint(Role::Arrow, LINE);
    let trim = marker_trim("arrow_point");
    let mut links = String::new();
    for link in &model.links {
        let (Some(&from), Some(&to)) = (position_of.get(link.source.as_str()), position_of.get(link.target.as_str())) else {
            continue;
        };
        if from == to {
            continue;
        }
        // A pipeline component belongs to its parent already.
        if members.get(link.target.as_str()).is_some_and(|ids| ids.contains(link.source.as_str())) {
            continue;
        }
        let (fx, fy, tx, ty) = (positions[from].x, positions[from].y, positions[to].x, positions[to].y);
        let dx = tx - fx;
        let dy = ty - fy;
        let distance = math::hypot(dx, dy);
        if distance < 0.01 {
            continue;
        }
        let ux = dx / distance;
        let uy = dy / distance;
        let start = reach(from, positions[from].node, ux, uy);
        let end = reach(to, positions[to].node, ux, uy);
        if start + end >= distance {
            continue;
        }
        occupy(from, ux, uy);
        occupy(to, -ux, -uy);
        let mut x1 = fx + ux * start;
        let mut y1 = fy + uy * start;
        let mut x2 = tx - ux * end;
        let mut y2 = ty - uy * end;
        let mut heads = String::new();
        if matches!(link.flow, Some(Flow::Forward | Flow::Bidirectional)) {
            write!(heads, "{}", marker("arrow_point", x2, y2, ux, uy, arrow)).unwrap();
            x2 -= ux * trim;
            y2 -= uy * trim;
        }
        if matches!(link.flow, Some(Flow::Backward | Flow::Bidirectional)) {
            write!(heads, "{}", marker("arrow_point", x1, y1, -ux, -uy, arrow)).unwrap();
            x1 += ux * trim;
            y1 += uy * trim;
        }
        let mut caption = String::new();
        if let Some(text_value) = link.label.as_deref().filter(|l| !l.is_empty()) {
            let label = text(text_value, tiny + 1.0, 200.0, 0);
            // Beside the middle of the line, on its upper side.
            let side = if ux >= 0.0 { 1.0 } else { -1.0 };
            let px = uy * side;
            let py = -ux * side;
            let away = 5.0 + (px.abs() * label.width) / 2.0 + (py.abs() * label.height) / 2.0;
            caption = centered(
                &mut ext,
                &label,
                (fx + tx) / 2.0 + px * away,
                (fy + ty) / 2.0 + py * away,
                &svg_format!(
                    " class=\"pele-edge-label\" font-size=\"",
                    js::to_string(tiny + 1.0),
                    "\" fill=\"",
                    ctx.paint(Role::SecondaryText, "var(--_m)"),
                    "\""
                ),
            );
        }
        svg_write!(
            links,
            "<g class=\"pele-edge",
            if link.dashed { " pele-wardley-dashed" } else { "" },
            "\" data-id=\"",
            esc_text(&link.source),
            "-",
            esc_text(&link.target),
            "\"><path d=\"M",
            num(x1),
            ",",
            num(y1),
            "L",
            num(x2),
            ",",
            num(y2),
            "\" stroke=\"",
            line,
            "\"",
            if link.dashed { " stroke-dasharray=\"6 6\"" } else { "" },
            "/>",
            heads,
            caption,
            "</g>"
        )
        .unwrap();
    }

    // Evolution: a dashed arrow to where the component is heading, drawn as an open circle.
    let mut trends = String::new();
    for trend in &model.trends {
        let Some(&origin) = position_of.get(trend.node_id.as_str()) else { continue };
        let (ox, oy) = (positions[origin].x, positions[origin].y);
        let tx = project_x(trend.target_x);
        let ty = project_y(trend.target_y);
        let dx = tx - ox;
        let dy = ty - oy;
        let distance = math::hypot(dx, dy);
        let mut arrow_out = String::new();
        let start = if distance > 0.0 { reach(origin, positions[origin].node, dx / distance, dy / distance) } else { 0.0 };
        if distance > start + R + 4.0 {
            let ux = dx / distance;
            let uy = dy / distance;
            let ex = tx - ux * (R + 2.0);
            let ey = ty - uy * (R + 2.0);
            arrow_out = svg_format!(
                "<path d=\"M",
                num(ox + ux * start),
                ",",
                num(oy + uy * start),
                "L",
                num(ex - ux * trim),
                ",",
                num(ey - uy * trim),
                "\" stroke=\"",
                accent,
                "\" stroke-dasharray=\"4 4\"/>",
                marker("arrow_point", ex, ey, ux, uy, accent)
            );
        }
        ext.grow(tx - R, ty - R, tx + R, ty + R);
        svg_write!(
            trends,
            "<g class=\"pele-wardley-trend\" data-id=\"",
            esc_text(&trend.node_id),
            "\">",
            arrow_out,
            "<circle cx=\"",
            num(tx),
            "\" cy=\"",
            num(ty),
            "\" r=\"",
            js::to_string(R),
            "\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke=\"",
            accent,
            "\"/></g>"
        )
        .unwrap();
    }

    let mut nodes = String::new();
    let tri = 1.2 * R;
    let dot = 0.7 * R;
    let node_fill = ctx.paint(Role::NodeFill, "var(--_bg)");
    let label_attrs = svg_format!(" class=\"pele-label\" font-size=\"", small_s, "\"");
    for (at, placed) in positions.iter().enumerate() {
        let node = placed.node;
        let (x, y) = (placed.x, placed.y);
        let anchor = node.class_name.as_deref() == Some("anchor");
        let strategy = node.source_strategy.as_deref().filter(|s| !s.is_empty());
        let mut shape = String::new();
        if let Some(strategy) = strategy {
            // The halo of a sourcing strategy: Pele's surface, alternate surface, muted color or
            // canvas, which Zebra has as the node fill, the secondary surface, the secondary
            // text color and the canvas.
            let fill = match strategy {
                "outsource" => ctx.paint(Role::SecondaryText, "var(--_m)"),
                "buy" => ctx.paint(Role::GroupHeader, "var(--_a)"),
                "build" => ctx.paint(Role::NodeFill, "var(--_s)"),
                _ => ctx.paint(Role::Background, "var(--_bg)"),
            };
            svg_write!(
                shape,
                "<circle class=\"pele-wardley-",
                strategy,
                "\" cx=\"",
                num(x),
                "\" cy=\"",
                num(y),
                "\" r=\"",
                js::to_string(2.0 * R),
                "\" fill=\"",
                fill,
                "\" stroke=\"",
                node_stroke,
                "\"/>"
            )
            .unwrap();
        }
        if strategy == Some("market") {
            let bx = tri * math::cos(PI / 6.0);
            let by = y + tri * math::sin(PI / 6.0);
            svg_write!(
                shape,
                "<path d=\"M",
                num(x),
                ",",
                num(y - tri),
                "L",
                num(x - bx),
                ",",
                num(by),
                "H",
                num(x + bx),
                "Z\" fill=\"none\" stroke=\"",
                node_stroke,
                "\"/>"
            )
            .unwrap();
            for (cx, cy) in [(x, y - tri), (x - bx, by), (x + bx, by)] {
                svg_write!(
                    shape,
                    "<circle cx=\"",
                    num(cx),
                    "\" cy=\"",
                    num(cy),
                    "\" r=\"",
                    num(dot),
                    "\" fill=\"",
                    node_fill,
                    "\" stroke=\"",
                    node_stroke,
                    "\"/>"
                )
                .unwrap();
            }
        } else if node.is_pipeline_parent {
            svg_write!(
                shape,
                "<rect x=\"",
                num(x - SQUARE / 2.0),
                "\" y=\"",
                num(y - SQUARE / 2.0),
                "\" width=\"",
                js::to_string(SQUARE),
                "\" height=\"",
                js::to_string(SQUARE),
                "\" fill=\"",
                node_fill,
                "\" stroke=\"",
                node_stroke,
                "\"/>"
            )
            .unwrap();
        } else if !anchor {
            svg_write!(
                shape,
                "<circle cx=\"",
                num(x),
                "\" cy=\"",
                num(y),
                "\" r=\"",
                js::to_string(R),
                "\" fill=\"",
                node_fill,
                "\" stroke=\"",
                node_stroke,
                "\"/>"
            )
            .unwrap();
        }
        if node.inertia == Some(true) {
            let offset = (if node.is_pipeline_parent { SQUARE / 2.0 } else { R }) + 10.0 + if strategy.is_some() { R + 4.0 } else { 0.0 };
            svg_write!(
                shape,
                "<rect class=\"pele-wardley-inertia\" x=\"",
                num(x + offset),
                "\" y=\"",
                num(y - R),
                "\" width=\"4\" height=\"",
                js::to_string(2.0 * R),
                "\" fill=\"",
                line,
                "\"/>"
            )
            .unwrap();
        }
        let ox = node.label_offset_x.map(|v| clamp(v, -5000.0, 5000.0));
        let oy = node.label_offset_y.map(|v| clamp(v, -5000.0, 5000.0));
        let caption = if anchor {
            let label = match anchor_labels.get(&at) {
                Some(label) => label.clone(),
                None => text(&node.label, small, anchor_room, style::BOLD),
            };
            // An anchor is drawn as its name alone, which Zebra marks with the accent color.
            centered(
                &mut ext,
                &label,
                x + ox.unwrap_or(0.0),
                y + oy.unwrap_or(-3.0),
                &svg_format!(" class=\"pele-label\" font-size=\"", small_s, "\" font-weight=\"var(--_hw)\"", own_fill(Role::Accent)),
            )
        } else {
            let label = text(&node.label, small, default_width, 0);
            let lift = LABEL_OFFSET + if strategy.is_some() { 10.0 } else { 0.0 };
            if ox.is_some() || oy.is_some() {
                starting(&mut ext, &label, x + ox.unwrap_or(lift), y + oy.unwrap_or(-lift), &label_attrs)
            } else if node.in_pipeline {
                centered(&mut ext, &label, x, y + 2.0 * R + 4.0 + label.height / 2.0, &label_attrs)
            } else {
                // The label goes up and to the right unless a link leaves that way; then the
                // next free corner.
                let busy = taken.get(&at).copied().unwrap_or(0);
                let corner = if busy & 1 == 0 {
                    1
                } else if busy & 2 == 0 {
                    2
                } else if busy & 4 == 0 {
                    4
                } else if busy & 8 == 0 {
                    8
                } else {
                    1
                };
                let right = corner == 1 || corner == 2;
                let up = corner == 1 || corner == 4;
                starting(
                    &mut ext,
                    &label,
                    if right { x + lift } else { x - lift - label.width },
                    if up { y - lift } else { y + lift + small * 0.75 },
                    &label_attrs,
                )
            }
        };
        ext.grow(x - 2.0 * R, y - 2.0 * R, x + 2.0 * R + 20.0, y + 2.0 * R);
        let kind = if anchor {
            "anchor"
        } else if node.is_pipeline_parent {
            "pipeline-parent"
        } else if node.in_pipeline {
            "pipeline-component"
        } else {
            "component"
        };
        svg_write!(nodes, "<g class=\"pele-node pele-wardley-", kind, "\" data-id=\"", esc_text(&node.id), "\">", shape, caption, "</g>")
            .unwrap();
    }

    let mut annotations = String::new();
    let tiny_s = js::to_string(tiny);
    for annotation in &model.annotations {
        let number = js::to_string(annotation.number);
        let label = text(&number, tiny, 4000.0, style::BOLD);
        let mut marks = String::new();
        for &(cx, cy) in &annotation.coordinates {
            let x = project_x(cx);
            let y = project_y(cy);
            let r = js::max(10.0, label.width / 2.0 + 4.0);
            ext.grow(x - r, y - r, x + r, y + r);
            svg_write!(
                marks,
                "<circle cx=\"",
                num(x),
                "\" cy=\"",
                num(y),
                "\" r=\"",
                num(r),
                "\" fill=\"",
                ctx.paint(Role::Background, "var(--_bg)"),
                "\" stroke=\"",
                line,
                "\"/>"
            )
            .unwrap();
            write!(marks, "{}", label_svg(&label, x, y, &svg_format!(" font-size=\"", tiny_s, "\" font-weight=\"var(--_hw)\""), None))
                .unwrap();
        }
        svg_write!(annotations, "<g class=\"pele-wardley-annotation\" data-id=\"", esc_text(&number), "\">", marks, "</g>").unwrap();
    }
    let mut listed: Vec<_> = model.annotations.iter().filter(|a| a.text.as_deref().is_some_and(|t| !t.is_empty())).collect();
    listed.sort_by(|a, b| (a.number - b.number).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal));
    if !listed.is_empty() {
        let room = if tight { js::max(80.0, plot_w - 20.0) } else { 320.0 };
        let lines: Vec<Label> = listed
            .iter()
            .map(|a| text(&format!("{}. {}", js::to_string(a.number), a.text.as_deref().unwrap_or("")), small, room, 0))
            .collect();
        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;
        for l in &lines {
            width = js::max(width, l.width);
            height += l.height;
        }
        width += 20.0;
        height += 14.0;
        // Without a position the list goes under the map, where it covers nothing.
        let (bx, by) = match model.annotations_box {
            Some((x, y)) => {
                (clamp(project_x(x), 0.0, js::max(0.0, plot_w - width)), clamp(project_y(y), 0.0, js::max(0.0, plot_h - height)))
            }
            None => (0.0, legend_top),
        };
        ext.grow(bx, by, bx + width, by + height);
        let mut rows = String::new();
        let mut y = by + 7.0;
        for l in &lines {
            rows.push_str(&start_label(l, bx + 10.0, y + l.line_height / 2.0 + small * 0.35, &svg_format!(" font-size=\"", small_s, "\"")));
            y += l.height;
        }
        svg_write!(
            annotations,
            "<g class=\"pele-wardley-annotations\"><rect x=\"",
            num(bx),
            "\" y=\"",
            num(by),
            "\" width=\"",
            num(width),
            "\" height=\"",
            num(height),
            "\" rx=\"",
            RADIUS,
            "\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke=\"",
            ctx.paint(Role::InnerStroke, "var(--_b)"),
            "\"/>",
            rows,
            "</g>"
        )
        .unwrap();
    }

    let mut notes = String::new();
    // Pele draws a note as bold text alone; Zebra writes notes in its warning text color.
    let note_attrs =
        svg_format!(" class=\"pele-wardley-note\" font-size=\"", small_s, "\" font-weight=\"var(--_hw)\"", own_fill(Role::WarningText));
    for note in &model.notes {
        let label = text(&note.text, small, js::min(360.0, plot_w * 0.6), style::BOLD);
        notes.push_str(&starting(&mut ext, &label, project_x(note.x), project_y(note.y), &note_attrs));
    }

    // Accelerators point right, deaccelerators left.
    let mut forces = String::new();
    let force_attrs = svg_format!(" font-size=\"", js::to_string(tiny + 1.0), "\" font-weight=\"var(--_hw)\"");
    let mut force = |ext: &mut Extent, name: &str, px: f64, py: f64, forward: bool| {
        let x = project_x(px);
        let y = project_y(py);
        let half = FORCE_HEIGHT / 2.0;
        let tip = if forward { x + FORCE_WIDTH } else { x };
        let tail = if forward { x } else { x + FORCE_WIDTH };
        let neck = if forward { x + FORCE_WIDTH - FORCE_HEAD } else { x + FORCE_HEAD };
        let d = svg_format!(
            "M",
            num(tail),
            ",",
            num(y - half),
            "H",
            num(neck),
            "V",
            num(y - half - 8.0),
            "L",
            num(tip),
            ",",
            num(y),
            "L",
            num(neck),
            ",",
            num(y + half + 8.0),
            "V",
            num(y + half),
            "H",
            num(tail),
            "Z"
        );
        ext.grow(x, y - half - 8.0, x + FORCE_WIDTH, y + half + 8.0);
        let label = text(name, tiny + 1.0, 160.0, style::BOLD);
        let caption = centered(ext, &label, x + FORCE_WIDTH / 2.0, y + half + 8.0 + 12.0, &force_attrs);
        svg_write!(
            forces,
            "<g class=\"pele-wardley-",
            if forward { "accelerator" } else { "deaccelerator" },
            "\" data-id=\"",
            esc_text(name),
            "\"><path d=\"",
            d,
            "\" fill=\"",
            ctx.paint(Role::NodeFill, "var(--_s)"),
            "\" stroke=\"",
            node_stroke,
            "\"/>",
            caption,
            "</g>"
        )
        .unwrap();
    };
    for a in &model.accelerators {
        force(&mut ext, &a.name, a.x, a.y, true);
    }
    for d in &model.deaccelerators {
        force(&mut ext, &d.name, d.x, d.y, false);
    }

    let title_max = if tight { js::max(80.0, plot_w) } else { 4000.0 };
    let title = layout_label(model.title.as_deref(), false, &m, size, title_max, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let left = js::min(ext.min_x, plot_w / 2.0 - title.width / 2.0);
    let right = js::max(ext.max_x, plot_w / 2.0 + title.width / 2.0);
    let tx = pad - left;
    let ty = pad + title_height - ext.min_y;
    let total_width = (right - left + 2.0 * pad).ceil();
    let total_height = (ext.max_y - ext.min_y + title_height + 2.0 * pad).ceil();
    let mut content =
        label_svg(&title, tx + plot_w / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None)
            .to_string();
    svg_write!(content, "<g transform=\"translate(", num(tx), ",", num(ty), ")\">").unwrap();
    svg_write!(content, "<g class=\"pele-wardley-frame\" fill=\"none\">", frame, "</g>").unwrap();
    content.push_str(&stage_labels);
    content.push_str(&axis_labels);
    content.push_str(&pipelines);
    if !links.is_empty() {
        svg_write!(content, "<g class=\"pele-edges\" fill=\"none\">", links, "</g>").unwrap();
    }
    if !trends.is_empty() {
        svg_write!(content, "<g class=\"pele-wardley-trends\" fill=\"none\">", trends, "</g>").unwrap();
    }
    content.push_str(&nodes);
    content.push_str(&annotations);
    content.push_str(&notes);
    content.push_str(&forces);
    content.push_str("</g>");

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("wardley", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
