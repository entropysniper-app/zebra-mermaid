//! Link and image addresses. Ported from Pele `src/util/url.ts`.
//!
//! `sanitize_url` follows the algorithm of @braintree/sanitize-url, which Mermaid applies to
//! click targets. The scheme allow-lists then keep only relative addresses and the schemes the
//! host allows. The `url` crate is used only to normalize http(s) addresses as the WHATWG URL
//! parser does; the security policy is the code here.

use crate::options::RenderOptions;
use crate::svg::builder::esc;
use crate::text::entities::decode_entities;
use crate::util::chars::is_space;
use crate::util::js;

const BLANK: &str = "about:blank";

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LinkInfo {
    pub id: String,
    pub href: String,
    pub internal: bool,
}

fn hex(b: u8) -> Option<u8> {
    (b as char).to_digit(16).map(|d| d as u8)
}

/// `decodeURIComponent`, or `None` where it throws a URIError.
pub fn decode_uri_component(s: &str) -> Option<String> {
    if !s.contains('%') {
        return Some(s.to_string());
    }
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    let byte_at = |i: usize| -> Option<u8> {
        if b.get(i) != Some(&b'%') {
            return None;
        }
        Some(hex(*b.get(i + 1)?)? * 16 + hex(*b.get(i + 2)?)?)
    };
    while i < b.len() {
        if b[i] != b'%' {
            out.push(b[i]);
            i += 1;
            continue;
        }
        let first = byte_at(i)?;
        i += 3;
        if first < 0x80 {
            out.push(first);
            continue;
        }
        let n = if first & 0xe0 == 0xc0 {
            2
        } else if first & 0xf0 == 0xe0 {
            3
        } else if first & 0xf8 == 0xf0 {
            4
        } else {
            return None;
        };
        let mut seq = vec![first];
        for _ in 1..n {
            let c = byte_at(i)?;
            if c & 0xc0 != 0x80 {
                return None;
            }
            seq.push(c);
            i += 3;
        }
        // Overlong forms, surrogates and values past U+10FFFF are errors, as from_utf8 finds.
        let text = std::str::from_utf8(&seq).ok()?;
        out.extend_from_slice(text.as_bytes());
    }
    String::from_utf8(out).ok()
}

fn decode(uri: &str) -> String {
    decode_uri_component(uri).unwrap_or_else(|| uri.to_string())
}

/// `[\u0000-\u001F\u007F-\u009F -‍﻿]`
fn is_ctrl(c: char) -> bool {
    let c = c as u32;
    c <= 0x1f || (0x7f..=0x9f).contains(&c) || (0x2000..=0x200d).contains(&c) || c == 0xfeff
}

fn strip_ctrl(s: &str) -> String {
    s.chars().filter(|&c| !is_ctrl(c)).collect()
}

/// `/&#(\w+)(^\w|;)?/g` replaced with `String.fromCharCode(\1)`. Works on UTF-16 units, so that
/// two codes that make a surrogate pair become one character, as in JavaScript.
fn replace_html_entities(s: &str) -> String {
    if !s.contains("&#") {
        return s.to_string();
    }
    let units: Vec<u16> = s.encode_utf16().collect();
    let mut out: Vec<u16> = Vec::with_capacity(units.len());
    let is_w = |u: u16| u < 128 && crate::util::chars::is_word(u as u8 as char);
    let mut i = 0;
    while i < units.len() {
        if units[i] == b'&' as u16 && units.get(i + 1) == Some(&(b'#' as u16)) && units.get(i + 2).is_some_and(|&u| is_w(u)) {
            let start = i + 2;
            let mut j = start;
            while j < units.len() && is_w(units[j]) {
                j += 1;
            }
            let word: String = units[start..j].iter().map(|&u| u as u8 as char).collect();
            let n = js::number(&word);
            let code = if n.is_finite() { (n.trunc().rem_euclid(65536.0)) as u16 } else { 0 };
            out.push(code);
            i = if units.get(j) == Some(&(b';' as u16)) { j + 1 } else { j };
            continue;
        }
        out.push(units[i]);
        i += 1;
    }
    String::from_utf16_lossy(&out)
}

fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() {
        return None;
    }
    (0..=h.len() - n.len()).find(|&i| h[i..i + n.len()].eq_ignore_ascii_case(n))
}

