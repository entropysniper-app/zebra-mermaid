//! Contrasting text on authored fills. Ported from Pele 1.13.0 `src/svg/ink.ts` (MIT).
use super::theme::ResolvedStyle;
use crate::util::{js, math};

pub const ON_LIGHT: &str = "var(--pele-fg-on-light,#1f1f1f)";
pub const ON_DARK: &str = "var(--pele-fg-on-dark,#fff)";
const DARK: &str = "black blue blueviolet brown crimson darkblue darkcyan darkgreen darkmagenta darkolivegreen darkorchid darkred darkslateblue darkslategray darkviolet dimgray firebrick forestgreen green indigo maroon mediumblue mediumslateblue mediumvioletred midnightblue navy olive purple rebeccapurple royalblue saddlebrown seagreen sienna slateblue steelblue teal";
const LIGHT: &str = "aliceblue antiquewhite aqua aquamarine azure beige bisque blanchedalmond burlywood cadetblue chartreuse chocolate coral cornflowerblue cornsilk cyan darkgoldenrod darkgray darkkhaki darkorange darksalmon darkseagreen darkturquoise deeppink deepskyblue dodgerblue floralwhite fuchsia gainsboro ghostwhite gold goldenrod gray greenyellow honeydew hotpink indianred ivory khaki lavender lavenderblush lawngreen lemonchiffon lightblue lightcoral lightcyan lightgoldenrodyellow lightgray lightgreen lightpink lightsalmon lightseagreen lightskyblue lightslategray lightsteelblue lightyellow lime limegreen linen magenta mediumaquamarine mediumorchid mediumpurple mediumseagreen mediumspringgreen mediumturquoise mintcream mistyrose moccasin navajowhite oldlace olivedrab orange orangered orchid palegoldenrod palegreen paleturquoise palevioletred papayawhip peachpuff peru pink plum powderblue red rosybrown salmon sandybrown seashell silver skyblue slategray snow springgreen tan thistle tomato turquoise violet wheat white whitesmoke yellow yellowgreen";

pub fn plain_color(raw: &str) -> bool {
    let value = js::trim(raw).to_lowercase();
    let name = value.replacen("grey", "gray", 1);
    value.strip_prefix('#').is_some_and(|h| matches!(h.len(), 3 | 4 | 6 | 8) && h.bytes().all(|b| b.is_ascii_hexdigit()))
        || DARK.split(' ').chain(LIGHT.split(' ')).any(|n| n == name)
        || matches!(value.as_str(), "transparent" | "currentcolor")
}
fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        math::pow((c + 0.055) / 1.055, 2.4)
    }
}
fn luminance(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}
fn clamp(n: f64) -> f64 {
    js::min(js::max(n, 0.0), 1.0)
}

/// A resolved SVG style property, including its required quote/semicolon boundary.
pub fn declaration<'a>(shape: &'a str, property: &str) -> Option<&'a str> {
    shape.split(['"', ';']).skip(1).find_map(|s| s.strip_prefix(property).and_then(|s| s.strip_prefix(':')))
}

