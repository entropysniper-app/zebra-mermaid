//! Radar parser. Ported from Pele `src/diagrams/radar/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, blanks, match_acc_descr, spaces, string_value, title_value, Pattern, Scanner, TokenType, ACC_DESCR,
    ACC_TITLE, DIRECTIVE, EOF, NEWLINE, SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::keyword_token;
use crate::util::chars::is_word;
use crate::util::js;

const RADAR_COLON: usize = 0;
const RADAR: usize = 1;
const SHOW_LEGEND: usize = 2;
const GRATICULE: usize = 3;
const CURVE: usize = 4;
const TICKS: usize = 5;
const AXIS: usize = 6;
const MAX: usize = 7;
const MIN: usize = 8;
const COLON: usize = 9;
const COMMA: usize = 10;
const OPEN: usize = 11;
const CLOSE: usize = 12;
const OPEN_BRACE: usize = 13;
const CLOSE_BRACE: usize = 14;
const GRATICULE_VALUE: usize = 15;
const BOOLEAN: usize = 16;
const T_ACC_DESCR: usize = 17;
const T_ACC_TITLE: usize = 18;
const T_TITLE: usize = 19;
const NUMBER: usize = 20;
const T_STRING: usize = 21;
const ID: usize = 22;
const T_NEWLINE: usize = 23;

fn digits(src: &[u8], at: usize) -> usize {
    let mut end = at;
    while end < src.len() && src[end].is_ascii_digit() {
        end += 1;
    }
    end
}

/// The NUMBER pattern, `(?:[0-9]+\.[0-9]+(?!\.))|(?:0|[1-9][0-9]*(?!\.))`, without its
/// backtracking. As a regular expression it reads to the end of a run of digits from every
/// position in the run, which is quadratic for a long run of zeros; this remembers the run, as
/// Pele's `matchNumber` does with its module state (reset for each text in `lexRadar`).
fn number_scanner() -> Scanner {
    let mut run_from = 0usize;
    let mut run_end = 0usize;
    Box::new(move |src: &str, at: usize| {
        let b = src.as_bytes();
        if at < run_from || at >= run_end {
            run_from = at;
            run_end = digits(b, at);
        }
        let whole = run_end;
        if whole == at {
            return None;
        }
        let dot = b.get(whole) == Some(&b'.');
        if dot {
            let fraction = digits(b, whole + 1);
            let length = fraction - whole - 1;
            if length > 0 && b.get(fraction) != Some(&b'.') {
                return Some(fraction);
            }
            if length > 1 {
                return Some(fraction - 1);
            }
        }
        if b[at] == b'0' {
            return Some(at + 1);
        }
        if !dot {
            return Some(whole);
        }
        if whole - 1 > at {
            Some(whole - 1)
        } else {
            None
        }
    })
}

/// In the other grammars an `accDescr {` that never closes ends the lexing. Here `accDescr` is
/// also a name and `{` a token, so each unclosed one would read to the end of the text again.
/// Pele's `matchAccDescr` finds the last closing brace once per text and refuses a braced
/// description that opens after it: `/[\t ]*accDescr\s*{/y` before the shared pattern.
fn acc_descr_scanner() -> Scanner {
    let mut last_brace: Option<Option<usize>> = None;
    Box::new(move |src: &str, at: usize| {
        let q = blanks(src, at);
        if src[q..].starts_with("accDescr") {
            let o = spaces(src, q + 8);
            if src[o..].starts_with('{') {
                let last = *last_brace.get_or_insert_with(|| src.rfind('}'));
                // `RE_BLOCK.lastIndex > lastBrace`, with -1 for no brace at all.
                if last.is_none_or(|l| o + 1 > l) {
                    return None;
                }
            }
        }
        match_acc_descr(src, at)
    })
}

/// `/[\w]([-\w]*\w)?/y`: a word character, then the longest run of word characters and dashes
/// that ends in a word character.
fn match_id(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if !b.get(p).is_some_and(|&c| is_word(c as char)) {
        return None;
    }
    let mut end = p + 1;
    let mut i = p + 1;
    while i < b.len() && (is_word(b[i] as char) || b[i] == b'-') {
        if b[i] != b'-' {
            end = i + 1;
        }
        i += 1;
    }
    Some(end)
}

