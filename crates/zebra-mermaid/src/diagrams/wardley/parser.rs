//! Wardley map parser. Ported from Pele `src/diagrams/wardley/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::accdescr::acc_descr;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, match_whitespace, string_value, title_value, Pattern, Scanner, TokenType, ACC_TITLE, DIRECTIVE, EOF,
    NEWLINE, SINGLE_LINE_COMMENT, STRING, TITLE, WHITESPACE, YAML,
};
use crate::error::Error;
use crate::util::chars::is_word_byte;
use crate::util::js;

const LBRACKET: usize = 0;
const COMMA: usize = 1;
const RBRACKET: usize = 2;
const AT: usize = 3;
const SLASH: usize = 4;
const LPAREN: usize = 5;
const RPAREN: usize = 6;
const MINUS: usize = 7;
const LBRACE: usize = 8;
const RBRACE: usize = 9;
const NUMBER: usize = 10;
const ARROW: usize = 11;
const LINK_PORT: usize = 12;
const LINK_ARROW: usize = 13;
const LINK_LABEL: usize = 14;
const STRATEGY: usize = 15;
const WARDLEY: usize = 16;
const SIZE: usize = 17;
const EVOLUTION: usize = 18;
const ANCHOR: usize = 19;
const COMPONENT: usize = 20;
const LABEL: usize = 21;
const INERTIA: usize = 22;
const EVOLVE: usize = 23;
const PIPELINE: usize = 24;
const NOTE: usize = 25;
const ANNOTATIONS: usize = 26;
const ANNOTATION: usize = 27;
const ACCELERATOR: usize = 28;
const DEACCELERATOR: usize = 29;
const NAME: usize = 30;
const T_ACC_DESCR: usize = 32;
const T_ACC_TITLE: usize = 33;
const T_TITLE: usize = 34;
const INT: usize = 35;
const T_STRING: usize = 36;
const ID: usize = 37;
const T_NEWLINE: usize = 38;
const T_YAML: usize = 40;

fn digits(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i
}

/// Answers as WARDLEY_NUMBER's pattern, `/[0-9]+\.[0-9]+/y`, does. The pattern reads a whole
/// run of digits before it finds there is no fraction, and would do so again from every digit
/// of a long run; this remembers the run.
fn number_scanner() -> Scanner {
    let mut from = 0usize;
    let mut end = 0usize;
    Box::new(move |src: &str, p: usize| {
        let b = src.as_bytes();
        if p < from || p >= end {
            from = p;
            end = digits(b, p);
        }
        if end == p || b.get(end) != Some(&b'.') {
            return None;
        }
        let stop = digits(b, end + 1);
        (stop > end + 1).then_some(stop)
    })
}

/// `/->/y`
fn match_arrow(src: &str, p: usize) -> Option<usize> {
    src[p..].starts_with("->").then_some(p + 2)
}

/// `/\+<>|\+>|\+</y`
fn match_link_port(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    if rest.starts_with("+<>") {
        Some(p + 3)
    } else if rest.starts_with("+>") || rest.starts_with("+<") {
        Some(p + 2)
    } else {
        None
    }
}

/// `/-->|-\.->|>|\+'[^']*'<>|\+'[^']*'<|\+'[^']*'>/y`. The quoted part runs to the next quote
/// whatever follows it, so the three quoted alternatives share one scan.
fn match_link_arrow(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    if rest.starts_with("-->") {
        return Some(p + 3);
    }
    if rest.starts_with("-.->") {
        return Some(p + 4);
    }
    if rest.starts_with('>') {
        return Some(p + 1);
    }
    if !rest.starts_with("+'") {
        return None;
    }
    let close = p + 2 + src[p + 2..].find('\'')?;
    let after = &src[close + 1..];
    if after.starts_with("<>") {
        Some(close + 3)
    } else if after.starts_with('<') || after.starts_with('>') {
        Some(close + 2)
    } else {
        None
    }
}

/// `/;[^\n\r]+/y`
fn match_link_label(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with(';') {
        return None;
    }
    let end = src[p + 1..].find(['\n', '\r']).map_or(src.len(), |k| p + 1 + k);
    (end > p + 1).then_some(end)
}

