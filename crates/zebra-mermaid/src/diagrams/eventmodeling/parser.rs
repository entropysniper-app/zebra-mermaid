//! Event modeling parser. Ported from Pele `src/diagrams/eventmodeling/parser.ts`.

use crate::diagrams::common::parse_tokens::{Grammar, Reader};
use std::sync::LazyLock;

use crate::detect::DiagramType;
use crate::diagrams::common::accdescr::acc_descr;
use crate::diagrams::common::tokens::{
    blanks, keyword_at, newline_at, TokenType, ACC_TITLE, DIRECTIVE, EOF, SINGLE_LINE_COMMENT, TITLE, YAML,
};
use crate::error::Error;
use crate::util::chars::{char_at, is_line_end, is_space};

// Indices into the token table. `word()` tokens take EM_ID as their longer alternative, as
// Langium gives every keyword that is also an identifier.
const KEYWORD: usize = 0;
const RESETFRAME: usize = 1;
const READMODEL: usize = 2;
const PROCESSOR: usize = 3;
const TIMEFRAME: usize = 4;
const COMMAND: usize = 5;
const ENTITY: usize = 6;
const EVENT: usize = 7;
const JSOBJ: usize = 8;
const FIGMA: usize = 9;
const GIVEN: usize = 10;
const JSON: usize = 11;
const SALT: usize = 12;
const HTML: usize = 13;
const TEXT: usize = 14;
const DATA: usize = 15;
const NOTE: usize = 16;
const WHEN: usize = 17;
const THEN: usize = 18;
const RMO: usize = 19;
const CMD: usize = 20;
const EVT: usize = 21;
const PCR: usize = 22;
const URI: usize = 23;
const ARROW: usize = 24;
const GWT: usize = 25;
const UI: usize = 26;
const MD: usize = 27;
const TF: usize = 28;
const OPEN: usize = 29;
const CLOSE: usize = 30;
const RF: usize = 31;
const TICK: usize = 32;
const DOT: usize = 33;
const ID: usize = 34;
const FRAME_ID: usize = 35;
const INLINE: usize = 36;
const BLOCK: usize = 37;
const T_ACC_DESCR: usize = 38;
const T_ACC_TITLE: usize = 39;
const T_TITLE: usize = 40;

const LONGER: &[usize] = &[ID];

const fn word(text: &'static str) -> TokenType {
    TokenType::literal(text, text).with_longer(LONGER)
}

/// `/[_a-zA-Z][\w_]*/y`
fn match_id(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    if !b.get(p).is_some_and(|&c| c == b'_' || c.is_ascii_alphabetic()) {
        return None;
    }
    let mut q = p + 1;
    while q < b.len() && (b[q] == b'_' || b[q].is_ascii_alphanumeric()) {
        q += 1;
    }
    Some(q)
}

/// `/\d{1,3}/y`
fn match_frame_id(src: &str, p: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut q = p;
    while q < b.len() && q - p < 3 && b[q].is_ascii_digit() {
        q += 1;
    }
    (q > p).then_some(q)
}

/// The end of the line that starts at `from`: the first line terminator, which `.` does not
/// match.
fn line_end(src: &str, from: usize) -> usize {
    let mut q = from;
    while let Some(c) = char_at(src, q) {
        if is_line_end(c) {
            break;
        }
        q += c.len_utf8();
    }
    q
}

/// `/\{(.*)\}|"(.*)"|'(.*)'/y`: the greedy `.*` backs off to the last closer on the line.
fn match_data_inline(src: &str, p: usize) -> Option<usize> {
    let close = match src.as_bytes().get(p) {
        Some(b'{') => '}',
        Some(b'"') => '"',
        Some(b'\'') => '\'',
        _ => return None,
    };
    let end = line_end(src, p + 1);
    src[p + 1..end].rfind(close).map(|k| p + 1 + k + 1)
}

/// `(?:\r?\n|(?!\S))` at `p`.
fn line_end_or_space(src: &str, p: usize) -> Option<usize> {
    if let Some(e) = newline_at(src, p) {
        return Some(e);
    }
    if char_at(src, p).is_none_or(is_space) {
        Some(p)
    } else {
        None
    }
}

/// `/\{[\t ]*\r?\n(?:[\S\s]*?\r?\n)?\}(?:\r?\n|(?!\S))/y`. The optional group is tried first,
/// and its lazy body makes the nearest closing brace after a line break win.
fn match_data_block(src: &str, p: usize) -> Option<usize> {
    if src.as_bytes().get(p) != Some(&b'{') {
        return None;
    }
    let q = blanks(src, p + 1);
    let q = if src[q..].starts_with('\r') { q + 1 } else { q };
    if !src[q..].starts_with('\n') {
        return None;
    }
    let s = q + 1;
    let mut from = s;
    while let Some(k) = src[from..].find('\n') {
        let f = from + k + 1;
        if src[f..].starts_with('}') {
            if let Some(e) = line_end_or_space(src, f + 1) {
                return Some(e);
            }
        }
        from = f;
    }
    if src[s..].starts_with('}') {
        return line_end_or_space(src, s + 1);
    }
    None
}