/// `/&(newline|tab);/gi` replaced with nothing.
fn strip_ctrl_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let a = find_ci(rest, "&newline;").map(|i| (i, 9));
        let b = find_ci(rest, "&tab;").map(|i| (i, 5));
        let next = match (a, b) {
            (Some(x), Some(y)) => Some(if x.0 <= y.0 { x } else { y }),
            (x, y) => x.or(y),
        };
        match next {
            Some((i, len)) => {
                out.push_str(&rest[..i]);
                rest = &rest[i + len..];
            }
            None => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

/// The end of `/(\\|%5[cC])((%(6[eE]|72|74))|[nrt])/` at `i`.
fn ws_escape_at(s: &[u8], i: usize) -> Option<usize> {
    let after = if s.get(i) == Some(&b'\\') {
        i + 1
    } else if s.get(i) == Some(&b'%') && s.get(i + 1) == Some(&b'5') && matches!(s.get(i + 2), Some(b'c' | b'C')) {
        i + 3
    } else {
        return None;
    };
    if s.get(after) == Some(&b'%') {
        let pair = (s.get(after + 1).copied(), s.get(after + 2).copied());
        if matches!(pair, (Some(b'6'), Some(b'e' | b'E')) | (Some(b'7'), Some(b'2' | b'4'))) {
            return Some(after + 3);
        }
    }
    if matches!(s.get(after), Some(b'n' | b'r' | b't')) {
        return Some(after + 1);
    }
    None
}

fn strip_ws_escapes(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if let Some(e) = ws_escape_at(b, i) {
            out.push_str(&s[last..i]);
            last = e;
            i = e;
        } else {
            i += 1;
        }
    }
    out.push_str(&s[last..]);
    out
}

fn has_html_entity(s: &str) -> bool {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(2)).any(|i| b[i] == b'&' && b[i + 1] == b'#' && crate::util::chars::is_word_byte(b[i + 2]))
}

/// `/^.+(:|&colon;)/gim`, first match: from a line start to the last colon on that line.
fn scheme_match(s: &str) -> Option<&str> {
    let mut line_start = 0;
    loop {
        let rest = &s[line_start..];
        let line_len = rest.find(['\n', '\r', '\u{2028}', '\u{2029}']).unwrap_or(rest.len());
        let line = &rest[..line_len];
        // `.+` needs at least one character before the colon.
        let colon = line.rfind(':').filter(|&k| k >= 1).map(|k| k + 1);
        let named = line.rfind("&colon;").filter(|&k| k >= 1).map(|k| k + 7);
        if let Some(end) = colon.max(named) {
            return Some(&line[..end]);
        }
        if line_len == rest.len() {
            return None;
        }
        let term = rest[line_len..].chars().next().expect("terminator");
        line_start += line_len + term.len_utf8();
    }
}

/// `/^([^\w]*)(javascript|data|vbscript)/im` against one line.
fn is_invalid_protocol(protocol: &str) -> bool {
    let lower = protocol.to_ascii_lowercase();
    for line in lower.split(['\n', '\r', '\u{2028}', '\u{2029}']) {
        let t = line.trim_start_matches(|c: char| !crate::util::chars::is_word(c));
        if t.starts_with("javascript") || t.starts_with("data") || t.starts_with("vbscript") {
            return true;
        }
    }
    false
}

/// Follows the algorithm of @braintree/sanitize-url, which Mermaid applies to click targets.
pub fn sanitize_url(url: &str) -> String {
    if url.is_empty() {
        return BLANK.into();
    }
    let mut decoded = decode(js::trim(url));
    let mut rounds = 0;
    loop {
        // Each round peels one layer of encoding. A URL wrapped this many times is not a real one.
        rounds += 1;
        if rounds > 32 {
            return BLANK.into();
        }
        let step = strip_ctrl(&decoded);
        let step = replace_html_entities(&step);
        let step = strip_ctrl_entities(&step);
        let step = strip_ctrl(&step);
        let step = strip_ws_escapes(&step);
        decoded = decode(js::trim(&step));
        let more = decoded.chars().any(is_ctrl)
            || has_html_entity(&decoded)
            || find_ci(&decoded, "&newline;").is_some()
            || find_ci(&decoded, "&tab;").is_some()
            || (0..decoded.len()).any(|i| ws_escape_at(decoded.as_bytes(), i).is_some());
        if !more {
            break;
        }
    }
    if decoded.is_empty() {
        return BLANK.into();
    }
    if decoded.starts_with('.') || decoded.starts_with('/') {
        return decoded;
    }
    let trimmed = decoded.trim_start_matches(is_space);
    let Some(scheme) = scheme_match(trimmed) else { return decoded };
    let protocol = js::trim(&scheme.to_lowercase()).to_string();
    if is_invalid_protocol(&protocol) {
        return BLANK.into();
    }
    let normalized = trimmed.replace('\\', "/");
    if protocol == "mailto:" || protocol.contains("://") {
        return normalized;
    }
    if protocol == "http:" || protocol == "https:" {
        return match url::Url::parse(&normalized) {
            Ok(parsed) => parsed.to_string(),
            Err(_) => BLANK.into(),
        };
    }
    normalized
}

/// A URL as written in a diagram: entity codes are read first, so they cannot hide a scheme.
pub fn safe_url(url: &str) -> String {
    sanitize_url(&decode_entities(url))
}

const LINK_SCHEMES: [&str; 4] = ["http", "https", "mailto", "tel"];
const IMAGE_SCHEMES: [&str; 2] = ["http", "https"];

