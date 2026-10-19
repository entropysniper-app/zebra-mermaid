//! The root `<svg>` element. Ported from Pele `src/svg/root.ts`.

use super::builder::{esc, esc_text};
use super::theme::{families, font_style, resolved_font_style, RADIUS};
use crate::diagrams::RenderCtx;
use crate::theme::{CanvasBackground, ColorMode, Role};
use crate::util::{js, strs};
use std::fmt::Write;

const ROUNDED: &str = " rx=\"var(--pele-radius,4px)\"";

/// Each `<rect ...>` tag of `content` (`/<rect\b[^>]*>/g`): the text before it since the last
/// one, and the tag. `f` returns false to stop. Returns the end of the last tag visited.
fn for_each_rect(content: &str, mut f: impl FnMut(&str, &str) -> bool) -> usize {
    let b = content.as_bytes();
    let mut last = 0;
    // One searcher finds every `<rect`; a match inside a tag already taken is skipped, as
    // Pele's global expression resumes after the tag it matched.
    for (start, _) in content.match_indices("<rect") {
        if start < last {
            continue;
        }
        let after = start + 5;
        if after < b.len() && (b[after].is_ascii_alphanumeric() || b[after] == b'_') {
            continue;
        }
        let Some(close) = content[after..].find('>') else { break };
        let end = after + close + 1;
        if !f(&content[last..start], &content[start..end]) {
            return end;
        }
        last = end;
    }
    last
}

/// A rectangle's corner radius comes from a variable. Browsers only lately take a variable in
/// the `rx` attribute, and all of them take one in a style, so that is where it is written. It
/// goes first there, for a radius the diagram's author set to come after it and win.
///
/// Appends `content` to `out` with that rewriting. The caller has found that `content` holds
/// the variable somewhere.
fn push_rounded(out: &mut String, content: &str) {
    let last = for_each_rect(content, |between, tag| {
        out.push_str(between);
        match strs::find(tag, ROUNDED) {
            None => out.push_str(tag),
            Some(at) => {
                // Work in the final output buffer instead of allocating one temporary
                // String per rectangle. Search after removal to preserve exact semantics.
                let begin = out.len();
                out.push_str(&tag[..at]);
                out.push_str(&tag[at + ROUNDED.len()..]);
                let rest = &out[begin..];
                if let Some(style) = strs::find(rest, " style=\"") {
                    out.insert_str(begin + style + 8, "rx:var(--_r);");
                } else {
                    let cut = if rest.ends_with("/>") { rest.len() - 2 } else { rest.len() - 1 };
                    out.insert_str(begin + cut, " style=\"rx:var(--_r)\"");
                }
            }
        }
        true
    });
    out.push_str(&content[last..]);
}

/// Whether [`push_rounded`] would rewrite any tag of `content`.
fn has_rounded_rect(content: &str) -> bool {
    let mut found = false;
    for_each_rect(content, |_, tag| {
        found = strs::contains(tag, ROUNDED);
        !found
    });
    found
}

// The weights of bold text, each set only in a diagram that has such text.
const WEIGHT_TITLE: &str = ";--_tw:var(--pele-title-weight,bold)";
const WEIGHT_HEADING: &str = ";--_hw:var(--pele-heading-weight,bold)";
const WEIGHT_BOLD: &str = ";--_w:var(--pele-bold-weight,bold)";

/// Which of the private variables `--_r`, `--_tw`, `--_hw` and `--_w` the content uses, found
/// in one pass over its `var(--_` references.
#[derive(Default, Clone, Copy)]
struct Uses {
    radius: bool,
    title: bool,
    heading: bool,
    bold: bool,
}

