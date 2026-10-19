//! Theme tokens, fonts and author styles. Ported from Pele `src/svg/theme.ts`.

use indexmap::IndexMap;

use super::builder::esc;
use crate::util::chars::is_space;
use crate::util::js;
use crate::util::unicode::{is_letter_property, is_number_property};

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

fn quoted_family(s: &[char], i: usize, quote: char) -> Option<usize> {
    let mut j = i + 1;
    while j < s.len() {
        let c = s[j];
        if c == quote {
            return Some(j + 1);
        }
        if matches!(c, '"' | '\'' | '\\' | '\n' | '<' | '>' | '{' | '}' | '(' | ')' | ';' | ':' | '@') {
            return None;
        }
        j += 1;
    }
    None
}

fn family(s: &[char], i: usize) -> Option<usize> {
    let c = *s.get(i)?;
    if c == '"' || c == '\'' {
        return quoted_family(s, i, c);
    }
    let lead = |c: char| is_letter_property(c) || is_number_property(c) || c == '_' || c == '-';
    if !lead(c) {
        return None;
    }
    let mut j = i + 1;
    while j < s.len() && (lead(s[j]) || s[j] == ' ' || s[j] == '.') {
        j += 1;
    }
    Some(j)
}

/// `^\s*FAMILY(?:\s*,\s*FAMILY)*\s*$`: a list of font names, quoted or not, and nothing that
/// could end the declaration it is put in.
pub fn is_font_list(list: &str) -> bool {
    let s: Vec<char> = list.chars().collect();
    let skip = |mut i: usize| {
        while i < s.len() && is_space(s[i]) {
            i += 1;
        }
        i
    };
    let mut i = skip(0);
    let Some(e) = family(&s, i) else {
        return false;
    };
    i = e;
    loop {
        i = skip(i);
        if i == s.len() {
            return true;
        }
        if s[i] != ',' {
            return false;
        }
        i = skip(i + 1);
        match family(&s, i) {
            Some(e) => i = e,
            None => return false,
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
        js::to_string(size),
        families(family, "sans-serif")
    )
}

/// The root's font declarations in resolved mode: Pele's, with the families written out.
pub fn resolved_font_style(size: f64, family: Option<&str>) -> String {
    format!(
        ";font:{}px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:{}",
        js::to_string(size),
        families(family, "sans-serif")
    )
}

const SHAPE_PROPS: &[&str] = &[
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

const TEXT_PROPS: &[&str] =
    &["color", "font-size", "font-family", "font-weight", "font-style", "text-decoration", "letter-spacing", "word-spacing", "opacity"];

/// `/url\s*\(|expression\s*\(|javascript:|[<>{};\\]|@import|image-set|\/\*/i`
pub fn is_unsafe_style(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.bytes().any(|b| matches!(b, b'<' | b'>' | b'{' | b'}' | b';' | b'\\')) {
        return true;
    }
    if lower.contains("javascript:") || lower.contains("@import") || lower.contains("image-set") || lower.contains("/*") {
        return true;
    }
    for word in ["url", "expression"] {
        let mut from = 0;
        while let Some(k) = lower[from..].find(word) {
            let after = from + k + word.len();
            let rest = lower[after..].trim_start_matches(is_space);
            if rest.starts_with('(') {
                return true;
            }
            from = from + k + 1;
        }
    }
    false
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

fn strip_important(value: &str) -> String {
    // `.replace(/!important/gi, '')`
    let lower = value.to_ascii_lowercase();
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    let mut from = 0;
    while let Some(k) = lower[from..].find("!important") {
        let at = from + k;
        out.push_str(&value[last..at]);
        last = at + "!important".len();
        from = last;
    }
    out.push_str(&value[last..]);
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
    let mut shape: IndexMap<String, String> = IndexMap::new();
    let mut text: IndexMap<String, String> = IndexMap::new();
    for decl in declarations {
        let decl = decl.as_ref();
        let Some(at) = decl.find(':') else { continue };
        if at == 0 {
            continue;
        }
        let prop = js::trim(&decl[..at]).to_lowercase();
        let value = strip_important(&decl[at + 1..]);
        let value = js::trim(&value);
        if value.is_empty() || is_unsafe_style(value) {
            continue;
        }
        if SHAPE_PROPS.contains(&prop.as_str()) {
            shape.insert(prop.clone(), value.to_string());
        }
        if TEXT_PROPS.contains(&prop.as_str()) {
            let key = if prop == "color" { "fill".to_string() } else { prop.clone() };
            text.insert(key, value.to_string());
        }
    }
    if shape.is_empty() && text.is_empty() {
        return ResolvedStyle::default();
    }
    let mut shape_css = String::new();
    let mut line_css = String::new();
    for (prop, value) in &shape {
        shape_css.push_str(&format!("{}:{};", prop, value));
        if prop != "fill" && prop != "fill-opacity" {
            line_css.push_str(&format!("{}:{};", prop, value));
        }
    }
    let mut text_css = String::new();
    for (prop, value) in &text {
        text_css.push_str(&format!("{}:{};", prop, value));
    }
    let weight = text.get("font-weight");
    let size = text.get("font-size");
    let px = size.filter(|s| is_plain_size(s)).map(|s| js::parse_float(s));
    let wrap = |css: &str| if css.is_empty() { String::new() } else { format!(" style=\"{}\"", esc(css)) };
    ResolvedStyle {
        shape: wrap(&shape_css),
        line: wrap(&line_css),
        text: wrap(&text_css),
        stroke: shape.get("stroke").cloned(),
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

const SERIES: [&str; 8] = ["#4c78a8", "#f58518", "#54a24b", "#e45756", "#72b7b2", "#eeca3b", "#b279a2", "#9d755d"];

/// Color for the nth data series in a chart, cycling through eight tokens.
pub fn series_color(index: i64) -> String {
    let k = index.rem_euclid(8) as usize;
    format!("var(--pele-series-{},{})", k + 1, SERIES[k])
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
    }
}
