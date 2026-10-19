//! Preprocessing: line breaks, front matter, directives, comments and entity codes. Ported
//! from Pele `src/preprocess.ts`, with a source map kept for every step.

pub mod source_map;

#[cfg(test)]
#[path = "../../tests/upstream/preprocess.rs"]
mod upstream_tests;

use crate::config::{json, merge, yaml, Config, ConfigMap, ConfigValue};
use crate::error::Error;
use crate::text::entities::{CLOSE, OPEN};
use crate::util::chars::{char_at, is_space, is_word_byte};
use crate::util::{js, strs};
use source_map::{Builder, SourceMap, StepMap};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Preprocessed {
    /// The text with comment lines removed, which detection and most parsers read.
    pub text: String,
    pub title: Option<String>,
    pub config: Config,
    /// The text before its comment lines are removed.
    pub with_comments: String,
    /// The number of front matter lines removed before `with_comments`.
    pub line_offset: usize,
    /// Maps `text` back to the source.
    pub text_map: SourceMap,
    /// Maps `with_comments` back to the source.
    pub with_comments_map: SourceMap,
}

/// `/\r\n?/g` → `\n`.
fn normalize_breaks(source: &str) -> (Cow<'_, str>, StepMap) {
    if !source.contains('\r') {
        return (Cow::Borrowed(source), StepMap::identity(source.len()));
    }
    let b = source.as_bytes();
    let mut out = Builder::new(source);
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\r' {
            out.copy(last, i);
            out.write("\n", i);
            i += if b.get(i + 1) == Some(&b'\n') { 2 } else { 1 };
            last = i;
        } else {
            i += 1;
        }
    }
    out.copy(last, b.len());
    let (text, map) = out.finish();
    (Cow::Owned(text), map)
}

