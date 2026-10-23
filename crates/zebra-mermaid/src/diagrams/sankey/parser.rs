//! Sankey lexer and parser. Ported from Pele `src/diagrams/sankey/lexer.ts` and `parser.ts`,
//! which follow Mermaid's Jison grammar for RFC 4180 CSV rows.

use std::collections::HashMap;

use super::{SankeyLink, SankeyModel, SankeyNode};
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::chars::is_word;
use crate::util::js;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum T {
    End,
    // Not a Jison token: the reference lexer would return the same empty token forever from here.
    Stuck,
    Sankey,
    Newline,
    Comma,
    Dquote,
    NonEscapedText,
    EscapedText,
    Eof,
}

impl T {
    fn name(self) -> &'static str {
        match self {
            T::End => "$end",
            T::Stuck => "STUCK",
            T::Sankey => "SANKEY",
            T::Newline => "NEWLINE",
            T::Comma => "COMMA",
            T::Dquote => "DQUOTE",
            T::NonEscapedText => "NON_ESCAPED_TEXT",
            T::EscapedText => "ESCAPED_TEXT",
            T::Eof => "EOF",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum S {
    Initial,
    Csv,
    EscapedText,
}

struct Token {
    kind: T,
    start: usize,
    end: usize,
}

/// RFC 4180 extended to Unicode, excluding controls and invalid XML characters.
#[inline]
fn is_text_data(c: char) -> bool {
    let n = c as u32;
    ((32..=126).contains(&n) && n != 34 && n != 44)
        || (0xa0..=0xd7ff).contains(&n)
        || (0xe000..=0xfffd).contains(&n)
        || (0x10000..=0x10ffff).contains(&n)
}

/// `src.slice(p, p + word.length).toLowerCase() === word && !isWord(src.charCodeAt(p + word.length))`,
/// with the end of the keyword. The slice is taken in UTF-16 code units and lower-cased with
/// Unicode's mappings, under which the Kelvin sign is a `k`.
fn keyword(src: &str, p: usize, word: &str) -> Option<usize> {
    let want = word.len();
    let mut units = 0;
    let mut end = p;
    for c in src[p..].chars() {
        if units >= want {
            break;
        }
        units += c.len_utf16();
        end += c.len_utf8();
    }
    // A shorter slice, or one that cuts a surrogate pair in two, cannot lower-case to `word`.
    if units != want || src[p..end].to_lowercase() != word {
        return None;
    }
    if src[end..].chars().next().is_some_and(is_word) {
        return None;
    }
    Some(end)
}

fn tokenize(src: &str) -> Vec<Token> {
    let b = src.as_bytes();
    let n = b.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<S> = Vec::new();
    let mut state = S::Initial;
    let mut p = 0;

    while p < n {
        let c = b[p];
        let start = p;
        let mut q = p;
        if state == S::EscapedText {
            if c == b'"' && b.get(p + 1) != Some(&b'"') {
                if let Some(s) = stack.pop() {
                    state = s;
                }
                tokens.push(Token { kind: T::Dquote, start: p, end: p + 1 });
                p += 1;
                continue;
            }
            while q < n {
                let d = src[q..].chars().next().unwrap();
                if is_text_data(d) || matches!(d, ',' | '\r' | '\n') {
                    q += d.len_utf8();
                } else if d == '"' && b.get(q + 1) == Some(&b'"') {
                    q += 2;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: T::EscapedText, start: p, end: q });
            p = q;
        } else {
            if state == S::Initial && (c | 32) == b's' {
                let end = keyword(src, p, "sankey-beta").or_else(|| keyword(src, p, "sankey"));
                if let Some(end) = end {
                    stack.push(state);
                    state = S::Csv;
                    tokens.push(Token { kind: T::Sankey, start: p, end });
                    p = end;
                    continue;
                }
            }
            if c == b'\n' || (c == b'\r' && b.get(p + 1) == Some(&b'\n')) {
                let end = if c == b'\n' { p + 1 } else { p + 2 };
                tokens.push(Token { kind: T::Newline, start: p, end });
                p = end;
                continue;
            }
            if c == b',' {
                tokens.push(Token { kind: T::Comma, start: p, end: p + 1 });
                p += 1;
                continue;
            }
            if c == b'"' {
                stack.push(state);
                state = S::EscapedText;
                tokens.push(Token { kind: T::Dquote, start: p, end: p + 1 });
                p += 1;
                continue;
            }
            while let Some(d) = src[q..].chars().next() {
                if !is_text_data(d) {
                    break;
                }
                q += d.len_utf8();
            }
            tokens.push(Token { kind: T::NonEscapedText, start: p, end: q });
            p = q;
        }
        // A character outside the CSV alphabet matches as empty text and is never consumed.
        if q == start {
            tokens.push(Token { kind: T::Stuck, start: p, end: p });
            return tokens;
        }
    }

    tokens.push(Token { kind: if state == S::EscapedText { T::EscapedText } else { T::Eof }, start: n, end: n });
    tokens.push(Token { kind: T::End, start: n, end: n });
    tokens
}

/// Blank as the first step of Mermaid's sankeyUtils sees it: white space other than line breaks.
fn is_blank(c: char) -> bool {
    let c = c as u32;
    if c == 10 || c == 13 {
        return false;
    }
    if c < 128 {
        return c == 32 || c == 9 || c == 11 || c == 12;
    }
    c == 0xa0
        || c == 0x1680
        || (0x2000..=0x200a).contains(&c)
        || c == 0x2028
        || c == 0x2029
        || c == 0x202f
        || c == 0x205f
        || c == 0x3000
        || c == 0xfeff
}

/// Mermaid's sankeyUtils: strip blanks at both ends of the text, then collapse runs of line
/// breaks (`/[\n\r]+/g`) so that no row is empty, then `trim()`. When `map` is given, it gets
/// the byte of `text` that each byte of the result, and its end, came from.
fn prepare(text: &str, map: Option<&mut Vec<usize>>) -> String {
    let mut start = 0;
    let mut end = text.len();
    while let Some(c) = text[start..end].chars().next().filter(|&c| is_blank(c)) {
        start += c.len_utf8();
    }
    while let Some(c) = text[start..end].chars().next_back().filter(|&c| is_blank(c)) {
        end -= c.len_utf8();
    }
    let b = text.as_bytes();
    let mut collapsed = String::with_capacity(end - start);
    let mut from: Vec<usize> = Vec::new();
    let mut i = start;
    let mut copied = start;
    while i < end {
        if b[i] == b'\n' || b[i] == b'\r' {
            collapsed.push_str(&text[copied..i]);
            if map.is_some() {
                from.extend(copied..i);
                from.push(i);
            }
            while i < end && (b[i] == b'\n' || b[i] == b'\r') {
                i += 1;
            }
            collapsed.push('\n');
            copied = i;
        } else {
            i += 1;
        }
    }
    collapsed.push_str(&text[copied..end]);
    let trimmed = js::trim(&collapsed);
    if let Some(map) = map {
        from.extend(copied..end);
        from.push(end);
        let lead = trimmed.as_ptr() as usize - collapsed.as_ptr() as usize;
        map.clear();
        map.extend_from_slice(&from[lead..=lead + trimmed.len()]);
    }
    trimmed.to_string()
}

/// Mermaid's sankeyUtils `prepareTextForParsing`. Its first step is a regex that rescans every
/// run of blanks; this does the same in one pass.
pub fn prepare_text_for_parsing(text: &str) -> String {
    prepare(text, None)
}

/// Prepares and parses the rows of `source` into `model`. A syntax error is located in the
/// prepared text, as Pele locates it; its byte offset is mapped back to `source`.
pub fn parse_sankey(source: &str, model: &mut SankeyModel) -> Result<(), Error> {
    let src = prepare_text_for_parsing(source);
    parse_rows(&src, model).map_err(|mut e| {
        if let Some(offset) = e.offset {
            let mut map = Vec::new();
            prepare(source, Some(&mut map));
            e.offset = map.get(offset).copied().or(e.offset);
        }
        e
    })
}

struct Parser<'a> {
    src: &'a str,
    tokens: Vec<Token>,
    i: usize,
}

impl<'a> Parser<'a> {
    fn kind(&self) -> T {
        self.tokens[self.i].kind
    }

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        // `TOKEN_NAMES[types[i - 1]]`, which is undefined before the first token.
        let got = if t == T::Stuck {
            if self.i == 0 {
                "undefined"
            } else {
                self.tokens[self.i - 1].kind.name()
            }
        } else {
            t.name()
        };
        syntax_error(DiagramType::Sankey, self.src, self.tokens[self.i].start, &format!("Expecting {}, got '{}'", expected, got), false)
    }

