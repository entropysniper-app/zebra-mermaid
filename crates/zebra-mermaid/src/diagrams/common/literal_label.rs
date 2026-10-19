//! Labels that keep every character. Ported from Pele `src/diagrams/common/literal-label.ts`.

use crate::svg::builder::{esc, label_svg, num, shared_style, span_style, text_style};
use crate::text::{Label, Measure, Span};
use crate::util::chars::is_space;
use crate::util::js;
use std::fmt::Write as _;

/// Literal labels, including filenames and protocol fields, must retain every character but
/// the spaces a line breaks at. Long words can break between code points; measurement stays
/// bounded by one line of text.
pub fn literal_label(text: &str, m: &Measure, size: f64, room: f64, style: u8) -> Label {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let chars: Vec<char> = raw.chars().collect();
        if chars.is_empty() {
            lines.push(String::new());
        }
        let mut from = 0;
        while from < chars.len() {
            let fits = |length: usize| -> bool {
                let s: String = chars[from..from + length].iter().collect();
                m.width(&s, size, style) <= room
            };
            let mut hi = 1usize;
            while hi < chars.len() - from && fits(hi) {
                hi = (chars.len() - from).min(hi * 2);
            }
            let mut lo = 1usize;
            while lo < hi {
                let mid = (lo + hi).div_ceil(2);
                if fits(mid) {
                    lo = mid;
                } else {
                    hi = mid - 1;
                }
            }
            let mut take = lo;
            let wraps = from + take < chars.len();
            if wraps && !is_space(chars[from + take]) {
                let mut i = take - 1;
                while i > 0 {
                    if is_space(chars[from + i]) {
                        take = i + 1;
                        break;
                    }
                    i -= 1;
                }
            }
            let line: String = chars[from..from + take].iter().collect();
            if wraps {
                let trimmed = js::trim_end(&line);
                lines.push(if trimmed.is_empty() { line.clone() } else { trimmed.to_string() });
            } else {
                lines.push(line);
            }
            from += take;
            while wraps && from < chars.len() && is_space(chars[from]) {
                from += 1;
            }
        }
    }
    if text.is_empty() {
        lines.clear();
    }
    let widths: Vec<f64> = lines.iter().map(|line| m.width(line, size, style)).collect();
    let mut width: f64 = 0.0;
    for &w in &widths {
        width = js::max(width, w);
    }
    let line_height = js::round(size * 1.5);
    let height = lines.len() as f64 * line_height;
    Label {
        lines: lines.into_iter().zip(&widths).map(|(text, &w)| vec![Span { text, style, icon: None, width: w }]).collect(),
        widths,
        width,
        height,
        size,
        line_height,
    }
}

/// Breaks the lines of a label that are wider than `room` between characters.
pub fn break_label(label: &Label, m: &Measure, room: f64) -> Label {
    if label.width <= room {
        return label.clone();
    }
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();
    for (i, line) in label.lines.iter().enumerate() {
        if label.widths[i] <= room {
            lines.push(line.clone());
            widths.push(label.widths[i]);
            continue;
        }
        let mut current: Vec<Span> = Vec::new();
        let mut width = 0.0;
        for span in line {
            if span.icon.is_some() {
                if !current.is_empty() && width + span.width > room {
                    lines.push(std::mem::take(&mut current));
                    widths.push(width);
                    width = 0.0;
                }
                current.push(span.clone());
                width += span.width;
                continue;
            }
            for ch in span.text.chars() {
                let joined = matches!(current.last(), Some(last) if last.icon.is_none() && last.style == span.style);
                let next_width = if joined {
                    let last = current.last().unwrap();
                    m.width(&format!("{}{}", last.text, ch), label.size, span.style)
                } else {
                    m.width(&ch.to_string(), label.size, span.style)
                };
                let mut added = next_width - if joined { current.last().unwrap().width } else { 0.0 };
                let mut can_join = joined;
                if !current.is_empty() && width + added > room {
                    lines.push(std::mem::take(&mut current));
                    widths.push(width);
                    width = 0.0;
                    can_join = false;
                    added = m.width(&ch.to_string(), label.size, span.style);
                }
                if can_join {
                    let last = current.last_mut().unwrap();
                    last.text.push(ch);
                    last.width += added;
                } else {
                    current.push(Span { text: ch.to_string(), style: span.style, icon: None, width: added });
                }
                width += added;
            }
        }
        if !current.is_empty() || line.is_empty() {
            lines.push(current);
            widths.push(width);
        }
    }
    let mut width: f64 = 0.0;
    for &w in &widths {
        width = js::max(width, w);
    }
    let height = lines.len() as f64 * label.line_height;
    Label { lines, widths, width, height, size: label.size, line_height: label.line_height }
}

/// Writes a label with every line starting at x. A line is anchored at its start and keeps its
/// spaces, so that where it begins does not depend on how its text was measured.
pub fn left_label_svg(label: &Label, x: f64, top: f64, attrs: &str) -> String {
    let shared = shared_style(label);
    let open = format!("<text{}", text_style(attrs, shared));
    let mut body = String::new();
    for (i, spans) in label.lines.iter().enumerate() {
        let cy = top + (i as f64 + 0.5) * label.line_height;
        if spans.iter().any(|s| s.icon.is_some()) {
            let width = label.widths[i];
            let single = Label {
                lines: vec![spans.clone()],
                widths: vec![width],
                width,
                height: label.line_height,
                size: label.size,
                line_height: label.line_height,
            };
            write!(body, "{}", label_svg(&single, x + width / 2.0, cy, attrs, None)).unwrap();
            continue;
        }
        let mut text = String::new();
        for span in spans {
            let own = span_style(span.style & !shared);
            if own.is_empty() {
                text.push_str(&esc(&span.text));
            } else {
                write!(text, "<tspan{}>{}</tspan>", own, esc(&span.text)).unwrap();
            }
        }
        if !text.is_empty() {
            write!(body, 
                "{} x=\"{}\" y=\"{}\" text-anchor=\"start\" xml:space=\"preserve\">{}</text>",
                open,
                num(x),
                num(cy + label.size * 0.35),
                text
            ).unwrap();
        }
    }
    body
}