fn number(raw: &str) -> Option<(f64, &str)> {
    let b = raw.as_bytes();
    let mut i = 0;
    if matches!(b.get(i), Some(b'+' | b'-')) {
        i += 1;
    }
    let from = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let whole = i - from;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let from = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == from {
            return None;
        }
    } else if whole == 0 {
        return None;
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let from = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == from {
            return None;
        }
    }
    let unit = &raw[i..];
    matches!(unit, "" | "%" | "deg" | "grad" | "rad" | "turn").then(|| (js::parse_float(raw), unit))
}
fn amount(raw: &str, full: f64) -> f64 {
    let Some((n, unit)) = number(raw) else { return f64::NAN };
    if !unit.is_empty() && unit != "%" {
        return f64::NAN;
    }
    clamp(n / if unit == "%" { 100.0 } else { full })
}
fn hue(raw: &str) -> f64 {
    let Some((n, unit)) = number(raw) else { return f64::NAN };
    match unit {
        "%" => f64::NAN,
        "turn" => n * 360.0,
        "rad" => n * 180.0 / std::f64::consts::PI,
        "grad" => n * 0.9,
        _ => n,
    }
}
fn args(body: &str, hsl: bool) -> Option<Vec<String>> {
    let legacy = body.contains(',');
    let parts: Vec<_> = if legacy { vec![body] } else { body.split('/').collect() };
    if parts.len() > 2 {
        return None;
    }
    let mut list: Vec<String> = if legacy {
        js::trim(parts[0]).split(',').map(|s| js::trim(s).to_owned()).collect()
    } else {
        js::trim(parts[0]).split(crate::util::chars::is_space).filter(|s| !s.is_empty()).map(str::to_owned).collect()
    };
    if if legacy { list.len() < 3 || list.len() > 4 } else { list.len() != 3 } {
        return None;
    }
    if let Some(alpha) = parts.get(1) {
        list.push(js::trim(alpha).to_owned());
    }
    if !legacy {
        for c in &mut list {
            if c == "none" {
                *c = "0".into();
            }
        }
        return Some(list);
    }
    let percent = list[if hsl { 1 } else { 0 }..3].iter().filter(|c| c.ends_with('%')).count();
    (percent == if hsl { 2 } else { 3 } || (!hsl && percent == 0)).then_some(list)
}
fn color(raw: &str) -> Option<(f64, f64)> {
    let value = js::trim(raw).to_lowercase();
    let name = value.replacen("grey", "gray", 1);
    if DARK.split_whitespace().any(|s| s == name) {
        return Some((0.0, 1.0));
    }
    if LIGHT.split_whitespace().any(|s| s == name) {
        return Some((1.0, 1.0));
    }
    if let Some(hex) = value.strip_prefix('#').filter(|h| matches!(h.len(), 3 | 4 | 6 | 8) && h.bytes().all(|b| b.is_ascii_hexdigit())) {
        let digits = if hex.len() < 6 { hex.chars().flat_map(|c| [c, c]).collect::<String>() } else { hex.to_owned() };
        let n = |i: usize| u8::from_str_radix(&digits[2 * i..2 * i + 2], 16).unwrap() as f64 / 255.0;
        return Some((luminance(n(0), n(1), n(2)), if digits.len() == 8 { n(3) } else { 1.0 }));
    }
    let (fun, body) = value.split_once('(')?;
    let body = body.strip_suffix(')')?;
    if !matches!(fun, "rgb" | "rgba" | "hsl" | "hsla") || body.contains(['(', ')']) {
        return None;
    }
    let hsl = fun.starts_with("hsl");
    let list = args(body, hsl)?;
    let alpha = if list.len() == 4 { amount(&list[3], 1.0) } else { 1.0 };
    let rgb = if !hsl {
        [amount(&list[0], 255.0), amount(&list[1], 255.0), amount(&list[2], 255.0)]
    } else {
        let h = ((hue(&list[0]) % 360.0) + 360.0) % 360.0 / 30.0;
        let s = amount(&list[1], 100.0);
        let l = amount(&list[2], 100.0);
        let a = s * js::min(l, 1.0 - l);
        [0.0, 8.0, 4.0].map(|k| {
            let t = (k + h) % 12.0;
            l - a * js::max(-1.0, js::min(js::min(t - 3.0, 9.0 - t), 1.0))
        })
    };
    let lum = luminance(rgb[0], rgb[1], rgb[2]);
    (!(lum + alpha).is_nan()).then_some((lum, alpha))
}
fn opacity(shape: &str) -> f64 {
    shape
        .split(['"', ';'])
        .skip(1)
        .filter_map(|s| s.strip_prefix("fill-opacity:").or_else(|| s.strip_prefix("opacity:")))
        .fold(1.0, |product, value| product * amount(js::trim(value), 1.0))
}
fn pick(shape: &str, text: &str) -> Option<&'static str> {
    let fill = declaration(shape, "fill")?;
    if declaration(text, "fill").is_some() {
        return None;
    }
    let (lum, alpha) = color(fill)?;
    if !(alpha * opacity(shape) >= 0.5) {
        return None;
    }
    let even = (1.05 * (luminance(31.0 / 255.0, 31.0 / 255.0, 31.0 / 255.0) + 0.05)).sqrt() - 0.05;
    Some(if lum > even { ON_LIGHT } else { ON_DARK })
}

pub fn ink(shape: &str, text: &str) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static KNOWN: OnceLock<Mutex<HashMap<String, Option<&'static str>>>> = OnceLock::new();
    let known = KNOWN.get_or_init(|| Mutex::new(HashMap::new()));
    let key = format!("{shape}\0{text}");
    if let Some(value) = known.lock().ok().and_then(|map| map.get(&key).copied()) {
        return value;
    }
    let value = pick(shape, text);
    if let Ok(mut map) = known.lock() {
        if map.len() > 1000 {
            map.clear();
        }
        map.insert(key, value);
    }
    value
}

/// A sequence diagram reads each raw rect/box color once, including unknown colors.
#[derive(Default)]
pub(crate) struct InkCache(std::cell::RefCell<std::collections::HashMap<String, Option<&'static str>>>);
impl InkCache {
    pub fn get(&self, fill: Option<&str>) -> Option<&'static str> {
        let fill = fill.filter(|s| !s.is_empty())?;
        if let Some(value) = self.0.borrow().get(fill) {
            return *value;
        }
        let shape = super::theme::resolve_style(&[format!("fill:{fill}")]).shape;
        let value = ink(&shape, "");
        self.0.borrow_mut().insert(fill.to_owned(), value);
        value
    }
    pub fn attribute(&self, fill: Option<&str>) -> String {
        self.get(fill).map_or_else(String::new, |s| format!(" fill=\"{s}\""))
    }
}
pub fn paints(shape: &str) -> bool {
    let Some(fill) = declaration(shape, "fill") else { return false };
    let fill = js::trim(fill).to_lowercase();
    if fill.is_empty() || matches!(fill.as_str(), "none" | "transparent") {
        return false;
    }
    let scale = opacity(shape);
    color(&fill).map_or(1.0, |(_, alpha)| alpha) * if scale.is_nan() { 1.0 } else { scale } > 0.0
}
pub fn ink_fill(style: &ResolvedStyle, fallback: Option<&str>) -> String {
    ink(&style.shape, &style.text).or(fallback).filter(|s| !s.is_empty()).map_or_else(String::new, |s| format!(" fill=\"{s}\""))
}
