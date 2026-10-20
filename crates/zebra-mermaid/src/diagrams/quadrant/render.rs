//! Quadrant chart rendering. Ported from Pele `src/diagrams/quadrant/render.ts`.
//!
//! Colors by role in resolved mode: the four quadrants are regions of the chart, filled and
//! outlined as groups; their labels and the point labels over them take the text color that
//! reads on that fill. Points take the first series color, axis labels are secondary text, and
//! the title takes the text color the root gives it. A point's own `color`, `stroke-color` and
//! `stroke-width` are written as Pele writes them, in a `style` attribute.

use super::{QuadrantModel, QuadrantText};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::cap_width;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, label_svg, num};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{class_names, resolve_style, RADIUS};
use crate::text::{layout_label, style, Label, Measure};
use crate::theme::Role;
use crate::util::js;
use std::fmt::Write as _;

const GAP: f64 = 4.0;
const CAREFUL: usize = 300;

struct Dot {
    cx: f64,
    cy: f64,
    reach: f64,
    text: Label,
}

fn setting<'a>(config: &'a Config, key: &str) -> Option<&'a ConfigValue> {
    config.lookup("quadrantChart", key)
}

fn length(config: &Config, key: &str, fallback: f64, max: f64) -> f64 {
    match setting(config, key).and_then(ConfigValue::as_f64) {
        Some(value) if value >= 0.0 => js::min(value, max),
        _ => fallback,
    }
}

fn hex(color: &str) -> String {
    if color.starts_with('#') {
        color.to_string()
    } else {
        format!("#{}", color)
    }
}

fn label(m: &Measure, text: Option<&QuadrantText>, font_size: f64, max: f64) -> Label {
    layout_label(text.map(|t| t.text.as_str()), text.is_some_and(|t| t.markdown), m, font_size, js::max(max, 40.0), 0)
}

