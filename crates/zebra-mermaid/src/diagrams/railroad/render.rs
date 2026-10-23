//! Railroad rendering. Ported from Pele `src/diagrams/railroad/render.ts`.
//!
//! Each rule is measured into a tree of boxes bottom-up and placed top-down, both with
//! explicit stacks as in Pele, so nesting depth never meets the call stack.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::HashMap;
use std::fmt::Write as _;

use super::common::fold;
use super::{NodeId, RailroadModel, RailroadNode};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{FONT_MONO, ROUNDED};
use crate::text::entities::decode_entities;
use crate::text::{layout_render_label as layout_label, style, Measure};
use crate::theme::Role;
use crate::util::js;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Terminal,
    NonTerminal,
    Special,
    Sequence,
    Choice,
    Bypass,
    Loop,
    Folded,
}

/// A laid-out element. The track enters at the left edge and leaves at the right edge on one
/// baseline, with `up` above it and `down` below it. Kids are indices into the box list.
struct Bx {
    kind: Kind,
    width: f64,
    up: f64,
    down: f64,
    kids: Vec<usize>,
    /// A leaf's text, or a loop's count.
    text: String,
    /// The name a non-terminal refers to.
    id: String,
    /// How far a bypass rises above the baseline, or a loop drops below it.
    reach: f64,
}

impl Bx {
    fn new(kind: Kind, width: f64, up: f64, down: f64, kids: Vec<usize>) -> Bx {
        Bx { kind, width, up, down, kids, text: String::new(), id: String::new(), reach: 0.0 }
    }
}

struct Metrics<'a> {
    measure: Measure<'a>,
    size: f64,
    padding: f64,
    height: f64,
    /// Space between elements in a sequence, and between stacked branches.
    gap: f64,
    sep: f64,
    arc: f64,
}

const LEAD: f64 = 12.0;
const HEAD_GAP: f64 = 6.0;
const RULE_GAP: f64 = 18.0;
const COUNT_SIZE: f64 = 12.0;
const ARROW: f64 = 7.0;

/// `config.railroad[key]` when it is a number of at least zero, capped at `max`.
fn option(config: &Config, key: &str, fallback: f64, max: f64) -> f64 {
    match config.lookup("railroad", key) {
        Some(ConfigValue::Number(value)) if *value >= 0.0 => js::min(*value, max),
        _ => fallback,
    }
}