/// `/^([a-z][a-z0-9+.-]*):/i`
fn scheme_name(url: &str) -> Option<&str> {
    let b = url.as_bytes();
    if !b.first()?.is_ascii_alphabetic() {
        return None;
    }
    let mut i = 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || matches!(b[i], b'+' | b'.' | b'-')) {
        i += 1;
    }
    (b.get(i) == Some(&b':')).then(|| &url[..i])
}

/// Stricter than Mermaid, which lets any scheme but three through: a diagram in a shared note
/// could otherwise open a local file, a network share, or another application.
fn allow<S: AsRef<str>>(url: &str, schemes: &[S]) -> Option<String> {
    let b = url.as_bytes();
    if b.len() >= 2 && matches!(b[0], b'\\' | b'/') && matches!(b[1], b'\\' | b'/') {
        return None;
    }
    match scheme_name(url) {
        None => Some(url.to_string()),
        Some(scheme) => {
            let lower = scheme.to_lowercase();
            schemes.iter().any(|s| s.as_ref() == lower).then(|| url.to_string())
        }
    }
}

/// Keeps a URL that `sanitize_url` has passed only when it is relative or its scheme is one
/// the host allows. One that is refused is left out of the drawing.
pub fn link_url(url: &str, options: &RenderOptions) -> Option<String> {
    if options.links == Some(false) {
        return None;
    }
    match &options.link_schemes {
        Some(list) => allow(url, list),
        None => allow(url, &LINK_SCHEMES),
    }
}

pub fn image_url(url: &str, options: &RenderOptions) -> Option<String> {
    if options.images == Some(false) {
        return None;
    }
    match &options.image_schemes {
        Some(list) => allow(url, list),
        None => allow(url, &IMAGE_SCHEMES),
    }
}

/// Wraps `body` in a link when the host allows the URL, and records the link.
pub fn linked(body: &str, url: &str, id: &str, links: &mut Vec<LinkInfo>, options: &RenderOptions, attrs: &str) -> String {
    let Some(href) = link_url(url, options) else { return body.to_string() };
    let out = format!("<a href=\"{}\"{}{}>{}</a>", esc(&href), attrs, rel_attr(options), body);
    links.push(LinkInfo { id: id.to_string(), href, internal: false });
    out
}

pub fn rel_attr(options: &RenderOptions) -> String {
    let rel = options.link_rel.as_deref().unwrap_or("noopener");
    if rel.is_empty() {
        String::new()
    } else {
        format!(" rel=\"{}\"", esc(rel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dangerous_schemes_become_blank() {
        for bad in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "  javascript:alert(1)",
            "java\u{0}script:alert(1)",
            "&#106;avascript:alert(1)",
            "%6A%61vascript:alert(1)",
            "data:text/html,<script>",
            "vbscript:msgbox",
            "java\\tscript:alert(1)",
        ] {
            assert_eq!(sanitize_url(bad), BLANK, "{bad}");
        }
        assert_eq!(sanitize_url(""), BLANK);
    }

    #[test]
    fn safe_urls_pass() {
        assert_eq!(sanitize_url("https://Example.com/a b"), "https://example.com/a%20b");
        assert_eq!(sanitize_url("/relative/path"), "/relative/path");
        assert_eq!(sanitize_url("./x"), "./x");
        assert_eq!(sanitize_url("mailto:a@b.c"), "mailto:a@b.c");
        assert_eq!(sanitize_url("https://example.com:8080/x"), "https://example.com:8080/x");
        assert_eq!(sanitize_url("plain"), "plain");
        assert_eq!(sanitize_url("http://%zz"), BLANK);
        // `\w+` takes the letters after the second code, which reads as NaN, so this is no scheme.
        assert_eq!(sanitize_url("&#0000106&#0000097vascript:alert(1)"), "j:alert(1)");
    }

    #[test]
    fn decode_uri() {
        assert_eq!(decode_uri_component("a%20b%E4%B8%AD").as_deref(), Some("a b中"));
        assert_eq!(decode_uri_component("%"), None);
        assert_eq!(decode_uri_component("%E4%B8"), None);
        assert_eq!(decode_uri_component("%C0%80"), None);
        assert_eq!(decode_uri_component("%ED%A0%80"), None);
    }

    #[test]
    fn host_policy() {
        let o = RenderOptions::default();
        assert_eq!(link_url("https://x", &o).as_deref(), Some("https://x"));
        assert_eq!(link_url("file:///etc/passwd", &o), None);
        assert_eq!(link_url("//share/x", &o), None);
        assert_eq!(link_url("\\\\share\\x", &o), None);
        assert_eq!(link_url("rel/x", &o).as_deref(), Some("rel/x"));
        assert_eq!(image_url("mailto:x", &o), None);
        let mut links = Vec::new();
        let html = linked("<text/>", "https://x?a=1&b=2", "A", &mut links, &o, "");
        assert_eq!(html, "<a href=\"https://x?a=1&amp;b=2\" rel=\"noopener\"><text/></a>");
        assert_eq!(links[0].href, "https://x?a=1&b=2");
    }
}