fn uses(content: &str) -> Uses {
    let mut found = Uses::default();
    for (at, _) in content.match_indices("var(--_") {
        let rest = &content[at + "var(--_".len()..];
        if rest.starts_with("r)") {
            found.radius = true;
        } else if rest.starts_with("tw)") {
            found.title = true;
        } else if rest.starts_with("hw)") {
            found.heading = true;
        } else if rest.starts_with("w)") {
            found.bold = true;
        }
    }
    found
}

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
    let prefix = esc(options.id_prefix.as_deref().unwrap_or("pele"));
    let mut head = String::new();
    let mut aria = String::new();
    if let Some(t) = acc.acc_title.filter(|t| !t.is_empty()) {
        write!(head, "<title id=\"{}-title\">{}</title>", prefix, esc_text(t)).unwrap();
        write!(aria, " aria-labelledby=\"{}-title\"", prefix).unwrap();
    }
    if let Some(d) = acc.acc_descr.filter(|d| !d.is_empty()) {
        write!(head, "<desc id=\"{}-desc\">{}</desc>", prefix, esc_text(d)).unwrap();
        write!(aria, " aria-describedby=\"{}-desc\"", prefix).unwrap();
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
            // The rewriting of rounded rectangles adds `var(--_r)` references; the root has to
            // declare the variable before the content, so the content is looked over first.
            let rounding = content.contains(ROUNDED);
            let found = uses(content);
            let radius = found.radius || (rounding && has_rounded_rect(content));
            let mut weights = if radius { format!(";--_r:{}", RADIUS) } else { String::new() };
            if found.title {
                weights.push_str(WEIGHT_TITLE);
            }
            if found.heading {
                weights.push_str(WEIGHT_HEADING);
            }
            if found.bold {
                weights.push_str(WEIGHT_BOLD);
            }
            let fonts = font_style(font_size, options.font_family.as_deref(), options.font_family_mono.as_deref());
            write!(out,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" style=\"{fit}{}{weights}{}\" fill=\"var(--_fg)\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>",
                theming.css_root_style(),
                esc(&fonts)
            ).unwrap();
            out.push_str(&head);
            if theming.canvas == CanvasBackground::Opaque {
                out.push_str(&canvas("var(--_bg)"));
            }
            if rounding {
                push_rounded(&mut out, content);
            } else {
                out.push_str(content);
            }
        }
        ColorMode::Resolved => {
            let text = theming.resolved.color(Role::Text);
            let fonts = resolved_font_style(font_size, options.font_family.as_deref());
            let mono = families(options.font_family_mono.as_deref(), "monospace");
            write!(out,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" style=\"{fit}color:{text}{}\" fill=\"{text}\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>",
                esc(&fonts)
            ).unwrap();
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

    fn rounded(content: &str) -> String {
        let mut out = String::new();
        push_rounded(&mut out, content);
        out
    }

    #[test]
    fn rounds_rects() {
        assert_eq!(rounded("<rect x=\"1\" rx=\"var(--pele-radius,4px)\"/>"), "<rect x=\"1\" style=\"rx:var(--_r)\"/>");
        assert_eq!(rounded("<rect rx=\"var(--pele-radius,4px)\" style=\"fill:red\">"), "<rect style=\"rx:var(--_r);fill:red\">");
        assert_eq!(rounded("<rectx rx=\"var(--pele-radius,4px)\"/>"), "<rectx rx=\"var(--pele-radius,4px)\"/>");
        assert_eq!(rounded("a<rect/>b<rect rx=\"var(--pele-radius,4px)\"/>c"), "a<rect/>b<rect style=\"rx:var(--_r)\"/>c");
        assert!(has_rounded_rect("<path/><rect rx=\"var(--pele-radius,4px)\"/>"));
        assert!(!has_rounded_rect("<rectx rx=\"var(--pele-radius,4px)\"/><rect/>"));
    }

    #[test]
    fn finds_variable_uses() {
        let u = uses("<text font-weight=\"var(--_w)\"/><g style=\"rx:var(--_r)\"/>var(--_hw) var(--_x)");
        assert!(u.bold && u.radius && u.heading && !u.title);
        let u = uses("nothing here");
        assert!(!u.bold && !u.radius && !u.heading && !u.title);
    }
}
