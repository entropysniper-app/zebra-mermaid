//! Theme tokens, fonts and author styles. Ported from Pele `src/svg/theme.ts`.

use std::borrow::Cow;

use super::builder::esc;
use crate::util::chars::is_space;
use crate::util::js;
use crate::util::unicode::{is_letter_property, is_number_property};
use std::fmt::Write as _;

/// Private aliases set once on the root element, so each use is short and the public
/// `--pele-*` tokens keep their fallbacks. The text color is set as well: an icon is drawn in
/// currentColor, which would otherwise be whatever color the page gives the container.
pub const ROOT_STYLE: &str = "--_bg:var(--pele-bg,#fff);--_fg:var(--pele-fg,#1f1f1f);--_m:var(--pele-muted,#6e6e6e);\
--_l:var(--pele-line,#1f1f1f);--_s:var(--pele-surface,#f3f3f3);--_a:var(--pele-surface-alt,#e6e6e6);\
--_b:var(--pele-border,#8a8a8a);--_c:var(--pele-accent,#5b7bd5);color:var(--_fg)";

pub const FONT_MONO: &str = "var(--_fm)";
pub const RADIUS: &str = "var(--pele-radius,4px)";
/// The color of what a diagram marks as critical. It is not one of the series colors, as a
/// theme may put those in any order.
pub const CRITICAL: &str = "var(--pele-critical,#e45756)";

fn family_word(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    } else {
        is_letter_property(c) || is_number_property(c)
    }
}

fn family(s: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let Some(first) = s.next() else { return false };
    if first == '"' || first == '\'' {
        for c in s.by_ref() {
            if c == first {
                return true;
            }
            if matches!(c, '"' | '\'' | '\\' | '\n' | '<' | '>' | '{' | '}' | '(' | ')' | ';' | ':' | '@') {
                return false;
            }
        }
        return false;
    }
    if !family_word(first) {
        return false;
    }
    while s.next_if(|&c| family_word(c) || c == ' ' || c == '.').is_some() {}
    true
}

/// `^\s*FAMILY(?:\s*,\s*FAMILY)*\s*$`: a list of font names, quoted or not, and nothing that
/// could end the declaration it is put in.
pub fn is_font_list(list: &str) -> bool {
    // The validator needs neither character indices nor an owned copy of the list.
    // ASCII names avoid Unicode range searches; other scalars keep the same properties.
    let mut s = list.chars().peekable();
    loop {
        while s.next_if(|&c| is_space(c)).is_some() {}
        if !family(&mut s) {
            return false;
        }
        while s.next_if(|&c| is_space(c)).is_some() {}
        match s.next() {
            None => return true,
            Some(',') => {}
            _ => return false,
        }
    }
}

/// The caller's font list when it is safe to write, else the generic family.
pub fn families<'a>(list: Option<&'a str>, generic: &'a str) -> &'a str {
    match list {
        Some(l) if !l.is_empty() && is_font_list(l) => l,
        _ => generic,
    }
}

/// The fonts the labels were measured in are the ones they are drawn in, unless the page
/// names others. The rest of what text inherits is pinned, as none of it was measured.
pub fn font_style(size: f64, family: Option<&str>, mono: Option<&str>) -> String {
    format!(
        ";--_fm:var(--pele-font-mono,{});font:{}px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:var(--pele-font,{})",
        families(mono, "monospace"),
        js::number_text(size),
        families(family, "sans-serif")
    )
}

/// The root's font declarations in resolved mode: Pele's, with the families written out.
pub fn resolved_font_style(size: f64, family: Option<&str>) -> String {
    format!(
        ";font:{}px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:{}",
        js::number_text(size),
        families(family, "sans-serif")
    )
}

const SHAPE_PROPS: &[&str; 12] = &[
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-opacity",
    "opacity",
    "rx",
    "ry",
];

const TEXT_PROPS: &[&str; 9] =
    &["color", "font-size", "font-family", "font-weight", "font-style", "text-decoration", "letter-spacing", "word-spacing", "opacity"];

/// `/url\s*\(|expression\s*\(|javascript:|[<>{};\\]|@import|image-set|\/\*/i`
pub fn is_unsafe_style(value: &str) -> bool {
    if value.bytes().any(|b| matches!(b, b'<' | b'>' | b'{' | b'}' | b';' | b'\\')) {
        return true;
    }
    if ["javascript:", "@import", "image-set", "/*"].iter().any(|needle| find_ascii_ci(value, needle).is_some()) {
        return true;
    }
    for word in ["url", "expression"] {
        let mut from = 0;
        while let Some(k) = find_ascii_ci(&value[from..], word) {
            let after = from + k + word.len();
            if value[after..].trim_start_matches(is_space).starts_with('(') {
                return true;
            }
            from += k + 1;
        }
    }
    false
}