/// `/circle|polygon/y`
fn match_graticule(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    if rest.starts_with("circle") {
        Some(p + 6)
    } else if rest.starts_with("polygon") {
        Some(p + 7)
    } else {
        None
    }
}

/// `/true|false/y`
fn match_boolean(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    if rest.starts_with("true") {
        Some(p + 4)
    } else if rest.starts_with("false") {
        Some(p + 5)
    } else {
        None
    }
}

/// A word that gives way to a longer name: Pele's `word()`, a literal with ID as its longer
/// alternative.
const fn word(text: &'static str) -> TokenType {
    TokenType::literal(text, text).with_longer(&[ID])
}

pub static RADAR_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        TokenType::literal("radar-beta:", "radar-beta:"),
        keyword_token!("radar-beta").with_longer(&[ID]),
        word("showLegend"),
        word("graticule"),
        word("curve"),
        word("ticks"),
        word("axis"),
        word("max"),
        word("min"),
        TokenType::literal(":", ":"),
        TokenType::literal(",", ","),
        TokenType::literal("[", "["),
        TokenType::literal("]", "]"),
        TokenType::literal("{", "{"),
        TokenType::literal("}", "}"),
        TokenType::func("GRATICULE", match_graticule, None),
        TokenType::func("BOOLEAN", match_boolean, None),
        TokenType { pattern: Pattern::Scanner(acc_descr_scanner), ..ACC_DESCR },
        ACC_TITLE,
        TITLE,
        TokenType { name: "NUMBER", pattern: Pattern::Scanner(number_scanner), hidden: false, opener: None, first: None, longer: &[] },
        STRING,
        TokenType::func("ID", match_id, None),
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
    ]
});

