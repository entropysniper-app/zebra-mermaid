//! TreeView parser. Ported from Pele `src/diagrams/treeview/parser.ts`.

use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::accdescr::acc_descr;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, newline_at, title_value, tokenize, Reader, TokenType, ACC_TITLE, EOF, TITLE,
};
use crate::error::Error;
use crate::keyword_token;
use crate::util::chars::{char_at, is_word};
use crate::util::js;

const KEYWORD: usize = 0;
const T_ACC_DESCR: usize = 1;
const T_ACC_TITLE: usize = 2;
const T_TITLE: usize = 3;
const CLASS_ANNOTATION: usize = 4;
const ICON_ANNOTATION: usize = 5;
const DESC_ANNOTATION: usize = 6;
const INDENTATION: usize = 7;
const QUOTED_NAME: usize = 8;
const BARE_NAME: usize = 12;

fn is_blank_byte(b: Option<&u8>) -> bool {
    matches!(b, Some(b' ' | b'\t'))
}

fn blanks_from(b: &[u8], mut i: usize) -> usize {
    while is_blank_byte(b.get(i)) {
        i += 1;
    }
    i
}

/// `[A-Za-z_]`
fn is_name_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Whether an annotation starts at `at`: `##`, `icon(`, or `:::` and a class name.
fn annotation_at(src: &str, at: usize) -> bool {
    let rest = &src[at..];
    if rest.starts_with("##") || rest.starts_with("icon(") {
        return true;
    }
    if !rest.starts_with(":::") {
        return false;
    }
    let b = src.as_bytes();
    let i = blanks_from(b, at + 3);
    b.get(i).is_some_and(|&c| is_name_start(c))
}

/// `/(?!:::|icon\(|##)[^ \t\n\r"'](?:(?![ \t]+:::[ \t]*[A-Za-z_]|[ \t]+icon\(|[ \t]+##)[^\n\r])*/y`.
/// The pattern looks ahead from every blank for an annotation, which rescans long runs of
/// blanks. This looks once per run, as Pele's `matchBareName` does.
fn match_bare_name(src: &str, at: usize) -> Option<usize> {
    let b = src.as_bytes();
    let first = *b.get(at)?;
    if matches!(first, b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'') {
        return None;
    }
    let rest = &src[at..];
    if rest.starts_with(":::") || rest.starts_with("icon(") || rest.starts_with("##") {
        return None;
    }
    let n = b.len();
    // The first character is a whole character, which may be several bytes.
    let mut i = at + char_at(src, at).map_or(1, char::len_utf8);
    while i < n {
        let c = b[i];
        if c == b'\n' || c == b'\r' {
            break;
        }
        if c == b' ' || c == b'\t' {
            let j = blanks_from(b, i + 1);
            if annotation_at(src, j) {
                break;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    Some(i)
}

/// `[\w-]`
fn is_name_char(c: u8) -> bool {
    is_word(c as char) || c == b'-'
}

fn name_run(b: &[u8], mut i: usize) -> usize {
    while b.get(i).is_some_and(|&c| is_name_char(c)) {
        i += 1;
    }
    i
}

/// `/[ \t]+:::[ \t]*[A-Za-z_][\w-]*/y`
fn match_class_annotation(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let q = blanks_from(b, p);
    if q == p || !src[q..].starts_with(":::") {
        return None;
    }
    let r = blanks_from(b, q + 3);
    if !b.get(r).is_some_and(|&c| is_name_start(c)) {
        return None;
    }
    Some(name_run(b, r + 1))
}

/// `/[ \t]+icon\([\w-]*(?::[\w-]+)?\)/y`
fn match_icon_annotation(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let q = blanks_from(b, p);
    if q == p || !src[q..].starts_with("icon(") {
        return None;
    }
    let mut i = name_run(b, q + 5);
    if b.get(i) == Some(&b':') {
        let j = name_run(b, i + 1);
        // The group needs a name after the colon; without one it is skipped, and the colon
        // stands where the closing parenthesis must.
        if j == i + 1 {
            return None;
        }
        i = j;
    }
    (b.get(i) == Some(&b')')).then_some(i + 1)
}

/// `/[ \t]+##[^\n\r]*/y`
fn match_desc_annotation(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let q = blanks_from(b, p);
    if q == p || !src[q..].starts_with("##") {
        return None;
    }
    let mut i = q + 2;
    while i < b.len() && b[i] != b'\n' && b[i] != b'\r' {
        i += 1;
    }
    Some(i)
}

/// `/[ \t]{1,}/y`, also WS.
fn match_blanks(src: &str, p: usize) -> Option<usize> {
    let e = blanks_from(src.as_bytes(), p);
    (e > p).then_some(e)
}

/// `/"[^"]*"|'[^']*'/y`
fn match_quoted(src: &str, p: usize) -> Option<usize> {
    let quote = *src.as_bytes().get(p)?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    src[p + 1..].find(quote as char).map(|k| p + 1 + k + 1)
}

/// `/\%\%[^\n]*/y`
fn match_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("%%") {
        return None;
    }
    Some(src[p + 2..].find('\n').map_or(src.len(), |k| p + 2 + k))
}

fn match_newline(src: &str, p: usize) -> Option<usize> {
    newline_at(src, p)
}

pub static TREEVIEW_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        keyword_token!("treeView-beta").with_longer(&[BARE_NAME]),
        acc_descr("ACC_DESCR"),
        ACC_TITLE,
        TITLE,
        TokenType::func("CLASS_ANNOTATION", match_class_annotation, None),
        TokenType::func("ICON_ANNOTATION", match_icon_annotation, None),
        TokenType::func("DESC_ANNOTATION", match_desc_annotation, None),
        TokenType::func("INDENTATION", match_blanks, None),
        TokenType::func("QUOTED_NAME", match_quoted, None),
        TokenType::func("WS", match_blanks, None).hidden(),
        TokenType::func("ML_COMMENT", match_comment, None).hidden(),
        TokenType::func("NL", match_newline, None).hidden(),
        TokenType::func("BARE_NAME", match_bare_name, None),
    ]
});

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeNodeAst {
    pub indent: Option<usize>,
    pub name: String,
    pub class_annotation: Option<String>,
    pub icon_annotation: Option<String>,
    pub desc_annotation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeViewAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub nodes: Vec<TreeNodeAst>,
}

