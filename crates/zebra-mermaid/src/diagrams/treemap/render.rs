//! Treemap rendering. Ported from Pele `src/diagrams/treemap/render.ts`.

use super::format::formatter;
use super::layout::{layout_treemap, Insets};
use super::TreemapModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_text::fit_text;
use crate::diagrams::common::fit_width::{cap_width, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg};
use crate::svg::concat::svg_format;
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{class_names, resolve_style, rounded};
use crate::text::entities::decode_entities;
use crate::text::{layout_render_label as layout_label, style};
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

fn title_part(raw: &str) -> String {
    js::trim(&esc(js::trim(&decode_entities(raw)))).to_owned()
}
fn corners(attrs: &str, b: &super::layout::Box, ctx: &RenderCtx) -> String {
    let cap = js::min(b.w, b.h) / 4.0;
    if cap < 4.0 && ctx.resolved() {
        format!(" rx=\"{}\"{attrs}", num(cap))
    } else if cap < 4.0 {
        rounded(attrs).replace("rx:var(--_r)", &format!("rx:min(var(--_r),{}px)", num(cap)))
    } else {
        rounded(attrs)
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
    let width = js::round(cap_width(options, option(config, "nodeWidth", 64.0, 1.0, 400.0) * 10.0, 200.0, 2.0 * pad));
    let height = js::round(option(config, "nodeHeight", 40.0, 1.0, 400.0) * 10.0);
    let show = config.lookup("treemap", "showValues").and_then(ConfigValue::as_bool) != Some(false);
    let format = formatter(config.lookup("treemap", "valueFormat").and_then(ConfigValue::as_str).unwrap_or(","));
    let mut stack = model.roots.clone();
    let mut count = 0;
    while let Some(i) = stack.pop() {
        let n = &model.nodes[i];
        if let Some(c) = &n.children {
            stack.extend(c);
        } else if n.value.is_some_and(|v| v.is_finite() && v > 0.0) {
            count += 1;
        }
    }
    let crowd = js::min(1.0, js::max(0.0, ((width * height) / js::max(1.0, count as f64) / 800.0).log10()));
    let insets = Insets {
        gap: option(config, "padding", js::round(2.0 + 2.0 * crowd), 0.0, 200.0),
        header: HEADER,
        short: 18.0,
        side: js::round(3.0 + 3.0 * crowd),
    };
    let (boxes, placed) = layout_treemap(&model.nodes, &model.roots, width, height, &insets);
    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, title_room(options), style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 8.0 } else { 0.0 };
    let left = pad + js::max(0.0, (title.width - width) / 2.0);
    let top = pad + title_h;
    let (mut sections, mut leaves) = (String::new(), String::new());
    for index in placed {
        let b = &boxes[index];
        let node = &model.nodes[b.node];
        let st = resolve_style(node.css_compiled_styles.as_deref().unwrap_or(&[]));
        let color = ctx.series(b.series as i64);
        let x = left + b.x;
        let y = top + b.y;
        let decoded = decode_entities(&node.name);
        let name = js::trim(&decoded);
        let value = if show { format.format(b.value) } else { String::new() };
        let group = format!("data-id=\"{}\">", esc_text(&node.name));
        let own = title_part(&node.name);
        let mut path = if own.is_empty() { Vec::new() } else { vec![own.clone()] };
        let mut parent = b.parent;
        while let Some(i) = parent {
            let p = &boxes[i];
            if p.headed {
                break;
            }
            let section = title_part(&model.nodes[p.node].name);
            if !section.is_empty() {
                path.insert(0, section);
            }
            parent = p.parent;
        }
        let full = path.join(" › ");
        let tooltip = |drawn: Option<&str>| {
            if full.is_empty() || (drawn == Some(name) && full == own) {
                String::new()
            } else {
                format!("<title>{full}</title>")
            }
        };
        let rect = format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{color}\"", num(x), num(y), num(b.w), num(b.h));
        let fixed = st.font_size;
        let text_style = (if st.bold { style::BOLD } else { 0 }) | (if st.italic { style::ITALIC } else { 0 });
        let ink = if crate::svg::theme::sets_fill(&st.shape) {
            crate::svg::ink::ink_fill(&st, None)
        } else {
            ctx.series_ink_fill(b.series as i64)
        };
        let classes = class_names(node.class_selector.as_deref().unwrap_or(""));
        if node.children.is_some() {
            let mut text = String::new();
            let mut drawn = None;
            if b.headed {
                let short = b.header < HEADER;
                let label_size = fixed.unwrap_or(if short { tiny } else { small });
                let value_size = fixed.unwrap_or(tiny);
                let room = b.w - 2.0 * SIDE - 4.0;
                let vw = if value.is_empty() { 0.0 } else { m.width(&value, value_size, text_style) };
                let with_value = !short && vw > 0.0 && room - vw - 12.0 >= 3.0 * label_size;
                let label = if label_size * LINE < b.header {
                    fit_text(name, if with_value { room - vw - 12.0 } else { room }, label_size, label_size, &m, text_style | style::BOLD)
                } else {
                    None
                };
                let baseline = y + b.header / 2.0 + 1.0;
                if let Some(label) = label {
                    drawn = Some(label.text.clone());
                    text.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-weight=\"var(--_hw)\"{}{ink}{}>{}</text>",
                        num(x + SIDE + 2.0),
                        num(baseline + label_size * 0.35),
                        if short { format!(" font-size=\"{}\"", js::number_text(label_size)) } else { String::new() },
                        st.text,
                        esc(&label.text)
                    ));
                }
                if with_value && value_size * LINE <= HEADER {
                    text.push_str(&format!(
                        "<text class=\"pele-value\" x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"end\"{}{}>{}</text>",
                        num(x + b.w - SIDE - 2.0),
                        num(baseline + value_size * 0.35),
                        js::number_text(tiny),
                        crate::svg::ink::ink_fill(&st, Some(ctx.paint(Role::SecondaryText, "var(--_m)"))),
                        st.text,
                        esc(&value)
                    ));
                }
            }
            let tint = if crate::svg::theme::sets_fill(&st.shape) { "" } else { " fill-opacity=\"0.12\"" };
            let shape = if b.headed {
                format!("{rect}{}/>", corners(&format!("{tint} stroke=\"{color}\"{}", st.shape), b, ctx))
            } else {
                format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\"/>", num(x), num(y), num(b.w), num(b.h))
            };
            sections
                .push_str(&format!("<g class=\"pele-node pele-section{classes}\" {group}{}{shape}{text}</g>", tooltip(drawn.as_deref())));
            continue;
        }
        let own_fill = if ink.is_empty() && ctx.resolved() { format!(" fill=\"{}\"", ctx.theming.text_on(&color, "")) } else { ink };
        let room = b.w - 8.0;
        let smallest = fixed.unwrap_or(least);
        let tallest = if let Some(fixed) = fixed {
            if fixed * LINE <= b.h - 4.0 {
                fixed
            } else {
                0.0
            }
        } else {
            js::min(size, ((b.h - 4.0) / LINE).floor())
        };
        let label = if tallest >= smallest { fit_text(name, room, tallest, smallest, &m, text_style) } else { None };
        let mut text = String::new();
        if let Some(label) = &label {
            let cx = num(x + b.w / 2.0);
            let cy = y + b.h / 2.0;
            let vs = fixed.unwrap_or_else(|| js::max(least, js::round(label.size * 0.8)));
            let lines = (label.size + vs) * LINE;
            if !value.is_empty() && lines <= b.h - 4.0 && m.width(&value, vs, text_style) <= room {
                let first = cy - lines / 2.0 + (label.size * LINE) / 2.0;
                let second = cy + lines / 2.0 - (vs * LINE) / 2.0;
                text=format!("<text x=\"{cx}\" y=\"{}\" font-size=\"{}\"{own_fill}{}>{}</text><text class=\"pele-value\" x=\"{cx}\" y=\"{}\" font-size=\"{}\"{own_fill}{}>{}</text>",num(first+label.size*0.35),num(label.size),st.text,esc(&label.text),num(second+vs*0.35),num(vs),st.text,esc(&value));
            } else {
                text = format!(
                    "<text x=\"{cx}\" y=\"{}\" font-size=\"{}\"{own_fill}{}>{}</text>",
                    num(cy + label.size * 0.35),
                    num(label.size),
                    st.text,
                    esc(&label.text)
                );
            }
        }
        leaves.push_str(&format!(
            "<g class=\"pele-node pele-leaf{classes}\" {group}{}{rect}{}/>{text}</g>",
            tooltip(label.as_ref().map(|l| l.text.as_str())),
            corners(&st.shape, b, ctx)
        ));
    }
    let total_width = (js::max(width, title.width) + 2.0 * pad).ceil();
    let total_height = (height + title_h + 2.0 * pad).ceil();
    let mut content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        ctx.icons()
    ));
    if !sections.is_empty() {
        content.push_str(&format!("<g class=\"pele-sections\" font-size=\"{}\">{sections}</g>", js::number_text(small)));
    }
    if !leaves.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-leaves\" text-anchor=\"middle\" fill=\"{}\">{leaves}</g>",
            ctx.paint(Role::Background, "var(--_bg)")
        ));
    }
    Ok(Rendered {
        svg: svg_document(
            "treemap",
            total_width,
            total_height,
            size,
            ctx,
            Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() },
            &content,
        ),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}
