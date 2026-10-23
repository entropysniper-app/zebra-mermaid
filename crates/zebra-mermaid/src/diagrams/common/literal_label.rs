//! Labels that keep every character. Ported from Pele `src/diagrams/common/literal-label.ts`.

use crate::svg::builder::{label_svg, shared_style_view, span_style, text_style, write_esc, write_num};
use crate::text::render_label::{LabelRead, LabelView};
use crate::text::{Label, Measure, RenderLabel, Span};
use crate::util::chars::is_space;
use crate::util::js;
use std::fmt;

/// Literal labels, including filenames and protocol fields, must retain every character but
/// the spaces a line breaks at. Long words can break between code points; measurement stays
/// bounded by one line of text.
pub fn literal_label(text: &str, m: &Measure, size: f64, room: f64, style: u8) -> Label {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        // ASCII scalar indices are already byte offsets. Other strings need one
        // boundary table, shared by every probe instead of allocating each prefix.
        let boundaries =
            (!raw.is_ascii()).then(|| raw.char_indices().map(|(i, _)| i).chain(std::iter::once(raw.len())).collect::<Vec<_>>());
        let count = boundaries.as_ref().map_or(raw.len(), |v| v.len() - 1);
        let byte_at = |i: usize| boundaries.as_ref().map_or(i, |v| v[i]);
        let space_at = |i: usize| is_space(raw[byte_at(i)..].chars().next().unwrap());
        if count == 0 {
            lines.push(String::new());
        }
        let mut from = 0;
        while from < count {
            let start = byte_at(from);
            let fits = |length: usize| m.width(&raw[start..byte_at(from + length)], size, style) <= room;
            let mut hi = 1usize;
            while hi < count - from && fits(hi) {
                hi = (count - from).min(hi * 2);
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
            let wraps = from + take < count;
            if wraps && !space_at(from + take) {
                let mut i = take - 1;
                while i > 0 {
                    if space_at(from + i) {
                        take = i + 1;
                        break;
                    }
                    i -= 1;
                }
            }
            let line = &raw[start..byte_at(from + take)];
            let trimmed = if wraps { js::trim_end(line) } else { line };
            lines.push(if trimmed.is_empty() { line } else { trimmed }.to_owned());
            from += take;
            while wraps && from < count && space_at(from) {
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
        lines: lines.into_iter().zip(&widths).map(|(text, &w)| vec![Span { text, style, icon: None, math: None, width: w }]).collect(),
        widths,
        width,
        height,
        size,
        line_height,
        heights: None,
    }
}

/// Breaks the lines of a label that are wider than `room` between characters.
pub fn break_label(label: &Label, m: &Measure, room: f64) -> Label {
    if label.width <= room {
        return label.clone();
    }
    break_view(label.view(), m, room)
}

pub(crate) fn break_render_label(label: &RenderLabel, m: &Measure, room: f64) -> RenderLabel {
    if label.width <= room {
        return label.clone();
    }
    RenderLabel::from_owned(break_view(label.view(), m, room))
}

pub(crate) fn literal_render_label(text: &str, m: &Measure, size: f64, room: f64, style: u8) -> RenderLabel {
    RenderLabel::from_owned(literal_label(text, m, size, room, style))
}

fn break_view(label: LabelView<'_>, m: &Measure, room: f64) -> Label {
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();
    for (i, line) in label.lines.iter().enumerate() {
        if label.widths[i] <= room {
            lines.push(line.to_vec());
            widths.push(label.widths[i]);
            continue;
        }
        let mut current: Vec<Span> = Vec::new();
        let mut width = 0.0;
        for span in line {
            if span.icon.is_some() || span.math.is_some() {
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
                let mut bytes = [0; 4];
                let character = ch.encode_utf8(&mut bytes);
                let joined = matches!(current.last(), Some(last) if last.icon.is_none() && last.math.is_none() && last.style == span.style);
                // Measure the actual mutable prefix. If it wraps, remove the candidate
                // before committing the old line; callbacks see the same strings/order.
                let next_width = if joined {
                    let last = current.last_mut().unwrap();
                    last.text.push(ch);
                    m.width(&last.text, label.size, span.style)
                } else {
                    m.width(character, label.size, span.style)
                };
                let mut added = next_width - if joined { current.last().unwrap().width } else { 0.0 };
                let mut can_join = joined;
                if !current.is_empty() && width + added > room {
                    if joined {
                        let last = current.last_mut().unwrap();
                        last.text.truncate(last.text.len() - ch.len_utf8());
                    }
                    lines.push(std::mem::take(&mut current));
                    widths.push(width);
                    width = 0.0;
                    can_join = false;
                    added = m.width(character, label.size, span.style);
                }
                if can_join {
                    current.last_mut().unwrap().width += added;
                } else {
                    current.push(Span { text: character.to_owned(), style: span.style, icon: None, math: None, width: added });
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
    let mut broken = Label { lines, widths, width, height, size: label.size, line_height: label.line_height, heights: None };
    if label.heights.is_some() {
        crate::text::label::with_heights(&mut broken);
    }
    broken
}

/// Writes a label with every line starting at x. A line is anchored at its start and keeps its
/// spaces, so that where it begins does not depend on how its text was measured.
pub fn left_label_svg(label: &Label, x: f64, top: f64, attrs: &str) -> String {
    let mut body = String::new();
    push_left_label_svg(&mut body, label, x, top, attrs);
    body
}

pub(crate) fn render_left_label_svg(label: &RenderLabel, x: f64, top: f64, attrs: &str) -> String {
    let mut body = String::new();
    push_left_label_svg(&mut body, label, x, top, attrs);
    body
}

pub(crate) fn push_left_label_svg(out: &mut String, label: &impl LabelRead, x: f64, top: f64, attrs: &str) {
    let _ = write_left_label_svg(out, &label.view(), x, top, attrs);
}

pub(crate) struct LeftLabelSvg<'a, L: LabelRead> {
    label: &'a L,
    x: f64,
    top: f64,
    attrs: &'a str,
}

pub(crate) fn left_label_markup<'a, L: LabelRead>(label: &'a L, x: f64, top: f64, attrs: &'a str) -> LeftLabelSvg<'a, L> {
    LeftLabelSvg { label, x, top, attrs }
}

impl<L: LabelRead> fmt::Display for LeftLabelSvg<'_, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_left_label_svg(f, &self.label.view(), self.x, self.top, self.attrs)
    }
}

impl<L: LabelRead> crate::svg::concat::Part for LeftLabelSvg<'_, L> {
    #[inline]
    fn append(&self, out: &mut String) {
        let _ = write_left_label_svg(out, &self.label.view(), self.x, self.top, self.attrs);
    }
}

fn write_left_label_svg<W: fmt::Write + ?Sized>(out: &mut W, label: &LabelView<'_>, x: f64, top: f64, attrs: &str) -> fmt::Result {
    let shared = shared_style_view(label);
    let open = text_style(attrs, shared);
    for (i, spans) in label.lines.iter().enumerate() {
        let cy = top + label.heights.map_or((i as f64 + 0.5) * label.line_height, |h| h[..i].iter().sum::<f64>() + h[i] / 2.0);
        if spans.iter().any(|s| s.icon.is_some() || s.math.is_some()) {
            let width = label.widths[i];
            let single = Label {
                lines: vec![spans.to_vec()],
                widths: vec![width],
                width,
                height: label.heights.map_or(label.line_height, |h| h[i]),
                size: label.size,
                line_height: label.line_height,
                heights: label.heights.as_ref().map(|h| vec![h[i]]),
            };
            write!(out, "{}", label_svg(&single, x + width / 2.0, cy, attrs, None))?;
            continue;
        }
        // The old temporary markup is empty only when every span has neither a
        // style wrapper nor an XML-visible character. Keep that emission rule.
        let visible = spans.iter().any(|span| {
            !span_style(span.style & !shared).is_empty()
                || span.text.bytes().any(|b| !matches!(b, 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f))
        });
        if !visible {
            continue;
        }
        out.write_str("<text")?;
        out.write_str(&open)?;
        out.write_str(" x=\"")?;
        write_num(out, x)?;
        out.write_str("\" y=\"")?;
        write_num(out, cy + label.size * 0.35)?;
        out.write_str("\" text-anchor=\"start\" xml:space=\"preserve\">")?;
        for span in spans {
            let own = span_style(span.style & !shared);
            if !own.is_empty() {
                out.write_str("<tspan")?;
                out.write_str(&own)?;
                out.write_char('>')?;
            }
            write_esc(out, &span.text)?;
            if !own.is_empty() {
                out.write_str("</tspan>")?;
            }
        }
        out.write_str("</text>")?;
    }
    Ok(())
}

#[cfg(test)]
mod adapter_reference {
    use super::*;
    use crate::svg::builder::adapter_reference::{bits, label};
    use serde_json::Value;

    #[test]
    fn direct_left_label_parts_match_independent_node_svg() {
        let data: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../../../../../tests/fixtures/pele-1.9.0/unit/literal-label-regression.json.gz")[..],
        ))
        .unwrap();
        assert_eq!(data["upstreamCommit"], crate::PELE_COMMIT);
        assert_eq!(data["seed"], "0x17e2b6");
        let cases = data["svg"].as_array().unwrap();
        assert_eq!(cases.len(), 444);
        for (index, case) in cases.iter().enumerate() {
            let owned = label(&case["label"]);
            let compact = RenderLabel::from_owned(owned.clone());
            let x = bits(case["x"].as_str().unwrap());
            let top = bits(case["top"].as_str().unwrap());
            let attrs = case["attrs"].as_str().unwrap();
            assert_eq!(
                crate::svg::concat::svg_format!(left_label_markup(&owned, x, top, attrs)),
                case["expected"].as_str().unwrap(),
                "owned {index}"
            );
            assert_eq!(
                crate::svg::concat::svg_format!(left_label_markup(&compact, x, top, attrs)),
                case["expected"].as_str().unwrap(),
                "render {index}"
            );
        }
    }
}