pub(crate) fn render(model: &QuadrantModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let axis_size = js::round(size * 0.875);
    let point_size = js::round(size * 0.75);
    let m = ctx.measure();
    let natural = js::round(js::max(100.0, length(config, "chartWidth", 500.0, 4000.0)));
    let width = js::round(cap_width(options, natural, 280.0, 0.0));
    let height = js::round((js::max(100.0, length(config, "chartHeight", 500.0, 4000.0)) * width) / natural);
    let outer = length(config, "quadrantPadding", ctx.padding(), 200.0);
    let x_pad = length(config, "xAxisLabelPadding", 5.0, 200.0);
    let y_pad = length(config, "yAxisLabelPadding", 5.0, 200.0);
    let has_points = !model.points.is_empty();

    let title = layout_label(model.title.as_deref(), false, &m, size, js::max(width - 2.0 * outer, 40.0), style::BOLD);
    let title_pad = length(config, "titlePadding", 10.0, 200.0);
    let title_space = if title.height > 0.0 { title.height + title_pad } else { 0.0 };

    let non_empty = |t: &Option<QuadrantText>| t.as_ref().is_some_and(|t| !t.text.is_empty());
    let has_x = non_empty(&model.x_axis_left) || non_empty(&model.x_axis_right);
    let has_y = non_empty(&model.y_axis_bottom) || non_empty(&model.y_axis_top);
    // Mermaid moves the x axis labels below the chart as soon as there are points.
    let x_below = has_points || setting(config, "xAxisPosition").and_then(ConfigValue::as_str) == Some("bottom");
    let y_right = setting(config, "yAxisPosition").and_then(ConfigValue::as_str) == Some("right");

    let side = width - 2.0 * outer;
    let x_left = label(&m, model.x_axis_left.as_ref(), axis_size, if model.x_axis_right.is_some() { side / 2.0 - 16.0 } else { side });
    let x_right = label(&m, model.x_axis_right.as_ref(), axis_size, side / 2.0 - 16.0);
    let x_height = js::max(x_left.height, x_right.height);
    let x_space = if has_x { x_height + x_pad } else { 0.0 };
    let tall = height - 2.0 * outer - title_space - x_space;
    let y_bottom = label(&m, model.y_axis_bottom.as_ref(), axis_size, if model.y_axis_top.is_some() { tall / 2.0 - 16.0 } else { tall });
    let y_top = label(&m, model.y_axis_top.as_ref(), axis_size, tall / 2.0 - 16.0);
    let y_height = js::max(y_bottom.height, y_top.height);
    let y_space = if has_y { y_height + y_pad } else { 0.0 };

    let left = outer + if y_right { 0.0 } else { y_space };
    let top = outer + title_space + if x_below { 0.0 } else { x_space };
    let w = js::max(0.0, side - y_space);
    let h = js::max(0.0, tall);
    let tile_w = js::max(0.0, (w - GAP) / 2.0);
    let tile_h = js::max(0.0, (h - GAP) / 2.0);

    // Boxes a point label keeps clear of: the gaps between quadrants, quadrant labels, points,
    // and the labels placed before it.
    let mut taken: Vec<f64> = vec![left + tile_w, top, left + w - tile_w, top + h, left, top + tile_h, left + w, top + h - tile_h];
    let is_clear = |taken: &[f64], from: usize, x0: f64, y0: f64, x1: f64, y1: f64| -> bool {
        let mut k = from;
        while k < taken.len() {
            if x0 < taken[k + 2] && x1 > taken[k] && y0 < taken[k + 3] && y1 > taken[k + 1] {
                return false;
            }
            k += 4;
        }
        true
    };

    // The quadrants are regions of the chart: in resolved mode they take the group colors, and
    // the text over them the color that reads on that fill.
    let region = ctx.paint(Role::GroupFill, "var(--_s)");
    let on_region = ctx.theming.text_on(region, "var(--_fg)");
    let text_top = length(config, "quadrantTextTopPadding", 5.0, 200.0);
    let mut quadrants = String::new();
    for k in 0..4 {
        let x = if k == 0 || k == 3 { left + w - tile_w } else { left };
        let y = if k < 2 { top } else { top + h - tile_h };
        let text = label(&m, model.quadrants[k].as_ref(), size, tile_w - 16.0);
        let cx = x + tile_w / 2.0;
        let cy = if has_points { y + text_top + text.height / 2.0 } else { y + tile_h / 2.0 };
        taken.extend_from_slice(&[cx - text.width / 2.0, cy - text.height / 2.0, cx + text.width / 2.0, cy + text.height / 2.0]);
        svg_write!(
            quadrants,
            "<g class=\"pele-quadrant\" data-id=\"",
            k + 1,
            "\"><rect x=\"",
            num(x),
            "\" y=\"",
            num(y),
            "\" width=\"",
            num(tile_w),
            "\" height=\"",
            num(tile_h),
            "\" rx=\"",
            RADIUS,
            "\"/>",
            label_svg(&text, cx, cy, &svg_format!(" class=\"pele-label\" fill=\"", on_region, "\" stroke=\"none\""), None),
            "</g>"
        )
        .unwrap();
    }

    let mut axes = String::new();
    if has_x {
        let y = if x_below { top + h + x_pad + x_height / 2.0 } else { top - x_pad - x_height / 2.0 };
        write!(axes, "{}", label_svg(&x_left, if x_right.height > 0.0 { left + w / 4.0 } else { left + x_left.width / 2.0 }, y, "", None))
            .unwrap();
        write!(axes, "{}", label_svg(&x_right, left + (w * 3.0) / 4.0, y, "", None)).unwrap();
    }
    if has_y {
        let x = if y_right { left + w + y_pad + y_height / 2.0 } else { left - y_pad - y_height / 2.0 };
        let turned = |text: &Label, y: f64| -> String {
            if text.height > 0.0 {
                svg_format!(
                    "<g transform=\"translate(",
                    num(x),
                    ",",
                    num(y),
                    ") rotate(-90)\">",
                    label_svg(text, 0.0, 0.0, "", None),
                    "</g>"
                )
            } else {
                String::new()
            }
        };
        axes.push_str(&turned(&y_bottom, if y_top.height > 0.0 { top + (h * 3.0) / 4.0 } else { top + h - y_bottom.width / 2.0 }));
        axes.push_str(&turned(&y_top, top + h / 4.0));
    }

    let radius = length(config, "pointRadius", 5.0, 200.0);
    let text_pad = length(config, "pointTextPadding", 5.0, 200.0);
    let fill = ctx.series(0);
    let mut dots: Vec<Dot> = Vec::new();
    let mut circles: Vec<String> = Vec::new();
    for point in &model.points {
        if !(point.x >= 0.0 && point.x <= 1.0 && point.y >= 0.0 && point.y <= 1.0) {
            continue;
        }
        let shared = model.classes.get(&point.class_name);
        let r = js::min(point.style.radius.or_else(|| shared.and_then(|s| s.radius)).unwrap_or(radius), 2000.0);
        let color = point.style.color.as_ref().or_else(|| shared.and_then(|s| s.color.as_ref()));
        let stroke_color = point.style.stroke_color.as_ref().or_else(|| shared.and_then(|s| s.stroke_color.as_ref()));
        let stroke_width = point.style.stroke_width.as_ref().or_else(|| shared.and_then(|s| s.stroke_width.as_ref()));
        let mut styles: Vec<String> = Vec::new();
        if let Some(color) = color.filter(|c| !c.is_empty()) {
            styles.push(format!("fill:{}", hex(color)));
        }
        let stroke_width = stroke_width.filter(|s| !s.is_empty());
        let stroke_color = stroke_color.filter(|s| !s.is_empty());
        if let Some(width) = stroke_width {
            if let Some(color) = stroke_color {
                styles.push(format!("stroke:{}", hex(color)));
            }
            styles.push(format!("stroke-width:{}", width));
        }
        let cx = left + point.x * w;
        let cy = top + (1.0 - point.y) * h;
        dots.push(Dot {
            cx,
            cy,
            reach: r + stroke_width.map_or(0.0, |s| js::min(js::parse_int(s, 0), 400.0) / 2.0),
            text: label(&m, Some(&point.text), point_size, 200.0),
        });
        let own_stroke =
            if stroke_width.is_some() && stroke_color.is_none() { svg_format!(" stroke=\"", fill, "\"") } else { String::new() };
        circles.push(svg_format!(
            "<circle class=\"pele-point",
            class_names(&point.class_name),
            "\" data-id=\"",
            esc_text(&point.text.text),
            "\" cx=\"",
            num(cx),
            "\" cy=\"",
            num(cy),
            "\" r=\"",
            num(r),
            "\"",
            own_stroke,
            resolve_style(&styles).shape,
            "/>"
        ));
    }

    // A label goes below its point, or above, right or left of it when that is the first place
    // that is inside the chart and free. A second round lets it cross the gaps between
    // quadrants. Large charts skip the search for a free place.
    let careful = dots.len() <= CAREFUL;
    if careful {
        for dot in &dots {
            taken.extend_from_slice(&[dot.cx - dot.reach, dot.cy - dot.reach, dot.cx + dot.reach, dot.cy + dot.reach]);
        }
    }
    let mut labels = String::new();
    for Dot { cx, cy, reach, text } in &dots {
        let (cx, cy) = (*cx, *cy);
        let half_w = text.width / 2.0;
        let half_h = text.size * 0.6 + (text.height - text.line_height) / 2.0;
        let away = reach + text_pad;
        let spots = [cx, cy + away + half_h, cx, cy - away - half_h, cx + away + half_w, cy, cx - away - half_w, cy];
        let mut tx = cx;
        let mut ty = spots[1];
        let mut found = false;
        let mut k = 0;
        while k < 16 {
            let spot = k % 8;
            let x = if spot < 4 && text.width < w { js::min(js::max(spots[spot], left + half_w), left + w - half_w) } else { spots[spot] };
            let y = spots[spot + 1];
            if x - half_w < left || x + half_w > left + w || y - half_h < top || y + half_h > top + h {
                k += 2;
                continue;
            }
            let free = !careful || is_clear(&taken, if k < 8 { 0 } else { 8 }, x - half_w, y - half_h, x + half_w, y + half_h);
            if !found || free {
                tx = x;
                ty = y;
                found = true;
            }
            if free {
                break;
            }
            k += 2;
        }
        if careful {
            taken.extend_from_slice(&[tx - half_w, ty - half_h, tx + half_w, ty + half_h]);
        }
        write!(labels, "{}", label_svg(text, tx, ty, " class=\"pele-label\"", None)).unwrap();
    }

    // Mermaid puts each new point under the ones before it.
    let mut content =
        label_svg(&title, width / 2.0, title_pad / 2.0 + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None)
            .to_string();
    svg_write!(
        content,
        "<g class=\"pele-quadrants\" fill=\"",
        region,
        "\" stroke=\"",
        ctx.paint(Role::GroupStroke, "var(--_b)"),
        "\">",
        quadrants,
        "</g>"
    )
    .unwrap();
    if !axes.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-axis-labels\" font-size=\"",
            js::to_string(axis_size),
            "\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            axes,
            "</g>"
        )
        .unwrap();
    }
    if !dots.is_empty() {
        circles.reverse();
        // Pele lets the point labels take the root's text color; in resolved mode they take the
        // color that reads on the quadrants they lie over.
        let label_fill = if ctx.resolved() { svg_format!(" fill=\"", on_region, "\"") } else { String::new() };
        svg_write!(content, "<g class=\"pele-points\" fill=\"", fill, "\">", circles.concat(), "</g>").unwrap();
        svg_write!(
            content,
            "<g class=\"pele-point-labels\" font-size=\"",
            js::to_string(point_size),
            "\"",
            label_fill,
            ">",
            labels,
            "</g>"
        )
        .unwrap();
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered { svg: svg_document("quadrantChart", width, height, size, ctx, acc, &content), width, height, links: Vec::new() })
}
