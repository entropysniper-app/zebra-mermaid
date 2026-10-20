//! Internal storage and prepared tables for fixed diagram grammars. The public tokenizer
//! keeps its owned vectors and dynamic-table behavior; both paths share the matching loop.

use super::tokens::{tokenize_into, Starting, TokenSink, TokenType, Tokens, EOF};
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};

#[derive(Clone, Copy)]
struct Token {
    start: u32,
    end: u32,
    kind: u32,
}
const _: () = assert!(std::mem::size_of::<Token>() == 12);

enum Storage<'a> {
    Compact(Vec<Token>),
    Wide(Tokens<'a>),
}

struct ParsedTokens<'a> {
    src: &'a str,
    storage: Storage<'a>,
}

impl<'a> TokenSink<'a> for ParsedTokens<'a> {
    fn new(src: &'a str, types: usize) -> Self {
        let storage = if src.len() <= u32::MAX as usize && types < u32::MAX as usize {
            Storage::Compact(Vec::new())
        } else {
            Storage::Wide(<Tokens<'a> as TokenSink>::new(src, types))
        };
        Self { src, storage }
    }
    fn push(&mut self, kind: usize, start: usize, end: usize) {
        match &mut self.storage {
            Storage::Compact(tokens) => {
                tokens.push(Token { start: start as u32, end: end as u32, kind: if kind == EOF { u32::MAX } else { kind as u32 } })
            }
            Storage::Wide(tokens) => tokens.push(kind, start, end),
        }
    }
}

impl ParsedTokens<'_> {
    #[inline]
    fn kind(&self, index: usize) -> usize {
        match &self.storage {
            Storage::Compact(tokens) => {
                let kind = tokens[index].kind;
                if kind == u32::MAX {
                    EOF
                } else {
                    kind as usize
                }
            }
            Storage::Wide(tokens) => tokens.kinds[index],
        }
    }
    #[inline]
    fn kind_at(&self, index: usize) -> usize {
        match &self.storage {
            Storage::Compact(tokens) => tokens.get(index).map_or(EOF, |t| if t.kind == u32::MAX { EOF } else { t.kind as usize }),
            Storage::Wide(tokens) => tokens.kinds.get(index).copied().unwrap_or(EOF),
        }
    }
    #[inline]
    fn span(&self, index: usize) -> (usize, usize) {
        match &self.storage {
            Storage::Compact(tokens) => {
                let token = tokens[index];
                (token.start as usize, token.end as usize)
            }
            Storage::Wide(tokens) => (tokens.starts[index], tokens.ends[index]),
        }
    }
}

/// Only immutable grammar metadata is prepared. No source, token or render result is retained.
pub(crate) struct Grammar {
    types: &'static [TokenType],
    starting: Starting,
}

impl Grammar {
    pub fn new(types: &'static [TokenType]) -> Self {
        Self { types, starting: Starting::new(types) }
    }
    pub fn reader<'a>(&self, src: &'a str, diagram: DiagramType) -> Result<Reader<'a, 'static>, Error> {
        let tokens = tokenize_into(src, self.types, &self.starting, diagram)?;
        Ok(Reader { i: 0, tokens, types: self.types, diagram })
    }
}

pub(crate) struct Reader<'a, 't> {
    pub i: usize,
    tokens: ParsedTokens<'a>,
    pub types: &'t [TokenType],
    pub diagram: DiagramType,
}

