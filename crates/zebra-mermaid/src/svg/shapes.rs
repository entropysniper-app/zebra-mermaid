//! Node shapes: their size around a label, their outline, and where edges meet them.
//! Ported from Pele `src/svg/shapes.ts`.
//!
//! The functions that take a node's layout box receive its center and size (`cx`, `cy`, `cw`,
//! `ch`) rather than a layout node, so that every diagram can call them.

use super::builder::{num, push_num};
use crate::svg::concat::{svg_format, svg_write};
use crate::theme::{Role, Theming};
use crate::util::{js, math};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeSize {
    pub w: f64,
    pub h: f64,
    /// Vertical shift of the label from the shape's center.
    pub dy: f64,
}

pub const PAD_X: f64 = 18.0;
pub const PAD_Y: f64 = 12.0;

/// Kept clear at each end of a straight side, for the corner.
const CORNER: f64 = 12.0;

const ROUNDED: &str = " rx=\"var(--pele-radius,4px)\"";

const SHAPES: [&str; 54] = [
    "rect",
    "rounded",
    "stadium",
    "fr-rect",
    "cyl",
    "circle",
    "dbl-circ",
    "ellipse",
    "diam",
    "hex",
    "odd",
    "lean-r",
    "lean-l",
    "trap-b",
    "trap-t",
    "text",
    "sm-circ",
    "f-circ",
    "fr-circ",
    "fork",
    "notch-rect",
    "lin-rect",
    "div-rect",
    "win-pane",
    "tri",
    "flip-tri",
    "sl-rect",
    "notch-pent",
    "hourglass",
    "delay",
    "h-cyl",
    "cross-circ",
    "st-rect",
    "doc",
    "flag",
    "datastore",
    "folder",
    "bucket",
    "console",
    "browser",
    "person",
    "bang",
    "cloud",
    "brace",
    "brace-r",
    "braces",
    "bolt",
    "lin-cyl",
    "curv-trap",
    "docs",
    "bow-rect",
    "tag-rect",
    "tag-doc",
    "lin-doc",
];
const COLLAPSED: &str = "collapsedGroup";

/// The shape name, or `rect` for a name Pele does not know.
fn known(shape: &str) -> &str {
    if shape == COLLAPSED || SHAPES.contains(&shape) {
        shape
    } else {
        "rect"
    }
}

fn is_known(shape: &str) -> bool {
    shape == COLLAPSED || SHAPES.contains(&shape)
}

/// How far an ellipse's outline has fallen away from its bounding box, `t` from the middle of
/// a side `half` long.
fn bulge(t: f64, half: f64, depth: f64) -> f64 {
    let k = js::min(1.0, t.abs() / half);
    depth * (1.0 - math::sqrt(1.0 - k * k))
}

fn round_inset(w: f64, h: f64, side: i32, t: f64) -> f64 {
    if side & 1 != 0 {
        bulge(t, h / 2.0, w / 2.0)
    } else {
        bulge(t, w / 2.0, h / 2.0)
    }
}

fn cap_inset(w: f64, _h: f64, side: i32, t: f64) -> f64 {
    if side & 1 != 0 {
        0.0
    } else {
        bulge(t, w / 2.0, js::min(12.0, js::max(6.0, w / 14.0)))
    }
}

fn lean_inset(_w: f64, h: f64, side: i32) -> f64 {
    if side & 1 != 0 {
        h / 8.0
    } else {
        0.0
    }
}

fn tri_size(tw: f64, th: f64, flip: bool) -> ShapeSize {
    let w = js::max(1.6 * tw + 40.0, 80.0);
    let h = (th + 10.0) / (1.0 - (tw + 8.0) / w);
    let dy = h / 2.0 - 10.0 - th / 2.0;
    ShapeSize { w, h, dy: if flip { -dy } else { dy } }
}

fn path(d: &str, a: &str) -> String {
    svg_format!("<path d=\"", d, "\"", a, "/>")
}

fn box_size(tw: f64, th: f64) -> ShapeSize {
    ShapeSize { w: js::max(tw + 2.0 * PAD_X, 2.0 * PAD_X + 12.0), h: th + 2.0 * PAD_Y, dy: 0.0 }
}

fn poly(points: &[f64], a: &str) -> String {
    let mut out = String::with_capacity(24 + points.len() * 8 + a.len());
    out.push_str("<polygon points=\"");
    let mut i = 0;
    while i + 1 < points.len() {
        if i > 0 {
            out.push(' ');
        }
        push_num(&mut out, points[i]);
        out.push(',');
        push_num(&mut out, points[i + 1]);
        i += 2;
    }
    out.push('"');
    out.push_str(a);
    out.push_str("/>");
    out
}

/// A rectangle centered on the origin; `rx` is the whole radius attribute, if any.
fn rect(w: f64, h: f64, a: &str, rx: &str) -> String {
    let mut out = String::with_capacity(64 + rx.len() + a.len());
    push_rect_open(&mut out, w, h);
    out.push_str(rx);
    out.push_str(a);
    out.push_str("/>");
    out
}

