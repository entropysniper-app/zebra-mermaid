//! Architecture parser. Ported from Pele `src/diagrams/architecture/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, match_string, string_value, title_value, TokenType, ACC_DESCR, ACC_TITLE, DIRECTIVE, EOF, NEWLINE,
    SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::util::chars::is_word_byte;
use crate::util::js;

const HEADER: usize = 0;
const JUNCTION: usize = 1;
const SERVICE: usize = 2;
const COLUMN: usize = 3;
const GROUP: usize = 4;
const ALIGN: usize = 5;
const ROW: usize = 6;
const DASHES: usize = 7;
const IN: usize = 8;
const COLON: usize = 9;
const DASH: usize = 10;
const DIRECTION: usize = 11;
const ARROW_GROUP: usize = 12;
const INTO: usize = 13;
const T_ACC_DESCR: usize = 14;
const T_ACC_TITLE: usize = 15;
const T_TITLE: usize = 16;
const T_STRING: usize = 17;
const ID: usize = 18;
const T_NEWLINE: usize = 19;
const T_YAML: usize = 21;
const ICON: usize = 24;
const LABEL: usize = 25;

/// `/[\w]([-\w]*\w)?/y`: a word character, then a run of word characters and dashes that
/// gives back the dashes it ends with.
pub(crate) fn match_id(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if p >= b.len() || !is_word_byte(b[p]) {
        return None;
    }
    let mut end = p + 1;
    let mut i = p + 1;
    while i < b.len() && (is_word_byte(b[i]) || b[i] == b'-') {
        i += 1;
        if is_word_byte(b[i - 1]) {
            end = i;
        }
    }
    Some(end)
}

/// `/L|R|T|B/y`
fn match_direction(src: &str, p: usize) -> Option<usize> {
    matches!(src.as_bytes().get(p), Some(b'L' | b'R' | b'T' | b'B')).then_some(p + 1)
}

/// `/\{group\}/y`
fn match_arrow_group(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with("{group}").then_some(p + 7)
}

/// `/<|>/y`
fn match_into(src: &str, p: usize) -> Option<usize> {
    matches!(src.as_bytes().get(p), Some(b'<' | b'>')).then_some(p + 1)
}

/// `/\([\w-:]+\)/y`: the class holds word characters, `-` and `:`, and cannot hold the `)`
/// that has to follow it, so the run is read once.
fn match_icon(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if b.get(p) != Some(&b'(') {
        return None;
    }
    let mut i = p + 1;
    while i < b.len() && (is_word_byte(b[i]) || b[i] == b'-' || b[i] == b':') {
        i += 1;
    }
    (i > p + 1 && b.get(i) == Some(&b')')).then_some(i + 1)
}

/// `/\[(?:"([^"\\]|\\.)*"|'([^'\\]|\\.)*'|[^\[\]\r\n]+)\]/y`. A quoted title ends at its first
/// unescaped quote, which must be followed by `]`; otherwise the expression backtracks to the
/// third alternative, a run of anything but brackets and line breaks up to the `]`.
fn match_label(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if b.get(p) != Some(&b'[') {
        return None;
    }
    let q = p + 1;
    if matches!(b.get(q), Some(b'"' | b'\'')) {
        if let Some(end) = match_string(src, q) {
            if b.get(end) == Some(&b']') {
                return Some(end + 1);
            }
        }
    }
    let mut i = q;
    while i < b.len() && !matches!(b[i], b'[' | b']' | b'\r' | b'\n') {
        i += 1;
    }
    (i > q && b.get(i) == Some(&b']')).then_some(i + 1)
}

/// A word that gives way to a longer identifier, as `word()` builds it in Pele.
const fn word(text: &'static str) -> TokenType {
    TokenType::literal(text, text).with_longer(&[ID])
}

