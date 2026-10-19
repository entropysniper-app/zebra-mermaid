//! Iterative flowchart parser, following Pele's token precedence and semantic actions.
use super::{lexer::tokenize, model::*, tokens::*};
use crate::error::syntax_error;
use crate::util::{chars::is_space, js};
use crate::{DiagramType, Error};
use indexmap::IndexMap;

const TEXT_TOKENS: &str = "'UNICODE_TEXT', 'TEXT', 'TAGSTART'";
const STATEMENT_START:&str="'SEMI', 'NEWLINE', 'SPACE', 'EOF', 'subgraph', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'STYLE', 'LINKSTYLE', 'CLASSDEF', 'CLASS', 'CLICK', 'NODE_STRING'";
fn id_token(t: u8) -> bool {
    matches!(t, NUM | NODE_STRING | DOWN | MINUS | DEFAULT | COMMA | COLON | AMP | BRKT | MULT | UNICODE_TEXT)
}
fn alpha_token(t: u8) -> bool {
    matches!(t, NUM | UNICODE_TEXT | NODE_STRING | DIR | DOWN | MINUS | COMMA | COLON | AMP | MULT | BRKT)
}
fn no_tag(t: u8) -> bool {
    matches!(
        t,
        NUM | NODE_STRING
            | SPACE
            | MINUS
            | AMP
            | UNICODE_TEXT
            | COLON
            | MULT
            | BRKT
            | STYLE
            | LINKSTYLE
            | CLASSDEF
            | CLASS
            | CLICK
            | GRAPH
            | DIR
            | SUBGRAPH_KW
            | END_KW
            | DOWN
            | UP
            | START_LINK
    )
}
fn style_token(t: u8) -> bool {
    matches!(t, NUM | NODE_STRING | COLON | SPACE | BRKT | STYLE)
}
struct Parser<'a> {
    src: &'a str,
    types: Vec<u8>,
    texts: Vec<String>,
    starts: Vec<usize>,
    i: usize,
    db: &'a mut FlowchartModel,
    pending: Option<String>,
}
#[derive(Default)]
struct Frame {
    id: Option<FlowText>,
    title: Option<FlowText>,
    same_title: bool,
    list: Vec<DocItem>,
    collect: bool,
}
impl Parser<'_> {
    fn kind(&self) -> u8 {
        self.types.get(self.i).copied().unwrap_or(END)
    }
    fn take(&mut self) -> String {
        let s = self.texts.get(self.i).cloned().unwrap_or_default();
        self.i += 1;
        s
    }
    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.starts.get(self.i).copied().unwrap_or(self.src.len());
        if t == ERROR {
            syntax_error(DiagramType::Flowchart, self.src, at, "", true)
        } else {
            syntax_error(DiagramType::Flowchart, self.src, at, &format!("Expecting {expected}, got '{}'", TOKEN_NAMES[t as usize]), false)
        }
    }
    fn expect(&mut self, kind: u8, also: &str) -> Result<String, Error> {
        if self.kind() != kind {
            return Err(self.fail(&format!(
                "'{}'{}",
                TOKEN_NAMES[kind as usize],
                if also.is_empty() { String::new() } else { format!(", {also}") }
            )));
        }
        Ok(self.take())
    }
    fn separator(&mut self) -> Result<(), Error> {
        if !matches!(self.kind(), NEWLINE | SEMI | EOF) {
            return Err(self.fail("'SEMI', 'NEWLINE', 'EOF'"));
        }
        self.i += 1;
        Ok(())
    }
    fn scan(&mut self, test: fn(u8) -> bool, expected: &str) -> Result<String, Error> {
        if !test(self.kind()) {
            return Err(self.fail(expected));
        }
        let mut s = self.take();
        while test(self.kind()) {
            s.push_str(&self.take());
        }
        Ok(s)
    }
    fn id(&mut self) -> Result<String, Error> {
        self.scan(id_token, "'NODE_STRING'")
    }
    fn alpha(&mut self) -> Result<String, Error> {
        self.scan(alpha_token, "'NODE_STRING'")
    }
    fn text(&mut self) -> Result<FlowText, Error> {
        let kind = match self.kind() {
            STR => "string",
            MD_STR => "markdown",
            TEXT | TAGSTART | TAGEND | UNICODE_TEXT => "text",
            _ => return Err(self.fail(&format!("{TEXT_TOKENS}, 'TAGEND', 'STR', 'MD_STR'"))),
        };
        let mut text = self.take();
        while matches!(self.kind(), TEXT | TAGSTART | TAGEND | UNICODE_TEXT) {
            text.push_str(&self.take());
        }
        Ok(FlowText { text, kind: kind.into() })
    }
    fn edge_text(&mut self) -> Result<FlowText, Error> {
        let kind = match self.kind() {
            STR => "string",
            MD_STR => "markdown",
            EDGE_TEXT | UNICODE_TEXT => "text",
            _ => return Err(self.fail("'EDGE_TEXT', 'STR', 'MD_STR'")),
        };
        let mut text = self.take();
        while matches!(self.kind(), EDGE_TEXT | UNICODE_TEXT) {
            text.push_str(&self.take());
        }
        Ok(FlowText { text, kind: kind.into() })
    }
    fn no_tags(&mut self) -> Result<FlowText, Error> {
        let kind = if self.kind() == MD_STR {
            "markdown"
        } else if self.kind() == STR || no_tag(self.kind()) {
            "text"
        } else {
            return Err(self.fail("'NODE_STRING', 'STR', 'MD_STR'"));
        };
        let mut text = self.take();
        while no_tag(self.kind()) {
            text.push_str(&self.take());
        }
        Ok(FlowText { text, kind: kind.into() })
    }
    fn shape_data(&mut self) -> String {
        let mut s = self.take();
        while self.kind() == SHAPE_DATA {
            s.push_str(&self.take());
        }
        s
    }
    fn vertex(&mut self) -> Result<String, Error> {
        let id = self.id()?;
        let t = self.kind();
        let mut props = None;
        let description = match t {
            SQS => Some((SQE, "square", None)),
            DOUBLECIRCLESTART => Some((DOUBLECIRCLEEND, "doublecircle", None)),
            ELLIPSE_START => Some((ELLIPSE_END, "ellipse", None)),
            STADIUMSTART => Some((STADIUMEND, "stadium", None)),
            SUBROUTINESTART => Some((SUBROUTINEEND, "subroutine", None)),
            CYLINDERSTART => Some((CYLINDEREND, "cylinder", None)),
            TAGEND => Some((SQE, "odd", None)),
            PS => {
                self.i += 1;
                if self.kind() == PS {
                    Some((PE, "circle", Some(PE)))
                } else {
                    self.i -= 1;
                    Some((PE, "round", None))
                }
            }
            DIAMOND_START => {
                self.i += 1;
                if self.kind() == DIAMOND_START {
                    Some((DIAMOND_STOP, "hexagon", Some(DIAMOND_STOP)))
                } else {
                    self.i -= 1;
                    Some((DIAMOND_STOP, "diamond", None))
                }
            }
            VERTEX_WITH_PROPS_START => {
                self.i += 1;
                let field = self.expect(NODE_STRING, "")?;
                self.expect(COLON, "")?;
                let value = self.expect(NODE_STRING, "")?;
                self.expect(PIPE, "")?;
                let mut p = IndexMap::new();
                p.insert(field, value);
                props = Some(p);
                self.i -= 1;
                Some((SQE, "rect", None))
            }
            TRAPSTART | INVTRAPSTART => {
                self.i += 1;
                let label = self.text()?;
                let shape = if t == TRAPSTART {
                    if self.kind() == INVTRAPEND {
                        self.i += 1;
                        "lean_right"
                    } else {
                        self.expect(TRAPEND, TEXT_TOKENS)?;
                        "trapezoid"
                    }
                } else if self.kind() == TRAPEND {
                    self.i += 1;
                    "lean_left"
                } else {
                    self.expect(INVTRAPEND, TEXT_TOKENS)?;
                    "inv_trapezoid"
                };
                self.db.vertex(&id, Some(label), Some(shape), None, None, None)?;
                return Ok(id);
            }
            _ => None,
        };
        if let Some((end, shape, extra)) = description {
            self.i += 1;
            let text = self.text()?;
            self.expect(end, TEXT_TOKENS)?;
            if let Some(extra) = extra {
                self.expect(extra, "")?;
            }
            self.db.vertex(&id, Some(text), Some(shape), None, props, None)?;
        } else {
            self.db.vertex(&id, None, None, None, None, None)?;
        }
        Ok(id)
    }
    fn styled_vertex(&mut self) -> Result<String, Error> {
        let id = self.vertex()?;
        if self.kind() == STYLE_SEPARATOR {
            self.i += 1;
            let class = self.id()?;
            self.db.set_class(&id, &class);
        }
        Ok(id)
    }
    fn node(&mut self) -> Result<Vec<String>, Error> {
        let mut ids = vec![self.styled_vertex()?];
        self.pending = None;
        loop {
            let data = if self.kind() == SHAPE_DATA { Some(self.shape_data()) } else { None };
            if self.kind() != SPACE {
                self.pending = data;
                return Ok(ids);
            }
            while self.kind() == SPACE {
                self.i += 1;
            }
            if self.kind() != AMP {
                if data.is_some() {
                    return Err(self.fail("'AMP'"));
                }
                return Ok(ids);
            }
            self.i += 1;
            if self.kind() != SPACE {
                return Err(self.fail("'SPACE'"));
            }
            while self.kind() == SPACE {
                self.i += 1;
            }
            let next = self.styled_vertex()?;
            if let Some(data) = data {
                self.db.vertex(ids.last().unwrap(), None, None, None, None, Some(&data))?;
            }
            ids.push(next);
        }
    }
    fn link(&mut self) -> Result<Link, Error> {
        let id = if self.kind() == LINK_ID { Some(self.take()) } else { None };
        let mut link = if self.kind() == LINK {
            let raw = self.take();
            let mut l = destruct_link(&raw, None);
            if self.kind() == PIPE {
                self.i += 1;
                l.text = Some(self.text()?);
                self.expect(PIPE, TEXT_TOKENS)?;
                if self.kind() == SPACE {
                    self.i += 1;
                }
            }
            l
        } else if self.kind() == START_LINK {
            let start = self.take();
            let text = self.edge_text()?;
            let end = self.expect(LINK, "'EDGE_TEXT'")?;
            let mut l = destruct_link(&end, Some(&start));
            l.text = Some(text);
            l
        } else {
            return Err(self.fail("'LINK', 'START_LINK'"));
        };
        link.id = id;
        Ok(link)
    }
    fn vertex_statement(&mut self) -> Result<Vec<String>, Error> {
        let mut stmt = self.node()?;
        if let Some(raw) = self.pending.take() {
            self.db.vertex(stmt.last().unwrap(), None, None, None, None, Some(&raw))?;
        }
        let mut nodes = stmt.clone();
        while matches!(self.kind(), LINK | START_LINK | LINK_ID) {
            let link = self.link()?;
            let next = self.node()?;
            if let Some(raw) = self.pending.take() {
                self.db.vertex(next.last().unwrap(), None, None, None, None, Some(&raw))?;
            }
            self.db.add_link(&stmt, &next, &link)?;
            let mut joined = next.clone();
            joined.extend(nodes);
            nodes = joined;
            stmt = next;
        }
        Ok(nodes)
    }
    fn styles(&mut self) -> Result<Vec<String>, Error> {
        let mut out = vec![self.scan(style_token, "'NODE_STRING', 'NUM', 'COLON', 'BRKT'")?];
        while self.kind() == COMMA {
            self.i += 1;
            out.push(self.scan(style_token, "'NODE_STRING', 'NUM', 'COLON', 'BRKT'")?);
        }
        Ok(out)
    }
    fn link_style(&mut self) -> Result<(), Error> {
        self.i += 1;
        self.expect(SPACE, "")?;
        let mut positions = vec![];
        if self.kind() == DEFAULT {
            positions.push(self.take());
        } else {
            positions.push(self.expect(NUM, "'DEFAULT'")?);
            while self.kind() == COMMA {
                self.i += 1;
                positions.push(self.expect(NUM, "")?);
            }
        }
        self.expect(SPACE, "")?;
        if self.kind() != INTERPOLATE {
            let s = self.styles()?;
            return self.db.update_link(&positions, s);
        }
        self.i += 1;
        self.expect(SPACE, "")?;
        let curve = self.alpha()?;
        if self.kind() == SPACE {
            self.i += 1;
            let s = self.styles()?;
            self.db.interpolate(&positions, &curve)?;
            self.db.update_link(&positions, s)
        } else {
            self.db.interpolate(&positions, &curve)
        }
    }
    fn link_tail(&mut self, id: &str, url: &str) -> Result<(), Error> {
        if self.kind() != SPACE {
            self.db.set_link(id, url, None);
            return Ok(());
        }
        self.i += 1;
        if self.kind() == LINK_TARGET {
            let target = self.take();
            self.db.set_link(id, url, Some(target));
            return Ok(());
        }
        let tooltip = self.expect(STR, "'LINK_TARGET'")?;
        let target = if self.kind() == SPACE {
            self.i += 1;
            Some(self.expect(LINK_TARGET, "")?)
        } else {
            None
        };
        self.db.set_link(id, url, target);
        self.db.tooltip(id, tooltip);
        Ok(())
    }
    fn click(&mut self) -> Result<(), Error> {
        let id = self.take();
        match self.kind() {
            CALLBACKNAME => {
                let name = self.take();
                let args = if self.kind() == CALLBACKARGS { Some(self.take()) } else { None };
                let tooltip = if self.kind() == SPACE {
                    self.i += 1;
                    Some(self.expect(STR, "")?)
                } else {
                    None
                };
                self.db.set_click_event(&id, &name, args.as_deref());
                if let Some(t) = tooltip {
                    self.db.tooltip(&id, t);
                }
            }
            HREF => {
                self.i += 1;
                let url = self.expect(STR, "")?;
                self.link_tail(&id, &url)?;
            }
            STR => {
                let url = self.take();
                self.link_tail(&id, &url)?;
            }
            _ => {
                let name = self.alpha()?;
                let tooltip = if self.kind() == SPACE {
                    self.i += 1;
                    Some(self.expect(STR, "")?)
                } else {
                    None
                };
                self.db.set_click_event(&id, &name, None);
                if let Some(t) = tooltip {
                    self.db.tooltip(&id, t);
                }
            }
        }
        Ok(())
    }
    fn header(&mut self) -> Result<Frame, Error> {
        self.i += 1;
        if matches!(self.kind(), NEWLINE | SEMI | EOF) {
            self.i += 1;
            return Ok(Frame::default());
        }
        self.expect(SPACE, "'SEMI', 'NEWLINE', 'EOF'")?;
        let id = self.no_tags()?;
        let mut title = id.clone();
        let mut same_title = true;
        if self.kind() == SQS {
            self.i += 1;
            title = self.text()?;
            self.expect(SQE, TEXT_TOKENS)?;
            same_title = false;
        }
        if !matches!(self.kind(), NEWLINE | SEMI | EOF) {
            return Err(self.fail("'SEMI', 'NEWLINE', 'EOF', 'SQS'"));
        }
        self.i += 1;
        Ok(Frame { id: Some(id), title: Some(title), same_title, ..Default::default() })
    }
    fn statement(&mut self) -> Result<DocItem, Error> {
        let t = self.kind();
        match t {
            STYLE => {
                self.i += 1;
                self.expect(SPACE, "")?;
                let id = self.id()?;
                self.expect(SPACE, "")?;
                let s = self.styles()?;
                self.db.vertex(&id, None, None, Some(s), None, None)?;
                self.separator()?;
            }
            LINKSTYLE => {
                self.link_style()?;
                self.separator()?;
            }
            CLASSDEF => {
                self.i += 1;
                self.expect(SPACE, "")?;
                let id = self.id()?;
                self.expect(SPACE, "")?;
                let s = self.styles()?;
                self.db.add_class(&id, s);
                self.separator()?;
            }
            CLASS => {
                self.i += 1;
                self.expect(SPACE, "")?;
                let ids = self.id()?;
                self.expect(SPACE, "")?;
                let class = self.id()?;
                self.db.set_class(&ids, &class);
                self.separator()?;
            }
            CLICK => {
                self.click()?;
                self.separator()?;
            }
            DIRECTION_TB..=DIRECTION_TD => {
                self.i += 1;
                return Ok(DocItem::Dir(["TB", "BT", "RL", "LR", "TD"][(t - DIRECTION_TB) as usize].into()));
            }
            ACC_TITLE | ACC_DESCR => {
                self.i += 1;
                let s = self.expect(if t == ACC_TITLE { ACC_TITLE_VALUE } else { ACC_DESCR_VALUE }, "")?;
                let value = js::trim(&s).to_string();
                if t == ACC_TITLE {
                    self.db.acc_title = Some(value.clone());
                } else {
                    self.db.acc_descr = Some(clean_description(&value));
                }
                return Ok(DocItem::Text(value));
            }
            ACC_DESCR_MULTILINE_VALUE => {
                let s = self.take();
                let value = js::trim(&s).to_string();
                self.db.acc_descr = Some(clean_description(&value));
                return Ok(DocItem::Text(value));
            }
            _ => {
                if !id_token(t) {
                    return Err(self.fail(STATEMENT_START));
                }
                let nodes = self.vertex_statement()?;
                self.separator()?;
                return Ok(DocItem::Nodes(nodes));
            }
        }
        Ok(DocItem::Nodes(vec![]))
    }
    fn document(&mut self) -> Result<(), Error> {
        let mut stack: Vec<Frame> = vec![];
        let mut list = vec![];
        let mut collect = false;
        loop {
            match self.kind() {
                END => break,
                END_KW => {
                    let Some(frame) = stack.pop() else {
                        break;
                    };
                    self.i += 1;
                    let id = self.db.add_subgraph(frame.id, list, frame.title, frame.same_title);
                    list = frame.list;
                    collect = frame.collect;
                    if collect {
                        list.push(DocItem::Text(id));
                    }
                }
                SEMI | NEWLINE | SPACE | EOF => {
                    let t = self.take();
                    if collect {
                        list.push(DocItem::Text(t));
                    }
                }
                SUBGRAPH_KW => {
                    let mut frame = self.header()?;
                    frame.list = list;
                    frame.collect = collect;
                    stack.push(frame);
                    list = vec![];
                    collect = true;
                }
                _ => {
                    let item = self.statement()?;
                    if collect && !matches!(&item,DocItem::Nodes(v)if v.is_empty()) {
                        list.push(item);
                    }
                }
            }
        }
        if !stack.is_empty() {
            return Err(self.fail("'end'"));
        }
        Ok(())
    }
}
fn clean_description(value: &str) -> String {
    crate::util::chars::normalize_accessibility_description(value)
}

