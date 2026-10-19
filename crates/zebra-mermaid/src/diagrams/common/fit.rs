//! Plain labels and labels fitted to a box. Ported from Pele `src/diagrams/common/fit.ts`.

use crate::text::label::br_end;
use crate::text::{layout_label, Label, Measure};
use crate::util::{js, strs};
use std::borrow::Cow;

/// `text.replace(/<(?!br\s*\/?>)/gi, '&lt;')`
fn escape_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (i, _) in text.match_indices('<') {
        if br_end(text, i).is_some() {
            continue;
        }
        out.push_str(&text[last..i]);
        out.push_str("&lt;");
        last = i + 1;
    }
    out.push_str(&text[last..]);
    out
}

/// Lays out text the way Mermaid's journey and timeline draw it: as plain text in which only
/// `<br>` is markup, so other tags and icon names show as written instead of being read as
/// formatting.
pub fn plain_label(raw: Option<&str>, m: &Measure, size: f64, max: f64, base: u8) -> Label {
    let text = raw.map(|t| {
        let mut t = Cow::Borrowed(t);
        if t.contains('<') {
            t = Cow::Owned(escape_tags(&t));
        }
        if strs::contains(&t, ":fa-") {
            t = Cow::Owned(t.replace(":fa-", "&colon;fa-"));
        }
        t
    });
    layout_label(text.as_deref(), false, m, size, max, base)
}

/// The pieces a word can be cut between: `/<br\s*\/?>|ﬂ°°?\w+¶ß|&#?\w+;|[\ud800-\udbff][\udc00-\udfff]|[\s\S]/gi`.
fn units(raw: &str) -> Vec<&str> {
    let b = raw.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let end = (|| {
            if let Some(e) = br_end(raw, i) {
                return Some(e);
            }
            let rest = &raw[i..];
            if let Some(after) = rest.strip_prefix("\u{fb02}\u{b0}") {
                let after = after.strip_prefix('\u{b0}').unwrap_or(after);
                let start = raw.len() - after.len();
                let mut j = start;
                while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                    j += 1;
                }
                if j > start && raw[j..].starts_with("\u{b6}\u{df}") {
                    return Some(j + "\u{b6}\u{df}".len());
                }
            }
            if b[i] == b'&' {
                let mut j = i + 1;
                if b.get(j) == Some(&b'#') {
                    j += 1;
                }
                let start = j;
                while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                    j += 1;
                }
                if j > start && b.get(j) == Some(&b';') {
                    return Some(j + 1);
                }
            }
            None
        })()
        .unwrap_or_else(|| i + raw[i..].chars().next().map_or(1, char::len_utf8));
        out.push(&raw[i..end]);
        i = end;
    }
    out
}

/// `layout_label` wraps at spaces only. For a box of fixed width, this also cuts any word too
/// long to fit.
pub fn fit_label(raw: Option<&str>, m: &Measure, size: f64, max: f64, base: u8) -> Label {
    let label = plain_label(raw, m, size, max, base);
    let Some(raw) = raw else { return label };
    if label.width <= max {
        return label;
    }
    let mut out = String::new();
    let mut width = 0.0;
    for unit in units(raw) {
        // `unit.length > 2` counts UTF-16 units: a surrogate pair is 2, so it is measured.
        let long = js::len16(unit) > 2;
        if js::trim(unit).is_empty() || (long && unit.starts_with('<')) {
            width = 0.0;
        } else {
            let w = m.width(if long { "x" } else { unit }, size, base);
            if width > 0.0 && width + w > max {
                out.push('\n');
                width = 0.0;
            }
            width += w;
        }
        out.push_str(unit);
    }
    plain_label(Some(&out), m, size, max, base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_split_codes_and_pairs() {
        assert_eq!(
            units("a<br/>&amp;😀\u{fb02}\u{b0}\u{b0}35\u{b6}\u{df}b"),
            vec!["a", "<br/>", "&amp;", "😀", "\u{fb02}\u{b0}\u{b0}35\u{b6}\u{df}", "b"]
        );
        assert_eq!(escape_tags("<b>x<br>y</b>"), "&lt;b>x<br>y&lt;/b>");
    }
}