/// Text is literal here: a terminal such as "<br>" or "*" is itself, not markup. Mermaid's
/// `#35;` codes still stand for characters, and escapes that would be invisible are written
/// out (`/[\n\r\t]/g`).
fn shown(text: &str) -> String {
    let decoded;
    let s = if text.contains("¶ß") {
        decoded = decode_entities(&text.replace('&', "&amp;"));
        decoded.as_str()
    } else {
        text
    };
    if !s.contains(['\n', '\r', '\t']) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn even(value: f64) -> f64 {
    (value / 2.0).ceil() * 2.0
}

fn push(boxes: &mut Vec<Bx>, b: Bx) -> usize {
    boxes.push(b);
    boxes.len() - 1
}

fn leaf(kind: Kind, text: String, id: String, st: u8, m: &Metrics) -> Bx {
    let half = m.height / 2.0;
    let width = js::max(even(m.measure.width(&text, m.size, st) + 2.0 * m.padding), m.height);
    Bx { kind, width, up: half, down: half, kids: Vec::new(), text, id, reach: 0.0 }
}

/// What a loop shows of its count, or nothing for "one or more".
fn count(min: f64, max: f64) -> String {
    if max == f64::INFINITY && min <= 1.0 {
        return String::new();
    }
    let text = |value: f64| if value.is_finite() { js::to_string(value) } else { "∞".to_string() };
    if min == max {
        return text(min) + "×";
    }
    if max == f64::INFINITY {
        return format!("≥{}×", text(min));
    }
    if min == 0.0 {
        format!("≤{}×", text(max))
    } else {
        format!("{}–{}×", text(min), text(max))
    }
}

/// Where the next branch of a choice sits below the baseline, given the one above it.
fn below(offset: f64, above: &Bx, branch: &Bx, m: &Metrics) -> f64 {
    js::max(offset + above.down + m.sep + branch.up, 2.0 * m.arc)
}

fn bypass(boxes: &mut Vec<Bx>, kid: usize, m: &Metrics) -> usize {
    let k = &boxes[kid];
    let reach = js::max(k.up + m.sep, 2.0 * m.arc);
    let mut b = Bx::new(Kind::Bypass, k.width + 4.0 * m.arc, reach, k.down, vec![kid]);
    b.reach = reach;
    push(boxes, b)
}

/// The largest `up` and `down` of some boxes.
fn extent(boxes: &[Bx], kids: &[usize]) -> (f64, f64) {
    let mut up: f64 = 0.0;
    let mut down: f64 = 0.0;
    for &kid in kids {
        up = js::max(up, boxes[kid].up);
        down = js::max(down, boxes[kid].down);
    }
    (up, down)
}

fn measure(model: &RailroadModel, root: NodeId, m: &Metrics, room: f64, boxes: &mut Vec<Bx>) -> usize {
    let mut budgets: HashMap<NodeId, f64> = HashMap::new();
    if room != f64::INFINITY {
        let mut pending: Vec<(NodeId, f64)> = vec![(root, room)];
        let mut i = 0;
        while i < pending.len() {
            let (node, width) = pending[i];
            budgets.insert(node, width);
            // A sequence that folds puts its rows inside rails, so its parts are sized for
            // those rows.
            let n = model.node(node);
            let around = match n {
                RailroadNode::Choice { .. } | RailroadNode::Optional { .. } => 4.0 * m.arc,
                RailroadNode::Repetition { min, .. } => (if *min == 0.0 { 6.0 } else { 2.0 }) * m.arc,
                RailroadNode::Sequence { .. } => 6.0 * m.arc,
                _ => 0.0,
            };
            for &child in n.parts() {
                pending.push((child, js::max(40.0, width - around)));
            }
            i += 1;
        }
    }
    fold(
        root,
        |n| model.node(n).parts().to_vec(),
        |node, kids: Vec<usize>| {
            let room = budgets.get(&node).copied().unwrap_or(f64::INFINITY);
            match model.node(node) {
                RailroadNode::Terminal { value } => {
                    let text = shown(value);
                    // `/^ +$/`: a terminal of spaces shows a visible space per space.
                    let text = if !text.is_empty() && text.bytes().all(|b| b == b' ') { "␣".repeat(text.len()) } else { text };
                    let b = leaf(Kind::Terminal, text, String::new(), style::MONO, m);
                    push(boxes, b)
                }
                RailroadNode::Nonterminal { name } => {
                    let b = leaf(Kind::NonTerminal, shown(name), name.clone(), 0, m);
                    push(boxes, b)
                }
                RailroadNode::Special { text } => {
                    let b = leaf(Kind::Special, format!("? {} ?", shown(text)), String::new(), style::ITALIC, m);
                    push(boxes, b)
                }
                RailroadNode::Sequence { .. } => sequence(boxes, kids, room, m),
                RailroadNode::Choice { .. } => {
                    let mut inner: f64 = 0.0;
                    let mut offset = 0.0;
                    for i in 0..kids.len() {
                        inner = js::max(inner, boxes[kids[i]].width);
                        if i > 0 {
                            offset = below(offset, &boxes[kids[i - 1]], &boxes[kids[i]], m);
                        }
                    }
                    let up = kids.first().map_or(0.0, |&k| boxes[k].up);
                    let down = kids.last().map_or(0.0, |&k| offset + boxes[k].down);
                    push(boxes, Bx::new(Kind::Choice, inner + 4.0 * m.arc, up, down, kids))
                }
                RailroadNode::Optional { .. } => bypass(boxes, kids[0], m),
                RailroadNode::Repetition { min, max, .. } => {
                    let (min, max) = (*min, *max);
                    let kid = kids[0];
                    if min == 1.0 && max == 1.0 {
                        return kid;
                    }
                    let text = count(min, max);
                    let k = &boxes[kid];
                    let reach = js::max(k.down + m.sep, 2.0 * m.arc);
                    let label = if text.is_empty() { 0.0 } else { even(m.measure.width(&text, COUNT_SIZE, 0)) };
                    let down = reach + if text.is_empty() { ARROW / 2.0 + 1.0 } else { COUNT_SIZE + 6.0 };
                    let mut lp = Bx::new(Kind::Loop, js::max(k.width, label) + 2.0 * m.arc, k.up, down, vec![kid]);
                    lp.text = text;
                    lp.reach = reach;
                    let lp = push(boxes, lp);
                    if min == 0.0 {
                        bypass(boxes, lp, m)
                    } else {
                        lp
                    }
                }
            }
        },
    )
}

/// A sequence, folded into rows inside rails when it is wider than its room.
fn sequence(boxes: &mut Vec<Bx>, kids: Vec<usize>, room: f64, m: &Metrics) -> usize {
    let mut width = js::max(0.0, kids.len() as f64 - 1.0) * m.gap;
    let mut up: f64 = 0.0;
    let mut down: f64 = 0.0;
    for &kid in &kids {
        width += boxes[kid].width;
        up = js::max(up, boxes[kid].up);
        down = js::max(down, boxes[kid].down);
    }
    if width <= room || kids.len() < 2 {
        return push(boxes, Bx::new(Kind::Sequence, width, up, down, kids));
    }
    let mut rows: Vec<usize> = Vec::new();
    let mut row: Vec<usize> = Vec::new();
    let mut row_w = 0.0;
    let flush = |boxes: &mut Vec<Bx>, rows: &mut Vec<usize>, row: &mut Vec<usize>, row_w: &mut f64| {
        let (up, down) = extent(boxes, row);
        let b = Bx::new(Kind::Sequence, *row_w, up, down, std::mem::take(row));
        rows.push(push(boxes, b));
        *row_w = 0.0;
    };
    for &kid in &kids {
        let kid_w = boxes[kid].width;
        if !row.is_empty() && row_w + m.gap + kid_w > js::max(40.0, room - 6.0 * m.arc) {
            flush(boxes, &mut rows, &mut row, &mut row_w);
        }
        row_w += (if row.is_empty() { 0.0 } else { m.gap }) + kid_w;
        row.push(kid);
    }
    flush(boxes, &mut rows, &mut row, &mut row_w);
    if rows.len() < 2 {
        return push(boxes, Bx::new(Kind::Sequence, width, up, down, kids));
    }
    let mut folded_width: f64 = 0.0;
    let mut offset = 0.0;
    for i in 0..rows.len() {
        folded_width = js::max(folded_width, boxes[rows[i]].width);
        if i > 0 {
            offset += boxes[rows[i - 1]].down + js::max(m.sep, 2.0 * m.arc) + boxes[rows[i]].up;
        }
    }
    let first_up = boxes[rows[0]].up;
    let last_down = boxes[rows[rows.len() - 1]].down;
    push(boxes, Bx::new(Kind::Folded, folded_width + 6.0 * m.arc, first_up, offset + last_down, rows))
}

pub(crate) fn render(model: &RailroadModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| Ok(draw(model, config, ctx, tight)))
}