fn brace_trailing(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut from = 0;
    for (at, c) in source.char_indices() {
        if at < from || c != '}' {
            continue;
        }
        let mut last = None;
        for (off, ch) in source[at + 1..].char_indices() {
            if !is_space(ch) {
                break;
            }
            if ch == '\n' {
                last = Some(at + 1 + off + 1);
            }
        }
        if let Some(end) = last {
            out.push_str(&source[from..at + 1]);
            out.push('\n');
            from = end;
        }
    }
    out.push_str(&source[from..]);
    out
}
pub fn parse(source: &str, db: &mut FlowchartModel) -> Result<(), Error> {
    let src = brace_trailing(source);
    let t = tokenize(&src);
    let mut p = Parser {
        src: &src,
        types: t.types,
        texts: t.texts.into_iter().map(|s| s.into_owned()).collect(),
        starts: t.starts,
        i: 0,
        db,
        pending: None,
    };
    while matches!(p.kind(), SPACE | NEWLINE) {
        p.i += 1;
    }
    p.expect(GRAPH, "'SPACE', 'NEWLINE'")?;
    if p.kind() == NODIR {
        p.i += 1;
        p.db.set_direction("TB");
    } else {
        let dir = p.expect(DIR, "'NODIR'")?;
        if p.kind() == SPACE {
            while p.kind() == SPACE {
                p.i += 1;
            }
            if p.kind() != NEWLINE {
                return Err(p.fail("'NEWLINE', 'SPACE'"));
            }
        } else if !matches!(p.kind(), SEMI | NEWLINE) {
            return Err(p.fail("'SEMI', 'NEWLINE', 'SPACE'"));
        }
        p.i += 1;
        p.db.set_direction(&dir);
    }
    p.document()?;
    if p.kind() != END {
        return Err(p.fail(STATEMENT_START));
    }
    Ok(())
}