/// A rectangle with a radius written as a number.
fn rect_radius(w: f64, h: f64, a: &str, rx: f64) -> String {
    let mut out = String::with_capacity(72 + a.len());
    push_rect_open(&mut out, w, h);
    out.push_str(" rx=\"");
    push_num(&mut out, rx);
    out.push('"');
    out.push_str(a);
    out.push_str("/>");
    out
}

fn push_rect_open(out: &mut String, w: f64, h: f64) {
    out.push_str("<rect x=\"");
    push_num(out, -w / 2.0);
    out.push_str("\" y=\"");
    push_num(out, -h / 2.0);
    out.push_str("\" width=\"");
    push_num(out, w);
    out.push_str("\" height=\"");
    push_num(out, h);
    out.push('"');
}

fn brace(x: f64, h: f64, dir: f64, line: &str) -> String {
    let y = h / 2.0;
    let k = 6.0 * dir;
    svg_format!(
        "<path d=\"M",
        num(x),
        ",",
        num(-y),
        "q",
        num(k),
        ",0 ",
        num(k),
        ",6V-6q0,6 ",
        num(k),
        ",6q",
        num(-k),
        ",0 ",
        num(-k),
        ",6V",
        num(y - 6.0),
        "q0,6 ",
        num(-k),
        ",6\" fill=\"none\"",
        line,
        "/>"
    )
}

fn wave(w: f64, h: f64, a: &str) -> String {
    let x = w / 2.0;
    let top = -h / 2.0;
    let base = h / 2.0 - 10.0;
    let q = w / 4.0;
    svg_format!(
        "<path d=\"M",
        num(-x),
        ",",
        num(top),
        "H",
        num(x),
        "V",
        num(base),
        "q",
        num(-q),
        ",-12 ",
        num(-2.0 * q),
        ",0t",
        num(-2.0 * q),
        ",0Z\"",
        a,
        "/>"
    )
}