/// Rewrites `="..."` to `='...'` inside `<word ...>` tags, up to the last `>`. The text keeps
/// its length, so no map is needed.
fn quote_attributes(text: &str) -> String {
    let Some(last_close) = text.rfind('>') else { return text.to_string() };
    if !text.contains('<') || last_close == 0 {
        return text.to_string();
    }
    let head = &text[..=last_close];
    let hb = head.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut i = 0;
    // `/<(\w+)([^>]*)>/g`
    while i < hb.len() {
        if hb[i] == b'<' && i + 1 < hb.len() && is_word_byte(hb[i + 1]) {
            let mut j = i + 1;
            while j < hb.len() && is_word_byte(hb[j]) {
                j += 1;
            }
            let attrs_start = j;
            while j < hb.len() && hb[j] != b'>' {
                j += 1;
            }
            if j < hb.len() {
                out.push_str(&head[last..attrs_start]);
                out.push_str(&requote(&head[attrs_start..j]));
                out.push('>');
                last = j + 1;
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&head[last..]);
    out.push_str(&text[last_close + 1..]);
    out
}

/// `attrs.replace(/="([^"]*)"/g, "='$1'")`
fn requote(attrs: &str) -> String {
    let b = attrs.as_bytes();
    let mut out = String::with_capacity(attrs.len());
    let mut last = 0;
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'=' && b[i + 1] == b'"' {
            if let Some(k) = attrs[i + 2..].find('"') {
                let close = i + 2 + k;
                out.push_str(&attrs[last..i]);
                out.push_str("='");
                out.push_str(&attrs[i + 2..close]);
                out.push('\'');
                last = close + 1;
                i = close + 1;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&attrs[last..]);
    out
}

struct FrontMatter {
    indent: String,
    body: String,
    length: usize,
}

fn is_line_break(c: Option<char>) -> bool {
    matches!(c, Some('\n' | '\r'))
}

/// Finds a `---` fenced YAML block at the start of the text the way Mermaid's front matter
/// regular expression does.
fn match_front_matter(text: &str) -> Option<FrontMatter> {
    let mut cursor = 0;
    while let Some(c) = char_at(text, cursor) {
        if !is_space(c) || c == '\n' || c == '\r' {
            break;
        }
        cursor += c.len_utf8();
    }
    let indent = &text[..cursor];
    if !text[cursor..].starts_with("---") {
        return None;
    }
    let mut opening_ends = Vec::new();
    let mut i = cursor + 3;
    while let Some(c) = char_at(text, i) {
        if !is_space(c) {
            break;
        }
        if c == '\n' || c == '\r' {
            opening_ends.push(i);
        }
        i += c.len_utf8();
    }
    if opening_ends.is_empty() {
        return None;
    }
    let fence = format!("{}---", indent);
    let closing_end = |i: usize| -> Option<usize> {
        if !is_line_break(char_at(text, i)) || !text[i + 1..].starts_with(&fence) {
            return None;
        }
        let mut last = None;
        let mut j = i + 1 + fence.len();
        while let Some(c) = char_at(text, j) {
            if !is_space(c) {
                break;
            }
            if c == '\n' || c == '\r' {
                last = Some(j);
            }
            j += c.len_utf8();
        }
        last
    };
    let needle = format!("\n{}", fence);
    let last_index_of = |from: usize| -> Option<usize> {
        let end = (from + needle.len()).min(text.len());
        text[..end].rfind(&needle)
    };
    let mut last_fence = None;
    let mut at = text.rfind(&needle);
    while let Some(i) = at {
        if i <= opening_ends[0] {
            break;
        }
        if closing_end(i).is_some() {
            last_fence = Some(i);
            break;
        }
        at = if i == 0 { None } else { last_index_of(i - 1) };
    }
    let last_fence = last_fence?;
    let mut opening_end = opening_ends[0];
    for &candidate in &opening_ends {
        if candidate < last_fence {
            opening_end = candidate;
        }
    }
    let body_start = opening_end + 1;
    let first_at = if body_start > 0 { body_start - 1 } else { 0 };
    let mut at = text[first_at..].find(&needle).map(|k| first_at + k);
    while let Some(i) = at {
        if i > last_fence {
            break;
        }
        if i >= body_start {
            if let Some(end) = closing_end(i) {
                return Some(FrontMatter { indent: indent.to_string(), body: text[body_start..i].to_string(), length: end + 1 });
            }
        }
        at = text[i + 1..].find(&needle).map(|k| i + 1 + k);
    }
    None
}

/// One match of Mermaid's directive pattern:
/// `/%{2}{\s*(?:(\w+)\s*:|(\w+))\s*(?:(\w+)|((?:(?!}%{2}).|\r?\n)*))?\s*(?:}%{2})?/gi`.
struct Directive<'a> {
    end: usize,
    kind: &'a str,
    args: Option<&'a str>,
}

fn skip_spaces(text: &str, mut i: usize) -> usize {
    while let Some(c) = char_at(text, i) {
        if !is_space(c) {
            break;
        }
        i += c.len_utf8();
    }
    i
}

fn word_end(text: &str, i: usize) -> usize {
    let b = text.as_bytes();
    let mut j = i;
    while j < b.len() && is_word_byte(b[j]) {
        j += 1;
    }
    j
}

fn directive_at(text: &str, start: usize) -> Option<Directive<'_>> {
    if !text[start..].starts_with("%%{") {
        return None;
    }
    let b = text.as_bytes();
    let p = skip_spaces(text, start + 3);
    let w = word_end(text, p);
    if w == p {
        return None;
    }
    // `(\w+)\s*:` is tried first; a shorter word cannot be followed by a colon either.
    let after = skip_spaces(text, w);
    let mut q = if b.get(after) == Some(&b':') { after + 1 } else { w };
    let kind = &text[p..w];
    q = skip_spaces(text, q);
    let mut args = None;
    let w2 = word_end(text, q);
    if w2 > q {
        q = w2;
    } else {
        // `((?:(?!}%%).|\r?\n)*)`
        let a = q;
        loop {
            if text[q..].starts_with("}%%") {
                break;
            }
            match char_at(text, q) {
                None => break,
                Some('\n') => q += 1,
                Some('\r') if b.get(q + 1) == Some(&b'\n') => q += 2,
                Some('\r' | '\u{2028}' | '\u{2029}') => break,
                Some(c) => q += c.len_utf8(),
            }
        }
        args = Some(&text[a..q]);
    }
    q = skip_spaces(text, q);
    if text[q..].starts_with("}%%") {
        q += 3;
    }
    Some(Directive { end: q, kind, args })
}

fn for_each_directive<'a>(text: &'a str, mut f: impl FnMut(usize, &Directive<'a>)) {
    let mut from = 0;
    while let Some(k) = text[from..].find("%%{") {
        let at = from + k;
        match directive_at(text, at) {
            Some(d) => {
                f(at, &d);
                from = d.end.max(at + 1);
            }
            None => from = at + 1,
        }
    }
}

fn directives(text: Cow<'_, str>, config: &mut ConfigMap) -> Result<(String, StepMap), Error> {
    if !strs::contains(&text, "%%{") {
        let len = text.len();
        return Ok((text.into_owned(), StepMap::identity(len)));
    }
    let text = &*text;
    let normalized = js::trim(text).replace('\'', "\"");
    let mut failure = None;
    for_each_directive(&normalized, |_, d| {
        if d.kind == "wrap" {
            config.insert("wrap", ConfigValue::Bool(true));
        } else if (d.kind == "init" || d.kind == "initialize") && d.args.is_some_and(|a| !a.is_empty()) {
            match json::parse_json(js::trim(d.args.unwrap_or_default())) {
                Ok(ConfigValue::Object(args)) => merge(config, &args),
                Ok(_) => {}
                // Mermaid ignores an init directive whose body is not valid JSON.
                Err(json::JsonError::Invalid) => {}
                Err(json::JsonError::TooDeep) => {
                    failure.get_or_insert_with(|| Error::limit("Diagram is nested too deeply."));
                }
            }
        }
    });
    if let Some(e) = failure {
        return Err(e);
    }
    let mut out = Builder::new(text);
    let mut last = 0;
    for_each_directive(text, |at, d| {
        out.copy(last, at);
        last = d.end;
    });
    out.copy(last, text.len());
    Ok(out.finish())
}

/// `text.replace(/^\s*%%(?!{)[^\n]+\n?/gm, '').trimStart()`, as two mapped steps.
pub(crate) fn cleanup_comments(text: &str) -> (String, Vec<StepMap>) {
    let mut out = Builder::new(text);
    let b = text.as_bytes();
    let mut last = 0;
    if strs::contains(text, "%%") {
        let mut p = 0;
        let mut line_start = true;
        while p < b.len() {
            if line_start {
                let q = skip_spaces(text, p);
                if text[q..].starts_with("%%") && b.get(q + 2) != Some(&b'{') {
                    let body = q + 2;
                    let end = text[body..].find('\n').map_or(text.len(), |k| body + k);
                    if end > body {
                        let stop = if end < text.len() { end + 1 } else { end };
                        out.copy(last, p);
                        last = stop;
                        p = stop;
                        continue;
                    }
                }
            }
            let c = char_at(text, p).expect("boundary");
            line_start = matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}');
            p += c.len_utf8();
        }
    }
    out.copy(last, text.len());
    let (mut removed, first) = out.finish();
    let start = removed.len() - js::trim_start(&removed).len();
    // The trimmed text is the same buffer with its start cut off, mapped as a copied run.
    let second = StepMap::suffix(removed.len(), start);
    if start > 0 {
        removed.drain(..start);
    }
    (removed, vec![first, second])
}

