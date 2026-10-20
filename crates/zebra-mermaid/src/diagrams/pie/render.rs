//! Pie rendering. Ported from Pele `src/diagrams/pie/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::f64::consts::PI;

use super::PieModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{fit_width, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::text::{layout_label, style, Label};
use crate::theme::Role;
use crate::util::{js, math};

const RADIUS: f64 = 120.0;
const SWATCH: f64 = 12.0;
const ROW: f64 = 26.0;
const GAP: f64 = 32.0;
const OUTSIDE: f64 = 28.0;

fn option<'a>(config: &'a Config, key: &str) -> Option<&'a ConfigValue> {
    config.lookup("pie", key)
}

fn point(angle: f64, radius: f64) -> String {
    format!("{},{}", num(math::sin(angle) * radius), num(-math::cos(angle) * radius))
}

fn slice(start: f64, end: f64, outer: f64, inner: f64) -> String {
    if end - start >= PI * 2.0 - 1e-9 {
        let ring = |r: f64, sweep: u8| {
            svg_format!(
                "M0,",
                num(-r),
                "A",
                num(r),
                ",",
                num(r),
                " 0 1 ",
                sweep,
                " 0,",
                num(r),
                "A",
                num(r),
                ",",
                num(r),
                " 0 1 ",
                sweep,
                " 0,",
                num(-r),
                "Z"
            )
        };
        return ring(outer, 1) + &if inner > 0.0 { ring(inner, 0) } else { String::new() };
    }
    let large = if end - start > PI { 1 } else { 0 };
    if inner <= 0.0 {
        return svg_format!("M0,0L", point(start, outer), "A", num(outer), ",", num(outer), " 0 ", large, " 1 ", point(end, outer), "Z");
    }
    svg_format!(
        "M",
        point(start, outer),
        "A",
        num(outer),
        ",",
        num(outer),
        " 0 ",
        large,
        " 1 ",
        point(end, outer),
        "L",
        point(end, inner),
        "A",
        num(inner),
        ",",
        num(inner),
        " 0 ",
        large,
        " 0 ",
        point(start, inner),
        "Z"
    )
}

pub(crate) fn render(model: &PieModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    fit_width(ctx.options, RADIUS * 2.0, 140.0, |diameter, tight| Ok(draw(model, config, ctx, diameter / 2.0, tight)))
}

struct Placed {
    x: f64,
    y: f64,
    width: f64,
}

