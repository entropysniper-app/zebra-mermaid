//! Fill-aware edge paint. Ported from Pele 1.12.0 `src/svg/edges.ts` (MIT).
use super::{marker, EdgePath};
use crate::svg::{builder::num, ink};
use crate::util::{js, math};

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub bounds: [f64; 4],
    pub shape: String,
}
pub enum FillUnder<'a> {
    Crossed,
    Inside(&'a Fill),
}

fn tokenize(d: &str) -> Vec<f64> {
    let b = d.as_bytes();
    let mut out = Vec::new();
    let mut at = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_uppercase() && c != b'E' {
            at = Some(out.len() + 1);
            out.extend([c as f64, 0.0]);
            i += 1;
        } else if c == b'-' || c == b'.' || c.is_ascii_digit() {
            let mut j = i + usize::from(c == b'-');
            let mut m = 0.0;
            let mut places = -1i32;
            while let Some(&k) = b.get(j).filter(|&&k| k.is_ascii_digit() || (k == b'.' && places < 0)) {
                if k == b'.' {
                    places = 0;
                } else {
                    m = m * 10.0 + (k - b'0') as f64;
                    if places >= 0 {
                        places += 1;
                    }
                }
                j += 1;
            }
            let v = if b.get(j).is_some_and(|&k| k | 32 == b'e') || m > 9007199254740991.0 || places > 22 {
                while b.get(j).is_some_and(|&k| {
                    k.is_ascii_digit() || matches!(k, b'.' | b'e' | b'E') || (matches!(k, b'-' | b'+') && j > 0 && b[j - 1] | 32 == b'e')
                }) {
                    j += 1;
                }
                js::number(&d[i..j])
            } else {
                let v = if places > 0 { m / 10f64.powi(places) } else { m };
                if c == b'-' {
                    -v
                } else {
                    v
                }
            };
            if let Some(at) = at {
                out.push(v);
                out[at] += 1.0;
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}
fn arc(out: &mut Vec<f64>, x1: f64, y1: f64, mut rx: f64, mut ry: f64, large: f64, sweep: f64, x2: f64, y2: f64) {
    let hx = (x1 - x2) / 2.0;
    let hy = (y1 - y2) / 2.0;
    if rx == 0.0 || ry == 0.0 || (hx == 0.0 && hy == 0.0) {
        out.extend([x2, y2]);
        return;
    }
    let scale = js::max(1.0, ((hx * hx) / (rx * rx) + (hy * hy) / (ry * ry)).sqrt());
    rx *= scale;
    ry *= scale;
    let k = (if large == sweep { -1.0 } else { 1.0 })
        * js::max(0.0, (rx * rx * ry * ry - rx * rx * hy * hy - ry * ry * hx * hx) / (rx * rx * hy * hy + ry * ry * hx * hx)).sqrt();
    let cx = (k * rx * hy) / ry + (x1 + x2) / 2.0;
    let cy = (-k * ry * hx) / rx + (y1 + y2) / 2.0;
    let from = math::atan2((y1 - cy) / ry, (x1 - cx) / rx);
    let mut turn = math::atan2((y2 - cy) / ry, (x2 - cx) / rx) - from;
    if sweep != 0.0 && turn < 0.0 {
        turn += 2.0 * std::f64::consts::PI;
    } else if sweep == 0.0 && turn > 0.0 {
        turn -= 2.0 * std::f64::consts::PI;
    }
    for i in 1..=8 {
        out.extend([cx + rx * math::cos(from + (turn * i as f64) / 8.0), cy + ry * math::sin(from + (turn * i as f64) / 8.0)]);
    }
}
fn sample(tokens: &[f64]) -> Vec<f64> {
    let mut out = Vec::new();
    let (mut x, mut y) = (0.0, 0.0);
    let mut i = 0;
    while i + 1 < tokens.len() {
        let cmd = tokens[i] as u8;
        let count = tokens[i + 1] as usize;
        let s = i + 2;
        let e = (s + count).min(tokens.len());
        i = e;
        if matches!(cmd, b'H' | b'V') {
            for &v in &tokens[s..e] {
                if cmd == b'H' {
                    x = v;
                } else {
                    y = v;
                }
                out.extend([x, y]);
            }
        } else if matches!(cmd, b'C' | b'Q') {
            let cubic = cmd == b'C';
            let step = if cubic { 6 } else { 4 };
            let mut k = s;
            while k + step <= e {
                let (x1, y1, x2, y2) = (tokens[k], tokens[k + 1], tokens[k + 2], tokens[k + 3]);
                let (x3, y3) = if cubic { (tokens[k + 4], tokens[k + 5]) } else { (x2, y2) };
                for n in 1..=8 {
                    let t = n as f64 * 0.125;
                    let (mut ax, mut ay) = (x + (x1 - x) * t, y + (y1 - y) * t);
                    let (mut bx, mut by) = (x1 + (x2 - x1) * t, y1 + (y2 - y1) * t);
                    if cubic {
                        let (cx, cy) = (x2 + (x3 - x2) * t, y2 + (y3 - y2) * t);
                        ax += (bx - ax) * t;
                        ay += (by - ay) * t;
                        bx += (cx - bx) * t;
                        by += (cy - by) * t;
                    }
                    out.extend([ax + (bx - ax) * t, ay + (by - ay) * t]);
                }
                x = x3;
                y = y3;
                k += step;
            }
        } else if cmd == b'A' {
            let mut k = s;
            while k + 7 <= e {
                arc(&mut out, x, y, tokens[k], tokens[k + 1], tokens[k + 3], tokens[k + 4], tokens[k + 5], tokens[k + 6]);
                x = tokens[k + 5];
                y = tokens[k + 6];
                k += 7;
            }
        } else {
            let mut k = s;
            while k + 2 <= e {
                if cmd == b'M' && k == s && !out.is_empty() {
                    out.extend([f64::NAN, f64::NAN]);
                }
                x = tokens[k];
                y = tokens[k + 1];
                out.extend([x, y]);
                k += 2;
            }
        }
    }
    out
}
fn clips(x: f64, y: f64, dx: f64, dy: f64, l: f64, t: f64, r: f64, b: f64) -> bool {
    let (mut lo, mut hi) = (0.0, 1.0);
    for (p, q) in [(-dx, x - l), (dx, r - x), (-dy, y - t), (dy, b - y)] {
        if p == 0.0 {
            if q < 0.0 {
                hi = -1.0;
            }
        } else if p < 0.0 {
            lo = js::max(lo, q / p);
        } else {
            hi = js::min(hi, q / p);
        }
    }
    lo < hi
}
fn enters(points: &[f64], l: f64, t: f64, r: f64, b: f64) -> bool {
    for p in points.windows(4).step_by(2) {
        let (x, y, x2, y2) = (p[0], p[1], p[2], p[3]);
        if (x < l && x2 < l) || (x > r && x2 > r) || (y < t && y2 < t) || (y > b && y2 > b) {
            continue;
        }
        if clips(x, y, x2 - x, y2 - y, l, t, r, b) {
            return true;
        }
    }
    false
}
pub fn fill_under<'a>(d: &str, tips: &[f64], fills: &'a [Fill], tokens: Option<&[f64]>) -> Option<FillUnder<'a>> {
    if fills.is_empty() {
        return None;
    }
    let parsed;
    let points = sample(if let Some(t) = tokens {
        t
    } else {
        parsed = tokenize(d);
        &parsed
    });
    let (mut minx, mut miny, mut maxx, mut maxy) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in points.chunks_exact(2).chain(tips.chunks_exact(2)) {
        if p[0] < minx {
            minx = p[0];
        }
        if p[0] > maxx {
            maxx = p[0];
        }
        if p[1] < miny {
            miny = p[1];
        }
        if p[1] > maxy {
            maxy = p[1];
        }
    }
    let mut holder = None;
    let mut least = f64::INFINITY;
    for fill in fills {
        let [l, t, r, b] = fill.bounds;
        if maxx < l + 0.5 || minx > r - 0.5 || maxy < t + 0.5 || miny > b - 0.5 {
            continue;
        }
        let touched = enters(&points, l + 0.5, t + 0.5, r - 0.5, b - 0.5)
            || tips.chunks_exact(2).any(|p| p[0] > l + 0.5 && p[0] < r - 0.5 && p[1] > t + 0.5 && p[1] < b - 0.5);
        if !touched {
            continue;
        }
        if minx < l || maxx > r || miny < t || maxy > b {
            return Some(FillUnder::Crossed);
        }
        let area = (r - l) * (b - t);
        if area < least {
            least = area;
            holder = Some(FillUnder::Inside(fill));
        }
    }
    holder
}
pub fn edge_ink(fill: &Fill, line: &str) -> &'static str {
    if ink::declaration(line, "stroke").is_some() {
        ""
    } else {
        ink::ink(&fill.shape, "").unwrap_or("")
    }
}
pub fn over_fills(d: &str, tips: &[f64], fills: &[Fill], line: &str, tokens: Option<&[f64]>) -> Option<&'static str> {
    fill_under(d, tips, fills, tokens).map(|fill| match fill {
        FillUnder::Crossed => "",
        FillUnder::Inside(fill) => edge_ink(fill, line),
    })
}
pub fn hollow_on(markers: &str, fill: &Fill, background: &str) -> String {
    let Some(color) = ink::declaration(&fill.shape, "fill") else { return markers.to_owned() };
    markers
        .replace(" fill=\"var(--_bg)\"", &format!(" fill=\"{}\"", js::trim(color)))
        .replace(&format!(" fill=\"{background}\""), &format!(" fill=\"{}\"", js::trim(color)))
}
pub fn casing(d: &str, markers: &str, line: &str, thick: bool) -> String {
    let decl = |p| ink::declaration(line, p).map(js::trim).filter(|s| !s.is_empty());
    if matches!(decl("stroke"), Some("none" | "transparent")) {
        return String::new();
    }
    let width = if thick { Some("2.5") } else { decl("stroke-width") };
    let mut attrs = String::new();
    let mut style = String::new();
    if let Some(width) = width {
        let number = width.strip_suffix("px").unwrap_or(width);
        if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            let n = js::parse_float(width);
            if n != 1.0 {
                attrs = format!(" stroke-width=\"{}\"", num(n + 4.0));
            }
        } else {
            style.push_str(&format!("stroke-width:calc({width} + 4px);"));
        }
    }
    for prop in ["opacity", "stroke-opacity"] {
        if let Some(value) = decl(prop) {
            style.push_str(&format!("{prop}:{value};"));
        }
    }
    let body = format!("<path d=\"{d}\"/>{}", markers.replace(" class=\"pele-marker\"", ""));
    if attrs.is_empty() && style.is_empty() {
        body
    } else {
        format!("<g{attrs}{}>{body}</g>", if style.is_empty() { String::new() } else { format!(" style=\"{style}\"") })
    }
}
pub fn edge_casing(path: &EdgePath, start: &str, end: &str, line: &str, thick: bool) -> String {
    let markers = format!(
        "{}{}",
        marker(start, path.sx, path.sy, path.sdx, path.sdy, "var(--_bg)"),
        marker(end, path.ex, path.ey, path.edx, path.edy, "var(--_bg)")
    )
    .replace(" stroke=\"none\"", " stroke=\"var(--_bg)\"");
    casing(&path.d, &markers, line, thick)
}
pub fn casings_svg(body: &str) -> String {
    if body.is_empty() {
        String::new()
    } else {
        format!("<g class=\"pele-casings\" fill=\"none\" stroke=\"var(--_bg)\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" pointer-events=\"none\">{body}</g>")
    }
}