/// The size of a shape around a label `tw` by `th`.
pub fn shape_size(shape: &str, tw: f64, th: f64) -> ShapeSize {
    let fixed = |w: f64, h: f64| ShapeSize { w, h, dy: 0.0 };
    match known(shape) {
        "rect" | "rounded" | "notch-rect" | "datastore" | "tag-rect" => box_size(tw, th),
        "stadium" => {
            let h = th + 2.0 * PAD_Y;
            ShapeSize { w: tw + h * 0.5 + 2.0 * PAD_X - 8.0, h, dy: 0.0 }
        }
        "fr-rect" => ShapeSize { w: tw + 2.0 * PAD_X + 16.0, h: th + 2.0 * PAD_Y, dy: 0.0 },
        "cyl" => {
            let w = js::max(tw + 2.0 * PAD_X, 56.0);
            let ry = js::min(12.0, js::max(6.0, w / 14.0));
            ShapeSize { w, h: th + 2.0 * PAD_Y + 2.0 * ry, dy: ry / 2.0 }
        }
        "circle" => {
            let d = js::max(math::sqrt(tw * tw + th * th) + 16.0, 44.0);
            ShapeSize { w: d, h: d, dy: 0.0 }
        }
        "dbl-circ" => {
            let d = js::max(math::sqrt(tw * tw + th * th) + 26.0, 54.0);
            ShapeSize { w: d, h: d, dy: 0.0 }
        }
        "ellipse" => ShapeSize { w: tw * 1.3 + 2.0 * PAD_X, h: th * 1.2 + 2.0 * PAD_Y, dy: 0.0 },
        "diam" => {
            let a = tw + 16.0;
            let b = th + 8.0;
            ShapeSize { w: a + 1.6 * b, h: a / 1.6 + b, dy: 0.0 }
        }
        "hex" | "lean-r" | "lean-l" | "trap-b" | "trap-t" => {
            let h = th + 2.0 * PAD_Y;
            ShapeSize { w: tw + 2.0 * PAD_X + h / 2.0, h, dy: 0.0 }
        }
        "odd" | "delay" => {
            let h = th + 2.0 * PAD_Y;
            ShapeSize { w: tw + 2.0 * PAD_X + h / 4.0, h, dy: 0.0 }
        }
        "text" => ShapeSize { w: tw + 8.0, h: th + 8.0, dy: 0.0 },
        "sm-circ" | "f-circ" => fixed(14.0, 14.0),
        "fr-circ" => fixed(18.0, 18.0),
        "fork" => fixed(72.0, 8.0),
        "lin-rect" => ShapeSize { w: tw + 2.0 * PAD_X + 8.0, h: th + 2.0 * PAD_Y, dy: 0.0 },
        "div-rect" => ShapeSize { w: tw + 2.0 * PAD_X, h: th + 2.0 * PAD_Y + 8.0, dy: 4.0 },
        "win-pane" | "st-rect" => ShapeSize { w: tw + 2.0 * PAD_X + 8.0, h: th + 2.0 * PAD_Y + 8.0, dy: 4.0 },
        "tri" => tri_size(tw, th, false),
        "flip-tri" => tri_size(tw, th, true),
        "sl-rect" => ShapeSize { w: tw + 2.0 * PAD_X, h: th + 2.0 * PAD_Y + 10.0, dy: 5.0 },
        "notch-pent" => ShapeSize { w: tw + 2.0 * PAD_X, h: th + 2.0 * PAD_Y, dy: 0.0 },
        "hourglass" => fixed(32.0, 40.0),
        "h-cyl" => {
            let h = th + 2.0 * PAD_Y;
            let rx = js::min(12.0, h / 4.0);
            ShapeSize { w: tw + 2.0 * PAD_X + 2.0 * rx, h, dy: 0.0 }
        }
        "cross-circ" => fixed(44.0, 44.0),
        "doc" | "tag-doc" => ShapeSize { w: tw + 2.0 * PAD_X, h: th + 2.0 * PAD_Y + 10.0, dy: -5.0 },
        "flag" => ShapeSize { w: tw + 2.0 * PAD_X, h: th + 2.0 * PAD_Y + 16.0, dy: 0.0 },
        "folder" => ShapeSize { w: js::max(tw + 2.0 * PAD_X, 64.0), h: th + 2.0 * PAD_Y + 8.0, dy: 4.0 },
        "bucket" => ShapeSize { w: tw + 2.0 * PAD_X + 20.0, h: th + 2.0 * PAD_Y + 10.0, dy: 3.0 },
        "console" | "browser" => ShapeSize { w: js::max(tw + 2.0 * PAD_X, 72.0), h: th + 2.0 * PAD_Y + 12.0, dy: 6.0 },
        "person" => ShapeSize { w: js::max(tw + 2.0 * PAD_X, 56.0), h: th + 2.0 * PAD_Y + 26.0, dy: 13.0 },
        "bang" => ShapeSize { w: tw + 2.0 * PAD_X + 28.0, h: th + 2.0 * PAD_Y + 24.0, dy: 0.0 },
        "cloud" => ShapeSize { w: tw + 2.0 * PAD_X + 32.0, h: th + 2.0 * PAD_Y + 24.0, dy: 0.0 },
        "brace" | "brace-r" => ShapeSize { w: tw + 2.0 * PAD_X + 12.0, h: th + 2.0 * PAD_Y, dy: 0.0 },
        "braces" => ShapeSize { w: tw + 2.0 * PAD_X + 24.0, h: th + 2.0 * PAD_Y, dy: 0.0 },
        "bolt" => fixed(36.0, 56.0),
        "lin-cyl" => {
            let w = js::max(tw + 2.0 * PAD_X, 56.0);
            let ry = js::min(12.0, js::max(6.0, w / 14.0));
            ShapeSize { w, h: th + 2.0 * PAD_Y + 2.0 * ry + 8.0, dy: ry / 2.0 + 4.0 }
        }
        "curv-trap" => {
            let h = th + 2.0 * PAD_Y;
            ShapeSize { w: tw + 2.0 * PAD_X + h * 0.6, h, dy: 0.0 }
        }
        "docs" => ShapeSize { w: tw + 2.0 * PAD_X + 8.0, h: th + 2.0 * PAD_Y + 18.0, dy: -1.0 },
        "bow-rect" => {
            let h = th + 2.0 * PAD_Y;
            ShapeSize { w: tw + 2.0 * PAD_X + h * 0.4, h, dy: 0.0 }
        }
        "lin-doc" => ShapeSize { w: tw + 2.0 * PAD_X + 8.0, h: th + 2.0 * PAD_Y + 10.0, dy: -5.0 },
        COLLAPSED => ShapeSize { w: tw + 2.0 * PAD_X + 8.0, h: th + 2.0 * PAD_Y + 8.0, dy: -2.0 },
        _ => box_size(tw, th),
    }
}

