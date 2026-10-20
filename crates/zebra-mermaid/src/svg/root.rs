//! The root `<svg>` element. Ported from Pele `src/svg/root.ts`.

use super::builder::{esc, esc_text};
use super::theme::{families, RADIUS};
use crate::diagrams::RenderCtx;
use crate::svg::concat::{svg_format, svg_write};
use crate::theme::{CanvasBackground, ColorMode, Role};
use crate::util::{js, strs};
use memchr::memmem::Finder;
use std::fmt::Write;
use std::sync::OnceLock;

const ROUNDED: &str = " rx=\"var(--pele-radius,4px)\"";

struct Searches {
    rect: Finder<'static>,
    rounded: Finder<'static>,
    variables: [Finder<'static>; 4],
}

fn searches() -> &'static Searches {
    static SEARCHES: OnceLock<Searches> = OnceLock::new();
    SEARCHES.get_or_init(|| Searches {
        rect: Finder::new("<rect"),
        rounded: Finder::new(ROUNDED),
        variables: ["var(--_r)", "var(--_tw)", "var(--_hw)", "var(--_w)"].map(Finder::new),
    })
}

/// Each `<rect ...>` tag of `content` (`/<rect\b[^>]*>/g`): the text before it since the last
/// one, and the tag. `f` returns false to stop. Returns the end of the last tag visited.
fn for_each_rect(content: &str, mut f: impl FnMut(&str, &str) -> bool) -> usize {
    let b = content.as_bytes();
    let mut last = 0;
    // One searcher finds every `<rect`; a match inside a tag already taken is skipped, as
    // Pele's global expression resumes after the tag it matched.
    for start in searches().rect.find_iter(b) {
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
        match searches().rounded.find(tag.as_bytes()) {
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
        found = searches().rounded.find(tag.as_bytes()).is_some();
        !found
    });
    found
}

// The weights of bold text, each set only in a diagram that has such text.
const WEIGHT_TITLE: &str = ";--_tw:var(--pele-title-weight,bold)";
const WEIGHT_HEADING: &str = ";--_hw:var(--pele-heading-weight,bold)";
const WEIGHT_BOLD: &str = ";--_w:var(--pele-bold-weight,bold)";

/// Which of the private variables `--_r`, `--_tw`, `--_hw` and `--_w` the content uses, found
/// using prepared searches that stop at the first use of each declaration.
#[derive(Default, Clone, Copy)]
struct Uses {
    radius: bool,
    title: bool,
    heading: bool,
    bold: bool,
}

fn uses(content: &str) -> Uses {
    // Search only for the four declarations we need, stopping at each first match.
    // Walking every var(--_) occurrence also visits all repeated fill/stroke colors.
    let [radius, title, heading, bold] = searches().variables.each_ref().map(|finder| finder.find(content.as_bytes()).is_some());
    Uses { radius, title, heading, bold }
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
        svg_write!(head, "<title id=\"", prefix, "-title\">", esc_text(t), "</title>").unwrap();
        svg_write!(aria, " aria-labelledby=\"", prefix, "-title\"").unwrap();
    }
    if let Some(d) = acc.acc_descr.filter(|d| !d.is_empty()) {
        svg_write!(head, "<desc id=\"", prefix, "-desc\">", esc_text(d), "</desc>").unwrap();
        svg_write!(aria, " aria-describedby=\"", prefix, "-desc\"").unwrap();
    }
    let fit = if ctx.responsive { "max-width:100%;height:auto;" } else { "" };
    let width_text = js::number_text(width);
    let height_text = js::number_text(height);
    let w = width_text.as_str();
    let h = height_text.as_str();
    let canvas = |fill: &str| {
        svg_format!(
            "<rect class=\"pele-canvas\" data-mermaid-canvas-background=\"opaque\" x=\"0\" y=\"0\" width=\"",
            w,
            "\" height=\"",
            h,
            "\" fill=\"",
            esc(fill),
            "\"/>"
        )
    };
    let mut out = String::with_capacity(content.len() + head.len() + 900);
    match theming.mode {
        ColorMode::Css => {
            // The rewriting of rounded rectangles adds `var(--_r)` references; the root has to
            // declare the variable before the content, so the content is looked over first.
            let rounding = searches().rounded.find(content.as_bytes()).is_some();
            let found = uses(content);
            let radius = found.radius || (rounding && has_rounded_rect(content));
            svg_write!(
                out,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-",
                kind,
                "\" viewBox=\"0 0 ",
                w,
                " ",
                h,
                "\" width=\"",
                w,
                "\" height=\"",
                h,
                "\" style=\"",
                fit,
                theming.css_root_style()
            )
            .unwrap();
            if radius {
                write!(out, ";--_r:{RADIUS}").unwrap();
            }
            if found.title {
                out.push_str(WEIGHT_TITLE);
            }
            if found.heading {
                out.push_str(WEIGHT_HEADING);
            }
            if found.bold {
                out.push_str(WEIGHT_BOLD);
            }
            // Escape only the user-supplied font lists, rather than allocating and scanning
            // the complete declaration string. All remaining bytes are fixed or numeric.
            svg_write!(
                out,
                ";--_fm:var(--pele-font-mono,",
                esc(families(options.font_family_mono.as_deref(), "monospace")),
                ");font:",
                js::number_text(font_size),
                "px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:var(--pele-font,",
                esc(families(options.font_family.as_deref(), "sans-serif")),
                ")\" fill=\"var(--_fg)\" role=\"graphics-document document\" aria-roledescription=\"",
                kind,
                "\"",
                aria,
                ">"
            )
            .unwrap();
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
            let mono = families(options.font_family_mono.as_deref(), "monospace");
            svg_write!(
                out,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-",
                kind,
                "\" viewBox=\"0 0 ",
                w,
                " ",
                h,
                "\" width=\"",
                w,
                "\" height=\"",
                h,
                "\" style=\"",
                fit,
                "color:",
                text,
                ";font:",
                js::number_text(font_size),
                "px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:",
                esc(families(options.font_family.as_deref(), "sans-serif")),
                "\" fill=\"",
                text,
                "\" role=\"graphics-document document\" aria-roledescription=\"",
                kind,
                "\"",
                aria,
                ">"
            )
            .unwrap();
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
