//! Label layout: line breaks, inline emphasis, icons and wrapping. Ported from Pele
//! `src/text/label.ts`.
//!
//! The regular expressions of the original are written out as scanners here. Each keeps the
//! JavaScript semantics it replaces: `\s` is JavaScript white space, `\w` is ASCII, and the
//! `i` flag makes the letters of `<br>` and `\n` match in either case.

use super::entities::decode_entities_cow;
use super::measurer::style;
use super::Measure;
use crate::util::chars::{char_at, char_before, is_space, is_word};
use crate::util::{js, strs};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: u8,
    pub icon: Option<String>,
    pub width: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Label {
    pub lines: Vec<Vec<Span>>,
    pub widths: Vec<f64>,
    pub width: f64,
    pub height: f64,
    pub size: f64,
    pub line_height: f64,
}

impl Label {
    pub fn empty() -> Label {
        Label::default()
    }
}

pub const LINE_HEIGHT: f64 = 1.5;

/// `[⺀-鿿豈-﫿＀-｠]`: Chinese and Japanese characters, between any
/// two of which a line may break. Hangul is left out: Korean puts spaces between words.
#[inline]
pub fn is_cjk(c: char) -> bool {
    let c = c as u32;
    (0x2e80..=0x9fff).contains(&c) || (0xf900..=0xfaff).contains(&c) || (0xff00..=0xff60).contains(&c)
}

const CLOSE: &str = "、。，．：；！？）］｝」』】〕〉》〙〗ー…‥・ゝゞぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ,.:;!?)]}";
const OPEN: &str = "（［｛「『【〔〈《〘〖([{";

#[inline]
fn is_close(c: char) -> bool {
    CLOSE.contains(c)
}

#[inline]
fn is_open(c: char) -> bool {
    OPEN.contains(c)
}

/// `/[<\\\n*_]|fa[bklrs]?:fa-/.test(raw)`
fn has_markup(raw: &str) -> bool {
    if raw.bytes().any(|b| matches!(b, b'<' | b'\\' | b'\n' | b'*' | b'_')) {
        return true;
    }
    find_icon_prefix(raw, 0).is_some()
}

/// The start of the next `fa[bklrs]?:fa-` at or after `from`, and the end of its prefix.
fn find_icon_prefix(text: &str, from: usize) -> Option<(usize, usize)> {
    let b = text.as_bytes();
    let mut i = from;
    while i + 1 < b.len() {
        if b[i] == b'f' && b[i + 1] == b'a' {
            let mut j = i + 2;
            if j < b.len() && matches!(b[j], b'b' | b'k' | b'l' | b'r' | b's') && text[j + 1..].starts_with(":fa-") {
                j += 1;
            }
            if text[j..].starts_with(":fa-") {
                return Some((i, j + 4));
            }
        }
        i += 1;
    }
    None
}

/// The next match of `/(fa[bklrs]?):fa-([\w-]+)/g` at or after `from`: start and end.
fn find_icon(text: &str, from: usize) -> Option<(usize, usize)> {
    let b = text.as_bytes();
    let mut at = from;
    while let Some((start, prefix_end)) = find_icon_prefix(text, at) {
        let mut end = prefix_end;
        while end < b.len() && (is_word(b[end] as char) || b[end] == b'-') {
            end += 1;
        }
        if end > prefix_end {
            return Some((start, end));
        }
        at = start + 1;
    }
    None
}

/// Splits on `/<br\s*\/?>|\\n|\n/gi`.
fn split_breaks(source: &str) -> Vec<&str> {
    let b = source.as_bytes();
    let mut parts = Vec::new();
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        let end = match b[i] {
            b'\n' => Some(i + 1),
            b'\\' if i + 1 < b.len() && (b[i + 1] == b'n' || b[i + 1] == b'N') => Some(i + 2),
            b'<' => br_end(source, i),
            _ => None,
        };
        match end {
            Some(e) => {
                parts.push(&source[last..i]);
                last = e;
                i = e;
            }
            None => i += 1,
        }
    }
    parts.push(&source[last..]);
    parts
}

/// The end of `<br\s*\/?>` (letters in either case) at `i`, if there is one.
pub(crate) fn br_end(source: &str, i: usize) -> Option<usize> {
    let b = source.as_bytes();
    if b.get(i) != Some(&b'<') || !matches!(b.get(i + 1), Some(b'b' | b'B')) || !matches!(b.get(i + 2), Some(b'r' | b'R')) {
        return None;
    }
    let mut p = i + 3;
    while let Some(c) = char_at(source, p) {
        if !is_space(c) {
            break;
        }
        p += c.len_utf8();
    }
    if b.get(p) == Some(&b'/') {
        p += 1;
    }
    if b.get(p) == Some(&b'>') {
        Some(p + 1)
    } else {
        None
    }
}

