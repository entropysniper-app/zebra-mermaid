//! The XY chart parser. Ported from Pele `src/diagrams/xychart/parser.ts`, a recursive descent
//! reading of Mermaid's Jison grammar over the tokens of [`super::lexer`].

use super::db::{DataPoint, XyChartDb};
use super::lexer::tokenize;
use super::tokens::*;
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

/// A type no token has: what Pele reads past the last token.
const NONE: u8 = u8::MAX;

/// Tokens that join into a text.
fn is_word(t: u8) -> bool {
    matches!(t, AMP | NUM | ALPHA | PLUS | EQUALS | MULT | DOT | BRKT | MINUS | UNDERSCORE)
}

fn is_eol(t: u8) -> bool {
    matches!(t, NEWLINE | SEMI | EOF)
}

fn is_text(t: u8) -> bool {
    t == STR || is_word(t)
}

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

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.starts.get(self.i).copied().unwrap_or(self.src.len());
        if t == ERROR {
            return syntax_error(DiagramType::Xychart, self.src, at, "", true);
        }
        let name = TOKEN_NAMES.get(t as usize).copied().unwrap_or("undefined");
        syntax_error(DiagramType::Xychart, self.src, at, &format!("Expecting {}, got '{}'", expected, name), false)
    }

    fn expect(&mut self, t: u8) -> Result<&'a str, Error> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", TOKEN_NAMES[t as usize])));
        }
        self.i += 1;
        Ok(self.text_at(self.i - 1))
    }

    /// A text: a string, or a run of words, which are joined without the spaces between them.
    fn text(&mut self) -> Result<String, Error> {
        if !is_text(self.kind()) {
            return Err(self.fail("'STR', 'ALPHA', 'NUM'"));
        }
        let mut s = self.text_at(self.i).to_string();
        self.i += 1;
        if self.types[self.i - 1] != STR {
            while is_word(self.kind()) {
                s.push_str(self.text_at(self.i));
                self.i += 1;
            }
        }
        Ok(s)
    }

    fn range(&mut self) -> Result<(f64, f64), Error> {
        let min = js::number(self.expect(NUMBER_WITH_DECIMAL)?);
        self.expect(ARROW_DELIMITER)?;
        Ok((min, js::number(self.expect(NUMBER_WITH_DECIMAL)?)))
    }

    fn points(&mut self) -> Result<Vec<DataPoint>, Error> {
        let mut out = Vec::new();
        self.expect(SQUARE_BRACES_START)?;
        loop {
            let value = js::number(self.expect(NUMBER_WITH_DECIMAL)?);
            let label = if self.kind() == STR {
                self.i += 1;
                self.text_at(self.i - 1).to_string()
            } else {
                String::new()
            };
            out.push(DataPoint { value, label });
            if self.kind() != COMMA {
                break;
            }
            self.i += 1;
        }
        self.expect(SQUARE_BRACES_END)?;
        Ok(out)
    }
}

pub(crate) fn parse_xychart(src: &str, db: &mut XyChartDb) -> Result<(), Error> {
    let tokens = tokenize(src);
    let mut r = Parser { src, types: tokens.types, starts: tokens.starts, ends: tokens.ends, i: 0 };

    loop {
        if is_eol(r.kind()) {
            r.i += 1;
        } else if r.kind() != XYCHART {
            break;
        } else {
            r.i += 1;
            if r.kind() == CHART_ORIENTATION {
                db.set_orientation(r.text_at(r.i));
                r.i += 1;
            }
        }
    }

    // Pele stops at the end token; past the last token (which only an error token can be
    // reached by, and it fails first) there is nothing left to read either.
    while r.kind() != END && r.kind() != NONE {
        let t = r.kind();
        r.i += 1;
        match t {
            TITLE => {
                let text = r.text()?;
                db.set_diagram_title(js::trim(&text).to_string());
            }
            X_AXIS | Y_AXIS => {
                let title = if is_text(r.kind()) { Some(r.text()?) } else { None };
                if t == X_AXIS && r.kind() == SQUARE_BRACES_START {
                    let mut categories = Vec::new();
                    loop {
                        r.i += 1;
                        categories.push(r.text()?);
                        if r.kind() != COMMA {
                            break;
                        }
                    }
                    r.expect(SQUARE_BRACES_END)?;
                    db.set_x_axis_band(categories);
                } else if r.kind() == NUMBER_WITH_DECIMAL || title.is_none() {
                    let (min, max) = r.range()?;
                    if t == X_AXIS {
                        db.set_x_axis_range_data(min, max);
                    } else {
                        db.set_y_axis_range_data(min, max);
                    }
                }
                let title = title.unwrap_or_default();
                if t == X_AXIS {
                    db.set_x_axis_title(&title);
                } else {
                    db.set_y_axis_title(&title);
                }
            }
            LINE | BAR => {
                let title = if r.kind() == SQUARE_BRACES_START { String::new() } else { r.text()? };
                let data = r.points()?;
                if t == LINE {
                    db.set_line_data(&title, data);
                } else {
                    db.set_bar_data(&title, data);
                }
            }
            ACC_TITLE => {
                let value = r.expect(ACC_TITLE_VALUE)?;
                db.set_acc_title(js::trim(value));
            }
            ACC_DESCR => {
                let value = r.expect(ACC_DESCR_VALUE)?;
                db.set_acc_description(js::trim(value));
            }
            ACC_DESCR_MULTILINE_VALUE => {
                let value = r.text_at(r.i - 1);
                db.set_acc_description(js::trim(value));
            }
            _ => {
                r.i -= 1;
                return Err(r.fail("'title', 'X_AXIS', 'Y_AXIS', 'LINE', 'BAR', 'acc_title', 'acc_descr', 'acc_descr_multiline_value'"));
            }
        }
        while is_eol(r.kind()) {
            r.i += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::xychart::{PlotType, XAxis};

    fn parse(src: &str) -> Result<XyChartDb, Error> {
        let mut db = XyChartDb::new(None);
        parse_xychart(src, &mut db)?;
        Ok(db)
    }

    #[test]
    fn reads_axes_and_series() {
        let db = parse("xychart-beta horizontal\n title Sales Revenue\n x-axis [jan, \"feb\"]\n y-axis \"Money\" 0 --> 100\n bar [10, 20]\n line \"L\" [1 \"a\", 2]\n").unwrap();
        let m = db.model;
        // Words of a title join without the spaces between them, as in Pele.
        assert_eq!(m.title.as_deref(), Some("SalesRevenue"));
        assert_eq!(m.x_axis, XAxis::Band { title: String::new(), categories: vec!["jan".into(), "feb".into()] });
        assert_eq!((m.y_axis.min, m.y_axis.max, m.y_axis.title.as_str()), (0.0, 100.0, "Money"));
        assert_eq!(m.plots.len(), 2);
        assert_eq!(m.plots[1].kind, PlotType::Line);
        assert_eq!(m.plots[1].point_labels, Some(vec!["a".to_string(), String::new()]));
        assert_eq!(m.plots[0].data, vec![("jan".to_string(), Some(10.0)), ("feb".to_string(), Some(20.0))]);
    }

    #[test]
    fn reports_unexpected_tokens() {
        let e = parse("xychart-beta\n x-axis Jan, Feb\n").err().unwrap();
        assert_eq!(e.message, "Parse error on line 2:\n x-axis Jan, Feb\n-----------^\nExpecting 'title', 'X_AXIS', 'Y_AXIS', 'LINE', 'BAR', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', got 'COMMA'");
        assert_eq!((e.line, e.column), (2, 12));
    }
}
