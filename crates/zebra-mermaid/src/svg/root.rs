//! The root `<svg>` element. Ported from Pele `src/svg/root.ts`.

use super::builder::{esc, esc_text};
use super::theme::{families, font_style, resolved_font_style, RADIUS};
use crate::diagrams::RenderCtx;
use crate::theme::{CanvasBackground, ColorMode, Role};
use crate::util::js;

const ROUNDED: &str = " rx=\"var(--pele-radius,4px)\"";

/// A rectangle's corner radius comes from a variable. Browsers only lately take a variable in
/// the `rx` attribute, and all of them take one in a style, so that is where it is written. It
/// goes first there, for a radius the diagram's author set to come after it and win.
fn rounded(content: &str) -> String {
    if !content.contains(ROUNDED) {
        return content.to_string();
    }
    let b = content.as_bytes();
    let mut out = String::with_capacity(content.len() + 64);
    let mut last = 0;
    let mut from = 0;
    // `/<rect\b[^>]*>/g`
    while let Some(k) = content[from..].find("<rect") {
        let start = from + k;
        let after = start + 5;
        if after < b.len() && (b[after].is_ascii_alphanumeric() || b[after] == b'_') {
            from = after;
            continue;
        }
        let Some(close) = content[after..].find('>') else { break };
        let end = after + close + 1;
        let tag = &content[start..end];
        out.push_str(&content[last..start]);
        match tag.find(ROUNDED) {
            None => out.push_str(tag),
            Some(at) => {
                let rest = format!("{}{}", &tag[..at], &tag[at + ROUNDED.len()..]);
                if let Some(style) = rest.find(" style=\"") {
                    out.push_str(&rest[..style + 8]);
                    out.push_str("rx:var(--_r);");
                    out.push_str(&rest[style + 8..]);
                } else {
                    let cut = if rest.ends_with("/>") { rest.len() - 2 } else { rest.len() - 1 };
                    out.push_str(&rest[..cut]);
                    out.push_str(" style=\"rx:var(--_r)\"");
                    out.push_str(&rest[cut..]);
                }
            }
        }
        last = end;
        from = end;
    }
    out.push_str(&content[last..]);
    out
}

// The weights of bold text, each set only in a diagram that has such text.
const WEIGHTS: [(&str, &str); 3] = [("--_tw", "title"), ("--_hw", "heading"), ("--_w", "bold")];

/// The accessible name and description of a diagram.
#[derive(Debug, Clone, Copy, Default)]
pub struct Accessible<'a> {
    pub acc_title: Option<&'a str>,
    pub acc_descr: Option<&'a str>,
}

/// Wraps drawn content in the root element, with the theme aliases, fonts and accessibility
/// attributes. `width` and `height` are written as JavaScript writes numbers.
///
/// In CSS mode this is Pele's root. In resolved mode the root carries concrete colors and
/// fonts and no custom properties. An opaque canvas adds a background rectangle over the whole
/// view box before the content; a transparent one leaves it out and changes nothing else.
pub fn svg_document(kind: &str, width: f64, height: f64, font_size: f64, ctx: &RenderCtx, acc: Accessible<'_>, content: &str) -> String {
    let options = ctx.options;
    let theming = ctx.theming;
    let prefix = esc(options.id_prefix.as_deref().unwrap_or("pele")).into_owned();
    let mut head = String::new();
    let mut aria = String::new();
    if let Some(t) = acc.acc_title.filter(|t| !t.is_empty()) {
        head.push_str(&format!("<title id=\"{}-title\">{}</title>", prefix, esc_text(t)));
        aria.push_str(&format!(" aria-labelledby=\"{}-title\"", prefix));
    }
    if let Some(d) = acc.acc_descr.filter(|d| !d.is_empty()) {
        head.push_str(&format!("<desc id=\"{}-desc\">{}</desc>", prefix, esc_text(d)));
        aria.push_str(&format!(" aria-describedby=\"{}-desc\"", prefix));
    }
    let fit = if ctx.responsive { "max-width:100%;height:auto;" } else { "" };
    let w = js::to_string(width);
    let h = js::to_string(height);
    let canvas = |fill: &str| {
        format!(
            "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
            esc(fill)
        )
    };
    let mut out = String::with_capacity(content.len() + head.len() + 900);
    match theming.mode {
        ColorMode::Css => {
            let content = rounded(content);
            let mut weights = if content.contains("var(--_r)") { format!(";--_r:{}", RADIUS) } else { String::new() };
            for (alias, name) in WEIGHTS {
                if content.contains(&format!("var({})", alias)) {
                    weights.push_str(&format!(";{}:var(--pele-{}-weight,bold)", alias, name));
                }
            }
            let fonts = font_style(font_size, options.font_family.as_deref(), options.font_family_mono.as_deref());
            out.push_str(&format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" style=\"{fit}{}{weights}{}\" fill=\"var(--_fg)\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>",
                theming.css_root_style(),
                esc(&fonts)
            ));
            out.push_str(&head);
            if theming.canvas == CanvasBackground::Opaque {
                out.push_str(&canvas("var(--_bg)"));
            }
            out.push_str(&content);
        }
        ColorMode::Resolved => {
            let text = theming.resolved.color(Role::Text);
            let fonts = resolved_font_style(font_size, options.font_family.as_deref());
            let mono = families(options.font_family_mono.as_deref(), "monospace");
            out.push_str(&format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" style=\"{fit}color:{text}{}\" fill=\"{text}\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>",
                esc(&fonts)
            ));
            out.push_str(&head);
            if theming.canvas == CanvasBackground::Opaque {
                out.push_str(&canvas(theming.resolved.color(Role::Background)));
            }
            out.push_str(&super::resolve::finish_resolved(content, mono));
        }
    }
    out.push_str("</svg>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_rects() {
        assert_eq!(rounded("<rect x=\"1\" rx=\"var(--pele-radius,4px)\"/>"), "<rect x=\"1\" style=\"rx:var(--_r)\"/>");
        assert_eq!(rounded("<rect rx=\"var(--pele-radius,4px)\" style=\"fill:red\">"), "<rect style=\"rx:var(--_r);fill:red\">");
        assert_eq!(rounded("<rectx rx=\"var(--pele-radius,4px)\"/>"), "<rectx rx=\"var(--pele-radius,4px)\"/>");
    }
}
