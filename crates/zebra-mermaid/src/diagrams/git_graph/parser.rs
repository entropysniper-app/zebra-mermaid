//! The gitGraph parser, ported from Pele `src/diagrams/git/parser.ts` (Mermaid's Langium
//! grammar, read with the shared tokenizer).

use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, blanks, spaces, string_value, title_value, tokenize, Reader, TokenType, ACC_DESCR, ACC_TITLE,
    DIRECTIVE, EOF, NEWLINE, SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::keyword_token;
use crate::util::js;

const CHERRY_PICK: usize = 0;
const GIT_GRAPH_COLON: usize = 1;
const HIGHLIGHT: usize = 2;
const GIT_GRAPH: usize = 3;
const CHECKOUT: usize = 4;
const REVERSE: usize = 5;
const PARENT: usize = 6;
const COMMIT: usize = 7;
const NORMAL: usize = 8;
const BRANCH: usize = 9;
const ORDER: usize = 10;
const SWITCH: usize = 11;
const TYPE: usize = 12;
const MERGE: usize = 13;
const MSG: usize = 14;
const TAG: usize = 15;
const ID: usize = 16;
const LR: usize = 17;
const TB: usize = 18;
const BT: usize = 19;
const COLON: usize = 20;
const T_ACC_DESCR: usize = 21;
const T_ACC_TITLE: usize = 22;
const T_TITLE: usize = 23;
const INT: usize = 24;
const T_STRING: usize = 25;
const T_NEWLINE: usize = 26;
const REFERENCE: usize = 31;

/// `/\w([-\./\w]*[-\w])?/y`: a word character, then the longest run of `-./` and word
/// characters that ends in a dash or a word character.
fn match_reference(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    if !b.get(p).is_some_and(|&c| is_word(c)) {
        return None;
    }
    let mut end = p + 1;
    let mut i = p + 1;
    while i < b.len() && (is_word(b[i]) || matches!(b[i], b'-' | b'.' | b'/')) {
        i += 1;
        if b[i - 1] != b'.' && b[i - 1] != b'/' {
            end = i;
        }
    }
    Some(end)
}

