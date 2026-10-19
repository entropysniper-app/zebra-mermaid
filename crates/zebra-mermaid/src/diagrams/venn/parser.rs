//! Venn parser. Ported from Pele `src/diagrams/venn/parser.ts`, which ports Mermaid's grammar.
//!
//! A token is read only when the grammar has to look at one to decide, because the lexer asks
//! the model whether an indented `text` continues a set, and the statements are what tell the
//! model so.

use super::db::VennDb;
use super::lexer::{VennLexer, T};
use super::VennModel;
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

const STATEMENT: &str = "'EOF', 'NEWLINE', 'TITLE', 'SET', 'UNION', 'TEXT', 'INDENT_TEXT', 'STYLE'";
const VALUE: &str = "'NUMERIC', 'IDENTIFIER', 'STRING', 'HEXCOLOR', 'RGBCOLOR', 'RGBACOLOR'";

struct Parser<'a, 'm> {
    src: &'a str,
    lexer: VennLexer<'a>,
    db: VennDb<'m>,
    /// The token read ahead and not taken yet.
    kind: Option<T>,
}

impl<'a> Parser<'a, '_> {
    fn peek(&mut self) -> T {
        match self.kind {
            Some(kind) => kind,
            None => {
                let kind = self.lexer.next(&mut self.db);
                self.kind = Some(kind);
                kind
            }
        }
    }

    fn fail(&mut self, expected: &str) -> Error {
        let kind = self.peek();
        if kind == T::Error {
            return syntax_error(DiagramType::Venn, self.src, self.lexer.start, "", true);
        }
        syntax_error(DiagramType::Venn, self.src, self.lexer.start, &format!("Expecting {}, got '{}'", expected, kind.name()), false)
    }

    fn take(&mut self) -> &'a str {
        self.kind = None;
        self.lexer.text
    }

    fn expect(&mut self, wanted: T) -> Result<&'a str, Error> {
        if self.peek() != wanted {
            return Err(self.fail(&format!("'{}'", wanted.name())));
        }
        Ok(self.take())
    }

    fn identifier(&mut self) -> Result<String, Error> {
        let kind = self.peek();
        if kind != T::Identifier && kind != T::String {
            return Err(self.fail("'IDENTIFIER', 'STRING'"));
        }
        Ok(self.take().to_string())
    }

    fn identifiers(&mut self) -> Result<Vec<String>, Error> {
        let mut list = vec![self.identifier()?];
        while self.peek() == T::Comma {
            self.take();
            list.push(self.identifier()?);
        }
        Ok(list)
    }

    fn subset(&mut self, ids: Vec<String>, union: bool) -> Result<(), Error> {
        let label = if self.peek() == T::BracketLabel { Some(self.take()) } else { None };
        let mut size = None;
        if self.peek() == T::Colon {
            self.take();
            size = Some(js::parse_float(self.expect(T::Numeric)?));
        }
        if union {
            if ids.len() < 2 {
                return Err(Error::semantic("union requires multiple identifiers").with_type(DiagramType::Venn));
            }
            self.db.validate_union_identifiers(&ids)?;
        }
        self.db.add_subset_data(&ids, label, size);
        self.db.set_indent_mode(true);
        Ok(())
    }

    /// The id of a text, and the label in brackets that an id which is not a number may carry.
    /// `sets` is `None` for an indented text, which belongs to the set or union before it.
    fn text(&mut self, sets: Option<Vec<String>>) -> Result<(), Error> {
        let kind = self.peek();
        if kind != T::Identifier && kind != T::String && kind != T::Numeric {
            return Err(self.fail("'NUMERIC', 'IDENTIFIER', 'STRING'"));
        }
        let id = self.take();
        let label = if kind != T::Numeric && self.peek() == T::BracketLabel { Some(self.take()) } else { None };
        let sets = match sets {
            Some(sets) => sets,
            None => match self.db.current_sets() {
                Some(current) => current.clone(),
                None => return Err(Error::semantic("text requires set").with_type(DiagramType::Venn)),
            },
        };
        self.db.add_text_data(&sets, id, label);
        Ok(())
    }

    fn style(&mut self) -> Result<(), Error> {
        let ids = self.identifiers()?;
        let mut styles: Vec<(String, String)> = Vec::new();
        loop {
            let key = self.expect(T::Identifier)?.to_string();
            self.expect(T::Colon)?;
            let value = if self.peek() == T::String {
                self.take().to_string()
            } else {
                let mut parts: Vec<&str> = Vec::new();
                loop {
                    let kind = self.peek();
                    if kind != T::Identifier && !(T::Numeric..=T::Rgbacolor).contains(&kind) {
                        break;
                    }
                    parts.push(self.take());
                }
                if parts.is_empty() {
                    return Err(self.fail(VALUE));
                }
                parts.join(" ")
            };
            styles.push((key, value));
            if self.peek() != T::Comma {
                break;
            }
            self.take();
        }
        self.db.add_style_data(&ids, styles);
        Ok(())
    }
}