/// `/<\/?([a-zA-Z][\w-]*)(?:\s[^<>]*)?\/?>/y` at `i`: the end of the tag and its name.
fn match_tag(source: &str, i: usize) -> Option<(usize, &str)> {
    let b = source.as_bytes();
    let mut p = i + 1;
    if b.get(p) == Some(&b'/') {
        p += 1;
    }
    if !b.get(p).is_some_and(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let name_start = p;
    p += 1;
    while p < b.len() && (is_word(b[p] as char) || b[p] == b'-') {
        p += 1;
    }
    let name = &source[name_start..p];
    if let Some(c) = char_at(source, p) {
        if is_space(c) {
            let mut q = p + c.len_utf8();
            while q < b.len() && b[q] != b'<' && b[q] != b'>' {
                q += 1;
            }
            if q < b.len() && b[q] == b'>' {
                return Some((q + 1, name));
            }
        }
    }
    if b.get(p) == Some(&b'>') {
        return Some((p + 1, name));
    }
    if b.get(p) == Some(&b'/') && b.get(p + 1) == Some(&b'>') {
        return Some((p + 2, name));
    }
    None
}

fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let indent = |line: &str| line.chars().take_while(|&c| is_space(c)).count();
    let mut min = usize::MAX;
    for line in &lines {
        if js::trim(line).is_empty() {
            continue;
        }
        min = min.min(indent(line));
    }
    if min == usize::MAX || min == 0 {
        return text.to_string();
    }
    let out: Vec<&str> = lines
        .iter()
        .map(|line| {
            let skip = min.min(indent(line));
            let bytes: usize = line.chars().take(skip).map(char::len_utf8).sum();
            &line[bytes..]
        })
        .collect();
    out.join("\n")
}

fn push_text(line: &mut Vec<Span>, text: &str, style: u8) {
    if text.is_empty() {
        return;
    }
    let decoded = decode_entities_cow(text);
    if let Some(last) = line.last_mut() {
        if last.style == style && last.icon.is_none() {
            last.text.push_str(&decoded);
            return;
        }
    }
    line.push(Span { text: decoded.into_owned(), style, icon: None, width: 0.0 });
}

fn push_icons(line: &mut Vec<Span>, text: &str, style: u8, m: &Measure) {
    if !strs::contains(text, ":fa-") {
        push_text(line, text, style);
        return;
    }
    let mut last = 0;
    let mut from = 0;
    while let Some((start, end)) = find_icon(text, from) {
        let icon = &text[start..end];
        push_text(line, &text[last..start], style);
        last = end;
        from = end;
        if !m.icon_markup(icon).is_empty() {
            line.push(Span { text: String::new(), style, icon: Some(icon.to_string()), width: 0.0 });
        } else if text.as_bytes().get(last) == Some(&b' ') {
            // An icon the host does not have leaves no gap, nor the space that set it off.
            last += 1;
        } else if last == text.len() {
            if let Some(before) = line.last_mut() {
                if before.icon.is_none() {
                    before.text.truncate(js::trim_end(&before.text).len());
                }
            }
        }
    }
    push_text(line, &text[last..], style);
}

/// Splits inline HTML and, for markdown labels, emphasis markers into styled spans.
fn inline(line: &mut Vec<Span>, source: &str, markdown: bool, base: u8, m: &Measure) {
    let mut st = base;
    let mut marked: u8 = 0;
    let mut text = String::new();
    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        let ch = char_at(source, i).expect("boundary");
        if ch == '<' {
            if let Some((end, name)) = match_tag(source, i) {
                let tag = name.to_ascii_lowercase();
                let closing = b.get(i + 1) == Some(&b'/');
                let bit = if tag == "b" || tag == "strong" {
                    style::BOLD
                } else if tag == "i" || tag == "em" {
                    style::ITALIC
                } else {
                    0
                };
                if bit != 0 {
                    push_icons(line, &text, st, m);
                    text.clear();
                    st = if closing { (st & !bit) | (base & bit) } else { st | bit };
                }
                i = end;
                continue;
            }
        } else if markdown && (ch == '*' || ch == '_') {
            let double = b.get(i + 1) == Some(&(ch as u8));
            let bit = if double { style::BOLD } else { style::ITALIC };
            let len = if double { 2 } else { 1 };
            let open = marked & bit == 0;
            let prev = if i > 0 { char_before(source, i).unwrap_or(' ') } else { ' ' };
            let next = if i + len < n { char_at(source, i + len).unwrap_or(' ') } else { ' ' };
            let marker = if double {
                if ch == '*' {
                    "**"
                } else {
                    "__"
                }
            } else if ch == '*' {
                "*"
            } else {
                "_"
            };
            let can_open = open && !is_space(next) && source[i + len..].contains(marker);
            let can_close = !open && !is_space(prev);
            let wordy = ch == '_' && is_word(prev) && is_word(next);
            if (can_open || can_close) && !wordy {
                push_icons(line, &text, st, m);
                text.clear();
                marked = if can_open { marked | bit } else { marked & !bit };
                st = if can_open { st | bit } else { (st & !bit) | (base & bit) };
                i += len;
                continue;
            }
        }
        text.push(ch);
        i += ch.len_utf8();
    }
    push_icons(line, &text, st, m);
}

