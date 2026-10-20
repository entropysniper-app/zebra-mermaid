//! Tokenizer for the diagram types Mermaid defines with Langium grammars. Ported from Pele
//! `src/diagrams/common/tokens.ts`.
//!
//! Token types are tried in order and the first match wins, as in Chevrotain. Pele writes
//! each token as a sticky regular expression; here each is a matcher function that follows
//! the expression's JavaScript semantics, written out by hand.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::chars::{char_at, is_space};
use crate::util::js;

/// Answers whether a token matches at `at` of `src`, with the end of the match.
pub type MatchFn = fn(&str, usize) -> Option<usize>;

/// A matcher that may remember what it scanned, made once per text.
pub type Scanner = Box<dyn FnMut(&str, usize) -> Option<usize>>;

#[derive(Clone, Copy)]
pub enum Pattern {
    /// A literal string.
    Literal(&'static str),
    /// A function answering as the original sticky expression does.
    Fn(MatchFn),
    /// A factory for a stateful matcher, for a pattern that would rescan the text.
    Scanner(fn() -> Scanner),
}

#[derive(Clone, Copy)]
pub struct TokenType {
    pub name: &'static str,
    pub pattern: Pattern,
    pub hidden: bool,
    /// Start of a construct that needs a closer. When this matches but the whole pattern does
    /// not, no closer exists in the rest of the text, so the pattern is not tried again.
    pub opener: Option<MatchFn>,
    /// Every ASCII character a match can start with, when that is known. The type is then
    /// tried only at those characters and at characters outside ASCII.
    pub first: Option<&'static str>,
    /// Chevrotain's LONGER_ALT, as indices into the token table. When this type matches, these
    /// are tried at the same place, and the first that matches more text becomes the token.
    pub longer: &'static [usize],
}

impl TokenType {
    pub const fn literal(name: &'static str, text: &'static str) -> Self {
        TokenType { name, pattern: Pattern::Literal(text), hidden: false, opener: None, first: None, longer: &[] }
    }

    pub const fn func(name: &'static str, f: MatchFn, first: Option<&'static str>) -> Self {
        TokenType { name, pattern: Pattern::Fn(f), hidden: false, opener: None, first, longer: &[] }
    }

    pub const fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub const fn with_opener(mut self, opener: MatchFn) -> Self {
        self.opener = Some(opener);
        self
    }

    pub const fn with_longer(mut self, longer: &'static [usize]) -> Self {
        self.longer = longer;
        self
    }
}

/// Kind of the token past the last one.
pub const EOF: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct Tokens<'a> {
    pub kinds: Vec<usize>,
    pub starts: Vec<usize>,
    pub ends: Vec<usize>,
    pub src: &'a str,
}

fn can_start(t: &TokenType, c: u8) -> bool {
    match t.pattern {
        Pattern::Literal(s) => s.as_bytes().first() == Some(&c),
        _ => match t.first {
            None => true,
            Some(first) => first.as_bytes().contains(&c),
        },
    }
}

/// For each ASCII character, the token types that can start with it, in matching order; then
/// every type, for characters outside ASCII. Pele keeps this per table in a WeakMap; here it is
/// kept by the table's address and checked against a fingerprint of its entries, so a table
/// built at run time that reuses a dropped table's address is not mistaken for it.
pub(super) struct Starting {
    fingerprint: u64,
    ascii: Vec<Vec<usize>>,
    every: Vec<usize>,
}

impl Starting {
    pub(super) fn new(types: &[TokenType]) -> Self {
        Self {
            fingerprint: 0,
            ascii: (0u8..128).map(|c| (0..types.len()).filter(|&k| can_start(&types[k], c)).collect()).collect(),
            every: (0..types.len()).collect(),
        }
    }
}

pub(super) trait TokenSink<'a>: Sized {
    fn new(src: &'a str, types: usize) -> Self;
    fn push(&mut self, kind: usize, start: usize, end: usize);
}
impl<'a> TokenSink<'a> for Tokens<'a> {
    fn new(src: &'a str, _: usize) -> Self {
        Self { kinds: Vec::new(), starts: Vec::new(), ends: Vec::new(), src }
    }
    fn push(&mut self, kind: usize, start: usize, end: usize) {
        self.kinds.push(kind);
        self.starts.push(start);
        self.ends.push(end);
    }
}

