//! Value formats. Ported from Pele `src/diagrams/treemap/format.ts`.
//!
//! The parts of d3-format's specifier that treemap values use: an optional `$`, `,` for
//! thousands separators, a precision, and a type of `f`, `%`, `d`, `e` or none. Fill, alignment
//! and width are read and ignored. Anything else falls back to the default, `,`.

use crate::util::chars::is_line_end;
use crate::util::js;

/// The groups of `RE_SPEC` that the format reads.
#[derive(Debug, Clone, Default, PartialEq)]
struct Spec {
    /// Group 1, the sign: `-`, `+`, `(` or a space.
    sign: Option<char>,
    /// Group 2, `$`.
    currency: bool,
    /// Group 3, `,`.
    comma: bool,
    /// Group 4, the precision digits.
    precision: Option<String>,
    /// Group 5, `~`.
    trim: bool,
    /// Group 6, the type letter or `%`.
    kind: Option<char>,
}

fn is_align(c: char) -> bool {
    matches!(c, '<' | '>' | '=' | '^')
}

/// What follows the fill and alignment in
/// `/^(?:.?[<>=^])?([-+( ])?(\$)?0?\d*(,)?(?:\.(\d+))?(~)?([a-zA-Z%])?$/`, matched from byte `p`
/// to the end. Each part takes characters no later part can, so the greedy reading is the only
/// one that can reach the end.
fn rest(s: &str, p: usize) -> Option<Spec> {
    let b = s.as_bytes();
    let mut i = p;
    let mut spec = Spec::default();
    if let Some(&c) = b.get(i) {
        if matches!(c, b'-' | b'+' | b'(' | b' ') {
            spec.sign = Some(c as char);
            i += 1;
        }
    }
    if b.get(i) == Some(&b'$') {
        spec.currency = true;
        i += 1;
    }
    // `0?\d*`: both read digits, and neither is captured.
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    if b.get(i) == Some(&b',') {
        spec.comma = true;
        i += 1;
    }
    if b.get(i) == Some(&b'.') {
        let mut j = i + 1;
        while b.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
        // `(?:\.(\d+))?` needs a digit after the point; otherwise the group is skipped and the
        // point is left for the rest, which cannot take it.
        if j > i + 1 {
            spec.precision = Some(s[i + 1..j].to_string());
            i = j;
        }
    }
    if b.get(i) == Some(&b'~') {
        spec.trim = true;
        i += 1;
    }
    if let Some(&c) = b.get(i) {
        if c.is_ascii_alphabetic() || c == b'%' {
            spec.kind = Some(c as char);
            i += 1;
        }
    }
    (i == b.len()).then_some(spec)
}

/// `RE_SPEC.exec(s)`. The optional fill and alignment group is tried first with a fill
/// character, then without one, then not at all, as the expression backtracks.
fn match_spec(s: &str) -> Option<Spec> {
    let mut chars = s.char_indices();
    let first = chars.next();
    let second = chars.next();
    // `.` matches one UTF-16 code unit that is not a line terminator; a character outside the
    // Basic Multilingual Plane is two units, so a fill of one leaves half of it for `[<>=^]`.
    if let (Some((_, fill)), Some((at, align))) = (first, second) {
        if !is_line_end(fill) && fill.len_utf16() == 1 && is_align(align) {
            if let Some(spec) = rest(s, at + align.len_utf8()) {
                return Some(spec);
            }
        }
    }
    if let Some((_, align)) = first {
        if is_align(align) {
            if let Some(spec) = rest(s, align.len_utf8()) {
                return Some(spec);
            }
        }
    }
    rest(s, 0)
}

fn group(digits: &str) -> String {
    let mut out = String::new();
    let mut i = digits.len();
    while i > 0 {
        let from = i.saturating_sub(3);
        out = if out.is_empty() { digits[from..i].to_string() } else { format!("{},{}", &digits[from..i], out) };
        i = from;
    }
    out
}

