//! A strict JSON reader with `JSON.parse` semantics, for `%%{init: ...}%%` directives.

use super::value::{ConfigMap, ConfigValue};

/// How deeply arrays and objects may nest. `JSON.parse` has no fixed limit, and Pele's merge
/// of the result recurses until the JavaScript stack runs out (thousands of levels). Here the
/// reader, the merge and dropping the value all recurse on the native stack, so a deeper
/// directive is refused as a limit error. Mermaid configuration nests a few levels at most.
pub const MAX_DEPTH: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    Invalid,
    TooDeep,
}

struct Parser<'a> {
    s: &'a [u8],
    src: &'a str,
    i: usize,
    max_depth: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Result<ConfigValue, JsonError> {
        if depth > self.max_depth {
            return Err(JsonError::TooDeep);
        }
        self.ws();
        let Some(&c) = self.s.get(self.i) else { return Err(JsonError::Invalid) };
        match c {
            b'{' => {
                self.i += 1;
                let mut map = ConfigMap::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(ConfigValue::Object(map));
                }
                loop {
                    self.ws();
                    if self.s.get(self.i) != Some(&b'"') {
                        return Err(JsonError::Invalid);
                    }
                    let key = self.string()?;
                    self.ws();
                    if self.s.get(self.i) != Some(&b':') {
                        return Err(JsonError::Invalid);
                    }
                    self.i += 1;
                    let v = self.value(depth + 1)?;
                    map.insert(key, v);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(ConfigValue::Object(map));
                        }
                        _ => return Err(JsonError::Invalid),
                    }
                }
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(ConfigValue::Array(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(ConfigValue::Array(items));
                        }
                        _ => return Err(JsonError::Invalid),
                    }
                }
            }
            b'"' => Ok(ConfigValue::String(self.string()?)),
            b't' => self.word("true", ConfigValue::Bool(true)),
            b'f' => self.word("false", ConfigValue::Bool(false)),
            b'n' => self.word("null", ConfigValue::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => Err(JsonError::Invalid),
        }
    }

    fn word(&mut self, w: &str, v: ConfigValue) -> Result<ConfigValue, JsonError> {
        if self.src[self.i..].starts_with(w) {
            self.i += w.len();
            Ok(v)
        } else {
            Err(JsonError::Invalid)
        }
    }

    fn number(&mut self) -> Result<ConfigValue, JsonError> {
        let start = self.i;
        if self.s[self.i] == b'-' {
            self.i += 1;
        }
        match self.s.get(self.i) {
            Some(b'0') => self.i += 1,
            Some(b'1'..=b'9') => {
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    self.i += 1;
                }
            }
            _ => return Err(JsonError::Invalid),
        }
        if self.s.get(self.i) == Some(&b'.') {
            self.i += 1;
            let f = self.i;
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
            if self.i == f {
                return Err(JsonError::Invalid);
            }
        }
        if matches!(self.s.get(self.i), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.s.get(self.i), Some(b'+' | b'-')) {
                self.i += 1;
            }
            let f = self.i;
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
            if self.i == f {
                return Err(JsonError::Invalid);
            }
        }
        self.src[start..self.i].parse::<f64>().map(ConfigValue::Number).map_err(|_| JsonError::Invalid)
    }

    fn hex4(&mut self) -> Result<u32, JsonError> {
        let h = self.src.get(self.i..self.i + 4).ok_or(JsonError::Invalid)?;
        if !h.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(JsonError::Invalid);
        }
        self.i += 4;
        u32::from_str_radix(h, 16).map_err(|_| JsonError::Invalid)
    }

    fn string(&mut self) -> Result<String, JsonError> {
        // At the opening quote.
        self.i += 1;
        let mut out = String::new();
        loop {
            let start = self.i;
            while self.i < self.s.len() && self.s[self.i] != b'"' && self.s[self.i] != b'\\' && self.s[self.i] >= 0x20 {
                self.i += 1;
            }
            out.push_str(&self.src[start..self.i]);
            match self.s.get(self.i) {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    let Some(&e) = self.s.get(self.i) else { return Err(JsonError::Invalid) };
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hi = self.hex4()?;
                            if (0xd800..0xdc00).contains(&hi) && self.src[self.i..].starts_with("\\u") {
                                let save = self.i;
                                self.i += 2;
                                let lo = self.hex4()?;
                                if (0xdc00..0xe000).contains(&lo) {
                                    let c = 0x10000 + ((hi - 0xd800) << 10) + (lo - 0xdc00);
                                    out.push(char::from_u32(c).unwrap_or('\u{fffd}'));
                                    continue;
                                }
                                self.i = save;
                            }
                            // A lone surrogate cannot be held in a Rust string.
                            out.push(char::from_u32(hi).unwrap_or('\u{fffd}'));
                        }
                        _ => return Err(JsonError::Invalid),
                    }
                }
                _ => return Err(JsonError::Invalid),
            }
        }
    }
}