    fn expect(&mut self, kind: T, also: &str) -> Result<(), Error> {
        if self.kind() != kind {
            return Err(self.fail(&format!("'{}'{}", kind.name(), also)));
        }
        self.i += 1;
        Ok(())
    }

    fn text(&mut self) -> &'a str {
        let t = &self.tokens[self.i];
        self.i += 1;
        &self.src[t.start..t.end]
    }

    fn field(&mut self) -> Result<&'a str, Error> {
        if self.kind() == T::NonEscapedText {
            return Ok(self.text());
        }
        if self.kind() != T::Dquote {
            return Err(self.fail("'DQUOTE', 'NON_ESCAPED_TEXT'"));
        }
        self.i += 1;
        if self.kind() != T::EscapedText {
            return Err(self.fail("'ESCAPED_TEXT'"));
        }
        let text = self.text();
        self.expect(T::Dquote, "")?;
        Ok(text)
    }

    fn name(&mut self) -> Result<String, Error> {
        let start = self.tokens[self.i].start;
        let text = js::trim(self.field()?).replace("\"\"", "\"");
        if crate::text::entities::decode_entities(&text).chars().any(|c| matches!(c as u32, 0..=8|11|12|14..=31|0xfffe|0xffff)) {
            return Err(syntax_error(DiagramType::Sankey, self.src, start, "Invalid XML character in node name", false));
        }
        Ok(text)
    }
}

