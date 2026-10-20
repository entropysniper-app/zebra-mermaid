//! Treemap parser. Ported from Pele `src/diagrams/treemap/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, newline_at, spaces, title_value, TokenType, ACC_DESCR, ACC_TITLE, EOF, TITLE,
};
use crate::error::Error;
use crate::util::js;

const T_ACC_DESCR: usize = 0;
const T_ACC_TITLE: usize = 1;
const T_TITLE: usize = 2;
const KEYWORD: usize = 3;
const CLASS_DEF: usize = 4;
const STYLE_SEPARATOR: usize = 5;
const SEPARATOR: usize = 6;
const COMMA: usize = 7;
const INDENTATION: usize = 8;
const ID: usize = 12;
const NUMBER: usize = 13;
const T_STRING: usize = 14;

/// `/treemap-beta|treemap/y`
fn match_keyword(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    if rest.starts_with("treemap-beta") {
        Some(p + 12)
    } else if rest.starts_with("treemap") {
        Some(p + 7)
    } else {
        None
    }
}

/// The parts of a class definition: the end of the name, and the style text when there is one.
struct ClassDefMatch {
    name: (usize, usize),
    style: Option<(usize, usize)>,
    end: usize,
}

/// `classDef\s+([a-zA-Z_][a-zA-Z0-9_]+)(?:\s+([^;\r\n]*))?(?:;)?` at `p`. White space may run
/// over line breaks, so the style text can come from a later line. Every part after the name
/// is optional and greedy, so the first reading that reaches the name is the match.
fn class_def_at(src: &str, p: usize) -> Option<ClassDefMatch> {
    if !src[p..].starts_with("classDef") {
        return None;
    }
    let b = src.as_bytes();
    let q = p + 8;
    let s = spaces(src, q);
    if s == q {
        return None;
    }
    if !b.get(s).is_some_and(|&c| c.is_ascii_alphabetic() || c == b'_') {
        return None;
    }
    let mut n = s + 1;
    while b.get(n).is_some_and(|&c| c.is_ascii_alphanumeric() || c == b'_') {
        n += 1;
    }
    if n - s < 2 {
        return None;
    }
    let mut end = n;
    let mut style = None;
    let t = spaces(src, n);
    if t > n {
        let mut u = t;
        while u < b.len() && !matches!(b[u], b';' | b'\r' | b'\n') {
            u += 1;
        }
        style = Some((t, u));
        end = u;
    }
    if b.get(end) == Some(&b';') {
        end += 1;
    }
    Some(ClassDefMatch { name: (s, n), style, end })
}

fn match_class_def(src: &str, p: usize) -> Option<usize> {
    class_def_at(src, p).map(|m| m.end)
}

/// `/:::/y`
fn match_style_separator(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with(":::").then_some(p + 3)
}

fn match_separator(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with(':').then_some(p + 1)
}

fn match_comma(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with(',').then_some(p + 1)
}

/// `/[ \t]{1,}/y`, also WS.
fn match_blanks(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut i = p;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    (i > p).then_some(i)
}

/// `/\%\%[^\n]*/y`
fn match_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("%%") {
        return None;
    }
    Some(src[p + 2..].find('\n').map_or(src.len(), |k| p + 2 + k))
}

/// `/\r?\n/y`
fn match_newline(src: &str, p: usize) -> Option<usize> {
    newline_at(src, p)
}

/// `/[a-zA-Z_][a-zA-Z0-9_]*/y`
fn match_id(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if !b.get(p).is_some_and(|&c| c.is_ascii_alphabetic() || c == b'_') {
        return None;
    }
    let mut i = p + 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    Some(i)
}

/// `/[0-9_\.\,]+/y`
fn match_number(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut i = p;
    while i < b.len() && (b[i].is_ascii_digit() || matches!(b[i], b'_' | b'.' | b',')) {
        i += 1;
    }
    (i > p).then_some(i)
}

/// `/"[^"]*"|'[^']*'/y`
fn match_string(src: &str, p: usize) -> Option<usize> {
    let quote = *src.as_bytes().get(p)?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    src[p + 1..].find(quote as char).map(|k| p + 1 + k + 1)
}

// Mermaid's treemap grammar has its own terminals. Every run of spaces or tabs is an
// INDENTATION token, since that comes before WS, and line breaks are hidden, so rows are told
// apart by their tokens alone and the indentation of a row is whatever whitespace precedes it.
pub static TREEMAP_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        ACC_DESCR,
        ACC_TITLE,
        TITLE,
        TokenType::func("TREEMAP_KEYWORD", match_keyword, None),
        TokenType::func("CLASS_DEF", match_class_def, None),
        TokenType::func("STYLE_SEPARATOR", match_style_separator, None),
        TokenType::func("SEPARATOR", match_separator, None),
        TokenType::func("COMMA", match_comma, None),
        TokenType::func("INDENTATION", match_blanks, None),
        TokenType::func("WS", match_blanks, None).hidden(),
        TokenType::func("ML_COMMENT", match_comment, None).hidden(),
        TokenType::func("NL", match_newline, None).hidden(),
        TokenType::func("ID2", match_id, None),
        TokenType::func("NUMBER2", match_number, None),
        TokenType::func("STRING2", match_string, None),
    ]
});

