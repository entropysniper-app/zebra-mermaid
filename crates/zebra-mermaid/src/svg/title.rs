//! A diagram title above content laid out without one. Ported from Pele `src/svg/title.ts`.

use super::builder::{label_svg, num};
use crate::svg::concat::svg_format;
use crate::text::{layout_label, style, Measure};

pub struct Titled {
    pub width: f64,
    pub height: f64,
    pub content: String,
}

/// Puts a diagram title above content that was laid out without one.
pub fn with_title(title: Option<&str>, m: &Measure, size: f64, pad: f64, width: f64, height: f64, content: String) -> Titled {
    let label = layout_label(title, false, m, size, 4000.0, style::BOLD);
    if label.height == 0.0 {
        return Titled { width, height, content };
    }
    let shift = label.height + 12.0;
    let full = crate::util::js::max(width, (label.width + 2.0 * pad).ceil());
    let content = svg_format!(
        label_svg(&label, full / 2.0, pad + label.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None),
        "<g transform=\"translate(",
        num((full - width) / 2.0),
        ",",
        num(shift),
        ")\">",
        content,
        "</g>"
    );
    Titled { width: full, height: (height + shift).ceil(), content }
}