/// `/build|buy|outsource|market/y`
fn match_strategy(src: &str, p: usize) -> Option<usize> {
    let rest = &src[p..];
    ["build", "buy", "outsource", "market"].iter().find(|w| rest.starts_with(**w)).map(|w| p + w.len())
}

/// A character of a name word: `[A-Za-z0-9_()&]`, or `-` not followed by `>`.
fn name_char(b: &[u8], i: usize) -> bool {
    match b.get(i) {
        Some(c) if c.is_ascii_alphanumeric() || matches!(c, b'_' | b'(' | b')' | b'&') => true,
        Some(b'-') => b.get(i + 1) != Some(&b'>'),
        _ => false,
    }
}

/// `/(?!title\s|accTitle|accDescr)[A-Za-z](?:[A-Za-z0-9_()&]|-(?!>))*(?:[ \t]+[A-Za-z(](?:[A-Za-z0-9_()&]|-(?!>))*)*/y`.
/// Nothing follows the words, so the greedy reading is the match: each further word is taken
/// when blanks and a letter or `(` come next.
fn match_name(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let rest = &src[p..];
    if rest.starts_with("accTitle") || rest.starts_with("accDescr") {
        return None;
    }
    if rest.starts_with("title") && crate::util::chars::char_at(src, p + 5).is_some_and(crate::util::chars::is_space) {
        return None;
    }
    if !b.get(p).is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    let mut i = p + 1;
    while name_char(b, i) {
        i += 1;
    }
    loop {
        let mut j = i;
        while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
            j += 1;
        }
        if j == i || !b.get(j).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'(') {
            return Some(i);
        }
        i = j + 1;
        while name_char(b, i) {
            i += 1;
        }
    }
}