#[derive(Debug, Clone, PartialEq)]
pub enum ItemAst {
    Section { name: String, class_selector: Option<String> },
    Leaf { name: String, value: f64, class_selector: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RowAst {
    Row { indent: Option<usize>, item: ItemAst },
    ClassDef { indent: Option<usize>, class_name: String, style_text: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreemapAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub rows: Vec<RowAst>,
}

pub fn parse_treemap(src: &str) -> Result<TreemapAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&TREEMAP_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Treemap)?;
    let mut ast = TreemapAst::default();

    fn class_selector(r: &mut Reader) -> Result<Option<String>, Error> {
        if r.accept(STYLE_SEPARATOR) {
            Ok(Some(r.expect(ID)?.to_string()))
        } else {
            Ok(None)
        }
    }

    r.expect(KEYWORD)?;
    while r.kind() != EOF {
        let kind = r.kind();
        if kind == T_ACC_DESCR {
            ast.acc_descr = Some(acc_descr_value(r.take()));
            continue;
        }
        if kind == T_ACC_TITLE {
            ast.acc_title = Some(acc_title_value(r.take()));
            continue;
        }
        if kind == T_TITLE {
            ast.title = Some(title_value(r.take()));
            continue;
        }
        if kind != INDENTATION && kind != T_STRING && kind != CLASS_DEF {
            return Err(r.fail("a node, a class definition, or a title"));
        }

        // INDENTATION is spaces and tabs, so its length in bytes is its length in code units.
        let indent = if kind == INDENTATION { Some(r.take().len()) } else { None };
        if r.kind() == CLASS_DEF {
            let text = r.take();
            // RE_CLASS_DEF reads the token's own text, which the token pattern matched.
            let Some(m) = class_def_at(text, 0) else {
                return Err(r.fail("a class definition"));
            };
            let style_text = m.style.map(|(a, b)| &text[a..b]).filter(|t| !t.is_empty()).map(str::to_string);
            ast.rows.push(RowAst::ClassDef { indent, class_name: text[m.name.0..m.name.1].to_string(), style_text });
            continue;
        }

        let quoted = r.expect(T_STRING)?;
        let name = quoted[1..quoted.len() - 1].to_string();
        // A leaf is a name followed by a colon or comma, with or without whitespace before it.
        let after = if r.kind() == INDENTATION { r.i + 1 } else { r.i };
        let mut item = if r.kind_at(after) == SEPARATOR || r.kind_at(after) == COMMA {
            r.i = after + 1;
            r.accept(INDENTATION);
            let number = r.expect(NUMBER)?;
            ItemAst::Leaf { name, value: js::parse_float(&number.replace(',', "")), class_selector: None }
        } else {
            ItemAst::Section { name, class_selector: None }
        };
        let selector = class_selector(&mut r)?;
        match &mut item {
            ItemAst::Section { class_selector, .. } | ItemAst::Leaf { class_selector, .. } => *class_selector = selector,
        }
        ast.rows.push(RowAst::Row { indent, item });
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("treemap", &super::TREEMAP_TOKENS, crate::DiagramType::Treemap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_definitions() {
        let m = class_def_at("classDef big fill:#f00;rest", 0).unwrap();
        assert_eq!((m.name, m.style, m.end), ((9, 12), Some((13, 22)), 23));
        // White space runs over a line break into the style text.
        let m = class_def_at("classDef ab\n  x:1\n", 0).unwrap();
        assert_eq!(m.style, Some((14, 17)));
        assert!(class_def_at("classDef a", 0).is_none());
        assert!(class_def_at("classDefab", 0).is_none());
        let m = class_def_at("classDef ab;", 0).unwrap();
        assert_eq!((m.style, m.end), (None, 12));
    }

    #[test]
    fn rows() {
        let ast = parse_treemap("treemap-beta\n\"A\"\n  \"B\": 1,000.5:::big\n  \"C\" , 2\nclassDef big fill:red\n").unwrap();
        assert_eq!(ast.rows.len(), 4);
        assert_eq!(
            ast.rows[1],
            RowAst::Row { indent: Some(2), item: ItemAst::Leaf { name: "B".into(), value: 1000.5, class_selector: Some("big".into()) } }
        );
    }
}