/// The members of a top-level JSON object in the order written, repeated keys included, with
/// nesting limited to `max_depth`. Serde-style readers need the repeats to refuse them.
pub fn parse_json_members(text: &str, max_depth: usize) -> Result<Vec<(String, ConfigValue)>, JsonError> {
    let mut p = Parser { s: text.as_bytes(), src: text, i: 0, max_depth };
    p.ws();
    if p.s.get(p.i) != Some(&b'{') {
        return Err(JsonError::Invalid);
    }
    p.i += 1;
    let mut out = Vec::new();
    p.ws();
    if p.s.get(p.i) == Some(&b'}') {
        p.i += 1;
    } else {
        loop {
            p.ws();
            if p.s.get(p.i) != Some(&b'"') {
                return Err(JsonError::Invalid);
            }
            let key = p.string()?;
            p.ws();
            if p.s.get(p.i) != Some(&b':') {
                return Err(JsonError::Invalid);
            }
            p.i += 1;
            out.push((key, p.value(1)?));
            p.ws();
            match p.s.get(p.i) {
                Some(b',') => p.i += 1,
                Some(b'}') => {
                    p.i += 1;
                    break;
                }
                _ => return Err(JsonError::Invalid),
            }
        }
    }
    p.ws();
    if p.i != p.s.len() {
        return Err(JsonError::Invalid);
    }
    Ok(out)
}

/// A JSON value with nesting limited to `max_depth`.
pub fn parse_json_limited(text: &str, max_depth: usize) -> Result<ConfigValue, JsonError> {
    let mut p = Parser { s: text.as_bytes(), src: text, i: 0, max_depth };
    let v = p.value(0)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(JsonError::Invalid);
    }
    Ok(v)
}

/// `JSON.parse(text)`.
pub fn parse_json(text: &str) -> Result<ConfigValue, JsonError> {
    let mut p = Parser { s: text.as_bytes(), src: text, i: 0, max_depth: MAX_DEPTH };
    let v = p.value(0)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(JsonError::Invalid);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_json() {
        let v = parse_json(r#" {"a": [1, -2.5e1, true, null], "b": "xé😀\n", "a": 3} "#).unwrap();
        let m = v.as_object().unwrap();
        assert_eq!(m.get("a"), Some(&ConfigValue::Number(3.0)));
        assert_eq!(m.get("b").unwrap().as_str(), Some("xé😀\n"));
        assert_eq!(m.keys(), vec!["a", "b"]);
    }

    #[test]
    fn rejects_what_json_parse_rejects() {
        for bad in ["{'a':1}", "{a:1}", "[1,]", "01", "1.", "\"\t\"", "", "{\"a\":1}}", "NaN", "+1"] {
            assert!(parse_json(bad).is_err(), "{bad}");
        }
        let deep = "[".repeat(MAX_DEPTH + 2) + &"]".repeat(MAX_DEPTH + 2);
        assert_eq!(parse_json(&deep), Err(JsonError::TooDeep));
    }
}