fn trim_blanks_end(text: &str) -> &str {
    text.trim_end_matches([' ', '\t'])
}

/// Mermaid's TreeViewValueConverter for the annotations: `CLASS_ANNOTATION`,
/// `ICON_ANNOTATION` and `DESC_ANNOTATION`.
fn convert_annotation(kind: usize, input: &str) -> String {
    let trimmed = js::trim(input);
    match kind {
        // `input.trim().substring(3).trim()`
        CLASS_ANNOTATION => js::trim(&trimmed[3..]).to_string(),
        // `trimmed.substring(5, trimmed.length - 1)`
        ICON_ANNOTATION => trimmed[5..trimmed.len() - 1].to_string(),
        // `input.trim().substring(2).trim()`
        _ => js::trim(&trimmed[2..]).to_string(),
    }
}

pub fn parse_tree_view(src: &str) -> Result<TreeViewAst, Error> {
    let types: &[TokenType] = &TREEVIEW_TOKENS;
    let mut r = Reader::new(tokenize(src, types, DiagramType::TreeView)?, types, DiagramType::TreeView);
    let mut ast = TreeViewAst::default();
    r.expect(KEYWORD)?;
    loop {
        match r.kind() {
            T_ACC_DESCR => ast.acc_descr = Some(acc_descr_value(r.take())),
            T_ACC_TITLE => ast.acc_title = Some(acc_title_value(r.take())),
            T_TITLE => ast.title = Some(title_value(r.take())),
            _ => break,
        }
    }
    while r.kind() != EOF {
        let mut node = TreeNodeAst::default();
        // INDENTATION is spaces and tabs, so its length in bytes is its length in code units.
        if r.kind() == INDENTATION {
            node.indent = Some(r.take().len());
        }
        if r.kind() == QUOTED_NAME {
            let text = r.take();
            node.name = text[1..text.len() - 1].to_string();
        } else if r.kind() == BARE_NAME {
            node.name = trim_blanks_end(r.take()).to_string();
        } else {
            return Err(r.fail("a name"));
        }
        loop {
            match r.kind() {
                CLASS_ANNOTATION => node.class_annotation = Some(convert_annotation(CLASS_ANNOTATION, r.take())),
                ICON_ANNOTATION => node.icon_annotation = Some(convert_annotation(ICON_ANNOTATION, r.take())),
                DESC_ANNOTATION => node.desc_annotation = Some(convert_annotation(DESC_ANNOTATION, r.take())),
                _ => break,
            }
        }
        ast.nodes.push(node);
    }
    Ok(ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_names() {
        assert_eq!(match_bare_name("a b  ## d", 0), Some(3));
        assert_eq!(match_bare_name("a b ::: 1", 0), Some(9));
        assert_eq!(match_bare_name("a ::: x", 0), Some(1));
        assert_eq!(match_bare_name("é x\n", 0), Some(4));
        assert_eq!(match_bare_name(":::x", 0), None);
        assert_eq!(match_bare_name("\"x", 0), None);
    }

    #[test]
    fn annotations() {
        assert_eq!(match_class_annotation(" ::: big-one x", 0), Some(12));
        assert_eq!(match_icon_annotation(" icon(fa:file)", 0), Some(14));
        assert_eq!(match_icon_annotation(" icon(fa:)", 0), None);
        assert_eq!(match_icon_annotation(" icon()", 0), Some(7));
        assert_eq!(match_desc_annotation("  ## a b\n", 0), Some(8));
    }

    #[test]
    fn nodes() {
        let ast = parse_tree_view("treeView-beta\n  src/ ::: highlight\n    \"a b.txt\" icon(file) ## main\n").unwrap();
        assert_eq!(ast.nodes.len(), 2);
        assert_eq!(ast.nodes[0].class_annotation.as_deref(), Some("highlight"));
        assert_eq!(ast.nodes[1].icon_annotation.as_deref(), Some("file"));
        assert_eq!(ast.nodes[1].desc_annotation.as_deref(), Some("main"));
        assert_eq!(ast.nodes[1].indent, Some(4));
    }
}