/// `list.slice(from).reverse()` in place.
fn reverse<T>(list: &mut [T], from: usize) {
    if from < list.len() {
        list[from..].reverse();
    }
}

/// What one rule's drawing collects while its boxes are placed.
struct Painter<'a, 'c> {
    m: &'a Metrics<'a>,
    ctx: &'a RenderCtx<'c>,
    boxes: &'a [Bx],
    arc: f64,
    box_top: f64,
    text_shift: f64,
    stroke: &'a str,
    east_to_south: String,
    south_to_east: String,
    east_to_north: String,
    north_to_east: String,
    loop_down: String,
    loop_up: String,
    track: String,
    arrows: String,
    nodes: String,
}

impl Painter<'_, '_> {
    fn line(&mut self, x1: f64, y: f64, x2: f64) {
        if x2 > x1 {
            write!(self.track, "M{},{}H{}", num(x1), num(y), num(x2)).unwrap();
        }
    }

    fn place(&mut self, root: usize, left: f64, baseline: f64) {
        let arc = self.arc;
        let gap = self.m.gap;
        let sep = self.m.sep;
        let mut stack: Vec<usize> = vec![root];
        let mut at: Vec<f64> = vec![left, baseline];
        while let Some(b) = stack.pop() {
            let (Some(y), Some(x)) = (at.pop(), at.pop()) else { break };
            let bx = &self.boxes[b];
            let kids = &bx.kids;
            let right = x + bx.width;
            match bx.kind {
                Kind::Sequence => {
                    let mut kx = right;
                    for i in (0..kids.len()).rev() {
                        kx -= self.boxes[kids[i]].width;
                        stack.push(kids[i]);
                        at.push(kx);
                        at.push(y);
                        if i > 0 {
                            self.line(kx - gap, y, kx);
                        }
                        kx -= gap;
                    }
                }
                Kind::Folded => {
                    let start = x + 2.0 * arc;
                    let from = stack.len();
                    let mut offset = 0.0;
                    self.line(x, y, start);
                    for i in 0..kids.len() {
                        let kid = &self.boxes[kids[i]];
                        let ky = y + offset;
                        // Pushed (y, x) and reversed with the stack below, as Pele does.
                        stack.push(kids[i]);
                        at.push(ky);
                        at.push(start);
                        if i + 1 < kids.len() {
                            let next = &self.boxes[kids[i + 1]];
                            let next_offset = offset + kid.down + js::max(sep, 2.0 * arc) + next.up;
                            let mid = y + offset + kid.down + js::max(sep, 2.0 * arc) / 2.0;
                            write!(
                                self.track,
                                "M{},{}H{}V{}H{}V{}H{}",
                                num(start + kid.width),
                                num(ky),
                                num(right - 2.0 * arc),
                                num(mid),
                                num(x + arc),
                                num(y + next_offset),
                                num(start)
                            )
                            .unwrap();
                            write!(self.arrows, "M{},{}l7,3.5l-7,3.5z", num(start - ARROW), num(y + next_offset - ARROW / 2.0)).unwrap();
                            offset = next_offset;
                        } else {
                            write!(self.track, "M{},{}H{}V{}H{}", num(start + kid.width), num(ky), num(right - arc), num(y), num(right))
                                .unwrap();
                        }
                    }
                    reverse(&mut stack, from);
                    reverse(&mut at, 2 * from);
                }
                Kind::Choice => {
                    let inner = bx.width - 4.0 * arc;
                    let rail_in = x + arc;
                    let rail_out = right - arc;
                    let from = stack.len();
                    let mut offset = 0.0;
                    for i in 0..kids.len() {
                        let kid = &self.boxes[kids[i]];
                        if i > 0 {
                            offset = below(offset, &self.boxes[kids[i - 1]], kid, self.m);
                        }
                        let ky = y + offset;
                        let kx = x + 2.0 * arc + (inner - kid.width) / 2.0;
                        let last = i == kids.len() - 1;
                        if i == 0 {
                            self.line(x, y, kx);
                            self.line(kx + kid.width, y, right);
                        } else {
                            let mut d = if last {
                                svg_format!("M", num(x), ",", num(y), self.east_to_south, "V", num(ky - arc))
                            } else {
                                svg_format!("M", num(rail_in), ",", num(ky - arc))
                            };
                            d.push_str(&self.south_to_east);
                            if kx > rail_in + arc {
                                write!(d, "H{}", num(kx)).unwrap();
                            }
                            svg_write!(d, "M", num(kx + kid.width), ",", num(ky)).unwrap();
                            if kx > rail_in + arc {
                                write!(d, "H{}", num(rail_out - arc)).unwrap();
                            }
                            d.push_str(&self.east_to_north);
                            if last {
                                write!(d, "V{}{}", num(y + arc), self.north_to_east).unwrap();
                            }
                            self.track.push_str(&d);
                        }
                        stack.push(kids[i]);
                        at.push(ky);
                        at.push(kx);
                    }
                    // Reversed, so that the branches come off the stack top to bottom.
                    reverse(&mut stack, from);
                    reverse(&mut at, 2 * from);
                }
                Kind::Bypass => {
                    let kid = kids[0];
                    let kx = x + 2.0 * arc;
                    self.line(x, y, kx);
                    self.line(kx + self.boxes[kid].width, y, right);
                    let d = svg_format!(
                        "M",
                        num(x),
                        ",",
                        num(y),
                        self.east_to_north,
                        "V",
                        num(y - bx.reach + arc),
                        self.north_to_east,
                        "H",
                        num(right - 2.0 * arc),
                        self.east_to_south,
                        "V",
                        num(y - arc),
                        self.south_to_east
                    );
                    self.track.push_str(&d);
                    stack.push(kid);
                    at.push(kx);
                    at.push(y);
                }
                Kind::Loop => {
                    let kid = kids[0];
                    let kid_w = self.boxes[kid].width;
                    let kx = x + (bx.width - kid_w) / 2.0;
                    let loop_y = y + bx.reach;
                    let middle = x + bx.width / 2.0;
                    self.line(x, y, kx);
                    self.line(kx + kid_w, y, right);
                    let d = svg_format!(
                        "M",
                        num(x + arc),
                        ",",
                        num(y),
                        self.loop_down,
                        "V",
                        num(loop_y - arc),
                        self.south_to_east,
                        "H",
                        num(right - arc),
                        self.east_to_north,
                        "V",
                        num(y + arc),
                        self.loop_up
                    );
                    self.track.push_str(&d);
                    write!(self.arrows, "M{},{}l7,-3.5v7z", num(middle - ARROW / 2.0), num(loop_y)).unwrap();
                    if !bx.text.is_empty() {
                        write!(
                            self.nodes,
                            "<text class=\"pele-edge-label pele-count\" x=\"{}\" y=\"{}\" font-size=\"12\" fill=\"{}\">{}</text>",
                            num(middle),
                            num(loop_y + COUNT_SIZE + 3.0),
                            self.ctx.paint(Role::SecondaryText, "var(--_m)"),
                            esc(&bx.text)
                        )
                        .unwrap();
                    }
                    stack.push(kid);
                    at.push(kx);
                    at.push(y);
                }
                Kind::Terminal | Kind::NonTerminal | Kind::Special => self.leaf(bx, x, y),
            }
        }
    }

