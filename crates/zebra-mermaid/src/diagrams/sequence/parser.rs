//! The sequence parser. Ported from Pele `src/diagrams/sequence/parser.ts`.
//!
//! Pele builds Mermaid's nested statement lists (a block's statements in an array of their
//! own, the sections of an `alt` folded from the last one back) and then walks them depth
//! first. Statements only take effect in that walk, after the whole text has parsed, and the
//! walk visits them in the order they are written. This parser writes that order directly
//! into one flat list, which needs no recursion for nested blocks.

use super::db::{Declare, Menu, SeqDb, SeqText, Stmt};
use super::lexer::{tokenize_spans, Tokens};
use super::line_type as lt;
use super::placement;
use super::tokens::*;
use super::NoteActor;
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;
use std::{borrow::Cow, ops::Range};

const SIGNAL_TOKENS: &str =
    "'SOLID_OPEN_ARROW', 'DOTTED_OPEN_ARROW', 'SOLID_ARROW', 'DOTTED_ARROW', 'SOLID_CROSS', 'DOTTED_CROSS', 'SOLID_POINT', 'DOTTED_POINT'";

/// The message kind of an arrow token, or -1.
fn signal_of(t: u8) -> i32 {
    match t {
        SOLID_OPEN_ARROW => lt::SOLID_OPEN,
        DOTTED_OPEN_ARROW => lt::DOTTED_OPEN,
        SOLID_ARROW => lt::SOLID,
        DOTTED_ARROW => lt::DOTTED,
        BIDIRECTIONAL_SOLID_ARROW => lt::BIDIRECTIONAL_SOLID,
        BIDIRECTIONAL_DOTTED_ARROW => lt::BIDIRECTIONAL_DOTTED,
        SOLID_CROSS => lt::SOLID_CROSS,
        DOTTED_CROSS => lt::DOTTED_CROSS,
        SOLID_POINT => lt::SOLID_POINT,
        DOTTED_POINT => lt::DOTTED_POINT,
        SOLID_ARROW_TOP => lt::SOLID_TOP,
        SOLID_ARROW_BOTTOM => lt::SOLID_BOTTOM,
        STICK_ARROW_TOP => lt::STICK_TOP,
        STICK_ARROW_BOTTOM => lt::STICK_BOTTOM,
        SOLID_ARROW_TOP_DOTTED => lt::SOLID_TOP_DOTTED,
        SOLID_ARROW_BOTTOM_DOTTED => lt::SOLID_BOTTOM_DOTTED,
        STICK_ARROW_TOP_DOTTED => lt::STICK_TOP_DOTTED,
        STICK_ARROW_BOTTOM_DOTTED => lt::STICK_BOTTOM_DOTTED,
        SOLID_ARROW_TOP_REVERSE => lt::SOLID_ARROW_TOP_REVERSE,
        SOLID_ARROW_BOTTOM_REVERSE => lt::SOLID_ARROW_BOTTOM_REVERSE,
        STICK_ARROW_TOP_REVERSE => lt::STICK_ARROW_TOP_REVERSE,
        STICK_ARROW_BOTTOM_REVERSE => lt::STICK_ARROW_BOTTOM_REVERSE,
        SOLID_ARROW_TOP_REVERSE_DOTTED => lt::SOLID_ARROW_TOP_REVERSE_DOTTED,
        SOLID_ARROW_BOTTOM_REVERSE_DOTTED => lt::SOLID_ARROW_BOTTOM_REVERSE_DOTTED,
        STICK_ARROW_TOP_REVERSE_DOTTED => lt::STICK_ARROW_TOP_REVERSE_DOTTED,
        STICK_ARROW_BOTTOM_REVERSE_DOTTED => lt::STICK_ARROW_BOTTOM_REVERSE_DOTTED,
        _ => -1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Top,
    Box,
    Loop,
    Rect,
    Opt,
    Break,
    Alt,
    Par,
    Critical,
}

/// An open block: its kind and the message kind that ends it.
struct Frame {
    kind: Kind,
    end: i32,
}

struct Parser<'a> {
    src: &'a str,
    units: Vec<u16>,
    /// The byte offset of each code unit, and of the end.
    bytes: Vec<usize>,
    types: Vec<u8>,
    texts: Vec<Range<usize>>,
    starts: Vec<usize>,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        let units: Vec<u16> = src.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(units.len() + 1);
        for (at, c) in src.char_indices() {
            // Both units of a surrogate pair map to the start of the character.
            for _ in 0..c.len_utf16() {
                bytes.push(at);
            }
        }
        bytes.push(src.len());
        let Tokens { types, texts, starts } = tokenize_spans(&units);
        Parser { src, units, bytes, types, texts, starts, i: 0 }
    }

    fn kind(&self) -> u8 {
        self.types[self.i.min(self.types.len() - 1)]
    }

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.bytes[self.starts[self.i.min(self.starts.len() - 1)].min(self.units.len())];
        if t == ERROR {
            return syntax_error(DiagramType::Sequence, self.src, at, "", true);
        }
        syntax_error(DiagramType::Sequence, self.src, at, &format!("Expecting {}, got '{}'", expected, TOKEN_NAMES[t as usize]), false)
    }

    fn take(&mut self) -> Cow<'a, str> {
        let span = std::mem::take(&mut self.texts[self.i]);
        self.i += 1;
        // Both halves of a surrogate pair map to its first byte. Borrow only complete
        // characters; preserve the original lossy conversion for a split pair.
        let boundary = |i: usize| i == 0 || i == self.units.len() || self.bytes[i] != self.bytes[i - 1];
        if boundary(span.start) && boundary(span.end) {
            Cow::Borrowed(&self.src[self.bytes[span.start]..self.bytes[span.end]])
        } else {
            Cow::Owned(String::from_utf16_lossy(&self.units[span]))
        }
    }

    fn expect(&mut self, t: u8) -> Result<Cow<'a, str>, Error> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", TOKEN_NAMES[t as usize])));
        }
        Ok(self.take())
    }

    fn actor(&mut self) -> Result<Cow<'a, str>, Error> {
        let t = self.kind();
        if t != ACTOR && t != LINK {
            return Err(self.fail("'ACTOR'"));
        }
        Ok(self.take())
    }

    /// `db.parseMessage(expect(T.TXT).trim().substring(1))`: the text after the colon.
    fn text2(&mut self, db: &SeqDb) -> Result<SeqText, Error> {
        let raw = self.expect(TXT)?;
        let trimmed = js::trim(&raw);
        // The token starts with its colon, which trimming keeps.
        let rest = trimmed.get(1..).unwrap_or("");
        Ok(db.parse_message(rest))
    }

    fn participant(&mut self, db: &SeqDb, create: bool) -> Result<Stmt<'a>, Error> {
        let t = self.kind();
        self.i += 1;
        if t == DESTROY {
            let actor = self.actor()?;
            self.expect(NEWLINE)?;
            return Ok(Stmt::Participant {
                kind: if create { Declare::Create } else { Declare::Destroy },
                actor,
                draw: None,
                description: None,
                config: None,
            });
        }
        let actor = self.actor()?;
        let mut config = None;
        if self.kind() == CONFIG_START && self.types[self.i - 1] == ACTOR {
            self.i += 1;
            let content = self.expect(CONFIG_CONTENT)?;
            self.expect(CONFIG_END)?;
            config = Some(js::trim(&content).to_string());
        }
        let draw = Some(if t == PARTICIPANT { "participant" } else { "actor" });
        let mut description = None;
        if self.kind() == AS {
            self.i += 1;
            let alias = self.expect(REST_OF_LINE)?;
            self.expect(NEWLINE)?;
            description = Some(db.parse_message(&alias));
        } else if self.kind() == NEWLINE {
            self.i += 1;
        } else {
            return Err(self.fail("'AS', 'NEWLINE'"));
        }
        Ok(Stmt::Participant { kind: if create { Declare::Create } else { Declare::Add }, actor, draw, description, config })
    }

    fn mention(actor: &Cow<'a, str>) -> Stmt<'a> {
        Stmt::Participant { kind: Declare::Add, actor: actor.clone(), draw: None, description: None, config: None }
    }

    fn signal(&mut self, db: &SeqDb, from: Cow<'a, str>, out: &mut Vec<Stmt<'a>>) -> Result<(), Error> {
        let mut central_from = false;
        if self.kind() == CENTRAL {
            central_from = true;
            self.i += 1;
        }
        let signal_type = signal_of(self.kind());
        if signal_type < 0 {
            return Err(self.fail(SIGNAL_TOKENS));
        }
        self.i += 1;
        let mark = self.kind();
        if mark == CENTRAL || (!central_from && (mark == PLUS || mark == MINUS)) {
            self.i += 1;
        }
        let to = self.actor()?;
        let msg = self.text2(db)?;
        out.push(Self::mention(&from));
        out.push(Self::mention(&to));
        let mut activate = false;
        let mut central = None;
        let mut after: Vec<Stmt<'a>> = Vec::new();
        if central_from {
            let dual = mark == CENTRAL;
            activate = dual;
            central = Some(if dual { lt::CENTRAL_CONNECTION_DUAL } else { lt::CENTRAL_CONNECTION_REVERSE });
            if dual {
                after.push(Stmt::Signal { signal_type: lt::CENTRAL_CONNECTION, actor: to.clone() });
            }
            after.push(Stmt::Signal { signal_type: lt::CENTRAL_CONNECTION_REVERSE, actor: from.clone() });
        } else if mark == PLUS {
            activate = true;
            after.push(Stmt::Signal { signal_type: lt::ACTIVE_START, actor: to.clone() });
        } else if mark == MINUS {
            after.push(Stmt::Signal { signal_type: lt::ACTIVE_END, actor: from.clone() });
        } else if mark == CENTRAL {
            activate = true;
            central = Some(lt::CENTRAL_CONNECTION);
            after.push(Stmt::Signal { signal_type: lt::CENTRAL_CONNECTION, actor: to.clone() });
        }
        out.push(Stmt::Message { from, to, signal_type, msg, activate, central });
        out.append(&mut after);
        Ok(())
    }

    fn menu(&mut self, db: &SeqDb, kind: Menu, out: &mut Vec<Stmt<'a>>) -> Result<(), Error> {
        self.i += 1;
        let actor = self.actor()?;
        let text = self.text2(db)?;
        out.push(Self::mention(&actor));
        out.push(Stmt::Menu { kind, actor, text });
        Ok(())
    }

    fn note(&mut self, db: &SeqDb, out: &mut Vec<Stmt<'a>>) -> Result<(), Error> {
        self.i += 1;
        let t = self.kind();
        if t == OVER {
            self.i += 1;
            let first = self.actor()?;
            let mut ids = [first.clone(), first.clone()];
            let mut second = None;
            if self.kind() == COMMA {
                self.i += 1;
                let s = self.actor()?;
                ids = [first.clone(), s.clone()];
                second = Some(s);
            }
            let text = self.text2(db)?;
            out.push(Self::mention(&first));
            if let Some(s) = second {
                out.push(Self::mention(&s));
            }
            out.push(Stmt::Note { actor: NoteActor::Two(ids.map(Cow::into_owned)), placement: placement::OVER, text });
            return Ok(());
        }
        if t != LEFT_OF && t != RIGHT_OF {
            return Err(self.fail("'over', 'left_of', 'right_of'"));
        }
        self.i += 1;
        let who = self.actor()?;
        let place = if t == LEFT_OF { placement::LEFT_OF } else { placement::RIGHT_OF };
        let text = self.text2(db)?;
        out.push(Self::mention(&who));
        out.push(Stmt::Note { actor: NoteActor::One(who.into_owned()), placement: place, text });
        Ok(())
    }

    fn autonumber(&mut self) -> Result<Stmt<'a>, Error> {
        self.i += 1;
        let mut start = None;
        let mut step = None;
        let mut visible = true;
        if self.kind() == NUM {
            start = Some(js::number(&self.take()));
            step = Some(if self.kind() == NUM { js::number(&self.take()) } else { 1.0 });
        } else if self.kind() == OFF {
            self.i += 1;
            visible = false;
        }
        if self.kind() != NEWLINE {
            return Err(self.fail("'NEWLINE', 'NUM', 'off'"));
        }
        self.i += 1;
        Ok(Stmt::SequenceIndex { start, step, visible })
    }

    /// The text of a `title` statement: past the keyword and the white space character after
    /// it, counted in UTF-16 units.
    fn title_text(text: &str, skip: usize) -> String {
        js::slice16(text, skip, js::len16(text)).to_string()
    }
}

