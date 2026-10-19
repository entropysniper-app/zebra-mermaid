#![allow(non_upper_case_globals)]
//! State parser using document indexes rather than recursive AST ownership.
use super::{
    lexer::{tokenize, Token},
    tokens::*,
    Description, State, StateModel, StateNote, Stmt,
};
use crate::{
    error::{locate, syntax_error},
    util::{chars::normalize_accessibility_description, js},
    DiagramType, Error, ErrorCode,
};
const STATEMENT:&str="'NL', 'ID', 'HIDE_EMPTY', 'scale', 'COMPOSIT_STATE', 'STATE_DESCR', 'FORK', 'JOIN', 'CHOICE', 'CONCURRENT', 'note', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'CLICK', 'classDef', 'style', 'class', 'direction_tb', 'direction_bt', 'direction_rl', 'direction_lr', 'EDGE_STATE'";
fn trim_colon(text: &str) -> String {
    js::trim(text.strip_prefix(':').unwrap_or(text)).into()
}
struct Parser<'a> {
    src: &'a str,
    t: Vec<Token>,
    i: usize,
    lex_error: Option<String>,
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
            if let Some(message) = &self.lex_error {
                let loc = locate(self.src, at);
                let mut e = Error::new(message.clone(), ErrorCode::Syntax).with_type(DiagramType::State);
                e.line = loc.line;
                e.column = loc.column;
                e.snippet = loc.snippet;
                e.offset = Some(at);
                e
            } else {
                syntax_error(DiagramType::State, self.src, at, "", true)
            }
        } else {
            syntax_error(DiagramType::State, self.src, at, &format!("Expecting {expected}, got '{}'", NAMES[self.kind() as usize]), false)
        }
    }
    fn expect(&mut self, k: u8) -> Result<String, Error> {
        if self.kind() != k {
            return Err(self.fail(&format!("'{}'", NAMES[k as usize])));
        }
        Ok(self.take())
    }
    fn id(&mut self) -> Result<State, Error> {
        if !matches!(self.kind(), ID | EDGE_STATE) {
            return Err(self.fail("'ID', 'EDGE_STATE'"));
        }
        let raw = self.take();
        let mut s = State {
            id: js::trim(&raw).into(),
            kind: Some("default".into()),
            description: Some(Description::Text(String::new())),
            ..Default::default()
        };
        if self.kind() == STYLE_SEPARATOR {
            self.i += 1;
            s.classes.push(js::trim(&self.expect(ID)?).into());
        }
        Ok(s)
    }
}
pub fn parse(src: &str, db: &mut StateModel) -> Result<(), Error> {
    let lex = tokenize(src);
    let mut p = Parser { src, t: lex.items, i: 0, lex_error: lex.error };
    while p.kind() == NL {
        p.i += 1;
    }
    if p.kind() != SD {
        return Err(p.fail("'NL', 'SD'"));
    }
    p.i += 1;
    let mut doc = 0;
    let mut stack = vec![];
    loop {
        let t = p.kind();
        let item = match t {
            NL => {
                p.i += 1;
                continue;
            }
            ID | EDGE_STATE => {
                let mut first = p.id()?;
                if p.kind() == DESCR {
                    first.description = Some(Description::Text(trim_colon(&p.take())));
                    Stmt::State(first)
                } else if p.kind() == ARROW {
                    p.i += 1;
                    let second = p.id()?;
                    let label = if p.kind() == DESCR { Some(trim_colon(&p.take())) } else { None };
                    Stmt::Relation { first, second, label }
                } else {
                    Stmt::State(first)
                }
            }
            HIDE_EMPTY => Stmt::Text(p.take()),
            scale => {
                p.i += 1;
                let value = p.t[p.i - 1].text.clone();
                p.expect(WIDTH)?;
                Stmt::Text(value)
            }
            COMPOSIT_STATE | STATE_DESCR => {
                let (id, description) = if t == COMPOSIT_STATE {
                    (p.take(), String::new())
                } else {
                    let d = p.take();
                    p.expect(AS)?;
                    (p.expect(ID)?, d)
                };
                if p.kind() == STRUCT_START {
                    p.i += 1;
                    let child = db.docs.len();
                    db.docs.push(vec![]);
                    db.docs[doc].push(Stmt::State(State {
                        id,
                        kind: Some("default".into()),
                        description: Some(Description::Text(description)),
                        doc: Some(child),
                        ..Default::default()
                    }));
                    stack.push(doc);
                    doc = child;
                    continue;
                }
                if t == COMPOSIT_STATE {
                    Stmt::Text(id)
                } else {
                    let (id, description) = if let Some((head, tail)) = id.split_once(':') {
                        (
                            head.to_string(),
                            Description::Lines(vec![js::trim(&description).into(), tail.split(':').next().unwrap_or("").into()]),
                        )
                    } else {
                        (id, Description::Text(js::trim(&description).into()))
                    };
                    Stmt::State(State { id, kind: Some("default".into()), description: Some(description), ..Default::default() })
                }
            }
            FORK | JOIN | CHOICE => Stmt::State(State {
                id: p.take(),
                kind: Some(
                    if t == FORK {
                        "fork"
                    } else if t == JOIN {
                        "join"
                    } else {
                        "choice"
                    }
                    .into(),
                ),
                ..Default::default()
            }),
            CONCURRENT => {
                p.i += 1;
                Stmt::State(State { id: db.divider(), kind: Some("divider".into()), ..Default::default() })
            }
            note => {
                let keyword = p.take();
                if matches!(p.kind(), left_of | right_of) {
                    let position = js::trim(&p.take()).to_string();
                    let id = js::trim(&p.expect(ID)?).to_string();
                    let text = js::trim(&p.expect(NOTE_TEXT)?).to_string();
                    Stmt::State(State { id, note: Some(StateNote { position, text }), ..Default::default() })
                } else {
                    if p.kind() != NOTE_TEXT {
                        return Err(p.fail("'NOTE_TEXT', 'left_of', 'right_of'"));
                    }
                    p.i += 1;
                    p.expect(AS)?;
                    p.expect(ID)?;
                    Stmt::Text(keyword)
                }
            }
            direction_tb..=direction_lr => {
                p.i += 1;
                Stmt::Dir(["TB", "BT", "RL", "LR"][(t - direction_tb) as usize].into())
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
                    db.acc_title = Some(value.clone());
                } else {
                    db.acc_descr = Some(normalize_accessibility_description(&value));
                }
                Stmt::Text(value)
            }
            CLICK => {
                p.i += 1;
                let id = p.id()?.id;
                let (url, tooltip) = if p.kind() == HREF {
                    p.i += 1;
                    (p.expect(STRING)?, String::new())
                } else {
                    if p.kind() != STRING {
                        return Err(p.fail("'STRING', 'HREF'"));
                    }
                    let url = p.take();
                    (url, p.expect(STRING)?)
                };
                p.expect(NL)?;
                Stmt::Click { id, url, tooltip }
            }
            classDef | style | class => {
                p.i += 1;
                let id = js::trim(&p.expect(if t == classDef {
                    CLASSDEF_ID
                } else if t == style {
                    STYLE_IDS
                } else {
                    CLASSENTITY_IDS
                })?)
                .to_string();
                let value = js::trim(&p.expect(if t == classDef {
                    CLASSDEF_STYLEOPTS
                } else if t == style {
                    STYLEDEF_STYLEOPTS
                } else {
                    STYLECLASS
                })?)
                .to_string();
                if t == classDef {
                    Stmt::ClassDef(id, value)
                } else if t == style {
                    Stmt::Style(id, value)
                } else {
                    Stmt::ApplyClass(id, value)
                }
            }
            STRUCT_STOP => {
                let Some(parent) = stack.pop() else {
                    return Err(p.fail(&format!("'$end', {STATEMENT}")));
                };
                p.i += 1;
                doc = parent;
                continue;
            }
            END => {
                if !stack.is_empty() {
                    return Err(p.fail(&format!("'STRUCT_STOP', {STATEMENT}")));
                }
                return Ok(());
            }
            _ => return Err(p.fail(&format!("{}, {STATEMENT}", if stack.is_empty() { "'$end'" } else { "'STRUCT_STOP'" }))),
        };
        if !matches!(&item,Stmt::Text(s)if s=="nl") {
            db.docs[doc].push(item);
        }
    }
}
