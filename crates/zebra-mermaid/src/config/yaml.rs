//! A small YAML reader covering what Mermaid feeds to js-yaml with the JSON schema: block and
//! flow collections, quoted and plain scalars, and block scalars. Ported line by line from
//! Pele `src/util/yaml.ts`, including the places where it is more lenient than js-yaml.

use std::borrow::Cow;

use super::value::{ConfigMap, ConfigValue};
use crate::util::chars::{char_at, char_before, is_space};
use crate::util::js;

pub const MAX_DEPTH: usize = 64;

/// A YAML error. `incomplete` marks a quoted scalar or flow collection that runs past the text
/// read so far, which makes the reader take more lines and try again.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlError {
    pub message: String,
    incomplete: bool,
}

impl YamlError {
    fn new(message: impl Into<String>) -> Self {
        YamlError { message: message.into(), incomplete: false }
    }

    fn incomplete(message: impl Into<String>) -> Self {
        YamlError { message: message.into(), incomplete: true }
    }
}

type Result<T> = std::result::Result<T, YamlError>;

/// The lines of the document. They borrow the text until one is rewritten, as the reader
/// does with a sequence entry that opens a mapping.
struct Reader<'a> {
    lines: Vec<Cow<'a, str>>,
    i: usize,
}

impl<'a> Reader<'a> {
    /// `normalized` has had its `\r\n` and `\r` turned into `\n`.
    fn new(normalized: &'a str) -> Self {
        Reader { lines: normalized.split('\n').map(Cow::Borrowed).collect(), i: 0 }
    }

    /// The current line from byte `from`, as text that outlives moves of the reader.
    fn line_from(&self, from: usize) -> Cow<'a, str> {
        match &self.lines[self.i] {
            Cow::Borrowed(s) => Cow::Borrowed(&s[from..]),
            Cow::Owned(s) => Cow::Owned(s[from..].to_string()),
        }
    }

    fn skip_blank(&mut self) {
        while self.i < self.lines.len() {
            let t = js::trim(&self.lines[self.i]);
            if !t.is_empty() && !t.starts_with('#') && t != "---" {
                break;
            }
            self.i += 1;
        }
    }

    fn indent(&self) -> Result<usize> {
        let line = self.lines[self.i].as_bytes();
        let mut k = 0;
        while k < line.len() && line[k] == b' ' {
            k += 1;
        }
        if line.get(k) == Some(&b'\t') {
            return Err(YamlError::new("YAML: tabs are not allowed for indentation"));
        }
        Ok(k)
    }
}

/// The number of leading white space characters, as `s.length - s.trimStart().length` counts.
fn lead(s: &str) -> usize {
    s.chars().take_while(|&c| is_space(c)).count()
}

/// `s.slice(n)` where the first `n` characters are known to be single UTF-16 units.
fn skip_chars(s: &str, n: usize) -> &str {
    let bytes: usize = s.chars().take(n).map(char::len_utf8).sum();
    &s[bytes..]
}

fn is_blank(b: Option<u8>) -> bool {
    matches!(b, Some(b' ' | b'\t' | b'\n' | b'\r'))
}

/// `/[\s#:,[\]{}&*!|>'"%@`]/`
fn is_key_start(c: char) -> bool {
    is_space(c) || "#:,[]{}&*!|>'\"%@`".contains(c)
}

