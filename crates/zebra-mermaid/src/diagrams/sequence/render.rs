//! Sequence rendering. Ported from Pele `src/diagrams/sequence/render.ts`.
//!
//! Colors follow Zebra's sequence semantics in resolved mode (`apply_theme` in Zebra's
//! renderer): participants take the node fill and outline, lifelines and message lines the
//! line color, arrowheads the arrow color, notes the warning pair, activations the selection
//! fill with an accent outline, and blocks the group outline with a group header tab.

use crate::svg::concat::{svg_format, svg_write};
use indexmap::IndexMap;

use super::actors::{actor_kind, actor_size, draw_actor, is_figure, ActorSize};
use super::line_type as lt;
use super::placement;
use super::{SeqActor, SeqContent, SeqMessage, SequenceModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::common::literal_label::break_render_label as break_label;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg, render_push_label_svg as push_label_svg};
use crate::svg::edges::{marker, push_marker, ARROW};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{class_names, resolve_style};
use crate::text::{layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::js;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Head {
    None,
    Arrow,
    Cross,
    Open,
    HalfSolid,
    HalfStick,
}

#[derive(Debug, Clone, Copy)]
struct LineStyle {
    dotted: bool,
    head: Head,
    // Where the heads are: at the receiver, at the sender only (the reverse arrows), or at both.
    at_start: bool,
    at_end: bool,
    bottom: bool,
}

const fn style_of(dotted: bool, head: Head, ends: u8, bottom: bool) -> Option<LineStyle> {
    Some(LineStyle { dotted, head, at_start: ends != 1, at_end: ends != 0, bottom })
}

/// The line style of each message kind that draws a line (Pele's `STYLES`).
fn line_style(kind: i32) -> Option<LineStyle> {
    let dotted = (51..=58).contains(&kind);
    match kind {
        lt::SOLID => style_of(false, Head::Arrow, 1, false),
        lt::DOTTED => style_of(true, Head::Arrow, 1, false),
        lt::SOLID_CROSS => style_of(false, Head::Cross, 1, false),
        lt::DOTTED_CROSS => style_of(true, Head::Cross, 1, false),
        lt::SOLID_OPEN => style_of(false, Head::None, 1, false),
        lt::DOTTED_OPEN => style_of(true, Head::None, 1, false),
        lt::SOLID_POINT => style_of(false, Head::Open, 1, false),
        lt::DOTTED_POINT => style_of(true, Head::Open, 1, false),
        lt::BIDIRECTIONAL_SOLID => style_of(false, Head::Arrow, 2, false),
        lt::BIDIRECTIONAL_DOTTED => style_of(true, Head::Arrow, 2, false),
        lt::SOLID_TOP | lt::SOLID_TOP_DOTTED => style_of(dotted, Head::HalfSolid, 1, false),
        lt::SOLID_BOTTOM | lt::SOLID_BOTTOM_DOTTED => style_of(dotted, Head::HalfSolid, 1, true),
        lt::STICK_TOP | lt::STICK_TOP_DOTTED => style_of(dotted, Head::HalfStick, 1, false),
        lt::STICK_BOTTOM | lt::STICK_BOTTOM_DOTTED => style_of(dotted, Head::HalfStick, 1, true),
        lt::SOLID_ARROW_TOP_REVERSE | lt::SOLID_ARROW_TOP_REVERSE_DOTTED => style_of(dotted, Head::HalfSolid, 0, false),
        lt::SOLID_ARROW_BOTTOM_REVERSE | lt::SOLID_ARROW_BOTTOM_REVERSE_DOTTED => style_of(dotted, Head::HalfSolid, 0, true),
        lt::STICK_ARROW_TOP_REVERSE | lt::STICK_ARROW_TOP_REVERSE_DOTTED => style_of(dotted, Head::HalfStick, 0, false),
        lt::STICK_ARROW_BOTTOM_REVERSE | lt::STICK_ARROW_BOTTOM_REVERSE_DOTTED => style_of(dotted, Head::HalfStick, 0, true),
        _ => None,
    }
}

fn frame_kind(kind: i32) -> Option<&'static str> {
    match kind {
        lt::LOOP_START => Some("loop"),
        lt::ALT_START => Some("alt"),
        lt::OPT_START => Some("opt"),
        lt::PAR_START | lt::PAR_OVER_START => Some("par"),
        lt::CRITICAL_START => Some("critical"),
        lt::BREAK_START => Some("break"),
        lt::RECT_START => Some("rect"),
        _ => None,
    }
}

const MSG_PAD: f64 = 12.0;
const SELF_W: f64 = 28.0;
const SELF_H: f64 = 22.0;
const ACT_W: f64 = 10.0;
const ACT_STEP: f64 = 4.0;
const ACT_MIN: f64 = 14.0;
const NOTE_GAP: f64 = 10.0;
const NOTE_OVER: f64 = 16.0;
const FRAME_PAD: f64 = 20.0;
const FRAME_NEST: f64 = 10.0;
const TAB_H: f64 = 20.0;
const FRAME_TEXT: f64 = 240.0;
const BADGE_H: f64 = 16.0;
const CENTRAL_R: f64 = 4.0;
const DASH: &str = " stroke-dasharray=\"4 4\"";

pub(super) struct Numbering {
    // The number each message line carries, whether or not it is shown. NaN for everything
    // else.
    pub(super) index: Vec<f64>,
    pub(super) shown: Vec<bool>,
    #[cfg(test)]
    pub(super) visible: bool,
}

fn is_line_message(kind: i32) -> bool {
    line_style(kind).is_some()
}

/// Counts message lines the way Mermaid's renderer does while it draws.
pub(super) fn number_messages(messages: &[SeqMessage], always: bool) -> Numbering {
    let mut index = vec![f64::NAN; messages.len()];
    let mut shown = vec![false; messages.len()];
    let mut current = 1.0;
    let mut step = 1.0;
    let mut visible = false;
    // `x || fallback` for a number that may be missing.
    let or = |v: Option<f64>, fallback: f64| match v {
        Some(v) if v != 0.0 && !v.is_nan() => v,
        _ => fallback,
    };
    for (i, m) in messages.iter().enumerate() {
        if m.kind == lt::AUTONUMBER {
            if let SeqContent::Numbering { start, step: s, visible: v } = &m.message {
                current = or(*start, current);
                step = or(*s, step);
                visible = *v;
            }
        } else if is_line_message(m.kind) {
            index[i] = current;
            shown[i] = visible || always;
            current = js::round((current + step) * 100.0) / 100.0;
        }
    }
    Numbering {
        index,
        shown,
        #[cfg(test)]
        visible,
    }
}

struct Column<'a> {
    id: &'a str,
    actor: &'a SeqActor,
    kind: &'a str,
    label: Label,
    size: ActorSize,
    x: f64,
    create_at: i64,
    destroy_at: i64,
    top: f64,
    life_start: f64,
    life_end: f64,
    activations: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum V {
    Line,
    SelfLoop,
    Note,
}

