//! Packet rendering. Ported from Pele `src/diagrams/packet/render.ts`.

use std::collections::HashMap;

use super::{packet_option, PacketBlock, PacketModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_text::{fit_text, Fitted};
use crate::diagrams::common::fit_width::{title_room, turn_to_fit};
use crate::diagrams::common::literal_label::literal_label;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::entities::decode_entities;
use crate::text::{layout_label, style, Label};
use crate::theme::Role;
use crate::util::{js, math};
use std::fmt::Write as _;

pub(crate) fn render(model: &PacketModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, down)))
}

fn draw(model: &PacketModel, config: &Config, ctx: &RenderCtx, down: bool) -> Rendered {
    let options = ctx.options;
    let size = ctx.font_size();
    let label_size = js::round(size * 0.875);
    let bit_size = js::round(size * 0.625);
    let m = ctx.measure();
    let pad = ctx.padding();

    let row_height = packet_option(config, "rowHeight", 32.0, 1.0, 400.0);
    let mut bit_width = packet_option(config, "bitWidth", 32.0, 1.0, 400.0);
    let padding_x = packet_option(config, "paddingX", 5.0, 0.0, 400.0);
    let padding_y = packet_option(config, "paddingY", 5.0, 0.0, 400.0);
    let show_bits = config.lookup("packet", "showBits").and_then(ConfigValue::as_bool) != Some(false);
    let descending = config.lookup("packet", "bitOrder").and_then(ConfigValue::as_str) == Some("descending");
    let mut bits_per_row = model.bits_per_row;
    let mut split: Vec<Vec<PacketBlock>> = Vec::new();
    let room_across = options.max_width.unwrap_or(320.0) - 2.0 * pad;
    if down {
        let count = js::max(1.0, (room_across / js::min(bit_width, 18.0)).floor());
        // A byte is the shortest row. Below that every label is one character to a line, and a
        // long packet is a row to a bit.
        bits_per_row = js::min(bits_per_row, js::max(8.0, math::pow(2.0, math::log2(count).floor())));
        let last = model.rows.last().and_then(|row| row.last()).map_or(0.0, |block| block.end);
        if ((last + 1.0) / bits_per_row).ceil() > 10000.0 {
            bits_per_row = model.bits_per_row;
        }
        bit_width = js::min(bit_width, js::max(18.0, room_across / bits_per_row));
        // The model has split each field at its own rows. Its pieces are joined again first, so
        // that a field is only split where one of the new rows ends. A piece counts one bit short.
        let piece = |block: &PacketBlock| block.bits == block.end - block.start;
        let mut whole: Vec<PacketBlock> = Vec::new();
        for row in &model.rows {
            for block in row {
                match whole.last_mut() {
                    Some(prev)
                        if piece(prev)
                            && piece(block)
                            && prev.label == block.label
                            && block.start == prev.end + 1.0
                            && block.start % model.bits_per_row == 0.0 =>
                    {
                        prev.end = block.end;
                        prev.bits = prev.end - prev.start;
                    }
                    _ => whole.push(block.clone()),
                }
            }
        }
        for block in &whole {
            let mut start = block.start;
            while start <= block.end {
                let end = js::min(block.end, ((start / bits_per_row).floor() + 1.0) * bits_per_row - 1.0);
                // The fields cover every bit from zero, so the rows fill in order.
                let index = (start / bits_per_row).floor() as usize;
                while split.len() <= index {
                    split.push(Vec::new());
                }
                split[index].push(PacketBlock { start, end, bits: block.bits, label: block.label.clone() });
                start = end + 1.0;
            }
        }
    }
    let rows: &[Vec<PacketBlock>] = if down { &split } else { &model.rows };
    let labels: Vec<Vec<Label>> = if down {
        rows.iter()
            .map(|row| {
                row.iter()
                    .map(|block| {
                        let room = js::max(1.0, (block.end - block.start + 1.0) * bit_width - js::min(padding_x, bit_width - 1.0) - 6.0);
                        let text = decode_entities(&block.label);
                        literal_label(js::trim(&text), &m, label_size, room, 0)
                    })
                    .collect()
            })
            .collect()
    } else {
        Vec::new()
    };
    // Bit numbers sit above each row and need room of their own.
    let numbers = if show_bits { bit_size + 4.0 } else { 0.0 };
    let row_heights: Vec<f64> = rows
        .iter()
        .enumerate()
        .map(|(r, row)| {
            let mut height = row_height;
            if down {
                for i in 0..row.len() {
                    height = js::max(height, labels[r][i].height + 10.0);
                }
            }
            height
        })
        .collect();
    let mut tops: Vec<f64> = Vec::with_capacity(row_heights.len());
    let mut rows_height = 0.0;
    for &height in &row_heights {
        tops.push(rows_height);
        rows_height += numbers + height + padding_y;
    }
    if !rows.is_empty() {
        rows_height -= padding_y;
    }

    let gap = js::min(padding_x, bit_width - 1.0);
    let width = if !rows.is_empty() { bits_per_row * bit_width - gap } else { 0.0 };
    let title = layout_label(model.title.as_deref(), false, &m, size, if down { title_room(options) } else { 4000.0 }, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + if rows_height > 0.0 { 12.0 } else { 0.0 } } else { 0.0 };

    // The pieces of a field split across rows share its label, so each width is fitted once.
    let mut fits: HashMap<(&str, u64), Option<Fitted>> = HashMap::new();
    let least = js::round(size * 0.5);

    let left = pad + js::max(0.0, (title.width - width) / 2.0);
    let fill = ctx.paint(Role::NodeFill, "var(--_s)");
    let stroke = ctx.paint(Role::NodeStroke, "var(--_b)");
    let mut fields = String::new();
    let mut bits = String::new();
    for (index, row) in rows.iter().enumerate() {
        let y = pad + tops[index] + numbers;
        let row_height = row_heights[index];
        for (i, block) in row.iter().enumerate() {
            let count = block.end - block.start + 1.0;
            let first = block.start % bits_per_row;
            let x = left + (if descending { bits_per_row - first - count } else { first }) * bit_width;
            let w = count * bit_width - gap;
            write!(fields, 
                "<g class=\"pele-node\" data-id=\"{}-{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\" stroke=\"{}\"/>",
                js::to_string(block.start),
                js::to_string(block.end),
                num(x),
                num(y),
                num(w),
                num(row_height),
                RADIUS,
                fill,
                stroke
            ).unwrap();
            if down {
                write!(fields, "{}", label_svg(&labels[index][i], x + w / 2.0, y + row_height / 2.0, "", ctx.icons())).unwrap();
            } else {
                let max = w - 6.0;
                let text = fits
                    .entry((block.label.as_str(), max.to_bits()))
                    .or_insert_with(|| {
                        let label = decode_entities(&block.label);
                        fit_text(js::trim(&label), max, label_size, least, &m, 0)
                    })
                    .as_ref();
                if let Some(text) = text.filter(|t| t.size <= row_height) {
                    let font = if text.size == label_size { String::new() } else { format!(" font-size=\"{}\"", num(text.size)) };
                    write!(fields, 
                        "<text x=\"{}\" y=\"{}\"{}>{}</text>",
                        num(x + w / 2.0),
                        num(y + row_height / 2.0 + text.size * 0.35),
                        font,
                        esc(&text.text)
                    ).unwrap();
                }
            }
            fields.push_str("</g>");
            if !show_bits {
                continue;
            }
            let leading = js::to_string(if descending { block.end } else { block.start });
            let trailing = js::to_string(if descending { block.start } else { block.end });
            if count == 1.0 {
                write!(bits, 
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
                    num(x + w / 2.0),
                    num(y - 4.0),
                    leading
                ).unwrap();
            } else {
                write!(bits, 
                    "<text x=\"{}\" y=\"{}\">{}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
                    num(x),
                    num(y - 4.0),
                    leading,
                    num(x + w),
                    num(y - 4.0),
                    trailing
                ).unwrap();
            }
        }
    }

    let total_width = (js::max(width, title.width) + 2.0 * pad).ceil();
    let total_height = (rows_height + title_height + 2.0 * pad).ceil();
    let mut content = String::new();
    if !fields.is_empty() {
        write!(content, 
            "<g class=\"pele-packet-fields\" font-size=\"{}\" text-anchor=\"middle\">{}</g>",
            js::to_string(label_size),
            fields
        ).unwrap();
    }
    if !bits.is_empty() {
        // Bit numbers are captions of the fields.
        write!(content, 
            "<g class=\"pele-packet-bits\" font-size=\"{}\" fill=\"{}\">{}</g>",
            js::to_string(bit_size),
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            bits
        ).unwrap();
    }
    // Mermaid puts a packet's title under it.
    write!(content, "{}", label_svg(
        &title,
        total_width / 2.0,
        total_height - pad - title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        ctx.icons(),
    )).unwrap();

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("packet", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