fn fingerprint(types: &[TokenType]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |v: usize| {
        h ^= v as u64;
        h = h.wrapping_mul(0x100000001b3);
    };
    for t in types {
        mix(t.name.as_ptr() as usize);
        mix(t.name.len());
        mix(t.first.map_or(0, |f| f.as_ptr() as usize));
        mix(match t.pattern {
            Pattern::Literal(s) => s.as_ptr() as usize,
            Pattern::Fn(f) => f as usize,
            Pattern::Scanner(f) => f as usize,
        });
        mix(t.hidden as usize);
    }
    h
}

fn starting_for(types: &[TokenType]) -> Arc<Starting> {
    static CACHE: OnceLock<Mutex<HashMap<(usize, usize), Arc<Starting>>>> = OnceLock::new();
    let key = (types.as_ptr() as usize, types.len());
    let print = fingerprint(types);
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(found) = cache.lock().ok().and_then(|c| c.get(&key).filter(|s| s.fingerprint == print).cloned()) {
        return found;
    }
    let built = Arc::new(Starting { fingerprint: print, ..Starting::new(types) });
    if let Ok(mut c) = cache.lock() {
        if c.len() < 512 {
            c.insert(key, built.clone());
        }
    }
    built
}

/// Splits `src` into tokens. Hidden tokens are matched and dropped. The list ends with an
/// `EOF` token at the end of the text.
pub fn tokenize<'a>(src: &'a str, types: &[TokenType], diagram: DiagramType) -> Result<Tokens<'a>, Error> {
    tokenize_into(src, types, &starting_for(types), diagram)
}

pub(super) fn tokenize_into<'a, T: TokenSink<'a>>(
    src: &'a str,
    types: &[TokenType],
    table: &Starting,
    diagram: DiagramType,
) -> Result<T, Error> {
    let mut out = T::new(src, types.len());
    let mut dead = vec![false; types.len()];
    let mut scanners: Vec<Option<Scanner>> = (0..types.len()).map(|_| None).collect();
    let starting = &table.ascii;
    let every = &table.every;
    let b = src.as_bytes();
    let n = b.len();

    let mut matcher = |k: usize, p: usize, dead: &mut Vec<bool>| -> Option<usize> {
        if dead[k] {
            return None;
        }
        let t = &types[k];
        let end = match t.pattern {
            Pattern::Literal(s) => src[p..].starts_with(s).then(|| p + s.len()),
            Pattern::Fn(f) => f(src, p),
            Pattern::Scanner(make) => {
                let scanner = scanners[k].get_or_insert_with(make);
                scanner(src, p)
            }
        };
        if end.is_none() {
            if let Some(opener) = t.opener {
                if opener(src, p).is_some() {
                    dead[k] = true;
                }
            }
        }
        end
    };

    let mut p = 0;
    'scan: while p < n {
        let c = b[p];
        let list = if c < 128 { &starting[c as usize] } else { every };
        for &k in list {
            let Some(mut end) = matcher(k, p, &mut dead) else { continue };
            if end <= p {
                continue;
            }
            let mut kind = k;
            for &alt in types[k].longer {
                if let Some(alt_end) = matcher(alt, p, &mut dead) {
                    if alt_end > end {
                        kind = alt;
                        end = alt_end;
                        break;
                    }
                }
            }
            if !types[kind].hidden {
                out.push(kind, p, end);
            }
            p = end;
            continue 'scan;
        }
        return Err(syntax_error(diagram, src, p, "", true));
    }
    out.push(EOF, n, n);
    Ok(out)
}

// ----- Shared pieces of patterns -----

/// `[\t ]*`
pub fn blanks(src: &str, mut p: usize) -> usize {
    let b = src.as_bytes();
    while p < b.len() && (b[p] == b' ' || b[p] == b'\t') {
        p += 1;
    }
    p
}