struct View {
    kind: V,
    a: usize,
    b: usize,
    label: Label,
    x: f64,
    w: f64,
}

struct Frame {
    kind: &'static str,
    x0: f64,
    x1: f64,
    top: f64,
    title: Label,
    tab_w: f64,
    fill: String,
    dividers: Vec<usize>,
    divider_y: Vec<f64>,
    divider_labels: Vec<Label>,
}

struct BoxRun {
    box_index: usize,
    first: usize,
    last: usize,
    title: Label,
    grow: f64,
}

/// A number from `sequence` configuration within 0..=2000, else the fallback.
fn option(conf: Option<&crate::config::ConfigMap>, key: &str, fallback: f64) -> f64 {
    match conf.and_then(|c| c.get(key)) {
        Some(ConfigValue::Number(v)) if *v >= 0.0 && *v <= 2000.0 => *v,
        _ => fallback,
    }
}

/// Adds a stroke to an element that Pele lets inherit its stroke, for resolved output.
fn own_stroke(element: String, color: Option<&str>) -> String {
    match color {
        Some(c) if element.ends_with("/>") => svg_format!(&element[..element.len() - 2], " stroke=\"", c, "\"/>"),
        _ => element,
    }
}

/// The colors of message lines and their heads.
struct Paint<'a> {
    line: &'a str,
    arrow: &'a str,
    /// The arrow color, written on heads that inherit their stroke from the line in Pele.
    head_stroke: Option<&'a str>,
}

fn half(x: f64, y: f64, dir: f64, up: f64, stick: bool, p: &Paint) -> String {
    let bx = num(x - dir * ARROW);
    if stick {
        return own_stroke(
            svg_format!("<path class=\"pele-marker\" d=\"M", num(x), ",", num(y), "L", bx, ",", num(y + up * 5.0), "\"/>"),
            p.head_stroke,
        );
    }
    svg_format!(
        "<path class=\"pele-marker\" d=\"M",
        num(x),
        ",",
        num(y),
        "L",
        bx,
        ",",
        num(y + up * 4.5),
        "V",
        num(y),
        "Z\" fill=\"",
        p.arrow,
        "\" stroke=\"none\"/>"
    )
}

/// Appends the head whose tip is at (x, y) and points along dir. `up` is the side a half head
/// sits on.
fn push_head(out: &mut String, kind: Head, x: f64, y: f64, dir: f64, up: f64, p: &Paint) {
    match kind {
        Head::Arrow => push_marker(out, "arrow_point", x, y, dir, 0.0, p.arrow),
        Head::Cross if p.head_stroke.is_none() => push_marker(out, "arrow_cross", x, y, dir, 0.0, p.arrow),
        Head::Cross => out.push_str(&own_stroke(marker("arrow_cross", x, y, dir, 0.0, p.arrow).to_string(), p.head_stroke)),
        Head::Open => {
            let element = svg_format!(
                "<path class=\"pele-marker\" d=\"M",
                num(x - dir * ARROW),
                ",",
                num(y - 4.5),
                "L",
                num(x),
                ",",
                num(y),
                "L",
                num(x - dir * ARROW),
                ",",
                num(y + 4.5),
                "\" stroke-linejoin=\"round\"/>"
            );
            out.push_str(&own_stroke(element, p.head_stroke));
        }
        Head::HalfSolid => out.push_str(&half(x, y, dir, up, false, p)),
        Head::HalfStick => out.push_str(&half(x, y, dir, up, true, p)),
        Head::None => {}
    }
}

fn trim(kind: Head) -> f64 {
    match kind {
        Head::Arrow => ARROW - 1.0,
        Head::Cross => 4.0,
        _ => 0.0,
    }
}

pub(crate) fn render(model: &SequenceModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| Ok(draw(model, config, ctx, tight)))
}

