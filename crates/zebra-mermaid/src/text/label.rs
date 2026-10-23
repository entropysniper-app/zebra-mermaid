//! Label layout: line breaks, inline emphasis, icons and wrapping. Ported from Pele
//! `src/text/label.ts`.
//!
//! The regular expressions of the original are written out as scanners here. Each keeps the
//! JavaScript semantics it replaces: `\s` is JavaScript white space, `\w` is ASCII, and the
//! `i` flag makes the letters of `<br>` and `\n` match in either case.

use super::entities::decode_entities_cow;
use super::measurer::style;
use super::MathMarkup;
use super::Measure;
use crate::util::chars::{char_at, char_before, is_space, is_word};
use crate::util::{js, strs};
use std::borrow::Cow;
use std::rc::Rc;

#[cfg(test)]
#[path = "label_scanning_reference.rs"]
mod scanning_reference;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: u8,
    pub icon: Option<String>,
    pub math: Option<Rc<MathMarkup>>,
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
    pub heights: Option<Vec<f64>>,
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
fn split_breaks(source: &str) -> impl Iterator<Item = &str> {
    let b = source.as_bytes();
    let mut last = 0;
    let mut i = 0;
    let mut finished = false;
    std::iter::from_fn(move || {
        if finished {
            return None;
        }
        while i < b.len() {
            let end = match b[i] {
                b'\n' => Some(i + 1),
                b'\\' if i + 1 < b.len() && (b[i + 1] == b'n' || b[i + 1] == b'N') => Some(i + 2),
                b'<' => br_end(source, i),
                _ => None,
            };
            match end {
                Some(e) => {
                    let part = &source[last..i];
                    last = e;
                    i = e;
                    return Some(part);
                }
                None => i += 1,
            }
        }
        finished = true;
        Some(&source[last..])
    })
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

fn push_plain(line: &mut Vec<Span>, text: &str, style: u8) {
    if text.is_empty() {
        return;
    }
    let decoded = decode_entities_cow(text);
    if let Some(last) = line.last_mut() {
        if last.style == style && plain(last) {
            last.text.push_str(&decoded);
            return;
        }
    }
    line.push(Span { text: decoded.into_owned(), style, icon: None, math: None, width: 0.0 });
}

fn plain(span: &Span) -> bool {
    span.icon.is_none() && span.math.is_none()
}

struct Marks {
    tex: Vec<String>,
    math: Vec<Option<Rc<MathMarkup>>>,
}

fn push_text(line: &mut Vec<Span>, text: &str, style: u8, marks: Option<&Marks>) {
    let Some(marks) = marks else {
        push_plain(line, text, style);
        return;
    };
    let mut start = 0;
    for (at, ch) in text.char_indices() {
        if ('\u{e000}'..='\u{f8ff}').contains(&ch) {
            if at > start {
                push_plain(line, &text[start..at], style);
            }
            let i = ch as usize - 0xe000;
            if let Some(math) = &marks.math[i] {
                line.push(Span { text: String::new(), style, icon: None, math: Some(math.clone()), width: math.width });
            } else {
                push_plain(line, &format!("$${}$$", marks.tex[i]), style);
            }
            start = at + ch.len_utf8();
        }
    }
    if start < text.len() {
        push_plain(line, &text[start..], style);
    }
}

fn push_icons(line: &mut Vec<Span>, text: &str, style: u8, m: &Measure, marks: Option<&Marks>) {
    if !strs::contains(text, ":fa-") {
        push_text(line, text, style, marks);
        return;
    }
    let mut last = 0;
    let mut from = 0;
    while let Some((start, end)) = find_icon(text, from) {
        let icon = &text[start..end];
        push_text(line, &text[last..start], style, marks);
        last = end;
        from = end;
        if !m.icon_markup(icon).is_empty() {
            line.push(Span { text: String::new(), style, icon: Some(icon.to_string()), math: None, width: 0.0 });
        } else if text.as_bytes().get(last) == Some(&b' ') {
            last += 1;
        } else if last == text.len() {
            if let Some(before) = line.last_mut() {
                if plain(before) {
                    before.text.truncate(js::trim_end(&before.text).len());
                }
            }
        }
    }
    push_text(line, &text[last..], style, marks);
}

#[derive(Default)]
struct Open {
    style: u8,
    shifts: Vec<u8>,
}

fn tag_style(tag: &str) -> u8 {
    match tag {
        "b" | "strong" => style::BOLD,
        "i" | "em" | "cite" | "dfn" | "var" => style::ITALIC,
        "code" | "kbd" | "samp" | "tt" => style::MONO,
        "sub" => style::SUB,
        "sup" => style::SUP,
        "u" | "ins" => style::UNDERLINE,
        _ => 0,
    }
}

const HTML_TAGS: &str = "a abbr address article aside audio b bdi bdo big blockquote body br button caption center cite code col colgroup data dd del details dfn dialog div dl dt em embed fieldset figcaption figure font footer form h1 h2 h3 h4 h5 h6 head header hr html i iframe img input ins kbd label legend li link main map mark meta meter nav noscript object ol optgroup option output p picture pre progress q rp rt ruby s samp script section select small source span strike strong style sub summary sup table tbody td template textarea tfoot th thead time title tr track tt u ul var video wbr";

/// Splits inline HTML and, for markdown labels, emphasis markers into styled spans.
fn inline(line: &mut Vec<Span>, source: &str, markdown: bool, base: u8, m: &Measure, open: &mut Open, marks: Option<&Marks>) {
    let mut st = open.style;
    let mut marked: u8 = 0;
    let mut text = String::new();
    let mut start = 0;
    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0;
    // Continuous text is borrowed. Only a tag removed without a style boundary needs
    // disjoint pieces joined before icon/entity recognition, preserving the original flushes.
    let flush = |line: &mut Vec<Span>, text: &mut String, start: usize, end: usize, style: u8| {
        if text.is_empty() {
            push_icons(line, &source[start..end], style, m, marks);
        } else {
            text.push_str(&source[start..end]);
            push_icons(line, text, style, m, marks);
            text.clear();
        }
    };
    while i < n {
        let next = if markdown { memchr::memchr3(b'<', b'*', b'_', &b[i..]) } else { memchr::memchr(b'<', &b[i..]) };
        let Some(next) = next else { break };
        i += next;
        let ch = b[i];
        if ch == b'<' {
            if let Some((end, name)) = match_tag(source, i) {
                let closing = b.get(i + 1) == Some(&b'/');
                let tag = name.to_ascii_lowercase();
                let bit = tag_style(&tag);
                let effective = st | open.shifts.last().copied().unwrap_or(0);
                if bit & (style::SUB | style::SUP) != 0 {
                    flush(line, &mut text, start, i, effective);
                    if !closing {
                        open.shifts.push(bit);
                    } else if let Some(at) = open.shifts.iter().rposition(|&b| b == bit) {
                        open.shifts.remove(at);
                    }
                } else if bit != 0 {
                    flush(line, &mut text, start, i, effective);
                    open.style = if closing { (open.style & !bit) | (base & bit) } else { open.style | bit };
                    st = if closing { (st & !bit) | (base & bit) } else { st | bit };
                }
                if bit != 0 || HTML_TAGS.split(' ').any(|t| t == tag) {
                    if bit == 0 {
                        text.push_str(&source[start..i]);
                    }
                    start = end;
                    i = end;
                    continue;
                }
            }
        } else if markdown && (ch == b'*' || ch == b'_') {
            let double = b.get(i + 1) == Some(&ch);
            let bit = if double { style::BOLD } else { style::ITALIC };
            let len = if double { 2 } else { 1 };
            let md_open = marked & bit == 0;
            let prev = if i > 0 { char_before(source, i).unwrap_or(' ') } else { ' ' };
            let next = if i + len < n { char_at(source, i + len).unwrap_or(' ') } else { ' ' };
            let marker = if double {
                if ch == b'*' {
                    "**"
                } else {
                    "__"
                }
            } else if ch == b'*' {
                "*"
            } else {
                "_"
            };
            let can_open = md_open && !is_space(next) && source[i + len..].contains(marker);
            let can_close = !md_open && !is_space(prev);
            let wordy = ch == b'_' && is_word(prev) && is_word(next);
            if (can_open || can_close) && !wordy {
                flush(line, &mut text, start, i, st | open.shifts.last().copied().unwrap_or(0));
                marked = if can_open { marked | bit } else { marked & !bit };
                st = if can_open { st | bit } else { (st & !bit) | (base & bit) };
                i += len;
                start = i;
                continue;
            }
        }
        i += 1;
    }
    flush(line, &mut text, start, n, st | open.shifts.last().copied().unwrap_or(0));
}

fn measure_spans(line: &mut [Span], m: &Measure, size: f64) -> f64 {
    let mut total = 0.0;
    for span in line.iter_mut() {
        span.width = if span.icon.is_some() {
            size * 1.25
        } else if let Some(math) = &span.math {
            math.width
        } else {
            m.width(&span.text, size, span.style)
        };
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
    // [^\sCJK HINT]+[HINT]*[CLOSE]* | [HINT]+
    let e = skip_while(text, s, |c| !is_space(c) && !is_cjk(c) && !is_hint(c));
    if e > s {
        return Some(skip_while(text, skip_while(text, e, is_hint), is_close));
    }
    let e = skip_while(text, s, is_hint);
    if e > s {
        return Some(e);
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

fn is_hint(c: char) -> bool {
    c == '\u{200b}' || c == '\u{ad}'
}

fn wrap(line: Vec<Span>, m: &Measure, size: f64, max: f64, out: &mut Vec<Vec<Span>>, widths: &mut Vec<f64>) {
    let mut current: Vec<Span> = Vec::new();
    let mut width = 0.0;
    let commit = |current: &mut Vec<Span>, width: &mut f64, out: &mut Vec<Vec<Span>>, widths: &mut Vec<f64>, hyphen: bool| {
        if let Some(last) = current.last_mut().filter(|s| plain(s)) {
            let trimmed = js::trim_end(&last.text);
            let trimmed = if hyphen && last.text.ends_with('\u{ad}') {
                Cow::Owned(format!("{}-", trimmed.strip_suffix('\u{ad}').unwrap_or(trimmed)))
            } else {
                Cow::Borrowed(trimmed)
            };
            if trimmed != last.text {
                let w = m.width(&trimmed, size, last.style);
                *width += w - last.width;
                last.text = trimmed.into_owned();
                last.width = w;
            }
        }
        out.push(std::mem::take(current));
        widths.push(*width);
        *width = 0.0;
    };
    let last_span = line.len().saturating_sub(1);
    for (index, span) in line.into_iter().enumerate() {
        if !plain(&span) {
            let w = span.math.as_ref().map_or(size * 1.25, |m| m.width);
            if width > 0.0 && width + w > max {
                commit(&mut current, &mut width, out, widths, false);
            }
            current.push(Span { width: w, ..span });
            width += w;
            continue;
        }
        let pieces: Vec<&str> = piece_iter(&span.text).collect();
        let soft = span.text.contains('\u{ad}');
        let n = if soft { pieces.len() } else { 0 };
        let mut own = vec![0.0; n];
        let mut rest = vec![0.0; n + 1];
        for q in (0..n).rev() {
            let shy = pieces[q].ends_with('\u{ad}');
            let end = js::trim_end(pieces[q]);
            own[q] = m.width(if shy { end.strip_suffix('\u{ad}').unwrap_or(end) } else { end }, size, span.style);
            rest[q] = if !shy {
                own[q]
            } else if q == n - 1 {
                if index == last_span {
                    own[q]
                } else {
                    f64::INFINITY
                }
            } else {
                own[q] + rest[q + 1]
            };
        }
        for (p, piece) in pieces.into_iter().enumerate() {
            let mut part = piece;
            let part_width = if soft { own[p] } else { m.width(js::trim_end(part), size, span.style) };
            let fits = if !soft || !part.ends_with('\u{ad}') {
                width + part_width <= max
            } else {
                width + m.width(&format!("{}-", part.strip_suffix('\u{ad}').unwrap()), size, span.style) <= max || width + rest[p] <= max
            };
            if width > 0.0 && !fits {
                commit(&mut current, &mut width, out, widths, js::trim_start(part) == part);
                part = js::trim_start(part);
            }
            match current.last_mut() {
                Some(last) if last.style == span.style && plain(last) => {
                    last.text.push_str(part);
                    if part_width > 0.0 || js::trim_end(part) != part {
                        let before = last.width;
                        last.width = m.width(&last.text, size, last.style);
                        width += last.width - before;
                    }
                }
                _ => {
                    let w = m.width(part, size, span.style);
                    current.push(Span { text: part.to_string(), style: span.style, icon: None, math: None, width: w });
                    width += w;
                }
            }
        }
    }
    if !current.is_empty() || out.is_empty() {
        commit(&mut current, &mut width, out, widths, false);
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
        Self { lines: vec![vec![span]], widths: vec![width], width, height: line_height, size, line_height, heights: None }
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
    let marked = if m.math_active() && raw.contains("$$") && !raw.chars().any(|c| ('\u{e000}'..='\u{f8ff}').contains(&c)) {
        mark_math(raw, m, size)
    } else {
        None
    };
    let raw = marked.as_ref().map_or(raw, |(raw, _)| raw.as_str());
    let marks = marked.as_ref().map(|(_, marks)| marks);

    if marks.is_none() && !has_markup(raw) && !raw.contains('&') && !raw.contains('\u{fb02}') {
        let text = js::trim(raw);
        let width = m.width(text, size, base);
        if width <= max_width || !(text.contains(' ') || text.chars().any(|c| is_cjk(c) || is_hint(c))) {
            return T::single(Span { text: text.to_string(), style: base, icon: None, math: None, width }, size, line_height);
        }
        let mut lines = Vec::new();
        let mut widths = Vec::new();
        wrap(
            vec![Span { text: text.to_string(), style: base, icon: None, math: None, width }],
            m,
            size,
            max_width,
            &mut lines,
            &mut widths,
        );
        let mut widest = 0.0;
        for &w in &widths {
            if w > widest {
                widest = w;
            }
        }
        let snug = if lines.len() > 1 { js::round(size * 1.3) } else { line_height };
        let height = lines.len() as f64 * snug;
        return T::from_owned(Label { lines, widths, width: widest, height, size, line_height: snug, heights: None });
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
    let mut open = Open { style: base, shifts: Vec::new() };
    for part in split_breaks(&source) {
        let mut line: Vec<Span> = Vec::new();
        inline(&mut line, js::trim(part), markdown, base, m, &mut open, marks);
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
    let snug = if lines.len() > 1 { js::round(size * 1.3) } else { line_height };
    let height = lines.len() as f64 * snug;
    let mut label = Label { lines, widths, width, height, size, line_height: snug, heights: None };
    if marks.is_some() {
        with_heights(&mut label);
    }
    T::from_owned(label)
}

fn mark_math(raw: &str, m: &Measure, size: f64) -> Option<(String, Marks)> {
    let mut out = String::with_capacity(raw.len());
    let mut tex = Vec::new();
    let mut cursor = 0;
    let mut last = 0;
    while let Some(at) = raw[cursor..].find("$$") {
        let start = cursor + at;
        let inside = start + 2;
        let Some(first) = raw.get(inside..).and_then(|s| s.chars().next()) else {
            break;
        };
        let search = inside + first.len_utf8();
        let Some(end) = raw[search..].find("$$").map(|p| search + p) else {
            break;
        };
        if raw[inside..end].chars().any(|c| matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}')) {
            cursor = start + 1;
            continue;
        }
        if tex.len() == 0x1900 {
            break;
        }
        out.push_str(&raw[last..start]);
        out.push(char::from_u32(0xe000 + tex.len() as u32).unwrap());
        tex.push(raw[inside..end].to_owned());
        cursor = end + 2;
        last = cursor;
    }
    if tex.is_empty() {
        return None;
    }
    out.push_str(&raw[last..]);
    let math = tex.iter().map(|tex| m.typeset(&decode_entities_cow(tex), size)).collect();
    Some((out, Marks { tex, math }))
}

pub fn has_math(line: &[Span]) -> bool {
    line.iter().any(|s| s.math.is_some())
}

pub fn with_heights(label: &mut Label) {
    let mut heights: Option<Vec<f64>> = None;
    for (i, line) in label.lines.iter().enumerate() {
        let mut h = label.line_height;
        for span in line {
            if let Some(math) = &span.math {
                h = js::max(h, (math.height + label.size / 4.0).ceil());
            }
        }
        if h != label.line_height && heights.is_none() {
            heights = Some(vec![label.line_height; label.lines.len()]);
        }
        if let Some(heights) = &mut heights {
            heights[i] = h;
        }
    }
    label.height = heights.as_ref().map_or(label.lines.len() as f64 * label.line_height, |heights| heights.iter().sum());
    label.heights = heights;
}

pub fn line_middle(label: &Label, i: usize) -> f64 {
    label.heights.as_ref().map_or((i as f64 + 0.5) * label.line_height, |h| h[..i].iter().sum::<f64>() + h[i] / 2.0)
}

pub fn label_line(label: &Label, i: usize) -> Label {
    let width = label.widths[i];
    let height = label.heights.as_ref().map_or(label.line_height, |h| h[i]);
    Label {
        lines: vec![label.lines[i].clone()],
        widths: vec![width],
        width,
        height,
        size: label.size,
        line_height: label.line_height,
        heights: (height != label.line_height).then(|| vec![height]),
    }
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
        // Pele 1.9.0 uses round(16 * 1.3) = 21 for multiline labels.
        assert_eq!(l.height, l.lines.len() as f64 * 21.0);
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
