#![allow(non_upper_case_globals)]
//! Iterative ER grammar, preserving Pele's token/error and subgraph rules.
use super::{
    lexer::{tokenize, Token},
    tokens::*,
    Attribute, DocItem, ErModel,
};
use crate::{error::syntax_error, util::js, DiagramType, Error};
const NAME: &str = "'UNICODE_TEXT', 'NUM', 'ENTITY_NAME', 'DECIMAL_NUM', 'ENTITY_ONE'";
const CARDS: &str = "'ZERO_OR_ONE', 'ZERO_OR_MORE', 'ONE_OR_MORE', 'ONLY_ONE', 'MD_PARENT'";
const LINE:&str="'EOF', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'SUBGRAPH', 'direction_tb', 'direction_bt', 'direction_rl', 'direction_lr', 'CLASSDEF', 'CLASS', 'STYLE', 'UNICODE_TEXT', 'NUM', 'ENTITY_NAME', 'DECIMAL_NUM', 'ENTITY_ONE'";
fn name_token(t: u8) -> bool {
    matches!(t, UNICODE_TEXT | ENTITY_NAME | NUM | DECIMAL_NUM | ENTITY_ONE)
}
fn card(t: u8) -> Option<&'static str> {
    match t {
        ZERO_OR_ONE => Some("zero_or_one"),
        ZERO_OR_MORE => Some("zero_or_more"),
        ONE_OR_MORE => Some("one_or_more"),
        ONLY_ONE => Some("only_one"),
        MD_PARENT => Some("md_parent"),
        _ => None,
    }
}
fn strip_quote(s: &str) -> String {
    if s.len() >= 2 {
        s[1..s.len() - 1].into()
    } else {
        String::new()
    }
}
struct Frame {
    id: String,
    title: String,
    list: Vec<DocItem>,
}
struct Parser<'a> {
    src: &'a str,
    t: Vec<Token>,
    i: usize,
    m: &'a mut ErModel,
}
impl Parser<'_> {
    fn kind(&self) -> u8 {
        self.t.get(self.i).map_or(END, |t| t.kind)
    }
    fn take(&mut self) -> String {
        let s = self.t.get(self.i).map_or_else(String::new, |t| t.text.clone());
        self.i += 1;
        s
    }
    fn fail(&self, expected: &str) -> Error {
        let at = self.t.get(self.i).map_or(self.src.len(), |t| t.start);
        if self.kind() == ERROR {
            return syntax_error(DiagramType::Er, self.src, at, "", true);
        }
        let got = if self.kind() == CHAR { self.t.get(self.i).map_or("", |t| t.text.as_str()) } else { NAMES[self.kind() as usize] };
        syntax_error(DiagramType::Er, self.src, at, &format!("Expecting {expected}, got '{got}'"), false)
    }
    fn expect(&mut self, t: u8) -> Result<String, Error> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", NAMES[t as usize])));
        }
        Ok(self.take())
    }
    fn separator(&mut self) -> Result<(), Error> {
        if !matches!(self.kind(), NEWLINE | SEMI | EOF) {
            return Err(self.fail("'EOF', 'NEWLINE', 'SEMI'"));
        }
        self.i += 1;
        Ok(())
    }
    fn name(&mut self) -> Result<String, Error> {
        let t = self.kind();
        if !name_token(t) {
            return Err(self.fail(NAME));
        }
        let s = self.take();
        Ok(if t == ENTITY_NAME { strip_quote(&s) } else { s })
    }
    fn ids(&mut self) -> Result<Vec<String>, Error> {
        let mut out = vec![];
        loop {
            if !matches!(self.kind(), UNICODE_TEXT | STYLE_TEXT) {
                return Err(self.fail("'UNICODE_TEXT', 'STYLE_TEXT'"));
            }
            out.push(self.take());
            if self.kind() != COMMA {
                return Ok(out);
            }
            self.i += 1;
        }
    }
    fn styles(&mut self) -> Result<Vec<String>, Error> {
        fn part(t: u8) -> bool {
            matches!(t, STYLE_TEXT | NUM | COLON | BRKT)
        }
        let mut out = vec![];
        loop {
            if !part(self.kind()) {
                return Err(self.fail("'STYLE_TEXT', 'COLON', 'NUM', 'BRKT'"));
            }
            let mut s = self.take();
            while part(self.kind()) {
                s.push_str(&self.take());
            }
            out.push(s);
            if self.kind() != COMMA {
                return Ok(out);
            }
            self.i += 1;
        }
    }
    fn is_char(&self, s: &str) -> bool {
        self.kind() == CHAR && self.t.get(self.i).is_some_and(|t| t.text == s)
    }
    fn block(&mut self) -> Result<Vec<Attribute>, Error> {
        if self.kind() != BLOCK_START {
            return Ok(vec![]);
        }
        self.i += 1;
        let mut out = vec![];
        if self.kind() == BLOCK_STOP {
            self.i += 1;
            return Ok(out);
        }
        loop {
            let mut kind = self.expect(ATTRIBUTE_WORD)?;
            if self.is_char("?") {
                kind.push_str(&self.take());
            }
            if self.kind() != ATTRIBUTE_WORD {
                return Err(self.fail(if kind.ends_with('?') { "'ATTRIBUTE_WORD'" } else { "'ATTRIBUTE_WORD', '?'" }));
            }
            let name = self.take();
            let mut keys = vec![];
            if self.kind() == ATTRIBUTE_KEY {
                keys.push(self.take());
                while self.is_char(",") {
                    self.i += 1;
                    keys.push(self.expect(ATTRIBUTE_KEY)?);
                }
            }
            let comment = if self.kind() == COMMENT { strip_quote(&self.take()) } else { String::new() };
            out.push(Attribute { kind, name, keys, comment });
            if self.kind() == BLOCK_STOP {
                self.i += 1;
                return Ok(out);
            }
        }
    }
    fn classes(&mut self) -> Result<Vec<String>, Error> {
        if self.kind() == STYLE_SEPARATOR {
            self.i += 1;
            self.ids()
        } else {
            Ok(vec![])
        }
    }
    fn entity_statement(&mut self) -> Result<Vec<String>, Error> {
        let a = self.name()?;
        if self.kind() == SQS {
            self.i += 1;
            let alias = self.name()?;
            self.expect(SQE)?;
            let classes = self.classes()?;
            let attrs = self.block()?;
            self.m.entity(&a, &alias).attributes.extend(attrs);
            self.m.set_class(&[a.clone()], &classes);
            return Ok(vec![a]);
        }
        let classes = self.classes()?;
        if let Some(card_b) = card(self.kind()) {
            self.i += 1;
            let rel = self.kind();
            if !matches!(rel, IDENTIFYING | NON_IDENTIFYING) {
                return Err(self.fail("'NON_IDENTIFYING', 'IDENTIFYING'"));
            }
            self.i += 1;
            let Some(card_a) = card(self.kind()) else {
                return Err(self.fail(CARDS));
            };
            self.i += 1;
            let b = self.name()?;
            let classes_b = self.classes()?;
            self.expect(COLON)?;
            let t = self.kind();
            if !matches!(t, WORD | ENTITY_NAME | UNICODE_TEXT) {
                return Err(self.fail("'UNICODE_TEXT', 'ENTITY_NAME', 'WORD'"));
            }
            let text = self.take();
            let role = if t == UNICODE_TEXT { text } else { strip_quote(&text) };
            self.m.entity(&a, "");
            self.m.entity(&b, "");
            self.m.relationship(&a, &b, role, card_a.into(), card_b.into(), rel == IDENTIFYING);
            self.m.set_class(&[a.clone()], &classes);
            self.m.set_class(&[b.clone()], &classes_b);
            Ok(vec![a, b])
        } else {
            let attrs = self.block()?;
            self.m.entity(&a, "").attributes.extend(attrs);
            self.m.set_class(&[a.clone()], &classes);
            Ok(vec![a])
        }
    }
}
pub fn parse(src: &str, model: &mut ErModel) -> Result<(), Error> {
    let mut p = Parser { src, t: tokenize(src), i: 0, m: model };
    p.expect(ER_DIAGRAM)?;
    let mut frames = vec![Frame { id: String::new(), title: String::new(), list: vec![] }];
    loop {
        let nested = frames.len() > 1;
        let t = p.kind();
        match t {
            NEWLINE => p.i += 1,
            EOF => {
                p.i += 1;
                if nested {
                    return Err(p.fail(&format!("'END', {LINE}")));
                }
                return Ok(());
            }
            END_KW => {
                if !nested {
                    return Err(p.fail(LINE));
                }
                p.i += 1;
                let f = frames.pop().unwrap();
                let id = p.m.group(&f.id, &f.title, f.list);
                if frames.len() > 1 {
                    frames.last_mut().unwrap().list.push(DocItem::Node(id));
                }
            }
            SUBGRAPH => {
                p.i += 1;
                let id = p.name()?;
                let mut title = id.clone();
                if p.kind() == SQS {
                    p.i += 1;
                    title = p.name()?;
                    while name_token(p.kind()) {
                        title.push(' ');
                        title.push_str(&p.name()?);
                    }
                    p.expect(SQE)?;
                }
                p.separator()?;
                frames.push(Frame { id, title, list: vec![] });
            }
            acc_title | acc_descr | acc_descr_multiline_value => {
                let raw = if t == acc_descr_multiline_value {
                    p.take()
                } else {
                    p.i += 1;
                    p.expect(if t == acc_title { acc_title_value } else { acc_descr_value })?
                };
                let value = js::trim(&raw).to_string();
                if t == acc_title {
                    p.m.acc_title = Some(value.clone());
                } else {
                    p.m.acc_descr = Some(crate::util::chars::normalize_accessibility_description(&value));
                }
                if nested {
                    frames.last_mut().unwrap().list.push(DocItem::Node(value));
                }
            }
            direction_tb..=direction_lr => {
                p.i += 1;
                let value = ["TB", "BT", "RL", "LR"][(t - direction_tb) as usize].to_string();
                if nested {
                    frames.last_mut().unwrap().list.push(DocItem::Dir(value));
                } else {
                    p.m.direction = value;
                }
            }
            CLASSDEF | CLASS | STYLE => {
                let keyword = p.take();
                let ids = p.ids()?;
                if t == CLASS {
                    let names = p.ids()?;
                    p.m.set_class(&ids, &names);
                } else {
                    let styles = p.styles()?;
                    p.separator()?;
                    if t == CLASSDEF {
                        p.m.class(&ids, &styles);
                    } else {
                        p.m.styles(&ids, &styles);
                    }
                }
                if nested {
                    frames.last_mut().unwrap().list.push(DocItem::Node(keyword));
                }
            }
            _ => {
                if !name_token(t) {
                    return Err(p.fail(&if nested { format!("'END', {LINE}") } else { LINE.to_owned() }));
                }
                let names = p.entity_statement()?;
                if nested {
                    frames.last_mut().unwrap().list.extend(names.into_iter().map(DocItem::Node));
                }
            }
        }
    }
}
