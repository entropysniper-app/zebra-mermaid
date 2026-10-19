//! Block arrows. Ported from Pele `src/diagrams/block/arrows.ts`.

use crate::svg::builder::num;
use crate::util::js;
use std::fmt::Write as _;

pub(super) const RIGHT: u32 = 1;
pub(super) const LEFT: u32 = 2;
pub(super) const UP: u32 = 4;
pub(super) const DOWN: u32 = 8;

/// How far a head reaches past each side of its shaft.
pub(super) const OVERHANG: f64 = 7.0;
const MAX_DEPTH: f64 = 20.0;
const NUB: f64 = 12.0;

pub(super) struct ArrowSize {
    pub w: f64,
    pub h: f64,
    /// Shift of the label from the center, away from the heads.
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    X,
    Y,
    None,
}

/// Mermaid compares directions exactly, so `( up )` draws no arrow there. Pele ignores the
/// spaces.
pub(super) fn arrow_mask(directions: Option<&[String]>) -> u32 {
    let mut mask = 0;
    for direction in directions.unwrap_or(&[]) {
        match js::trim(direction) {
            "right" => mask |= RIGHT,
            "left" => mask |= LEFT,
            "up" => mask |= UP,
            "down" => mask |= DOWN,
            "x" => mask |= RIGHT | LEFT,
            "y" => mask |= UP | DOWN,
            _ => {}
        }
    }
    mask
}

fn bit(mask: u32, flag: u32) -> f64 {
    if mask & flag != 0 {
        1.0
    } else {
        0.0
    }
}

/// An arrow along one axis has a shaft and heads wider than it. One that points along both
/// axes is a box with a small point on each side it names.
pub(super) fn arrow_axis(mask: u32) -> Axis {
    if mask != 0 && (mask & (UP | DOWN)) == 0 {
        return Axis::X;
    }
    if mask != 0 && (mask & (RIGHT | LEFT)) == 0 {
        return Axis::Y;
    }
    Axis::None
}

pub(super) fn arrow_size(mask: u32, tw: f64, th: f64) -> ArrowSize {
    let axis = arrow_axis(mask);
    let across = bit(mask, LEFT) - bit(mask, RIGHT);
    let along = bit(mask, UP) - bit(mask, DOWN);
    if axis == Axis::X {
        let h = js::max(th + 8.0, 14.0) + 2.0 * OVERHANG;
        let depth = js::min(h / 2.0, MAX_DEPTH);
        let both = if mask == (RIGHT | LEFT) { 2.0 } else { 1.0 };
        return ArrowSize { w: js::max(tw + 12.0, 16.0) + depth * both, h, dx: (across * depth) / 2.0, dy: 0.0 };
    }
    if axis == Axis::Y {
        let w = js::max(tw + 16.0, 14.0) + 2.0 * OVERHANG;
        let depth = js::min(w / 2.0, MAX_DEPTH);
        let both = if mask == (UP | DOWN) { 2.0 } else { 1.0 };
        return ArrowSize { w, h: js::max(th + 8.0, 16.0) + depth * both, dx: 0.0, dy: (along * depth) / 2.0 };
    }
    ArrowSize {
        w: js::max(tw + 16.0, 24.0) + NUB * (bit(mask, LEFT) + bit(mask, RIGHT)),
        h: js::max(th + 8.0, 24.0) + NUB * (bit(mask, UP) + bit(mask, DOWN)),
        dx: (across * NUB) / 2.0,
        dy: (along * NUB) / 2.0,
    }
}

pub(super) fn draw_arrow(mask: u32, w: f64, h: f64, attrs: &str) -> String {
    let axis = arrow_axis(mask);
    let x = w / 2.0;
    let y = h / 2.0;
    let mut p: Vec<f64>;
    if axis == Axis::X {
        let depth = js::min(y, MAX_DEPTH);
        let s = y - OVERHANG;
        let x0 = if mask & LEFT != 0 { -x + depth } else { -x };
        let x1 = if mask & RIGHT != 0 { x - depth } else { x };
        p = vec![x0, -s, x1, -s];
        if mask & RIGHT != 0 {
            p.extend([x1, -y, x, 0.0, x1, y]);
        }
        p.extend([x1, s, x0, s]);
        if mask & LEFT != 0 {
            p.extend([x0, y, -x, 0.0, x0, -y]);
        }
    } else if axis == Axis::Y {
        let depth = js::min(x, MAX_DEPTH);
        let s = x - OVERHANG;
        let y0 = if mask & UP != 0 { -y + depth } else { -y };
        let y1 = if mask & DOWN != 0 { y - depth } else { y };
        p = vec![s, y0, s, y1];
        if mask & DOWN != 0 {
            p.extend([x, y1, 0.0, y, -x, y1]);
        }
        p.extend([-s, y1, -s, y0]);
        if mask & UP != 0 {
            p.extend([-x, y0, 0.0, -y, x, y0]);
        }
    } else {
        let x0 = if mask & LEFT != 0 { -x + NUB } else { -x };
        let x1 = if mask & RIGHT != 0 { x - NUB } else { x };
        let y0 = if mask & UP != 0 { -y + NUB } else { -y };
        let y1 = if mask & DOWN != 0 { y - NUB } else { y };
        let cx = (x0 + x1) / 2.0;
        let cy = (y0 + y1) / 2.0;
        let kx = js::min(10.0, (x1 - x0) / 2.0);
        let ky = js::min(10.0, (y1 - y0) / 2.0);
        p = vec![x0, y0];
        if mask & UP != 0 {
            p.extend([cx - kx, y0, cx, -y, cx + kx, y0]);
        }
        p.extend([x1, y0]);
        if mask & RIGHT != 0 {
            p.extend([x1, cy - ky, x, cy, x1, cy + ky]);
        }
        p.extend([x1, y1]);
        if mask & DOWN != 0 {
            p.extend([cx + kx, y1, cx, y, cx - kx, y1]);
        }
        p.extend([x0, y1]);
        if mask & LEFT != 0 {
            p.extend([x0, cy + ky, -x, cy, x0, cy - ky]);
        }
    }
    let mut points = String::new();
    let mut i = 0;
    while i + 1 < p.len() {
        if i > 0 {
            points.push(' ');
        }
        write!(points, "{},{}", num(p[i]), num(p[i + 1])).unwrap();
        i += 2;
    }
    format!("<polygon points=\"{}\" stroke-linejoin=\"round\"{}/>", points, attrs)
}
