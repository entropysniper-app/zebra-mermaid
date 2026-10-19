//! The quadrant chart parser. Ported from Pele `src/diagrams/quadrant/parser.ts`, a recursive
//! descent reading of Mermaid's Jison grammar over the tokens of [`super::lexer`].

use super::db::QuadrantDb;
use super::lexer::tokenize;
use super::tokens::*;
use super::QuadrantText;
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

/// A type no token has: what Pele reads past the last token.
const NONE: u8 = u8::MAX;

/// Tokens that join into a text.
fn is_word(t: u8) -> bool {
    matches!(t, PUNCTUATION | AMP | NUM | ALPHA | COMMA | PLUS | EQUALS | MULT | DOT | BRKT | UNDERSCORE | UNICODE_TEXT)
}

/// Tokens of a class name in `classDef`.
fn is_id(t: u8) -> bool {
    matches!(t, ALPHA | NUM | MINUS | COMMA | COLON | AMP | BRKT | MULT | UNICODE_TEXT)
}

/// Tokens of a style.
fn is_style(t: u8) -> bool {
    matches!(t, ALPHA | NUM | COLON | SPACE | BRKT | MINUS)
}

fn is_eol(t: u8) -> bool {
    matches!(t, NEWLINE | SEMI | EOF)
}

fn is_text(t: u8) -> bool {
    t == STR || t == MD_STR || is_word(t)
}

const TEXT_TOKENS: &str = "'STR', 'MD_STR', 'ALPHA', 'NUM', 'UNICODE_TEXT', 'PUNCTUATION'";

struct Parser<'a> {
    src: &'a str,
    types: Vec<u8>,
    starts: Vec<usize>,
    ends: Vec<usize>,
    i: usize,
}

impl<'a> Parser<'a> {
    fn kind(&self) -> u8 {
        self.types.get(self.i).copied().unwrap_or(NONE)
    }

    fn text_at(&self, i: usize) -> &'a str {
        &self.src[self.starts[i]..self.ends[i]]
    }

    /// The text of the current token, and moves past it.
    fn take(&mut self) -> &'a str {
        self.i += 1;
        self.text_at(self.i - 1)
    }

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.starts.get(self.i).copied().unwrap_or(self.src.len());
        if t == ERROR {
            return syntax_error(DiagramType::QuadrantChart, self.src, at, "", true);
        }
        let name = TOKEN_NAMES.get(t as usize).copied().unwrap_or("undefined");
        syntax_error(DiagramType::QuadrantChart, self.src, at, &format!("Expecting {}, got '{}'", expected, name), false)
    }

    fn expect(&mut self, t: u8) -> Result<&'a str, Error> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", TOKEN_NAMES[t as usize])));
        }
        Ok(self.take())
    }

    /// A text: its first token, and the words, spaces and dashes after it.
    fn text(&mut self) -> Result<QuadrantText, Error> {
        let first = self.kind();
        if !is_text(first) {
            return Err(self.fail(TEXT_TOKENS));
        }
        let mut s = self.take().to_string();
        while is_word(self.kind()) || self.kind() == SPACE || self.kind() == MINUS {
            s.push_str(self.take());
        }
        Ok(QuadrantText { text: s, markdown: first == MD_STR })
    }

    fn styles(&mut self) -> Result<Vec<String>, Error> {
        let mut out = Vec::new();
        loop {
            if !is_style(self.kind()) {
                return Err(self.fail("'ALPHA', 'NUM', 'COLON', 'SPACE', 'BRKT', 'MINUS'"));
            }
            let mut s = self.take().to_string();
            while is_style(self.kind()) {
                s.push_str(self.take());
            }
            out.push(js::trim(&s).to_string());
            if self.kind() != COMMA {
                return Ok(out);
            }
            self.i += 1;
        }
    }
}

