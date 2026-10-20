//! Cynefin parser. Ported from Pele `src/diagrams/cynefin/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, string_value, title_value, TokenType, ACC_DESCR, ACC_TITLE, DIRECTIVE, EOF, NEWLINE,
    SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::keyword_token;

const KEYWORD_COLON: usize = 0;
const KEYWORD: usize = 1;
const ARROW: usize = 2;
const COLON: usize = 3;
const DOMAIN: usize = 4;
const T_ACC_DESCR: usize = 5;
const T_ACC_TITLE: usize = 6;
const T_TITLE: usize = 7;
const T_STRING: usize = 8;
const T_NEWLINE: usize = 9;

/// The five domains, in the order `/complex|complicated|clear|chaotic|confusion/y` tries them.
pub const DOMAIN_NAMES: [&str; 5] = ["complex", "complicated", "clear", "chaotic", "confusion"];

/// `/complex|complicated|clear|chaotic|confusion/y`: the first alternative that matches wins.
fn match_domain(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    DOMAIN_NAMES.iter().find(|name| rest.starts_with(*name)).map(|name| p + name.len())
}

pub static CYNEFIN_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        TokenType::literal("cynefin-beta:", "cynefin-beta:"),
        keyword_token!("cynefin-beta"),
        TokenType::literal("-->", "-->"),
        TokenType::literal(":", ":"),
        TokenType::func("DOMAIN_NAME", match_domain, Some("c")),
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
pub struct DomainBlockAst {
    pub domain: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransitionAst {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CynefinAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub domains: Vec<DomainBlockAst>,
    pub transitions: Vec<TransitionAst>,
}

pub fn parse_cynefin(src: &str) -> Result<CynefinAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&CYNEFIN_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Cynefin)?;
    let mut ast = CynefinAst::default();

    fn skip_lines(r: &mut Reader) {
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
        skip_lines(r);
        Ok(())
    }

    skip_lines(&mut r);
    if r.kind() != KEYWORD && r.kind() != KEYWORD_COLON {
        return Err(r.fail("'cynefin-beta'"));
    }
    r.i += 1;
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
            DOMAIN => {
                let name = r.take().to_string();
                if r.accept(ARROW) {
                    let to = r.expect(DOMAIN)?.to_string();
                    let mut transition = TransitionAst { from: name, to, label: None };
                    if r.accept(COLON) {
                        transition.label = Some(string_value(r.expect(T_STRING)?));
                    }
                    end_of_line(&mut r)?;
                    ast.transitions.push(transition);
                    continue;
                }
                let mut block = DomainBlockAst { domain: name, items: Vec::new() };
                skip_lines(&mut r);
                while r.kind() == T_STRING {
                    block.items.push(string_value(r.take()));
                    skip_lines(&mut r);
                }
                ast.domains.push(block);
            }
            _ => return Err(r.fail("a domain, a transition, a title, or a line break")),
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("cynefin", &super::CYNEFIN_TOKENS, crate::DiagramType::Cynefin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_and_transitions() {
        let ast = parse_cynefin("cynefin-beta\n  complex\n    \"A\"\n\n    'B'\n  complex --> clear : \"go\"\n  chaotic\n").unwrap();
        assert_eq!(ast.domains.len(), 2);
        assert_eq!(ast.domains[0].items, ["A", "B"]);
        assert_eq!(ast.transitions, [TransitionAst { from: "complex".into(), to: "clear".into(), label: Some("go".into()) }]);
        assert_eq!(match_domain("complicated", 0), Some(11));
        assert_eq!(match_domain("complexity", 0), Some(7));
    }

    #[test]
    fn errors() {
        let e = parse_cynefin("cynefin-beta\n  complex -->\n").unwrap_err();
        assert!(e.message.ends_with("Expecting token of type 'DOMAIN_NAME' but found `\n`."), "{}", e.message);
        let e = parse_cynefin("cynefin\n").unwrap_err();
        assert_eq!(e.code, crate::error::ErrorCode::Syntax);
    }
}
