//! SVG text building blocks: escaping, coordinates and labels. Ported from Pele
//! `src/svg/builder.rs`.

use std::borrow::Cow;
use std::fmt;
use std::fmt::Write as _;

use crate::text::entities::decode_entities_cow;
use crate::text::{style, Icons, Label};
use crate::util::{js, strs};

#[inline]
fn needs_esc(b: u8) -> bool {
    matches!(b, b'&' | b'<' | b'>' | b'"' | 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f)
}

/// `prefix` followed by `extra`: the attributes of an element and what a style adds to them.
/// The prefix is borrowed when there is nothing to add.
pub fn joined<'a>(prefix: &'a str, extra: &str) -> Cow<'a, str> {
    if extra.is_empty() {
        Cow::Borrowed(prefix)
    } else {
        let mut out = String::with_capacity(prefix.len() + extra.len());
        out.push_str(prefix);
        out.push_str(extra);
        Cow::Owned(out)
    }
}

/// Escapes text for XML and drops the control characters XML does not allow.
pub fn esc(text: &str) -> Cow<'_, str> {
    if !text.bytes().any(needs_esc) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 16);
    push_esc(&mut out, text);
    Cow::Owned(out)
}

/// Appends `text` escaped as [`esc`] escapes it.
pub fn push_esc(out: &mut String, text: &str) {
    let _ = write_esc(out, text);
}

/// Writes `text` escaped as [`esc`] escapes it.
pub fn write_esc<W: fmt::Write + ?Sized>(out: &mut W, text: &str) -> fmt::Result {
    let mut last = 0;
    for (i, b) in text.bytes().enumerate() {
        if !needs_esc(b) {
            continue;
        }
        out.write_str(&text[last..i])?;
        match b {
            b'&' => out.write_str("&amp;")?,
            b'<' => out.write_str("&lt;")?,
            b'>' => out.write_str("&gt;")?,
            b'"' => out.write_str("&quot;")?,
            _ => {}
        }
        last = i + 1;
    }
    out.write_str(&text[last..])
}

/// Escapes text that is only ever read as text: a tooltip, an accessible name, a `data-id`.
/// Entity codes are turned into their characters first. A value that was checked before it is
/// written, such as a style, a colour or a URL, must go through [`esc`] instead, so that what
/// was checked is what is written. Text with nothing to decode or escape is borrowed.
pub fn esc_text(text: &str) -> Cow<'_, str> {
    match decode_entities_cow(text) {
        Cow::Borrowed(t) => esc(t),
        Cow::Owned(t) => match esc(&t) {
            Cow::Borrowed(_) => Cow::Owned(t),
            Cow::Owned(escaped) => Cow::Owned(escaped),
        },
    }
}

/// A coordinate written with at most two decimals, as Pele's `num()` writes it.
#[derive(Clone, Copy, Debug)]
pub struct Num(pub f64);

/// Formats a coordinate: rounded to hundredths (half-way up), with no trailing zeros.
#[inline]
pub fn num(value: f64) -> Num {
    Num(value)
}

/// The digits of a coordinate whose rounded magnitude is below 1e11, written from the end of
/// `buf`: the index they start at. `None` for anything else, which takes `js::to_string`.
#[inline]
fn num_digits(value: f64, buf: &mut [u8; 32]) -> Option<usize> {
    let n = js::round(value * 100.0);
    if n == 0.0 {
        buf[31] = b'0';
        return Some(31);
    }
    let (neg, m) = if n > 0.0 {
        if n < 1e11 {
            (false, n)
        } else {
            return None;
        }
    } else if n > -1e11 {
        (true, -n)
    } else {
        return None;
    };
    // The rounded magnitude is an exact integer below 1e11. Write it once from a stack
    // buffer, avoiding nested Formatter calls and tiny String growth steps.
    let cents = m as u64;
    let mut integer = cents / 100;
    let fraction = (cents % 100) as u8;
    let mut start = buf.len();
    if fraction != 0 {
        if fraction % 10 != 0 {
            start -= 1;
            buf[start] = b'0' + fraction % 10;
        }
        start -= 1;
        buf[start] = b'0' + fraction / 10;
        start -= 1;
        buf[start] = b'.';
    }
    loop {
        start -= 1;
        buf[start] = b'0' + (integer % 10) as u8;
        integer /= 10;
        if integer == 0 {
            break;
        }
    }
    if neg {
        start -= 1;
        buf[start] = b'-';
    }
    Some(start)
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0_u8; 32];
        match num_digits(self.0, &mut buf) {
            Some(start) => f.write_str(std::str::from_utf8(&buf[start..]).expect("decimal ASCII")),
            None => f.write_str(&js::to_string(js::round(self.0 * 100.0) / 100.0)),
        }
    }
}