pub static ARCHITECTURE_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        word("architecture-beta"),
        word("junction"),
        word("service"),
        word("column"),
        word("group"),
        word("align"),
        word("row"),
        TokenType::literal("--", "--").with_longer(&[T_YAML]),
        word("in"),
        TokenType::literal(":", ":"),
        TokenType::literal("-", "-").with_longer(&[T_YAML]),
        // The direction letters come before identifiers and have no longer alternative, so
        // Mermaid reads an id that starts with L, R, T or B as a direction followed by the rest.
        TokenType::func("ARROW_DIRECTION", match_direction, Some("LRTB")),
        TokenType::func("ARROW_GROUP", match_arrow_group, Some("{")),
        TokenType::func("ARROW_INTO", match_into, Some("<>")),
        ACC_DESCR,
        ACC_TITLE,
        TITLE,
        STRING,
        TokenType::func("ID", match_id, None),
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
        TokenType::func("ARCH_ICON", match_icon, Some("(")),
        TokenType::func("ARCH_TITLE", match_label, Some("[")),
    ]
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Side {
    L,
    R,
    T,
    B,
}

impl Side {
    fn from_token(text: &str) -> Side {
        match text {
            "L" => Side::L,
            "R" => Side::R,
            "T" => Side::T,
            _ => Side::B,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AlignDirection {
    Row,
    Column,
}

impl AlignDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlignDirection::Row => "row",
            AlignDirection::Column => "column",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroupAst {
    pub id: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceAst {
    pub id: String,
    pub icon: Option<String>,
    pub icon_text: Option<String>,
    pub title: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JunctionAst {
    pub id: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EdgeAst {
    pub lhs_id: String,
    pub lhs_group: bool,
    pub lhs_dir: Side,
    pub lhs_into: bool,
    pub title: Option<String>,
    pub rhs_into: bool,
    pub rhs_dir: Side,
    pub rhs_id: String,
    pub rhs_group: bool,
}

#[derive(Debug, Clone)]
pub struct AlignmentAst {
    pub direction: AlignDirection,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArchitectureAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub groups: Vec<GroupAst>,
    pub services: Vec<ServiceAst>,
    pub junctions: Vec<JunctionAst>,
    pub edges: Vec<EdgeAst>,
    pub alignments: Vec<AlignmentAst>,
}

/// Mermaid's ArchitectureValueConverter for ARCH_TITLE.
fn label_value(text: &str) -> String {
    let inner = &text[1..text.len() - 1];
    let mut result = js::trim(inner).to_string();
    if (result.starts_with('"') && result.ends_with('"')) || (result.starts_with('\'') && result.ends_with('\'')) {
        let n = js::len16(&result);
        result = js::slice16(&result, 1, n.saturating_sub(1)).replace("\\\"", "\"").replace("\\'", "'");
    }
    js::trim(&result).to_string()
}

pub fn parse_architecture(src: &str) -> Result<ArchitectureAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&ARCHITECTURE_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Architecture)?;
    let mut ast = ArchitectureAst::default();

    fn end_of_line(r: &mut Reader) -> Result<(), Error> {
        if r.kind() == EOF {
            return Ok(());
        }
        if r.kind() != T_NEWLINE {
            return Err(r.fail("a line break or the end of input"));
        }
        while r.kind() == T_NEWLINE {
            r.i += 1;
        }
        Ok(())
    }

    while r.kind() == T_NEWLINE {
        r.i += 1;
    }
    r.expect(HEADER)?;
    while r.kind() != EOF {
        match r.kind() {
            T_NEWLINE => {
                r.i += 1;
                continue;
            }
            T_ACC_DESCR => ast.acc_descr = Some(acc_descr_value(r.take())),
            T_ACC_TITLE => ast.acc_title = Some(acc_title_value(r.take())),
            T_TITLE => ast.title = Some(title_value(r.take())),
            GROUP => {
                r.i += 1;
                let mut group = GroupAst { id: r.expect(ID)?.to_string(), ..Default::default() };
                if r.kind() == ICON {
                    let t = r.take();
                    group.icon = Some(t[1..t.len() - 1].to_string());
                }
                if r.kind() == LABEL {
                    group.title = Some(label_value(r.take()));
                }
                if r.accept(IN) {
                    group.parent = Some(r.expect(ID)?.to_string());
                }
                ast.groups.push(group);
            }
            SERVICE => {
                r.i += 1;
                let mut service = ServiceAst { id: r.expect(ID)?.to_string(), ..Default::default() };
                if r.kind() == T_STRING {
                    service.icon_text = Some(string_value(r.take()));
                } else if r.kind() == ICON {
                    let t = r.take();
                    service.icon = Some(t[1..t.len() - 1].to_string());
                }
                if r.kind() == LABEL {
                    service.title = Some(label_value(r.take()));
                }
                if r.accept(IN) {
                    service.parent = Some(r.expect(ID)?.to_string());
                }
                ast.services.push(service);
            }
            JUNCTION => {
                r.i += 1;
                let mut junction = JunctionAst { id: r.expect(ID)?.to_string(), parent: None };
                if r.accept(IN) {
                    junction.parent = Some(r.expect(ID)?.to_string());
                }
                ast.junctions.push(junction);
            }
            ID => {
                let lhs_id = r.take().to_string();
                let lhs_group = r.accept(ARROW_GROUP);
                r.expect(COLON)?;
                let lhs_dir = Side::from_token(r.expect(DIRECTION)?);
                let lhs_into = r.accept(INTO);
                let mut title = None;
                if !r.accept(DASHES) {
                    if r.kind() != DASH {
                        return Err(r.fail("'--' or '-'"));
                    }
                    r.i += 1;
                    title = Some(label_value(r.expect(LABEL)?));
                    r.expect(DASH)?;
                }
                let rhs_into = r.accept(INTO);
                let rhs_dir = Side::from_token(r.expect(DIRECTION)?);
                r.expect(COLON)?;
                let rhs_id = r.expect(ID)?.to_string();
                let rhs_group = r.accept(ARROW_GROUP);
                ast.edges.push(EdgeAst { lhs_id, lhs_group, lhs_dir, lhs_into, title, rhs_into, rhs_dir, rhs_id, rhs_group });
            }
            ALIGN => {
                r.i += 1;
                if r.kind() != ROW && r.kind() != COLUMN {
                    return Err(r.fail("'row' or 'column'"));
                }
                let direction = if r.take() == "row" { AlignDirection::Row } else { AlignDirection::Column };
                let mut members = vec![r.expect(ID)?.to_string(), r.expect(ID)?.to_string()];
                while r.kind() == ID {
                    members.push(r.take().to_string());
                }
                ast.alignments.push(AlignmentAst { direction, members });
            }
            _ => return Err(r.fail("a group, a service, a junction, an edge, an alignment, a title, or a line break")),
        }
        end_of_line(&mut r)?;
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table(
            "architecture",
            &super::ARCHITECTURE_TOKENS,
            crate::DiagramType::Architecture,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matchers() {
        assert_eq!(match_id("a--b-- x", 0), Some(4));
        assert_eq!(match_id("a--", 0), Some(1));
        assert_eq!(match_id("-a", 0), None);
        assert_eq!(match_icon("(logos:aws-s3)x", 0), Some(14));
        assert_eq!(match_icon("()", 0), None);
        assert_eq!(match_icon("(a b)", 0), None);
        assert_eq!(match_label("[\"a]b\"]", 0), Some(7));
        assert_eq!(match_label("['a'b]", 0), Some(6));
        assert_eq!(match_label("[\"a\" ]", 0), Some(6));
        assert_eq!(match_label("[]", 0), None);
        assert_eq!(match_label("[a\nb]", 0), None);
        assert_eq!(match_label("[\"a\nb\"]", 0), Some(7));
    }

    #[test]
    fn label_values() {
        assert_eq!(label_value("[ \"Big \\\"one\\\"\" ]"), "Big \"one\"");
        assert_eq!(label_value("['it\\'s']"), "it's");
        assert_eq!(label_value("[\"]"), "");
        assert_eq!(label_value("[ plain ]"), "plain");
    }

    #[test]
    fn reads_directions_before_ids() {
        let ast = parse_architecture("architecture-beta\n service Lb(server)[LB]\n").unwrap_err();
        assert!(ast.message.contains("Expecting token of type 'ID' but found `L`."), "{}", ast.message);
    }
}
