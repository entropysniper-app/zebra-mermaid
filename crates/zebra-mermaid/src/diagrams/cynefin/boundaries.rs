//! The geometry of Mermaid's Cynefin boundaries: seeded wavy lines between the domains and the
//! S-shaped cliff between Clear and Chaotic. Ported from Pele
//! `src/diagrams/cynefin/boundaries.ts`. Curves are flat lists of numbers: a start point, then
//! six numbers (two control points and an end point) per cubic segment.

use crate::svg::builder::num;
use crate::util::{js, math};
use std::fmt::Write as _;

/// mulberry32, one step, with JavaScript's integer conversions: `|` and `^` take ToInt32 of
/// their operands, `>>>` ToUint32, and `+` adds doubles.
pub fn seeded_random(seed: f64) -> f64 {
    // `(seed + 0x6d2b79f5) | 0`: the seed may be any double, a fraction included.
    let mut t = js::to_int32(seed + 1831565813.0);
    t = math::imul(t ^ ((t as u32) >> 15) as i32, t | 1);
    // `t + Math.imul(…)` is a double sum of two int32 values; ToInt32 of it wraps.
    t ^= t.wrapping_add(math::imul(t ^ ((t as u32) >> 7) as i32, t | 61));
    ((t ^ ((t as u32) >> 14) as i32) as u32) as f64 / 4294967296.0
}

/// Java's string hash over UTF-16 code units, kept to 32 bits after every step.
pub fn hash_string(s: &str) -> i32 {
    let mut hash: i32 = 0;
    for unit in s.encode_utf16() {
        // `(hash << 5) - hash + code`, exact in doubles, then `| 0`.
        hash = (i64::from(hash.wrapping_shl(5)) - i64::from(hash) + i64::from(unit)) as i32;
    }
    hash
}

/// A configured seed other than zero wins; otherwise the seed comes from the diagram's id.
pub fn resolve_seed(configured: f64, id: &str) -> f64 {
    if configured.is_finite() && configured != 0.0 {
        return configured;
    }
    f64::from(hash_string(id))
}

#[derive(Debug, Clone, Copy)]
pub struct WaveOptions {
    pub segments: usize,
    pub vertical: bool,
    /// Multipliers Mermaid uses to vary the seed per point and per control point.
    pub point_step: f64,
    pub control_step: f64,
    pub control_shift: f64,
    pub pin_start: bool,
    pub pin_end: bool,
}

pub const FOLD: WaveOptions =
    WaveOptions { segments: 7, vertical: true, point_step: 17.0, control_step: 31.0, control_shift: 7.0, pin_start: false, pin_end: false };
pub const HORIZON: WaveOptions = WaveOptions {
    segments: 7,
    vertical: false,
    point_step: 23.0,
    control_step: 37.0,
    control_shift: 11.0,
    pin_start: false,
    pin_end: false,
};

/// A wavy line along one axis from `from` to `to`, centered on `center` in the other axis.
/// Points jitter by up to the amplitude; a pinned end stays on the center line.
pub fn wave(from: f64, to: f64, center: f64, seed: f64, amplitude: f64, o: WaveOptions) -> Vec<f64> {
    let step = (to - from) / o.segments as f64;
    let mut along: Vec<f64> = Vec::with_capacity(o.segments + 1);
    let mut across: Vec<f64> = Vec::with_capacity(o.segments + 1);
    for i in 0..=o.segments {
        let pinned = (i == 0 && o.pin_start) || (i == o.segments && o.pin_end);
        along.push(from + i as f64 * step);
        across.push(if pinned { center } else { center + seeded_random(seed + i as f64 * o.point_step) * amplitude * 2.0 - amplitude });
    }
    let mut out = if o.vertical { vec![across[0], along[0]] } else { vec![along[0], across[0]] };
    for i in 0..o.segments {
        let mid = (along[i] + along[i + 1]) / 2.0;
        let turn = if i.is_multiple_of(2) { 1.0 } else { -1.0 };
        let offset = amplitude * 1.5 * turn * seeded_random(seed + i as f64 * o.control_step + o.control_shift);
        if o.vertical {
            out.extend([across[i] + offset, mid, across[i + 1] - offset, mid, across[i + 1], along[i + 1]]);
        } else {
            out.extend([mid, across[i] + offset, mid, across[i + 1] - offset, along[i + 1], across[i + 1]]);
        }
    }
    out
}

/// The cliff: a steep S from (cx, top) down to (cx, bottom).
pub fn cliff(cx: f64, top: f64, bottom: f64, amplitude: f64) -> Vec<f64> {
    let h = bottom - top;
    vec![
        cx,
        top,
        cx + amplitude,
        top + h * 0.2,
        cx - amplitude * 1.5,
        top + h * 0.55,
        cx + amplitude * 0.5,
        top + h * 0.75,
        cx - amplitude,
        top + h * 0.85,
        cx + amplitude * 0.3,
        top + h * 0.95,
        cx,
        bottom,
    ]
}

/// Path commands that continue from the curve's start point to its end.
pub fn forward(curve: &[f64]) -> String {
    let mut d = String::new();
    let mut i = 2;
    while i + 5 < curve.len() {
        write!(
            d,
            "C{},{} {},{} {},{}",
            num(curve[i]),
            num(curve[i + 1]),
            num(curve[i + 2]),
            num(curve[i + 3]),
            num(curve[i + 4]),
            num(curve[i + 5])
        )
        .unwrap();
        i += 6;
    }
    d
}

/// Path commands that continue from the curve's end point back to its start.
pub fn backward(curve: &[f64]) -> String {
    let mut d = String::new();
    let mut i = curve.len() as isize - 6;
    while i >= 2 {
        let k = i as usize;
        write!(
            d,
            "C{},{} {},{} {},{}",
            num(curve[k + 2]),
            num(curve[k + 3]),
            num(curve[k]),
            num(curve[k + 1]),
            num(curve[k - 2]),
            num(curve[k - 1])
        )
        .unwrap();
        i -= 6;
    }
    d
}

pub fn curve_path(curve: &[f64]) -> String {
    format!("M{},{}{}", num(curve[0]), num(curve[1]), forward(curve))
}

pub fn ellipse_path(cx: f64, cy: f64, rx: f64, ry: f64) -> String {
    let r = format!("{},{}", num(rx), num(ry));
    format!("M{},{}A{} 0 1,1 {},{}A{} 0 1,1 {},{}Z", num(cx - rx), num(cy), r, num(cx + rx), num(cy), r, num(cx - rx), num(cy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulberry32_matches_javascript() {
        // Values from node, running Pele's seededRandom.
        assert_eq!(seeded_random(0.0), 0.26642920868471265);
        assert_eq!(seeded_random(1.0), 0.6270739405881613);
        assert_eq!(seeded_random(-1.0), 0.8964226141106337);
        assert_eq!(seeded_random(3433098.0), 0.533290577121079);
        assert_eq!(seeded_random(12345.678), 0.9797282677609473);
        assert_eq!(seeded_random(1e15), 0.4975225117523223);
        assert_eq!(seeded_random(-1e15), 0.30435564112849534);
        assert_eq!(seeded_random(2147483647.5), 0.4290980885270983);
        assert_eq!(seeded_random(-3000000000.25), 0.3323475457727909);
    }

    #[test]
    fn hashes_utf16_units() {
        assert_eq!(hash_string("pele"), 3437102);
        assert_eq!(hash_string(""), 0);
        assert_eq!(hash_string("a-much-longer-identifier-\u{1F600}"), -1223410249);
        assert_eq!(hash_string("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"), 700540986);
    }
}