fn measure_spans(line: &mut [Span], m: &Measure, size: f64) -> f64 {
    let mut total = 0.0;
    for span in line.iter_mut() {
        span.width = if span.icon.is_some() { size * 1.25 } else { m.width(&span.text, size, span.style) };
        total += span.width;
    }
    total
}

/// The pieces of `/\s*(?:[OPEN]*[CJK][CLOSE]*|[^\sCJK]+[CLOSE]*)\s*|\s+/g`: a word with the
/// space around it, or one Chinese or Japanese character with its punctuation.
fn piece_iter(text: &str) -> impl Iterator<Item = &str> {
    let mut p = 0;
    std::iter::from_fn(move || {
        while p < text.len() {
            match piece_at(text, p) {
                Some(end) if end > p => {
                    let piece = &text[p..end];
                    p = end;
                    return Some(piece);
                }
                // Every character is white space, CJK or neither, so a match always exists;
                // keep the iterator finite should that ever change.
                _ => p += char_at(text, p).map_or(1, char::len_utf8),
            }
        }
        None
    })
}

#[cfg(test)]
pub(crate) fn pieces(text: &str) -> Vec<&str> {
    piece_iter(text).collect()
}

fn skip_while(text: &str, mut p: usize, f: impl Fn(char) -> bool) -> usize {
    while let Some(c) = char_at(text, p) {
        if !f(c) {
            break;
        }
        p += c.len_utf8();
    }
    p
}

fn group_at(text: &str, s: usize) -> Option<usize> {
    // [OPEN]*[CJK][CLOSE]*, with OPEN* giving back characters until a CJK one follows. The
    // candidates are tried from the end of the run of opening marks back to `s`.
    let q = skip_while(text, s, is_open);
    let mut j = q;
    loop {
        if let Some(c) = char_at(text, j) {
            if is_cjk(c) {
                return Some(skip_while(text, j + c.len_utf8(), is_close));
            }
        }
        if j <= s {
            break;
        }
        j -= char_before(text, j).map_or(1, char::len_utf8);
    }
    // [^\sCJK]+[CLOSE]*
    let e = skip_while(text, s, |c| !is_space(c) && !is_cjk(c));
    if e > s {
        return Some(skip_while(text, e, is_close));
    }
    None
}

fn piece_at(text: &str, p: usize) -> Option<usize> {
    // `\s*` gives back white space until a group follows: the starts are tried from the end
    // of the run back to `p`.
    let q = skip_while(text, p, is_space);
    let mut s = q;
    loop {
        if let Some(g) = group_at(text, s) {
            return Some(skip_while(text, g, is_space));
        }
        if s <= p {
            break;
        }
        s -= char_before(text, s).map_or(1, char::len_utf8);
    }
    if q > p {
        Some(q)
    } else {
        None
    }
}