// Matching an ASCII byte always yields a UTF-8 boundary.
fn find_ascii_ci(value: &str, needle: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let first = needle.as_bytes()[0];
    memchr::memchr2_iter(first, first.to_ascii_uppercase(), bytes)
        .find(|&at| bytes.get(at..at + needle.len()).is_some_and(|part| part.eq_ignore_ascii_case(needle.as_bytes())))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolvedStyle {
    pub shape: String,
    pub line: String,
    pub text: String,
    pub stroke: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub font_size: Option<f64>,
}

fn strip_important(value: &str) -> Cow<'_, str> {
    let Some(first) = find_ascii_ci(value, "!important") else { return Cow::Borrowed(js::trim(value)) };
    let mut out = String::with_capacity(value.len());
    out.push_str(&value[..first]);
    let mut from = first + "!important".len();
    while let Some(k) = find_ascii_ci(&value[from..], "!important") {
        let at = from + k;
        out.push_str(&value[from..at]);
        from = at + "!important".len();
    }
    out.push_str(&value[from..]);
    out.truncate(js::trim_end(&out).len());
    let start = out.len() - js::trim_start(&out).len();
    if start > 0 {
        out.drain(..start);
    }
    Cow::Owned(out)
}

// Allowlists bound storage and insertion order. Replacement keeps the first position,
// matching the upstream Map, without allocating keys or hash-table storage.
struct StyleSlots<'a, const N: usize> {
    keys: &'static [&'static str; N],
    values: [Option<Cow<'a, str>>; N],
    order: [usize; N],
    len: usize,
}

impl<'a, const N: usize> StyleSlots<'a, N> {
    fn new(keys: &'static [&'static str; N]) -> Self {
        Self { keys, values: std::array::from_fn(|_| None), order: [0; N], len: 0 }
    }
    fn insert(&mut self, index: usize, value: Cow<'a, str>) {
        if self.values[index].is_none() {
            self.order[self.len] = index;
            self.len += 1;
        }
        self.values[index] = Some(value);
    }
    fn get(&self, key: &str) -> Option<&str> {
        self.keys.iter().position(|&k| k == key).and_then(|i| self.values[i].as_deref())
    }
    fn iter(&self) -> impl Iterator<Item = (&'static str, &str)> {
        self.order[..self.len].iter().map(|&i| (self.keys[i], self.values[i].as_deref().unwrap()))
    }
}

fn append_style(out: &mut String, property: &str, value: &str) {
    if out.is_empty() {
        out.push_str(" style=\"");
    }
    write!(out, "{property}:{};", esc(value)).unwrap();
}

fn finish_style(mut out: String) -> String {
    if !out.is_empty() {
        out.push('"');
    }
    out
}

/// `/^[\d.]+(px)?$/`
fn is_plain_size(size: &str) -> bool {
    let body = size.strip_suffix("px").unwrap_or(size);
    !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit() || b == b'.')
}

/// Splits Mermaid style declarations into what applies to a shape and what applies to its text.
pub fn resolve_style<S: AsRef<str>>(declarations: &[S]) -> ResolvedStyle {
    if declarations.is_empty() {
        return ResolvedStyle::default();
    }
    let mut shape = StyleSlots::new(SHAPE_PROPS);
    let mut text = StyleSlots::new(TEXT_PROPS);
    for decl in declarations {
        let decl = decl.as_ref();
        let Some(at) = decl.find(':') else { continue };
        if at == 0 {
            continue;
        }
        let property = js::trim(&decl[..at]);
        let prop = if property.is_ascii() && !property.bytes().any(|b| b.is_ascii_uppercase()) {
            Cow::Borrowed(property)
        } else {
            Cow::Owned(property.to_lowercase())
        };
        let shape_index = SHAPE_PROPS.iter().position(|&key| key == prop);
        let text_index = TEXT_PROPS.iter().position(|&key| key == prop);
        if shape_index.is_none() && text_index.is_none() {
            continue;
        }
        let value = strip_important(&decl[at + 1..]);
        if value.is_empty() || is_unsafe_style(&value) {
            continue;
        }
        match (shape_index, text_index) {
            (Some(a), Some(b)) => {
                shape.insert(a, value.clone());
                text.insert(b, value);
            }
            (Some(a), None) => shape.insert(a, value),
            (None, Some(b)) => text.insert(b, value),
            (None, None) => unreachable!(),
        }
    }
    let mut shape_css = String::new();
    let mut line_css = String::new();
    for (prop, value) in shape.iter() {
        append_style(&mut shape_css, prop, value);
        if prop != "fill" && prop != "fill-opacity" {
            append_style(&mut line_css, prop, value);
        }
    }
    let mut text_css = String::new();
    for (prop, value) in text.iter() {
        append_style(&mut text_css, if prop == "color" { "fill" } else { prop }, value);
    }
    let weight = text.get("font-weight");
    let size = text.get("font-size");
    let px = size.filter(|s| is_plain_size(s)).map(js::parse_float);
    ResolvedStyle {
        shape: finish_style(shape_css),
        line: finish_style(line_css),
        text: finish_style(text_css),
        stroke: shape.get("stroke").map(str::to_owned),
        bold: weight.is_some_and(|w| w == "bold" || w == "bolder" || js::number(w) >= 600.0),
        italic: text.get("font-style").is_some_and(|s| s == "italic"),
        font_size: px.filter(|&p| p > 0.0 && p <= 200.0),
    }
}

