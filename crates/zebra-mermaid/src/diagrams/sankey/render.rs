//! Sankey rendering. Ported from Pele `src/diagrams/sankey/render.ts`.

use std::collections::HashMap;

use super::layout::{layout_sankey, Alignment, LayoutOptions};
use super::SankeyModel;
use crate::config::{Config, ConfigMap, ConfigValue};
use crate::diagrams::common::fit_width::{fit_width, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::resolve_style;
use crate::text::{layout_label, style};
use crate::theme::Role;
use crate::util::js;
use std::fmt::Write as _;

const LABEL_GAP: f64 = 6.0;
const VALUE_GAP: f64 = 15.0;

fn number_option(conf: &ConfigMap, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match conf.get(key) {
        Some(ConfigValue::Number(value)) if *value >= min => js::min(*value, max),
        _ => fallback,
    }
}

fn text_option(conf: &ConfigMap, key: &str) -> String {
    match conf.get(key) {
        Some(ConfigValue::String(s)) => s.clone(),
        Some(ConfigValue::Number(n)) if n.is_finite() => js::to_string(*n),
        _ => String::new(),
    }
}

/// Two decimals, as Mermaid shows values; numbers too large for that are left as they are.
fn rounded(value: f64) -> f64 {
    if value.abs() < 1e15 {
        js::round(value * 100.0) / 100.0
    } else {
        value
    }
}

/// A colour from the configuration, as an inline style that overrides the series colour.
fn fill_style(color: &ConfigValue) -> String {
    match color {
        ConfigValue::String(c) => resolve_style(&[format!("fill:{}", c)]).shape,
        _ => String::new(),
    }
}

pub(crate) fn render(model: &SankeyModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let empty = ConfigMap::new();
    let conf = config.section("sankey").unwrap_or(&empty);
    fit_width(ctx.options, number_option(conf, "width", 600.0, 1.0, 10000.0), 160.0, |chart_width, _| draw(model, conf, ctx, chart_width))
}

fn draw(model: &SankeyModel, conf: &ConfigMap, ctx: &RenderCtx, chart_width: f64) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();

    let chart_height = number_option(conf, "height", 400.0, 1.0, 10000.0);
    let node_width = js::min(number_option(conf, "nodeWidth", 10.0, 1.0, 200.0), chart_width);
    let show_values = conf.get("showValues") != Some(&ConfigValue::Bool(false));
    let prefix = text_option(conf, "prefix");
    let suffix = text_option(conf, "suffix");
    let outlined = conf.get("labelStyle").and_then(ConfigValue::as_str) == Some("outlined");
    let alignment = match conf.get("nodeAlignment").and_then(ConfigValue::as_str) {
        Some("left") => Alignment::Left,
        Some("right") => Alignment::Right,
        Some("center") => Alignment::Center,
        _ => Alignment::Justify,
    };
    let link_color = conf.get("linkColor").and_then(ConfigValue::as_str).unwrap_or("gradient");

    let (nodes, links) = layout_sankey(
        model,
        &LayoutOptions {
            width: chart_width,
            height: chart_height,
            node_width,
            node_padding: number_option(conf, "nodePadding", 12.0, 0.0, 200.0) + if show_values { VALUE_GAP } else { 0.0 },
            label_height: small * 1.15,
            alignment,
        },
    )?;

    let mut custom: HashMap<&str, String> = HashMap::new();
    if let Some(colors) = conf.section("nodeColors") {
        for (id, color) in colors.iter() {
            let style = fill_style(color);
            if !style.is_empty() {
                custom.insert(id, style);
            }
        }
    }
    let node_style = |i: usize| -> &str { custom.get(model.nodes[i].id.as_str()).map_or("", String::as_str) };

    // The colors of this render. Bars and bands take the chart series; a value beside a name is
    // secondary text; the halo of an outlined label is the canvas color.
    let value_fill = ctx.paint(Role::SecondaryText, "var(--_m)");
    let halo = ctx.paint(Role::Background, "var(--_bg)");

    // Labels sit beside the bars, on the side that faces the middle of the chart.
    let mut central = 0;
    let mut most = 0.0;
    for node in &nodes {
        if node.value > most {
            most = node.value;
            central = node.layer;
        }
    }
    let mut min_x: f64 = 0.0;
    let mut max_x = chart_width;
    let mut min_y: f64 = 0.0;
    let mut max_y = chart_height;
    let mut labels_out = String::new();
    let mut nodes_out = String::new();
    let small_s = js::to_string(small);
    for (i, node) in nodes.iter().enumerate() {
        let id = model.nodes[i].id.as_str();
        let height = js::max(node.height, 1.0);
        write!(nodes_out, 
            "<rect class=\"pele-sankey-node\" data-id=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"{}/>",
            esc_text(id),
            num(node.x),
            num(node.y),
            num(node_width),
            num(height),
            num(js::min(2.0, height / 2.0)),
            ctx.series(i as i64),
            node_style(i)
        ).unwrap();

        let value = if show_values { format!("{}{}{}", prefix, js::to_string(rounded(node.value)), suffix) } else { String::new() };
        let after = if outlined { node.layer >= central } else { node.x < chart_width / 2.0 };
        let text_width = m.width(if !value.is_empty() { format!("{} {}", id, value) } else { id.to_string() }.as_str(), small, 0);
        let x = if after { node.x + node_width + LABEL_GAP } else { node.x - LABEL_GAP };
        let y = node.y + node.height / 2.0;
        min_x = js::min(min_x, if after { x } else { x - text_width });
        max_x = js::max(max_x, if after { x + text_width } else { x });
        min_y = js::min(min_y, y - small * 0.75);
        max_y = js::max(max_y, y + small * 0.75);
        let position = format!(" x=\"{}\" y=\"{}\" text-anchor=\"{}\"", num(x), num(y + small * 0.35), if after { "start" } else { "end" });
        let mut text = esc(id).into_owned();
        if !value.is_empty() {
            write!(text, " <tspan class=\"pele-sankey-value\" fill=\"{}\">{}</tspan>", value_fill, esc(&value)).unwrap();
        }
        write!(labels_out, "<g class=\"pele-label\" data-id=\"{}\">", esc_text(id)).unwrap();
        if outlined {
            write!(labels_out, 
                "<text{} fill=\"none\" stroke=\"{}\" stroke-width=\"3\" stroke-linejoin=\"round\">{}{}</text>",
                position,
                halo,
                esc(id),
                if !value.is_empty() { format!(" {}", esc(&value)) } else { String::new() }
            ).unwrap();
        }
        write!(labels_out, "<text{}>{}</text></g>", position, text).unwrap();
    }

    let fixed = if link_color == "gradient" || link_color == "source" || link_color == "target" {
        String::new()
    } else {
        resolve_style(&[format!("fill:{}", link_color)]).shape
    };
    let mut links_out = String::new();
    for link in &links {
        // Without definitions there are no gradients, so a gradient link takes its source's colour.
        let end = if link_color == "target" { link.target } else { link.source };
        let half = js::max(link.thickness, 1.0) / 2.0;
        let x0 = nodes[link.source].x + node_width;
        let x1 = nodes[link.target].x;
        let mid = num((x0 + x1) / 2.0);
        let from = esc_text(&model.nodes[link.source].id);
        let to = esc_text(&model.nodes[link.target].id);
        write!(links_out, 
            "<path class=\"pele-sankey-link\" data-id=\"{from}-&gt;{to}\" d=\"M{},{}C{mid},{} {mid},{} {},{}L{},{}C{mid},{} {mid},{} {},{}Z\" fill=\"{}\"{}><title>{from} → {to}: {}{}{}</title></path>",
            num(x0),
            num(link.source_y - half),
            num(link.source_y - half),
            num(link.target_y - half),
            num(x1),
            num(link.target_y - half),
            num(x1),
            num(link.target_y + half),
            num(link.target_y + half),
            num(link.source_y + half),
            num(x0),
            num(link.source_y + half),
            ctx.series(end as i64),
            if !fixed.is_empty() { fixed.as_str() } else { node_style(end) },
            esc_text(&prefix),
            js::to_string(rounded(link.value)),
            esc_text(&suffix)
        ).unwrap();
    }

    let title = layout_label(model.title.as_deref(), false, &m, size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(max_x - min_x, title.width);
    let ox = pad - min_x + (inner - (max_x - min_x)) / 2.0;
    let oy = pad + title_height - min_y;
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();

    let mut content = label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
    write!(content, "<g transform=\"translate({},{})\">", num(ox), num(oy)).unwrap();
    write!(content, "<g class=\"pele-sankey-links\" fill-opacity=\"0.4\">{}</g>", links_out).unwrap();
    write!(content, "<g class=\"pele-sankey-nodes\">{}</g>", nodes_out).unwrap();
    write!(content, "<g class=\"pele-labels\" font-size=\"{}\">{}</g>", small_s, labels_out).unwrap();
    content.push_str("</g>");

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered { svg: svg_document("sankey", width, height, size, ctx, acc, &content), width, height, links: Vec::new() })
}