/// `/0|[1-9][0-9]*(?!\.)/y`. A run of digits that a dot follows gives back its last digit,
/// after which a digit follows instead.
fn match_int(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    match b.get(p) {
        Some(b'0') => Some(p + 1),
        Some(b'1'..=b'9') => {
            let end = digits(b, p + 1);
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

/// `/[\w]([-\w]*\w)?/y`
fn match_id(src: &str, p: usize) -> Option<usize> {
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

/// A word token: Pele's `word(name, text)`, a sticky expression of the text with no boundary.
macro_rules! word {
    ($name:literal, $text:literal) => {{
        fn matcher(src: &str, p: usize) -> Option<usize> {
            src[p..].starts_with($text).then_some(p + $text.len())
        }
        TokenType::func($name, matcher, Some(&$text[..1]))
    }};
}

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub static WARDLEY_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        TokenType::literal("[", "["),
        TokenType::literal(",", ","),
        TokenType::literal("]", "]"),
        TokenType::literal("@", "@"),
        TokenType::literal("/", "/"),
        TokenType::literal("(", "("),
        TokenType::literal(")", ")"),
        TokenType::literal("-", "-").with_longer(&[ARROW, LINK_ARROW, T_YAML]),
        TokenType::literal("{", "{"),
        TokenType::literal("}", "}"),
        TokenType {
            name: "WARDLEY_NUMBER",
            pattern: Pattern::Scanner(number_scanner),
            hidden: false,
            opener: None,
            first: Some("0123456789"),
            longer: &[],
        },
        TokenType::func("ARROW", match_arrow, Some("-")),
        TokenType::func("LINK_PORT", match_link_port, Some("+")),
        TokenType::func("LINK_ARROW", match_link_arrow, Some("->+")),
        TokenType::func("LINK_LABEL", match_link_label, Some(";")),
        TokenType::func("STRATEGY", match_strategy, Some("bom")),
        word!("KW_WARDLEY", "wardley-beta"),
        word!("KW_SIZE", "size"),
        word!("KW_EVOLUTION", "evolution"),
        word!("KW_ANCHOR", "anchor"),
        word!("KW_COMPONENT", "component"),
        word!("KW_LABEL", "label"),
        word!("KW_INERTIA", "inertia"),
        word!("KW_EVOLVE", "evolve"),
        word!("KW_PIPELINE", "pipeline"),
        word!("KW_NOTE", "note"),
        word!("KW_ANNOTATIONS", "annotations"),
        word!("KW_ANNOTATION", "annotation"),
        word!("KW_ACCELERATOR", "accelerator"),
        word!("KW_DEACCELERATOR", "deaccelerator"),
        TokenType::func("NAME_WITH_SPACES", match_name, Some(LETTERS)),
        TokenType::func("WS", match_whitespace, Some("\t ")).hidden(),
        acc_descr("ACC_DESCR"),
        ACC_TITLE,
        TITLE,
        TokenType::func("INT", match_int, Some("0123456789")),
        STRING,
        TokenType::func("ID", match_id, Some("_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")),
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
    ]
});

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAst {
    pub neg_x: bool,
    pub offset_x: f64,
    pub neg_y: bool,
    pub offset_y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageAst {
    pub name: String,
    pub boundary: Option<f64>,
    pub second_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentAst {
    pub name: String,
    pub visibility: f64,
    pub evolution: f64,
    pub label: Option<LabelAst>,
    pub strategy: Option<String>,
    pub inertia: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineComponentAst {
    pub name: String,
    pub evolution: f64,
    pub label: Option<LabelAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkAst {
    pub from: String,
    pub from_port: Option<String>,
    pub arrow: Option<String>,
    pub to: String,
    pub to_port: Option<String>,
    pub link_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnchorAst {
    pub name: String,
    pub visibility: f64,
    pub evolution: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvolveAst {
    pub component: String,
    pub target: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineAst {
    pub parent: String,
    pub components: Vec<PipelineComponentAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteAst {
    pub text: String,
    pub visibility: f64,
    pub evolution: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationAst {
    pub number: f64,
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// An accelerator or a deaccelerator.
#[derive(Debug, Clone, PartialEq)]
pub struct ForceAst {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WardleyAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub size: Option<(f64, f64)>,
    pub evolution: Option<Vec<StageAst>>,
    pub anchors: Vec<AnchorAst>,
    pub components: Vec<ComponentAst>,
    pub links: Vec<LinkAst>,
    pub evolves: Vec<EvolveAst>,
    pub pipelines: Vec<PipelineAst>,
    pub notes: Vec<NoteAst>,
    /// The positions given by `annotations` statements, as (x, y).
    pub annotations: Vec<(f64, f64)>,
    pub annotation: Vec<AnnotationAst>,
    pub accelerators: Vec<ForceAst>,
    pub deaccelerators: Vec<ForceAst>,
}

struct Parser<'a, 't> {
    r: Reader<'a, 't>,
}

impl<'a, 't> Parser<'a, 't> {
    fn at(&self, kind: usize) -> bool {
        self.r.kind() == kind
    }

    fn is_name(&self) -> bool {
        self.at(T_STRING) || self.at(ID) || self.at(NAME)
    }

    fn name(&mut self) -> Result<String, Error> {
        if self.at(T_STRING) {
            return Ok(string_value(self.r.take()));
        }
        if self.at(ID) || self.at(NAME) {
            return Ok(self.r.take().to_string());
        }
        Err(self.r.fail("a name"))
    }

    fn number(&mut self) -> Result<f64, Error> {
        Ok(js::number(self.r.expect(NUMBER)?))
    }

    fn int(&mut self) -> Result<f64, Error> {
        Ok(js::parse_int(self.r.expect(INT)?, 0))
    }

    fn coordinate(&mut self) -> Result<f64, Error> {
        if self.at(NUMBER) {
            Ok(js::number(self.r.take()))
        } else {
            self.int()
        }
    }

    fn end_of_line(&mut self) -> Result<(), Error> {
        if self.at(EOF) {
            return Ok(());
        }
        if !self.at(T_NEWLINE) {
            return Err(self.r.fail("a line break or the end of input"));
        }
        while self.at(T_NEWLINE) {
            self.r.i += 1;
        }
        Ok(())
    }

    fn pair(&mut self, read: fn(&mut Self) -> Result<f64, Error>) -> Result<(f64, f64), Error> {
        self.r.expect(LBRACKET)?;
        let first = read(self)?;
        self.r.expect(COMMA)?;
        let second = read(self)?;
        self.r.expect(RBRACKET)?;
        Ok((first, second))
    }

    fn label(&mut self) -> Result<LabelAst, Error> {
        self.r.expect(LABEL)?;
        self.r.expect(LBRACKET)?;
        let neg_x = self.r.accept(MINUS);
        let offset_x = self.int()?;
        self.r.expect(COMMA)?;
        let neg_y = self.r.accept(MINUS);
        let offset_y = self.int()?;
        self.r.expect(RBRACKET)?;
        Ok(LabelAst { neg_x, offset_x, neg_y, offset_y })
    }

    fn stage(&mut self) -> Result<StageAst, Error> {
        let mut out = StageAst { name: self.name()?, boundary: None, second_name: None };
        if self.r.accept(AT) {
            out.boundary = Some(self.number()?);
        }
        if self.r.accept(SLASH) {
            out.second_name = Some(self.name()?);
        }
        Ok(out)
    }
}

pub fn parse_wardley(src: &str) -> Result<WardleyAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&WARDLEY_TOKENS));
    let r = GRAMMAR.reader(src, DiagramType::Wardley)?;
    let mut p = Parser { r };
    let mut ast = WardleyAst::default();

    while p.at(T_NEWLINE) {
        p.r.i += 1;
    }
    p.r.expect(WARDLEY)?;
    while !p.at(EOF) {
        match p.r.kind() {
            T_NEWLINE => p.r.i += 1,
            T_ACC_DESCR => {
                ast.acc_descr = Some(acc_descr_value(p.r.take()));
                p.end_of_line()?;
            }
            T_ACC_TITLE => {
                ast.acc_title = Some(acc_title_value(p.r.take()));
                p.end_of_line()?;
            }
            T_TITLE => {
                ast.title = Some(title_value(p.r.take()));
                p.end_of_line()?;
            }
            SIZE => {
                p.r.i += 1;
                let size = p.pair(Parser::int)?;
                p.end_of_line()?;
                ast.size = Some(size);
            }
            EVOLUTION => {
                p.r.i += 1;
                let mut stages = vec![p.stage()?];
                p.r.expect(ARROW)?;
                stages.push(p.stage()?);
                while p.r.accept(ARROW) {
                    stages.push(p.stage()?);
                }
                p.end_of_line()?;
                ast.evolution = Some(stages);
            }
            ANCHOR => {
                p.r.i += 1;
                let name = p.name()?;
                let (visibility, evolution) = p.pair(Parser::number)?;
                p.end_of_line()?;
                ast.anchors.push(AnchorAst { name, visibility, evolution });
            }
            COMPONENT => {
                p.r.i += 1;
                let name = p.name()?;
                let (visibility, evolution) = p.pair(Parser::number)?;
                let mut component = ComponentAst { name, visibility, evolution, label: None, strategy: None, inertia: false };
                if p.at(LABEL) {
                    component.label = Some(p.label()?);
                }
                if p.at(LPAREN) && p.r.kind_at(p.r.i + 1) == STRATEGY {
                    p.r.i += 1;
                    component.strategy = Some(p.r.take().to_string());
                    p.r.expect(RPAREN)?;
                }
                if p.r.accept(INERTIA) {
                    component.inertia = true;
                } else if p.r.accept(LPAREN) {
                    p.r.expect(INERTIA)?;
                    p.r.expect(RPAREN)?;
                    component.inertia = true;
                }
                p.end_of_line()?;
                ast.components.push(component);
            }
            EVOLVE => {
                p.r.i += 1;
                let component = p.name()?;
                let target = p.number()?;
                p.end_of_line()?;
                ast.evolves.push(EvolveAst { component, target });
            }
            PIPELINE => {
                p.r.i += 1;
                let parent = p.name()?;
                p.r.expect(LBRACE)?;
                p.r.expect(T_NEWLINE)?;
                while p.at(T_NEWLINE) {
                    p.r.i += 1;
                }
                let mut components = Vec::new();
                loop {
                    p.r.expect(COMPONENT)?;
                    let name = p.name()?;
                    p.r.expect(LBRACKET)?;
                    let evolution = p.number()?;
                    p.r.expect(RBRACKET)?;
                    let mut component = PipelineComponentAst { name, evolution, label: None };
                    if p.at(LABEL) {
                        component.label = Some(p.label()?);
                    }
                    p.end_of_line()?;
                    components.push(component);
                    if !p.at(COMPONENT) {
                        break;
                    }
                }
                p.r.expect(RBRACE)?;
                p.end_of_line()?;
                ast.pipelines.push(PipelineAst { parent, components });
            }
            NOTE => {
                p.r.i += 1;
                let text = string_value(p.r.expect(T_STRING)?);
                let (visibility, evolution) = p.pair(Parser::number)?;
                p.end_of_line()?;
                ast.notes.push(NoteAst { text, visibility, evolution });
            }
            ANNOTATIONS => {
                p.r.i += 1;
                let at = p.pair(Parser::coordinate)?;
                p.end_of_line()?;
                ast.annotations.push(at);
            }
            ANNOTATION => {
                p.r.i += 1;
                let number = p.int()?;
                p.r.expect(COMMA)?;
                let (x, y) = p.pair(Parser::coordinate)?;
                let text = string_value(p.r.expect(T_STRING)?);
                p.end_of_line()?;
                ast.annotation.push(AnnotationAst { number, x, y, text });
            }
            ACCELERATOR | DEACCELERATOR => {
                let slow = p.at(DEACCELERATOR);
                p.r.i += 1;
                let name = p.name()?;
                let (x, y) = p.pair(Parser::number)?;
                p.end_of_line()?;
                if slow {
                    ast.deaccelerators.push(ForceAst { name, x, y });
                } else {
                    ast.accelerators.push(ForceAst { name, x, y });
                }
            }
            _ => {
                if !p.is_name() {
                    return Err(p.r.fail("a statement"));
                }
                let from = p.name()?;
                let mut link = LinkAst { from, from_port: None, arrow: None, to: String::new(), to_port: None, link_label: None };
                if p.at(LINK_PORT) {
                    link.from_port = Some(p.r.take().to_string());
                }
                if p.at(LINK_ARROW) || p.at(ARROW) {
                    link.arrow = Some(p.r.take().to_string());
                }
                link.to = p.name()?;
                if p.at(LINK_PORT) {
                    link.to_port = Some(p.r.take().to_string());
                }
                if p.at(LINK_LABEL) {
                    link.link_label = Some(js::trim(&p.r.take()[1..]).to_string());
                }
                p.end_of_line()?;
                ast.links.push(link);
            }
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("wardley", &super::WARDLEY_TOKENS, crate::DiagramType::Wardley);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matchers() {
        assert_eq!(match_int("12.5", 0), Some(1));
        assert_eq!(match_int("1.5", 0), None);
        assert_eq!(match_int("0.5", 0), Some(1));
        assert_eq!(match_int("125]", 0), Some(3));
        assert_eq!(match_name("Foo Bar  baz [0.1", 0), Some(12));
        assert_eq!(match_name("a-b->c", 0), Some(3));
        assert_eq!(match_name("title x", 0), None);
        assert_eq!(match_name("titles x", 0), Some(8));
        assert_eq!(match_name("x (y)", 0), Some(5));
        assert_eq!(match_name("x 1", 0), Some(1));
        assert_eq!(match_link_arrow("+'a b'<>x", 0), Some(8));
        assert_eq!(match_link_arrow("+'a'x", 0), None);
        assert_eq!(match_link_arrow("-.->", 0), Some(4));
        assert_eq!(match_link_label(";", 0), None);
        assert_eq!(match_link_label("; a\n", 0), Some(3));
        let mut s = number_scanner();
        assert_eq!(s("12.50x", 0), Some(5));
        assert_eq!(s("12.50x", 1), Some(5));
        let mut s = number_scanner();
        assert_eq!(s("12.", 0), None);
    }

    #[test]
    fn parses_a_map() {
        let ast = parse_wardley("wardley-beta\ntitle Tea\nanchor Business [0.95, 0.63]\ncomponent Cup [0.73, 0.78] label [-5, 10] (build) inertia\nBusiness -> Cup\n").unwrap();
        assert_eq!(ast.title.as_deref(), Some("Tea"));
        assert_eq!(ast.components[0].strategy.as_deref(), Some("build"));
        assert!(ast.components[0].inertia);
        assert_eq!(ast.components[0].label, Some(LabelAst { neg_x: true, offset_x: 5.0, neg_y: false, offset_y: 10.0 }));
        assert_eq!(ast.links[0].arrow.as_deref(), Some("->"));
    }
}
