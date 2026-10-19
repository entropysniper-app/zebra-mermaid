//! XY chart rendering. Ported from Pele `src/diagrams/xychart/render.ts`.
//!
//! Colors by role in resolved mode: series (bars, lines, points and their legend swatches)
//! take the chart palette in their order in the source; the grid is an inner stroke, the axes
//! and the baseline under bars are lines, axis labels are secondary text, and titles, values
//! and legend text take the text color the root gives them.

use super::ticks::{tick_index, tick_step, tick_value, ticks_between};
use super::{PlotType, XAxis, XyChartModel};
use crate::config::{Config, ConfigMap, ConfigValue};
use crate::detect::DiagramType;
use crate::diagrams::common::fit_width::cap_width;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::text::{layout_label, style, Label, Measure};
use crate::theme::Role;
use crate::util::js;

const TICK: f64 = 4.0;
const SWATCH: f64 = 12.0;
const ROW: f64 = 26.0;
const LEGEND_GAP: f64 = 24.0;
const LIMIT: f64 = 1e300;

struct Mark {
    at: f64,
    label: Label,
}

fn on(config: &ConfigMap, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(ConfigValue::as_bool).unwrap_or(fallback)
}

fn dimension(value: Option<&ConfigValue>, fallback: f64) -> f64 {
    match value.and_then(ConfigValue::as_f64) {
        Some(v) if v > 0.0 => js::round(js::min(js::max(v, 100.0), 4000.0)),
        _ => fallback,
    }
}

fn finite(value: Option<f64>) -> Option<f64> {
    match value {
        Some(v) if v.is_finite() => Some(js::min(js::max(v, -LIMIT), LIMIT)),
        _ => None,
    }
}

fn widest(marks: &[Mark]) -> f64 {
    let mut width: f64 = 0.0;
    for mark in marks {
        width = js::max(width, mark.label.width);
    }
    width
}

#[derive(Clone, Copy, PartialEq)]
enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    fn as_str(self) -> &'static str {
        match self {
            Anchor::Start => "start",
            Anchor::Middle => "middle",
            Anchor::End => "end",
        }
    }
}

/// Writes a label with its left edge, center or right edge at x. A plain one-line label keeps
/// its place when the text is wider or narrower than measured.
fn aligned(label: &Label, x: f64, y: f64, anchor: Anchor) -> String {
    if label.lines.len() == 1 {
        let line = &label.lines[0];
        if line.len() == 1 && line[0].icon.is_none() && line[0].style == 0 {
            return format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\">{}</text>",
                num(x),
                num(y + label.size * 0.35),
                anchor.as_str(),
                esc(&line[0].text)
            );
        }
    }
    let shift = match anchor {
        Anchor::Start => label.width / 2.0,
        Anchor::End => -label.width / 2.0,
        Anchor::Middle => 0.0,
    };
    label_svg(label, x + shift, y, "", None)
}

/// Lays out plain text of a chart, as Pele's `text` helper does.
fn text(m: &Measure, raw: Option<&str>, font_size: f64, max: f64) -> Label {
    layout_label(raw, false, m, font_size, max, 0)
}