/// `/\s+/y`
fn match_ws(src: &str, p: usize) -> Option<usize> {
    let mut q = p;
    while let Some(c) = char_at(src, q) {
        if !is_space(c) {
            break;
        }
        q += c.len_utf8();
    }
    (q > p).then_some(q)
}

/// `/\/\*[\s\S]*?\*\//y`
fn match_ml_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("/*") {
        return None;
    }
    src[p + 2..].find("*/").map(|k| p + 2 + k + 2)
}

/// `/\/\/[^\n\r]*/y`
fn match_sl_comment(src: &str, p: usize) -> Option<usize> {
    if !src[p..].starts_with("//") {
        return None;
    }
    Some(src[p + 2..].find(['\n', '\r']).map_or(src.len(), |k| p + 2 + k))
}

fn match_keyword(src: &str, p: usize) -> Option<usize> {
    keyword_at(src, p, "eventmodeling")
}

pub static EVENTMODELING_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        TokenType::func("eventmodeling", match_keyword, Some("e")).with_longer(LONGER),
        word("resetframe"),
        word("readmodel"),
        word("processor"),
        word("timeframe"),
        word("command"),
        word("entity"),
        word("event"),
        word("jsobj"),
        word("figma"),
        word("given"),
        word("json"),
        word("salt"),
        word("html"),
        word("text"),
        word("data"),
        word("note"),
        word("when"),
        word("then"),
        word("rmo"),
        word("cmd"),
        word("evt"),
        word("pcr"),
        word("uri"),
        TokenType::literal("->>", "->>"),
        word("gwt"),
        word("ui"),
        word("md"),
        word("tf"),
        TokenType::literal("[[", "[["),
        TokenType::literal("]]", "]]"),
        word("rf"),
        TokenType::literal("`", "`"),
        TokenType::literal(".", "."),
        TokenType::func("EM_ID", match_id, None),
        TokenType::func("EM_FID", match_frame_id, None),
        TokenType::func("EM_DATA_INLINE", match_data_inline, None),
        TokenType::func("EM_DATA_BLOCK", match_data_block, None),
        acc_descr("EM_ACC_DESCR"),
        TokenType { name: "EM_ACC_TITLE", ..ACC_TITLE },
        TokenType { name: "EM_TITLE", ..TITLE },
        TokenType::func("EM_WS", match_ws, None).hidden(),
        TokenType { name: "EM_YAML", ..YAML },
        TokenType { name: "EM_DIRECTIVE", ..DIRECTIVE },
        TokenType { name: "EM_SINGLE_LINE_COMMENT", ..SINGLE_LINE_COMMENT },
        TokenType::func("EM_ML_COMMENT", match_ml_comment, None).hidden(),
        TokenType::func("EM_SL_COMMENT", match_sl_comment, None).hidden(),
    ]
});

fn is_time_frame(kind: usize) -> bool {
    kind == TF || kind == TIMEFRAME
}

fn is_reset_frame(kind: usize) -> bool {
    kind == RF || kind == RESETFRAME
}

fn is_entity_type(kind: usize) -> bool {
    matches!(kind, RMO | READMODEL | UI | CMD | COMMAND | EVT | EVENT | PCR | PROCESSOR)
}

