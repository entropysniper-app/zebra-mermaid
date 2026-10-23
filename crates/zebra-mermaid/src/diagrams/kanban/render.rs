//! Kanban rendering. Ported from Pele `src/diagrams/kanban/render.ts`.

use super::{KanbanModel, KanbanNode};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::title_room;
use crate::diagrams::common::icon::icon_svg;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg, text_style};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::class_names;
use crate::text::{layout_render_label as layout_label, style, Icons, Measure, RenderLabel as Label, Span};
use crate::theme::Role;
use crate::util::js;
use crate::util::url::{link_url, rel_attr, safe_url, LinkInfo};

const COLUMN_GAP: f64 = 12.0;
const COLUMN_PAD: f64 = 8.0;
const CARD_PAD: f64 = 10.0;
const CARD_GAP: f64 = 8.0;
const META_GAP: f64 = 4.0;
const ICON_GAP: f64 = 6.0;

/// Mermaid names five priorities and gives Medium no color. Matching here ignores case. The high
/// ones are marked in the critical color and the low ones in the muted one, the nearer to Medium
/// the fainter. They are not series colors, as a theme may put those in any order.
fn priority_attrs(priority: &str, ctx: &RenderCtx) -> Option<String> {
    let muted = || ctx.paint(Role::SecondaryText, "var(--_m)");
    match priority {
        "very high" => Some(svg_format!("stroke=\"", ctx.theming.critical(), "\"")),
        "high" => Some(svg_format!("stroke=\"", ctx.theming.critical(), "\" opacity=\"0.5\"")),
        "low" => Some(svg_format!("stroke=\"", muted(), "\" opacity=\"0.5\"")),
        "very low" => Some(svg_format!("stroke=\"", muted(), "\"")),
        _ => None,
    }
}

/// Lays out text for a fixed width. A word too long for a line is broken between characters,
/// because a card cannot grow sideways to hold it.
fn fit_label(raw: Option<&str>, markdown: bool, m: &Measure, size: f64, max: f64, base: u8) -> Label {
    let label = layout_label(raw, markdown, m, size, max, base);
    if label.width <= max {
        return label;
    }
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();
    let mut widest: f64 = 0.0;
    let mut line: Vec<Span> = Vec::new();
    let mut width: f64 = 0.0;
    let commit = |lines: &mut Vec<Vec<Span>>, widths: &mut Vec<f64>, widest: &mut f64, line: &mut Vec<Span>, width: &mut f64| {
        lines.push(std::mem::take(line));
        widths.push(*width);
        *widest = js::max(*widest, *width);
        *width = 0.0;
    };
    for i in 0..label.lines().len() {
        if label.widths()[i] <= max {
            line = label.lines()[i].to_vec();
            width = label.widths()[i];
            commit(&mut lines, &mut widths, &mut widest, &mut line, &mut width);
            continue;
        }
        for span in &label.lines()[i] {
            if span.icon.is_some() || span.math.is_some() {
                if width > 0.0 && width + span.width > max {
                    commit(&mut lines, &mut widths, &mut widest, &mut line, &mut width);
                }
                line.push(span.clone());
                width += span.width;
                continue;
            }
            let mut text = String::new();
            let mut from = width;
            let mut buf = [0u8; 4];
            for ch in span.text.chars() {
                let w = m.width(ch.encode_utf8(&mut buf), size, span.style);
                if width > 0.0 && width + w > max {
                    if !text.is_empty() {
                        line.push(Span { text: std::mem::take(&mut text), style: span.style, icon: None, math: None, width: width - from });
                    }
                    commit(&mut lines, &mut widths, &mut widest, &mut line, &mut width);
                    from = 0.0;
                }
                text.push(ch);
                width += w;
            }
            if !text.is_empty() {
                line.push(Span { text, style: span.style, icon: None, math: None, width: width - from });
            }
        }
        commit(&mut lines, &mut widths, &mut widest, &mut line, &mut width);
    }
    let height = lines.len() as f64 * label.line_height;
    let mut fitted = Label::from_lines(lines, widths, widest, height, label.size, label.line_height);
    if label.heights.is_some() {
        fitted.with_heights();
    }
    fitted
}

