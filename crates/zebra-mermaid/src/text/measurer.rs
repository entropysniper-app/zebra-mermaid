//! Text width measurement. Ported from Pele `src/text/measurer.ts`.
//!
//! [`MetricsMeasurer`] estimates widths from a table of Arial advances, as Pele does where no
//! canvas is available (Node, workers). It is an estimate: there is no shaping, kerning or
//! font fallback. A host that has real fonts implements [`TextMeasurer`] itself.

use std::sync::OnceLock;

use super::metrics::{BOLD, REGULAR};

/// Style bits of a span of text, as Pele numbers them.
pub mod style {
    pub const BOLD: u8 = 1;
    pub const ITALIC: u8 = 2;
    pub const MONO: u8 = 4;
    pub const SUB: u8 = 8;
    pub const SUP: u8 = 16;
    pub const UNDERLINE: u8 = 32;
}

/// Measures the advance width of a run of text in pixels.
///
/// `style` is a combination of the [`style`] bits. Implementations must be deterministic for
/// a given input during one render; the layout measures the same text more than once and
/// expects the same answer.
pub trait TextMeasurer {
    fn width(&self, text: &str, size: f64, style: u8) -> f64;
}

impl<F: Fn(&str, f64, u8) -> f64> TextMeasurer for F {
    fn width(&self, text: &str, size: f64, style: u8) -> f64 {
        self(text, size, style)
    }
}

fn unpack(packed: &str) -> [u16; 95] {
    let mut out = [0u16; 95];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u16::from_str_radix(&packed[i * 2..i * 2 + 2], 36).expect("base 36");
    }
    out
}

fn regular() -> &'static [u16; 95] {
    static T: OnceLock<[u16; 95]> = OnceLock::new();
    T.get_or_init(|| unpack(REGULAR))
}

fn bold() -> &'static [u16; 95] {
    static T: OnceLock<[u16; 95]> = OnceLock::new();
    T.get_or_init(|| unpack(BOLD))
}

// Both tests take a UTF-16 code unit: a character outside the BMP is a high surrogate (wide)
// followed by a low surrogate (no width), as Pele reads it.
fn is_wide(c: u16) -> bool {
    (0x1100..=0x115f).contains(&c)
        || (0x2e80..=0xa4cf).contains(&c)
        || (0xac00..=0xd7a3).contains(&c)
        || (0xf900..=0xfaff).contains(&c)
        || (0xfe30..=0xfe4f).contains(&c)
        || (0xff00..=0xff60).contains(&c)
        || (0xffe0..=0xffe6).contains(&c)
        || (0xd800..=0xdbff).contains(&c)
}

fn is_zero_width(c: u16) -> bool {
    c == 0xad
        || (0x300..=0x36f).contains(&c)
        || (0x200b..=0x200f).contains(&c)
        || (0xdc00..=0xdfff).contains(&c)
        || (0xfe00..=0xfe0f).contains(&c)
        || c == 0xfeff
}

/// Pele's `metricsMeasurer`: Arial advance widths for ASCII, a full em for CJK and other wide
/// characters, nothing for combining marks and zero-width characters, and 0.58 em otherwise.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsMeasurer;

impl TextMeasurer for MetricsMeasurer {
    fn width(&self, text: &str, size: f64, style: u8) -> f64 {
        let mono = style & style::MONO != 0;
        let table = if style & style::BOLD != 0 { bold() } else { regular() };
        let mut units: f64 = 0.0;
        if text.is_ascii() {
            for &b in text.as_bytes() {
                if b < 32 {
                    continue;
                }
                if b < 127 {
                    units += if mono { 600.0 } else { table[(b - 32) as usize] as f64 };
                } else {
                    // DEL is neither printable nor wide.
                    units += if mono { 600.0 } else { 580.0 };
                }
            }
        } else {
            for c in text.encode_utf16() {
                if c < 32 {
                    continue;
                }
                if c < 127 {
                    units += if mono { 600.0 } else { table[(c - 32) as usize] as f64 };
                } else if is_zero_width(c) {
                    continue;
                } else if is_wide(c) {
                    units += if mono { 1200.0 } else { 1000.0 };
                } else {
                    units += if mono { 600.0 } else { 580.0 };
                }
            }
        }
        let effective = if style & (style::SUB | style::SUP) != 0 { size / 1.2 } else { size };
        (units * effective) / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_follow_the_table() {
        let m = MetricsMeasurer;
        // "A" is 667 thousandths in Arial.
        assert_eq!(m.width("A", 1000.0, 0), 667.0);
        assert_eq!(m.width("中", 16.0, 0), 16.0);
        assert_eq!(m.width("中", 10.0, style::MONO), 12.0);
        assert_eq!(m.width("😀", 10.0, 0), 10.0);
        assert_eq!(m.width("e\u{301}", 1000.0, 0), m.width("e", 1000.0, 0));
        assert_eq!(m.width("\u{7f}", 1000.0, 0), 580.0);
        assert_eq!(m.width("\n\t", 16.0, 0), 0.0);
    }
}