/// Splits `key: rest` at the start of a line: the raw key and where the value starts.
fn split_key(line: &str) -> Option<(&str, usize)> {
    let b = line.as_bytes();
    let first = *b.first()?;
    let end;
    let mut colon;
    if first == b'"' || first == b'\'' {
        let mut i = 1;
        loop {
            if i >= b.len() {
                return None;
            }
            let c = b[i];
            if first == b'"' && c == b'\\' {
                // Skips the backslash and the character after it.
                i += 1 + char_at(line, i + 1).map_or(1, char::len_utf8);
            } else if c == first {
                if first == b'"' || b.get(i + 1) != Some(&b'\'') {
                    break;
                }
                i += 2;
            } else {
                i += 1;
            }
        }
        end = i + 1;
        colon = end;
        while is_blank(b.get(colon).copied()) {
            colon += 1;
        }
        if b.get(colon) != Some(&b':') {
            return None;
        }
    } else {
        let c = char_at(line, 0)?;
        if is_key_start(c) {
            return None;
        }
        if first == b'-' && (b.len() < 2 || is_blank(Some(b[1])) || b[1] == b':' || b[1] == b'#') {
            return None;
        }
        colon = 1;
        while colon < b.len() && b[colon] != b':' && b[colon] != b'#' {
            colon += 1;
        }
        if b.get(colon) != Some(&b':') {
            return None;
        }
        let mut e = colon;
        while e > 0 && is_blank(Some(b[e - 1])) {
            e -= 1;
        }
        end = e;
    }
    let mut length = colon + 1;
    if length < b.len() && !is_blank(Some(b[length])) {
        return None;
    }
    while is_blank(b.get(length).copied()) {
        length += 1;
    }
    Some((&line[..end], length))
}

fn is_int(s: &str) -> bool {
    let body = s.strip_prefix(['-', '+']).unwrap_or(s);
    body == "0" || (!body.is_empty() && body.as_bytes()[0] != b'0' && body.bytes().all(|b| b.is_ascii_digit()))
}

/// `/^[-+]?(?:[0-9][0-9]*(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][-+]?[0-9]+)?$/`
fn is_float(s: &str) -> bool {
    let b = s.strip_prefix(['-', '+']).unwrap_or(s).as_bytes();
    let mut i = 0;
    if i < b.len() && b[i].is_ascii_digit() {
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i < b.len() && b[i] == b'.' {
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        }
    } else if i < b.len() && b[i] == b'.' {
        i += 1;
        let f = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == f {
            return false;
        }
    } else {
        return false;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
            i += 1;
        }
        let f = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == f {
            return false;
        }
    }
    i == b.len()
}

fn is_radix(s: &str, prefix: &str, digit: fn(u8) -> bool) -> bool {
    let body = s.strip_prefix(['-', '+']).unwrap_or(s);
    body.len() > 2 && body.starts_with(prefix) && body.as_bytes()[2..].iter().all(|&b| digit(b))
}

fn scalar(raw: &str) -> ConfigValue {
    let s = js::trim(raw);
    match s {
        "" | "~" | "null" | "Null" | "NULL" => return ConfigValue::Null,
        "true" | "True" | "TRUE" => return ConfigValue::Bool(true),
        "false" | "False" | "FALSE" => return ConfigValue::Bool(false),
        _ => {}
    }
    if is_int(s) || is_float(s) {
        return ConfigValue::Number(js::number(s));
    }
    if is_radix(s, "0x", |b| b.is_ascii_hexdigit())
        || is_radix(s, "0o", |b| (b'0'..=b'7').contains(&b))
        || is_radix(s, "0b", |b| b == b'0' || b == b'1')
    {
        let neg = s.starts_with('-');
        let v = js::number(if neg || s.starts_with('+') { &s[1..] } else { s });
        return ConfigValue::Number(if neg { -v } else { v });
    }
    let unsigned = s.strip_prefix(['-', '+']).unwrap_or(s);
    if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
        return ConfigValue::Number(if s.starts_with('-') { f64::NEG_INFINITY } else { f64::INFINITY });
    }
    if matches!(s, ".nan" | ".NaN" | ".NAN") {
        return ConfigValue::Number(f64::NAN);
    }
    ConfigValue::String(s.to_string())
}