impl<'a> Reader<'a, '_> {
    #[inline]
    pub fn kind(&self) -> usize {
        self.tokens.kind(self.i)
    }
    #[inline]
    pub fn kind_at(&self, index: usize) -> usize {
        self.tokens.kind_at(index)
    }
    pub fn start(&self) -> usize {
        self.tokens.span(self.i).0
    }
    pub fn text_at(&self, index: usize) -> &'a str {
        let (start, end) = self.tokens.span(index);
        &self.tokens.src[start..end]
    }
    pub fn text(&self) -> &'a str {
        self.text_at(self.i)
    }
    pub fn take(&mut self) -> &'a str {
        let text = self.text_at(self.i);
        self.i += 1;
        text
    }
    pub fn accept(&mut self, kind: usize) -> bool {
        if self.kind() != kind {
            return false;
        }
        self.i += 1;
        true
    }
    pub fn expect(&mut self, kind: usize) -> Result<&'a str, Error> {
        if self.kind() != kind {
            return Err(self.fail(&format!("token of type '{}'", self.types[kind].name)));
        }
        Ok(self.take())
    }
    pub fn fail(&self, expected: &str) -> Error {
        let found = if self.kind() == EOF { "end of input".to_string() } else { format!("`{}`", self.text()) };
        syntax_error(self.diagram, self.tokens.src, self.start(), &format!("Expecting {} but found {}.", expected, found), false)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::{json, Value};

    struct ForcedWide<'a>(ParsedTokens<'a>);
    impl<'a> TokenSink<'a> for ForcedWide<'a> {
        fn new(src: &'a str, types: usize) -> Self {
            Self(ParsedTokens { src, storage: Storage::Wide(<Tokens as TokenSink>::new(src, types)) })
        }
        fn push(&mut self, kind: usize, start: usize, end: usize) {
            self.0.push(kind, start, end);
        }
    }

    fn failure(error: &Error) -> Value {
        json!({"ok":false,"code":error.code.as_str(),"type":error.diagram_type.map(DiagramType::as_str),
            "line":error.line,"column":error.column,"snippet":error.snippet,"message":error.message})
    }
    fn token(src: &str, kind: usize, start: usize, end: usize) -> Value {
        json!([
            if kind == EOF { -1 } else { kind as i64 },
            src[..start].encode_utf16().count(),
            src[..end].encode_utf16().count(),
            &src[start..end]
        ])
    }

    pub(crate) fn check_table(name: &str, types: &'static [TokenType], diagram: DiagramType) {
        let fixture: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../../../../../tests/fixtures/unit/token-storage-reference.json.gz")[..],
        ))
        .unwrap();
        assert_eq!(fixture["upstreamCommit"], crate::PELE_COMMIT);
        assert_eq!(fixture["seed"], "0x70c3a9");
        assert_eq!(fixture["mutationsPerTable"], 96);
        assert_eq!(fixture["tables"].as_array().unwrap().len(), 15);
        assert_eq!(fixture["count"], 2008);
        let table = fixture["tables"].as_array().unwrap().iter().find(|t| t["name"] == name).unwrap();
        assert!(table["documentationSources"].as_u64().unwrap() > 0);
        assert_eq!(serde_json::to_value(types.iter().map(|t| t.name).collect::<Vec<_>>()).unwrap(), table["tokenNames"]);
        let grammar = Grammar::new(types);
        for row in table["cases"].as_array().unwrap() {
            let src = row["source"].as_str().unwrap();
            let expected = &row["expect"];
            let public = super::super::tokens::tokenize(src, types, diagram);
            let compact = grammar.reader(src, diagram);
            let wide =
                tokenize_into::<ForcedWide>(src, types, &grammar.starting, diagram).map(|t| Reader { i: 0, tokens: t.0, types, diagram });
            let label = format!("{name}: {}", row["id"]);
            match public {
                Ok(tokens) => {
                    assert_eq!(expected["ok"], true, "public {label}");
                    let actual: Vec<_> =
                        (0..tokens.kinds.len()).map(|i| token(src, tokens.kinds[i], tokens.starts[i], tokens.ends[i])).collect();
                    assert_eq!(serde_json::to_value(actual).unwrap(), expected["tokens"], "public {label}");
                    let mut reader = super::super::tokens::Reader::new(tokens, types, diagram);
                    for record in expected["readerErrors"].as_array().unwrap() {
                        reader.i = record["index"].as_u64().unwrap() as usize;
                        assert_eq!(failure(&reader.fail("token probe")), record["error"], "public reader {label}");
                    }
                }
                Err(error) => assert_eq!(failure(&error), *expected, "public {label}"),
            }
            for (mode, result) in [("compact", compact), ("wide", wide)] {
                match result {
                    Ok(mut reader) => {
                        assert_eq!(expected["ok"], true, "{mode} {label}");
                        let actual: Vec<_> = (0..expected["tokens"].as_array().unwrap().len())
                            .map(|i| {
                                let (start, end) = reader.tokens.span(i);
                                token(src, reader.kind_at(i), start, end)
                            })
                            .collect();
                        assert_eq!(serde_json::to_value(actual).unwrap(), expected["tokens"], "{mode} {label}");
                        assert_eq!(reader.kind_at(usize::MAX), EOF);
                        for record in expected["readerErrors"].as_array().unwrap() {
                            reader.i = record["index"].as_u64().unwrap() as usize;
                            assert_eq!(failure(&reader.fail("token probe")), record["error"], "{mode} reader {label}");
                        }
                        reader.i = 0;
                        for record in expected["tokens"].as_array().unwrap() {
                            let kind = record[0].as_i64().unwrap();
                            let kind = if kind < 0 { EOF } else { kind as usize };
                            assert_eq!(reader.kind(), kind);
                            assert_eq!(reader.text(), record[3].as_str().unwrap());
                            assert_eq!(reader.expect(kind).unwrap(), record[3].as_str().unwrap());
                        }
                    }
                    Err(error) => assert_eq!(failure(&error), *expected, "{mode} {label}"),
                }
            }
        }
    }

    #[test]
    fn wide_storage_preserves_native_kind_range_and_eof() {
        let mut tokens = ParsedTokens::new("", u32::MAX as usize);
        assert!(matches!(tokens.storage, Storage::Wide(_)));
        tokens.push(usize::MAX - 1, 0, 0);
        tokens.push(EOF, 0, 0);
        assert_eq!(tokens.kind(0), usize::MAX - 1);
        assert_eq!(tokens.kind(1), EOF);
        assert_eq!(tokens.kind_at(2), EOF);
        assert_eq!(tokens.span(0), (0, 0));
    }

    #[test]
    fn compact_storage_preserves_last_non_sentinel_kind_and_offsets() {
        let mut tokens = ParsedTokens::new("", 0);
        assert!(matches!(tokens.storage, Storage::Compact(_)));
        tokens.push((u32::MAX - 1) as usize, u32::MAX as usize, u32::MAX as usize);
        tokens.push(EOF, 0, 0);
        assert_eq!(tokens.kind(0), (u32::MAX - 1) as usize);
        assert_eq!(tokens.span(0), (u32::MAX as usize, u32::MAX as usize));
        assert_eq!(tokens.kind(1), EOF);
    }
}