fn parse_rows(src: &str, model: &mut SankeyModel) -> Result<(), Error> {
    let mut p = Parser { src, tokens: tokenize(src), i: 0 };
    let mut lookup: HashMap<String, usize> = HashMap::new();
    let mut find_or_create = |model: &mut SankeyModel, id: String| -> usize {
        if let Some(&index) = lookup.get(&id) {
            return index;
        }
        let index = model.nodes.len();
        lookup.insert(id.clone(), index);
        model.nodes.push(SankeyNode { id, index });
        index
    };

    p.expect(T::Sankey, "")?;
    p.expect(T::Newline, "")?;
    loop {
        let from = p.name()?;
        p.expect(T::Comma, ", 'NEWLINE', 'EOF'")?;
        let to = p.name()?;
        p.expect(T::Comma, ", 'NEWLINE', 'EOF'")?;
        let amount = p.field()?;
        let source = find_or_create(model, from);
        let target = find_or_create(model, to);
        model.links.push(SankeyLink { source, target, value: js::parse_float(js::trim(amount)) });
        if p.kind() != T::Newline {
            break;
        }
        p.i += 1;
    }
    if p.kind() == T::Eof {
        p.i += 1;
    }
    if p.kind() != T::End {
        return Err(p.fail("'NEWLINE', 'EOF'"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepares_like_sankey_utils() {
        assert_eq!(prepare_text_for_parsing("  \t sankey\r\n\r\n a,b,1 \n\n"), "sankey\n a,b,1");
        assert_eq!(prepare_text_for_parsing("\u{a0}\n sankey"), "sankey");
        let mut map = Vec::new();
        let text = " sankey\n\n\nx,y,1\n";
        let prepared = prepare(text, Some(&mut map));
        assert_eq!(prepared, "sankey\nx,y,1");
        assert_eq!(map.len(), prepared.len() + 1);
        assert_eq!(map[0], 1);
        assert_eq!(map[6], 7);
        assert_eq!(&text[map[7]..map[7] + 1], "x");
    }

    #[test]
    fn keywords_fold_case_in_code_units() {
        assert_eq!(keyword("SANKEY-beta\n", 0, "sankey-beta"), Some(11));
        assert_eq!(keyword("sankeyx", 0, "sankey"), None);
        assert_eq!(keyword("san\u{212a}ey\n", 0, "sankey"), Some(8));
        assert_eq!(keyword("sank", 0, "sankey"), None);
    }

    #[test]
    fn parses_rows_and_quotes() {
        let mut model = SankeyModel::default();
        parse_sankey("sankey\n\"a, \"\"b\"\"\",c,1.5\nc,d,2\n", &mut model).unwrap();
        assert_eq!(model.nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), ["a, \"b\"", "c", "d"]);
        assert_eq!(model.links[0], SankeyLink { source: 0, target: 1, value: 1.5 });
    }

    #[test]
    fn error_offsets_point_into_the_source() {
        // Pele locates the error in the prepared text; the offset is mapped back to the source.
        let source = "sankey\n\n\n  a,b\n";
        let mut model = SankeyModel::default();
        let e = parse_sankey(source, &mut model).unwrap_err();
        assert_eq!(e.message, "Parse error on line 2:\n  a,b\n-----^\nExpecting 'COMMA', 'NEWLINE', 'EOF', got 'EOF'");
        assert_eq!(e.offset, Some(14));
        let source = "\t sankey\r\n\r\nx,y,1\n\n\u{e9},z,2\n";
        // Pele 1.9.0 accepts Unicode CSV names at the same original input.
        parse_sankey(source, &mut model).unwrap();
        assert_eq!(model.nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), ["x", "y", "é", "z"]);
        assert_eq!(model.links.iter().map(|l| l.value).collect::<Vec<_>>(), [1.0, 2.0]);
    }

    #[test]
    fn unicode_names_no_longer_leave_the_lexer_stuck() {
        for (source, names, values) in
            [("sankey\na,b,1\né,c,2\n", vec!["a", "b", "é", "c"], vec![1.0, 2.0]), ("sankey\na,\"bé\",1\n", vec!["a", "bé"], vec![1.0])]
        {
            let mut model = SankeyModel::default();
            parse_sankey(source, &mut model).unwrap();
            assert_eq!(model.nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), names);
            assert_eq!(model.links.iter().map(|l| l.value).collect::<Vec<_>>(), values);
        }
    }
    #[test]
    fn excluded_control_characters_still_report_a_stuck_token() {
        for source in ["sankey\na,b,1\n\u{7f},c,2\n", "sankey\na,\"b\u{80}\",1\n"] {
            let mut model = SankeyModel::default();
            assert!(parse_sankey(source, &mut model).is_err());
        }
    }
}