/// Folds the line breaks of a scalar: blanks around a break go, one break becomes a space,
/// and each further break stays. Text without a break is borrowed.
fn fold(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') {
        return Cow::Borrowed(text);
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len() - 1;
    let mut trimmed: Vec<&str> = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let b = line.as_bytes();
        let mut from = 0;
        let mut to = b.len();
        if i > 0 {
            while from < to && (b[from] == b' ' || b[from] == b'\t') {
                from += 1;
            }
        }
        if i < last {
            while to > from && (b[to - 1] == b' ' || b[to - 1] == b'\t') {
                to -= 1;
            }
        }
        trimmed.push(&line[from..to]);
    }
    let joined = trimmed.join("\n");
    // `.replace(/\n+/g, (m) => (m.length === 1 ? ' ' : '\n'.repeat(m.length - 1)))`
    let mut out = String::with_capacity(joined.len());
    let mut run: usize = 0;
    for c in joined.chars() {
        if c == '\n' {
            run += 1;
            continue;
        }
        if run > 0 {
            if run == 1 {
                out.push(' ');
            } else {
                for _ in 0..run - 1 {
                    out.push('\n');
                }
            }
            run = 0;
        }
        out.push(c);
    }
    if run == 1 {
        out.push(' ');
    } else {
        for _ in 0..run.saturating_sub(1) {
            out.push('\n');
        }
    }
    Cow::Owned(out)
}

fn escape(e: char) -> Option<char> {
    Some(match e {
        '0' => '\0',
        'a' => '\u{7}',
        'b' => '\u{8}',
        't' => '\t',
        'n' => '\n',
        'v' => '\u{b}',
        'f' => '\u{c}',
        'r' => '\r',
        'e' => '\u{1b}',
        ' ' => ' ',
        '"' => '"',
        '/' => '/',
        '\\' => '\\',
        'N' => '\u{85}',
        '_' => '\u{a0}',
        'L' => '\u{2028}',
        'P' => '\u{2029}',
        _ => return None,
    })
}

/// `body.replace(/\\\n[ \t]*/g, '')`
fn drop_escaped_breaks(body: &str) -> String {
    let b = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut last = 0;
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'\\' && b[i + 1] == b'\n' {
            out.push_str(&body[last..i]);
            let mut j = i + 2;
            while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            last = j;
            i = j;
        } else {
            i += 1;
        }
    }
    out.push_str(&body[last..]);
    out
}

fn unquote_double(body: &str) -> Result<String> {
    let dropped = drop_escaped_breaks(body);
    let s = fold(&dropped);
    let mut out = String::with_capacity(s.len());
    let mut chars = s.char_indices().peekable();
    // A high surrogate from a `\u` escape, waiting for the low one that may follow it.
    let mut pending: Option<u32> = None;
    let flush = |out: &mut String, pending: &mut Option<u32>| {
        if pending.take().is_some() {
            out.push('\u{fffd}');
        }
    };
    while let Some((i, ch)) = chars.next() {
        if ch != '\\' {
            flush(&mut out, &mut pending);
            out.push(ch);
            continue;
        }
        let Some((j, e)) = chars.next() else {
            return Err(YamlError::new("YAML: unknown escape sequence"));
        };
        let _ = i;
        if e == 'x' || e == 'u' || e == 'U' {
            let len = match e {
                'x' => 2,
                'u' => 4,
                _ => 8,
            };
            let start = j + 1;
            let end = js::byte_at16(&s[start..], len) + start;
            let n = js::parse_int(&s[start..end], 16);
            // Skip `len` UTF-16 units.
            let mut skipped = 0;
            while skipped < len {
                match chars.peek() {
                    Some(&(_, c)) => {
                        skipped += c.len_utf16();
                        chars.next();
                    }
                    None => break,
                }
            }
            if !(n.is_finite() && (0.0..=1_114_111.0).contains(&n) && n.fract() == 0.0) {
                return Err(YamlError::new(format!("Invalid code point {}", js::to_string(n))));
            }
            let code = n as u32;
            if (0xdc00..0xe000).contains(&code) {
                if let Some(hi) = pending.take() {
                    let c = 0x10000 + ((hi - 0xd800) << 10) + (code - 0xdc00);
                    out.push(char::from_u32(c).unwrap_or('\u{fffd}'));
                    continue;
                }
                out.push('\u{fffd}');
                continue;
            }
            flush(&mut out, &mut pending);
            if (0xd800..0xdc00).contains(&code) {
                pending = Some(code);
                continue;
            }
            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
        } else if let Some(c) = escape(e) {
            flush(&mut out, &mut pending);
            out.push(c);
        } else {
            return Err(YamlError::new("YAML: unknown escape sequence"));
        }
    }
    flush(&mut out, &mut pending);
    Ok(out)
}

