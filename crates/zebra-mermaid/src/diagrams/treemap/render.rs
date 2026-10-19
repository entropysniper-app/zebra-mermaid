//! Treemap rendering. Ported from Pele `src/diagrams/treemap/render.ts`.

use super::format::formatter;
use super::layout::{layout_treemap, Insets};
use super::TreemapModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_text::fit_text;
use crate::diagrams::common::fit_width::{cap_width, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{class_names, resolve_style, RADIUS};
use crate::text::entities::decode_entities;
use crate::text::{layout_label, style};
use crate::theme::Role;
use crate::util::js;

const HEADER: f64 = 28.0;
const SIDE: f64 = 6.0;
const LINE: f64 = 1.25;

/// A number from the `treemap` section of the configuration, at least `min` and capped at `max`.
fn option(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("treemap", key).and_then(ConfigValue::as_f64) {
        Some(value) if value >= min => js::min(value, max),
        _ => fallback,
    }
}

pub(crate) fn render(model: &TreemapModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.75);
    let least = js::round(size * 0.5625);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or_else(|| option(config, "diagramPadding", 8.0, 0.0, 2000.0));

    // Mermaid sizes the diagram as ten times `nodeWidth` by ten times `nodeHeight`.
    let width = js::round(cap_width(options, option(config, "nodeWidth", 64.0, 1.0, 400.0) * 10.0, 200.0, 2.0 * pad));
    let height = js::round(option(config, "nodeHeight", 40.0, 1.0, 400.0) * 10.0);
    let show_values = config.lookup("treemap", "showValues").and_then(ConfigValue::as_bool) != Some(false);
    let format = formatter(config.lookup("treemap", "valueFormat").and_then(ConfigValue::as_str).unwrap_or(","));
    let insets = Insets { gap: option(config, "padding", 4.0, 0.0, 200.0), header: HEADER, side: SIDE };
    let (boxes, placed) = layout_treemap(&model.nodes, &model.roots, width, height, &insets);

    let title = layout_label(model.title.as_deref(), false, &m, size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 8.0 } else { 0.0 };
    let left = pad + js::max(0.0, (title.width - width) / 2.0);
    let top = pad + title_height;

    let resolved = ctx.resolved();
    let mut sections = String::new();
    let mut leaves = String::new();
    for &index in &placed {
        let b = &boxes[index];
        let node = &model.nodes[b.node];
        let style = resolve_style(node.css_compiled_styles.as_deref().unwrap_or(&[]));
        let color = ctx.series(b.series as i64);
        let x = left + b.x;
        let y = top + b.y;
        let name = decode_entities(&node.name);
        let name = js::trim(&name);
        let value = if show_values { format.format(b.value) } else { String::new() };
        let group = format!("data-id=\"{}\">", esc_text(&node.name));
        let rect = format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"",
            num(x),
            num(y),
            num(b.w),
            num(b.h),
            RADIUS,
            color
        );
        let classes = class_names(node.class_selector.as_deref().unwrap_or(""));

        if node.children.is_some() {
            let mut text = String::new();
            if b.h >= HEADER {
                let room = b.w - 2.0 * SIDE - 4.0;
                let value_width = if !value.is_empty() { m.width(&value, tiny, 0) } else { 0.0 };
                let with_value = value_width > 0.0 && room - value_width - 12.0 >= 3.0 * small;
                let label = fit_text(name, if with_value { room - value_width - 12.0 } else { room }, small, small, &m, style::BOLD);
                let baseline = y + HEADER / 2.0 + 1.0;
                if let Some(label) = label {
                    text.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-weight=\"var(--_hw)\"{}>{}</text>",
                        num(x + SIDE + 2.0),
                        num(baseline + small * 0.35),
                        style.text,
                        esc(&label.text)
                    ));
                }
                if with_value {
                    // The value is a caption beside the section's name.
                    text.push_str(&format!(
                        "<text class=\"pele-value\" x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"end\" fill=\"{}\"{}>{}</text>",
                        num(x + b.w - SIDE - 2.0),
                        num(baseline + tiny * 0.35),
                        js::to_string(tiny),
                        ctx.paint(Role::SecondaryText, "var(--_m)"),
                        style.text,
                        esc(&value)
                    ));
                }
            }
            // A section is a tint of its color, unless a class gives it a fill of its own.
            let tint = if style.shape.contains("\"fill:") || style.shape.contains(";fill:") { "" } else { " fill-opacity=\"0.12\"" };
            sections.push_str(&format!(
                "<g class=\"pele-node pele-section{}\" {}{}{} stroke=\"{}\"{}/>{}</g>",
                classes, group, rect, tint, color, style.shape, text
            ));
            continue;
        }

        // Labels on a leaf sit on its color. Pele gives them the background color through the
        // group; resolved output picks, for each leaf, a color that reads on its fill.
        let own = if resolved { format!(" fill=\"{}\"", ctx.theming.text_on(&color, "")) } else { String::new() };
        let mut text = String::new();
        let room = b.w - 8.0;
        let tallest = js::min(size, ((b.h - 4.0) / LINE).floor());
        let label = if tallest >= least { fit_text(name, room, tallest, least, &m, 0) } else { None };
        if let Some(label) = label {
            let cx = num(x + b.w / 2.0);
            let cy = y + b.h / 2.0;
            let value_size = js::max(least, js::round(label.size * 0.8));
            let lines = (label.size + value_size) * LINE;
            if !value.is_empty() && lines <= b.h - 4.0 && m.width(&value, value_size, 0) <= room {
                let first = cy - lines / 2.0 + (label.size * LINE) / 2.0;
                let second = cy + lines / 2.0 - (value_size * LINE) / 2.0;
                text = format!(
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\"{}{}>{}</text><text class=\"pele-value\" x=\"{}\" y=\"{}\" font-size=\"{}\"{}{}>{}</text>",
                    cx,
                    num(first + label.size * 0.35),
                    num(label.size),
                    own,
                    style.text,
                    esc(&label.text),
                    cx,
                    num(second + value_size * 0.35),
                    num(value_size),
                    own,
                    style.text,
                    esc(&value)
                );
            } else {
                text = format!(
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\"{}{}>{}</text>",
                    cx,
                    num(cy + label.size * 0.35),
                    num(label.size),
                    own,
                    style.text,
                    esc(&label.text)
                );
            }
        }
        leaves.push_str(&format!("<g class=\"pele-node pele-leaf{}\" {}{}{}/>{}</g>", classes, group, rect, style.shape, text));
    }

    let total_width = (js::max(width, title.width) + 2.0 * pad).ceil();
    let total_height = (height + title_height + 2.0 * pad).ceil();
    let mut content =
        label_svg(&title, total_width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", ctx.icons());
    if !sections.is_empty() {
        content.push_str(&format!("<g class=\"pele-sections\" font-size=\"{}\">{}</g>", js::to_string(small), sections));
    }
    if !leaves.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-leaves\" text-anchor=\"middle\" fill=\"{}\">{}</g>",
            ctx.paint(Role::Background, "var(--_bg)"),
            leaves
        ));
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered {
        svg: svg_document("treemap", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}
