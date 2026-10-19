//! Mermaid's box-drawing preprocessor: turns `├──`, `└──`, `│` (and the heavy variants) into
//! the indentation the grammar reads. Ported from Pele `src/diagrams/treeview/boxDrawing.ts`.

use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::chars::{is_space, is_word};
use crate::util::js;

/// `/[─━│┃└┗├┣]/`
fn is_box_char(c: char) -> bool {
    matches!(c, '─' | '━' | '│' | '┃' | '└' | '┗' | '├' | '┣')
}

/// `/[└┗├┣]/`
fn is_branch_char(c: char) -> bool {
    matches!(c, '└' | '┗' | '├' | '┣')
}

/// `/[─━]/`
fn is_dash_char(c: char) -> bool {
    matches!(c, '─' | '━')
}

fn has_box_char(text: &str) -> bool {
    text.chars().any(is_box_char)
}

/// `/^[\s│┃]+$/`
fn is_decoration_only(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| is_space(c) || c == '│' || c == '┃')
}

/// `/^[\s─━│┃└┗├┣]+$/`
fn is_box_only(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| is_space(c) || is_box_char(c))
}

/// `/^\s*(title[\t ]|accTitle[\t ]*:|accDescr[\t ]*[:{])/`
fn is_metadata_line(line: &str) -> bool {
    let rest = js::trim_start(line);
    if let Some(after) = rest.strip_prefix("title") {
        return after.starts_with([' ', '\t']);
    }
    if let Some(after) = rest.strip_prefix("accTitle") {
        return after.trim_start_matches([' ', '\t']).starts_with(':');
    }
    if let Some(after) = rest.strip_prefix("accDescr") {
        return after.trim_start_matches([' ', '\t']).starts_with([':', '{']);
    }
    false
}

/// `/^\s*%%/`
fn is_comment_line(line: &str) -> bool {
    js::trim_start(line).starts_with("%%")
}

const INDENT_UNIT: &str = "    ";

pub struct PreprocessResult {
    pub text: String,
    /// Output line number to original line number, both from 1. Empty when the text is unchanged.
    pub line_map: Vec<usize>,
}

impl PreprocessResult {
    /// `lineMap.get(line)`
    pub fn original(&self, line: usize) -> Option<usize> {
        line.checked_sub(1).and_then(|i| self.line_map.get(i)).copied()
    }
}

/// The first branch character of a line: its byte offset and its offset in UTF-16 code units.
fn branch_at(line: &str) -> Option<(usize, usize)> {
    let (at, _) = line.char_indices().find(|&(_, c)| is_branch_char(c))?;
    Some((at, js::len16(&line[..at])))
}

fn infer_segment_width(content_lines: &[String]) -> usize {
    for line in content_lines {
        if let Some((_, index)) = branch_at(line) {
            if index > 0 {
                return index;
            }
        }
    }
    4
}

/// `message.replace(/\bline\s+(\d+)\b/gi, ...)`: each line number that the map knows becomes the
/// original one, written `line N` in lower case.
pub fn remap_error_lines(message: &str, result: &PreprocessResult) -> String {
    let b = message.as_bytes();
    let mut out = String::with_capacity(message.len());
    let mut last = 0;
    let mut i = 0;
    while i + 4 <= b.len() {
        let boundary_before = i == 0 || !is_word(b[i - 1] as char);
        if !boundary_before || !b[i..i + 4].eq_ignore_ascii_case(b"line") {
            i += 1;
            continue;
        }
        let ws = crate::diagrams::common::tokens::spaces(message, i + 4);
        let mut d = ws;
        while d < b.len() && b[d].is_ascii_digit() {
            d += 1;
        }
        // `\s+`, then `\d+` up to a word boundary. The digits cannot give back to reach one, as
        // a boundary never falls between two digits.
        let matched = ws > i + 4 && d > ws && (d == b.len() || !is_word(b[d] as char));
        if matched {
            let original = message[ws..d].parse::<usize>().ok().and_then(|n| result.original(n));
            if let Some(original) = original {
                out.push_str(&message[last..i]);
                out.push_str(&format!("line {}", original));
                last = d;
            }
            i = d;
            continue;
        }
        i += 1;
    }
    out.push_str(&message[last..]);
    out
}

fn fail(line: usize, message: &str) -> Error {
    Error::syntax(format!("Line {}: {}", line, message)).with_type(DiagramType::TreeView).with_location(line, 0, "")
}