fn wrap(line: Vec<Span>, m: &Measure, size: f64, max: f64, out: &mut Vec<Vec<Span>>, widths: &mut Vec<f64>) {
    let mut current: Vec<Span> = Vec::new();
    let mut width = 0.0;
    let commit = |current: &mut Vec<Span>, width: &mut f64, out: &mut Vec<Vec<Span>>, widths: &mut Vec<f64>| {
        if let Some(last) = current.last_mut() {
            if last.icon.is_none() {
                let trimmed = js::trim_end(&last.text);
                if trimmed.len() != last.text.len() {
                    let w = m.width(trimmed, size, last.style);
                    *width += w - last.width;
                    last.text.truncate(trimmed.len());
                    last.width = w;
                }
            }
        }
        out.push(std::mem::take(current));
        widths.push(*width);
        *width = 0.0;
    };
    for span in line {
        if span.icon.is_some() {
            let w = size * 1.25;
            if width > 0.0 && width + w > max {
                commit(&mut current, &mut width, out, widths);
            }
            current.push(Span { width: w, ..span });
            width += w;
            continue;
        }
        for piece in piece_iter(&span.text) {
            let mut part = piece;
            if width > 0.0 && width + m.width(js::trim_end(part), size, span.style) > max {
                commit(&mut current, &mut width, out, widths);
                part = js::trim_start(part);
            }
            match current.last_mut() {
                Some(last) if last.style == span.style && last.icon.is_none() => {
                    last.text.push_str(part);
                    last.width = m.width(&last.text, size, last.style);
                    width = 0.0;
                    for s in &current {
                        width += s.width;
                    }
                }
                _ => {
                    let w = m.width(part, size, span.style);
                    current.push(Span { text: part.to_string(), style: span.style, icon: None, width: w });
                    width += w;
                }
            }
        }
    }
    if !current.is_empty() || out.is_empty() {
        commit(&mut current, &mut width, out, widths);
    }
}

/// Lays out a label: splits it into lines at `<br>`, `\n` and line breaks, reads inline
/// `<b>`/`<i>` (and `*`/`_` emphasis when `markdown`), makes room for icons the host has, and
/// wraps lines wider than `max_width` at spaces and between Chinese and Japanese characters.
pub fn layout_label(raw: Option<&str>, markdown: bool, m: &Measure, size: f64, max_width: f64, base: u8) -> Label {
    layout_as::<Label>(raw, markdown, m, size, max_width, base)
}

pub(crate) fn layout_render_label(
    raw: Option<&str>,
    markdown: bool,
    m: &Measure,
    size: f64,
    max_width: f64,
    base: u8,
) -> super::RenderLabel {
    layout_as::<super::RenderLabel>(raw, markdown, m, size, max_width, base)
}

trait LabelOutput {
    fn empty() -> Self;
    fn single(span: Span, size: f64, line_height: f64) -> Self;
    fn from_owned(label: Label) -> Self;
}
impl LabelOutput for Label {
    fn empty() -> Self {
        Label::empty()
    }
    fn single(span: Span, size: f64, line_height: f64) -> Self {
        let width = span.width;
        Self { lines: vec![vec![span]], widths: vec![width], width, height: line_height, size, line_height }
    }
    fn from_owned(label: Label) -> Self {
        label
    }
}
impl LabelOutput for super::RenderLabel {
    fn empty() -> Self {
        Self::empty()
    }
    fn single(span: Span, size: f64, line_height: f64) -> Self {
        Self::single(span, size, line_height)
    }
    fn from_owned(label: Label) -> Self {
        Self::from_owned(label)
    }
}

