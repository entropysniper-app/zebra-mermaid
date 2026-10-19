//! A port of Zebra's `MermaidSVGSecurityPolicy.validate` (Zebra/Editor/MermaidSVGSecurityPolicy.swift)
//! for tests: what Zebra's host accepts from its renderer.

use quick_xml::events::Event;
use quick_xml::Reader;

const FORBIDDEN_ELEMENTS: [&str; 10] = ["a", "audio", "canvas", "embed", "foreignobject", "iframe", "image", "object", "script", "video"];
const FORBIDDEN_SCHEMES: [&str; 7] = ["data:", "file:", "ftp:", "http:", "https:", "javascript:", "vbscript:"];

/// `MermaidResourcePolicy.containsForbiddenExternalReference`.
pub fn contains_forbidden_external_reference(value: &str) -> bool {
    let normalized = value.trim().to_lowercase();
    if FORBIDDEN_SCHEMES.iter().any(|s| normalized.contains(s)) || normalized.contains("@import") {
        return true;
    }
    let mut rest = normalized.as_str();
    while let Some(k) = rest.find("url(") {
        let suffix = rest[k + 4..].trim_start_matches(|c: char| c.is_whitespace() || c == '\'' || c == '"');
        if !suffix.starts_with('#') {
            return true;
        }
        let Some(close) = suffix.find(')') else { return true };
        rest = &suffix[close + 1..];
    }
    false
}

fn local_name(name: &str) -> String {
    let s = name.to_lowercase();
    s.rsplit(':').next().unwrap_or(&s).to_string()
}

/// Whether Zebra's host would accept this SVG. Returns the reason when it would not.
pub fn validate(svg: &str, maximum_utf8_bytes: usize) -> Result<(), String> {
    if svg.is_empty() || svg.len() > maximum_utf8_bytes {
        return Err("size".into());
    }
    let lower = svg.to_lowercase();
    if lower.contains("<!doctype") || lower.contains("<!entity") {
        return Err("doctype or entity".into());
    }
    let mut reader = Reader::from_str(svg);
    let mut depth: i64 = 0;
    let mut count = 0;
    let mut root: Option<String> = None;
    let mut style_depth: Option<i64> = None;
    let mut style_text = String::new();
    loop {
        let event = reader.read_event().map_err(|e| format!("xml: {e}"))?;
        let (start, empty) = match &event {
            Event::Start(e) => (Some(e.clone()), false),
            Event::Empty(e) => (Some(e.clone()), true),
            _ => (None, false),
        };
        if let Some(e) = start {
            let name = local_name(e.name().as_ref());
            root.get_or_insert_with(|| name.clone());
            count += 1;
            depth += 1;
            if count > 50_000 || depth > 256 || FORBIDDEN_ELEMENTS.contains(&name.as_str()) {
                return Err(format!("element {name}"));
            }
            if name == "style" {
                style_depth = Some(depth);
                style_text.clear();
            }
            for attr in e.attributes() {
                let attr = attr.map_err(|e| format!("attribute: {e}"))?;
                let key = attr.key.as_ref().to_lowercase();
                let value = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0).map_err(|e| format!("value: {e}"))?.to_string();
                let lv = value.to_lowercase();
                if (key == "xmlns" && lv == "http://www.w3.org/2000/svg") || (key == "xmlns:xlink" && lv == "http://www.w3.org/1999/xlink")
                {
                    continue;
                }
                if key.starts_with("on") {
                    return Err(format!("event attribute {key}"));
                }
                if key == "href" || key == "xlink:href" {
                    if !value.trim().starts_with('#') {
                        return Err(format!("href {value}"));
                    }
                } else if contains_forbidden_external_reference(&value) {
                    return Err(format!("{key}={value}"));
                }
            }
            if empty {
                depth -= 1;
            }
            continue;
        }
        match event {
            Event::End(e) => {
                let name = local_name(e.name().as_ref());
                if name == "style" && style_depth == Some(depth) {
                    if contains_forbidden_external_reference(&style_text) {
                        return Err("style text".into());
                    }
                    style_depth = None;
                }
                depth -= 1;
                if depth < 0 {
                    return Err("depth".into());
                }
            }
            Event::Text(t) => {
                if style_depth.is_some() {
                    style_text.push_str(t.as_ref());
                }
            }
            Event::PI(_) => return Err("processing instruction".into()),
            Event::DocType(_) => return Err("doctype".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    if root.as_deref() != Some("svg") || depth != 0 {
        return Err("root".into());
    }
    Ok(())
}