pub(super) fn parse_sequence(src: &str, db: &mut SeqDb) -> Result<(), Error> {
    let mut p = Parser::new(src);
    let mut out: Vec<Stmt<'_>> = Vec::new();

    while p.kind() == NEWLINE {
        p.i += 1;
    }
    p.expect(SD)?;

    let mut stack: Vec<Frame> = vec![Frame { kind: Kind::Top, end: 0 }];
    loop {
        let t = p.kind();
        let frame = stack.last().map_or(Kind::Top, |f| f.kind);
        if frame == Kind::Box {
            if t == PARTICIPANT || t == PARTICIPANT_ACTOR || t == DESTROY {
                let s = p.participant(db, false)?;
                out.push(s);
            } else if t == NEWLINE {
                p.i += 1;
            } else if t == KW_END {
                p.i += 1;
                stack.pop();
                out.push(Stmt::BoxEnd);
            } else {
                return Err(p.fail("'end', 'NEWLINE', 'participant', 'participant_actor', 'destroy'"));
            }
            continue;
        }
        match t {
            NEWLINE | INVALID => p.i += 1,
            PARTICIPANT | PARTICIPANT_ACTOR | DESTROY => {
                let s = p.participant(db, false)?;
                out.push(s);
            }
            CREATE => {
                p.i += 1;
                let next = p.kind();
                if next != PARTICIPANT && next != PARTICIPANT_ACTOR && next != DESTROY {
                    return Err(p.fail("'participant', 'participant_actor', 'destroy'"));
                }
                let s = p.participant(db, true)?;
                out.push(s);
            }
            ACTOR => {
                let from = p.actor()?;
                p.signal(db, from, &mut out)?;
                p.expect(NEWLINE)?;
            }
            LINK => {
                let next = p.types.get(p.i + 1).copied().unwrap_or(END);
                if next == ACTOR || next == LINK {
                    p.menu(db, Menu::ALink, &mut out)?;
                } else {
                    let from = p.actor()?;
                    p.signal(db, from, &mut out)?;
                }
                p.expect(NEWLINE)?;
            }
            LINKS => {
                p.menu(db, Menu::Links, &mut out)?;
                p.expect(NEWLINE)?;
            }
            PROPERTIES => {
                p.menu(db, Menu::Properties, &mut out)?;
                p.expect(NEWLINE)?;
            }
            DETAILS => {
                p.menu(db, Menu::Details, &mut out)?;
                p.expect(NEWLINE)?;
            }
            NOTE => {
                p.note(db, &mut out)?;
                p.expect(NEWLINE)?;
            }
            AUTONUMBER => {
                let s = p.autonumber()?;
                out.push(s);
            }
            ACTIVATE | DEACTIVATE => {
                p.i += 1;
                let who = p.actor()?;
                p.expect(NEWLINE)?;
                let signal_type = if t == ACTIVATE { lt::ACTIVE_START } else { lt::ACTIVE_END };
                out.push(Stmt::Signal { signal_type, actor: who });
            }
            TITLE | LEGACY_TITLE => {
                let text = p.take();
                db.set_diagram_title(Parser::title_text(&text, if t == TITLE { 6 } else { 7 }));
            }
            ACC_TITLE => {
                p.i += 1;
                let value = p.expect(ACC_TITLE_VALUE)?;
                db.set_acc_title(js::trim(&value));
            }
            ACC_DESCR => {
                p.i += 1;
                let value = p.expect(ACC_DESCR_VALUE)?;
                db.set_acc_description(js::trim(&value));
            }
            ACC_DESCR_MULTILINE_VALUE => {
                let value = p.take();
                db.set_acc_description(js::trim(&value));
            }
            LOOP | RECT | OPT | BREAK | ALT | PAR | PAR_OVER | CRITICAL | BOX => {
                p.i += 1;
                let text = p.expect(REST_OF_LINE)?;
                let (kind, start, end) = match t {
                    LOOP => (Kind::Loop, lt::LOOP_START, lt::LOOP_END),
                    RECT => (Kind::Rect, lt::RECT_START, lt::RECT_END),
                    OPT => (Kind::Opt, lt::OPT_START, lt::OPT_END),
                    BREAK => (Kind::Break, lt::BREAK_START, lt::BREAK_END),
                    ALT => (Kind::Alt, lt::ALT_START, lt::ALT_END),
                    PAR => (Kind::Par, lt::PAR_START, lt::PAR_END),
                    PAR_OVER => (Kind::Par, lt::PAR_OVER_START, lt::PAR_END),
                    CRITICAL => (Kind::Critical, lt::CRITICAL_START, lt::CRITICAL_END),
                    _ => (Kind::Box, 0, 0),
                };
                if kind == Kind::Box {
                    out.push(Stmt::BoxStart(db.parse_box_data(&text)));
                } else {
                    out.push(Stmt::Section { signal_type: start, text: db.parse_message(&text) });
                }
                stack.push(Frame { kind, end });
            }
            ELSE | AND | OPTION => {
                let wanted = match t {
                    ELSE => Kind::Alt,
                    AND => Kind::Par,
                    _ => Kind::Critical,
                };
                if frame != wanted {
                    return Err(p.fail("'end'"));
                }
                p.i += 1;
                let text = p.expect(REST_OF_LINE)?;
                let signal_type = match t {
                    ELSE => lt::ALT_ELSE,
                    AND => lt::PAR_AND,
                    _ => lt::CRITICAL_OPTION,
                };
                out.push(Stmt::Section { signal_type, text: db.parse_message(&text) });
            }
            KW_END => {
                if frame == Kind::Top {
                    return Err(p.fail("'NEWLINE', 'EOF'"));
                }
                p.i += 1;
                if let Some(done) = stack.pop() {
                    out.push(Stmt::BlockEnd { signal_type: done.end });
                }
            }
            END => {
                if frame != Kind::Top {
                    return Err(p.fail("'end'"));
                }
                return db.apply(out);
            }
            _ => return Err(p.fail("'NEWLINE', 'ACTOR', 'participant', 'note', 'end'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_token_text_matches_utf16_decoding_at_every_boundary() {
        let source = "sequenceDiagram\nparticipant 😀 as 中文🦀\n😀->>Bob: e\u{301}\u{feff}";
        let mut parser = Parser::new(source);
        for start in 0..=parser.units.len() {
            for end in start..=parser.units.len() {
                let expected = String::from_utf16_lossy(&parser.units[start..end]);
                parser.texts.clear();
                parser.texts.push(start..end);
                parser.i = 0;
                let actual = parser.take();
                assert_eq!(actual, expected, "UTF-16 span {start}..{end}");
                let boundary = |i: usize| i == 0 || i == parser.units.len() || parser.bytes[i] != parser.bytes[i - 1];
                if boundary(start) && boundary(end) {
                    assert!(matches!(actual, Cow::Borrowed(_)));
                }
            }
        }
    }
}