/// `\s*` (JavaScript white space).
pub fn spaces(src: &str, mut p: usize) -> usize {
    while let Some(c) = char_at(src, p) {
        if !is_space(c) {
            break;
        }
        p += c.len_utf8();
    }
    p
}

/// `[^\n\r]*`
pub fn line_rest(src: &str, p: usize) -> usize {
    src[p..].find(['\n', '\r']).map_or(src.len(), |k| p + k)
}

/// `[^\n\r]*?(?=%%)|[^\n\r]*`: up to the first `%%` on the line, or to its end.
pub fn line_rest_before_comment(src: &str, p: usize) -> usize {
    let end = line_rest(src, p);
    src[p..end].find("%%").map_or(end, |k| p + k)
}

/// `(?:(?=%%)|(?!\S))`: a comment, white space or the end follows.
pub fn keyword_end(src: &str, p: usize) -> bool {
    src[p..].starts_with("%%") || char_at(src, p).is_none_or(is_space)
}

/// `(?:\r?\n|(?!\S))` at `p`: the end of what it matches, or `None`.
fn line_end_or_space(src: &str, p: usize) -> Option<usize> {
    if src[p..].starts_with("\r\n") {
        return Some(p + 2);
    }
    if src[p..].starts_with('\n') {
        return Some(p + 1);
    }
    if char_at(src, p).is_none_or(is_space) {
        Some(p)
    } else {
        None
    }
}

/// `\r?\n` at `p`.
pub fn newline_at(src: &str, p: usize) -> Option<usize> {
    if src[p..].starts_with("\r\n") {
        Some(p + 2)
    } else if src[p..].starts_with('\n') {
        Some(p + 1)
    } else {
        None
    }
}

// ----- Shared tokens -----

/// `/[\t ]*accDescr(?:[\t ]*:([^\n\r]*?(?=%%)|[^\n\r]*)|\s*{([^}]*)})/y`
pub fn match_acc_descr(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    if !src[q..].starts_with("accDescr") {
        return None;
    }
    let q = q + 8;
    let c = blanks(src, q);
    if src[c..].starts_with(':') {
        return Some(line_rest_before_comment(src, c + 1));
    }
    let o = spaces(src, q);
    if src[o..].starts_with('{') {
        return src[o + 1..].find('}').map(|k| o + 1 + k + 1);
    }
    None
}

/// `/[\t ]*accTitle[\t ]*:(?:[^\n\r]*?(?=%%)|[^\n\r]*)/y`
pub fn match_acc_title(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    if !src[q..].starts_with("accTitle") {
        return None;
    }
    let c = blanks(src, q + 8);
    if !src[c..].starts_with(':') {
        return None;
    }
    Some(line_rest_before_comment(src, c + 1))
}

/// `/[\t ]*title(?:[\t ][^\n\r]*?(?=%%)|[\t ][^\n\r]*|)/y`
pub fn match_title(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    if !src[q..].starts_with("title") {
        return None;
    }
    let q = q + 5;
    match src.as_bytes().get(q) {
        Some(b' ' | b'\t') => Some(line_rest_before_comment(src, q + 1)),
        _ => Some(q),
    }
}

/// `/"([^"\\]|\\.)*"|'([^'\\]|\\.)*'/y`
pub fn match_string(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let quote = *b.get(p)?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let mut i = p + 1;
    loop {
        let c = char_at(src, i)?;
        if c as u32 == quote as u32 {
            return Some(i + 1);
        }
        if c == '\\' {
            let next = char_at(src, i + 1)?;
            if matches!(next, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                return None;
            }
            i += 1 + next.len_utf8();
        } else {
            i += c.len_utf8();
        }
    }
}

pub fn match_newline(src: &str, p: usize) -> Option<usize> {
    newline_at(src, p)
}

pub fn match_whitespace(src: &str, p: usize) -> Option<usize> {
    let e = blanks(src, p);
    (e > p).then_some(e)
}

/// `/---[\t ]*\r?\n/y`
pub fn yaml_opener(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("---") {
        return None;
    }
    newline_at(src, blanks(src, p + 3))
}