    fn leaf(&mut self, bx: &Bx, x: f64, y: f64) {
        let ctx = self.ctx;
        let text = &bx.text;
        let space = if text.starts_with(' ') || text.ends_with(' ') || text.contains("  ") { " xml:space=\"preserve\"" } else { "" };
        let shape = svg_format!(
            "<rect x=\"",
            num(x),
            "\" y=\"",
            num(y - self.box_top),
            "\" width=\"",
            num(bx.width),
            "\" height=\"",
            num(self.m.height),
            "\""
        );
        let label = svg_format!("<text x=\"", num(x + bx.width / 2.0), "\" y=\"", num(y + self.text_shift), "\"", space);
        let stroke_color = ctx.paint(Role::NodeStroke, "var(--_b)");
        match bx.kind {
            Kind::Terminal => write!(
                self.nodes,
                "<g class=\"pele-node pele-terminal\">{} rx=\"{}\" fill=\"{}\" stroke=\"{}\"{}/>{} font-family=\"{}\">{}</text></g>",
                shape,
                num(self.box_top),
                ctx.paint(Role::NodeFill, "var(--_s)"),
                stroke_color,
                self.stroke,
                label,
                FONT_MONO,
                esc(text)
            )
            .unwrap(),
            // A non-terminal is a box on the canvas in Pele, set apart from the filled
            // terminals; in resolved mode it takes the secondary surface to stay set apart.
            Kind::NonTerminal => write!(
                self.nodes,
                "<g class=\"pele-node pele-nonterminal\" data-id=\"{}\">{} fill=\"{}\" stroke=\"{}\"{}{ROUNDED}/>{}>{}</text></g>",
                esc_text(&bx.id),
                shape,
                ctx.paint(Role::GroupHeader, "var(--_bg)"),
                stroke_color,
                self.stroke,
                label,
                esc(text)
            )
            .unwrap(),
            // A special sequence is an annotation: a dashed box that reads as empty.
            _ => {
                let muted = ctx.paint(Role::SecondaryText, "var(--_m)");
                // `text.slice(2, -2)`: the text between "? " and " ?".
                let inner = &text[2..text.len() - 2];
                write!(self.nodes,
                    "<g class=\"pele-node pele-special\">{} fill=\"{}\" stroke=\"{}\" stroke-dasharray=\"4 3\"{}{ROUNDED}/>{} font-style=\"italic\"><tspan fill=\"{}\">? </tspan>{}<tspan fill=\"{}\"> ?</tspan></text></g>",
                    shape,
                        ctx.paint(Role::Background, "var(--_bg)"),
                    stroke_color,
                    self.stroke,
                    label,
                    muted,
                    esc(inner),
                    muted
                ).unwrap();
            }
        }
    }
}