/// `x.toExponential(fraction)`: the digits `toPrecision(fraction + 1)` gives, always written
/// with an exponent.
fn to_exponential(x: f64, fraction: u32) -> String {
    let fixed = js::to_precision(x, fraction + 1);
    if fixed.contains('e') || !x.is_finite() {
        return fixed;
    }
    let (sign, body) = match fixed.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", fixed.as_str()),
    };
    let point = body.find('.').unwrap_or(body.len());
    let all: String = body.chars().filter(char::is_ascii_digit).collect();
    let lead = all.find(|c| c != '0');
    let (digits, exponent) = match lead {
        // Zero keeps its digits: `(0).toExponential(2)` is "0.00e+0".
        None => (all[..(fraction as usize + 1).min(all.len())].to_string(), 0i64),
        Some(k) => (all[k..].to_string(), point as i64 - 1 - k as i64),
    };
    let mut out = String::from(sign);
    out.push_str(&digits[..1]);
    if digits.len() > 1 {
        out.push('.');
        out.push_str(&digits[1..]);
    }
    out.push('e');
    out.push(if exponent < 0 { '-' } else { '+' });
    out.push_str(&exponent.abs().to_string());
    out
}

/// `.replace(/\.?0+$/, '')`
fn trim_zeros(body: &str) -> String {
    let kept = body.trim_end_matches('0');
    if kept.len() == body.len() {
        return body.to_string();
    }
    kept.strip_suffix('.').unwrap_or(kept).to_string()
}

/// A value formatter for a d3-format specifier.
pub struct Formatter {
    spec: Spec,
    precision: Option<u32>,
}

pub fn formatter(spec: &str) -> Formatter {
    // Mermaid documents `$0,0` for a dollar amount with thousands separators.
    let spec = match_spec(if spec == "$0,0" { "$," } else { spec }).or_else(|| match_spec(",")).unwrap_or_default();
    // `Math.min(Number(m[4]), 20)`
    let precision = spec.precision.as_deref().map(|p| js::min(js::number(p), 20.0) as u32);
    Formatter { spec, precision }
}

impl Formatter {
    pub fn format(&self, value: f64) -> String {
        let spec = &self.spec;
        let abs = value.abs();
        let mut suffix = "";
        let mut body = match spec.kind {
            Some('f') => js::to_fixed(abs, self.precision.unwrap_or(6)),
            Some('%') => {
                suffix = "%";
                js::to_fixed(abs * 100.0, self.precision.unwrap_or(6))
            }
            Some('d') => js::to_fixed(js::round(abs), 0),
            Some('e') => to_exponential(abs, self.precision.unwrap_or(6)),
            _ => js::to_string(js::number(&js::to_precision(abs, self.precision.unwrap_or(12).max(1)))),
        };
        if spec.trim && body.contains('.') && !body.contains('e') {
            body = trim_zeros(&body);
        }
        if spec.comma && !body.contains('e') {
            body = match body.find('.') {
                None => group(&body),
                Some(dot) => format!("{}{}", group(&body[..dot]), &body[dot..]),
            };
        }
        let sign = if value < 0.0 {
            "-"
        } else if spec.sign == Some('+') {
            "+"
        } else {
            ""
        };
        format!("{}{}{}{}", sign, if spec.currency { "$" } else { "" }, body, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs() {
        assert_eq!(match_spec(",").map(|s| s.comma), Some(true));
        assert_eq!(
            match_spec("$,.2f").map(|s| (s.currency, s.comma, s.precision, s.kind)),
            Some((true, true, Some("2".into()), Some('f')))
        );
        assert_eq!(match_spec("*>10.1%").map(|s| (s.precision, s.kind)), Some((Some("1".into()), Some('%'))));
        assert_eq!(match_spec(">+").map(|s| s.sign), Some(Some('+')));
        // `<` alone is an alignment, then nothing.
        assert_eq!(match_spec("<"), Some(Spec::default()));
        assert_eq!(match_spec("abc"), None);
        assert_eq!(match_spec("1."), None);
        assert_eq!(match_spec("\n<"), None);
    }

    #[test]
    // This is a decimal formatting input, not a mathematical constant. Keep its exact value.
    #[allow(clippy::approx_constant)]
    fn formats() {
        assert_eq!(formatter(",").format(1234567.0), "1,234,567");
        assert_eq!(formatter("$0,0").format(1234.5), "$1,234.5");
        assert_eq!(formatter(".2f").format(3.14159), "3.14");
        assert_eq!(formatter(".1%").format(0.256), "25.6%");
        assert_eq!(formatter("d").format(2.5), "3");
        assert_eq!(formatter(".2e").format(12345.0), "1.23e+4");
        assert_eq!(formatter("e").format(0.00012), "1.200000e-4");
        assert_eq!(formatter(".0e").format(5.0), "5e+0");
        assert_eq!(formatter(",.2~f").format(1000.10), "1,000.1");
        assert_eq!(formatter("+,").format(5.0), "+5");
    }
}