/// `/---[\t ]*\r?\n(?:[\S\s]*?\r?\n)?---(?:\r?\n|(?!\S))/y`
pub fn match_yaml(src: &str, p: usize) -> Option<usize> {
    let s = yaml_opener(src, p)?;
    // The optional group is tried first: the nearest fence after a line break wins.
    let mut from = s;
    while let Some(k) = src[from..].find('\n') {
        let f = from + k + 1;
        if src[f..].starts_with("---") {
            if let Some(e) = line_end_or_space(src, f + 3) {
                return Some(e);
            }
        }
        from = f;
    }
    if src[s..].starts_with("---") {
        return line_end_or_space(src, s + 3);
    }
    None
}

/// `/[\t ]*%%{/y`
pub fn directive_opener(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    src[q..].starts_with("%%{").then_some(q + 3)
}

/// `/[\t ]*%%{[\S\s]*?}%%(?:\r?\n|(?!\S))/y`
pub fn match_directive(src: &str, p: usize) -> Option<usize> {
    let mut from = directive_opener(src, p)?;
    while let Some(k) = src[from..].find("}%%") {
        let at = from + k;
        if let Some(e) = line_end_or_space(src, at + 3) {
            return Some(e);
        }
        from = at + 1;
    }
    None
}

/// `/[\t ]*%%[^\n\r]*/y`
pub fn match_comment(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    src[q..].starts_with("%%").then(|| line_rest(src, q + 2))
}

pub const ACC_DESCR: TokenType = TokenType::func("ACC_DESCR", match_acc_descr, Some("\t a"));
pub const ACC_TITLE: TokenType = TokenType::func("ACC_TITLE", match_acc_title, Some("\t a"));
pub const TITLE: TokenType = TokenType::func("TITLE", match_title, Some("\t t"));
pub const STRING: TokenType = TokenType::func("STRING", match_string, Some("\"'"));
pub const NEWLINE: TokenType = TokenType::func("NEWLINE", match_newline, Some("\r\n"));
pub const WHITESPACE: TokenType = TokenType::func("WHITESPACE", match_whitespace, Some("\t ")).hidden();
pub const YAML: TokenType = TokenType::func("YAML", match_yaml, Some("-")).hidden().with_opener(yaml_opener);
pub const DIRECTIVE: TokenType = TokenType::func("DIRECTIVE", match_directive, Some("\t %")).hidden().with_opener(directive_opener);
pub const SINGLE_LINE_COMMENT: TokenType = TokenType::func("SINGLE_LINE_COMMENT", match_comment, Some("\t %")).hidden();

/// Matches `word` followed by a comment, white space or the end, as Mermaid ends keywords.
pub fn keyword_at(src: &str, p: usize, word: &str) -> Option<usize> {
    (src[p..].starts_with(word) && keyword_end(src, p + word.len())).then_some(p + word.len())
}

/// A keyword token type. Its matcher must be a function that calls [`keyword_at`] with the
/// word, as Rust cannot make a function from a string at compile time.
pub const fn keyword(name: &'static str, f: MatchFn, first: &'static str) -> TokenType {
    TokenType::func(name, f, Some(first))
}

// ----- Values of tokens -----

/// `text.trim().replace(/[\t ]{2,}/gm, ' ')`
pub fn single(text: &str) -> String {
    collapse_blanks(js::trim(text))
}

