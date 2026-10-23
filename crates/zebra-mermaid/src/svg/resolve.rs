//! The last step of resolved color output: theme references that are not colors.
//!
//! In resolved mode every color is written by the diagram code from its role (see
//! [`crate::theme::Theming::paint`]). What remains of Pele's custom properties are the font
//! weights of bold text (`--_w`, `--_tw`, `--_hw`), the monospace font (`--_fm`) and the corner
//! radius written as ` rx="var(--pele-radius,4px)"`. This pass replaces them with values, so
//! the SVG needs no CSS custom properties at all. Only attribute values that Pele writes are
//! matched; text content cannot contain these exact attribute strings, as text is escaped.

use super::builder::esc;
use crate::svg::concat::svg_format;

/// Pele's default corner radius, written as an attribute.
const RADIUS: &str = "4";

pub(crate) fn finish_resolved(content: &str, mono_family: &str) -> String {
    let mono = format!("\"{}\"", esc(mono_family));
    resolve_radius(content)
        .replace(" rx=\"var(--pele-radius,4px)\"", &svg_format!(" rx=\"", RADIUS, "\""))
        .replace("\"var(--_w)\"", "\"bold\"")
        .replace("\"var(--_tw)\"", "\"bold\"")
        .replace("\"var(--_hw)\"", "\"bold\"")
        .replace("\"var(--_fm)\"", &mono)
}
// Native consumers retain the concrete rx attribute in its original geometry position.
// Author styles remain after that default and keep their precedence.
fn resolve_radius(content: &str) -> std::borrow::Cow<'_, str> {
    const PREFIX: &str = " style=\"rx:var(--_r)";
    if !content.contains(PREFIX) {
        return std::borrow::Cow::Borrowed(content);
    }
    let mut out = String::with_capacity(content.len());
    let mut from = 0;
    while let Some(at) = content[from..].find("<rect") {
        let at = from + at;
        out.push_str(&content[from..at]);
        let Some(end) = content[at..].find('>').map(|n| at + n + 1) else {
            out.push_str(&content[at..]);
            return std::borrow::Cow::Owned(out);
        };
        let tag = &content[at..end];
        if let Some(p) = tag.find(PREFIX) {
            let tail = p + PREFIX.len();
            let mut tag = if tag[tail..].starts_with('"') {
                format!("{}{}", &tag[..p], &tag[tail + 1..])
            } else if tag[tail..].starts_with(';') {
                format!("{} style=\"{}", &tag[..p], &tag[tail + 1..])
            } else {
                out.push_str(tag);
                from = end;
                continue;
            };
            let insert = tag
                .find(" height=\"")
                .and_then(|h| {
                    let value = h + 9;
                    tag[value..].find('"').map(|end| value + end + 1)
                })
                .unwrap_or(5);
            tag.insert_str(insert, " rx=\"4\"");
            out.push_str(&tag);
        } else {
            out.push_str(tag);
        }
        from = end;
    }
    out.push_str(&content[from..]);
    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_attribute_values() {
        let out = finish_resolved(
            "<rect rx=\"var(--pele-radius,4px)\"/><text font-weight=\"var(--_w)\" font-family=\"var(--_fm)\">var(--_w)</text>",
            "\"Fira Code\", monospace",
        );
        assert_eq!(out, "<rect rx=\"4\"/><text font-weight=\"bold\" font-family=\"&quot;Fira Code&quot;, monospace\">var(--_w)</text>");
    }
}