fn draw(db: &SequenceModel, config: &Config, ctx: &RenderCtx, tight: bool) -> Rendered {
    let options = ctx.options;
    let conf = config.section("sequence");
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.75);
    let m: Measure = ctx.measure();
    let icons = ctx.icons();
    let pad = ctx.padding();
    let flag = |key: &str| conf.and_then(|c| c.get(key));
    let mirror = !matches!(flag("mirrorActors"), Some(ConfigValue::Bool(false)));
    let right_angles = matches!(flag("rightAngles"), Some(ConfigValue::Bool(true)));
    let actor_margin = js::min(option(conf, "actorMargin", 40.0), if tight { 8.0 } else { f64::INFINITY });
    let min_w = js::min(option(conf, "width", 64.0), if tight { 48.0 } else { f64::INFINITY });
    let min_h = option(conf, "height", js::round(size * 1.5) + 20.0);
    let wrap_w = js::min(
        option(conf, "width", 160.0),
        if tight {
            js::max(32.0, (options.max_width.unwrap_or(320.0) - 2.0 * pad) / js::max(1.0, db.actors.len() as f64) - actor_margin - 24.0)
        } else {
            f64::INFINITY
        },
    );
    // Mermaid's default of 35 corresponds to Pele's default gap. Other values move it by the
    // difference.
    let msg_gap = js::max(4.0, 14.0 + option(conf, "messageMargin", 35.0) - 35.0);
    let note_pad = js::min(option(conf, "noteMargin", 10.0), if tight { 6.0 } else { f64::INFINITY });
    let box_pad = js::min(option(conf, "boxMargin", 10.0), if tight { 6.0 } else { f64::INFINITY });
    let align = match flag("messageAlign") {
        Some(ConfigValue::String(s)) if s == "left" => -1,
        Some(ConfigValue::String(s)) if s == "right" => 1,
        _ => 0,
    };

    let msg_pad = if tight { 6.0 } else { MSG_PAD };
    let note_over = if tight { 8.0 } else { NOTE_OVER };
    let frame_pad = if tight { 12.0 } else { FRAME_PAD };
    let frame_nest = if tight { 6.0 } else { FRAME_NEST };
    let messages = &db.messages;
    let count = messages.len();
    let numbering = number_messages(messages, matches!(flag("showSequenceNumbers"), Some(ConfigValue::Bool(true))));
    let text = |raw: Option<&str>, font_size: f64, wrap: bool, width: f64, base: u8| -> Label {
        let label = layout_label(raw, false, &m, font_size, if tight || wrap { width } else { f64::INFINITY }, base);
        if tight {
            break_label(&label, &m, js::max(120.0, width))
        } else {
            label
        }
    };

    // Colors.
    let resolved = ctx.resolved();
    let line_color = ctx.paint(Role::Line, "var(--_l)");
    let paint = Paint {
        line: line_color,
        arrow: ctx.paint(Role::Arrow, "var(--_l)"),
        head_stroke: if resolved { Some(ctx.paint(Role::Arrow, "var(--_l)")) } else { None },
    };
    let patch = ctx.paint(Role::Background, "var(--_bg)");
    let badge_fill = ctx.paint(Role::Accent, "var(--_l)");
    let badge_text = ctx.theming.text_on(ctx.theming.resolved.color(Role::Accent), "var(--_bg)");
    let frame_text = ctx.paint(Role::SecondaryText, "var(--_m)");
    let frame_kind_text = ctx.paint(Role::Text, "var(--_m)");
    let frame_label_attrs = svg_format!(" class=\"pele-frame-label\" fill=\"", frame_text, "\" stroke=\"none\"");

    let used: Option<Vec<Option<&str>>> = if matches!(flag("hideUnusedParticipants"), Some(ConfigValue::Bool(true))) {
        let mut set: Vec<Option<&str>> = Vec::new();
        for msg in messages {
            set.push(msg.from.as_deref());
            set.push(msg.to.as_deref());
        }
        Some(set)
    } else {
        None
    };
    let used: Option<std::collections::HashSet<Option<&str>>> = used.map(|v| v.into_iter().collect());

    let mut next_line = vec![-1i64; count + 1];
    for i in (0..count).rev() {
        next_line[i] = if is_line_message(messages[i].kind) { i as i64 } else { next_line[i + 1] };
    }

    let mut cols: Vec<Column> = Vec::new();
    let mut col_of: IndexMap<&str, usize> = IndexMap::new();
    for (id, actor) in &db.actors {
        if let Some(used) = &used {
            if !used.contains(&Some(id.as_str())) {
                continue;
            }
        }
        let kind = actor_kind(&actor.kind);
        let label = text(Some(&actor.description), size, actor.wrap, wrap_w, 0);
        let created = db.created_actors.get(id);
        let destroyed = db.destroyed_actors.get(id);
        let mut create_at = match created {
            None => -1,
            Some(&c) => next_line[c.min(count)],
        };
        if create_at >= 0 && messages[create_at as usize].to.as_deref() != Some(id.as_str()) {
            create_at = -1;
        }
        let mut destroy_at = match destroyed {
            None => -1,
            Some(&d) => next_line[d.min(count)],
        };
        if destroy_at >= 0 {
            let at = &messages[destroy_at as usize];
            if at.to.as_deref() != Some(id.as_str()) && at.from.as_deref() != Some(id.as_str()) {
                destroy_at = -1;
            }
        }
        col_of.insert(id.as_str(), cols.len());
        let size = actor_size(kind, &label, min_w, min_h, if tight { Some(8.0) } else { None });
        cols.push(Column {
            id,
            actor,
            kind,
            label,
            size,
            x: 0.0,
            create_at,
            destroy_at,
            top: 0.0,
            life_start: 0.0,
            life_end: -1.0,
            activations: Vec::new(),
        });
    }
    let n_cols = cols.len() as i64;
    let last = n_cols - 1;

    // Widths between lifelines. Every message and note says how far apart the lifelines it
    // touches must be.
    let mut needs: IndexMap<i64, f64> = IndexMap::new();
    let mut need = |lo: i64, hi: i64, width: f64| {
        if lo < 0 || hi > last {
            return;
        }
        let key = lo * n_cols + hi;
        if needs.get(&key).copied().unwrap_or(0.0) < width {
            needs.insert(key, width);
        }
    };
    let badge_width = |i: usize| -> f64 {
        if numbering.shown[i] {
            js::max(BADGE_H, m.width(&js::number_text(numbering.index[i]), tiny, 0) + 8.0)
        } else {
            0.0
        }
    };

    let mut views: Vec<Option<View>> = (0..count).map(|_| None).collect();
    for i in 0..count {
        let msg = &messages[i];
        let a = msg.from.as_deref().and_then(|f| col_of.get(f).copied());
        let b = msg.to.as_deref().and_then(|t| col_of.get(t).copied());
        let (Some(a), Some(b), Some(body)) = (a, b, msg.message.text()) else { continue };
        let (ai, bi) = (a as i64, b as i64);
        if msg.kind == lt::NOTE {
            let label = text(Some(body), small, msg.wrap, wrap_w, 0);
            let w = label.width + 2.0 * note_pad;
            if msg.placement == Some(placement::LEFT_OF) {
                need(ai - 1, ai, w + 2.0 * NOTE_GAP + ACT_W);
            } else if msg.placement == Some(placement::RIGHT_OF) {
                need(ai, ai + 1, w + 2.0 * NOTE_GAP + ACT_W);
            } else if a == b {
                need(ai - 1, ai, w / 2.0 + NOTE_GAP);
                need(ai, ai + 1, w / 2.0 + NOTE_GAP);
            } else {
                need(ai.min(bi), ai.max(bi), w - 2.0 * note_over);
            }
            views[i] = Some(View { kind: V::Note, a, b, label, x: 0.0, w });
        } else if is_line_message(msg.kind) {
            let label = text(Some(body), small, msg.wrap, wrap_w, 0);
            let badge = badge_width(i);
            if a == b {
                need(ai, ai + 1, SELF_W + 8.0 + label.width + 16.0 + ACT_W + badge);
                views[i] = Some(View { kind: V::SelfLoop, a, b, label, x: 0.0, w: 0.0 });
            } else {
                let created = if cols[b].create_at == i as i64 { cols[b].size.w / 2.0 } else { 0.0 };
                let central = if msg.central_connection.is_some_and(|c| c != 0) { 4.0 * CENTRAL_R } else { 0.0 };
                let shown = if tight { js::min(label.width, js::max(16.0, wrap_w - 16.0)) } else { label.width };
                need(ai.min(bi), ai.max(bi), shown + 2.0 * msg_pad + ACT_W + badge + created + central);
                views[i] = Some(View { kind: V::Line, a, b, label, x: 0.0, w: 0.0 });
            }
        }
    }

    let mut runs: Vec<BoxRun> = Vec::new();
    for i in 0..cols.len() {
        let Some(box_index) = cols[i].actor.box_index else { continue };
        if let Some(run) = runs.last_mut() {
            if run.box_index == box_index && run.last + 1 == i {
                run.last = i;
                continue;
            }
        }
        let b = &db.boxes[box_index];
        runs.push(BoxRun { box_index, first: i, last: i, title: text(b.name.as_deref(), small, b.wrap, wrap_w, 0), grow: 0.0 });
    }

    let n_gaps = if last > 0 { last as usize } else { 0 };
    let mut gaps = vec![0.0; n_gaps];
    for i in 0..n_gaps {
        let left = &cols[i];
        let right = &cols[i + 1];
        let boxed = if left.actor.box_index != right.actor.box_index {
            (if left.actor.box_index.is_some() { box_pad } else { 0.0 }) + if right.actor.box_index.is_some() { box_pad } else { 0.0 }
        } else {
            0.0
        };
        let key = i as i64 * n_cols + i as i64 + 1;
        gaps[i] = js::max(left.size.w / 2.0 + right.size.w / 2.0 + actor_margin + boxed, needs.get(&key).copied().unwrap_or(0.0));
    }
    // A message across several lifelines widens each gap it crosses by an equal share of what
    // is missing.
    let mut spans: Vec<(i64, i64, f64)> = Vec::new();
    for (&key, &width) in &needs {
        let lo = key / n_cols;
        let hi = key % n_cols;
        if hi - lo > 1 {
            spans.push((lo, hi, width));
        }
    }
    spans.sort_by(|p, q| (p.1 - p.0).cmp(&(q.1 - q.0)).then(p.0.cmp(&q.0)));
    for &(lo, hi, width) in &spans {
        let mut total = 0.0;
        for g in &gaps[lo as usize..hi as usize] {
            total += g;
        }
        if total >= width {
            continue;
        }
        let share = (width - total) / (hi - lo) as f64;
        for g in &mut gaps[lo as usize..hi as usize] {
            *g += share;
        }
    }
    let place = |cols: &mut Vec<Column>, gaps: &[f64]| {
        for i in 1..cols.len() {
            cols[i].x = cols[i - 1].x + gaps[i - 1];
        }
    };
    place(&mut cols, &gaps);
    for run in &mut runs {
        let inner = cols[run.last].x + cols[run.last].size.w / 2.0 - (cols[run.first].x - cols[run.first].size.w / 2.0);
        run.grow = js::max(0.0, (run.title.width - inner) / 2.0);
        if run.grow == 0.0 {
            continue;
        }
        if run.first > 0 {
            gaps[run.first - 1] += run.grow;
        }
        if (run.last as i64) < last {
            gaps[run.last] += run.grow;
        }
    }
    place(&mut cols, &gaps);

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let extend = |min_x: &mut f64, max_x: &mut f64, x0: f64, x1: f64| {
        if x0 < *min_x {
            *min_x = x0;
        }
        if x1 > *max_x {
            *max_x = x1;
        }
    };
    for col in &cols {
        extend(&mut min_x, &mut max_x, col.x - col.size.w / 2.0, col.x + col.size.w / 2.0);
    }
    for run in &runs {
        extend(
            &mut min_x,
            &mut max_x,
            cols[run.first].x - cols[run.first].size.w / 2.0 - box_pad - run.grow,
            cols[run.last].x + cols[run.last].size.w / 2.0 + box_pad + run.grow,
        );
    }

    // Horizontal extents of every block, from what it holds.
    let mut frames: Vec<Frame> = Vec::new();
    let mut frame_at: Vec<Option<usize>> = vec![None; count];
    let mut open: Vec<usize> = Vec::new();
    let span_all = || -> (f64, f64) {
        if last < 0 {
            (0.0, 120.0)
        } else {
            (cols[0].x - frame_pad, cols[last as usize].x + frame_pad)
        }
    };
    for i in 0..count {
        let msg = &messages[i];
        let kind = msg.kind;
        if let Some(fk) = frame_kind(kind) {
            let fill = match (&msg.message, fk) {
                (SeqContent::Text(t), "rect") => t.clone(),
                _ => String::new(),
            };
            frames.push(Frame {
                kind: fk,
                x0: f64::INFINITY,
                x1: f64::NEG_INFINITY,
                top: 0.0,
                title: Label::empty(),
                tab_w: 0.0,
                fill,
                dividers: vec![i],
                divider_y: Vec::new(),
                divider_labels: Vec::new(),
            });
            frame_at[i] = Some(frames.len() - 1);
            open.push(frames.len() - 1);
        } else if kind == lt::ALT_ELSE || kind == lt::PAR_AND || kind == lt::CRITICAL_OPTION {
            if let Some(&f) = open.last() {
                frames[f].dividers.push(i);
            }
        } else if matches!(kind, lt::LOOP_END | lt::ALT_END | lt::OPT_END | lt::PAR_END | lt::CRITICAL_END | lt::BREAK_END | lt::RECT_END) {
            let Some(f) = open.pop() else { continue };
            frame_at[i] = Some(f);
            if frames[f].x0 > frames[f].x1 {
                let (x0, x1) = span_all();
                frames[f].x0 = x0;
                frames[f].x1 = x1;
            }
            if frames[f].kind != "rect" {
                let frame = &mut frames[f];
                frame.tab_w = m.width(frame.kind, tiny, style::BOLD) + 14.0;
                // A block grows to the right to give its texts up to FRAME_TEXT of width before
                // they wrap.
                for k in 0..frame.dividers.len() {
                    let raw = messages[frame.dividers[k]].message.text().unwrap_or("");
                    let tab = if k == 0 { frame.tab_w + 8.0 } else { 0.0 };
                    let room = frame.x1 - frame.x0 - 16.0 - tab;
                    let label = if !raw.is_empty() {
                        let wrap_to = js::max(room, if tight { js::min(FRAME_TEXT, wrap_w) } else { FRAME_TEXT });
                        text(Some(&format!("[{}]", raw)), small, true, wrap_to, 0)
                    } else {
                        Label::empty()
                    };
                    if label.width > room {
                        frame.x1 += label.width - room;
                    }
                    if k == 0 {
                        frame.title = label;
                    } else {
                        frame.divider_labels.push(label);
                    }
                }
            }
            let (fx0, fx1) = (frames[f].x0, frames[f].x1);
            extend(&mut min_x, &mut max_x, fx0, fx1);
            if let Some(&parent) = open.last() {
                frames[parent].x0 = js::min(frames[parent].x0, fx0 - frame_nest);
                frames[parent].x1 = js::max(frames[parent].x1, fx1 + frame_nest);
            }
        } else if let Some(view) = views[i].as_mut() {
            let a = &cols[view.a];
            let b = &cols[view.b];
            let x0: f64;
            let x1: f64;
            if view.kind == V::Note {
                if msg.placement == Some(placement::LEFT_OF) {
                    view.x = a.x - NOTE_GAP - view.w;
                } else if msg.placement == Some(placement::RIGHT_OF) {
                    view.x = a.x + NOTE_GAP;
                } else if view.a == view.b {
                    view.x = a.x - view.w / 2.0;
                } else {
                    let lo = js::min(a.x, b.x) - note_over;
                    let hi = js::max(a.x, b.x) + note_over;
                    if tight || msg.wrap {
                        view.label = text(msg.message.text(), small, true, js::max(wrap_w, hi - lo - 2.0 * note_pad), 0);
                    }
                    view.w = js::max(hi - lo, view.label.width + 2.0 * note_pad);
                    view.x = (lo + hi - view.w) / 2.0;
                }
                x0 = view.x;
                x1 = view.x + view.w;
            } else if view.kind == V::SelfLoop {
                x0 = a.x;
                x1 = a.x + ACT_W + SELF_W + 8.0 + view.label.width + badge_width(i);
            } else {
                let mut lo = js::min(a.x, b.x);
                let mut hi = js::max(a.x, b.x);
                if tight || msg.wrap {
                    view.label = text(msg.message.text(), small, true, js::max(wrap_w, hi - lo - 2.0 * msg_pad - ACT_W), 0);
                }
                if b.create_at == i as i64 {
                    if b.x > a.x {
                        hi += b.size.w / 2.0;
                    } else {
                        lo -= b.size.w / 2.0;
                    }
                }
                x0 = lo;
                x1 = hi;
            }
            extend(&mut min_x, &mut max_x, x0, x1);
            if let Some(&f) = open.last() {
                frames[f].x0 = js::min(frames[f].x0, x0 - frame_pad);
                frames[f].x1 = js::max(frames[f].x1, x1 + frame_pad);
            }
        }
    }

    // Vertical placement, top to bottom, drawing as it goes.
    let mut box_title_h: f64 = 0.0;
    for run in &runs {
        box_title_h = js::max(box_title_h, run.title.height);
    }
    let box_top = if !runs.is_empty() { box_pad + if box_title_h > 0.0 { box_title_h + 4.0 } else { 0.0 } } else { 0.0 };
    let mut row_h: f64 = 0.0;
    for col in &cols {
        if col.create_at < 0 {
            row_h = js::max(row_h, col.size.h);
        }
    }
    let head_bottom = box_top + row_h;
    for col in &mut cols {
        col.top = head_bottom - col.size.h;
        col.life_start = head_bottom;
    }

    let xs: Vec<f64> = cols.iter().map(|c| c.x).collect();
    // Whether a lifeline runs between two x positions, where text would be struck through.
    let crosses = |x0: f64, x1: f64| -> bool {
        let mut lo = 0usize;
        let mut hi = xs.len();
        while lo < hi {
            let mid = (lo + hi) >> 1;
            if xs[mid] <= x0 {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo < xs.len() && xs[lo] < x1
    };
    // A label, on a patch of the background where a lifeline would strike through it.
    let push_backed = |out: &mut String, label: &Label, cx: f64, cy: f64, attrs: &str| {
        if label.width == 0.0 {
            return;
        }
        let l = cx - label.width / 2.0;
        if crosses(l, l + label.width) {
            svg_write!(
                out,
                "<rect x=\"",
                num(l - 3.0),
                "\" y=\"",
                num(cy - label.height / 2.0 + 2.0),
                "\" width=\"",
                num(label.width + 6.0),
                "\" height=\"",
                num(label.height - 4.0),
                "\" rx=\"3\" fill=\"",
                patch,
                "\" stroke=\"none\"/>"
            )
            .unwrap();
        }
        push_label_svg(out, label, cx, cy, attrs, icons);
    };
    let note_label_attrs = svg_format!(" class=\"pele-label\" stroke=\"none\" fill=\"", ctx.paint(Role::Text, "var(--_fg)"), "\"");
    let left_edge = |col: &Column, extra: usize| -> f64 {
        if col.activations.len() + extra > 0 {
            col.x - ACT_W / 2.0
        } else {
            col.x
        }
    };
    let right_edge = |col: &Column, extra: usize| -> f64 {
        let depth = col.activations.len() + extra;
        if depth > 0 {
            col.x + ACT_W / 2.0 + (depth - 1) as f64 * ACT_STEP
        } else {
            col.x
        }
    };

    // Shaded blocks and activations close from the inside out. They are written in reverse,
    // so that the inner ones end up on top.
    let mut rects_out: Vec<String> = Vec::new();
    let mut activations_out: Vec<String> = Vec::new();
    let mut frames_out = String::new();
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut notes_out = String::new();
    let mut actors_out = String::new();
    let mut crosses_out = String::new();
    let end_activation = |col: &mut Column, end: f64, min_x: &mut f64, max_x: &mut f64, out: &mut Vec<String>| -> f64 {
        let start = col.activations.pop().unwrap_or(f64::NAN);
        let bottom = js::max(end, start + ACT_MIN);
        let x = col.x - ACT_W / 2.0 + col.activations.len() as f64 * ACT_STEP;
        extend(min_x, max_x, x, x + ACT_W);
        out.push(svg_format!(
            "<rect class=\"pele-activation\" data-id=\"",
            esc_text(col.id),
            "\" x=\"",
            num(x),
            "\" y=\"",
            num(start),
            "\" width=\"",
            js::number_text(ACT_W),
            "\" height=\"",
            num(bottom - start),
            "\" rx=\"2\"/>"
        ));
        bottom
    };
    // The lifeline of a destroyed participant ends in a cross a little below the message that
    // ends it.
    let destroy = |col: &mut Column, line_y: f64, out: &mut String| -> f64 {
        let cx = col.x;
        let cy = line_y + 16.0;
        col.life_end = cy;
        svg_write!(
            out,
            "<path class=\"pele-destroy\" data-id=\"",
            esc_text(col.id),
            "\" d=\"M",
            num(cx - 6.0),
            ",",
            num(cy - 6.0),
            "L",
            num(cx + 6.0),
            ",",
            num(cy + 6.0),
            "M",
            num(cx + 6.0),
            ",",
            num(cy - 6.0),
            "L",
            num(cx - 6.0),
            ",",
            num(cy + 6.0),
            "\"/>"
        )
        .unwrap();
        cy + 6.0
    };
    let actor_svg = |col: &Column, top: f64, footer: bool| -> String {
        let cls = match col.actor.properties.get("class") {
            Some(ConfigValue::String(s)) => class_names(s),
            _ => String::new(),
        };
        svg_format!(
            "<g class=\"pele-node pele-actor pele-actor-",
            col.kind,
            if footer { " pele-actor-footer" } else { "" },
            cls,
            "\" data-id=\"",
            esc_text(col.id),
            "\">",
            draw_actor(col.kind, col.x, top, col.size, &col.label, icons, ctx),
            "</g>"
        )
    };
    let tiny_s = js::number_text(tiny);
    let badge = |i: usize, cx: f64, cy: f64, w: f64| -> String {
        svg_format!(
            "<g class=\"pele-sequence-number\"><rect x=\"",
            num(cx - w / 2.0),
            "\" y=\"",
            num(cy - BADGE_H / 2.0),
            "\" width=\"",
            num(w),
            "\" height=\"",
            js::number_text(BADGE_H),
            "\" rx=\"",
            js::number_text(BADGE_H / 2.0),
            "\" fill=\"",
            badge_fill,
            "\" stroke=\"none\"/><text x=\"",
            num(cx),
            "\" y=\"",
            num(cy + tiny * 0.35),
            "\" text-anchor=\"middle\" font-size=\"",
            tiny_s,
            "\" fill=\"",
            badge_text,
            "\" stroke=\"none\">",
            esc(&js::number_text(numbering.index[i])),
            "</text></g>"
        )
    };

    let mut y = head_bottom + 6.0;
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..count {
        let msg = &messages[i];
        let kind = msg.kind;
        let frame = frame_at[i];

        if kind == lt::ACTIVE_START {
            if let Some(c) = msg.from.as_deref().and_then(|f| col_of.get(f).copied()) {
                cols[c].activations.push(y);
            }
        } else if kind == lt::ACTIVE_END {
            if let Some(c) = msg.from.as_deref().and_then(|f| col_of.get(f).copied()) {
                if !cols[c].activations.is_empty() {
                    let bottom = end_activation(&mut cols[c], y, &mut min_x, &mut max_x, &mut activations_out);
                    y = js::max(y, bottom);
                }
            }
        } else if let Some(f) = frame.filter(|&f| frames[f].dividers[0] == i) {
            stack.push(f);
            let frame = &mut frames[f];
            if frame.kind == "rect" {
                frame.top = y + 8.0;
                y = frame.top;
            } else {
                frame.top = y + 12.0;
                y = frame.top + js::max(TAB_H, frame.title.height + 2.0);
            }
        } else if let Some(f) = frame {
            stack.pop();
            let frame = &frames[f];
            let bottom = y + 12.0;
            let x = num(frame.x0);
            let w = num(frame.x1 - frame.x0);
            if frame.kind == "rect" {
                let fill = if !frame.fill.is_empty() { resolve_style(&[format!("fill:{}", frame.fill)]).shape } else { String::new() };
                rects_out.push(svg_format!(
                    "<rect class=\"pele-rect\" x=\"",
                    x,
                    "\" y=\"",
                    num(frame.top),
                    "\" width=\"",
                    w,
                    "\" height=\"",
                    num(bottom - frame.top),
                    "\"",
                    crate::svg::theme::rounded(&svg_format!(if fill.is_empty() { " fill-opacity=\"0.5\"" } else { "" }, fill)),
                    "/>"
                ));
            } else {
                let mut inner = svg_format!(
                    "<rect x=\"",
                    x,
                    "\" y=\"",
                    num(frame.top),
                    "\" width=\"",
                    w,
                    "\" height=\"",
                    num(bottom - frame.top),
                    "\"",
                    crate::svg::theme::rounded(&svg_format!(" fill=\"none\"")),
                    "/>"
                );
                for k in 0..frame.divider_y.len() {
                    let dy = frame.divider_y[k];
                    let label = &frame.divider_labels[k];
                    svg_write!(inner, "<path d=\"M", x, ",", num(dy), "H", num(frame.x1), "\"", DASH, "/>").unwrap();
                    push_backed(&mut inner, label, frame.x0 + 8.0 + label.width / 2.0, dy + 2.0 + label.height / 2.0, &frame_label_attrs);
                }
                svg_write!(
                    inner,
                    "<rect x=\"",
                    x,
                    "\" y=\"",
                    num(frame.top),
                    "\" width=\"",
                    num(frame.tab_w),
                    "\" height=\"",
                    js::number_text(TAB_H),
                    "\"",
                    crate::svg::theme::rounded(&svg_format!(" fill=\"", ctx.paint(Role::GroupHeader, "var(--_s)"), "\"")),
                    "/>"
                )
                .unwrap();
                svg_write!(
                    inner,
                    "<text class=\"pele-frame-kind\" x=\"",
                    num(frame.x0 + frame.tab_w / 2.0),
                    "\" y=\"",
                    num(frame.top + TAB_H / 2.0 + tiny * 0.35),
                    "\" text-anchor=\"middle\" font-size=\"",
                    tiny_s,
                    "\" font-weight=\"var(--_hw)\" fill=\"",
                    frame_kind_text,
                    "\" stroke=\"none\">",
                    frame.kind,
                    "</text>"
                )
                .unwrap();
                push_backed(
                    &mut inner,
                    &frame.title,
                    frame.x0 + frame.tab_w + 8.0 + frame.title.width / 2.0,
                    frame.top + js::max(TAB_H, frame.title.height) / 2.0,
                    &frame_label_attrs,
                );
                svg_write!(frames_out, "<g class=\"pele-frame pele-frame-", frame.kind, "\">", inner, "</g>").unwrap();
            }
            y = bottom;
        } else if kind == lt::ALT_ELSE || kind == lt::PAR_AND || kind == lt::CRITICAL_OPTION {
            if let Some(&f) = stack.last() {
                let current = &mut frames[f];
                let height = current.divider_labels.get(current.divider_y.len()).map_or(0.0, |l| l.height);
                current.divider_y.push(y + 12.0);
                y += 12.0 + if height > 0.0 { height } else { 0.0 };
            }
        } else if let Some(view) = views[i].as_ref().filter(|v| v.kind == V::Note) {
            let a = &cols[view.a];
            let h = view.label.height + 2.0 * js::max(4.0, note_pad - 4.0);
            let top = y + 10.0;
            let mut x = view.x;
            if msg.placement == Some(placement::RIGHT_OF) {
                x += right_edge(a, 0) - a.x;
            } else if msg.placement == Some(placement::LEFT_OF) {
                x += left_edge(a, 0) - a.x;
            }
            extend(&mut min_x, &mut max_x, x, x + view.w);
            svg_write!(
                notes_out,
                "<g class=\"pele-note\" data-id=\"i",
                msg.id,
                "\"><rect x=\"",
                num(x),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(view.w),
                "\" height=\"",
                num(h),
                "\"",
                crate::svg::theme::ROUNDED,
                "/>",
                label_svg(&view.label, x + view.w / 2.0, top + h / 2.0, &note_label_attrs, icons),
                "</g>"
            )
            .unwrap();
            y = top + h;
        } else if let Some(view) = views[i].as_ref() {
            let Some(st) = line_style(kind) else { continue };
            let (ai, bi) = (view.a, view.b);
            let label = &view.label;
            let bw = badge_width(i);
            // The activation a `+` starts belongs to this message, so the arrow stops at its
            // edge.
            let mut starting = 0usize;
            for next in &messages[i + 1..] {
                let t = next.kind;
                if t == lt::ACTIVE_START {
                    if next.from == msg.to {
                        starting += 1;
                    }
                } else if t != lt::ACTIVE_END && t != lt::CENTRAL_CONNECTION && t != lt::CENTRAL_CONNECTION_REVERSE {
                    break;
                }
            }
            let up = if st.bottom { 1.0 } else { -1.0 };
            let attrs = if st.dotted { DASH } else { "" };
            let cc = msg.central_connection.unwrap_or(0);
            let circle_to = cc == lt::CENTRAL_CONNECTION || cc == lt::CENTRAL_CONNECTION_DUAL;
            let circle_from = cc == lt::CENTRAL_CONNECTION_REVERSE || cc == lt::CENTRAL_CONNECTION_DUAL;
            let circle = |cx: f64, cy: f64| -> String {
                svg_format!(
                    "<circle class=\"pele-central\" cx=\"",
                    num(cx),
                    "\" cy=\"",
                    num(cy),
                    "\" r=\"",
                    js::number_text(CENTRAL_R),
                    "\" fill=\"",
                    paint.arrow,
                    "\" stroke=\"none\"/>"
                )
            };
            let tail_at_end = st.at_start && !st.at_end;
            let head_trim = trim(st.head);
            let mut inner = String::new();
            let line_y: f64;

            if view.kind == V::SelfLoop {
                let a = &cols[ai];
                let top = y + msg_gap + js::max(0.0, (label.height - SELF_H) / 2.0);
                let bottom = top + SELF_H;
                let mut x0 = if circle_from { a.x + CENTRAL_R } else { right_edge(a, 0) };
                let mut x1 = if circle_to { a.x + CENTRAL_R } else { right_edge(a, starting) };
                let out = js::max(x0, x1) + SELF_W + bw / 2.0;
                if circle_from {
                    inner.push_str(&circle(a.x, top));
                }
                if circle_to {
                    inner.push_str(&circle(a.x, bottom));
                }
                if bw > 0.0 {
                    inner.push_str(&badge(i, if tail_at_end { x1 } else { x0 }, if tail_at_end { bottom } else { top }, bw));
                    if tail_at_end {
                        x1 += bw / 2.0;
                    } else {
                        x0 += bw / 2.0;
                    }
                }
                let from = num(x0 + if st.at_start { head_trim } else { 0.0 });
                let to = num(x1 + if st.at_end { head_trim } else { 0.0 });
                svg_write!(edges_out, "<g class=\"pele-edge pele-message\" data-id=\"i", msg.id, "\"><path d=\"").unwrap();
                if right_angles {
                    svg_write!(edges_out, "M", from, ",", num(top), "H", num(out), "V", num(bottom), "H", to).unwrap();
                } else {
                    svg_write!(
                        edges_out,
                        "M",
                        from,
                        ",",
                        num(top),
                        "H",
                        num(out - 6.0),
                        "A6,6 0 0 1 ",
                        num(out),
                        ",",
                        num(top + 6.0),
                        "V",
                        num(bottom - 6.0),
                        "A6,6 0 0 1 ",
                        num(out - 6.0),
                        ",",
                        num(bottom),
                        "H",
                        to
                    )
                    .unwrap();
                }
                edges_out.push('"');
                edges_out.push_str(attrs);
                edges_out.push_str("/>");
                // The loop leaves to the right and comes back heading left, which turns a half
                // head over.
                if st.at_start {
                    push_head(&mut edges_out, st.head, x0, top, -1.0, up, &paint);
                }
                if st.at_end {
                    push_head(&mut edges_out, st.head, x1, bottom, -1.0, -up, &paint);
                }
                edges_out.push_str(&inner);
                edges_out.push_str("</g>");
                let ax = a.x;
                extend(&mut min_x, &mut max_x, ax, out + 8.0 + label.width);
                if label.width > 0.0 {
                    svg_write!(
                        labels_out,
                        "<g class=\"pele-edge-label\" data-id=\"i",
                        msg.id,
                        "\">",
                        label_svg(label, out + 8.0 + label.width / 2.0, (top + bottom) / 2.0, "", icons),
                        "</g>"
                    )
                    .unwrap();
                }
                line_y = js::max(bottom, (top + bottom) / 2.0 + label.height / 2.0);
            } else {
                let (a, b) = (&cols[ai], &cols[bi]);
                let dir = if b.x > a.x { 1.0 } else { -1.0 };
                let creating = b.create_at == i as i64;
                line_y = y
                    + msg_gap
                    + js::max(
                        if label.height > 0.0 { label.height + 2.0 } else { 6.0 },
                        if creating { b.size.h / 2.0 - msg_gap + 6.0 } else { 0.0 },
                    );
                let mut x0 = if dir > 0.0 { right_edge(a, 0) } else { left_edge(a, 0) };
                let mut x1 = if dir > 0.0 { left_edge(b, starting) } else { right_edge(b, starting) };
                if creating {
                    x1 = b.x - dir * if is_figure(b.kind) { 18.0 } else { b.size.w / 2.0 + 1.0 };
                }
                if circle_from {
                    inner.push_str(&circle(a.x, line_y));
                    x0 = a.x + dir * CENTRAL_R;
                }
                if circle_to {
                    inner.push_str(&circle(b.x, line_y));
                    x1 = b.x - dir * CENTRAL_R;
                }
                if bw > 0.0 {
                    // The number sits on the lifeline at the tail of the arrow, past a
                    // connection circle if there is one.
                    if tail_at_end {
                        let cx = if circle_to { x1 - dir * (bw / 2.0 + 2.0) } else { x1 };
                        inner.push_str(&badge(i, cx, line_y, bw));
                        x1 = cx - (dir * bw) / 2.0;
                    } else {
                        let cx = if circle_from { x0 + dir * (bw / 2.0 + 2.0) } else { x0 };
                        inner.push_str(&badge(i, cx, line_y, bw));
                        x0 = cx + (dir * bw) / 2.0;
                    }
                }
                let from = x0 + if st.at_start { dir * head_trim } else { 0.0 };
                let to = x1 - if st.at_end { dir * head_trim } else { 0.0 };
                svg_write!(
                    edges_out,
                    "<g class=\"pele-edge pele-message\" data-id=\"i",
                    msg.id,
                    "\"><path d=\"M",
                    num(from),
                    ",",
                    num(line_y),
                    "H",
                    num(to),
                    "\"",
                    attrs,
                    "/>"
                )
                .unwrap();
                if st.at_start {
                    push_head(&mut edges_out, st.head, x0, line_y, -dir, up * dir, &paint);
                }
                if st.at_end {
                    push_head(&mut edges_out, st.head, x1, line_y, dir, up * dir, &paint);
                }
                edges_out.push_str(&inner);
                edges_out.push_str("</g>");
                if label.width > 0.0 {
                    let l = js::min(x0, x1);
                    let r = js::max(x0, x1);
                    let cx = if align < 0 {
                        l + msg_pad + label.width / 2.0
                    } else if align > 0 {
                        r - msg_pad - label.width / 2.0
                    } else {
                        (l + r) / 2.0
                    };
                    extend(&mut min_x, &mut max_x, cx - label.width / 2.0, cx + label.width / 2.0);
                    svg_write!(labels_out, "<g class=\"pele-edge-label\" data-id=\"i", msg.id, "\">").unwrap();
                    push_backed(&mut labels_out, label, cx, line_y - 4.0 - label.height / 2.0, "");
                    labels_out.push_str("</g>");
                }
                if creating {
                    let b = &mut cols[bi];
                    b.top = line_y - b.size.h / 2.0;
                    b.life_start = b.top + b.size.h;
                    y = line_y + b.size.h / 2.0 - 6.0;
                }
            }
            y = js::max(y, line_y);
            if cols[ai].destroy_at == i as i64 {
                let end = destroy(&mut cols[ai], line_y, &mut crosses_out);
                y = js::max(y, end);
            }
            if cols[bi].destroy_at == i as i64 && bi != ai {
                let end = destroy(&mut cols[bi], line_y, &mut crosses_out);
                y = js::max(y, end);
            }
        }
    }

    y += 20.0;
    for col in &mut cols {
        while !col.activations.is_empty() {
            end_activation(col, y, &mut min_x, &mut max_x, &mut activations_out);
        }
    }
    let mut bottom = y;
    if mirror {
        let mut foot_h: f64 = 0.0;
        for col in &cols {
            if col.life_end >= 0.0 {
                continue;
            }
            foot_h = js::max(foot_h, col.size.h);
            actors_out.push_str(&actor_svg(col, y, true));
        }
        bottom = y + foot_h;
    }
    let mut lifelines = String::new();
    for col in &cols {
        actors_out.push_str(&actor_svg(col, col.top, false));
        let end = if col.life_end >= 0.0 { col.life_end } else { y };
        if end > col.life_start {
            svg_write!(
                lifelines,
                "<path class=\"pele-lifeline\" data-id=\"",
                esc_text(col.id),
                "\" d=\"M",
                num(col.x),
                ",",
                num(col.life_start),
                "V",
                num(end),
                "\"/>"
            )
            .unwrap();
        }
    }

    let mut boxes_out = String::new();
    let box_label_attrs = svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"");
    for run in &runs {
        let b = &db.boxes[run.box_index];
        let x0 = cols[run.first].x - cols[run.first].size.w / 2.0 - box_pad - run.grow;
        let x1 = cols[run.last].x + cols[run.last].size.w / 2.0 + box_pad + run.grow;
        let fill = if b.fill == "transparent" { String::new() } else { resolve_style(&[format!("fill:{}", b.fill)]).shape };
        let data_id = match b.name.as_deref() {
            Some(name) if !name.is_empty() => svg_format!(" data-id=\"", esc_text(name), "\""),
            _ => String::new(),
        };
        svg_write!(
            boxes_out,
            "<g class=\"pele-cluster pele-box\"",
            data_id,
            "><rect x=\"",
            num(x0),
            "\" y=\"0\" width=\"",
            num(x1 - x0),
            "\" height=\"",
            num(bottom + box_pad),
            "\"",
            crate::svg::theme::rounded(&svg_format!(
                " fill=\"",
                ctx.paint(Role::GroupFill, "var(--_a)"),
                "\" stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"",
                if fill.is_empty() { " fill-opacity=\"0.5\"" } else { "" },
                fill
            )),
            "/>",
            label_svg(&run.title, (x0 + x1) / 2.0, box_pad / 2.0 + 2.0 + run.title.height / 2.0, &box_label_attrs, icons),
            "</g>"
        )
        .unwrap();
    }
    if !runs.is_empty() {
        bottom += box_pad;
    }

    let mut min_y = 0.0;
    let mut title_out = String::new();
    let room = title_room(options);
    let plain_title =
        layout_label(db.title.as_deref(), false, &m.without_math(), size, if tight { room } else { f64::INFINITY }, style::BOLD);
    let title = if tight { break_label(&plain_title, &m, js::max(120.0, room)) } else { plain_title };
    if min_x > max_x {
        extend(&mut min_x, &mut max_x, 0.0, js::max(title.width, 40.0));
    }
    if title.width > 0.0 {
        let cx = (min_x + max_x) / 2.0;
        extend(&mut min_x, &mut max_x, cx - title.width / 2.0, cx + title.width / 2.0);
        min_y = -title.height - 12.0;
        title_out =
            label_svg(&title, cx, min_y + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", icons).to_string();
    }

    let width = (max_x - min_x + 2.0 * pad).ceil();
    let height = (bottom - min_y + 2.0 * pad).ceil();
    let small_s = js::number_text(small);
    let mut content = svg_format!("<g transform=\"translate(", num(pad - min_x), ",", num(pad - min_y), ")\">");
    content.push_str(&title_out);
    if !boxes_out.is_empty() {
        svg_write!(content, "<g class=\"pele-boxes\" font-size=\"", small_s, "\">", boxes_out, "</g>").unwrap();
    }
    if !rects_out.is_empty() {
        rects_out.reverse();
        svg_write!(
            content,
            "<g class=\"pele-rects\" fill=\"",
            ctx.paint(Role::GroupHeader, "var(--_a)"),
            "\">",
            rects_out.concat(),
            "</g>"
        )
        .unwrap();
    }
    if !lifelines.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-lifelines\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_b)"),
            "\">",
            lifelines,
            "</g>"
        )
        .unwrap();
    }
    if !activations_out.is_empty() {
        activations_out.reverse();
        svg_write!(
            content,
            "<g class=\"pele-activations\" fill=\"",
            ctx.paint(Role::Selection, "var(--_s)"),
            "\" stroke=\"",
            ctx.paint(Role::Accent, "var(--_b)"),
            "\">",
            activations_out.concat(),
            "</g>"
        )
        .unwrap();
    }
    if !frames_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-frames\" font-size=\"",
            small_s,
            "\" stroke=\"",
            ctx.paint(Role::GroupStroke, "var(--_b)"),
            "\">",
            frames_out,
            "</g>"
        )
        .unwrap();
    }
    if !edges_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-edges\" fill=\"none\" stroke=\"",
            paint.line,
            "\" stroke-linecap=\"round\">",
            edges_out,
            "</g>"
        )
        .unwrap();
    }
    if !crosses_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-destroys\" fill=\"none\" stroke=\"",
            line_color,
            "\" stroke-width=\"1.5\" stroke-linecap=\"round\">",
            crosses_out,
            "</g>"
        )
        .unwrap();
    }
    if !labels_out.is_empty() {
        // Message text inherits the text color in Pele; resolved output names it.
        let fill = if resolved { svg_format!(" fill=\"", ctx.paint(Role::Text, ""), "\"") } else { String::new() };
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", small_s, "\"", fill, ">", labels_out, "</g>").unwrap();
    }
    if !notes_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-notes\" font-size=\"",
            small_s,
            "\" fill=\"",
            ctx.paint(Role::WarningBackground, "var(--_a)"),
            "\" stroke=\"",
            ctx.paint(Role::WarningText, "var(--_b)"),
            "\">",
            notes_out,
            "</g>"
        )
        .unwrap();
    }
    if !actors_out.is_empty() {
        svg_write!(content, "<g class=\"pele-nodes\">", actors_out, "</g>").unwrap();
    }
    content.push_str("</g>");

    let acc = Accessible { acc_title: db.acc_title.as_deref(), acc_descr: db.acc_descr.as_deref() };
    Rendered { svg: svg_document("sequence", width, height, size, ctx, acc, &content), width, height, links: Vec::new() }
}
