//! Timeline rendering. Ported from Pele `src/diagrams/timeline/render.ts`.

use super::TimelineModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit::{fit_render_label as fit_label, plain_render_label as plain_label};
use crate::diagrams::common::fit_width::turn_to_fit;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::edges::{marker, ARROW};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::{style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::js;

const PAD_X: f64 = 12.0;
const PAD_Y: f64 = 8.0;
const MIN_TEXT: f64 = 152.0;
const MAX_TEXT: f64 = 216.0;
const EVENT_TEXT_TD: f64 = 264.0;
const GAP: f64 = 20.0;
const AXIS: f64 = 20.0;
const SPINE: f64 = 14.0;
const EVENT_GAP: f64 = 8.0;
const SECTION_GAP: f64 = 8.0;
const TAIL: f64 = 12.0;

struct Group {
    /// Index into the sections, -1 for the periods that come before the first one.
    section: i64,
    periods: Vec<usize>,
}

fn rect(x: f64, y: f64, w: f64, h: f64, attrs: &str) -> String {
    svg_format!(
        "<rect x=\"",
        num(x),
        "\" y=\"",
        num(y),
        "\" width=\"",
        num(w),
        "\" height=\"",
        num(h),
        "\" rx=\"",
        RADIUS,
        "\"",
        attrs,
        "/>"
    )
}

pub(crate) fn render(model: &TimelineModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if model.direction == "TD" {
        return Ok(draw(model, config, ctx, true));
    }
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, down)))
}