/// `tight` is set when the chart does not fit the width it has with its legend beside it.
fn draw(model: &PieModel, config: &Config, ctx: &RenderCtx, radius: f64, tight: bool) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);

    let text_position = match option(config, "textPosition").and_then(ConfigValue::as_f64) {
        Some(p) if (0.0..=1.0).contains(&p) => p,
        _ => 0.75,
    };
    let inner = match option(config, "donutHole").and_then(ConfigValue::as_f64) {
        Some(h) if h > 0.0 && h <= 0.9 => h * radius,
        _ => 0.0,
    };
    let placed = match option(config, "legendPosition") {
        None | Some(ConfigValue::Null) => "right".to_string(),
        Some(v) => v.to_js_string(),
    };
    let legend_position = if tight && (placed == "left" || placed == "right") { "bottom".to_string() } else { placed };

    let entries: Vec<(&String, f64)> = model.sections.iter().map(|(k, v)| (k, *v)).collect();
    let mut sum = 0.0;
    for &(_, value) in &entries {
        sum += value;
    }

    let legend: Vec<Label> = entries
        .iter()
        .map(|&(label, value)| {
            let text = if model.show_data { format!("{} [{}]", label, js::to_string(value)) } else { label.clone() };
            layout_label(Some(&text), false, &m, small, 4000.0, 0)
        })
        .collect();
    let mut legend_width: f64 = 0.0;
    for label in &legend {
        legend_width = js::max(legend_width, label.width);
    }
    legend_width += if !legend.is_empty() { SWATCH + 8.0 } else { 0.0 };
    let legend_height = legend.len() as f64 * ROW;

    let title = layout_label(model.title.as_deref(), false, &m, size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };

    // A slice too thin for its percentage gets the label outside the rim, which needs a margin.
    let shares: Vec<f64> = entries
        .iter()
        .map(|&(_, value)| {
            let share = if sum > 0.0 { value / sum } else { 0.0 };
            if share * 100.0 >= 1.0 && js::to_fixed(share * 100.0, 0) != "0" {
                share
            } else {
                0.0
            }
        })
        .collect();
    let narrow: Vec<bool> = shares
        .iter()
        .map(|&share| {
            if share == 0.0 || share >= 0.5 {
                return false;
            }
            let chord = 2.0 * radius * text_position * math::sin(share * PI);
            chord < m.width(&format!("{}%", js::to_fixed(share * 100.0, 0)), small, 0) + 6.0
        })
        .collect();
    let margin = if narrow.contains(&true) { OUTSIDE } else { 0.0 };

    let side = match legend_position.as_str() {
        "left" | "right" | "center" => legend_position.as_str(),
        _ => "",
    };
    let stacked = legend_position == "top" || legend_position == "bottom";
    let diameter = (radius + margin) * 2.0;
    let mut width = diameter;
    let mut height = diameter;
    if side == "left" || side == "right" {
        width += if legend_width > 0.0 { GAP + legend_width } else { 0.0 };
        height = js::max(height, legend_height);
    } else if stacked {
        width = js::max(width, legend_width);
        height += if legend_height > 0.0 { 20.0 + legend_height } else { 0.0 };
    }
    width = js::max(width, title.width);

    let cx = pad
        + if side == "left" {
            legend_width + GAP + diameter / 2.0
        } else if side == "right" {
            diameter / 2.0
        } else {
            width / 2.0
        };
    let body_top = pad + title_height;
    let cy = body_top
        + if legend_position == "top" {
            legend_height + 20.0 + diameter / 2.0
        } else if stacked {
            diameter / 2.0
        } else {
            height / 2.0
        };

    let resolved = ctx.resolved();
    let mut slices = String::new();
    let mut labels = String::new();
    let mut outside = String::new();
    let mut angle = 0.0;
    let rim = radius + OUTSIDE / 2.0;
    let mut placed_outside: Vec<Placed> = Vec::new();
    for (index, &(label, _)) in entries.iter().enumerate() {
        let share = shares[index];
        if share == 0.0 {
            continue;
        }
        let end = angle + share * PI * 2.0;
        let fill = ctx.series(index as i64);
        svg_write!(
            slices,
            "<path class=\"pele-slice\" data-id=\"",
            esc_text(label),
            "\" d=\"",
            slice(angle, end, radius, inner),
            "\" fill=\"",
            fill,
            "\"/>"
        )
        .unwrap();
        let mid = (angle + end) / 2.0;
        let text = format!("{}%", js::to_fixed(share * 100.0, 0));
        if narrow[index] {
            let w = m.width(&text, small, 0);
            let clear = |x: f64, y: f64, placed: &[Placed]| {
                placed.iter().all(|o| (x - o.x).abs() >= (w + o.width) / 2.0 + 2.0 || (y - o.y).abs() >= small + 2.0)
            };
            let mut at = mid;
            let mut step = 0;
            while step < 360 && !clear(math::sin(at) * rim, -math::cos(at) * rim, &placed_outside) {
                at += 2.0 / rim;
                step += 1;
            }
            let x = math::sin(at) * rim;
            let y = -math::cos(at) * rim;
            placed_outside.push(Placed { x, y, width: w });
            svg_write!(outside, "<text x=\"", num(x), "\" y=\"", num(y + small * 0.35), "\">", text, "</text>").unwrap();
        } else {
            let r = radius * text_position;
            // In resolved mode each label takes a color that reads on its own slice.
            let own = if resolved { svg_format!(" fill=\"", ctx.theming.resolved.text_on(&fill), "\"") } else { String::new() };
            svg_write!(
                labels,
                "<text x=\"",
                num(math::sin(mid) * r),
                "\" y=\"",
                num(-math::cos(mid) * r + small * 0.35),
                "\"",
                own,
                ">",
                text,
                "</text>"
            )
            .unwrap();
        }
        angle = end;
    }
    if slices.is_empty() {
        slices = svg_format!(
            "<circle class=\"pele-slice\" r=\"",
            js::to_string(radius),
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\"/>"
        );
    }

    let mut legend_out = String::new();
    let (lx, ly) = if side == "right" {
        (cx + diameter / 2.0 + GAP, body_top + (height - legend_height) / 2.0)
    } else if side == "left" {
        (pad, body_top + (height - legend_height) / 2.0)
    } else if side == "center" {
        (cx - legend_width / 2.0, cy - legend_height / 2.0)
    } else {
        (pad + (width - legend_width) / 2.0, if legend_position == "top" { body_top } else { body_top + diameter + 20.0 })
    };
    for (index, label) in legend.iter().enumerate() {
        let y = ly + index as f64 * ROW + ROW / 2.0;
        svg_write!(
            legend_out,
            "<g class=\"pele-legend-item\" data-id=\"",
            esc_text(entries[index].0),
            "\"><rect x=\"",
            num(lx),
            "\" y=\"",
            num(y - SWATCH / 2.0),
            "\" width=\"12\" height=\"12\" rx=\"2\" fill=\"",
            ctx.series(index as i64),
            "\"/>",
            label_svg(label, lx + SWATCH + 8.0 + label.width / 2.0, y, "", None),
            "</g>"
        )
        .unwrap();
    }

    let total_width = (width + 2.0 * pad).ceil();
    let total_height = (height + title_height + 2.0 * pad).ceil();
    let small_s = js::to_string(small);
    let mut content =
        label_svg(&title, total_width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None)
            .to_string();
    svg_write!(
        content,
        "<g class=\"pele-slices\" transform=\"translate(",
        num(cx),
        ",",
        num(cy),
        ")\" stroke=\"",
        ctx.paint(Role::Background, "var(--_bg)"),
        "\" stroke-width=\"1\" stroke-linejoin=\"round\">",
        slices,
        "</g>"
    )
    .unwrap();
    if !labels.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-slice-labels\" transform=\"translate(",
            num(cx),
            ",",
            num(cy),
            ")\" font-size=\"",
            small_s,
            "\" text-anchor=\"middle\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\">",
            labels,
            "</g>"
        )
        .unwrap();
    }
    if !outside.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-slice-labels\" transform=\"translate(",
            num(cx),
            ",",
            num(cy),
            ")\" font-size=\"",
            small_s,
            "\" text-anchor=\"middle\">",
            outside,
            "</g>"
        )
        .unwrap();
    }
    if !legend_out.is_empty() {
        // Zebra writes legend text in the text color when the caller gave series colors, and in
        // the muted text color otherwise. Pele lets it inherit the text color.
        let legend_fill = if resolved {
            let role = if ctx.theming.resolved.series.is_some() { Role::Text } else { Role::MutedText };
            svg_format!(" fill=\"", ctx.theming.resolved.color(role), "\"")
        } else {
            String::new()
        };
        svg_write!(content, "<g class=\"pele-legend\" font-size=\"", small_s, "\"", legend_fill, ">", legend_out, "</g>").unwrap();
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("pie", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
