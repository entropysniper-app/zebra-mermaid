//! Pie parser. Ported from Pele `src/diagrams/pie/parser.ts`.

use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, string_value, title_value, tokenize, Pattern, Reader, Scanner, TokenType, ACC_DESCR, ACC_TITLE,
    DIRECTIVE, EOF, NEWLINE, SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::keyword_token;
use crate::util::js;

const SHOW_DATA: usize = 0;
const PIE: usize = 1;
const COLON: usize = 2;
const NUMBER: usize = 3;
const T_ACC_DESCR: usize = 4;
const T_ACC_TITLE: usize = 5;
const T_TITLE: usize = 6;
const T_STRING: usize = 7;
const T_NEWLINE: usize = 8;

fn digits(src: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < src.len() && src[i].is_ascii_digit() {
        i += 1;
    }
    i
}

/// Answers as NUMBER_PIE's pattern, `(?:-?[0-9]+\.[0-9]+(?!\.))|(?:-?(0|[1-9][0-9]*)(?!\.))`,
/// does. The pattern reads a whole run of digits before it finds there is no fraction, and
/// would do so again from every digit of a long run of zeros; this remembers the run.
fn number_scanner() -> Scanner {
    let mut from = 0usize;
    let mut end = 0usize;
    let mut fraction: Option<usize> = None;
    Box::new(move |src: &str, p: usize| {
        let b = src.as_bytes();
        let at = if b.get(p) == Some(&b'-') { p + 1 } else { p };
        if at < from || at >= end {
            from = at;
            end = digits(b, at);
            fraction = None;
            if end > at && b.get(end) == Some(&b'.') {
                let stop = digits(b, end + 1);
                let length = stop - end - 1;
                if length > 0 && b.get(stop) != Some(&b'.') {
                    fraction = Some(stop);
                } else if length > 1 {
                    fraction = Some(stop - 1);
                }
            }
        }
        if end == at {
            return None;
        }
        if let Some(f) = fraction {
            return Some(f);
        }
        if b.get(at) == Some(&b'0') {
            return if b.get(at + 1) == Some(&b'.') { None } else { Some(at + 1) };
        }
        if b.get(end) != Some(&b'.') {
            return Some(end);
        }
        if end - 1 > at {
            Some(end - 1)
        } else {
            None
        }
    })
}

pub static PIE_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        keyword_token!("showData"),
        keyword_token!("pie"),
        TokenType::literal(":", ":"),
        TokenType {
            name: "NUMBER_PIE",
            pattern: Pattern::Scanner(number_scanner),
            hidden: false,
            opener: None,
            first: Some("-0123456789"),
            longer: &[],
        },
        ACC_DESCR,
        ACC_TITLE,
        TITLE,
        STRING,
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
    ]
});

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PieAst {
    pub show_data: bool,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub sections: Vec<(String, f64)>,
}

pub fn parse_pie(src: &str) -> Result<PieAst, Error> {
    let types: &[TokenType] = &PIE_TOKENS;
    let mut r = Reader::new(tokenize(src, types, DiagramType::Pie)?, types, DiagramType::Pie);
    let mut ast = PieAst::default();

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
    r.expect(PIE)?;
    ast.show_data = r.accept(SHOW_DATA);
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
            T_STRING => {
                let label = string_value(r.take());
                r.expect(COLON)?;
                let value = js::number(r.expect(NUMBER)?);
                ast.sections.push((label, value));
                end_of_line(&mut r)?;
            }
            _ => return Err(r.fail("a section, a title, or a line break")),
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        // A scanner remembers the text it scans, so each text gets its own.
        assert_eq!(number_scanner()("12.5x", 0), Some(4));
        assert_eq!(number_scanner()("-3 ", 0), Some(2));
        assert_eq!(number_scanner()("0.5", 0), Some(3));
        let mut s = number_scanner();
        assert_eq!(s("1.2.3", 0), None);
        let mut s = number_scanner();
        assert_eq!(s("05", 0), Some(1));
        let mut s = number_scanner();
        assert_eq!(s("12.", 0), Some(1));
    }

    #[test]
    fn parses_sections() {
        let ast = parse_pie("pie showData\n  title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85.5\n").unwrap();
        assert!(ast.show_data);
        assert_eq!(ast.title.as_deref(), Some("Pets"));
        assert_eq!(ast.sections, vec![("Dogs".to_string(), 386.0), ("Cats".to_string(), 85.5)]);
    }

    #[test]
    fn reports_positions() {
        let e = parse_pie("pie\n  \"a\" 1\n").unwrap_err();
        assert_eq!(e.line, 2);
        assert_eq!(e.message, "Parse error on line 2:\n  \"a\" 1\n------^\nExpecting token of type ':' but found `1`.");
    }
}