fn skip(s: &str, mut pos: usize, commas: bool) -> usize {
    let b = s.as_bytes();
    while pos < b.len() {
        let c = b[pos];
        if c == b' ' || c == b'\n' || c == b'\t' || c == b'\r' || (commas && c == b',') {
            pos += 1;
        } else if c == b'#' && (pos == 0 || char_before(s, pos).is_some_and(is_space)) {
            pos = s[pos..].find('\n').map_or(s.len(), |k| pos + k);
        } else {
            break;
        }
    }
    pos
}

/// Entries of a flow collection are separated by exactly one comma, as js-yaml requires.
fn entry_start(ch: u8, separated: bool) -> Result<()> {
    if !separated {
        return Err(YamlError::new("YAML: missed comma between flow collection entries"));
    }
    if ch == b',' {
        return Err(YamlError::new("YAML: expected the node content, but found ','"));
    }
    Ok(())
}

/// Parses a flow value starting at `s[pos]`; returns the value and the index after it.
fn flow(s: &str, pos: usize, stops: &str, depth: usize) -> Result<(ConfigValue, usize)> {
    if depth > MAX_DEPTH {
        return Err(YamlError::new("YAML: nesting is too deep"));
    }
    let b = s.as_bytes();
    let mut pos = skip(s, pos, false);
    let ch = b.get(pos).copied();
    if ch == Some(b'{') {
        let mut out = ConfigMap::new();
        pos += 1;
        let mut separated = true;
        loop {
            pos = skip(s, pos, false);
            if pos >= b.len() {
                return Err(YamlError::incomplete("YAML: unexpected end of the stream within a flow collection"));
            }
            if b[pos] == b'}' {
                return Ok((ConfigValue::Object(out), pos + 1));
            }
            entry_start(b[pos], separated)?;
            let (key, p) = flow(s, pos, ":,}", depth + 1)?;
            pos = skip(s, p, false);
            let mut value = ConfigValue::Null;
            if b.get(pos) == Some(&b':') {
                let (v, p) = flow(s, pos + 1, ",}", depth + 1)?;
                value = v;
                pos = p;
            }
            out.insert(key.into_js_string(), value);
            pos = skip(s, pos, false);
            separated = b.get(pos) == Some(&b',');
            if separated {
                pos += 1;
            }
        }
    }
    if ch == Some(b'[') {
        let mut out = Vec::new();
        pos += 1;
        let mut separated = true;
        loop {
            pos = skip(s, pos, false);
            if pos >= b.len() {
                return Err(YamlError::incomplete("YAML: unexpected end of the stream within a flow collection"));
            }
            if b[pos] == b']' {
                return Ok((ConfigValue::Array(out), pos + 1));
            }
            entry_start(b[pos], separated)?;
            let (value, p) = flow(s, pos, ",]", depth + 1)?;
            out.push(value);
            pos = skip(s, p, false);
            separated = b.get(pos) == Some(&b',');
            if separated {
                pos += 1;
            }
        }
    }
    if ch == Some(b'"') {
        let mut end = pos + 1;
        while end < b.len() && b[end] != b'"' {
            end += if b[end] == b'\\' { 1 + char_at(s, end + 1).map_or(1, char::len_utf8) } else { 1 };
        }
        if end >= b.len() {
            return Err(YamlError::incomplete("YAML: unexpected end of the stream within a double quoted scalar"));
        }
        return Ok((ConfigValue::String(unquote_double(&s[pos + 1..end])?), end + 1));
    }
    if ch == Some(b'\'') {
        let mut end = pos + 1;
        while end < b.len() {
            if b[end] == b'\'' {
                if b.get(end + 1) != Some(&b'\'') {
                    break;
                }
                end += 1;
            }
            end += 1;
        }
        if end >= b.len() {
            return Err(YamlError::incomplete("YAML: unexpected end of the stream within a single quoted scalar"));
        }
        return Ok((ConfigValue::String(fold(&s[pos + 1..end]).replace("''", "'")), end + 1));
    }
    let mut end = pos;
    while end < b.len() {
        let c = b[end];
        if stops.as_bytes().contains(&c) {
            let next_breaks = end + 1 >= b.len() || {
                let n = char_at(s, end + 1).expect("boundary");
                is_space(n) || matches!(n, ',' | '[' | ']' | '{' | '}')
            };
            if c != b':' || next_breaks {
                break;
            }
        }
        if c == b'#' && end > pos && char_before(s, end).is_some_and(is_space) {
            let nl = s[end..].find('\n').map_or(s.len(), |k| end + k);
            return Ok((scalar(&fold(js::trim(&s[pos..end]))), nl));
        }
        end += 1;
    }
    Ok((scalar(&fold(js::trim(&s[pos..end]))), end))
}

