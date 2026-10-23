//! Participant shapes. Ported from Pele `src/diagrams/sequence/actors.ts`.

use crate::diagrams::RenderCtx;
use crate::svg::builder::{num, render_label_svg as label_svg};
use crate::svg::concat::svg_format;
use crate::svg::theme::ROUNDED;
use crate::text::{Icons, RenderLabel as Label};
use crate::theme::Role;
use crate::util::js;

const PAD_X: f64 = 16.0;
const PAD_Y: f64 = 10.0;
const GLYPH: f64 = 34.0;
const GLYPH_GAP: f64 = 4.0;
const STACK: f64 = 5.0;
const CAP: f64 = 6.0;

const KINDS: [&str; 8] = ["participant", "actor", "boundary", "control", "entity", "database", "collections", "queue"];
const FIGURES: [&str; 5] = ["actor", "boundary", "control", "entity", "database"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ActorSize {
    pub w: f64,
    pub h: f64,
}

pub(super) fn actor_kind(kind: &str) -> &str {
    if KINDS.contains(&kind) {
        kind
    } else {
        "participant"
    }
}

/// A figure stands above its label. The other kinds hold the label inside.
pub(super) fn is_figure(kind: &str) -> bool {
    FIGURES.contains(&kind)
}

pub(super) fn actor_size(kind: &str, label: &Label, min_w: f64, min_h: f64, padding_x: Option<f64>) -> ActorSize {
    let padding_x = padding_x.unwrap_or(PAD_X);
    if is_figure(kind) {
        return ActorSize {
            w: js::max(40.0, label.width + 8.0),
            h: GLYPH + if label.height > 0.0 { GLYPH_GAP + label.height } else { 0.0 },
        };
    }
    let extra = if kind == "collections" {
        STACK
    } else if kind == "queue" {
        2.0 * CAP
    } else {
        0.0
    };
    ActorSize {
        w: js::max(min_w, label.width + 2.0 * padding_x) + extra,
        h: js::max(min_h, label.height + 2.0 * PAD_Y) + if kind == "collections" { STACK } else { 0.0 },
    }
}

/// The paint of a participant's shape: its fill and outline (Pele `SHAPE`), and of the lines
/// of a figure (Pele `LINE`).
struct Paint {
    shape: String,
    line: String,
}

impl Paint {
    fn new(ctx: &RenderCtx) -> Self {
        let fill = ctx.paint(Role::NodeFill, "var(--_s)");
        let stroke = ctx.paint(Role::NodeStroke, "var(--_b)");
        Paint { shape: format!(" fill=\"{}\" stroke=\"{}\"", fill, stroke), line: format!(" fill=\"none\" stroke=\"{}\"", stroke) }
    }
}

fn figure(kind: &str, cx: f64, top: f64, p: &Paint) -> String {
    let x = num(cx);
    let y = |dy: f64| num(top + dy);
    let (shape, line) = (&p.shape, &p.line);
    match kind {
        "actor" => format!(
            "<circle cx=\"{x}\" cy=\"{}\" r=\"6\"{shape}/><path d=\"M{x},{}V{}M{},{}H{}M{},{}L{x},{}L{},{}\"{line}/>",
            y(7.0),
            y(13.0),
            y(24.0),
            num(cx - 10.0),
            y(17.0),
            num(cx + 10.0),
            num(cx - 8.0),
            y(34.0),
            y(24.0),
            num(cx + 8.0),
            y(34.0)
        ),
        "boundary" => format!(
            "<path d=\"M{},{}V{}M{},{}H{}\"{line}/><circle cx=\"{}\" cy=\"{}\" r=\"11\"{shape}/>",
            num(cx - 19.0),
            y(6.0),
            y(28.0),
            num(cx - 19.0),
            y(17.0),
            num(cx - 8.0),
            num(cx + 3.0),
            y(17.0)
        ),
        "control" => format!(
            "<circle cx=\"{x}\" cy=\"{}\" r=\"11\"{shape}/><path d=\"M{},{}L{},{}L{},{}\"{line}/>",
            y(21.0),
            num(cx + 5.0),
            y(5.0),
            num(cx - 1.0),
            y(10.0),
            num(cx + 5.0),
            y(15.0)
        ),
        "entity" => format!(
            "<circle cx=\"{x}\" cy=\"{}\" r=\"11\"{shape}/><path d=\"M{},{}H{}\"{line}/>",
            y(15.0),
            num(cx - 11.0),
            y(30.0),
            num(cx + 11.0)
        ),
        _ => {
            let l = num(cx - 14.0);
            let r = num(cx + 14.0);
            format!(
                "<path d=\"M{l},{}V{}A14,5 0 0 0 {r},{}V{}A14,5 0 0 0 {l},{}Z\"{shape}/><path d=\"M{l},{}A14,5 0 0 0 {r},{}\"{line}/>",
                y(7.0),
                y(27.0),
                y(27.0),
                y(7.0),
                y(7.0),
                y(7.0),
                y(7.0)
            )
        }
    }
}

/// Draws a participant whose bounding box has its top center at (cx, top).
pub(super) fn draw_actor(kind: &str, cx: f64, top: f64, size: ActorSize, label: &Label, icons: Option<&Icons>, ctx: &RenderCtx) -> String {
    let ActorSize { w, h } = size;
    let p = Paint::new(ctx);
    let text = " class=\"pele-label\"";
    if is_figure(kind) {
        return figure(kind, cx, top, &p) + &svg_format!(label_svg(label, cx, top + GLYPH + GLYPH_GAP + label.height / 2.0, text, icons));
    }
    let left = cx - w / 2.0;
    let shape = &p.shape;
    if kind == "collections" {
        let bw = w - STACK;
        let bh = h - STACK;
        let back =
            format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{shape}{ROUNDED}/>", num(left + STACK), num(top), num(bw), num(bh));
        let front =
            format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{shape}{ROUNDED}/>", num(left), num(top + STACK), num(bw), num(bh));
        return back + &front + &svg_format!(label_svg(label, left + bw / 2.0, top + STACK + bh / 2.0, text, icons));
    }
    if kind == "queue" {
        let l = num(left + CAP);
        let r = num(left + w - CAP);
        let b = num(top + h);
        let ry = num(h / 2.0);
        let t = num(top);
        return format!(
            "<path d=\"M{l},{t}H{r}A{CAP},{ry} 0 0 1 {r},{b}H{l}A{CAP},{ry} 0 0 1 {l},{t}Z\"{shape}/><path d=\"M{r},{t}A{CAP},{ry} 0 0 0 {r},{b}\"{}/>",
            p.line
        ) + &svg_format!(label_svg(label, cx - CAP / 2.0, top + h / 2.0, text, icons));
    }
    format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{shape}{ROUNDED}/>", num(left), num(top), num(w), num(h))
        + &svg_format!(label_svg(label, cx, top + h / 2.0, text, icons))
}