#[derive(Debug, Clone, PartialEq)]
pub struct AxisAst {
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryAst {
    /// The text of the axis reference; Langium leaves it unresolved and Mermaid reads only it.
    pub axis: Option<String>,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurveAst {
    pub name: String,
    pub label: Option<String>,
    pub entries: Vec<EntryAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    Bool(bool),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionAst {
    /// `showLegend`, `ticks`, `max`, `min` or `graticule`.
    pub name: String,
    pub value: OptionValue,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RadarAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub axes: Vec<AxisAst>,
    pub curves: Vec<CurveAst>,
    pub options: Vec<OptionAst>,
}

pub fn parse_radar(src: &str) -> Result<RadarAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&RADAR_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Radar)?;
    let mut ast = RadarAst::default();

    fn newlines(r: &mut Reader) {
        while r.kind() == T_NEWLINE {
            r.i += 1;
        }
    }

    fn end_of_line(r: &mut Reader) -> Result<(), Error> {
        if r.kind() == EOF {
            return Ok(());
        }
        if r.kind() != T_NEWLINE {
            return Err(r.fail("a line break or the end of input"));
        }
        newlines(r);
        Ok(())
    }

    fn label(r: &mut Reader) -> Result<Option<String>, Error> {
        if !r.accept(OPEN) {
            return Ok(None);
        }
        let text = string_value(r.expect(T_STRING)?);
        r.expect(CLOSE)?;
        Ok(Some(text))
    }

    fn entry(r: &mut Reader, detailed: bool) -> Result<EntryAst, Error> {
        if !detailed {
            return Ok(EntryAst { axis: None, value: js::number(r.expect(NUMBER)?) });
        }
        let axis = r.expect(ID)?.to_string();
        r.accept(COLON);
        Ok(EntryAst { axis: Some(axis), value: js::number(r.expect(NUMBER)?) })
    }

    fn curve(r: &mut Reader) -> Result<CurveAst, Error> {
        let name = r.expect(ID)?.to_string();
        let label = label(r)?;
        let mut node = CurveAst { name, label, entries: Vec::new() };
        r.expect(OPEN_BRACE)?;
        newlines(r);
        if r.kind() != NUMBER && r.kind() != ID {
            return Err(r.fail("a number or an axis name"));
        }
        let detailed = r.kind() == ID;
        node.entries.push(entry(r, detailed)?);
        while r.accept(COMMA) {
            newlines(r);
            node.entries.push(entry(r, detailed)?);
        }
        newlines(r);
        r.expect(CLOSE_BRACE)?;
        Ok(node)
    }

    fn axis(r: &mut Reader) -> Result<AxisAst, Error> {
        let name = r.expect(ID)?.to_string();
        let label = label(r)?;
        Ok(AxisAst { name, label })
    }

    fn option(r: &mut Reader) -> Result<OptionAst, Error> {
        let kind = r.kind();
        let name = r.take().to_string();
        let value = if kind == SHOW_LEGEND {
            OptionValue::Bool(r.expect(BOOLEAN)? == "true")
        } else if kind == GRATICULE {
            OptionValue::Text(r.expect(GRATICULE_VALUE)?.to_string())
        } else {
            OptionValue::Number(js::number(r.expect(NUMBER)?))
        };
        Ok(OptionAst { name, value })
    }

    fn is_option(r: &Reader) -> bool {
        matches!(r.kind(), SHOW_LEGEND | TICKS | MAX | MIN | GRATICULE)
    }

    newlines(&mut r);
    if !r.accept(RADAR_COLON) {
        r.expect(RADAR)?;
        r.accept(COLON);
    }
    newlines(&mut r);
    while r.kind() != EOF {
        match r.kind() {
            T_NEWLINE => r.i += 1,
            T_ACC_DESCR => {
                ast.acc_descr = Some(acc_descr_value(r.take()));
                end_of_line(&mut r)?;
            }
            T_ACC_TITLE => {
                ast.acc_title = Some(acc_title_value(r.take()));
                end_of_line(&mut r)?;
            }
            T_TITLE => {
                ast.title = Some(title_value(r.take()));
                end_of_line(&mut r)?;
            }
            AXIS => {
                r.i += 1;
                loop {
                    ast.axes.push(axis(&mut r)?);
                    if !r.accept(COMMA) {
                        break;
                    }
                }
            }
            CURVE => {
                r.i += 1;
                loop {
                    ast.curves.push(curve(&mut r)?);
                    if !r.accept(COMMA) {
                        break;
                    }
                }
            }
            _ => {
                if !is_option(&r) {
                    return Err(r.fail("an axis, a curve, an option, a title, or a line break"));
                }
                ast.options.push(option(&mut r)?);
                while r.accept(COMMA) {
                    if !is_option(&r) {
                        return Err(r.fail("an option"));
                    }
                    ast.options.push(option(&mut r)?);
                }
            }
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("radar", &super::RADAR_TOKENS, crate::DiagramType::Radar);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(number_scanner()("12.5x", 0), Some(4));
        assert_eq!(number_scanner()("0.5", 0), Some(3));
        assert_eq!(number_scanner()("1.2.3", 0), None);
        assert_eq!(number_scanner()("05", 0), Some(1));
        assert_eq!(number_scanner()("12.", 0), Some(1));
        assert_eq!(number_scanner()("1.", 0), None);
        assert_eq!(number_scanner()("0.", 0), Some(1));
        assert_eq!(number_scanner()("1.23.4", 0), Some(3));
        assert_eq!(number_scanner()("x", 0), None);
        // The remembered run answers for later positions in it.
        let mut s = number_scanner();
        assert_eq!(s("1234", 0), Some(4));
        assert_eq!(s("1234", 2), Some(4));
    }

    #[test]
    fn ids() {
        assert_eq!(match_id("a-b-", 0), Some(3));
        assert_eq!(match_id("a--", 0), Some(1));
        assert_eq!(match_id("_x9 y", 0), Some(3));
        assert_eq!(match_id("-a", 0), None);
        assert_eq!(match_id("é", 0), None);
    }

    #[test]
    fn unclosed_braced_descriptions() {
        let mut s = acc_descr_scanner();
        assert_eq!(s("accDescr { a } accDescr {", 0), Some(14));
        assert_eq!(s("accDescr { a } accDescr {", 15), None);
        let mut s = acc_descr_scanner();
        assert_eq!(s("accDescr: x\n", 0), Some(11));
    }

    #[test]
    fn parses() {
        let ast = parse_radar("radar-beta\n  axis A[\"Alpha\"], B\n  curve c{1, 2}\n  max 10, ticks 3\n").unwrap();
        assert_eq!(ast.axes.len(), 2);
        assert_eq!(ast.axes[0].label.as_deref(), Some("Alpha"));
        assert_eq!(ast.curves[0].entries.len(), 2);
        assert_eq!(ast.options.len(), 2);
    }
}
