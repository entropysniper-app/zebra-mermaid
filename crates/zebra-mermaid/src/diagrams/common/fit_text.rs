//! Ported from Pele `src/diagrams/common/fit-text.ts`.

use crate::text::Measure;
use crate::util::js;

#[derive(Debug, Clone, PartialEq)]
pub struct Fitted {
    pub text: String,
    pub size: f64,
    pub width: f64,
}

/// Fits one line of text into a width: first by shrinking the font down to `min`, then by
/// cutting the text and adding an ellipsis. Returns `None` when not even one character fits.
pub fn fit_text(text: &str, max: f64, size: f64, min: f64, m: &Measure, style: u8) -> Option<Fitted> {
    if text.is_empty() || max <= 0.0 {
        return None;
    }
    let mut width = m.width(text, size, style);
    if width <= max {
        return Some(Fitted { text: text.to_string(), size, width });
    }
    // Width grows with font size, so one division lands on the right size or just above it.
    let mut fitted = js::max(min, ((size * max) / width).floor());
    while fitted >= min {
        width = m.width(text, fitted, style);
        if width <= max {
            return Some(Fitted { text: text.to_string(), size: fitted, width });
        }
        fitted -= 1.0;
    }
    let chars: Vec<char> = text.chars().collect();
    let cut_at = |n: usize| -> String {
        let head: String = chars[..n].iter().collect();
        format!("{}…", js::trim_end(&head))
    };
    let mut low = 0usize;
    let mut high = chars.len() - 1;
    while low < high {
        let mid = (low + high + 1) >> 1;
        if m.width(&cut_at(mid), min, style) <= max {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    if low == 0 {
        return None;
    }
    let cut = cut_at(low);
    width = m.width(&cut, min, style);
    if width <= max {
        Some(Fitted { text: cut, size: min, width })
    } else {
        None
    }
}