fn layout_as<T: LabelOutput>(raw: Option<&str>, markdown: bool, m: &Measure, size: f64, max_width: f64, base: u8) -> T {
    let raw = match raw {
        None | Some("") => return T::empty(),
        Some(r) => r,
    };
    let line_height = js::round(size * LINE_HEIGHT);

    if !has_markup(raw) && !raw.contains('&') && !raw.contains('\u{fb02}') {
        let text = js::trim(raw);
        let width = m.width(text, size, base);
        if width <= max_width || !(text.contains(' ') || text.chars().any(is_cjk)) {
            return T::single(Span { text: text.to_string(), style: base, icon: None, width }, size, line_height);
        }
        let mut lines = Vec::new();
        let mut widths = Vec::new();
        wrap(vec![Span { text: text.to_string(), style: base, icon: None, width }], m, size, max_width, &mut lines, &mut widths);
        let mut widest = 0.0;
        for &w in &widths {
            if w > widest {
                widest = w;
            }
        }
        let height = lines.len() as f64 * line_height;
        return T::from_owned(Label { lines, widths, width: widest, height, size, line_height });
    }

    let source = if markdown {
        let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
        Cow::Owned(js::trim(&dedent(&normalized)).to_string())
    } else {
        Cow::Borrowed(js::trim(raw))
    };
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();
    let mut width = 0.0;
    for part in split_breaks(&source) {
        let mut line: Vec<Span> = Vec::new();
        inline(&mut line, js::trim(part), markdown, base, m);
        let w = measure_spans(&mut line, m, size);
        if w > max_width {
            wrap(line, m, size, max_width, &mut lines, &mut widths);
        } else {
            lines.push(line);
            widths.push(w);
        }
    }
    for &w in &widths {
        if w > width {
            width = w;
        }
    }
    let height = lines.len() as f64 * line_height;
    T::from_owned(Label { lines, widths, width, height, size, line_height })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Icons, MetricsMeasurer};

    fn label(raw: &str, markdown: bool, max: f64) -> Label {
        let icons = Icons::none();
        let m = Measure::new(&MetricsMeasurer, &icons);
        layout_label(Some(raw), markdown, &m, 16.0, max, 0)
    }

    fn texts(l: &Label) -> Vec<Vec<(String, u8)>> {
        l.lines.iter().map(|line| line.iter().map(|s| (s.text.clone(), s.style)).collect()).collect()
    }

    #[test]
    fn pieces_keep_every_character() {
        for text in ["hello world  foo", "中文（测试）。好", "a\u{3000}b", "\u{3000}", "(中", "（", "x, y.", "😀 ok"] {
            assert_eq!(pieces(text).concat(), text);
        }
        assert_eq!(pieces("中文。好"), vec!["中", "文。", "好"]);
        assert_eq!(pieces("（中）文"), vec!["（中）", "文"]);
        assert_eq!(pieces("ab cd"), vec!["ab ", "cd"]);
        assert_eq!(pieces("  ab"), vec!["  ab"]);
        assert_eq!(pieces("ab  "), vec!["ab  "]);
    }

    #[test]
    fn breaks_and_tags() {
        let l = label("a<br/>b<BR >c\\nd\\Ne\nf", false, 4000.0);
        assert_eq!(l.lines.len(), 6);
        let l = label("<b>bold</b> and <i>it</i>", false, 4000.0);
        assert_eq!(texts(&l), vec![vec![("bold".into(), style::BOLD), (" and ".into(), 0), ("it".into(), style::ITALIC)]]);
    }

    #[test]
    fn markdown_emphasis() {
        let l = label("**bold** _it_ snake_case", true, 4000.0);
        assert_eq!(
            texts(&l),
            vec![vec![("bold".into(), style::BOLD), (" ".into(), 0), ("it".into(), style::ITALIC), (" snake_case".into(), 0)]]
        );
    }

    #[test]
    fn wraps_chinese_between_characters() {
        let l = label("这是一个很长的中文标签用于测试换行", false, 100.0);
        assert!(l.lines.len() > 1);
        for w in &l.widths {
            assert!(*w <= 100.0);
        }
        let joined: String = l.lines.iter().flat_map(|line| line.iter().map(|s| s.text.clone())).collect();
        assert_eq!(joined, "这是一个很长的中文标签用于测试换行");
    }

    #[test]
    fn wraps_words_and_trims() {
        let l = label("alpha beta gamma delta", false, 60.0);
        assert!(l.lines.len() >= 2);
        for line in &l.lines {
            assert!(!line.last().unwrap().text.ends_with(' '));
        }
        assert_eq!(l.height, l.lines.len() as f64 * 24.0);
    }

    #[test]
    fn empty_and_blank() {
        assert_eq!(label("", false, 10.0), Label::empty());
        let l = label("   ", false, 10.0);
        assert_eq!(l.lines.len(), 1);
    }

    #[test]
    fn tag_matching() {
        assert_eq!(match_tag("<b>", 0), Some((3, "b")));
        assert_eq!(match_tag("</strong>", 0), Some((9, "strong")));
        assert_eq!(match_tag("<br />", 0), Some((6, "br")));
        assert_eq!(match_tag("<b/>", 0), Some((4, "b")));
        assert_eq!(match_tag("<a href='x'>", 0), Some((12, "a")));
        assert_eq!(match_tag("<1>", 0), None);
        assert_eq!(match_tag("<b", 0), None);
        assert_eq!(match_tag("<b x<y>", 0), None);
    }

    #[test]
    fn icons_need_a_resolver() {
        let icons = Icons::new(Some(std::sync::Arc::new(|name: &str| (name == "fa:fa-car").then(|| "<path/>".to_string()))));
        let m = Measure::new(&MetricsMeasurer, &icons);
        let l = layout_label(Some("fa:fa-car drive fa:fa-none here"), false, &m, 16.0, 4000.0, 0);
        let line = &l.lines[0];
        assert_eq!(line[0].icon.as_deref(), Some("fa:fa-car"));
        assert_eq!(line[1].text, " drive here");
    }
}
