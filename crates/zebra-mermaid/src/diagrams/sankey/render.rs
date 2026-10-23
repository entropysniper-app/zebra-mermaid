//! Sankey rendering. Ported from Pele `src/diagrams/sankey/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use crate::text::entities::decode_entities;
use indexmap::IndexMap;
use std::collections::HashMap;

use super::layout::{layout_sankey, Alignment, LayoutOptions};
use super::SankeyModel;
use crate::config::{Config, ConfigMap, ConfigValue};
use crate::diagrams::common::fit_width::{fit_width, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::resolve_style;
use crate::text::{layout_render_label as layout_label, style};
use crate::theme::Role;
use crate::util::js;

const LABEL_GAP: f64 = 6.0;
const VALUE_GAP: f64 = 15.0;
const LABEL_ROOM: f64 = 1.8;
const CAREFUL: usize = 300;

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
    let natural = number_option(conf, "width", 600.0, 1.0, 10000.0);
    fit_width(ctx.options, natural, 160.0, |chart_width, _| draw(model, conf, ctx, chart_width, natural))
}

fn draw(model: &SankeyModel, conf: &ConfigMap, ctx: &RenderCtx, chart_width: f64, natural: f64) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let least = js::round(size * 0.625);
    let mut small = js::max(least, js::round(size * 0.875 * js::min(1.0, chart_width / natural)));
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

    let lay = |small: f64| {
        layout_sankey(
            model,
            &LayoutOptions {
                width: chart_width,
                height: chart_height,
                node_width,
                node_padding: number_option(conf, "nodePadding", 12.0, 0.0, 200.0) + if show_values { VALUE_GAP } else { 0.0 },
                label_height: small * 1.15,
                alignment,
            },
        )
    };
    let (mut nodes, mut links) = lay(small)?;
    let mut rows = HashMap::<usize, usize>::new();
    let mut fullest = 1;
    for node in &nodes {
        let count = rows.entry(node.layer).or_default();
        *count += 1;
        fullest = fullest.max(*count);
    }
    let crowded = js::max(least, js::round(chart_height / fullest as f64 / LABEL_ROOM));
    if crowded < small {
        small = crowded;
        (nodes, links) = lay(small)?;
    }
    let line_height = small * 1.5;

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
    struct Label {
        value: String,
        width: f64,
        after: bool,
        left: f64,
        y: f64,
    }
    let mut labels = Vec::with_capacity(nodes.len());
    for (i, node) in nodes.iter().enumerate() {
        let name = decode_entities(&model.nodes[i].id);
        let value = if show_values { format!("{}{}{}", prefix, js::number_text(rounded(node.value)), suffix) } else { String::new() };
        let width = m.width(&if value.is_empty() { name } else { format!("{name} {value}") }, small, 0);
        let after = if outlined { node.layer >= central } else { node.x < chart_width / 2.0 };
        let x = if after { node.x + node_width + LABEL_GAP } else { node.x - LABEL_GAP };
        let y = node.y + node.height / 2.0;
        min_x = js::min(min_x, if after { x } else { x - width });
        max_x = js::max(max_x, if after { x + width } else { x });
        min_y = js::min(min_y, y - small * 0.75);
        max_y = js::max(max_y, y + small * 0.75);
        labels.push(Label { value, width, after, left: if after { x } else { x - width }, y });
    }
    let mut bands = HashMap::<i64, Vec<usize>>::new();
    if nodes.len() <= CAREFUL {
        for (i, label) in labels.iter().enumerate() {
            let band = (label.y / line_height).floor() as i64;
            for key in band - 2..=band + 2 {
                bands.entry(key).or_default().push(i);
            }
        }
    }
    let overlaps = |labels: &[Label], i: usize, left: f64, y: f64| {
        bands.get(&((y / line_height).floor() as i64)).is_some_and(|ids| {
            ids.iter().any(|&j| {
                j != i
                    && (labels[j].y - y).abs() < line_height
                    && left < labels[j].left + labels[j].width + LABEL_GAP
                    && left + labels[i].width > labels[j].left - LABEL_GAP
            })
        })
    };
    let mut labels_out = String::new();
    let mut nodes_out = String::new();
    let small_s = js::number_text(small);
    for (i, node) in nodes.iter().enumerate() {
        let id = &model.nodes[i].id;
        let name = decode_entities(id);
        let height = js::max(node.height, 1.0);
        svg_write!(
            nodes_out,
            "<rect class=\"pele-sankey-node\" data-id=\"",
            esc_text(id),
            "\" x=\"",
            num(node.x),
            "\" y=\"",
            num(node.y),
            "\" width=\"",
            num(node_width),
            "\" height=\"",
            num(height),
            "\" rx=\"",
            num(js::min(2.0, height / 2.0)),
            "\" fill=\"",
            ctx.series(i as i64),
            "\"",
            node_style(i),
            "/>"
        )
        .unwrap();
        if nodes.len() <= CAREFUL && overlaps(&labels, i, labels[i].left, labels[i].y) {
            'place: for dy in [0.0, -line_height, line_height] {
                for after in [labels[i].after, !labels[i].after] {
                    let left = if after { node.x + node_width + LABEL_GAP } else { node.x - LABEL_GAP - labels[i].width };
                    let y = labels[i].y + dy;
                    if overlaps(&labels, i, left, y)
                        || left < min_x
                        || left + labels[i].width > max_x
                        || y - small * 0.75 < min_y
                        || y + small * 0.75 > max_y
                    {
                        continue;
                    }
                    if nodes.iter().enumerate().any(|(j, bar)| {
                        j != i
                            && left < bar.x + node_width + LABEL_GAP
                            && left + labels[i].width > bar.x - LABEL_GAP
                            && y - small * 0.75 < bar.y + bar.height
                            && y + small * 0.75 > bar.y
                    }) {
                        continue;
                    }
                    let own = js::max(0.0, js::max(node.y - y, y - node.y - node.height));
                    if nodes
                        .iter()
                        .enumerate()
                        .any(|(j, bar)| j != i && bar.x == node.x && js::max(0.0, js::max(bar.y - y, y - bar.y - bar.height)) <= own)
                    {
                        continue;
                    }
                    labels[i].left = left;
                    labels[i].y = y;
                    labels[i].after = after;
                    break 'place;
                }
            }
        }
        let label = &labels[i];
        let value = &label.value;
        let x = if label.after { node.x + node_width + LABEL_GAP } else { node.x - LABEL_GAP };
        let position = svg_format!(
            " x=\"",
            num(x),
            "\" y=\"",
            num(label.y + small * 0.35),
            "\" text-anchor=\"",
            if label.after { "start" } else { "end" },
            "\""
        );
        let mut text = esc(&name).into_owned();
        if !value.is_empty() {
            svg_write!(text, " <tspan class=\"pele-sankey-value\" fill=\"", value_fill, "\">", esc(value), "</tspan>").unwrap();
        }
        svg_write!(labels_out, "<g class=\"pele-label\" data-id=\"", esc_text(id), "\">").unwrap();
        if outlined {
            svg_write!(
                labels_out,
                "<text",
                position,
                " fill=\"none\" stroke=\"",
                halo,
                "\" stroke-width=\"3\" stroke-linejoin=\"round\">",
                esc(&name),
                if !value.is_empty() { format!(" {}", esc(value)) } else { String::new() },
                "</text>"
            )
            .unwrap();
        }
        svg_write!(labels_out, "<text", position, ">", text, "</text></g>").unwrap();
    }

    let fixed = if link_color == "gradient" || link_color == "source" || link_color == "target" {
        String::new()
    } else {
        resolve_style(&[format!("fill:{}", link_color)]).shape
    };
    let node_color = |i: usize| {
        let style = node_style(i);
        style
            .strip_prefix(" style=\"fill:")
            .and_then(|s| s.strip_suffix(";\""))
            .map_or_else(|| ctx.series(i as i64).into_owned(), str::to_owned)
    };
    let id_prefix = options.id_prefix.as_deref().unwrap_or("");
    let gradient =
        link_color == "gradient" && !id_prefix.is_empty() && id_prefix.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-');
    let mut gradients = IndexMap::<String, String>::new();
    let mut links_out = String::new();
    for link in &links {
        let end = if link_color == "target" { link.target } else { link.source };
        let half = js::max(link.thickness, 1.0) / 2.0;
        let x0 = nodes[link.source].x + node_width;
        let x1 = nodes[link.target].x;
        let mut paint =
            svg_format!("fill=\"", ctx.series(end as i64), "\"", if !fixed.is_empty() { fixed.as_str() } else { node_style(end) });
        if gradient {
            let stops = svg_format!(
                "x1=\"",
                num(x0),
                "\" x2=\"",
                num(x1),
                "\"><stop offset=\"0\" stop-color=\"",
                node_color(link.source),
                "\"/><stop offset=\"1\" stop-color=\"",
                node_color(link.target),
                "\"/>"
            );
            let id = format!("{}-sankey-{}", id_prefix, content_id(&stops));
            gradients.insert(
                id.clone(),
                svg_format!("<linearGradient id=\"", id, "\" gradientUnits=\"userSpaceOnUse\" ", stops, "</linearGradient>"),
            );
            paint = svg_format!("fill=\"url(#", id, ") ", node_color(link.source), "\"");
        }
        let mid = num((x0 + x1) / 2.0);
        let from = esc_text(&model.nodes[link.source].id);
        let to = esc_text(&model.nodes[link.target].id);
        svg_write!(
            links_out,
            "<path class=\"pele-sankey-link\" data-id=\"",
            from,
            "-&gt;",
            to,
            "\" d=\"M",
            num(x0),
            ",",
            num(link.source_y - half),
            "C",
            mid,
            ",",
            num(link.source_y - half),
            " ",
            mid,
            ",",
            num(link.target_y - half),
            " ",
            num(x1),
            ",",
            num(link.target_y - half),
            "L",
            num(x1),
            ",",
            num(link.target_y + half),
            "C",
            mid,
            ",",
            num(link.target_y + half),
            " ",
            mid,
            ",",
            num(link.source_y + half),
            " ",
            num(x0),
            ",",
            num(link.source_y + half),
            "Z\" ",
            paint,
            "><title>",
            from,
            " → ",
            to,
            ": ",
            esc_text(&prefix),
            js::number_text(rounded(link.value)),
            esc_text(&suffix),
            "</title></path>"
        )
        .unwrap();
    }

    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(max_x - min_x, title.width);
    let ox = pad - min_x + (inner - (max_x - min_x)) / 2.0;
    let oy = pad + title_height - min_y;
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();

    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
    svg_write!(content, "<g transform=\"translate(", num(ox), ",", num(oy), ")\">").unwrap();
    if !gradients.is_empty() {
        content.push_str("<defs>");
        for g in gradients.values() {
            content.push_str(g);
        }
        content.push_str("</defs>");
    }
    svg_write!(content, "<g class=\"pele-sankey-links\" fill-opacity=\"0.4\">", links_out, "</g>").unwrap();
    svg_write!(content, "<g class=\"pele-sankey-nodes\">", nodes_out, "</g>").unwrap();
    svg_write!(content, "<g class=\"pele-labels\" font-size=\"", small_s, "\">", labels_out, "</g>").unwrap();
    content.push_str("</g>");

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered { svg: svg_document("sankey", width, height, size, ctx, acc, &content), width, height, links: Vec::new() })
}

fn content_id(text: &str) -> String {
    fn base36(mut x: u32) -> String {
        let mut digits = Vec::new();
        loop {
            digits.push(char::from_digit(x % 36, 36).unwrap());
            x /= 36;
            if x == 0 {
                break;
            }
        }
        digits.into_iter().rev().collect()
    }
    let (mut a, mut b) = (0x811c9dc5u32, 5381u32);
    for c in text.encode_utf16() {
        a = (a ^ c as u32).wrapping_mul(0x01000193);
        b = b.wrapping_mul(33).wrapping_add(c as u32);
    }
    format!("{}{}", base36(a), base36(b))
}