/// Appends a coordinate as [`Num`] writes it, without going through a formatter.
#[inline]
pub fn push_num(out: &mut String, value: f64) {
    let _ = write_num(out, value);
}

/// Writes a coordinate as [`Num`] writes it.
#[inline]
pub fn write_num<W: fmt::Write + ?Sized>(out: &mut W, value: f64) -> fmt::Result {
    let mut buf = [0_u8; 32];
    match num_digits(value, &mut buf) {
        Some(start) => {
            for &b in &buf[start..] {
                out.write_char(b as char)?;
            }
            Ok(())
        }
        None => out.write_str(&js::to_string(js::round(value * 100.0) / 100.0)),
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
/// Attributes that take nothing are borrowed.
pub fn text_style(attrs: &str, shared: u8) -> Cow<'_, str> {
    let bold = shared & style::BOLD != 0 && !strs::contains(attrs, " font-weight=\"");
    let italic = shared & style::ITALIC != 0 && !strs::contains(attrs, " font-style=\"");
    if !bold && !italic {
        return Cow::Borrowed(attrs);
    }
    let mut out = String::with_capacity(attrs.len() + BOLD.len() + ITALIC.len());
    out.push_str(attrs);
    if bold {
        out.push_str(BOLD);
    }
    if italic {
        out.push_str(ITALIC);
    }
    Cow::Owned(out)
}

/// A measured label as SVG text centered on (cx, cy), written when it is formatted. Used as
/// an argument of `format!` or `write!`, it goes straight into the output without a buffer of
/// its own; `to_string` gives the markup as text.
#[derive(Clone, Copy)]
pub struct LabelSvg<'a> {
    label: &'a Label,
    cx: f64,
    cy: f64,
    attrs: &'a str,
    icons: Option<&'a Icons>,
}

impl LabelSvg<'_> {
    /// Whether the label draws nothing.
    pub fn is_empty(&self) -> bool {
        self.label.lines.is_empty()
    }
}

impl fmt::Display for LabelSvg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_label_svg(f, self.label, self.cx, self.cy, self.attrs, self.icons)
    }
}

/// A measured label as SVG text centered on (cx, cy). See [`LabelSvg`].
pub fn label_svg<'a>(label: &'a Label, cx: f64, cy: f64, attrs: &'a str, icons: Option<&'a Icons>) -> LabelSvg<'a> {
    LabelSvg { label, cx, cy, attrs, icons }
}

/// Appends a measured label as SVG text centered on (cx, cy) to a buffer.
pub fn push_label_svg(out: &mut String, label: &Label, cx: f64, cy: f64, attrs: &str, icons: Option<&Icons>) {
    let _ = write_label_svg(out, label, cx, cy, attrs, icons);
}