fn strip_comment(s: &str) -> &str {
    let b = s.as_bytes();
    let mut quote = 0u8;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if quote != 0 {
            if c == b'\\' && quote == b'"' {
                i += 1;
            } else if c == quote {
                quote = 0;
            }
        } else if c == b'"' || c == b'\'' {
            quote = c;
        } else if c == b'#' && (i == 0 || b[i - 1] == b' ' || b[i - 1] == b'\t') {
            return &s[..i];
        }
        i += 1;
    }
    s
}

fn block_scalar(r: &mut Reader<'_>, header: &str, parent_indent: isize) -> String {
    let folded = header.starts_with('>');
    let chomp = if header.contains('-') {
        '-'
    } else if header.contains('+') {
        '+'
    } else {
        ' '
    };
    let explicit = header.bytes().find(|b| (b'1'..=b'9').contains(b));
    let mut indent: Option<isize> = explicit.map(|d| parent_indent + (d - b'0') as isize);
    let mut out: Vec<String> = Vec::new();
    while r.i < r.lines.len() {
        let line: &str = &r.lines[r.i];
        if js::trim(line).is_empty() {
            out.push(String::new());
            r.i += 1;
            continue;
        }
        let k = lead(line) as isize;
        let ind = match indent {
            Some(i) => i,
            None => {
                if k <= parent_indent {
                    break;
                }
                indent = Some(k);
                k
            }
        };
        if k < ind {
            break;
        }
        // `line.slice(indent)`; a negative indent (from an explicit indicator) counts from the end.
        let kept = if ind >= 0 {
            skip_chars(line, ind as usize).to_string()
        } else {
            js::slice16(line, js::len16(line).saturating_sub((-ind) as usize), usize::MAX).to_string()
        };
        out.push(kept);
        r.i += 1;
    }
    let mut trailing = 0;
    while out.last().is_some_and(String::is_empty) {
        out.pop();
        trailing += 1;
    }
    let text = if folded {
        let mut text = String::new();
        for i in 0..out.len() {
            let cur = &out[i];
            if i > 0 {
                let prev = &out[i - 1];
                if cur.is_empty() {
                    text.push('\n');
                } else if !prev.is_empty() {
                    let starts_blank = |s: &str| s.starts_with(' ') || s.starts_with('\t');
                    text.push(if starts_blank(prev) || starts_blank(cur) { '\n' } else { ' ' });
                }
            }
            text.push_str(cur);
        }
        text
    } else {
        out.join("\n")
    };
    if out.is_empty() {
        return if chomp == '+' { "\n".repeat(trailing) } else { String::new() };
    }
    match chomp {
        '-' => text,
        '+' => text + &"\n".repeat(trailing + 1),
        _ => text + "\n",
    }
}