/// `/0|[1-9][0-9]*(?!\.)/y`. The lookahead belongs to the second branch only, and the digits
/// give back one when a dot follows them: `12.5` matches `1`, `1.5` nothing.
fn match_int(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    match b.get(p) {
        Some(b'0') => Some(p + 1),
        Some(b'1'..=b'9') => {
            let mut e = p + 1;
            while e < b.len() && b[e].is_ascii_digit() {
                e += 1;
            }
            if b.get(e) != Some(&b'.') {
                Some(e)
            } else if e - p > 1 {
                Some(e - 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `/[\t ]*accDescr\s*{/y`: without a closing brace the block form scans to the end of the
/// text, and a brace is not a token on its own, so the first failure is final.
fn acc_descr_opener(src: &str, p: usize) -> Option<usize> {
    let q = blanks(src, p);
    if !src[q..].starts_with("accDescr") {
        return None;
    }
    let o = spaces(src, q + 8);
    src[o..].starts_with('{').then_some(o + 1)
}

const NAME: &[usize] = &[REFERENCE];

/// A keyword that is the start of a longer name is part of that name: `commits` is a branch.
const fn word(name: &'static str) -> TokenType {
    TokenType::literal(name, name).with_longer(NAME)
}

pub static GIT_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        word("cherry-pick"),
        TokenType::literal("gitGraph:", "gitGraph:"),
        word("HIGHLIGHT"),
        keyword_token!("gitGraph").with_longer(NAME),
        word("checkout"),
        word("REVERSE"),
        TokenType::literal("parent:", "parent:"),
        word("commit"),
        word("NORMAL"),
        word("branch"),
        TokenType::literal("order:", "order:"),
        word("switch"),
        TokenType::literal("type:", "type:"),
        word("merge"),
        TokenType::literal("msg:", "msg:"),
        TokenType::literal("tag:", "tag:"),
        TokenType::literal("id:", "id:"),
        word("LR"),
        word("TB"),
        word("BT"),
        TokenType::literal(":", ":"),
        ACC_DESCR.with_opener(acc_descr_opener),
        ACC_TITLE,
        TITLE,
        TokenType::func("INT", match_int, Some("0123456789")),
        STRING,
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
        TokenType::func("REFERENCE", match_reference, Some("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz")),
    ]
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitTypeName {
    Normal,
    Reverse,
    Highlight,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementAst {
    Commit { id: Option<String>, message: Option<String>, tags: Vec<String>, kind: Option<CommitTypeName> },
    Branch { name: String, order: Option<f64> },
    Merge { branch: String, id: Option<String>, tags: Vec<String>, kind: Option<CommitTypeName> },
    Checkout { branch: String },
    CherryPicking { id: Option<String>, tags: Vec<String>, parent: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GitDirection {
    LR,
    TB,
    BT,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GitGraphAst {
    pub dir: Option<GitDirection>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub statements: Vec<StatementAst>,
}

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

fn name(r: &mut Reader) -> Result<String, Error> {
    match r.kind() {
        REFERENCE => Ok(r.take().to_string()),
        T_STRING => Ok(string_value(r.take())),
        _ => Err(r.fail("a branch name")),
    }
}

fn string(r: &mut Reader) -> Result<String, Error> {
    Ok(string_value(r.expect(T_STRING)?))
}

fn commit_type(r: &mut Reader) -> Result<CommitTypeName, Error> {
    let kind = match r.kind() {
        NORMAL => CommitTypeName::Normal,
        REVERSE => CommitTypeName::Reverse,
        HIGHLIGHT => CommitTypeName::Highlight,
        _ => return Err(r.fail("NORMAL, REVERSE, or HIGHLIGHT")),
    };
    r.i += 1;
    Ok(kind)
}

pub fn parse_git(src: &str) -> Result<GitGraphAst, Error> {
    let types: &[TokenType] = &GIT_TOKENS;
    let mut r = Reader::new(tokenize(src, types, DiagramType::GitGraph)?, types, DiagramType::GitGraph);
    let mut ast = GitGraphAst::default();

    while r.kind() == T_NEWLINE {
        r.i += 1;
    }
    if !r.accept(GIT_GRAPH_COLON) {
        r.expect(GIT_GRAPH)?;
        let next = r.kind_at(r.i + 1);
        let dir = match r.kind() {
            LR => Some(GitDirection::LR),
            TB => Some(GitDirection::TB),
            BT => Some(GitDirection::BT),
            _ => None,
        };
        match dir {
            Some(dir) if next == COLON => {
                ast.dir = Some(dir);
                r.i += 2;
            }
            _ => {
                r.accept(COLON);
            }
        }
    }

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
            COMMIT => {
                r.i += 1;
                let (mut id, mut message, mut tags, mut kind) = (None, None, Vec::new(), None);
                loop {
                    if r.accept(ID) {
                        id = Some(string(&mut r)?);
                    } else if r.accept(TAG) {
                        tags.push(string(&mut r)?);
                    } else if r.accept(TYPE) {
                        kind = Some(commit_type(&mut r)?);
                    } else if r.accept(MSG) || r.kind() == T_STRING {
                        message = Some(string(&mut r)?);
                    } else {
                        break;
                    }
                }
                end_of_line(&mut r)?;
                ast.statements.push(StatementAst::Commit { id, message, tags, kind });
            }
            BRANCH => {
                r.i += 1;
                let name = name(&mut r)?;
                let mut order = None;
                if r.accept(ORDER) {
                    order = Some(js::parse_int(r.expect(INT)?, 0));
                }
                end_of_line(&mut r)?;
                ast.statements.push(StatementAst::Branch { name, order });
            }
            MERGE => {
                r.i += 1;
                let branch = name(&mut r)?;
                let (mut id, mut tags, mut kind) = (None, Vec::new(), None);
                loop {
                    if r.accept(ID) {
                        id = Some(string(&mut r)?);
                    } else if r.accept(TAG) {
                        tags.push(string(&mut r)?);
                    } else if r.accept(TYPE) {
                        kind = Some(commit_type(&mut r)?);
                    } else {
                        break;
                    }
                }
                end_of_line(&mut r)?;
                ast.statements.push(StatementAst::Merge { branch, id, tags, kind });
            }
            CHECKOUT | SWITCH => {
                r.i += 1;
                let branch = name(&mut r)?;
                end_of_line(&mut r)?;
                ast.statements.push(StatementAst::Checkout { branch });
            }
            CHERRY_PICK => {
                r.i += 1;
                let (mut id, mut tags, mut parent) = (None, Vec::new(), None);
                loop {
                    if r.accept(ID) {
                        id = Some(string(&mut r)?);
                    } else if r.accept(TAG) {
                        tags.push(string(&mut r)?);
                    } else if r.accept(PARENT) {
                        parent = Some(string(&mut r)?);
                    } else {
                        break;
                    }
                }
                end_of_line(&mut r)?;
                ast.statements.push(StatementAst::CherryPicking { id, tags, parent });
            }
            _ => return Err(r.fail("a statement, a title, or a line break")),
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_matchers() {
        assert_eq!(match_reference("feature/x.y-", 0), Some(12));
        assert_eq!(match_reference("a./", 0), Some(1));
        assert_eq!(match_reference("a.b/", 0), Some(3));
        assert_eq!(match_reference("-a", 0), None);
        assert_eq!(match_reference("main ", 0), Some(4));
        assert_eq!(match_int("0123", 0), Some(1));
        assert_eq!(match_int("12.5", 0), Some(1));
        assert_eq!(match_int("123.", 0), Some(2));
        assert_eq!(match_int("1.5", 0), None);
        assert_eq!(match_int("42 ", 0), Some(2));
        assert_eq!(acc_descr_opener("  accDescr \n {", 0), Some(14));
        assert_eq!(acc_descr_opener("accDescr: x", 0), None);
    }

    #[test]
    fn keywords_give_way_to_longer_names() {
        let ast = parse_git("gitGraph\n  checkout cherry-picked\n  cherry-pick id: \"x\"\n").unwrap();
        assert_eq!(
            ast.statements,
            vec![
                StatementAst::Checkout { branch: "cherry-picked".into() },
                StatementAst::CherryPicking { id: Some("x".into()), tags: vec![], parent: None },
            ]
        );
        let ast = parse_git("gitGraph TB:\n  commit id: \"a\" tag: \"v1\" type: HIGHLIGHT\n  branch commits order: 2\n").unwrap();
        assert_eq!(ast.dir, Some(GitDirection::TB));
        assert_eq!(
            ast.statements,
            vec![
                StatementAst::Commit {
                    id: Some("a".into()),
                    message: None,
                    tags: vec!["v1".into()],
                    kind: Some(CommitTypeName::Highlight)
                },
                StatementAst::Branch { name: "commits".into(), order: Some(2.0) },
            ]
        );
    }
}