fn draw(model: &RailroadModel, config: &Config, ctx: &RenderCtx, tight: bool) -> Rendered {
    let options = ctx.options;
    let base = ctx.font_size();
    let font_size = option(config, "fontSize", 0.0, 200.0);
    // `fontSize || ...`: zero means unset.
    let size = if font_size != 0.0 { font_size } else { js::round(base * 0.875) };
    let head_size = if font_size != 0.0 { font_size } else { base };
    let padding = option(config, "padding", -1.0, 200.0);
    let line_height = js::round(size * 1.5);
    let m = Metrics {
        measure: ctx.measure(),
        size,
        padding: if padding < 0.0 { 12.0 } else { padding },
        height: even(line_height + if padding < 0.0 { 9.0 } else { 2.0 * padding }),
        gap: js::min(option(config, "horizontalSeparation", 16.0, 500.0), if tight { 12.0 } else { f64::INFINITY }),
        sep: option(config, "verticalSeparation", 10.0, 500.0),
        arc: js::min(option(config, "arcRadius", 10.0, 200.0), if tight { 6.0 } else { f64::INFINITY }),
    };
    let show_markers = !matches!(config.lookup("railroad", "showMarkers"), Some(ConfigValue::Bool(false)));
    let radius = if show_markers { option(config, "markerRadius", 4.0, 50.0) } else { 0.0 };
    let stroke_width = option(config, "strokeWidth", 0.0, 20.0);
    let stroke = if stroke_width > 0.0 { svg_format!(" stroke-width=\"", num(stroke_width), "\"") } else { String::new() };
    let pad = ctx.padding();
    let arc = m.arc;

    let a = svg_format!(num(arc));
    let turn = |sweep: u8, dx: &str, dy: &str| format!("a{a},{a} 0 0 {sweep} {dx}{a},{dy}{a}");

    let title = layout_label(
        model.title.as_deref(),
        false,
        &m.measure.without_math(),
        base,
        if tight { title_room(options) } else { 4000.0 },
        style::BOLD,
    );
    let title_height = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let head_height = js::round(head_size * 1.5);

    let line_color = ctx.paint(Role::Line, "var(--_l)");
    let mut rules = String::new();
    let mut width = title.width;
    let mut top = pad + title_height;
    let mut bottom = pad + title.height;
    for rule in &model.rules {
        let room = if tight {
            js::max(80.0, options.max_width.unwrap_or(320.0) - 2.0 * pad - 4.0 * radius - 2.0 * LEAD - 1.0)
        } else {
            f64::INFINITY
        };
        let mut boxes: Vec<Bx> = Vec::new();
        let root = measure(model, rule.definition, &m, room, &mut boxes);
        let (box_width, box_up, box_down) = (boxes[root].width, boxes[root].up, boxes[root].down);
        let name = shown(&rule.name);
        let baseline = top + head_height + HEAD_GAP + box_up;
        let start = pad + 2.0 * radius;
        let left = start + LEAD;
        let end = left + box_width + LEAD;
        let mut painter = Painter {
            m: &m,
            ctx,
            boxes: &boxes,
            arc,
            box_top: m.height / 2.0,
            text_shift: size * 0.35,
            stroke: &stroke,
            east_to_south: turn(1, "", ""),
            south_to_east: turn(0, "", ""),
            east_to_north: turn(0, "", "-"),
            north_to_east: turn(1, "", "-"),
            loop_down: turn(0, "-", ""),
            loop_up: turn(0, "-", "-"),
            track: String::new(),
            arrows: String::new(),
            nodes: String::new(),
        };
        painter.line(start, baseline, left);
        painter.line(left + box_width, baseline, end);
        painter.place(root, left, baseline);
        let markers = if radius > 0.0 {
            svg_format!(
                "<circle class=\"pele-marker pele-start\" cx=\"",
                num(pad + radius),
                "\" cy=\"",
                num(baseline),
                "\" r=\"",
                num(radius),
                "\" fill=\"",
                line_color,
                "\"/><circle class=\"pele-marker pele-end\" cx=\"",
                num(end + radius),
                "\" cy=\"",
                num(baseline),
                "\" r=\"",
                num(radius),
                "\" fill=\"",
                line_color,
                "\"/>"
            )
        } else {
            String::new()
        };
        svg_write!(
            rules,
            "<g class=\"pele-rule\" data-id=\"",
            esc_text(&rule.name),
            "\"><text class=\"pele-rule-name\" x=\"",
            num(pad),
            "\" y=\"",
            num(top + head_height / 2.0 + head_size * 0.35),
            "\" font-size=\"",
            num(head_size),
            "\" font-weight=\"var(--_hw)\" text-anchor=\"start\">",
            esc(&name),
            "</text><path class=\"pele-edge pele-track\" d=\"",
            painter.track,
            "\" fill=\"none\" stroke=\"",
            line_color,
            "\"",
            stroke,
            "/>"
        )
        .unwrap();
        if !painter.arrows.is_empty() {
            svg_write!(
                rules,
                "<path class=\"pele-marker\" d=\"",
                painter.arrows,
                "\" fill=\"",
                ctx.paint(Role::Arrow, "var(--_l)"),
                "\"/>"
            )
            .unwrap();
        }
        rules.push_str(&markers);
        rules.push_str(&painter.nodes);
        rules.push_str("</g>");
        width = js::max(js::max(width, end + 2.0 * radius - pad), m.measure.width(&name, head_size, style::BOLD));
        bottom = baseline + box_down;
        top = bottom + m.sep + RULE_GAP;
    }

    let total_width = (width + 2.0 * pad).ceil();
    let total_height = (bottom + pad).ceil();
    let mut content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        None
    ));
    if !rules.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-rules\" transform=\"translate(0.5,0.5)\" font-size=\"",
            num(size),
            "\" text-anchor=\"middle\">",
            rules,
            "</g>"
        )
        .unwrap();
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("railroad", total_width, total_height, base, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(count(0.0, f64::INFINITY), "");
        assert_eq!(count(1.0, f64::INFINITY), "");
        assert_eq!(count(2.0, f64::INFINITY), "≥2×");
        assert_eq!(count(3.0, 3.0), "3×");
        assert_eq!(count(0.0, 5.0), "≤5×");
        assert_eq!(count(2.0, 5.0), "2–5×");
        assert_eq!(count(f64::INFINITY, f64::INFINITY), "∞×");
        assert_eq!(count(1e21, 1e21), "1e+21×");
    }

    #[test]
    fn shown_text() {
        assert_eq!(shown("a\nb\tc\rd"), "a\\nb\\tc\\rd");
        assert_eq!(shown("plain & <x>"), "plain & <x>");
        assert_eq!(even(13.0), 14.0);
        assert_eq!(even(14.0), 14.0);
        assert_eq!(even(13.2), 14.0);
    }
}
