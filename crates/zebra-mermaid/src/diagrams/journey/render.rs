//! Journey rendering. Ported from Pele `src/diagrams/journey/render.ts`.
//!
//! Colors in resolved mode: Pele paints every task card and section band in the neutral
//! surfaces. Zebra gives each section a color of its own, as Mermaid's journey does with its
//! section fills: a section's band and its task cards take the chart color of that section
//! (counted by the runs of tasks that draw a band), and the text on them takes a color that
//! reads on that fill. Tasks outside any section keep the node fill. CSS output is Pele's.

use crate::svg::concat::{svg_format, svg_write};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;

use super::{actor_names, JourneyModel};
use crate::config::Config;
use crate::diagrams::common::fit::{fit_label, plain_label};
use crate::diagrams::common::fit_width::turn_to_fit;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::{style, Label};
use crate::theme::Role;
use crate::util::js;

const PAD_X: f64 = 10.0;
const PAD_Y: f64 = 8.0;
const MIN_TEXT: f64 = 124.0;
const MAX_TEXT: f64 = 200.0;
const GAP: f64 = 12.0;
const BAND_PAD: f64 = 6.0;
const GUTTER: f64 = 24.0;
const LEVEL: f64 = 26.0;
const FACE: f64 = 11.0;
const DOT: f64 = 5.0;
const DOT_STEP: f64 = 13.0;
const LEGEND_GAP: f64 = 20.0;

const EYES: &str = "<circle cx=\"-3.6\" cy=\"-2.8\" r=\"1.3\"/><circle cx=\"3.6\" cy=\"-2.8\" r=\"1.3\"/>";

#[derive(Clone, Copy)]
enum Mood {
    Happy,
    Neutral,
    Sad,
}

impl Mood {
    /// Above three is happy and below it sad; a score that is not a number is neutral.
    fn of(score: f64) -> Mood {
        if score > 3.0 {
            Mood::Happy
        } else if score < 3.0 {
            Mood::Sad
        } else {
            Mood::Neutral
        }
    }

    fn name(self) -> &'static str {
        match self {
            Mood::Happy => "happy",
            Mood::Neutral => "neutral",
            Mood::Sad => "sad",
        }
    }

    fn mouth(self) -> &'static str {
        match self {
            Mood::Happy => "M-5,2A6,6 0 0 0 5,2",
            Mood::Neutral => "M-4.5,4.5H4.5",
            Mood::Sad => "M-5,6.5A6,6 0 0 1 5,6.5",
        }
    }
}

/// A face on the score line: a disc of the canvas color that hides the line behind it, drawn
/// in the line color.
fn face(out: &mut String, ctx: &RenderCtx, x: f64, y: f64, mood: Mood) {
    let line = ctx.paint(Role::Line, "var(--_l)");
    svg_write!(
        out,
        "<g class=\"pele-face pele-face-",
        mood.name(),
        "\" transform=\"translate(",
        num(x),
        ",",
        num(y),
        ")\"><circle r=\"",
        num(FACE),
        "\" fill=\"",
        ctx.paint(Role::Background, "var(--_bg)"),
        "\" stroke=\"",
        line,
        "\"/><g fill=\"",
        line,
        "\">",
        EYES,
        "</g><path d=\"",
        mood.mouth(),
        "\" fill=\"none\" stroke=\"",
        line,
        "\" stroke-linecap=\"round\"/></g>"
    )
    .unwrap();
}

/// The section color of each task: the number of the run of tasks it belongs to, among the
/// runs that have a section, or `None` outside any section. Runs are split as the bands are.
fn section_colors(model: &JourneyModel) -> Vec<Option<i64>> {
    let tasks = &model.tasks;
    let mut out = Vec::with_capacity(tasks.len());
    let mut next = 0;
    let mut current = None;
    for (i, task) in tasks.iter().enumerate() {
        if i == 0 || task.section != tasks[i - 1].section {
            current = if task.section.is_empty() {
                None
            } else {
                next += 1;
                Some(next - 1)
            };
        }
        out.push(current);
    }
    out
}