/// `/^[|>][-+0-9]*\s*(?:#.*)?$/`
fn is_block_header(rest: &str) -> bool {
    let mut it = rest.char_indices().peekable();
    match it.next() {
        Some((_, '|' | '>')) => {}
        _ => return false,
    }
    while matches!(it.peek(), Some((_, '-' | '+' | '0'..='9'))) {
        it.next();
    }
    while it.peek().is_some_and(|&(_, c)| is_space(c)) {
        it.next();
    }
    match it.next() {
        None => true,
        Some((_, '#')) => it.all(|(_, c)| !matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')),
        Some(_) => false,
    }
}

fn inline_value(r: &mut Reader<'_>, rest: &str, indent: isize) -> Result<ConfigValue> {
    let first = rest.as_bytes().first().copied();
    if (first == Some(b'|') || first == Some(b'>')) && is_block_header(rest) {
        return Ok(ConfigValue::String(block_scalar(r, rest, indent)));
    }
    if matches!(first, Some(b'{' | b'[' | b'"' | b'\'')) {
        // The value may continue on later lines. Read them in growing batches until it closes.
        let start = r.i;
        let mut text = rest.to_string();
        let mut taken = 0;
        let mut batch = 1;
        loop {
            let attempt = (|| -> Result<(ConfigValue, usize)> {
                let (value, end) = flow(&text, 0, "", 0)?;
                let used = text[..end].matches('\n').count();
                let line_end = text[end..].find('\n').map_or(text.len(), |k| end + k);
                if !js::trim(strip_comment(&text[end..line_end])).is_empty() {
                    return Err(YamlError::new("YAML: unexpected content after a value"));
                }
                Ok((value, used))
            })();
            match attempt {
                Ok((value, used)) => {
                    r.i = start + used;
                    return Ok(value);
                }
                Err(err) if err.incomplete => {
                    if start + taken >= r.lines.len() {
                        return Err(YamlError::new(err.message));
                    }
                    let to = (start + taken + batch).min(r.lines.len());
                    let more = &r.lines[start + taken..to];
                    text.push('\n');
                    text.push_str(&more.join("\n"));
                    taken += more.len();
                    batch *= 2;
                }
                Err(err) => return Err(err),
            }
        }
    }
    // A plain scalar is borrowed until a further line continues it.
    let mut text: Cow<'_, str> = Cow::Borrowed(js::trim(strip_comment(rest)));
    while r.i < r.lines.len() {
        let line: &str = &r.lines[r.i];
        if js::trim(line).is_empty() {
            let mut j = r.i;
            while j < r.lines.len() && js::trim(&r.lines[j]).is_empty() {
                j += 1;
            }
            if j >= r.lines.len() || lead(&r.lines[j]) as isize <= indent {
                break;
            }
            let breaks = j - r.i;
            text.to_mut().push_str(&"\n".repeat(breaks));
            r.i = j;
            continue;
        }
        if lead(line) as isize <= indent {
            break;
        }
        let more = js::trim(strip_comment(line));
        let text = text.to_mut();
        if !text.ends_with('\n') {
            text.push(' ');
        }
        text.push_str(more);
        r.i += 1;
    }
    Ok(scalar(&text))
}

fn is_seq_line(line: &str) -> bool {
    line == "-" || line.starts_with("- ")
}

fn block(r: &mut Reader<'_>, min_indent: usize, depth: usize) -> Result<ConfigValue> {
    if depth > MAX_DEPTH {
        return Err(YamlError::new("YAML: nesting is too deep"));
    }
    r.skip_blank();
    if r.i >= r.lines.len() {
        return Ok(ConfigValue::Null);
    }
    let indent = r.indent()?;
    if indent < min_indent {
        return Ok(ConfigValue::Null);
    }
    let line = r.line_from(indent);

    if is_seq_line(&line) {
        let mut out = Vec::new();
        while r.i < r.lines.len() {
            r.skip_blank();
            if r.i >= r.lines.len() || r.indent()? != indent {
                break;
            }
            let cur = r.line_from(indent);
            if !is_seq_line(&cur) {
                break;
            }
            let rest = &cur[1..];
            let pad = lead(rest) + 1;
            let body = js::trim(strip_comment(rest));
            if body.is_empty() {
                r.i += 1;
                out.push(block(r, indent + 1, depth + 1)?);
            } else if split_key(body).is_some() {
                r.lines[r.i] = Cow::Owned(format!("{}{}", " ".repeat(indent + pad), js::trim_start(rest)));
                out.push(block(r, indent + pad, depth + 1)?);
            } else {
                r.i += 1;
                out.push(inline_value(r, js::trim(rest), indent as isize)?);
            }
        }
        return Ok(ConfigValue::Array(out));
    }

    if split_key(&line).is_none() {
        r.i += 1;
        return inline_value(r, js::trim(&line), indent as isize - 1);
    }

    let mut out = ConfigMap::new();
    while r.i < r.lines.len() {
        r.skip_blank();
        if r.i >= r.lines.len() {
            break;
        }
        let k = r.indent()?;
        if k < indent {
            break;
        }
        if k > indent {
            return Err(YamlError::new("YAML: bad indentation of a mapping entry"));
        }
        let cur = r.line_from(indent);
        let Some((raw_key, length)) = split_key(&cur) else {
            return Err(YamlError::new("YAML: can not read a block mapping entry"));
        };
        let key = flow(raw_key, 0, "", 0)?.0.into_js_string();
        let rest = &cur[length..];
        r.i += 1;
        if js::trim(strip_comment(rest)).is_empty() {
            r.skip_blank();
            if r.i < r.lines.len() {
                let next = r.indent()?;
                let seq = is_seq_line(&r.lines[r.i][next..]);
                let value = if next > indent || (seq && next == indent) {
                    block(r, if seq { indent } else { indent + 1 }, depth + 1)?
                } else {
                    ConfigValue::Null
                };
                out.insert(key, value);
            } else {
                out.insert(key, ConfigValue::Null);
            }
        } else {
            let value = inline_value(r, js::trim(rest), indent as isize)?;
            out.insert(key, value);
        }
    }
    Ok(ConfigValue::Object(out))
}

/// Reads a YAML document. Errors carry js-yaml style messages.
pub fn parse_yaml(src: &str) -> std::result::Result<ConfigValue, YamlError> {
    let trimmed = js::trim(src);
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        match flow(trimmed, 0, "", 0) {
            Ok((value, end)) => {
                if skip(trimmed, end, false) == trimmed.len() {
                    return Ok(value);
                }
            }
            Err(err) if err.incomplete => return Err(YamlError::new(err.message)),
            Err(err) => return Err(err),
        }
    }
    let normalized: Cow<'_, str> =
        if src.contains('\r') { Cow::Owned(src.replace("\r\n", "\n").replace('\r', "\n")) } else { Cow::Borrowed(src) };
    let mut r = Reader::new(&normalized);
    let value = block(&mut r, 0, 0)?;
    r.skip_blank();
    if r.i < r.lines.len() && js::trim(&r.lines[r.i]) != "..." {
        return Err(YamlError::new("YAML: end of the stream or a document separator is expected"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(v: &ConfigValue) -> &ConfigMap {
        v.as_object().expect("object")
    }

    #[test]
    fn mappings_and_scalars() {
        let v = parse_yaml(
            "title: Hello\nconfig:\n  pie:\n    textPosition: 0.5\n  flag: true\n  n: ~\n  hex: 0x1F\n  neg: -0o10\n  inf: -.inf\n",
        )
        .unwrap();
        let m = obj(&v);
        assert_eq!(m.get("title").unwrap().as_str(), Some("Hello"));
        let c = obj(m.get("config").unwrap());
        assert_eq!(c.lookup("pie", "textPosition").unwrap().as_f64(), Some(0.5));
        assert_eq!(c.get("flag"), Some(&ConfigValue::Bool(true)));
        assert_eq!(c.get("n"), Some(&ConfigValue::Null));
        assert_eq!(c.get("hex").unwrap().as_f64(), Some(31.0));
        assert_eq!(c.get("neg").unwrap().as_f64(), Some(-8.0));
        assert_eq!(c.get("inf").unwrap().as_f64(), Some(f64::NEG_INFINITY));
    }

    #[test]
    fn flow_and_quotes() {
        let v = parse_yaml("a: {x: 1, 'y z': [1, \"two\", three]}\nb: 'it''s'\nc: \"tab\\tq\\u00e9\"\n").unwrap();
        let m = obj(&v);
        let a = obj(m.get("a").unwrap());
        assert_eq!(a.get("x").unwrap().as_f64(), Some(1.0));
        assert_eq!(a.get("y z").unwrap().as_array().unwrap().len(), 3);
        assert_eq!(m.get("b").unwrap().as_str(), Some("it's"));
        assert_eq!(m.get("c").unwrap().as_str(), Some("tab\tqé"));
    }

    #[test]
    fn multi_line_flow_and_block_scalars() {
        let v = parse_yaml("a: [1,\n  2,\n  3]\nb: |\n  line1\n  line2\nc: >-\n  folded\n  text\nd: plain\n  continued\n").unwrap();
        let m = obj(&v);
        assert_eq!(m.get("a").unwrap().as_array().unwrap().len(), 3);
        assert_eq!(m.get("b").unwrap().as_str(), Some("line1\nline2\n"));
        assert_eq!(m.get("c").unwrap().as_str(), Some("folded text"));
        assert_eq!(m.get("d").unwrap().as_str(), Some("plain continued"));
    }

    #[test]
    fn sequences() {
        let v = parse_yaml("- a\n- b: 1\n  c: 2\n-\n  - nested\n").unwrap();
        let a = v.as_array().unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].as_str(), Some("a"));
        assert_eq!(obj(&a[1]).keys(), vec!["b", "c"]);
        assert_eq!(a[2].as_array().unwrap()[0].as_str(), Some("nested"));
    }

    #[test]
    fn errors() {
        assert_eq!(parse_yaml("a: [1, 2").unwrap_err().message, "YAML: unexpected end of the stream within a flow collection");
        assert_eq!(parse_yaml("\ta: 1").unwrap_err().message, "YAML: tabs are not allowed for indentation");
        // A more indented line continues a plain scalar, as in Pele (js-yaml would refuse it).
        assert_eq!(parse_yaml("a: 1\n  b: 2").unwrap().get("a").unwrap().as_str(), Some("1 b: 2"));
        assert_eq!(parse_yaml("a:\n    b: 1\n  c: 2").unwrap_err().message, "YAML: bad indentation of a mapping entry");
        assert_eq!(parse_yaml("a: \"\\q\"").unwrap_err().message, "YAML: unknown escape sequence");
        // A plain scalar in a flow collection may hold spaces, so this is one entry.
        assert_eq!(parse_yaml("a: [1 2]").unwrap().get("a").unwrap().as_array().unwrap()[0].as_str(), Some("1 2"));
        assert_eq!(parse_yaml("a: \"\\xZZ\"").unwrap_err().message, "Invalid code point NaN");
        let deep = "a: ".to_string() + &"[".repeat(70) + &"]".repeat(70);
        assert_eq!(parse_yaml(&deep).unwrap_err().message, "YAML: nesting is too deep");
    }

    #[test]
    fn unicode_keys_and_values() {
        let v = parse_yaml("标题: 中文 # comment\n'é': \"\\ud83d\\ude00\"\n").unwrap();
        let m = obj(&v);
        assert_eq!(m.get("标题").unwrap().as_str(), Some("中文"));
        assert_eq!(m.get("é").unwrap().as_str(), Some("😀"));
    }
}