/// Writes a measured label as SVG text centered on (cx, cy).
pub fn write_label_svg<W: fmt::Write + ?Sized>(
    out: &mut W,
    label: &Label,
    cx: f64,
    cy: f64,
    attrs: &str,
    icons: Option<&Icons>,
) -> fmt::Result {
    let lines = &label.lines;
    if lines.is_empty() {
        return Ok(());
    }
    let lh = label.line_height;
    let shift = label.size * 0.35;
    let mut y = cy - label.height / 2.0 + lh / 2.0 + shift;

    if lines.len() == 1 && lines[0].len() == 1 && lines[0][0].icon.is_none() && lines[0][0].style == 0 {
        out.write_str("<text")?;
        out.write_str(attrs)?;
        out.write_str(" x=\"")?;
        write_num(out, cx)?;
        out.write_str("\" y=\"")?;
        write_num(out, y)?;
        out.write_str("\" text-anchor=\"middle\">")?;
        write_esc(out, &lines[0][0].text)?;
        return out.write_str("</text>");
    }

    let shared = shared_style(label);
    let attrs = text_style(attrs, shared);
    out.write_str("<text")?;
    out.write_str(&attrs)?;
    out.write_str(" text-anchor=\"middle\" xml:space=\"preserve\">")?;
    // Icons are drawn after the text element, so they are kept aside until it is closed.
    let mut extra = String::new();
    for (i, line) in lines.iter().enumerate() {
        let has_icon = line.iter().any(|s| s.icon.is_some());
        if has_icon {
            let mut x = cx - label.widths[i] / 2.0;
            for span in line {
                if let Some(icon) = &span.icon {
                    let side = label.size * 1.1;
                    let inner = icons.map(|i| i.markup(icon)).unwrap_or_else(|| "".into());
                    write!(extra,
                        "<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
                        esc_text(icon),
                        num(x),
                        num(y - shift - side / 2.0),
                        num(side),
                        num(side),
                        inner
                    ).unwrap();
                } else if !span.text.is_empty() {
                    out.write_str("<tspan x=\"")?;
                    write_num(out, x)?;
                    out.write_str("\" y=\"")?;
                    write_num(out, y)?;
                    out.write_str("\" text-anchor=\"start\"")?;
                    out.write_str(span_style(span.style & !shared))?;
                    out.write_char('>')?;
                    write_esc(out, &span.text)?;
                    out.write_str("</tspan>")?;
                }
                x += span.width;
            }
        } else {
            for (j, span) in line.iter().enumerate() {
                out.write_str("<tspan")?;
                if j == 0 {
                    out.write_str(" x=\"")?;
                    write_num(out, cx)?;
                    out.write_str("\" y=\"")?;
                    write_num(out, y)?;
                    out.write_char('"')?;
                }
                out.write_str(span_style(span.style & !shared))?;
                out.write_char('>')?;
                write_esc(out, &span.text)?;
                out.write_str("</tspan>")?;
            }
            if line.is_empty() {
                out.write_str("<tspan x=\"")?;
                write_num(out, cx)?;
                out.write_str("\" y=\"")?;
                write_num(out, y)?;
                out.write_str("\"> </tspan>")?;
            }
        }
        y += lh;
    }
    out.write_str("</text>")?;
    out.write_str(&extra)
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
        assert!(matches!(esc_text("plain"), Cow::Borrowed(_)));
        assert_eq!(esc_text("\u{fb02}\u{b0}lt\u{b6}\u{df}"), "&lt;");
    }

    #[test]
    fn pushed_numbers_match_display() {
        for v in [0.0, -0.004, 1.5, -2.345, 12.3, 0.07, 1e10, -1e12, 123456789012.345, f64::NAN, f64::INFINITY] {
            let mut out = String::from("x");
            push_num(&mut out, v);
            assert_eq!(out, format!("x{}", num(v)), "{v}");
        }
    }

    #[test]
    fn text_style_borrows_when_nothing_is_added() {
        assert!(matches!(text_style(" class=\"a\"", 0), Cow::Borrowed(_)));
        assert!(matches!(text_style(" font-weight=\"bold\"", style::BOLD), Cow::Borrowed(_)));
        assert_eq!(text_style(" x", style::BOLD | style::ITALIC), " x font-weight=\"var(--_w)\" font-style=\"italic\"");
    }
}