/// Writes a label with every line starting at x, or ending there, and the first line's box starting at y.
fn text_block(label: &Label, x: f64, y: f64, attrs: &str, icons: Option<&Icons>, end: bool) -> String {
    let mut out = String::new();
    for (i, line) in label.lines().iter().enumerate() {
        let cy = y + label.line_middle(i);
        if line.len() == 1
            && line[0].icon.is_none()
            && line[0].math.is_none()
            && line[0].style & (style::SUB | style::SUP | style::UNDERLINE) == 0
        {
            svg_write!(
                out,
                "<text",
                text_style(attrs, line[0].style),
                " x=\"",
                num(x),
                "\" y=\"",
                num(cy + label.size * 0.35),
                "\"",
                if end { " text-anchor=\"end\"" } else { "" },
                ">",
                esc(&line[0].text),
                "</text>"
            )
            .unwrap();
        } else {
            let width = label.widths()[i];
            let one = label.label_line(i);
            let half = (if end { -width } else { width }) / 2.0;
            svg_write!(out, label_svg(&one, x + half, cy, attrs, icons)).unwrap();
        }
    }
    out
}

struct Card<'a> {
    item: &'a KanbanNode,
    label: Label,
    ticket: Label,
    assigned: Label,
    /// Height of the rows under the text that hold the ticket and the assignee.
    meta: f64,
    height: f64,
}