/// Mermaid's `text.replace(/keyword.*:\S*#.*;/g, ...)`, done in one pass: on a line where the
/// keyword is followed by a colon, a color-like `#`, and later a semicolon, the last semicolon
/// goes. Mermaid's pattern backtracks without bound; this is one pass, as in Pele.
fn drop_color_semicolons(text: &str, keyword: &str) -> (String, StepMap) {
    let mut out = Builder::new(text);
    let mut last = 0;
    let mut from = text.find(keyword);
    while let Some(f) = from {
        // The first line break at or after `from`.
        let mut line_end = text.len();
        let mut break_len = 0;
        for (k, c) in text[f..].char_indices() {
            if matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                line_end = f + k;
                break_len = c.len_utf8();
                break;
            }
        }
        let mut after_colon = false;
        let mut hash = None;
        let scan_from = (f + keyword.len()).min(line_end);
        for (k, c) in text[scan_from..line_end].char_indices() {
            if c == ':' {
                after_colon = true;
            } else if c == '#' {
                if after_colon {
                    hash = Some(scan_from + k);
                    break;
                }
            } else if is_space(c) {
                after_colon = false;
            }
        }
        if let Some(h) = hash {
            if let Some(semi) = text[..line_end].rfind(';') {
                if semi > h {
                    out.copy(last, semi);
                    last = semi + 1;
                }
            }
        }
        from = if line_end >= text.len() { None } else { text[line_end + break_len..].find(keyword).map(|k| line_end + break_len + k) };
    }
    out.copy(last, text.len());
    out.finish()
}

