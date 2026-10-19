//! SVG text building blocks: escaping, coordinates and labels. Ported from Pele
//! `src/svg/builder.rs`.

use std::borrow::Cow;
use std::fmt;

use crate::text::entities::decode_entities;
use crate::text::{style, Icons, Label};
use crate::util::js;

#[inline]
fn needs_esc(b: u8) -> bool {
    matches!(b, b'&' | b'<' | b'>' | b'"' | 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f)
}

/// Escapes text for XML and drops the control characters XML does not allow.
pub fn esc(text: &str) -> Cow<'_, str> {
    if !text.bytes().any(needs_esc) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut last = 0;
    for (i, b) in text.bytes().enumerate() {
        if !needs_esc(b) {
            continue;
        }
        out.push_str(&text[last..i]);
        match b {
            b'&' => out.push_str("&amp;"),
            b'<' => out.push_str("&lt;"),
            b'>' => out.push_str("&gt;"),
            b'"' => out.push_str("&quot;"),
            _ => {}
        }
        last = i + 1;
    }
    out.push_str(&text[last..]);
    Cow::Owned(out)
}

/// Escapes text that is only ever read as text: a tooltip, an accessible name, a `data-id`.
/// Entity codes are turned into their characters first. A value that was checked before it is
/// written, such as a style, a colour or a URL, must go through [`esc`] instead, so that what
/// was checked is what is written.
pub fn esc_text(text: &str) -> String {
    esc(&decode_entities(text)).into_owned()
}

/// A coordinate written with at most two decimals, as Pele's `num()` writes it.
#[derive(Clone, Copy, Debug)]
pub struct Num(pub f64);

/// Formats a coordinate: rounded to hundredths (half-way up), with no trailing zeros.
#[inline]
pub fn num(value: f64) -> Num {
    Num(value)
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = js::round(self.0 * 100.0);
        if n == 0.0 {
            return f.write_str("0");
        }
        let (neg, m) = if n > 0.0 {
            if n < 1e11 {
                (false, n)
            } else {
                return f.write_str(&js::to_string(n / 100.0));
            }
        } else if n > -1e11 {
            (true, -n)
        } else {
            return f.write_str(&js::to_string(n / 100.0));
        };
        let q = (m / 100.0).floor();
        let frac = (m - q * 100.0) as i64;
        if neg {
            f.write_str("-")?;
        }
        write!(f, "{}", q as i64)?;
        if frac != 0 {
            if frac % 10 == 0 {
                write!(f, ".{}", frac / 10)?;
            } else {
                write!(f, ".{:02}", frac)?;
            }
        }
        Ok(())
    }
}

impl Num {
    pub fn string(self) -> String {
        self.to_string()
    }
}

const BOLD: &str = " font-weight=\"var(--_w)\"";
const ITALIC: &str = " font-style=\"italic\"";

pub fn span_style(st: u8) -> &'static str {
    match (st & style::BOLD != 0, st & style::ITALIC != 0) {
        (true, true) => " font-weight=\"var(--_w)\" font-style=\"italic\"",
        (true, false) => BOLD,
        (false, true) => ITALIC,
        (false, false) => "",
    }
}

/// The emphasis every span of a label has. It is written once, on the text element, where a
/// CSS rule for the label reaches it.
pub fn shared_style(label: &Label) -> u8 {
    let mut shared = style::BOLD | style::ITALIC;
    let mut any = false;
    for line in &label.lines {
        for span in line {
            if span.icon.is_some() || span.text.is_empty() {
                continue;
            }
            shared &= span.style;
            any = true;
        }
    }
    if any {
        shared
    } else {
        0
    }
}

/// Adds that emphasis to a text element's attributes, unless they give a weight of their own.
pub fn text_style(attrs: &str, shared: u8) -> String {
    let mut out = attrs.to_string();
    if shared & style::BOLD != 0 && !attrs.contains(" font-weight=\"") {
        out.push_str(BOLD);
    }
    if shared & style::ITALIC != 0 && !attrs.contains(" font-style=\"") {
        out.push_str(ITALIC);
    }
    out
}