/// `.replace(/[\t ]{2,}/g, ' ')`
pub fn collapse_blanks(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b' ' || b[i] == b'\t' {
            let mut j = i;
            while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            if j - i >= 2 {
                out.push_str(&text[last..i]);
                out.push(' ');
                last = j;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out.push_str(&text[last..]);
    out
}

/// The value of a TITLE token: `/title([\t ][^\n\r]*|)/` group 1, made single-spaced.
pub fn title_value(text: &str) -> String {
    let at = text.find("title").map_or(text.len(), |k| k + 5);
    let rest = &text[at..];
    let group = match rest.as_bytes().first() {
        Some(b' ' | b'\t') => &rest[..line_rest(rest, 0)],
        _ => "",
    };
    single(group)
}

/// The value of an ACC_TITLE token: `/accTitle[\t ]*:([^\n\r]*)/` group 1.
pub fn acc_title_value(text: &str) -> String {
    let at = text.find("accTitle").map_or(0, |k| k + 8);
    let c = blanks(text, at);
    let start = if text[c..].starts_with(':') { c + 1 } else { c };
    single(&text[start..line_rest(text, start)])
}

/// `/^\s*/gm` replaced with nothing: white space that starts a line goes, line breaks with it.
pub fn strip_line_starts(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut p = 0;
    let mut line_start = true;
    while p < text.len() {
        if line_start {
            let q = spaces(text, p);
            if q > p {
                p = q;
                // A run that ends after a line break leaves the next character at a line start,
                // but that character is not white space, so nothing more goes there.
                line_start = false;
                continue;
            }
        }
        let c = char_at(text, p).expect("boundary");
        out.push(c);
        line_start = matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}');
        p += c.len_utf8();
    }
    out
}

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// `/\s+$/gm` replaced with nothing: white space that ends a line goes. A run that spans line
/// breaks loses everything up to its last line break, as the expression backtracks to there.
pub fn strip_line_ends(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut a = 0;
    while a < text.len() {
        let c = char_at(text, a).expect("boundary");
        if !is_space(c) {
            out.push(c);
            a += c.len_utf8();
            continue;
        }
        let b = spaces(text, a);
        // The largest e in (a, b] where `$` holds: the end of the text, or a line terminator.
        let mut e = None;
        if b == text.len() {
            e = Some(b);
        } else {
            let mut q = a;
            while q < b {
                let ch = char_at(text, q).expect("boundary");
                if q > a && is_line_terminator(ch) {
                    e = Some(q);
                }
                q += ch.len_utf8();
            }
        }
        match e {
            Some(e) => a = e,
            None => {
                out.push(c);
                a += c.len_utf8();
            }
        }
    }
    out
}

/// `.replace(/[\n\r]{2,}/g, '\n')`
pub fn collapse_breaks(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\n' || b[i] == b'\r' {
            let mut j = i;
            while j < b.len() && (b[j] == b'\n' || b[j] == b'\r') {
                j += 1;
            }
            if j - i >= 2 {
                out.push_str(&text[last..i]);
                out.push('\n');
                last = j;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out.push_str(&text[last..]);
    out
}

/// The value of an ACC_DESCR token: `/accDescr(?:[\t ]*:([^\n\r]*)|\s*{([^}]*)})/`.
pub fn acc_descr_value(text: &str) -> String {
    let at = text.find("accDescr").map_or(0, |k| k + 8);
    let c = blanks(text, at);
    if text[c..].starts_with(':') {
        return single(&text[c + 1..line_rest(text, c + 1)]);
    }
    let o = spaces(text, at);
    let body_start = if text[o..].starts_with('{') { o + 1 } else { o };
    let body_end = text[body_start..].find('}').map_or(text.len(), |k| body_start + k);
    let body = &text[body_start..body_end];
    collapse_breaks(&collapse_blanks(&strip_line_ends(&strip_line_starts(body))))
}

/// The value of a STRING token: the text between its quotes with backslash escapes read.
pub fn string_value(text: &str) -> String {
    let inner = js::slice16(text, 1, js::len16(text).saturating_sub(1));
    if !text.contains('\\') {
        return inner.to_string();
    }
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('b') => out.push('\u{8}'),
                Some('f') => out.push('\u{c}'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('v') => out.push('\u{b}'),
                Some('0') => out.push('\0'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Reads tokens for a hand-written parser, with Chevrotain-style messages on failure.
pub struct Reader<'a, 't> {
    pub i: usize,
    pub tokens: Tokens<'a>,
    pub types: &'t [TokenType],
    pub diagram: DiagramType,
}

impl<'a, 't> Reader<'a, 't> {
    pub fn new(tokens: Tokens<'a>, types: &'t [TokenType], diagram: DiagramType) -> Self {
        Reader { i: 0, tokens, types, diagram }
    }

    #[inline]
    pub fn kind(&self) -> usize {
        self.tokens.kinds[self.i]
    }

    #[inline]
    pub fn kind_at(&self, index: usize) -> usize {
        self.tokens.kinds.get(index).copied().unwrap_or(EOF)
    }

    pub fn text_at(&self, index: usize) -> &'a str {
        &self.tokens.src[self.tokens.starts[index]..self.tokens.ends[index]]
    }

    pub fn text(&self) -> &'a str {
        self.text_at(self.i)
    }

    pub fn take(&mut self) -> &'a str {
        let t = self.text_at(self.i);
        self.i += 1;
        t
    }

    pub fn accept(&mut self, kind: usize) -> bool {
        if self.kind() != kind {
            return false;
        }
        self.i += 1;
        true
    }

    pub fn expect(&mut self, kind: usize) -> Result<&'a str, Error> {
        if self.kind() != kind {
            return Err(self.fail(&format!("token of type '{}'", self.types[kind].name)));
        }
        Ok(self.take())
    }

    pub fn fail(&self, expected: &str) -> Error {
        let found = if self.kind() == EOF { "end of input".to_string() } else { format!("`{}`", self.text()) };
        syntax_error(
            self.diagram,
            self.tokens.src,
            self.tokens.starts[self.i],
            &format!("Expecting {} but found {}.", expected, found),
            false,
        )
    }
}

/// A keyword token type: the word, ended as Mermaid ends keywords.
#[macro_export]
#[doc(hidden)]
macro_rules! keyword_token {
    ($word:literal) => {{
        fn matcher(src: &str, p: usize) -> Option<usize> {
            $crate::diagrams::common::tokens::keyword_at(src, p, $word)
        }
        $crate::diagrams::common::tokens::TokenType::func($word, matcher, Some(&$word[..1]))
    }};
    ($name:literal, $word:literal) => {{
        fn matcher(src: &str, p: usize) -> Option<usize> {
            $crate::diagrams::common::tokens::keyword_at(src, p, $word)
        }
        $crate::diagrams::common::tokens::TokenType::func($name, matcher, Some(&$word[..1]))
    }};
}

#[cfg(test)]
#[path = "scanner_reference.rs"]
mod state_contract;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_patterns() {
        assert_eq!(match_title("title Hello %% c", 0), Some(12));
        assert_eq!(match_title("titleX", 0), Some(5));
        assert_eq!(match_acc_title("accTitle : x y", 0), Some(14));
        assert_eq!(match_acc_descr("accDescr {\n a\n}", 0), Some(15));
        assert_eq!(match_acc_descr("accDescr {", 0), None);
        assert_eq!(match_string("\"a\\\"b\" x", 0), Some(6));
        assert_eq!(match_string("\"a\\\nb\"", 0), None);
        assert_eq!(match_string("'中文'", 0), Some(8));
        assert_eq!(match_yaml("---\na: 1\n---\nrest", 0), Some(13));
        assert_eq!(match_yaml("---\n---\nfoo\n---\n", 0), Some(16));
        assert_eq!(match_directive("%%{init: {}}%%x}%% y", 0), Some(18));
        assert_eq!(match_comment("  %% hi\nx", 0), Some(7));
        assert!(keyword_end("pie", 3));
        assert!(keyword_end("pie%%", 3));
        assert!(!keyword_end("pies", 3));
    }

    #[test]
    fn values() {
        assert_eq!(title_value("title   Big    Title  "), "Big Title");
        assert_eq!(title_value("title"), "");
        assert_eq!(acc_title_value("accTitle: My  title"), "My title");
        assert_eq!(acc_descr_value("accDescr: one  line"), "one line");
        assert_eq!(acc_descr_value("accDescr {\n  first line\n\n  second   line  \n}"), "first line\nsecond line");
        assert_eq!(string_value("\"a\\nb\\\"c\""), "a\nb\"c");
        assert_eq!(string_value("'plain'"), "plain");
    }

    #[test]
    fn line_regexes() {
        assert_eq!(strip_line_starts("  a\n  \n  b"), "a\nb");
        assert_eq!(strip_line_ends("x  \n\n y"), "x\n y");
        assert_eq!(strip_line_ends("x \n\n\ny"), "x\ny");
        assert_eq!(strip_line_ends("a  "), "a");
        assert_eq!(collapse_breaks("a\n\n\nb\r\nc"), "a\nb\nc");
    }
}
