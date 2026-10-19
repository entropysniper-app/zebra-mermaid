#![allow(non_upper_case_globals)] // Token spelling follows the upstream grammar.
//! Requirement grammar and right-to-left body-field application, as in Pele.
use super::{
    lexer::{tokenize, Token},
    tokens::*,
    Relation, RequirementModel, RequirementNode,
};
use crate::{error::syntax_error, util::js, DiagramType, Error};
const VALUES: &[&str] = &[
    "Requirement",
    "Functional Requirement",
    "Interface Requirement",
    "Performance Requirement",
    "Physical Requirement",
    "Design Constraint",
    "Low",
    "Medium",
    "High",
    "Analysis",
    "Demonstration",
    "Inspection",
    "Test",
    "",
    "contains",
    "copies",
    "derives",
    "satisfies",
    "verifies",
    "refines",
    "traces",
];
struct Parser<'a> {
    src: &'a str,
    t: Vec<Token>,
    i: usize,
    m: &'a mut RequirementModel,
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
    fn fail(&self, from: u8, to: u8, other: &[u8]) -> Error {
        let at = self.t.get(self.i).map_or(self.src.len(), |t| t.start);
        if self.kind() == ERROR {
            return syntax_error(DiagramType::Requirement, self.src, at, "", true);
        }
        let expected: Vec<String> = (from..=to).chain(other.iter().copied()).map(|t| format!("'{}'", NAMES[t as usize])).collect();
        syntax_error(
            DiagramType::Requirement,
            self.src,
            at,
            &format!("Expecting {}, got '{}'", expected.join(", "), NAMES[self.kind() as usize]),
            false,
        )
    }
    fn expect(&mut self, k: u8) -> Result<String, Error> {
        if self.kind() != k {
            return Err(self.fail(k, k, &[]));
        }
        Ok(self.take())
    }
    fn keyword(&mut self, from: u8, to: u8) -> Result<String, Error> {
        let t = self.kind();
        if t < from || t > to {
            return Err(self.fail(from, to, &[]));
        }
        self.i += 1;
        Ok(VALUES[(t - REQUIREMENT) as usize].into())
    }
    fn string(&mut self) -> Result<String, Error> {
        if !matches!(self.kind(), qString | unqString) {
            return Err(self.fail(qString, unqString, &[]));
        }
        Ok(self.take())
    }
    fn directive(&mut self) -> Result<bool, Error> {
        let t = self.kind();
        if !matches!(t, acc_title | acc_descr | acc_descr_multiline_value) {
            return Ok(false);
        }
        let value = if t == acc_descr_multiline_value {
            self.take()
        } else {
            self.i += 1;
            self.expect(if t == acc_title { acc_title_value } else { acc_descr_value })?
        };
        let v = js::trim(&value);
        if t == acc_title {
            self.m.acc_title = Some(v.into());
        } else {
            self.m.acc_descr = Some(crate::util::chars::normalize_accessibility_description(&v));
        }
        Ok(true)
    }
    fn ids(&mut self) -> Result<Vec<String>, Error> {
        let mut out = vec![];
        loop {
            if !matches!(self.kind(), ALPHA | qString | unqString) {
                return Err(self.fail(qString, unqString, &[ALPHA]));
            }
            out.push(self.take());
            if self.kind() != COMMA {
                return Ok(out);
            }
            self.i += 1;
        }
    }
    fn styles(&mut self) -> Result<Vec<String>, Error> {
        fn part(k: u8) -> bool {
            matches!(k, ALPHA | COLON | BRKT | MINUS | SEMICOLON)
        }
        let mut out = vec![];
        loop {
            if !part(self.kind()) {
                return Err(self.fail(ALPHA, SEMICOLON, &[MINUS, BRKT]));
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
    fn classes(&mut self) -> Result<Option<Vec<String>>, Error> {
        let out = if self.kind() == STYLE_SEPARATOR {
            self.i += 1;
            let classes = self.ids()?;
            self.expect(STRUCT_START)?;
            Some(classes)
        } else if self.kind() == STRUCT_START {
            self.i += 1;
            None
        } else {
            return Err(self.fail(STRUCT_START, STRUCT_START, &[STYLE_SEPARATOR]));
        };
        self.expect(NEWLINE)?;
        Ok(out)
    }
    fn body(&mut self, requirement: bool) -> Result<RequirementNode, Error> {
        let mut fields = vec![];
        loop {
            let t = self.kind();
            if t == STRUCT_STOP {
                self.i += 1;
                break;
            }
            if t == NEWLINE {
                self.i += 1;
                continue;
            }
            if if requirement { !(ID..=VERIFYMTHD).contains(&t) } else { !matches!(t, TYPE | DOCREF) } {
                return Err(if requirement {
                    self.fail(ID, VERIFYMTHD, &[NEWLINE, STRUCT_STOP])
                } else {
                    self.fail(TYPE, DOCREF, &[NEWLINE, STRUCT_STOP])
                });
            }
            self.i += 1;
            self.expect(COLONSEP)?;
            let v = if t == RISK {
                self.keyword(LOW_RISK, HIGH_RISK)?
            } else if t == VERIFYMTHD {
                self.keyword(VERIFY_ANALYSIS, VERIFY_TEST)?
            } else {
                self.string()?
            };
            fields.push((t, v));
            self.expect(NEWLINE)?;
        }
        let mut node = RequirementNode { classes: vec!["default".into()], ..Default::default() };
        for (k, v) in fields.into_iter().rev() {
            match k {
                ID => node.id = v,
                TEXT => node.text = v,
                RISK => node.risk = v,
                VERIFYMTHD => node.verify = v,
                TYPE => node.kind = v,
                _ => node.doc_ref = v,
            }
        }
        Ok(node)
    }
}
pub fn parse(src: &str, model: &mut RequirementModel) -> Result<(), Error> {
    let mut p = Parser { src, t: tokenize(src), i: 0, m: model };
    while p.directive()? {
        if p.kind() == NEWLINE {
            p.i += 1;
        }
    }
    if p.kind() != RD {
        return Err(p.fail(RD, RD, &[acc_title, acc_descr, acc_descr_multiline_value]));
    }
    p.i += 1;
    p.expect(NEWLINE)?;
    loop {
        let t = p.kind();
        if t == EOF {
            return Ok(());
        }
        if t == NEWLINE {
            p.i += 1;
            continue;
        }
        if p.directive()? {
            continue;
        }
        if (direction_tb..=direction_lr).contains(&t) {
            p.i += 1;
            p.m.direction = ["TB", "BT", "RL", "LR"][(t - direction_tb) as usize].into();
        } else if (REQUIREMENT..=DESIGN_CONSTRAINT).contains(&t) {
            let kind = p.keyword(REQUIREMENT, DESIGN_CONSTRAINT)?;
            let name = p.string()?;
            let classes = p.classes()?;
            let mut node = p.body(true)?;
            node.kind = kind;
            node.name = name.clone();
            p.m.requirements.entry(name.clone()).or_insert(node);
            if let Some(classes) = classes {
                p.m.set_class(&[name], &classes)?;
            }
        } else if t == ELEMENT {
            p.i += 1;
            let name = p.string()?;
            let classes = p.classes()?;
            let mut node = p.body(false)?;
            node.name = name.clone();
            p.m.elements.entry(name.clone()).or_insert(node);
            if let Some(classes) = classes {
                p.m.set_class(&[name], &classes)?;
            }
        } else if matches!(t, STYLE | CLASSDEF | CLASS) {
            p.i += 1;
            let ids = p.ids()?;
            if t == CLASS {
                let classes = p.ids()?;
                p.m.set_class(&ids, &classes)?;
            } else {
                let styles = p.styles()?;
                if t == STYLE {
                    p.m.set_style(&ids, &styles)?;
                } else {
                    p.m.define_class(&ids, &styles)?;
                }
            }
        } else if matches!(t, unqString | qString) {
            let id = p.take();
            let next = p.kind();
            if next == STYLE_SEPARATOR {
                p.i += 1;
                let classes = p.ids()?;
                p.m.set_class(&[id], &classes)?;
            } else if matches!(next, END_ARROW_L | LINE) {
                p.i += 1;
                let kind = p.keyword(CONTAINS, TRACES)?;
                p.expect(if next == LINE { END_ARROW_R } else { LINE })?;
                let other = p.string()?;
                let (src, dst) = if next == LINE { (id, other) } else { (other, id) };
                p.m.relations.push(Relation { kind, src, dst });
            } else {
                return Err(p.fail(STYLE_SEPARATOR, STYLE_SEPARATOR, &[END_ARROW_L, LINE]));
            }
        } else {
            return Err(p.fail(
                NEWLINE,
                EOF,
                &[
                    acc_title,
                    acc_descr,
                    acc_descr_multiline_value,
                    direction_tb,
                    direction_bt,
                    direction_rl,
                    direction_lr,
                    REQUIREMENT,
                    FUNCTIONAL_REQUIREMENT,
                    INTERFACE_REQUIREMENT,
                    PERFORMANCE_REQUIREMENT,
                    PHYSICAL_REQUIREMENT,
                    DESIGN_CONSTRAINT,
                    ELEMENT,
                    CLASSDEF,
                    CLASS,
                    STYLE,
                    unqString,
                    qString,
                ],
            ));
        }
    }
}