/// Writes a measured label as SVG text centered on (cx, cy).
pub fn label_svg(label: &Label, cx: f64, cy: f64, attrs: &str, icons: Option<&Icons>) -> String {
    let lines = &label.lines;
    if lines.is_empty() {
        return String::new();
    }
    let lh = label.line_height;
    let shift = label.size * 0.35;
    let mut y = cy - label.height / 2.0 + lh / 2.0 + shift;

    if lines.len() == 1 && lines[0].len() == 1 && lines[0][0].icon.is_none() && lines[0][0].style == 0 {
        return format!("<text{} x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>", attrs, num(cx), num(y), esc(&lines[0][0].text));
    }

    let shared = shared_style(label);
    let attrs = text_style(attrs, shared);
    let mut body = String::new();
    let mut extra = String::new();
    for (i, line) in lines.iter().enumerate() {
        let has_icon = line.iter().any(|s| s.icon.is_some());
        if has_icon {
            let mut x = cx - label.widths[i] / 2.0;
            for span in line {
                if let Some(icon) = &span.icon {
                    let side = label.size * 1.1;
                    let inner = icons.map(|i| i.markup(icon)).unwrap_or_else(|| "".into());
                    extra.push_str(&format!(
                        "<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
                        esc_text(icon),
                        num(x),
                        num(y - shift - side / 2.0),
                        num(side),
                        num(side),
                        inner
                    ));
                } else if !span.text.is_empty() {
                    body.push_str(&format!(
                        "<tspan x=\"{}\" y=\"{}\" text-anchor=\"start\"{}>{}</tspan>",
                        num(x),
                        num(y),
                        span_style(span.style & !shared),
                        esc(&span.text)
                    ));
                }
                x += span.width;
            }
        } else {
            for (j, span) in line.iter().enumerate() {
                let pos = if j == 0 { format!(" x=\"{}\" y=\"{}\"", num(cx), num(y)) } else { String::new() };
                body.push_str(&format!("<tspan{}{}>{}</tspan>", pos, span_style(span.style & !shared), esc(&span.text)));
            }
            if line.is_empty() {
                body.push_str(&format!("<tspan x=\"{}\" y=\"{}\"> </tspan>", num(cx), num(y)));
            }
        }
        y += lh;
    }
    format!("<text{} text-anchor=\"middle\" xml:space=\"preserve\">{}</text>{}", attrs, body, extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_have_two_decimals() {
        assert_eq!(num(0.0).to_string(), "0");
        assert_eq!(num(-0.004).to_string(), "0");
        assert_eq!(num(1.0).to_string(), "1");
        assert_eq!(num(1.5).to_string(), "1.5");
        assert_eq!(num(1.25).to_string(), "1.25");
        assert_eq!(num(1.005).to_string(), "1");
        assert_eq!(num(-1.005).to_string(), "-1");
        // -2.345 * 100 is -234.50000000000003, which rounds to -235 in JavaScript as well.
        assert_eq!(num(-2.345).to_string(), "-2.35");
        assert_eq!(num(12.3).to_string(), "12.3");
        assert_eq!(num(0.07).to_string(), "0.07");
        assert_eq!(num(-0.5).to_string(), "-0.5");
        assert_eq!(num(1e10).to_string(), "10000000000");
        assert_eq!(num(f64::NAN).to_string(), "NaN");
        assert_eq!(num(f64::INFINITY).to_string(), "Infinity");
        assert_eq!(num(-f64::INFINITY).to_string(), "-Infinity");
    }

    #[test]
    fn escaping() {
        assert_eq!(esc("a<b>&\"c\""), "a&lt;b&gt;&amp;&quot;c&quot;");
        assert_eq!(esc("x\u{1}\u{8}y\tz\n"), "xy\tz\n");
        assert_eq!(esc("中文'"), "中文'");
        assert_eq!(esc_text("#quot;&amp;<"), "#quot;&amp;&lt;");
    }
}