pub(crate) fn parse_quadrant(src: &str, db: &mut QuadrantDb) -> Result<(), Error> {
    let tokens = tokenize(src);
    let mut r = Parser { src, types: tokens.types, starts: tokens.starts, ends: tokens.ends, i: 0 };

    while is_eol(r.kind()) || r.kind() == SPACE {
        r.i += 1;
    }
    r.expect(QUADRANT)?;

    // Pele stops at the end token. Past the last token nothing is left to read either; only an
    // error token ends the list without one, and it fails first.
    while r.kind() != END && r.kind() != NONE {
        while r.kind() == SPACE {
            r.i += 1;
        }
        let t = r.kind();
        match t {
            CLASSDEF => {
                r.i += 1;
                r.expect(SPACE)?;
                if !is_id(r.kind()) {
                    return Err(r.fail("'ALPHA', 'NUM', 'UNICODE_TEXT'"));
                }
                let mut id = r.take().to_string();
                while is_id(r.kind()) {
                    id.push_str(r.take());
                }
                r.expect(SPACE)?;
                let styles = r.styles()?;
                db.add_class(id, &styles)?;
            }
            X_AXIS | Y_AXIS => {
                r.i += 1;
                let mut from = r.text()?;
                let mut to = None;
                if r.kind() == DELIMITER {
                    r.i += 1;
                    if is_text(r.kind()) {
                        to = Some(r.text()?);
                    } else {
                        from.text.push_str(" ⟶ ");
                    }
                }
                if t == X_AXIS {
                    db.set_x_axis_left_text(from);
                    if let Some(to) = to {
                        db.set_x_axis_right_text(to);
                    }
                } else {
                    db.set_y_axis_bottom_text(from);
                    if let Some(to) = to {
                        db.set_y_axis_top_text(to);
                    }
                }
            }
            QUADRANT_1 | QUADRANT_2 | QUADRANT_3 | QUADRANT_4 => {
                r.i += 1;
                let text = r.text()?;
                db.set_quadrant_text((t - QUADRANT_1) as usize, text);
            }
            TITLE => {
                r.i += 1;
                let value = r.expect(TITLE_VALUE)?;
                db.set_diagram_title(js::trim(value));
            }
            ACC_TITLE => {
                r.i += 1;
                let value = r.expect(ACC_TITLE_VALUE)?;
                db.set_acc_title(js::trim(value));
            }
            ACC_DESCR => {
                r.i += 1;
                let value = r.expect(ACC_DESCR_VALUE)?;
                db.set_acc_description(js::trim(value));
            }
            ACC_DESCR_MULTILINE_VALUE => {
                let value = r.take();
                db.set_acc_description(js::trim(value));
            }
            _ => {
                if is_text(t) {
                    let label = r.text()?;
                    let class_name = if r.kind() == CLASS_NAME { r.take().to_string() } else { String::new() };
                    r.expect(POINT_START)?;
                    let x = r.expect(POINT_X)?;
                    let y = r.expect(POINT_Y)?;
                    let styles = if is_style(r.kind()) { r.styles()? } else { Vec::new() };
                    db.add_point(label, class_name, x, y, &styles)?;
                }
            }
        }
        if !is_eol(r.kind()) {
            return Err(r.fail("'NEWLINE', 'SEMI', 'EOF'"));
        }
        r.i += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<QuadrantDb, Error> {
        let mut db = QuadrantDb::new(None);
        parse_quadrant(src, &mut db)?;
        Ok(db)
    }

    #[test]
    fn reads_a_chart() {
        let db = parse(
            "quadrantChart\n  title Reach\n  x-axis Low Reach --> High Reach\n  y-axis Low\n  quadrant-1 We should expand\n  Campaign A:::hot: [0.3, 0.6] radius: 12\n  classDef hot color: #ff3300\n",
        )
        .unwrap();
        let m = db.model;
        assert_eq!(m.title.as_deref(), Some("Reach"));
        assert_eq!(m.x_axis_left.as_ref().unwrap().text, "Low Reach");
        assert_eq!(m.x_axis_right.as_ref().unwrap().text, "High Reach");
        assert_eq!(m.y_axis_bottom.as_ref().unwrap().text, "Low");
        assert_eq!(m.quadrants[0].as_ref().unwrap().text, "We should expand");
        assert_eq!(m.points[0].text.text, "Campaign A");
        assert_eq!(m.points[0].class_name, "hot");
        assert_eq!((m.points[0].x, m.points[0].y), (0.3, 0.6));
        assert_eq!(m.points[0].style.radius, Some(12.0));
        assert_eq!(m.classes["hot"].color.as_deref(), Some("#ff3300"));
    }

    #[test]
    fn an_open_arrow_marks_the_axis() {
        let db = parse("quadrantChart\n  x-axis Low -->\n").unwrap();
        assert_eq!(db.model.x_axis_left.unwrap().text, "Low ⟶");
    }

    #[test]
    fn reports_unexpected_tokens() {
        let e = parse("quadrantChart\n  A [0.3, 0.6]\n").err().unwrap();
        // `[` starts no token outside a point.
        assert_eq!((e.line, e.column), (2, 5));
        assert_eq!(e.message, "Lexical error on line 2. Unrecognized text.\n  A [0.3, 0.6]\n----^");
        let e = parse("quadrantChart\n  A: [0.3, 0.6] B\n").err().unwrap();
        // A word after a point is read as its style.
        assert_eq!(e.message, "style named B is not supported.");
        let e = parse("quadrantChart\n  A: [0.3, 0.6] radius: 4 ]\n").err().unwrap();
        assert_eq!(e.message, "Lexical error on line 2. Unrecognized text.\n  A: [0.3, 0.6] radius: 4 ]\n--------------------------^");
    }
}