fn is_data_type(kind: usize) -> bool {
    matches!(kind, JSON | JSOBJ | FIGMA | SALT | URI | MD | HTML | TEXT)
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmFrame {
    pub reset: bool,
    pub name: String,
    pub model_entity_type: String,
    pub entity_identifier: String,
    pub source_frames: Vec<String>,
    pub data_reference: Option<String>,
    pub data_type: Option<String>,
    pub data_inline_value: Option<String>,
    /// Offset of the frame in the source, for error messages.
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmDataEntity {
    pub name: String,
    pub data_type: Option<String>,
    pub data_block_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmNoteEntity {
    pub source_frame: String,
    pub data_type: Option<String>,
    pub data_block_value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EmGwt {
    pub source_frame: String,
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventModelAst {
    /// The whole token, keyword included: the grammar's own terminals get no value conversion.
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub model_entities: Vec<String>,
    pub frames: Vec<EmFrame>,
    pub data_entities: Vec<EmDataEntity>,
    pub note_entities: Vec<EmNoteEntity>,
    pub gwt_entities: Vec<EmGwt>,
}

pub fn parse_event_model(src: &str) -> Result<EventModelAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&EVENTMODELING_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Eventmodeling)?;
    let mut ast = EventModelAst::default();

    fn one<'a>(r: &mut Reader<'a, '_>, test: fn(usize) -> bool, expected: &str) -> Result<&'a str, Error> {
        if test(r.kind()) {
            Ok(r.take())
        } else {
            Err(r.fail(expected))
        }
    }
    fn qualified_name(r: &mut Reader) -> Result<String, Error> {
        let mut name = r.expect(ID)?.to_string();
        while r.accept(DOT) {
            name.push('.');
            name.push_str(r.expect(ID)?);
        }
        Ok(name)
    }
    fn data_type(r: &mut Reader) -> Result<Option<String>, Error> {
        if !r.accept(TICK) {
            return Ok(None);
        }
        let ty = one(r, is_data_type, "a data type")?.to_string();
        r.expect(TICK)?;
        Ok(Some(ty))
    }
    fn statements(r: &mut Reader) -> Result<Vec<String>, Error> {
        let mut out = Vec::new();
        loop {
            one(r, is_entity_type, "an entity type")?;
            out.push(r.expect(ID)?.to_string());
            if !is_entity_type(r.kind()) {
                break;
            }
        }
        Ok(out)
    }

    r.expect(KEYWORD)?;
    while r.kind() != EOF {
        let kind = r.kind();
        if kind == T_ACC_DESCR {
            ast.acc_descr = Some(r.take().to_string());
        } else if kind == T_ACC_TITLE {
            ast.acc_title = Some(r.take().to_string());
        } else if kind == T_TITLE {
            ast.title = Some(r.take().to_string());
        } else if kind == ENTITY {
            r.i += 1;
            let name = qualified_name(&mut r)?;
            ast.model_entities.push(name);
        } else if is_time_frame(kind) || is_reset_frame(kind) {
            let offset = r.start();
            r.i += 1;
            let name = r.expect(FRAME_ID)?.to_string();
            let model_entity_type = one(&mut r, is_entity_type, "an entity type")?.to_string();
            let entity_identifier = qualified_name(&mut r)?;
            let mut frame = EmFrame {
                reset: !is_time_frame(kind),
                name,
                model_entity_type,
                entity_identifier,
                source_frames: Vec::new(),
                data_reference: None,
                data_type: None,
                data_inline_value: None,
                offset,
            };
            while r.accept(ARROW) {
                frame.source_frames.push(r.expect(FRAME_ID)?.to_string());
            }
            if r.accept(OPEN) {
                frame.data_reference = Some(r.expect(ID)?.to_string());
                r.expect(CLOSE)?;
            }
            if r.kind() == TICK || r.kind() == INLINE {
                frame.data_type = data_type(&mut r)?;
                frame.data_inline_value = Some(r.expect(INLINE)?.to_string());
            }
            ast.frames.push(frame);
        } else if kind == DATA {
            r.i += 1;
            let name = r.expect(ID)?.to_string();
            let data_type = data_type(&mut r)?;
            let data_block_value = r.expect(BLOCK)?.to_string();
            ast.data_entities.push(EmDataEntity { name, data_type, data_block_value });
        } else if kind == NOTE {
            r.i += 1;
            let source_frame = r.expect(FRAME_ID)?.to_string();
            let data_type = data_type(&mut r)?;
            let data_block_value = r.expect(BLOCK)?.to_string();
            ast.note_entities.push(EmNoteEntity { source_frame, data_type, data_block_value });
        } else if kind == GWT {
            r.i += 1;
            let mut gwt = EmGwt { source_frame: r.expect(FRAME_ID)?.to_string(), ..Default::default() };
            r.expect(GIVEN)?;
            gwt.given = statements(&mut r)?;
            if r.accept(WHEN) {
                gwt.when = statements(&mut r)?;
            }
            r.expect(THEN)?;
            gwt.then = statements(&mut r)?;
            ast.gwt_entities.push(gwt);
        } else {
            return Err(r.fail("a frame, an entity, a data block, a note, or a specification"));
        }
    }
    Ok(ast)
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table(
            "eventmodeling",
            &super::EVENTMODELING_TOKENS,
            crate::DiagramType::Eventmodeling,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_patterns() {
        assert_eq!(match_data_inline("{a}b}c\n}", 0), Some(5));
        assert_eq!(match_data_inline("\"a\" \"b\"\n\"", 0), Some(7));
        assert_eq!(match_data_inline("{a\n}", 0), None);
        assert_eq!(match_data_block("{\n a\n}\nx", 0), Some(7));
        assert_eq!(match_data_block("{ \r\n}x\n}", 0), Some(8));
        assert_eq!(match_data_block("{\n}", 0), Some(3));
        assert_eq!(match_data_block("{\n a\n}x", 0), None);
        assert_eq!(match_data_block("{x\n}", 0), None);
        assert_eq!(match_data_block("{\r\r\n}", 0), None);
        assert_eq!(match_frame_id("12345", 0), Some(3));
        assert_eq!(match_ml_comment("/* a */ b", 0), Some(7));
        assert_eq!(match_sl_comment("// a\nb", 0), Some(4));
    }

    #[test]
    fn frames() {
        let ast = parse_event_model("eventmodeling\ntf 01 cmd Shop.Add ->> 02 [[D]] `json` {\"a\": 1}\n").unwrap();
        let f = &ast.frames[0];
        assert_eq!(f.entity_identifier, "Shop.Add");
        assert_eq!(f.source_frames, ["02"]);
        assert_eq!(f.data_reference.as_deref(), Some("D"));
        assert_eq!(f.data_type.as_deref(), Some("json"));
        assert_eq!(f.data_inline_value.as_deref(), Some("{\"a\": 1}"));
    }
}