pub(crate) fn render(model: &XyChartModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if model.plots.is_empty() {
        return Err(Error::semantic("No Plot to render, please provide a plot with some data").with_type(DiagramType::Xychart));
    }
    let none = ConfigMap::new();
    let chart = config.section(DiagramType::Xychart.config_section()).unwrap_or(&none);
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.75);
    let m = ctx.measure();
    let pad = ctx.padding();
    let natural = dimension(chart.get("width"), 700.0);
    let width = cap_width(options, natural, 240.0, 0.0);
    // A narrower chart is not as much shorter, so that the plot keeps room for its marks.
    let height = js::round(dimension(chart.get("height"), 500.0) * js::max(0.6, width / natural));
    let horizontal = match model.orientation {
        Some(o) => o == super::Orientation::Horizontal,
        None => chart.get("chartOrientation").and_then(ConfigValue::as_str) == Some("horizontal"),
    };
    // The value of a bar is always written past its end, which is what showDataLabelOutsideBar
    // asks for in Mermaid. Inside, whether it fits changes from bar to bar and with the width of
    // the chart.
    let show_values = on(chart, "showDataLabel", false);
    let x = &model.x_axis;
    let band = matches!(x, XAxis::Band { .. });
    let plots = &model.plots;
    let line_height = text(&m, Some("0"), small, 4000.0).height;
    let tiny_height = text(&m, Some("0"), tiny, 4000.0).height;

    let title = layout_label(
        if on(chart, "showTitle", true) { model.title.as_deref() } else { None },
        false,
        &m,
        size,
        width - 2.0 * pad,
        style::BOLD,
    );
    let title_space = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };

    let mut legend: Vec<(usize, Label)> = Vec::new();
    let mut legend_width: f64 = 0.0;
    if on(chart, "showLegend", true) {
        for (index, plot) in plots.iter().enumerate() {
            if plot.title.is_empty() {
                continue;
            }
            let label = text(&m, Some(&plot.title), small, width * 0.3);
            legend_width = js::max(legend_width, label.width + SWATCH + 8.0);
            legend.push((index, label));
        }
    }
    let legend_space = if !legend.is_empty() { legend_width + LEGEND_GAP } else { 0.0 };

    // The value range: as written, or taken from the data, where bars stand on zero and the
    // range grows to whole steps.
    let bars = plots.iter().filter(|plot| plot.kind == PlotType::Bar).count();
    let mut lo = model.y_axis.min;
    let mut hi = model.y_axis.max;
    let exact = model.has_set_y_axis && lo.is_finite() && hi.is_finite() && lo != hi;
    if exact {
        lo = js::min(js::max(lo, -LIMIT), LIMIT);
        hi = js::min(js::max(hi, -LIMIT), LIMIT);
    } else {
        lo = f64::INFINITY;
        hi = f64::NEG_INFINITY;
        for plot in plots {
            for &(_, raw) in &plot.data {
                let Some(value) = finite(raw) else { continue };
                if value < lo {
                    lo = value;
                }
                if value > hi {
                    hi = value;
                }
            }
        }
        if lo > hi {
            lo = 0.0;
            hi = 0.0;
        }
        if bars > 0 {
            lo = js::min(lo, 0.0);
            hi = js::max(hi, 0.0);
        }
        if lo == hi {
            if lo == 0.0 {
                hi = 1.0;
            } else {
                lo -= lo.abs() / 2.0;
                hi += hi.abs() / 2.0;
            }
        }
    }

    // The marks of the value axis. When the range comes from the data it grows to whole steps
    // here, before anything is placed on it.
    let value_marks = |lo: &mut f64, hi: &mut f64, length: f64, spacing: f64| -> Vec<Mark> {
        let step = tick_step((*hi - *lo).abs(), js::max(2.0, js::min(10.0, (length / spacing).floor())));
        if !exact {
            *lo = tick_value(step, (tick_index(step, *lo) + 1e-9).floor());
            *hi = tick_value(step, (tick_index(step, *hi) - 1e-9).ceil());
        }
        let (lo, hi) = (*lo, *hi);
        ticks_between(js::min(lo, hi), js::max(lo, hi), step)
            .into_iter()
            .map(|value| Mark { at: (value - lo) / (hi - lo), label: text(&m, Some(&js::to_string(value)), small, 4000.0) })
            .collect()
    };

    // Places along the category axis are fractions of its length. Categories and bars sit in
    // the middle of equal slots; the points of a line on a numeric axis run from end to end.
    let mut count: f64 = 1.0;
    match x {
        XAxis::Band { categories, .. } => count = js::max(categories.len() as f64, 1.0),
        XAxis::Linear { .. } => {
            for plot in plots {
                count = js::max(count, plot.data.len() as f64);
            }
        }
    }
    let slot = 1.0 / count;
    let inset = if band || bars > 0 { slot / 2.0 } else { 0.0 };
    let (x0, x1) = match x {
        XAxis::Linear { min, max, .. } => (finite(Some(*min)), finite(Some(*max))),
        XAxis::Band { .. } => (None, None),
    };
    let ranged = matches!((x0, x1), (Some(a), Some(b)) if a != b);
    let place = |i: usize, n: usize| -> f64 {
        if band {
            (i as f64 + 0.5) * slot
        } else {
            inset
                + (if n > 1 {
                    i as f64 / (n - 1) as f64
                } else if ranged {
                    0.0
                } else {
                    0.5
                }) * (1.0 - 2.0 * inset)
        }
    };

    let category_conf = chart.section("xAxis").unwrap_or(&none);
    let value_conf = chart.section("yAxis").unwrap_or(&none);
    let left_conf = if horizontal { category_conf } else { value_conf };
    let bottom_conf = if horizontal { value_conf } else { category_conf };
    let x_title = match x {
        XAxis::Band { title, .. } | XAxis::Linear { title, .. } => title.as_str(),
    };
    let y_title = model.y_axis.title.as_str();
    let left_title = text(
        &m,
        if on(left_conf, "showTitle", true) { Some(if horizontal { x_title } else { y_title }) } else { None },
        small,
        height * 0.8,
    );
    let bottom_title = text(
        &m,
        if on(bottom_conf, "showTitle", true) { Some(if horizontal { y_title } else { x_title }) } else { None },
        small,
        width * 0.8,
    );
    let left_tick = if on(left_conf, "showTick", true) { TICK } else { 0.0 };
    let bottom_tick = if on(bottom_conf, "showTick", true) { TICK } else { 0.0 };
    let left_labels = on(left_conf, "showLabel", true);
    let bottom_labels = on(bottom_conf, "showLabel", true);

    let wrap_at = if horizontal { width * 0.3 } else { js::max(48.0, (width - 2.0 * pad - 64.0 - legend_space) * slot - 8.0) };
    let categories: Vec<Label> = match x {
        XAxis::Band { categories, .. } if on(category_conf, "showLabel", true) => {
            categories.iter().map(|category| text(&m, Some(category), small, wrap_at)).collect()
        }
        _ => Vec::new(),
    };
    let mut category_width: f64 = 0.0;
    let mut category_height: f64 = 0.0;
    for label in &categories {
        category_width = js::max(category_width, label.width);
        category_height = js::max(category_height, label.height);
    }

    // The labelled places of the category axis. Labels that would run into each other are
    // thinned out.
    let category_marks = |length: f64| -> Vec<Mark> {
        let mut marks = Vec::new();
        if let XAxis::Band { categories: names, .. } = x {
            let need = if categories.is_empty() {
                8.0
            } else if horizontal {
                category_height
            } else {
                category_width + 12.0
            };
            let every = js::max(1.0, (need / (length * slot)).ceil());
            let mut i = 0.0;
            while i < names.len() as f64 {
                let label = categories.get(i as usize).cloned().unwrap_or_else(|| text(&m, None, small, 0.0));
                marks.push(Mark { at: (i + 0.5) * slot, label });
                i += every;
            }
        } else if let (true, Some(x0), Some(x1)) = (ranged, x0, x1) {
            let step = tick_step((x1 - x0).abs(), js::max(2.0, js::min(10.0, (length / if horizontal { 32.0 } else { 80.0 }).floor())));
            for value in ticks_between(js::min(x0, x1), js::max(x0, x1), step) {
                marks.push(Mark {
                    at: inset + ((value - x0) / (x1 - x0)) * (1.0 - 2.0 * inset),
                    label: text(&m, Some(&js::to_string(value)), small, 4000.0),
                });
            }
        } else if let Some(x0) = x0 {
            marks.push(Mark { at: 0.5, label: text(&m, Some(&js::to_string(x0)), small, 4000.0) });
        }
        marks
    };

    // Room past the end of the value axis for what is written beside a point or a bar.
    let mut point_label_width: f64 = 0.0;
    let mut value_width: f64 = 0.0;
    for plot in plots {
        for label in plot.point_labels.iter().flatten() {
            point_label_width = js::max(point_label_width, m.width(label, tiny, 0));
        }
        if show_values && horizontal && plot.kind == PlotType::Bar {
            for &(_, value) in &plot.data {
                let shown = finite(value).map(js::to_string).unwrap_or_default();
                value_width = js::max(value_width, m.width(&shown, tiny, 0));
            }
        }
    }

    let bottom_label_height = if !bottom_labels {
        0.0
    } else if horizontal || !band {
        line_height
    } else {
        category_height
    };
    let bottom_space = bottom_tick
        + (if bottom_label_height > 0.0 { bottom_label_height + 4.0 } else { 0.0 })
        + (if bottom_title.height > 0.0 { bottom_title.height + 4.0 } else { 0.0 });
    let headroom = if horizontal {
        0.0
    } else if point_label_width > 0.0 || show_values {
        tiny_height + 4.0
    } else if left_labels {
        line_height / 2.0
    } else {
        0.0
    };
    let top = pad + title_space + headroom;
    let plot_height = js::max(1.0, height - pad - bottom_space - top);
    let bottom = top + plot_height;

    let left_marks = if horizontal { category_marks(plot_height) } else { value_marks(&mut lo, &mut hi, plot_height, 56.0) };
    let left_label_width = if left_labels { widest(&left_marks) } else { 0.0 };
    let left = pad
        + (if left_title.height > 0.0 { left_title.height + 6.0 } else { 0.0 })
        + (if left_label_width > 0.0 { left_label_width + 6.0 } else { 0.0 })
        + left_tick;
    let beside = js::max(
        if point_label_width > 0.0 { point_label_width + 12.0 } else { 0.0 },
        if value_width > 0.0 { value_width + 8.0 } else { 0.0 },
    );
    let reach = if horizontal { js::min(width * 0.25, beside) } else { 0.0 };
    let mut plot_width = js::max(1.0, width - pad - left - legend_space - reach);
    let bottom_marks = if horizontal { value_marks(&mut lo, &mut hi, plot_width, 80.0) } else { category_marks(plot_width) };
    if bottom_labels && legend_space + reach == 0.0 {
        let mut over: f64 = 0.0;
        for mark in &bottom_marks {
            over = js::max(over, mark.at * plot_width + mark.label.width / 2.0 - plot_width);
        }
        plot_width = js::max(1.0, plot_width - over);
    }
    let right = left + plot_width;

    let px = |c: f64, v: f64| -> f64 {
        if horizontal {
            left + v * plot_width
        } else {
            left + c * plot_width
        }
    };
    let py = |c: f64, v: f64| -> f64 {
        if horizontal {
            top + c * plot_height
        } else {
            bottom - v * plot_height
        }
    };
    let scale = |value: f64| -> f64 { (value - lo) / (hi - lo) };
    let clamp = |v: f64| -> f64 { js::min(js::max(v, 0.0), 1.0) };

    // A line across the plot at a place on the value axis.
    let rule = |v: f64| -> String {
        if horizontal {
            format!("M{},{}V{}", num(px(0.0, v)), num(top), num(bottom))
        } else {
            format!("M{},{}H{}", num(left), num(py(0.0, v)), num(right))
        }
    };

    let mut grid = String::new();
    for mark in if horizontal { &bottom_marks } else { &left_marks } {
        grid.push_str(&rule(mark.at));
    }

    let mut axis = String::new();
    let mut labels = String::new();
    if on(left_conf, "showAxisLine", true) {
        axis.push_str(&format!("M{},{}V{}", num(left), num(top), num(bottom)));
    }
    if on(bottom_conf, "showAxisLine", true) {
        axis.push_str(&format!("M{},{}H{}", num(left), num(bottom), num(right)));
    }
    for mark in &left_marks {
        let y = if horizontal { top + mark.at * plot_height } else { bottom - mark.at * plot_height };
        if left_tick != 0.0 {
            axis.push_str(&format!("M{},{}h{}", num(left), num(y), js::to_string(-left_tick)));
        }
        if left_labels {
            labels.push_str(&aligned(&mark.label, left - left_tick - 6.0, y, Anchor::End));
        }
    }
    for mark in &bottom_marks {
        let at = left + mark.at * plot_width;
        if bottom_tick != 0.0 {
            axis.push_str(&format!("M{},{}v{}", num(at), num(bottom), js::to_string(bottom_tick)));
        }
        if bottom_labels {
            labels.push_str(&aligned(&mark.label, at, bottom + bottom_tick + 4.0 + mark.label.height / 2.0, Anchor::Middle));
        }
    }

    let span = (if horizontal { plot_height } else { plot_width }) * slot;
    let group = span * 0.7;
    let bars_f = bars as f64;
    let thickness = group / js::max(bars_f, 1.0);
    let gap = if bars > 1 { js::min(2.0, thickness * 0.2) } else { 0.0 };
    let base = clamp(scale(0.0));
    // Bars that go both ways get a line to stand on.
    if bars > 0 && base > 0.0 && base < 1.0 {
        axis.push_str(&rule(base));
    }
    // Lines are drawn over the bars wherever they were declared, as a bar would hide part of
    // one under it. A series keeps the color of its place in the source.
    let mut series = String::new();
    let mut lines = String::new();
    let mut outside = String::new();
    let mut point_labels = String::new();
    let mut bar = 0.0;
    for (index, plot) in plots.iter().enumerate() {
        let color = ctx.series(index as i64);
        let id = if !plot.title.is_empty() { format!(" data-id=\"{}\"", esc_text(&plot.title)) } else { String::new() };
        // A series declared before the categories can hold more values than there are
        // categories.
        let n = if band { js::min(plot.data.len() as f64, count) as usize } else { plot.data.len() };
        if plot.kind == PlotType::Bar {
            let offset = (bar - (bars_f - 1.0) / 2.0) * thickness - (thickness - gap) / 2.0;
            bar += 1.0;
            let mut rects = String::new();
            for i in 0..n {
                let Some(value) = finite(plot.data[i].1) else { continue };
                let c = place(i, n);
                let v = clamp(scale(value));
                let from = js::min(base, v);
                let to = js::max(base, v);
                let bx = if horizontal { px(c, from) } else { px(c, 0.0) + offset };
                let by = if horizontal { py(c, 0.0) + offset } else { py(c, to) };
                let bw = if horizontal { (to - from) * plot_width } else { thickness - gap };
                let bh = if horizontal { thickness - gap } else { (to - from) * plot_height };
                rects.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>", num(bx), num(by), num(bw), num(bh)));
                if !show_values {
                    continue;
                }
                let label = js::to_string(value);
                let w = m.width(&label, tiny, 0);
                // Towards the end of the bar: 1 when it runs up or right from the baseline, -1
                // the other way.
                let out: f64 = if v >= base { 1.0 } else { -1.0 };
                let mut tx = bx + bw / 2.0;
                let mut ty = by + bh / 2.0;
                let mut anchor = "middle";
                if horizontal {
                    if bh < tiny {
                        continue;
                    }
                    tx = (if out > 0.0 { bx + bw } else { bx }) + out * 6.0;
                    anchor = if out > 0.0 { "start" } else { "end" };
                } else {
                    if w + 2.0 > span / bars_f {
                        continue;
                    }
                    ty = (if out > 0.0 { by } else { by + bh }) - out * (tiny_height / 2.0 + 2.0);
                }
                outside.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\">{}</text>",
                    num(tx),
                    num(ty + tiny * 0.35),
                    anchor,
                    label
                ));
            }
            series.push_str(&format!("<g class=\"pele-series pele-bars\"{} fill=\"{}\">{}</g>", id, color, rects));
            continue;
        }

        // A line is cut where it leaves the value range, so it stays inside the plot.
        let mut d = String::new();
        let mut dots = String::new();
        let mut pen = false;
        let mut pc = 0.0;
        let mut pv = f64::NAN;
        for i in 0..n {
            let Some(value) = finite(plot.data[i].1) else {
                pv = f64::NAN;
                continue;
            };
            let c = place(i, n);
            let v = scale(value);
            let visible = (0.0..=1.0).contains(&v);
            if pv.is_nan() {
                if visible {
                    d.push_str(&format!("M{},{}", num(px(c, v)), num(py(c, v))));
                }
                pen = visible;
            } else {
                let dv = v - pv;
                let mut t0 = 0.0;
                let mut t1 = 1.0;
                if dv != 0.0 {
                    let a = -pv / dv;
                    let b = (1.0 - pv) / dv;
                    t0 = js::max(t0, js::min(a, b));
                    t1 = js::min(t1, js::max(a, b));
                } else if !visible {
                    t0 = 2.0;
                }
                if t0 <= t1 {
                    let c0 = pc + (c - pc) * t0;
                    let v0 = pv + dv * t0;
                    let c1 = pc + (c - pc) * t1;
                    let v1 = pv + dv * t1;
                    if !pen || t0 > 0.0 {
                        d.push_str(&format!("M{},{}", num(px(c0, v0)), num(py(c0, v0))));
                    }
                    d.push_str(&format!("L{},{}", num(px(c1, v1)), num(py(c1, v1))));
                }
                pen = visible;
            }
            pc = c;
            pv = v;
            if !visible {
                continue;
            }
            let cx = px(c, v);
            let cy = py(c, v);
            if span >= 12.0 {
                dots.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"3\"/>", num(cx), num(cy)));
            }
            let raw = plot.point_labels.as_ref().and_then(|l| l.get(i)).map(String::as_str);
            let label = text(&m, raw, tiny, 4000.0);
            if label.height > 0.0 {
                let half = label.width / 2.0;
                point_labels.push_str(&if horizontal {
                    aligned(&label, cx + 10.0, cy, Anchor::Start)
                } else {
                    aligned(&label, js::max(js::min(cx, right - half), left + half), cy - 6.0 - label.height / 2.0, Anchor::Middle)
                });
            }
        }
        lines.push_str(&format!("<g class=\"pele-series pele-line\"{} fill=\"{}\">", id, color));
        if !d.is_empty() {
            lines.push_str(&format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                d, color
            ));
        }
        lines.push_str(&dots);
        lines.push_str("</g>");
    }

    let mut titles = String::new();
    if left_title.height > 0.0 {
        titles.push_str(&format!(
            "<g transform=\"translate({},{}) rotate(-90)\">{}</g>",
            num(pad + left_title.height / 2.0),
            num(top + plot_height / 2.0),
            label_svg(&left_title, 0.0, 0.0, "", None)
        ));
    }
    titles.push_str(&label_svg(&bottom_title, left + plot_width / 2.0, height - pad - bottom_title.height / 2.0, "", None));

    let mut legend_out = String::new();
    let mut legend_height = 0.0;
    for (_, label) in &legend {
        legend_height += js::max(ROW, label.height + 6.0);
    }
    let mut ly = top + js::max(0.0, (plot_height - legend_height) / 2.0);
    let lx = right + reach + LEGEND_GAP;
    for (index, label) in &legend {
        let row = js::max(ROW, label.height + 6.0);
        if ly + row > height {
            break;
        }
        let y = ly + row / 2.0;
        let color = ctx.series(*index as i64);
        let plot = &plots[*index];
        legend_out.push_str(&format!("<g class=\"pele-legend-item\" data-id=\"{}\">", esc_text(&plot.title)));
        if plot.kind == PlotType::Bar {
            legend_out.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{}\"/>",
                num(lx),
                num(y - SWATCH / 2.0),
                color
            ));
        } else {
            legend_out.push_str(&format!(
                "<path d=\"M{},{}h12\" stroke=\"{}\" stroke-width=\"2\" stroke-linecap=\"round\"/>",
                num(lx),
                num(y),
                color
            ));
        }
        legend_out.push_str(&label_svg(label, lx + SWATCH + 8.0 + label.width / 2.0, y, "", None));
        legend_out.push_str("</g>");
        ly += row;
    }

    let small_s = js::to_string(small);
    let tiny_s = js::to_string(tiny);
    let mut content = label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None);
    if !grid.is_empty() {
        content.push_str(&format!("<path class=\"pele-grid\" d=\"{}\" stroke=\"{}\"/>", grid, ctx.paint(Role::InnerStroke, "var(--_a)")));
    }
    content.push_str(&format!("<g class=\"pele-plot\">{}{}</g>", series, lines));
    if !axis.is_empty() {
        content.push_str(&format!(
            "<path class=\"pele-axis\" d=\"{}\" fill=\"none\" stroke=\"{}\"/>",
            axis,
            ctx.paint(Role::Line, "var(--_b)")
        ));
    }
    if !labels.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-axis-labels\" font-size=\"{}\" fill=\"{}\">{}</g>",
            small_s,
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            labels
        ));
    }
    if !titles.is_empty() {
        content.push_str(&format!("<g class=\"pele-axis-titles\" font-size=\"{}\">{}</g>", small_s, titles));
    }
    if !outside.is_empty() {
        content.push_str(&format!("<g class=\"pele-data-labels\" font-size=\"{}\">{}</g>", tiny_s, outside));
    }
    if !point_labels.is_empty() {
        content.push_str(&format!("<g class=\"pele-point-labels\" font-size=\"{}\">{}</g>", tiny_s, point_labels));
    }
    if !legend_out.is_empty() {
        content.push_str(&format!("<g class=\"pele-legend\" font-size=\"{}\">{}</g>", small_s, legend_out));
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered { svg: svg_document("xychart", width, height, size, ctx, acc, &content), width, height, links: Vec::new() })
}