/// Draws a shape of size `w` by `h` centered on the origin. `a` is the fill and stroke
/// attributes of the outline; `line` those of inner lines (`fill="none"` is added).
pub fn draw_shape(shape: &str, w: f64, h: f64, a: &str, line: &str, theming: &Theming) -> String {
    let rr = |w: f64, h: f64, a: &str| rect(w, h, a, ROUNDED);
    let lcolor = theming.paint(Role::Line, "var(--_l)");
    match known(shape) {
        "rect" => rr(w, h, a),
        "rounded" => rect_radius(w, h, a, js::min(12.0, h / 2.0)),
        "stadium" => rect_radius(w, h, a, h / 2.0),
        "fr-rect" => {
            let x = w / 2.0 - 8.0;
            svg_format!(
                rr(w, h, a),
                "<path d=\"M",
                num(-x),
                ",",
                num(-h / 2.0),
                "V",
                num(h / 2.0),
                "M",
                num(x),
                ",",
                num(-h / 2.0),
                "V",
                num(h / 2.0),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "cyl" => {
            let rx = w / 2.0;
            let ry = js::min(12.0, js::max(6.0, w / 14.0));
            let top = -h / 2.0 + ry;
            let bottom = h / 2.0 - ry;
            svg_format!(
                "<path d=\"M",
                num(-rx),
                ",",
                num(top),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 1 ",
                num(rx),
                ",",
                num(top),
                "V",
                num(bottom),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 1 ",
                num(-rx),
                ",",
                num(bottom),
                "Z\"",
                a,
                "/><path d=\"M",
                num(-rx),
                ",",
                num(top),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 0 ",
                num(rx),
                ",",
                num(top),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "circle" | "sm-circ" => svg_format!("<circle r=\"", num(w / 2.0), "\"", a, "/>"),
        "dbl-circ" => {
            svg_format!("<circle r=\"", num(w / 2.0), "\"", a, "/><circle r=\"", num(w / 2.0 - 5.0), "\" fill=\"none\"", line, "/>")
        }
        "ellipse" => svg_format!("<ellipse rx=\"", num(w / 2.0), "\" ry=\"", num(h / 2.0), "\"", a, "/>"),
        "diam" => poly(&[0.0, -h / 2.0, w / 2.0, 0.0, 0.0, h / 2.0, -w / 2.0, 0.0], a),
        "hex" => {
            let m = h / 4.0;
            poly(
                &[-w / 2.0 + m, -h / 2.0, w / 2.0 - m, -h / 2.0, w / 2.0, 0.0, w / 2.0 - m, h / 2.0, -w / 2.0 + m, h / 2.0, -w / 2.0, 0.0],
                a,
            )
        }
        "odd" => {
            let notch = h / 4.0;
            poly(&[-w / 2.0, -h / 2.0, w / 2.0, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0, h / 2.0, -w / 2.0 + notch, 0.0], a)
        }
        "lean-r" => {
            let s = h / 4.0;
            poly(&[-w / 2.0 + s, -h / 2.0, w / 2.0, -h / 2.0, w / 2.0 - s, h / 2.0, -w / 2.0, h / 2.0], a)
        }
        "lean-l" => {
            let s = h / 4.0;
            poly(&[-w / 2.0, -h / 2.0, w / 2.0 - s, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0 + s, h / 2.0], a)
        }
        "trap-b" => {
            let s = h / 4.0;
            poly(&[-w / 2.0 + s, -h / 2.0, w / 2.0 - s, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0, h / 2.0], a)
        }
        "trap-t" => {
            let s = h / 4.0;
            poly(&[-w / 2.0, -h / 2.0, w / 2.0, -h / 2.0, w / 2.0 - s, h / 2.0, -w / 2.0 + s, h / 2.0], a)
        }
        "text" => rect(w, h, " fill=\"none\" stroke=\"none\"", ""),
        "f-circ" => svg_format!("<circle r=\"", num(w / 2.0), "\" fill=\"", lcolor, "\"", line, "/>"),
        "fr-circ" => {
            svg_format!("<circle r=\"", num(w / 2.0), "\"", a, "/><circle r=\"", num(w / 2.0 - 4.0), "\" fill=\"", lcolor, "\"", line, "/>")
        }
        "fork" => rect(w, h, &svg_format!(" fill=\"", lcolor, "\"", line), " rx=\"2\""),
        "notch-rect" => {
            let c = 10.0;
            poly(&[-w / 2.0 + c, -h / 2.0, w / 2.0, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0, h / 2.0, -w / 2.0, -h / 2.0 + c], a)
        }
        "lin-rect" => {
            svg_format!(
                rr(w, h, a),
                "<path d=\"M",
                num(-w / 2.0 + 8.0),
                ",",
                num(-h / 2.0),
                "V",
                num(h / 2.0),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "div-rect" => {
            svg_format!(
                rr(w, h, a),
                "<path d=\"M",
                num(-w / 2.0),
                ",",
                num(-h / 2.0 + 8.0),
                "H",
                num(w / 2.0),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "win-pane" => svg_format!(
            rr(w, h, a),
            "<path d=\"M",
            num(-w / 2.0 + 8.0),
            ",",
            num(-h / 2.0),
            "V",
            num(h / 2.0),
            "M",
            num(-w / 2.0),
            ",",
            num(-h / 2.0 + 8.0),
            "H",
            num(w / 2.0),
            "\" fill=\"none\"",
            line,
            "/>"
        ),
        "tri" => poly(&[0.0, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0, h / 2.0], a),
        "flip-tri" => poly(&[-w / 2.0, -h / 2.0, w / 2.0, -h / 2.0, 0.0, h / 2.0], a),
        "sl-rect" => poly(&[-w / 2.0, -h / 2.0 + 10.0, w / 2.0, -h / 2.0, w / 2.0, h / 2.0, -w / 2.0, h / 2.0], a),
        "notch-pent" => {
            let c = 10.0;
            poly(
                &[
                    -w / 2.0 + c,
                    -h / 2.0,
                    w / 2.0 - c,
                    -h / 2.0,
                    w / 2.0,
                    -h / 2.0 + c,
                    w / 2.0,
                    h / 2.0,
                    -w / 2.0,
                    h / 2.0,
                    -w / 2.0,
                    -h / 2.0 + c,
                ],
                a,
            )
        }
        "hourglass" => poly(&[-w / 2.0, -h / 2.0, w / 2.0, -h / 2.0, -w / 2.0, h / 2.0, w / 2.0, h / 2.0], a),
        "delay" => {
            let r = h / 2.0;
            svg_format!(
                "<path d=\"M",
                num(-w / 2.0),
                ",",
                num(-r),
                "H",
                num(w / 2.0 - r),
                "A",
                num(r),
                ",",
                num(r),
                " 0 0 1 ",
                num(w / 2.0 - r),
                ",",
                num(r),
                "H",
                num(-w / 2.0),
                "Z\"",
                a,
                "/>"
            )
        }
        "h-cyl" => {
            let ry = h / 2.0;
            let rx = js::min(12.0, h / 4.0);
            let l = -w / 2.0 + rx;
            let r = w / 2.0 - rx;
            svg_format!(
                "<path d=\"M",
                num(l),
                ",",
                num(-ry),
                "H",
                num(r),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 1 ",
                num(r),
                ",",
                num(ry),
                "H",
                num(l),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 1 ",
                num(l),
                ",",
                num(-ry),
                "Z\"",
                a,
                "/><path d=\"M",
                num(r),
                ",",
                num(-ry),
                "A",
                num(rx),
                ",",
                num(ry),
                " 0 0 0 ",
                num(r),
                ",",
                num(ry),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "cross-circ" => {
            let r = w / 2.0;
            let k = r * std::f64::consts::FRAC_1_SQRT_2;
            svg_format!(
                "<circle r=\"",
                num(r),
                "\"",
                a,
                "/><path d=\"M",
                num(-k),
                ",",
                num(-k),
                "L",
                num(k),
                ",",
                num(k),
                "M",
                num(k),
                ",",
                num(-k),
                "L",
                num(-k),
                ",",
                num(k),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "st-rect" => {
            let bw = w - 8.0;
            let bh = h - 8.0;
            let mut out = String::new();
            for i in 0..3 {
                let x = -w / 2.0 + (2 - i) as f64 * 4.0;
                let y = -h / 2.0 + i as f64 * 4.0;
                svg_write!(
                    out,
                    "<rect x=\"",
                    num(x),
                    "\" y=\"",
                    num(y),
                    "\" width=\"",
                    num(bw),
                    "\" height=\"",
                    num(bh),
                    "\"",
                    ROUNDED,
                    a,
                    "/>"
                )
                .unwrap();
            }
            out
        }
        "doc" => wave(w, h, a),
        "flag" => {
            let x = w / 2.0;
            let y = h / 2.0 - 8.0;
            let q = w / 4.0;
            svg_format!(
                "<path d=\"M",
                num(-x),
                ",",
                num(-y),
                "q",
                num(q),
                ",-10 ",
                num(2.0 * q),
                ",0t",
                num(2.0 * q),
                ",0V",
                num(y),
                "q",
                num(-q),
                ",10 ",
                num(-2.0 * q),
                ",0t",
                num(-2.0 * q),
                ",0Z\"",
                a,
                "/>"
            )
        }
        "datastore" => {
            let stroke = svg_format!("stroke=\"", theming.paint(Role::NodeStroke, "var(--_b)"), "\"");
            svg_format!(
                rect(w, h, &a.replacen(&stroke, "stroke=\"none\"", 1), ""),
                "<path d=\"M",
                num(-w / 2.0),
                ",",
                num(-h / 2.0),
                "H",
                num(w / 2.0),
                "M",
                num(-w / 2.0),
                ",",
                num(h / 2.0),
                "H",
                num(w / 2.0),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "folder" => {
            let x = w / 2.0;
            let y = h / 2.0;
            let tab = js::min(w * 0.4, 48.0);
            path(
                &svg_format!(
                    "M",
                    num(-x),
                    ",",
                    num(-y),
                    "H",
                    num(-x + tab),
                    "L",
                    num(-x + tab + 8.0),
                    ",",
                    num(-y + 8.0),
                    "H",
                    num(x),
                    "V",
                    num(y),
                    "H",
                    num(-x),
                    "Z"
                ),
                a,
            )
        }
        "bucket" => {
            let x = w / 2.0;
            let y = h / 2.0;
            let ry = 5.0;
            let taper = h / 6.0;
            path(
                &svg_format!(
                    "M",
                    num(-x),
                    ",",
                    num(-y + ry),
                    "A",
                    num(x),
                    ",",
                    num(ry),
                    " 0 0 1 ",
                    num(x),
                    ",",
                    num(-y + ry),
                    "L",
                    num(x - taper),
                    ",",
                    num(y - ry),
                    "A",
                    num(x - taper),
                    ",",
                    num(ry),
                    " 0 0 1 ",
                    num(-x + taper),
                    ",",
                    num(y - ry),
                    "Z"
                ),
                a,
            ) + &svg_format!(
                "<path d=\"M",
                num(-x),
                ",",
                num(-y + ry),
                "A",
                num(x),
                ",",
                num(ry),
                " 0 0 0 ",
                num(x),
                ",",
                num(-y + ry),
                "\" fill=\"none\"",
                line,
                "/>"
            )
        }
        "console" => svg_format!(
            rr(w, h, a),
            "<path d=\"M",
            num(-w / 2.0),
            ",",
            num(-h / 2.0 + 12.0),
            "H",
            num(w / 2.0),
            "M",
            num(-w / 2.0 + 7.0),
            ",",
            num(-h / 2.0 + 4.0),
            "l3,2.5l-3,2.5M",
            num(-w / 2.0 + 13.0),
            ",",
            num(-h / 2.0 + 9.0),
            "h4\" fill=\"none\"",
            line,
            "/>"
        ),
        "browser" => {
            let mut dots = String::new();
            for i in 0..3 {
                svg_write!(
                    dots,
                    "<circle cx=\"",
                    num(-w / 2.0 + 8.0 + i as f64 * 7.0),
                    "\" cy=\"",
                    num(-h / 2.0 + 6.0),
                    "\" r=\"1.5\" fill=\"none\"",
                    line,
                    "/>"
                )
                .unwrap();
            }
            svg_format!(
                rr(w, h, a),
                "<path d=\"M",
                num(-w / 2.0),
                ",",
                num(-h / 2.0 + 12.0),
                "H",
                num(w / 2.0),
                "\" fill=\"none\"",
                line,
                "/>",
                dots
            )
        }
        "person" => {
            let top = -h / 2.0;
            svg_format!(
                "<rect x=\"",
                num(-w / 2.0),
                "\" y=\"",
                num(top + 26.0),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(h - 26.0),
                "\" rx=\"12\"",
                a,
                "/><circle cy=\"",
                num(top + 14.0),
                "\" r=\"13\"",
                a,
                "/>"
            )
        }
        "bang" => {
            let spikes = 14;
            let mut pts: Vec<f64> = Vec::with_capacity(spikes * 4);
            for i in 0..spikes * 2 {
                let angle = (std::f64::consts::PI * i as f64) / spikes as f64 - std::f64::consts::PI / 2.0;
                let k = if i % 2 == 0 { 1.0 } else { 0.82 };
                pts.push(math::cos(angle) * (w / 2.0) * k);
                pts.push(math::sin(angle) * (h / 2.0) * k);
            }
            poly(&pts, a)
        }
        "cloud" => {
            let x = w / 2.0;
            let y = h / 2.0;
            let r = y * 0.55;
            path(
                &svg_format!(
                    "M",
                    num(-x + r),
                    ",",
                    num(y),
                    "A",
                    num(r),
                    ",",
                    num(r),
                    " 0 0 1 ",
                    num(-x + r * 0.6),
                    ",",
                    num(-y * 0.1),
                    "A",
                    num(x * 0.42),
                    ",",
                    num(y * 0.7),
                    " 0 0 1 ",
                    num(-x * 0.1),
                    ",",
                    num(-y * 0.72),
                    "A",
                    num(x * 0.4),
                    ",",
                    num(y * 0.62),
                    " 0 0 1 ",
                    num(x * 0.62),
                    ",",
                    num(-y * 0.42),
                    "A",
                    num(r * 1.05),
                    ",",
                    num(r * 1.05),
                    " 0 0 1 ",
                    num(x - r),
                    ",",
                    num(y),
                    "Z"
                ),
                a,
            )
        }
        "brace" => rect(w, h, " fill=\"none\" stroke=\"none\"", "") + &brace(-w / 2.0 + 12.0, h, -1.0, line),
        "brace-r" => rect(w, h, " fill=\"none\" stroke=\"none\"", "") + &brace(w / 2.0 - 12.0, h, 1.0, line),
        "braces" => {
            rect(w, h, " fill=\"none\" stroke=\"none\"", "") + &brace(-w / 2.0 + 12.0, h, -1.0, line) + &brace(w / 2.0 - 12.0, h, 1.0, line)
        }
        "bolt" => poly(
            &[w * 0.15, -h / 2.0, -w / 2.0, h * 0.08, -w * 0.05, h * 0.08, -w * 0.15, h / 2.0, w / 2.0, -h * 0.08, w * 0.05, -h * 0.08],
            a,
        ),
        "lin-cyl" => {
            let rx = w / 2.0;
            let ry = js::min(12.0, js::max(6.0, w / 14.0));
            let top = -h / 2.0 + ry;
            let bottom = h / 2.0 - ry;
            let arc = |y: f64| svg_format!("M", num(-rx), ",", num(y), "A", num(rx), ",", num(ry), " 0 0 0 ", num(rx), ",", num(y));
            path(
                &svg_format!(
                    "M",
                    num(-rx),
                    ",",
                    num(top),
                    "A",
                    num(rx),
                    ",",
                    num(ry),
                    " 0 0 1 ",
                    num(rx),
                    ",",
                    num(top),
                    "V",
                    num(bottom),
                    "A",
                    num(rx),
                    ",",
                    num(ry),
                    " 0 0 1 ",
                    num(-rx),
                    ",",
                    num(bottom),
                    "Z"
                ),
                a,
            ) + &svg_format!("<path d=\"", arc(top), arc(top + 8.0), "\" fill=\"none\"", line, "/>")
        }
        "curv-trap" => {
            let x = w / 2.0;
            let y = h / 2.0;
            let k = h * 0.3;
            path(
                &svg_format!(
                    "M",
                    num(-x + k),
                    ",",
                    num(-y),
                    "H",
                    num(x - k),
                    "A",
                    num(k),
                    ",",
                    num(y),
                    " 0 0 1 ",
                    num(x - k),
                    ",",
                    num(y),
                    "H",
                    num(-x + k),
                    "L",
                    num(-x),
                    ",0Z"
                ),
                a,
            )
        }
        "docs" => {
            let mut out = String::new();
            for i in 0..3i32 {
                svg_write!(out, "<g transform=\"translate(", (1 - i) * 4, ",", (i - 1) * 4, ")\">", wave(w - 8.0, h - 8.0, a), "</g>")
                    .unwrap();
            }
            out
        }
        "bow-rect" => {
            let x = w / 2.0;
            let y = h / 2.0;
            let k = h * 0.2;
            path(
                &svg_format!(
                    "M",
                    num(-x + k),
                    ",",
                    num(-y),
                    "H",
                    num(x),
                    "A",
                    num(k),
                    ",",
                    num(y),
                    " 0 0 0 ",
                    num(x),
                    ",",
                    num(y),
                    "H",
                    num(-x + k),
                    "A",
                    num(k),
                    ",",
                    num(y),
                    " 0 0 1 ",
                    num(-x + k),
                    ",",
                    num(-y),
                    "Z"
                ),
                a,
            )
        }
        "tag-rect" => svg_format!(
            rr(w, h, a),
            "<path d=\"M",
            num(w / 2.0 - 12.0),
            ",",
            num(h / 2.0),
            "L",
            num(w / 2.0),
            ",",
            num(h / 2.0 - 12.0),
            "\" fill=\"none\"",
            line,
            "/>"
        ),
        "tag-doc" => svg_format!(
            wave(w, h, a),
            "<path d=\"M",
            num(w / 2.0 - 12.0),
            ",",
            num(h / 2.0 - 14.0),
            "L",
            num(w / 2.0),
            ",",
            num(h / 2.0 - 26.0),
            "\" fill=\"none\"",
            line,
            "/>"
        ),
        "lin-doc" => svg_format!(
            wave(w, h, a),
            "<path d=\"M",
            num(-w / 2.0 + 8.0),
            ",",
            num(-h / 2.0),
            "V",
            num(h / 2.0 - 6.0),
            "\" fill=\"none\"",
            line,
            "/>"
        ),
        COLLAPSED => svg_format!(
            "<rect x=\"",
            num(-w / 2.0 + 6.0),
            "\" y=\"",
            num(-h / 2.0 + 8.0),
            "\" width=\"",
            num(w - 12.0),
            "\" height=\"",
            num(h - 8.0),
            "\"",
            ROUNDED,
            a,
            "/><rect x=\"",
            num(-w / 2.0),
            "\" y=\"",
            num(-h / 2.0),
            "\" width=\"",
            num(w),
            "\" height=\"",
            num(h - 4.0),
            "\"",
            ROUNDED,
            a,
            "/>"
        ),
        _ => rr(w, h, a),
    }
}

/// Distance from the bounding box to the outline on a side (0 top, 1 right, 2 bottom, 3 left),
/// `t` along the side from its middle.
pub fn shape_inset(shape: &str, w: f64, h: f64, side: i32, t: f64) -> f64 {
    if !is_known(shape) {
        return 0.0;
    }
    match shape {
        "stadium" => {
            if side & 1 != 0 {
                bulge(t, h / 2.0, h / 2.0)
            } else {
                0.0
            }
        }
        "cyl" | "lin-cyl" => cap_inset(w, h, side, t),
        "circle" | "dbl-circ" | "ellipse" | "sm-circ" | "f-circ" | "fr-circ" | "cross-circ" => round_inset(w, h, side, t),
        "diam" => {
            if side & 1 != 0 {
                (t.abs() * w) / h
            } else {
                (t.abs() * h) / w
            }
        }
        "hex" => {
            if side & 1 != 0 {
                t.abs() / 2.0
            } else {
                0.0
            }
        }
        "odd" => {
            if side == 3 {
                h / 4.0
            } else {
                0.0
            }
        }
        "lean-r" | "lean-l" | "trap-b" | "trap-t" => lean_inset(w, h, side),
        "tri" | "flip-tri" | "bolt" => {
            if side & 1 != 0 {
                w / 4.0
            } else {
                0.0
            }
        }
        "sl-rect" => {
            if side == 0 {
                5.0
            } else {
                0.0
            }
        }
        "hourglass" => {
            if side & 1 != 0 {
                w / 2.0
            } else {
                0.0
            }
        }
        "doc" | "docs" | "tag-doc" | "lin-doc" => {
            if side == 2 {
                10.0
            } else {
                0.0
            }
        }
        "flag" => {
            if side & 1 != 0 {
                0.0
            } else {
                8.0
            }
        }
        "bucket" => {
            if side & 1 != 0 {
                h / 12.0
            } else {
                0.0
            }
        }
        "bang" => 3.0,
        "cloud" => {
            if side & 1 != 0 {
                2.0
            } else {
                1.0
            }
        }
        "bow-rect" if side == 1 => h * 0.2,
        _ => 0.0,
    }
}

/// How much of a side of the shape edges may spread along, when the flow runs down or, if
/// `sideways`, across.
pub fn shape_span(shape: &str, w: f64, h: f64, sideways: bool) -> f64 {
    let along = if sideways { h } else { w };
    let flat = along - 2.0 * CORNER;
    let flat_across = |cut: f64| if sideways { 0.0 } else { w - 2.0 * cut - 2.0 * CORNER };
    let v = if !is_known(shape) {
        0.0
    } else {
        match shape {
            "rect" | "rounded" | "fr-rect" | "text" | "notch-rect" | "lin-rect" | "div-rect" | "win-pane" | "notch-pent" | "datastore"
            | "console" | "browser" | "tag-rect" => flat,
            "stadium" => {
                if sideways {
                    h * 0.6
                } else {
                    w - h
                }
            }
            "cyl" => {
                if sideways {
                    h - 2.0 * CORNER - 12.0
                } else {
                    w * 0.6
                }
            }
            "lin-cyl" => {
                if sideways {
                    h - 2.0 * CORNER - 20.0
                } else {
                    w * 0.6
                }
            }
            "circle" | "dbl-circ" | "ellipse" | "sm-circ" | "f-circ" | "fr-circ" | "cross-circ" | "diam" => along * 0.6,
            "hex" => {
                if sideways {
                    h * 0.5
                } else {
                    w - h / 2.0 - CORNER
                }
            }
            "odd" => flat_across(0.0),
            "lean-r" | "lean-l" | "trap-b" | "trap-t" => flat_across(h / 4.0),
            "fork" => along - 8.0,
            "st-rect" => along - 2.0 * CORNER - 8.0,
            _ => 0.0,
        }
    };
    js::max(0.0, v)
}

/// Whether the shape draws its node's label.
pub fn shape_has_label(shape: &str) -> bool {
    !matches!(shape, "sm-circ" | "f-circ" | "fr-circ" | "fork" | "hourglass" | "cross-circ" | "bolt")
}

/// The side of a node's layout box (center `cx`, `cy`, size `cw` by `ch`) that a route end
/// lies on: 0 top, 1 right, 2 bottom, 3 left, or -1 for none.
pub fn end_side(route: &[f64], at: usize, cx: f64, cy: f64, cw: f64, ch: f64) -> i32 {
    let dx = route[at] - cx;
    let dy = route[at + 1] - cy;
    if (dy.abs() - ch / 2.0).abs() < 0.5 {
        return if dy < 0.0 { 0 } else { 2 };
    }
    if (dx.abs() - cw / 2.0).abs() < 0.5 {
        return if dx < 0.0 { 3 } else { 1 };
    }
    -1
}

/// Moves a route end from a node's layout box onto the outline of the shape drawn in it.
#[allow(clippy::too_many_arguments)]
pub fn inset_route(route: &mut [f64], at: usize, shape: &str, vw: f64, vh: f64, cx: f64, cy: f64, cw: f64, ch: f64, is_group: bool) {
    if is_group {
        return;
    }
    let side = end_side(route, at, cx, cy, cw, ch);
    if side == 0 || side == 2 {
        let amount = shape_inset(shape, vw, vh, side, route[at] - cx) + (ch - vh) / 2.0;
        route[at + 1] += if side == 0 { amount } else { -amount };
    } else if side >= 0 {
        let amount = shape_inset(shape, vw, vh, side, route[at + 1] - cy) + (cw - vw) / 2.0;
        route[at] += if side == 3 { amount } else { -amount };
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_markup() {
        let t = Theming::pele();
        assert_eq!(draw_shape("rect", 10.0, 4.0, " a=\"1\"", "", &t), "<rect x=\"-5\" y=\"-2\" width=\"10\" height=\"4\" rx=\"var(--pele-radius,4px)\" a=\"1\"/>");
        assert_eq!(draw_shape("rounded", 10.0, 4.0, "", "", &t), "<rect x=\"-5\" y=\"-2\" width=\"10\" height=\"4\" rx=\"2\"/>");
        assert_eq!(draw_shape("stadium", 10.0, 5.0, "", "", &t), "<rect x=\"-5\" y=\"-2.5\" width=\"10\" height=\"5\" rx=\"2.5\"/>");
        assert_eq!(draw_shape("diam", 10.0, 4.0, " a", "", &t), "<polygon points=\"0,-2 5,0 0,2 -5,0\" a/>");
    }

    #[test]
    fn fallbacks() {
        assert_eq!(shape_size("nope", 10.0, 10.0), shape_size("rect", 10.0, 10.0));
        assert_eq!(shape_inset("constructor", 10.0, 10.0, 1, 0.0), 0.0);
        assert!(shape_has_label("nope"));
        assert!(!shape_has_label("fork"));
        let t = Theming::pele();
        assert_eq!(draw_shape("nope", 10.0, 10.0, "", "", &t), draw_shape("rect", 10.0, 10.0, "", "", &t));
    }
}
