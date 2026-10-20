//! Packet parser. Ported from Pele `src/diagrams/packet/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, string_value, title_value, TokenType, ACC_DESCR, ACC_TITLE, DIRECTIVE, EOF, NEWLINE,
    SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::keyword_token;
use crate::util::js;

const PACKET_BETA: usize = 0;
const PACKET: usize = 1;
const DASH: usize = 2;
const PLUS: usize = 3;
const COLON: usize = 4;
const T_ACC_DESCR: usize = 5;
const T_ACC_TITLE: usize = 6;
const T_TITLE: usize = 7;
const INT: usize = 8;
const T_STRING: usize = 9;
const T_NEWLINE: usize = 10;
const T_YAML: usize = 12;

/// `/0|[1-9][0-9]*(?!\.)/y`: a zero, or a number that a point does not follow. The digits give
/// back one at a time until the next character is not a point.
fn match_int(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    match b.get(p) {
        Some(b'0') => Some(p + 1),
        Some(b'1'..=b'9') => {
            let mut end = p + 1;
            while end < b.len() && b[end].is_ascii_digit() {
                end += 1;
            }
            if b.get(end) != Some(&b'.') {
                Some(end)
            } else if end - 1 > p {
                Some(end - 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub static PACKET_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        TokenType::literal("packet-beta", "packet-beta"),
        keyword_token!("packet"),
        // A front matter block also starts with dashes, so Langium lets it win over a dash.
        TokenType::literal("-", "-").with_longer(&[T_YAML]),
        TokenType::literal("+", "+"),
        TokenType::literal(":", ":"),
        ACC_DESCR,
        ACC_TITLE,
        TITLE,
        TokenType::func("INT", match_int, None),
        STRING,
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
    ]
});

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PacketBlockAst {
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub bits: Option<f64>,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PacketAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub blocks: Vec<PacketBlockAst>,
}

pub fn parse_packet(src: &str) -> Result<PacketAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&PACKET_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Packet)?;
    let mut ast = PacketAst::default();

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
    if !r.accept(PACKET_BETA) {
        r.expect(PACKET)?;
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
            INT | PLUS => {
                let mut block = PacketBlockAst::default();
                if r.accept(PLUS) {
                    block.bits = Some(js::number(r.expect(INT)?));
                } else {
                    block.start = Some(js::number(r.take()));
                    if r.accept(DASH) {
                        block.end = Some(js::number(r.expect(INT)?));
                    }
                }
                r.expect(COLON)?;
                block.label = string_value(r.expect(T_STRING)?);
                ast.blocks.push(block);
                end_of_line(&mut r)?;
            }
            _ => return Err(r.fail("a field, a title, or a line break")),
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("packet", &super::PACKET_TOKENS, crate::DiagramType::Packet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ints() {
        assert_eq!(match_int("0.5", 0), Some(1));
        assert_eq!(match_int("12.", 0), Some(1));
        assert_eq!(match_int("1.", 0), None);
        assert_eq!(match_int("123-", 0), Some(3));
        assert_eq!(match_int("x", 0), None);
    }

    #[test]
    fn blocks() {
        let ast = parse_packet("packet\n0-15: \"Source\"\n+16: \"Dest\"\n").unwrap();
        assert_eq!(ast.blocks.len(), 2);
        assert_eq!(ast.blocks[0].end, Some(15.0));
        assert_eq!(ast.blocks[1].bits, Some(16.0));
    }
}
