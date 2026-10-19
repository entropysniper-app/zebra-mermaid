//! Whether the first word of a box title is a color. Ported from Pele
//! `src/diagrams/sequence/colors.ts`.
//!
//! Mermaid asks the browser whether the first word of a box title is a color. Pele has no
//! browser, so it knows the color keywords and the rgb() and hsl() notations. Other functions,
//! such as calc() and var(), are not recognized.

use crate::util::chars::is_space;
use crate::util::js;

const KEYWORDS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
    "transparent",
    "currentcolor",
    "inherit",
    "initial",
    "unset",
    "revert",
    "accentcolor",
    "accentcolortext",
    "activetext",
    "buttonborder",
    "buttonface",
    "buttontext",
    "canvas",
    "canvastext",
    "field",
    "fieldtext",
    "graytext",
    "highlight",
    "highlighttext",
    "linktext",
    "mark",
    "marktext",
    "selecteditem",
    "selecteditemtext",
    "visitedtext",
    "activeborder",
    "activecaption",
    "appworkspace",
    "background",
    "buttonhighlight",
    "buttonshadow",
    "captiontext",
    "inactiveborder",
    "inactivecaption",
    "inactivecaptiontext",
    "infobackground",
    "infotext",
    "menu",
    "menutext",
    "scrollbar",
    "threeddarkshadow",
    "threedface",
    "threedhighlight",
    "threedlightshadow",
    "threedshadow",
    "window",
    "windowframe",
    "windowtext",
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum K {
    Bad,
    Number,
    Percent,
    Angle,
    None,
}

/// `/^[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?(%|deg|grad|rad|turn)?$/i`: the unit, or None
/// when the term does not match.
fn term_unit(term: &str) -> Option<&str> {
    let b = term.as_bytes();
    let digits = |i: usize| i + b[i.min(b.len())..].iter().take_while(|c| c.is_ascii_digit()).count();
    let mut i = 0;
    if matches!(b.first(), Some(b'+' | b'-')) {
        i = 1;
    }
    let whole = digits(i);
    if whole > i {
        i = whole;
        // `(?:\.\d+)?`: taken only when digits follow the point.
        if b.get(i) == Some(&b'.') {
            let frac = digits(i + 1);
            if frac > i + 1 {
                i = frac;
            }
        }
    } else if b.get(i) == Some(&b'.') {
        let frac = digits(i + 1);
        if frac == i + 1 {
            return None;
        }
        i = frac;
    } else {
        return None;
    }
    // `(?:e[+-]?\d+)?`
    if matches!(b.get(i), Some(b'e' | b'E')) {
        let mut j = i + 1;
        if matches!(b.get(j), Some(b'+' | b'-')) {
            j += 1;
        }
        let end = digits(j);
        if end > j {
            i = end;
        }
    }
    let unit = &term[i..];
    let lower = unit.to_ascii_lowercase();
    match lower.as_str() {
        "" | "%" | "deg" | "grad" | "rad" | "turn" => Some(unit),
        _ => None,
    }
}

fn kind(term: &str) -> K {
    if term.to_lowercase() == "none" {
        return K::None;
    }
    match term_unit(term) {
        None => K::Bad,
        Some("") => K::Number,
        Some("%") => K::Percent,
        Some(_) => K::Angle,
    }
}

fn is_alpha(term: &str, modern: bool) -> bool {
    let k = kind(term);
    k == K::Number || k == K::Percent || (modern && k == K::None)
}

/// `str.split(/\s+/)`
fn split_space(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if is_space(c) {
            let mut end = i + c.len_utf8();
            while let Some(&(j, d)) = chars.peek() {
                if !is_space(d) {
                    break;
                }
                end = j + d.len_utf8();
                chars.next();
            }
            parts.push(&s[start..i]);
            start = end;
        }
    }
    parts.push(&s[start..]);
    parts
}

fn is_function(color: &str) -> bool {
    let hsl = color.starts_with("hsl");
    let Some(open) = color.find('(') else { return false };
    let name = &color[..open];
    if name != "rgb" && name != "rgba" && name != "hsl" && name != "hsla" {
        return false;
    }
    if js::len16(color) > 120 || color.find(')') != Some(color.len() - 1) {
        return false;
    }
    let inner = js::trim(&color[open + 1..color.len() - 1]);
    let legacy = inner.contains(',');
    let mut alpha: Option<&str> = None;
    let mut parts: Vec<&str>;
    if legacy {
        parts = inner.split(',').map(js::trim).collect();
        if parts.len() == 4 {
            alpha = parts.pop();
        }
    } else {
        let slash: Vec<&str> = inner.split('/').collect();
        if slash.len() > 2 {
            return false;
        }
        parts = split_space(js::trim(slash[0]));
        if slash.len() == 2 {
            alpha = Some(js::trim(slash[1]));
        }
    }
    if parts.len() != 3 || alpha.is_some_and(|a| !is_alpha(a, !legacy)) {
        return false;
    }
    let kinds: Vec<K> = parts.iter().map(|p| kind(p)).collect();
    if legacy {
        if hsl {
            return (kinds[0] == K::Number || kinds[0] == K::Angle) && kinds[1] == K::Percent && kinds[2] == K::Percent;
        }
        return (kinds[0] == K::Number || kinds[0] == K::Percent) && kinds[1] == kinds[0] && kinds[2] == kinds[0];
    }
    for (i, &k) in kinds.iter().enumerate() {
        let ok = k == K::Number || k == K::None || if hsl && i == 0 { k == K::Angle } else { k == K::Percent };
        if !ok {
            return false;
        }
    }
    true
}

pub(super) fn is_css_color(color: &str) -> bool {
    if color.contains('(') {
        return is_function(color);
    }
    let lower = color.to_lowercase();
    KEYWORDS.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_and_functions() {
        assert!(is_css_color("Aqua"));
        assert!(is_css_color("transparent"));
        assert!(!is_css_color("Group"));
        assert!(is_css_color("rgb(1, 2, 3)"));
        assert!(is_css_color("rgba(1, 2, 3, 0.5)"));
        assert!(is_css_color("rgb(10% 20% 30% / 50%)"));
        assert!(is_css_color("hsl(120deg 50% 50%)"));
        assert!(is_css_color("hsl(120, 50%, 50%)"));
        assert!(!is_css_color("hsl(120%, 50%, 50%)"));
        assert!(!is_css_color("rgb(1, 2%, 3)"));
        assert!(!is_css_color("rgb(1 2)"));
        assert!(is_css_color("rgb(none 2 3)"));
        assert!(!is_css_color("rgb(1, none, 3)"));
        assert!(is_css_color("rgb(1e2 .5 +3)"));
        assert!(!is_css_color("rgb(1. 2 3)"));
        assert!(!is_css_color("calc(1)"));
    }

    #[test]
    fn terms() {
        assert_eq!(term_unit("1.5e3DEG"), Some("DEG"));
        assert_eq!(term_unit("1.e3"), None);
        assert_eq!(term_unit("1e"), None);
        assert_eq!(term_unit("-.5%"), Some("%"));
        assert_eq!(split_space("a  b\u{3000}c"), vec!["a", "b", "c"]);
        assert_eq!(split_space(""), vec![""]);
    }
}