fn draw(model: &TimelineModel, config: &Config, ctx: &RenderCtx, vertical: bool) -> Rendered {
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let multicolor = !matches!(config.lookup("timeline", "disableMulticolor"), Some(ConfigValue::Bool(true)));
    let periods = &model.periods;
    let has_sections = !model.sections.is_empty();

    // Mermaid draws nothing for periods that come before the first section; here they lead,
    // in a neutral color.
    let mut groups: Vec<Group> = (0..model.sections.len()).map(|section| Group { section: section as i64, periods: Vec::new() }).collect();
    let mut leading = Group { section: -1, periods: Vec::new() };
    for (i, period) in periods.iter().enumerate() {
        match usize::try_from(period.section_index).ok().and_then(|k| groups.get_mut(k)) {
            Some(group) => group.periods.push(i),
            None => leading.periods.push(i),
        }
    }
    if !leading.periods.is_empty() {
        groups.insert(0, leading);
    }

    // Columns widen, up to a point, for a word that would not fit; past that the word is cut.
    // Running down, the two columns share the width the host has.
    let room = match options.max_width {
        Some(w) if vertical && w > 0.0 => w - 2.0 * pad,
        _ => f64::INFINITY,
    };
    let taken = AXIS + SPINE * 2.0 + 4.0 * PAD_X;
    let period_text = js::min(MIN_TEXT, js::max(80.0, js::round((room - taken) * 0.38)));
    let event_text = js::min(EVENT_TEXT_TD, js::max(110.0, room - taken - period_text));
    let mut text_w = period_text;
    let event_text_w = |text_w: f64| if vertical { event_text } else { text_w };
    let mut period_labels: Vec<Label> = periods.iter().map(|p| plain_label(Some(&p.text), &m, size, text_w, 0)).collect();
    let mut event_labels: Vec<Vec<Label>> =
        periods.iter().map(|p| p.events.iter().map(|e| plain_label(Some(e), &m, small, event_text_w(text_w), 0)).collect()).collect();
    let mut widest: f64 = 0.0;
    for label in &period_labels {
        widest = js::max(widest, label.width);
    }
    if !vertical {
        for labels in &event_labels {
            for label in labels {
                widest = js::max(widest, label.width);
            }
        }
    }
    let mut overflow = widest > text_w;
    if vertical {
        for labels in &event_labels {
            for label in labels {
                overflow = overflow || label.width > event_text;
            }
        }
    }
    if overflow {
        text_w = js::min(js::max(widest.ceil(), period_text), if room == f64::INFINITY { MAX_TEXT } else { period_text });
        period_labels = periods.iter().map(|p| fit_label(Some(&p.text), &m, size, text_w, 0)).collect();
        event_labels =
            periods.iter().map(|p| p.events.iter().map(|e| fit_label(Some(e), &m, small, event_text_w(text_w), 0)).collect()).collect();
    }

    let line_height = js::round(size * 1.5);
    let col_w = text_w + 2.0 * PAD_X;
    let event_w = event_text_w(text_w) + 2.0 * PAD_X;
    let chip_height = |label: &Label| js::max(label.height, line_height) + 2.0 * PAD_Y;
    let event_height = |label: &Label| js::max(label.height, js::round(small * 1.5)) + 2.0 * PAD_Y;
    // Leading periods take Pele's muted color when sections follow; in resolved mode that is
    // the secondary text color, the neutral of Zebra's palette.
    let color_of = |group: &Group, period: usize| -> String {
        if group.section >= 0 {
            ctx.series(group.section).into_owned()
        } else if has_sections {
            ctx.paint(Role::SecondaryText, "var(--_m)").to_string()
        } else {
            ctx.series(if multicolor { period as i64 } else { 0 }).into_owned()
        }
    };

    let has_events = event_labels.iter().any(|labels| !labels.is_empty());
    let mut slots = 0.0;
    for group in &groups {
        slots += group.periods.len().max(1) as f64;
    }

    let pitch = col_w + GAP;
    let drawn = !periods.is_empty();
    let body_w = if vertical {
        col_w
            + if has_events {
                AXIS + SPINE * 2.0 + event_w
            } else if drawn {
                AXIS + TAIL
            } else {
                0.0
            }
    } else if slots > 0.0 {
        slots * pitch - GAP + if drawn { TAIL + ARROW } else { 0.0 }
    } else {
        0.0
    };
    let title = fit_label(model.title.as_deref(), &m, size, js::max(body_w, 320.0), style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let width = js::max(body_w, title.width);
    let left = pad + (width - body_w) / 2.0;
    let top = pad + title_h;

    // Text on a series fill: the canvas color in CSS mode, a contrasting color when resolved.
    let on_color = |fill: &str| svg_format!(" fill=\"", ctx.theming.text_on(fill, "var(--_bg)"), "\"");
    let section_label = |group: &Group, w: f64| -> Label {
        fit_label(Some(&model.sections[group.section as usize]), &m, size, w - 2.0 * PAD_X, style::BOLD)
    };
    let header = |group: &Group, label: &Label, x: f64, y: f64, w: f64, h: f64| -> String {
        let fill = ctx.series(group.section);
        rect(x, y, w, h, &svg_format!(" fill=\"", fill, "\""))
            + &svg_format!(label_svg(label, x + w / 2.0, y + h / 2.0, &svg_format!(" font-weight=\"var(--_hw)\"", on_color(&fill)), None))
    };
    let period = |labels: &[Label], i: usize, color: &str, x: f64, y: f64, h: f64| -> String {
        rect(x, y, col_w, h, &svg_format!(" fill=\"", color, "\""))
            + &svg_format!(label_svg(&labels[i], x + col_w / 2.0, y + h / 2.0, &on_color(color), None))
    };
    let event = |label: &Label, color: &str, x: f64, y: f64, h: f64| -> String {
        svg_format!(
            "<g class=\"pele-node pele-event\">",
            rect(x, y, event_w, h, &svg_format!(" fill=\"", color, "\" fill-opacity=\"0.16\"")),
            label_svg(label, x + event_w / 2.0, y + h / 2.0, &svg_format!(" font-size=\"", js::to_string(small), "\""), None),
            "</g>"
        )
    };
    let dot = |x: f64, y: f64, color: &str| -> String {
        svg_format!(
            "<circle class=\"pele-dot\" cx=\"",
            num(x),
            "\" cy=\"",
            num(y),
            "\" r=\"4\" fill=\"",
            color,
            "\" stroke=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke-width=\"2\"/>"
        )
    };
    let open = |group: &Group| -> String {
        if group.section >= 0 {
            svg_format!(
                "<g class=\"pele-cluster pele-section\" data-id=\"",
                esc_text(js::trim(&model.sections[group.section as usize])),
                "\">"
            )
        } else {
            String::new()
        }
    };
    let open_period = |i: usize| svg_format!("<g class=\"pele-node pele-period\" data-id=\"", esc_text(js::trim(&periods[i].text)), "\">");
    // The connector from a period to its events: Pele draws it in the border color.
    let line = svg_format!(" fill=\"none\" stroke=\"", ctx.paint(Role::Line, "var(--_b)"), "\"");
    let axis_color = ctx.paint(Role::Line, "var(--_l)");
    let arrow_color = ctx.paint(Role::Arrow, "var(--_l)");

    let mut body = String::new();
    let height: f64;

    if vertical {
        let axis_x = left + col_w + AXIS;
        let spine_x = axis_x + SPINE;
        let events_x = spine_x + SPINE;
        let mut y = top;
        for group in &groups {
            body.push_str(&open(group));
            if group.section >= 0 {
                let label = section_label(group, body_w);
                let h = chip_height(&label);
                body.push_str(&header(group, &label, left, y, body_w, h));
                y += h + SECTION_GAP + 4.0;
            }
            for &i in &group.periods {
                let color = color_of(group, i);
                let h = chip_height(&period_labels[i]);
                let cy = y + h / 2.0;
                let mut d = svg_format!("M", num(left + col_w), ",", num(cy), "H", num(axis_x));
                let mut cards = String::new();
                let mut ey = y;
                let mut last = cy;
                for label in &event_labels[i] {
                    let eh = event_height(label);
                    last = ey + eh / 2.0;
                    svg_write!(d, "M", num(spine_x), ",", num(last), "H", num(events_x)).unwrap();
                    cards.push_str(&event(label, &color, events_x, ey, eh));
                    ey += eh + EVENT_GAP;
                }
                if !cards.is_empty() {
                    svg_write!(d, "M", num(axis_x), ",", num(cy), "H", num(spine_x), "V", num(last)).unwrap();
                }
                body.push_str(&open_period(i));
                body.push_str(&period(&period_labels, i, &color, left, y, h));
                svg_write!(body, "<path class=\"pele-edge\" d=\"", d, "\"", line, "/>").unwrap();
                body.push_str(&dot(axis_x, cy, &color));
                body.push_str(&cards);
                body.push_str("</g>");
                y = js::max(y + h, ey - EVENT_GAP) + GAP;
            }
            if group.periods.is_empty() {
                y += GAP - SECTION_GAP - 4.0;
            }
            if group.section >= 0 {
                body.push_str("</g>");
            }
        }
        if !groups.is_empty() {
            y -= GAP;
        }
        if drawn {
            let tip = y + TAIL + ARROW;
            body = svg_format!(
                "<path class=\"pele-axis\" d=\"M",
                num(axis_x),
                ",",
                num(top),
                "V",
                num(tip - ARROW + 1.0),
                "\" fill=\"none\" stroke=\"",
                axis_color,
                "\"/>",
                marker("arrow_point", axis_x, tip, 0.0, 1.0, arrow_color),
                body
            );
            y = tip;
        }
        height = y + pad;
    } else {
        let mut header_h: f64 = 0.0;
        let section_labels: Vec<Option<Label>> = groups
            .iter()
            .map(|group| {
                if group.section < 0 {
                    return None;
                }
                let label = section_label(group, group.periods.len().max(1) as f64 * pitch - GAP);
                header_h = js::max(header_h, chip_height(&label));
                Some(label)
            })
            .collect();
        let mut chip_h: f64 = 0.0;
        for label in &period_labels {
            chip_h = js::max(chip_h, chip_height(label));
        }
        let periods_y = top + if has_sections { header_h + SECTION_GAP } else { 0.0 };
        let axis_y = periods_y + chip_h + if chip_h > 0.0 { AXIS } else { 0.0 };
        let events_y = axis_y + AXIS;
        let mut bottom = if drawn { axis_y + TAIL } else { top + header_h };
        let mut slot = 0.0;
        for (g, group) in groups.iter().enumerate() {
            let span = group.periods.len().max(1) as f64;
            let gx = left + slot * pitch;
            body.push_str(&open(group));
            if let Some(label) = &section_labels[g] {
                body.push_str(&header(group, label, gx, top, span * pitch - GAP, header_h));
            }
            for (k, &i) in group.periods.iter().enumerate() {
                let color = color_of(group, i);
                let x = gx + k as f64 * pitch;
                let cx = x + col_w / 2.0;
                let mut d = svg_format!(
                    "M",
                    num(cx),
                    ",",
                    num(periods_y + chip_h),
                    "V",
                    num(if !event_labels[i].is_empty() { events_y } else { axis_y })
                );
                let mut cards = String::new();
                let mut ey = events_y;
                for event_label in &event_labels[i] {
                    let eh = event_height(event_label);
                    if ey > events_y {
                        svg_write!(d, "M", num(cx), ",", num(ey - EVENT_GAP), "V", num(ey)).unwrap();
                    }
                    cards.push_str(&event(event_label, &color, x, ey, eh));
                    ey += eh + EVENT_GAP;
                }
                bottom = js::max(bottom, ey - EVENT_GAP);
                body.push_str(&open_period(i));
                body.push_str(&period(&period_labels, i, &color, x, periods_y, chip_h));
                svg_write!(body, "<path class=\"pele-edge\" d=\"", d, "\"", line, "/>").unwrap();
                body.push_str(&dot(cx, axis_y, &color));
                body.push_str(&cards);
                body.push_str("</g>");
            }
            if group.section >= 0 {
                body.push_str("</g>");
            }
            slot += span;
        }
        if drawn {
            let tip = left + body_w;
            body = svg_format!(
                "<path class=\"pele-axis\" d=\"M",
                num(left),
                ",",
                num(axis_y),
                "H",
                num(tip - ARROW + 1.0),
                "\" fill=\"none\" stroke=\"",
                axis_color,
                "\"/>",
                marker("arrow_point", tip, axis_y, 1.0, 0.0, arrow_color),
                body
            );
        }
        height = bottom + pad;
    }

    let total_width = (width + 2.0 * pad).ceil();
    let total_height = js::max(height, pad * 2.0).ceil();
    let content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        None
    )) + &body;
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("timeline", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