/// Reads a venn diagram into `model`.
pub fn parse_venn(src: &str, model: &mut VennModel) -> Result<(), Error> {
    let mut p = Parser { src, lexer: VennLexer::new(src), db: VennDb::new(model), kind: None };
    while p.peek() == T::Newline {
        p.take();
    }
    p.expect(T::Venn)?;
    loop {
        let statement = p.peek();
        if statement == T::Eof {
            return Ok(());
        }
        if statement < T::Newline || statement > T::Style || statement == T::Venn {
            return Err(p.fail(STATEMENT));
        }
        let text = p.take();
        match statement {
            T::Title => p.db.set_diagram_title(js::slice16(text, 6, js::len16(text)).to_string()),
            T::Set => {
                let id = p.identifier()?;
                p.subset(vec![id], false)?;
            }
            T::Union => {
                let ids = p.identifiers()?;
                p.subset(ids, true)?;
            }
            T::Text => {
                let ids = p.identifiers()?;
                p.text(Some(ids))?;
            }
            T::IndentText => p.text(None)?,
            T::Style => p.style()?,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<VennModel, Error> {
        let mut model = VennModel::default();
        parse_venn(src, &mut model).map(|_| model)
    }

    #[test]
    fn sets_unions_and_indented_texts() {
        let m = parse("venn-beta\n  title Pets\n  set A[\"Cats\"]:20\n  set B\n    text b1[\"Rex\"]\n  union B, A\n    text 5\n").unwrap();
        assert_eq!(m.title.as_deref(), Some("Pets"));
        assert_eq!(m.subsets.len(), 3);
        assert_eq!(m.subsets[0].label.as_deref(), Some("Cats"));
        assert_eq!(m.subsets[0].size, 20.0);
        assert_eq!(m.subsets[2].sets, ["A", "B"]);
        assert_eq!(m.subsets[2].size, 2.5);
        assert_eq!(m.text_nodes[0].sets, ["B"]);
        assert_eq!(m.text_nodes[0].label.as_deref(), Some("Rex"));
        assert_eq!(m.text_nodes[1].sets, ["A", "B"]);
        assert_eq!(m.text_nodes[1].id, "5");
    }

    #[test]
    fn styles_join_their_parts() {
        let m = parse("venn-beta\nset A\nstyle A fill: #ff0000, stroke-width: 2 px, color: \"red\"\n").unwrap();
        let s = &m.styles[0].styles;
        assert_eq!(s["fill"], "#ff0000");
        assert_eq!(s["stroke-width"], "2 px");
        assert_eq!(s["color"], "red");
    }

    #[test]
    fn errors() {
        assert_eq!(parse("venn-beta\nunion A, B\n").unwrap_err().message, "unknown set identifier: A, B");
        assert_eq!(parse("venn-beta\nset A\nunion A\n").unwrap_err().message, "union requires multiple identifiers");
        assert_eq!(parse("venn-beta\nset A ?\n").unwrap_err().message, "Lexical error on line 2. Unrecognized text.\nset A ?\n------^");
    }
}