pub(crate) fn render(model: &KanbanModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.875);
    let line = js::round(size * 1.5);
    let small_line = js::round(small * 1.5);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let icons = ctx.icons();
    let has = |name: Option<&str>| name.is_some_and(|n| !n.is_empty() && !m.icon_markup(n).is_empty());
    let glyph = small * 1.25;
    let mut math_width = 0.0;
    let mut holds = |raw: Option<&str>, font_size: f64, reserved: f64| {
        let Some(raw) = raw.filter(|r| r.contains("$$")) else {
            return;
        };
        let label = layout_label(Some(raw), true, &m, font_size, f64::INFINITY, 0);
        for spans in label.lines().iter() {
            for span in spans {
                if let Some(math) = &span.math {
                    math_width = js::max(math_width, math.width + reserved);
                }
            }
        }
    };
    for section in &model.sections {
        let count = m.width(&section.items.len().to_string(), small, 0);
        holds(
            Some(&section.node.label),
            size,
            2.0 * COLUMN_PAD + 8.0 + count + 12.0 + if has(section.node.icon.as_deref()) { glyph + ICON_GAP } else { 0.0 },
        );
        for item in &section.items {
            let reserved = 2.0 * (COLUMN_PAD + CARD_PAD);
            holds(Some(&item.label), small, reserved + if has(item.icon.as_deref()) { glyph + ICON_GAP } else { 0.0 });
            holds(item.ticket.as_deref(), small, reserved);
            holds(item.assigned.as_deref(), small, reserved);
        }
    }
    let own_width = js::max(
        match config.lookup("kanban", "sectionWidth") {
            Some(ConfigValue::Number(w)) if *w > 0.0 => js::min(js::max(*w, 80.0), 2000.0),
            _ => 200.0,
        },
        math_width.ceil(),
    );
    // A board wider than the width the host has is folded into rows of columns, which then
    // share the row between them.
    let room = match options.max_width {
        Some(a) if a > 0.0 => a - 2.0 * pad,
        _ => f64::INFINITY,
    };
    let count = model.sections.len() as f64;
    let fits = count * (own_width + COLUMN_GAP) - COLUMN_GAP <= room;
    let per_row = if fits { js::max(count, 1.0) } else { js::max(1.0, ((room + COLUMN_GAP) / (own_width + COLUMN_GAP)).floor()) };
    let column_width = js::max(
        if fits {
            own_width
        } else {
            js::max(js::min(own_width, room), js::min(((room + COLUMN_GAP) / per_row).floor() - COLUMN_GAP, own_width * 2.0))
        },
        math_width.ceil(),
    );
    let base_url = match config.lookup("kanban", "ticketBaseUrl") {
        Some(ConfigValue::String(s)) => s.as_str(),
        _ => "",
    };
    let card_width = column_width - 2.0 * COLUMN_PAD;
    let text_width = card_width - 2.0 * CARD_PAD;
    let mut links: Vec<LinkInfo> = Vec::new();

    let sections = &model.sections;
    let mut titles: Vec<Label> = Vec::new();
    let mut count_widths: Vec<f64> = Vec::new();
    let mut cards: Vec<Vec<Card>> = Vec::new();
    let mut head_height = line;
    let mut body_heights: Vec<f64> = Vec::new();
    for section in sections {
        let count_width = m.width(&section.items.len().to_string(), small, 0);
        let reserved = count_width + 12.0 + if has(section.node.icon.as_deref()) { glyph + ICON_GAP } else { 0.0 };
        let title = fit_label(Some(&section.node.label), true, &m, size, card_width - 8.0 - reserved, style::BOLD);
        head_height = js::max(head_height, title.height);
        count_widths.push(count_width);
        titles.push(title);

        let mut list: Vec<Card> = Vec::new();
        let mut height = 0.0;
        for item in &section.items {
            let label = fit_label(
                Some(&item.label),
                true,
                &m,
                small,
                text_width - if has(item.icon.as_deref()) { glyph + ICON_GAP } else { 0.0 },
                0,
            );
            let ticket = fit_label(item.ticket.as_deref(), false, &m, small, text_width, 0);
            let assigned = fit_label(item.assigned.as_deref(), false, &m, small, text_width, 0);
            let meta = if ticket.width + assigned.width + 8.0 > text_width {
                ticket.height + assigned.height
            } else {
                js::max(ticket.height, assigned.height)
            };
            let card_height =
                2.0 * CARD_PAD + js::max(label.height, small_line) + if meta != 0.0 && !meta.is_nan() { META_GAP + meta } else { 0.0 };
            list.push(Card { item, label, ticket, assigned, meta, height: card_height });
            height += card_height + CARD_GAP;
        }
        cards.push(list);
        body_heights.push(height);
    }
    head_height += 2.0 * COLUMN_PAD;
    let mut row_heights: Vec<f64> = Vec::new();
    let mut row_tops: Vec<f64> = Vec::new();
    // Counted in numbers, as Pele counts: a row length that is not a number (from a padding that
    // is not one) ends the loop after one row instead of never.
    let mut first = 0.0;
    let mut y = 0.0;
    while first < count {
        let mut tallest = CARD_GAP;
        let stop = js::min(first + per_row, count);
        let mut index = first;
        while index < stop {
            tallest = js::max(tallest, body_heights[index as usize]);
            index += 1.0;
        }
        row_tops.push(y);
        row_heights.push(head_height + tallest);
        y += head_height + tallest + COLUMN_GAP;
        first += per_row;
    }
    let board_height = match (row_tops.last(), row_heights.last()) {
        (Some(top), Some(height)) => top + height,
        _ => 0.0,
    };

    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let board_width = js::max(js::min(count, per_row) * (column_width + COLUMN_GAP) - COLUMN_GAP, 0.0);
    let width = js::max(board_width, title.width);
    let left = pad + (width - board_width) / 2.0;
    let edge = column_width - COLUMN_PAD - 4.0;

    let resolved = ctx.resolved();
    let column_paint = if resolved {
        // Zebra draws groups with a fill and an outline. Its group fill is the canvas color
        // unless the caller gives one, and a column would not show without the outline.
        svg_format!(" fill=\"", ctx.paint(Role::GroupFill, ""), "\" stroke=\"", ctx.paint(Role::GroupStroke, ""), "\"")
    } else {
        " fill=\"var(--_s)\"".to_string()
    };
    let muted = ctx.paint(Role::SecondaryText, "var(--_m)");
    let mut out = String::new();
    for (index, section) in sections.iter().enumerate() {
        // `rowHeights[row]` and `rowTops[row]`, undefined (NaN in arithmetic) past the end.
        let row = (index as f64 / per_row).floor();
        let row_height = row_heights.get(row as usize).copied().filter(|_| row >= 0.0).unwrap_or(f64::NAN);
        let row_top = row_tops.get(row as usize).copied().filter(|_| row >= 0.0).unwrap_or(f64::NAN);
        let mut body = svg_format!(
            "<rect width=\"",
            js::number_text(column_width),
            "\" height=\"",
            num(row_height),
            "\"",
            crate::svg::theme::rounded(&svg_format!(column_paint)),
            "/>"
        );
        body.push_str(&text_block(
            &titles[index],
            COLUMN_PAD + 4.0,
            COLUMN_PAD,
            " class=\"pele-cluster-label\" font-weight=\"var(--_hw)\"",
            icons,
            false,
        ));
        svg_write!(
            body,
            "<text class=\"pele-count\" x=\"",
            js::number_text(edge),
            "\" y=\"",
            num(COLUMN_PAD + line / 2.0 + small * 0.35),
            "\" text-anchor=\"end\" font-size=\"",
            js::number_text(small),
            "\" fill=\"",
            muted,
            "\">",
            section.items.len(),
            "</text>"
        )
        .unwrap();
        if let Some(icon) = section.node.icon.as_deref().filter(|&i| has(Some(i))) {
            body.push_str(&icon_svg(icon, edge - count_widths[index] - ICON_GAP - glyph, COLUMN_PAD + (line - glyph) / 2.0, glyph, icons));
        }

        let mut y = head_height;
        for card in &cards[index] {
            let item = card.item;
            let priority = js::trim(item.priority.as_deref().unwrap_or("")).to_lowercase();
            let color = priority_attrs(&priority, ctx);
            let mut inner = svg_format!(
                "<rect width=\"",
                js::number_text(card_width),
                "\" height=\"",
                num(card.height),
                "\"",
                crate::svg::theme::rounded(&svg_format!(
                    " fill=\"",
                    ctx.paint(Role::NodeFill, "var(--_bg)"),
                    "\" stroke=\"",
                    ctx.paint(Role::NodeStroke, "var(--_b)"),
                    "\""
                )),
                "/>"
            );
            if let Some(color) = &color {
                svg_write!(
                    inner,
                    "<path class=\"pele-priority\" d=\"M2.5,5V",
                    num(card.height - 5.0),
                    "\" fill=\"none\" ",
                    color,
                    " stroke-width=\"3\" stroke-linecap=\"round\"/>"
                )
                .unwrap();
            }
            inner.push_str(&text_block(&card.label, CARD_PAD, CARD_PAD, " class=\"pele-label\"", icons, false));
            if let Some(icon) = item.icon.as_deref().filter(|&i| has(Some(i))) {
                inner.push_str(&icon_svg(icon, card_width - CARD_PAD - glyph, CARD_PAD + (small_line - glyph) / 2.0, glyph, icons));
            }
            let bottom = card.height - CARD_PAD;
            if let Some(ticket) = item.ticket.as_deref().filter(|t| !t.is_empty()) {
                let top = bottom - card.meta;
                let href = if base_url.is_empty() { None } else { link_url(&safe_url(&base_url.replacen("#TICKET#", ticket, 1)), options) };
                match href {
                    None => {
                        inner.push_str(&text_block(
                            &card.ticket,
                            CARD_PAD,
                            top,
                            &svg_format!(" class=\"pele-ticket\" fill=\"", muted, "\""),
                            icons,
                            false,
                        ));
                    }
                    Some(href) => {
                        svg_write!(inner, "<a href=\"", esc(&href), "\" target=\"_blank\"", rel_attr(options), ">").unwrap();
                        inner.push_str(&text_block(
                            &card.ticket,
                            CARD_PAD,
                            top,
                            &svg_format!(" class=\"pele-ticket\" fill=\"", ctx.paint(Role::Accent, "var(--_c)"), "\""),
                            icons,
                            false,
                        ));
                        inner.push_str("</a>");
                        links.push(LinkInfo { id: item.id.clone(), href, internal: false });
                    }
                }
            }
            if item.assigned.as_deref().is_some_and(|a| !a.is_empty()) {
                inner.push_str(&text_block(
                    &card.assigned,
                    card_width - CARD_PAD,
                    bottom - card.assigned.height,
                    &svg_format!(" class=\"pele-assigned\" fill=\"", muted, "\""),
                    icons,
                    true,
                ));
            }
            svg_write!(
                body,
                "<g class=\"pele-node pele-card",
                if color.is_none() { String::new() } else { format!(" pele-priority-{}", priority.replacen(' ', "-", 1)) },
                class_names(item.css_classes.as_deref().unwrap_or("")),
                "\" data-id=\"",
                esc_text(&item.id),
                "\" transform=\"translate(",
                js::number_text(COLUMN_PAD),
                ",",
                num(y),
                ")\" font-size=\"",
                js::number_text(small),
                "\">",
                inner,
                "</g>"
            )
            .unwrap();
            y += card.height + CARD_GAP;
        }

        svg_write!(
            out,
            "<g class=\"pele-cluster pele-column",
            class_names(section.node.css_classes.as_deref().unwrap_or("")),
            "\" data-id=\"",
            esc_text(&section.node.id),
            "\" transform=\"translate(",
            num(left + (index as f64 % per_row) * (column_width + COLUMN_GAP)),
            ",",
            num(pad + title_height + row_top),
            ")\">",
            body,
            "</g>"
        )
        .unwrap();
    }

    let total_width = (width + 2.0 * pad).ceil();
    let total_height = (board_height + title_height + 2.0 * pad).ceil();
    let mut content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        None
    ));
    content.push_str(&out);

    Ok(Rendered {
        svg: svg_document("kanban", total_width, total_height, size, ctx, Accessible::default(), &content),
        width: total_width,
        height: total_height,
        links,
    })
}