/// `text.replace(/#\w+;/g, ...)`: each code goes behind Mermaid's placeholder characters.
fn hide_entities(text: &str) -> (String, StepMap) {
    let b = text.as_bytes();
    let mut out = Builder::new(text);
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'#' {
            let mut j = i + 1;
            while j < b.len() && is_word_byte(b[j]) {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == b';' {
                let inner = &text[i + 1..j];
                let numeric = inner.bytes().all(|c| c.is_ascii_digit());
                out.copy(last, i);
                out.write(&format!("{}{}{}{}", OPEN, if numeric { "\u{b0}" } else { "" }, inner, CLOSE), i);
                last = j + 1;
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out.copy(last, text.len());
    out.finish()
}

/// Mermaid hides `#name;` entity codes behind placeholder characters so `#` and `;` survive
/// parsing. Returns the text and the steps that map it back to its input.
pub fn encode_entities(text: &str) -> (String, SourceMap) {
    let mut map = SourceMap::default();
    if !text.contains('#') {
        map.push(StepMap::identity(text.len()));
        // Room for the line break a parser's input ends with, so it is not grown again.
        let mut copy = String::with_capacity(text.len() + 1);
        copy.push_str(text);
        return (copy, map);
    }
    let (a, m1) = drop_color_semicolons(text, "style");
    let (b, m2) = drop_color_semicolons(&a, "classDef");
    let (c, m3) = hide_entities(&b);
    map.push(m1);
    map.push(m2);
    map.push(m3);
    (c, map)
}

fn is_record(v: &ConfigValue) -> bool {
    matches!(v, ConfigValue::Object(_))
}

/// Preprocesses diagram text the way Mermaid does before it detects and parses a diagram.
pub fn preprocess(source: &str) -> Result<Preprocessed, Error> {
    let mut map = SourceMap::default();
    let (text, step) = normalize_breaks(source);
    map.push(step);
    let text = if text.contains('<') { Cow::Owned(quote_attributes(&text)) } else { text };

    let mut config = ConfigMap::new();
    let mut title = None;
    let mut line_offset = 0;

    let mut text = text;
    let front = if strs::contains(&text, "---") { match_front_matter(&text) } else { None };
    if let Some(front) = front {
        let body = if front.indent.is_empty() {
            front.body.clone()
        } else {
            front.body.split('\n').map(|line| line.strip_prefix(front.indent.as_str()).unwrap_or(line)).collect::<Vec<_>>().join("\n")
        };
        let parsed = yaml::parse_yaml(&body).map_err(|e| Error::syntax(format!("Front matter is not valid YAML. {}", e.message)))?;
        if let ConfigValue::Object(fm) = &parsed {
            if let Some(t) = fm.get("title").filter(|t| t.truthy()) {
                title = Some(t.to_js_string());
            }
            if let Some(ConfigValue::Object(c)) = fm.get("config") {
                merge(&mut config, c);
            }
            if let Some(mode) = fm.get("displayMode").filter(|m| m.truthy()) {
                let mode = mode.to_js_string();
                if !config.get("gantt").is_some_and(is_record) {
                    config.insert("gantt", ConfigValue::Object(ConfigMap::new()));
                }
                if let Some(ConfigValue::Object(g)) = config.get_mut("gantt") {
                    g.insert("displayMode", ConfigValue::String(mode));
                }
            }
        }
        line_offset = text[..front.length.min(text.len())].matches('\n').count();
        let mut b = Builder::new(&text);
        b.copy(front.length, text.len());
        let (rest, step) = b.finish();
        map.push(step);
        text = Cow::Owned(rest);
    }

    let (with_comments, step) = directives(text, &mut config)?;
    map.push(step);
    let with_comments_map = map.clone();
    let (cleaned, steps) = cleanup_comments(&with_comments);
    for step in steps {
        map.push(step);
    }
    Ok(Preprocessed { text: cleaned, title, config, with_comments, line_offset, text_map: map, with_comments_map })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_and_title() {
        let p = preprocess("---\ntitle: Hello\nconfig:\n  pie:\n    textPosition: 0.5\n---\npie\n  \"a\": 1\n").unwrap();
        assert_eq!(p.title.as_deref(), Some("Hello"));
        assert_eq!(p.text, "pie\n  \"a\": 1\n");
        assert_eq!(p.line_offset, 6);
        assert_eq!(p.config.lookup("pie", "textPosition").unwrap().as_f64(), Some(0.5));
        let at = p.text.find("\"a\"").unwrap();
        let src = "---\ntitle: Hello\nconfig:\n  pie:\n    textPosition: 0.5\n---\npie\n  \"a\": 1\n";
        assert_eq!(p.text_map.to_source(at), src.find("\"a\"").unwrap());
    }

    #[test]
    fn front_matter_needs_a_closing_break() {
        let p = preprocess("---\ntitle: x\n---").unwrap();
        assert_eq!(p.title, None);
    }

    #[test]
    fn directives_merge_json_and_disappear() {
        let p = preprocess("%%{init: {'pie': {'textPosition': 0.3}}}%%\npie\n").unwrap();
        assert_eq!(p.config.lookup("pie", "textPosition").unwrap().as_f64(), Some(0.3));
        assert_eq!(p.text, "pie\n");
        let p = preprocess("%%{wrap}%%\ngraph TD").unwrap();
        assert_eq!(p.config.get("wrap"), Some(&ConfigValue::Bool(true)));
        let p = preprocess("%%{init: {bad json}}%%\npie").unwrap();
        assert!(p.config.is_empty());
    }

    #[test]
    fn comments_and_crlf() {
        let src = "pie\r\n%% a comment\r\n  \"a\": 1\r\n%%\r\n";
        let p = preprocess(src).unwrap();
        // An empty `%%` line is not a comment to Mermaid's pattern.
        assert_eq!(p.text, "pie\n  \"a\": 1\n%%\n");
        assert_eq!(p.with_comments, "pie\n%% a comment\n  \"a\": 1\n%%\n");
        let at = p.text.find("\"a\"").unwrap();
        assert_eq!(p.text_map.to_source(at), src.find("\"a\"").unwrap());
    }

    #[test]
    fn leading_blank_lines_go_with_a_comment() {
        let p = preprocess("\n\n  %% c\npie").unwrap();
        assert_eq!(p.text, "pie");
    }

    #[test]
    fn html_attributes_take_single_quotes() {
        let p = preprocess("graph TD\nA[\"<span class=\"x\">a</span>\"]").unwrap();
        assert_eq!(p.text, "graph TD\nA[\"<span class='x'>a</span>\"]");
    }

    #[test]
    fn entities_are_hidden() {
        let (out, map) = encode_entities("A[#quot;x#59;]");
        assert_eq!(out, "A[\u{fb02}\u{b0}quot\u{b6}\u{df}x\u{fb02}\u{b0}\u{b0}59\u{b6}\u{df}]");
        assert_eq!(map.to_source(out.find('x').unwrap()), 8);
        assert_eq!(map.to_source(out.find('\u{b6}').unwrap()), 2);
        let (out, _) = encode_entities("style A fill:#f9f;");
        assert_eq!(out, "style A fill:#f9f");
        let (out, _) = encode_entities("classDef c fill:#f9f,stroke:#333;");
        assert_eq!(out, "classDef c fill:#f9f,stroke:#333");
    }
}