/// `raw.replace(/[^\w-]+/g, ' ').trim()`, with a leading space when anything is left.
pub fn class_names(raw: &str) -> String {
    let mut cleaned = String::with_capacity(raw.len());
    let mut in_run = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            cleaned.push(c);
            in_run = false;
        } else if !in_run {
            cleaned.push(' ');
            in_run = true;
        }
    }
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        String::new()
    } else {
        format!(" {}", cleaned)
    }
}

#[cfg(test)]
const SERIES: [&str; 8] = ["#4c78a8", "#f58518", "#54a24b", "#e45756", "#72b7b2", "#eeca3b", "#b279a2", "#9d755d"];

/// `var(--pele-series-N,color)` for each of the eight series, written out once.
const SERIES_VARS: [&str; 8] = [
    "var(--pele-series-1,#4c78a8)",
    "var(--pele-series-2,#f58518)",
    "var(--pele-series-3,#54a24b)",
    "var(--pele-series-4,#e45756)",
    "var(--pele-series-5,#72b7b2)",
    "var(--pele-series-6,#eeca3b)",
    "var(--pele-series-7,#b279a2)",
    "var(--pele-series-8,#9d755d)",
];

/// Color for the nth data series in a chart, cycling through eight tokens.
pub fn series_color(index: i64) -> &'static str {
    SERIES_VARS[index.rem_euclid(8) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_lists() {
        assert!(is_font_list("Arial"));
        assert!(is_font_list(" 'Noto Sans SC', \"Helvetica Neue\", sans-serif "));
        assert!(is_font_list("思源黑体, Roboto"));
        assert!(!is_font_list("Arial; color: red"));
        assert!(!is_font_list("a,"));
        assert!(!is_font_list("\"x"));
        assert!(!is_font_list("url(x)"));
    }

    #[test]
    fn styles_are_filtered() {
        let s = resolve_style(&["fill:#f9f", "stroke:#333", "stroke-width:4px", "color:red", "font-weight:bold"]);
        assert_eq!(s.shape, " style=\"fill:#f9f;stroke:#333;stroke-width:4px;\"");
        assert_eq!(s.line, " style=\"stroke:#333;stroke-width:4px;\"");
        assert_eq!(s.text, " style=\"fill:red;font-weight:bold;\"");
        assert!(s.bold);
        let s = resolve_style(&["fill:url(#x)", "stroke: expression (1)", "color:red}", "fill:javascript:alert(1)"]);
        assert_eq!(s, ResolvedStyle::default());
        let s = resolve_style(&["font-size:20px", "font-weight: 700 !IMPORTANT"]);
        assert_eq!(s.font_size, Some(20.0));
        assert!(s.bold);
    }

    #[test]
    fn classes() {
        assert_eq!(class_names("a b"), " a b");
        assert_eq!(class_names("<x>"), " x");
        assert_eq!(class_names("中文"), "");
    }

    #[test]
    fn series() {
        assert_eq!(series_color(9), "var(--pele-series-2,#f58518)");
        assert_eq!(series_color(-1), "var(--pele-series-8,#9d755d)");
        for (k, color) in SERIES.iter().enumerate() {
            assert_eq!(series_color(k as i64), format!("var(--pele-series-{},{})", k + 1, color));
        }
    }
}
