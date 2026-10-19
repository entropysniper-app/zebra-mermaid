#![allow(non_upper_case_globals)]
//! Class grammar with iterative namespace frames, ported from Pele.
use super::{
    lexer::{tokenize, Token},
    tokens::*,
    ClassModel, Relation,
};
use crate::{error::syntax_error, util::js, DiagramType, Error};
const NAME_TOKENS: &str = "'UNICODE_TEXT', 'NUM', 'ALPHA', 'MINUS', 'BQUOTE_STR'";
fn alpha(t: u8) -> bool {
    matches!(t, UNICODE_TEXT | ALPHA | MINUS)
}
fn end_type(t: u8) -> Option<usize> {
    match t {
        AGGREGATION => Some(0),
        EXTENSION => Some(1),
        COMPOSITION => Some(2),
        DEPENDENCY => Some(3),
        LOLLIPOP => Some(4),
        _ => None,
    }
}
fn clean(s: &str) -> String {
    js::trim(s.strip_prefix(':').unwrap_or(s)).into()
}
struct Frame {
    id: String,
    classes: Vec<String>,
    notes: Vec<String>,
}
struct Parser<'a> {
    src: &'a str,
    t: Vec<Token>,
    i: usize,
    db: &'a mut ClassModel,
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
            syntax_error(DiagramType::Class, self.src, at, "", true)
        } else {
            syntax_error(DiagramType::Class, self.src, at, &format!("Expecting {expected}, got '{}'", NAMES[self.kind() as usize]), false)
        }
    }
    fn expect(&mut self, t: u8) -> Result<String, Error> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", NAMES[t as usize])));
        }
        Ok(self.take())
    }
    fn alpha(&mut self) -> Result<String, Error> {
        if !alpha(self.kind()) {
            return Err(self.fail("'UNICODE_TEXT', 'NUM', 'ALPHA', 'MINUS'"));
        }
        Ok(self.take())
    }
    fn name(&mut self, generic: bool) -> Result<String, Error> {
        let mut out = String::new();
        loop {
            if self.kind() == BQUOTE_STR {
                out.push_str(&self.take());
                break;
            }
            if !alpha(self.kind()) {
                return Err(self.fail(NAME_TOKENS));
            }
            out.push_str(&self.take());
            let t = self.kind();
            if t == DOT {
                out.push('.');
                self.i += 1;
            } else if !alpha(t) && t != BQUOTE_STR {
                break;
            }
        }
        if generic && self.kind() == GENERICTYPE {
            out.push('~');
            out.push_str(&self.take());
            out.push('~');
        }
        Ok(out)
    }
    fn label(&mut self) -> Result<String, Error> {
        self.i += 1;
        let s = self.expect(STR)?;
        self.expect(SQE)?;
        Ok(s)
    }
    fn members(&mut self, id: &str) -> Result<(), Error> {
        if self.kind() != MEMBER {
            return Err(self.fail("'STRUCT_STOP', 'MEMBER'"));
        }
        let mut rows = vec![];
        while self.kind() == MEMBER {
            rows.push(self.take());
        }
        self.expect(STRUCT_STOP)?;
        for row in rows {
            self.db.member(id, &row);
        }
        Ok(())
    }
    fn class(&mut self) -> Result<String, Error> {
        self.i += 1;
        let id = self.name(true)?;
        self.db.class(&id);
        if self.kind() == SQS {
            let label = self.label()?;
            self.db.ensure(&id).label = label;
        }
        match self.kind() {
            STYLE_SEPARATOR => {
                self.i += 1;
                let css = self.alpha()?;
                self.db.css_class(&id, &css);
                if self.kind() == STRUCT_START {
                    self.i += 1;
                    self.members(&id)?;
                }
            }
            STRUCT_START => {
                self.i += 1;
                if self.kind() == STRUCT_STOP {
                    self.i += 1;
                } else {
                    self.members(&id)?;
                }
            }
            ANNOTATION_START => {
                self.i += 1;
                let ann = self.alpha()?;
                self.expect(ANNOTATION_END)?;
                self.db.ensure(&id).annotations.push(ann);
                if self.kind() == STRUCT_START {
                    self.i += 1;
                    if self.kind() == STRUCT_STOP {
                        self.i += 1;
                    } else {
                        self.members(&id)?;
                    }
                }
            }
            _ => {}
        }
        Ok(id)
    }
    fn note(&mut self) -> Result<String, Error> {
        let t = self.kind();
        self.i += 1;
        let class = if t == NOTE_FOR { Some(self.name(true)?) } else { None };
        let text = self.expect(STR)?;
        Ok(self.db.note(text, class))
    }
    fn namespace(&mut self) -> Result<(), Error> {
        let mut frames: Vec<Frame> = vec![];
        loop {
            self.i += 1;
            let ns = self.name(false)?;
            let label = if self.kind() == SQS { Some(self.label()?) } else { None };
            let id = self.db.namespace(&ns, label);
            self.expect(STRUCT_START)?;
            if self.kind() == NEWLINE {
                self.i += 1;
            }
            frames.push(Frame { id, classes: vec![], notes: vec![] });
            loop {
                match self.kind() {
                    NAMESPACE => break,
                    CLASS => {
                        let id = self.class()?;
                        frames.last_mut().unwrap().classes.push(id);
                    }
                    NOTE | NOTE_FOR => {
                        let id = self.note()?;
                        frames.last_mut().unwrap().notes.push(id);
                    }
                    _ => return Err(self.fail("'NAMESPACE', 'CLASS', 'NOTE_FOR', 'NOTE'")),
                }
                loop {
                    if self.kind() == NEWLINE {
                        let next = self.t.get(self.i + 1).map_or(END, |t| t.kind);
                        self.i += 1;
                        if matches!(next, NAMESPACE | CLASS | NOTE | NOTE_FOR) {
                            break;
                        }
                    }
                    self.expect(STRUCT_STOP)?;
                    let done = frames.pop().unwrap();
                    self.db.assign_namespace(&done.id, &done.classes, &done.notes);
                    if frames.is_empty() {
                        return Ok(());
                    }
                }
            }
        }
    }
    fn relation_end(&mut self) -> Option<usize> {
        let end = end_type(self.kind());
        if end.is_some() {
            self.i += 1;
        }
        end
    }
    fn class_or_relation(&mut self) -> Result<(), Error> {
        let id1 = self.name(true)?;
        if self.kind() == LABEL {
            let member = clean(&self.take());
            self.db.member(&id1, &member);
            return Ok(());
        }
        let title1 = if self.kind() == STR {
            self.take()
        } else {
            if end_type(self.kind()).is_none() && !matches!(self.kind(), LINE | DOTTED_LINE) {
                return Ok(());
            }
            "none".into()
        };
        let end1 = self.relation_end();
        let t = self.kind();
        if !matches!(t, LINE | DOTTED_LINE) {
            return Err(self.fail("'AGGREGATION', 'EXTENSION', 'COMPOSITION', 'DEPENDENCY', 'LOLLIPOP', 'LINE', 'DOTTED_LINE'"));
        }
        self.i += 1;
        let end2 = self.relation_end();
        let title2 = if self.kind() == STR { self.take() } else { "none".into() };
        let id2 = self.name(true)?;
        let title = if self.kind() == LABEL { Some(clean(&self.take())) } else { None };
        self.db.relation(Relation { id1, id2, title1, title2, title, end1, end2, dotted: t == DOTTED_LINE });
        Ok(())
    }
    fn styles(&mut self) -> Result<Vec<String>, Error> {
        fn part(t: u8) -> bool {
            matches!(t, ALPHA | COLON | BRKT | STYLE | PCT | LABEL)
        }
        let mut out = vec![];
        loop {
            if !part(self.kind()) {
                return Err(self.fail("'ALPHA', 'NUM', 'COLON', 'UNIT', 'SPACE', 'BRKT', 'STYLE', 'PCT', 'LABEL'"));
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
    fn tooltip(&mut self, id: &str) {
        if self.kind() == STR {
            let tip = self.take();
            self.db.tooltip(id, tip);
        }
    }
    fn link(&mut self, id: &str) -> Result<(), Error> {
        let url = self.expect(STR)?;
        let tip = if self.kind() == STR { Some(self.take()) } else { None };
        let target = if self.kind() == LINK_TARGET { Some(self.take()) } else { None };
        self.db.link(id, &url, target);
        if let Some(t) = tip {
            self.db.tooltip(id, t);
        }
        Ok(())
    }
    fn statement(&mut self) -> Result<(), Error> {
        let t = self.kind();
        match t{
  UNICODE_TEXT|ALPHA|MINUS|BQUOTE_STR=>self.class_or_relation()?,NAMESPACE=>self.namespace()?,CLASS=>{self.class()?;},MEMBER=>self.i+=1,
  ANNOTATION_START=>{self.i+=1;let ann=self.alpha()?;self.expect(ANNOTATION_END)?;let id=self.name(true)?;self.db.ensure(&id).annotations.push(ann);},
  CALLBACK=>{self.i+=1;let id=self.name(true)?;self.expect(STR)?;self.db.callback(&id);self.tooltip(&id);},
  LINK=>{self.i+=1;let id=self.name(true)?;self.link(&id)?;},CLICK=>{self.i+=1;let id=self.name(true)?;if self.kind()==HREF{self.i+=1;self.link(&id)?;}else{self.expect(CALLBACK_NAME)?;if self.kind()==CALLBACK_ARGS{self.i+=1;}self.db.callback(&id);self.tooltip(&id);}},
  STYLE=>{self.i+=1;let id=self.expect(ALPHA)?;let styles=self.styles()?;if let Some(node)=self.db.classes.get_mut(&id){node.styles.extend(styles.iter().flat_map(|s|s.split(',').map(str::to_owned)));}},
  CSSCLASS=>{self.i+=1;let ids=self.expect(STR)?;let class=self.expect(ALPHA)?;self.db.css_class(&ids,&class);},NOTE|NOTE_FOR=>{self.note()?;},
  CLASSDEF=>{self.i+=1;let mut ids=vec![self.expect(ALPHA)?];while self.kind()==COMMA{self.i+=1;ids.push(self.expect(ALPHA)?);}let styles=self.styles()?;self.db.define_class(&ids,&styles);},
  direction_tb..=direction_lr=>{self.i+=1;self.db.direction=["TB","BT","RL","LR"][(t-direction_tb)as usize].into();},
  acc_title=>{self.i+=1;let v=self.expect(acc_title_value)?;self.db.acc_title=Some(js::trim(&v).into());},
  acc_descr|acc_descr_multiline_value=>{let v=if t==acc_descr{self.i+=1;self.expect(acc_descr_value)?}else{self.take()};self.db.acc_descr=Some(crate::util::chars::normalize_accessibility_description(js::trim(&v)));},
  _=>return Err(self.fail("'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'NAMESPACE', 'ANNOTATION_START', 'CLASS', 'MEMBER', 'NOTE_FOR', 'NOTE', 'CLASSDEF', 'direction_tb', 'CALLBACK', 'LINK', 'CLICK', 'STYLE', 'CSSCLASS', 'ALPHA', 'MINUS', 'BQUOTE_STR'"))
 }
        Ok(())
    }
}
pub fn parse(src: &str, db: &mut ClassModel) -> Result<(), Error> {
    let mut p = Parser { src, t: tokenize(src), i: 0, db };
    let header = p.kind() == CLASS_DIAGRAM;
    if header {
        p.i += 1;
        p.expect(NEWLINE)?;
    }
    loop {
        p.statement()?;
        if p.kind() != NEWLINE {
            break;
        }
        p.i += 1;
        if matches!(p.kind(), EOF | END) {
            break;
        }
    }
    if header {
        p.expect(EOF)?;
    }
    p.expect(END)?;
    Ok(())
}