/// Rewrites box-drawing input as indented lines. A failure names the line of `input`, and its
/// byte offset there.
pub fn preprocess_box_drawing(input: &str) -> Result<PreprocessResult, (Error, usize)> {
    let unchanged = || PreprocessResult { text: input.to_string(), line_map: Vec::new() };
    if !has_box_char(input) {
        return Ok(unchanged());
    }
    let lines: Vec<&str> = input.split('\n').collect();
    let mut starts = Vec::with_capacity(lines.len());
    let mut at = 0;
    for line in &lines {
        starts.push(at);
        at += line.len() + 1;
    }

    let Some(keyword_idx) = lines.iter().position(|line| js::trim(line) == "treeView-beta") else {
        return Ok(unchanged());
    };

    let passes = |line: &str| js::trim(line).is_empty() || is_comment_line(line) || is_metadata_line(line);

    let mut content_lines: Vec<String> = Vec::new();
    for line in &lines[keyword_idx + 1..] {
        if passes(line) || is_decoration_only(line) {
            continue;
        }
        content_lines.push(line.replace('\t', "    "));
    }
    if !content_lines.iter().any(|line| has_box_char(line)) {
        return Ok(unchanged());
    }

    let segment_width = infer_segment_width(&content_lines);
    let mut out: Vec<String> = Vec::new();
    let mut line_map: Vec<usize> = Vec::new();
    let mut keep = |text: String, index: usize| {
        out.push(text);
        line_map.push(index + 1);
    };

    for (i, line) in lines.iter().enumerate().take(keyword_idx + 1) {
        keep(line.to_string(), i);
    }
    for (i, &line) in lines.iter().enumerate().skip(keyword_idx + 1) {
        if passes(line) {
            keep(line.to_string(), i);
            continue;
        }
        if is_decoration_only(line) {
            continue;
        }

        let normalized = line.replace('\t', "    ");
        if let Some((at, index)) = branch_at(&normalized) {
            let depth = js::round(index as f64 / segment_width as f64) as usize + 1;
            let mut pos = at + normalized[at..].chars().next().map_or(1, char::len_utf8);
            while let Some(c) = normalized[pos..].chars().next().filter(|&c| is_dash_char(c)) {
                pos += c.len_utf8();
            }
            while normalized.as_bytes().get(pos) == Some(&b' ') {
                pos += 1;
            }
            let content = js::trim_end(&normalized[pos..]);
            if content.is_empty() {
                return Err((fail(i + 1, "Empty node — expected a filename or directory name after the box-drawing prefix"), starts[i]));
            }
            keep(format!("{}{}", INDENT_UNIT.repeat(depth), content), i);
        } else if is_box_only(&normalized) {
            continue;
        } else if has_box_char(&normalized) {
            // A name that happens to contain a box character, such as "Section ─ A.txt", is a
            // root item.
            keep(line.to_string(), i);
        } else if normalized.chars().next().is_some_and(is_space) {
            return Err((
                fail(
                    i + 1,
                    "Unexpected indentation without box-drawing characters. In box-drawing format, use ├── or └── prefixes for indented nodes.",
                ),
                starts[i],
            ));
        } else {
            keep(line.to_string(), i);
        }
    }
    Ok(PreprocessResult { text: out.join("\n"), line_map })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_branches() {
        let r = preprocess_box_drawing("treeView-beta\nroot/\n├── a.txt\n│   └── b/\n└── c\n").ok().unwrap();
        assert_eq!(r.text, "treeView-beta\nroot/\n    a.txt\n        b/\n    c\n");
        assert_eq!(r.line_map, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn plain_input_is_unchanged() {
        let r = preprocess_box_drawing("treeView-beta\n  a\n").ok().unwrap();
        assert_eq!(r.text, "treeView-beta\n  a\n");
        assert!(r.line_map.is_empty());
    }

    #[test]
    fn remaps_line_numbers() {
        let r = PreprocessResult { text: String::new(), line_map: vec![1, 2, 5] };
        assert_eq!(remap_error_lines("Parse error on line 3:", &r), "Parse error on line 5:");
        assert_eq!(remap_error_lines("Line 3: x, line 9, line 3a, inline 3", &r), "line 5: x, line 9, line 3a, inline 3");
    }

    #[test]
    fn metadata_lines() {
        assert!(is_metadata_line("  title x"));
        assert!(!is_metadata_line("title"));
        assert!(is_metadata_line("accTitle : x"));
        assert!(is_metadata_line("accDescr {"));
        assert!(!is_metadata_line("accDescr x"));
    }
}