/// What fills a task card or section band: Pele's surface `css` in CSS mode; in resolved mode
/// the section's chart color, or `role` outside any section.
fn section_fill<'a>(ctx: &'a RenderCtx, color: Option<i64>, role: Role, css: &'static str) -> Cow<'a, str> {
    match color {
        Some(k) if ctx.resolved() => ctx.series(k),
        _ => Cow::Borrowed(ctx.paint(role, css)),
    }
}

/// A fill attribute for text on `fill` in resolved mode. In CSS mode the text inherits the
/// root color, as in Pele, and nothing is written.
fn text_on(ctx: &RenderCtx, fill: &str) -> String {
    if ctx.resolved() {
        svg_format!(" fill=\"", ctx.theming.text_on(fill, ""), "\"")
    } else {
        String::new()
    }
}

pub(crate) fn render(model: &JourneyModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, down)))
}

struct LegendItem<'a> {
    name: &'a str,
    label: Label,
    x: f64,
    y: f64,
    h: f64,
}

struct Run {
    start: usize,
    count: usize,
    label: Option<Label>,
}

fn draw(model: &JourneyModel, _config: &Config, ctx: &RenderCtx, down: bool) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.75);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    if down {
        return vertical_journey(model, ctx);
    }
    let tasks = &model.tasks;
    let n = tasks.len();
    let colors = section_colors(model);

    // `Task: 5:` names an actor with no name; Mermaid gives it a dot and an empty legend row.
    let mut names = actor_names(tasks);
    names.retain(|name| !name.is_empty());
    let actor: HashMap<&str, usize> = names.iter().enumerate().map(|(i, name)| (*name, i)).collect();

    // Cards widen, up to a point, for a word that would not fit; past that the word is cut.
    let mut text_w = MIN_TEXT;
    let mut labels: Vec<Label> = tasks.iter().map(|task| plain_label(Some(&task.task), &m, small, text_w, 0)).collect();
    let mut widest: f64 = 0.0;
    for label in &labels {
        widest = js::max(widest, label.width);
    }
    if widest > text_w {
        text_w = js::min(widest.ceil(), MAX_TEXT);
        labels = tasks.iter().map(|task| fit_label(Some(&task.task), &m, small, text_w, 0)).collect();
    }

    let card_w = text_w + 2.0 * PAD_X;
    let pitch = card_w + GAP;
    let plot_w = if n > 0 { n as f64 * pitch - GAP } else { 0.0 };

    let line_h = js::round(small * 1.5);
    let limit = js::max(plot_w, 240.0);
    let mut legend: Vec<LegendItem> = Vec::new();
    let mut legend_w: f64 = 0.0;
    let mut legend_h: f64 = 0.0;
    let mut lx: f64 = 0.0;
    for name in &names {
        let label = fit_label(Some(name), &m, small, limit - 2.0 * DOT - 6.0, 0);
        let w = 2.0 * DOT + 6.0 + label.width;
        if lx > 0.0 && lx + w > limit {
            lx = 0.0;
            legend_h += 4.0;
        }
        let y = match legend.last() {
            Some(last) if lx > 0.0 => last.y,
            _ => legend_h,
        };
        let h = js::max(label.height, line_h);
        legend.push(LegendItem { name, label, x: lx, y, h });
        lx += w;
        legend_w = js::max(legend_w, lx);
        legend_h = js::max(legend_h, y + h);
        lx += LEGEND_GAP;
    }

    let body_w = if n > 0 { GUTTER + plot_w } else { 0.0 };
    let title = fit_label(model.title.as_deref(), &m, size, js::max(body_w, 320.0), style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let width = js::max(js::max(body_w, title.width), if n > 0 { GUTTER + legend_w } else { 0.0 });
    let left = pad + (width - body_w) / 2.0 + GUTTER;
    let top = pad + title_h;

    // A band covers a run of tasks that share a section, as in Mermaid; a section with no
    // tasks draws nothing.
    let mut runs: Vec<Run> = Vec::new();
    let mut band_h: f64 = 0.0;
    let mut i = 0;
    while i < n {
        let mut end = i + 1;
        while end < n && tasks[end].section == tasks[i].section {
            end += 1;
        }
        let mut label = None;
        if !tasks[i].section.is_empty() {
            let l = fit_label(Some(&tasks[i].section), &m, small, (end - i) as f64 * pitch - GAP - 2.0 * PAD_X, style::BOLD);
            band_h = js::max(band_h, js::max(l.height, line_h) + 2.0 * BAND_PAD);
            label = Some(l);
        }
        runs.push(Run { start: i, count: end - i, label });
        i = end;
    }

    let mut text_h = line_h;
    for label in &labels {
        text_h = js::max(text_h, label.height);
    }
    let dots_h = if !names.is_empty() { 2.0 * DOT + 6.0 } else { 0.0 };
    let card_h = text_h + dots_h + 2.0 * PAD_Y;
    let cards_y = top + if band_h > 0.0 { band_h + BAND_PAD } else { 0.0 };
    let chart_top = cards_y + card_h + FACE + 16.0;
    let chart_bottom = chart_top + 4.0 * LEVEL;
    let right = left + plot_w;

    let small_s = js::to_string(small);
    let mut body = String::new();
    let mut bottom = top;
    if n > 0 {
        let mut grid = String::new();
        let mut scale = String::new();
        for score in (1..=5).rev() {
            let y = chart_top + (5 - score) as f64 * LEVEL;
            svg_write!(grid, "M", num(left - 6.0), ",", num(y), "H", num(right)).unwrap();
            svg_write!(scale, "<text x=\"", num(left - 12.0), "\" y=\"", num(y + tiny * 0.35), "\">", score, "</text>").unwrap();
        }
        svg_write!(
            body,
            "<path class=\"pele-grid\" d=\"",
            grid,
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::InnerStroke, "var(--_a)"),
            "\"/><g class=\"pele-scale\" font-size=\"",
            js::to_string(tiny),
            "\" text-anchor=\"end\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            scale,
            "</g>"
        )
        .unwrap();

        let mut line = String::new();
        let mut stems = String::new();
        let mut faces = String::new();
        let mut cards = String::new();
        let mut card_offsets = Vec::with_capacity(n + 1);
        card_offsets.push(0);
        for (i, task) in tasks.iter().enumerate() {
            let x = left + i as f64 * pitch;
            let cx = x + card_w / 2.0;
            let score = task.score;
            let level = if score.is_nan() { 3.0 } else { js::min(5.0, js::max(1.0, score)) };
            let y = chart_top + (5.0 - level) * LEVEL;
            write!(line, "{}{},{}", if i == 0 { "M" } else { "L" }, num(cx), num(y)).unwrap();
            svg_write!(stems, "M", num(cx), ",", num(cards_y + card_h), "V", num(y - FACE)).unwrap();
            face(&mut faces, ctx, cx, y, Mood::of(score));

            let fill = section_fill(ctx, colors[i], Role::NodeFill, "var(--_s)");
            svg_write!(
                cards,
                "<g class=\"pele-node pele-task\" data-id=\"",
                esc_text(js::trim(&task.task)),
                "\"><rect x=\"",
                num(x),
                "\" y=\"",
                num(cards_y),
                "\" width=\"",
                num(card_w),
                "\" height=\"",
                num(card_h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                fill,
                "\" stroke=\"",
                ctx.paint(Role::NodeStroke, "var(--_b)"),
                "\"/>",
                label_svg(
                    &labels[i],
                    cx,
                    cards_y + PAD_Y + text_h / 2.0,
                    &svg_format!(" font-size=\"", small_s, "\"", text_on(ctx, &fill)),
                    None
                )
            )
            .unwrap();
            let people = task.people.iter().filter(|name| !name.is_empty());
            let count = people.clone().count() as f64;
            let step = js::min(DOT_STEP, (card_w - 2.0 * PAD_X - 2.0 * DOT) / js::max(count - 1.0, 1.0));
            for (k, name) in people.enumerate() {
                // The ring around a dot is the card's own fill, which parts dots that touch.
                svg_write!(
                    cards,
                    "<circle class=\"pele-actor\" cx=\"",
                    num(cx + (k as f64 - (count - 1.0) / 2.0) * step),
                    "\" cy=\"",
                    num(cards_y + card_h - PAD_Y - DOT),
                    "\" r=\"",
                    num(DOT),
                    "\" fill=\"",
                    ctx.series(actor.get(name.as_str()).map_or(0, |&a| a as i64)),
                    "\" stroke=\"",
                    fill,
                    "\"><title>",
                    esc_text(name),
                    "</title></circle>"
                )
                .unwrap();
            }
            cards.push_str("</g>");
            card_offsets.push(cards.len());
        }
        svg_write!(
            body,
            "<path class=\"pele-stems\" d=\"",
            stems,
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\" stroke-dasharray=\"2 3\"/>"
        )
        .unwrap();
        if n > 1 {
            svg_write!(
                body,
                "<path class=\"pele-edge\" d=\"",
                line,
                "\" fill=\"none\" stroke=\"",
                ctx.paint(Role::Line, "var(--_l)"),
                "\" stroke-linejoin=\"round\"/>"
            )
            .unwrap();
        }
        body.push_str(&faces);

        for run in &runs {
            let members = &cards[card_offsets[run.start]..card_offsets[run.start + run.count]];
            let Some(label) = &run.label else {
                body.push_str(members);
                continue;
            };
            let x = left + run.start as f64 * pitch;
            let w = run.count as f64 * pitch - GAP;
            // A band always belongs to a section, so in resolved mode it has a section color.
            let fill = section_fill(ctx, colors[run.start], Role::GroupHeader, "var(--_a)");
            svg_write!(
                body,
                "<g class=\"pele-cluster pele-section\" data-id=\"",
                esc_text(js::trim(&tasks[run.start].section)),
                "\"><rect x=\"",
                num(x),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(band_h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                fill,
                "\"/>",
                label_svg(
                    label,
                    x + w / 2.0,
                    top + band_h / 2.0,
                    &svg_format!(" font-size=\"", small_s, "\" font-weight=\"var(--_hw)\"", text_on(ctx, &fill)),
                    None
                ),
                members,
                "</g>"
            )
            .unwrap();
        }
        bottom = chart_bottom + FACE;

        if !legend.is_empty() {
            let x0 = left + (plot_w - legend_w) / 2.0;
            let y0 = bottom + 16.0;
            svg_write!(body, "<g class=\"pele-legend\" font-size=\"", small_s, "\">").unwrap();
            for item in &legend {
                let cy = y0 + item.y + item.h / 2.0;
                svg_write!(
                    body,
                    "<g class=\"pele-legend-item\" data-id=\"",
                    esc_text(item.name),
                    "\"><circle cx=\"",
                    num(x0 + item.x + DOT),
                    "\" cy=\"",
                    num(cy),
                    "\" r=\"",
                    num(DOT),
                    "\" fill=\"",
                    ctx.series(actor.get(item.name).map_or(0, |&a| a as i64)),
                    "\"/>",
                    label_svg(&item.label, x0 + item.x + 2.0 * DOT + 6.0 + item.label.width / 2.0, cy, "", None),
                    "</g>"
                )
                .unwrap();
            }
            body.push_str("</g>");
            bottom = y0 + legend_h;
        }
    }

    let total_width = (width + 2.0 * pad).ceil();
    let total_height = (js::max(bottom, pad) + pad).ceil();
    let content = if title.lines.is_empty() {
        body
    } else {
        label_svg(&title, pad + width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string()
            + &body
    };
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("journey", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}

struct Person<'a> {
    name: &'a str,
    label: Label,
}

/// One card under another, for a host too narrow for the chart.
fn vertical_journey(model: &JourneyModel, ctx: &RenderCtx) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let room = js::max(140.0, options.max_width.unwrap_or(320.0) - 2.0 * pad);
    let title = fit_label(model.title.as_deref(), &m, size, room, style::BOLD);
    let mut names = actor_names(&model.tasks);
    names.retain(|name| !name.is_empty());
    let actors: HashMap<&str, usize> = names.iter().enumerate().map(|(i, name)| (*name, i)).collect();
    let colors = section_colors(model);
    let small_s = js::to_string(small);
    let truthy = |v: f64| v != 0.0 && !v.is_nan();
    let mut y = pad + if truthy(title.height) { title.height + 16.0 } else { 0.0 };
    let mut section: Option<&str> = None;
    let mut body = String::new();
    let mut width = title.width;
    let count = model.tasks.len();
    for (index, task) in model.tasks.iter().enumerate() {
        if section != Some(task.section.as_str()) {
            section = Some(task.section.as_str());
            let head = fit_label(section, &m, size, room - 16.0, style::BOLD);
            if truthy(head.height) {
                write!(
                    body,
                    "{}",
                    label_svg(
                        &head,
                        pad + 8.0 + head.width / 2.0,
                        y + head.height / 2.0,
                        " class=\"pele-cluster-label\" font-weight=\"var(--_hw)\"",
                        None,
                    )
                )
                .unwrap();
                width = js::max(width, head.width + 16.0);
                y += head.height + 8.0;
            }
        }
        let label = fit_label(Some(&task.task), &m, small, room - 72.0, 0);
        let people: Vec<Person> = task
            .people
            .iter()
            .filter(|name| !name.is_empty())
            .map(|name| Person { name, label: fit_label(Some(name), &m, small, room - 44.0, 0) })
            .collect();
        let text_h = js::max(label.height, 42.0);
        let mut h = 2.0 * PAD_Y + text_h;
        for person in &people {
            h += js::max(16.0, person.label.height) + 4.0;
        }
        let w = js::max(room, label.width + 72.0);
        let fill = section_fill(ctx, colors[index], Role::NodeFill, "var(--_s)");
        let on = text_on(ctx, &fill);
        svg_write!(
            body,
            "<g class=\"pele-node pele-journey-step\" data-id=\"",
            index,
            "\"><rect x=\"",
            num(pad),
            "\" y=\"",
            num(y),
            "\" width=\"",
            num(w),
            "\" height=\"",
            num(h),
            "\" rx=\"",
            RADIUS,
            "\" fill=\"",
            fill,
            "\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\"/>"
        )
        .unwrap();
        write!(
            body,
            "{}",
            label_svg(
                &label,
                pad + PAD_X + label.width / 2.0,
                y + PAD_Y + label.height / 2.0,
                &svg_format!(" class=\"pele-label\"", on),
                None,
            )
        )
        .unwrap();
        let cx = pad + w - 28.0;
        face(&mut body, ctx, cx, y + PAD_Y + FACE, Mood::of(task.score));
        if !task.score.is_nan() {
            svg_write!(
                body,
                "<text class=\"pele-journey-score\" x=\"",
                num(cx),
                "\" y=\"",
                num(y + PAD_Y + 2.0 * FACE + small),
                "\" font-size=\"",
                small_s,
                "\" text-anchor=\"middle\"",
                on,
                ">",
                num(js::min(5.0, js::max(1.0, task.score))),
                "</text>"
            )
            .unwrap();
        }
        let mut py = y + PAD_Y + text_h + 4.0;
        for person in &people {
            let ph = js::max(16.0, person.label.height);
            svg_write!(
                body,
                "<circle class=\"pele-actor\" cx=\"",
                num(pad + 16.0),
                "\" cy=\"",
                num(py + ph / 2.0),
                "\" r=\"",
                num(DOT),
                "\" fill=\"",
                ctx.series(actors.get(person.name).map_or(0, |&a| a as i64)),
                "\"/>"
            )
            .unwrap();
            write!(
                body,
                "{}",
                label_svg(
                    &person.label,
                    pad + 28.0 + person.label.width / 2.0,
                    py + ph / 2.0,
                    &svg_format!(" class=\"pele-journey-actor\" font-size=\"", small_s, "\"", on),
                    None,
                )
            )
            .unwrap();
            py += ph + 4.0;
        }
        body.push_str("</g>");
        y += h;
        if index + 1 < count {
            svg_write!(
                body,
                "<path class=\"pele-edge\" d=\"M",
                num(pad + w / 2.0),
                ",",
                num(y),
                "v16\" fill=\"none\" stroke=\"",
                ctx.paint(Role::Line, "var(--_l)"),
                "\"/>"
            )
            .unwrap();
        }
        y += 16.0;
        width = js::max(width, w);
    }
    let total_width = (width + 2.0 * pad).ceil();
    let height = (js::max(pad + title.height, y - if count > 0 { 16.0 } else { 0.0 }) + pad).ceil();
    let content = if title.lines.is_empty() {
        body
    } else {
        label_svg(&title, total_width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string()
            + &body
    };
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered { svg: svg_document("journey", total_width, height, size, ctx, acc, &content), width: total_width, height, links: Vec::new() }
}
