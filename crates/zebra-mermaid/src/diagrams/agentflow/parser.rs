//! Iterative agentflow parser, including retained comments, blocks and source mappings.
use super::{db::AgentflowModel, lexer::tokenize, tokens::*, types::*};
use crate::error::syntax_error;
use crate::util::{chars::is_space, js};
use crate::{DiagramType, Error};
use indexmap::IndexMap;

const TEXT_TOKENS: &str = "'UNICODE_TEXT', 'TEXT', 'TAGSTART'";
const STATEMENT_START:&str="'COMMENT', 'SEMI', 'NEWLINE', 'SPACE', 'EOF', 'flow', 'global', 'connector', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'STYLE', 'LINKSTYLE', 'CLASSDEF', 'CLASS', 'CLICK', 'NODE_STRING'";
const SEPARATORS: &str = "'COMMENT', 'SEMI', 'NEWLINE', 'EOF'";
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
            | FLOW_KW
            | CONNECTOR_KW
            | GLOBAL_KW
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
    locs: Vec<Loc>,
    data_from: usize,
    data_to: usize,
    i: usize,
    db: &'a mut AgentflowModel,
    pending: Option<String>,
}
#[derive(Default)]
struct Frame {
    id: Option<FlowText>,
    title: Option<FlowText>,
    global: bool,
    at: usize,
    data: Option<String>,
    data_from: usize,
    data_to: usize,
    list: Vec<DocItem>,
    collect: bool,
}
impl Parser<'_> {
    fn kind(&self) -> u8 {
        self.types.get(self.i).copied().unwrap_or(END)
    }
    fn take(&mut self) -> String {
        let s = self.texts.get_mut(self.i).map(std::mem::take).unwrap_or_default();
        self.i += 1;
        s
    }
    fn append_next(&mut self, out: &mut String) {
        if let Some(text) = self.texts.get_mut(self.i) {
            out.push_str(&std::mem::take(text));
        }
        self.i += 1;
    }
    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = js::byte_at16(self.src, self.starts.get(self.i).copied().unwrap_or(js::len16(self.src)));
        if t == ERROR {
            syntax_error(DiagramType::Agentflow, self.src, at, "", true)
        } else {
            syntax_error(DiagramType::Agentflow, self.src, at, &format!("Expecting {expected}, got '{}'", TOKEN_NAMES[t as usize]), false)
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
        if !matches!(self.kind(), NEWLINE | SEMI | EOF | COMMENT) {
            return Err(self.fail(SEPARATORS));
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
            self.append_next(&mut s);
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
            self.append_next(&mut text);
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
            self.append_next(&mut text);
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
            self.append_next(&mut text);
        }
        Ok(FlowText { text, kind: kind.into() })
    }
    fn shape_data(&mut self) -> String {
        self.data_from = self.i;
        let mut s = self.take();
        while self.kind() == SHAPE_DATA {
            self.append_next(&mut s);
        }
        self.data_to = self.i - 1;
        s
    }
    fn vertex(&mut self) -> Result<String, Error> {
        let from = self.i;
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
                self.db.vertex(&id, Some(label), Some(shape), None, None, None, None)?;
                self.db.vertex_mapping(&id, Some(shape), self.span(from, self.i - 1));
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
            self.db.vertex(&id, Some(text), Some(shape), None, props, None, None)?;
            self.db.vertex_mapping(&id, Some(shape), self.span(from, self.i - 1));
        } else {
            self.db.vertex(&id, None, None, None, None, None, None)?;
            self.db.vertex_mapping(&id, None, self.span(from, self.i - 1));
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
            let (from, to) = (self.data_from, self.data_to);
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
                self.attach(ids.last().unwrap(), &data, from, to)?;
            }
            ids.push(next);
        }
    }
    fn link(&mut self) -> Result<Link, Error> {
        let id = if self.kind() == LINK_ID { Some(self.take()) } else { None };
        let mut link = if self.kind() == LINK {
            let raw = self.take();
            let mut l = AgentflowModel::destruct_link(&raw, None);
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
            let mut l = AgentflowModel::destruct_link(&end, Some(&start));
            l.text = Some(text);
            l
        } else {
            return Err(self.fail("'LINK', 'START_LINK'"));
        };
        link.id = id;
        Ok(link)
    }
    fn vertex_statement(&mut self) -> Result<Vec<String>, Error> {
        let from = self.i;
        let mut stmt = self.node()?;
        if let Some(raw) = self.pending.take() {
            self.attach(stmt.last().unwrap(), &raw, self.data_from, self.data_to)?;
        }
        let mut nodes = stmt.clone();
        while matches!(self.kind(), LINK | START_LINK | LINK_ID) {
            let link = self.link()?;
            let next = self.node()?;
            if let Some(raw) = self.pending.take() {
                self.attach(next.last().unwrap(), &raw, self.data_from, self.data_to)?;
            }
            self.db.add_link(&stmt, &next, &link)?;
            self.db.edge_mapping(&next, self.span(from, self.i - 1));
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
            self.db.set_link(id, url, Some(&target));
            return Ok(());
        }
        let tooltip = self.expect(STR, "'LINK_TARGET'")?;
        let target = if self.kind() == SPACE {
            self.i += 1;
            Some(self.expect(LINK_TARGET, "")?)
        } else {
            None
        };
        self.db.set_link(id, url, target.as_deref());
        self.db.tooltip(id, &tooltip);
        Ok(())
    }
    fn click(&mut self) -> Result<(), Error> {
        let id = self.take();
        match self.kind() {
            CALLBACKNAME => {
                self.i += 1;
                if self.kind() == CALLBACKARGS {
                    self.i += 1;
                }
                let tooltip = if self.kind() == SPACE {
                    self.i += 1;
                    Some(self.expect(STR, "")?)
                } else {
                    None
                };
                self.db.set_class(&id, "clickable");
                if let Some(t) = tooltip {
                    self.db.tooltip(&id, &t);
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
                self.alpha()?;
                let tooltip = if self.kind() == SPACE {
                    self.i += 1;
                    Some(self.expect(STR, "")?)
                } else {
                    None
                };
                self.db.set_class(&id, "clickable");
                if let Some(t) = tooltip {
                    self.db.tooltip(&id, &t);
                }
            }
        }
        Ok(())
    }
    fn span(&self, a: usize, b: usize) -> Loc {
        let first = self.locs[a];
        let last = self.locs[b];
        Loc { first_line: first.first_line, first_column: first.first_column, last_line: last.last_line, last_column: last.last_column }
    }
    fn attach(&mut self, id: &str, data: &str, from: usize, to: usize) -> Result<(), Error> {
        let loc = self.span(from, to);
        self.db.vertex(id, None, None, None, None, Some(data), Some(loc))?;
        self.db.extend_mapping(id, loc);
        Ok(())
    }
    fn attach_data(&mut self, id: &str, frame: &Frame) -> Result<(), Error> {
        if let Some(data) = &frame.data {
            self.db.vertex(id, None, None, None, None, Some(data), Some(self.span(frame.data_from, frame.data_to)))?;
        }
        Ok(())
    }
    fn header(&mut self, at: usize) -> Result<Frame, Error> {
        self.expect(SPACE, SEPARATORS)?;
        let id = self.no_tags()?;
        let mut title = FlowText { text: String::new(), kind: "text".into() };
        if self.kind() == SQS {
            self.i += 1;
            title = self.text()?;
            self.expect(SQE, TEXT_TOKENS)?;
        }
        let data = if self.kind() == SHAPE_DATA { Some(self.shape_data()) } else { None };
        if !matches!(self.kind(), COMMENT | SEMI | NEWLINE | EOF) {
            return Err(self.fail(&format!("{SEPARATORS}, 'SHAPE_DATA'")));
        }
        Ok(Frame { id: Some(id), title: Some(title), at, data, data_from: self.data_from, data_to: self.data_to, ..Default::default() })
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
                self.db.vertex(&id, None, None, Some(s), None, None, None)?;
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
            CONNECTOR_KW => {
                let at = self.i;
                self.i += 1;
                let frame = self.header(at)?;
                let sep = self.i;
                self.i += 1;
                let id = self.db.add_connector(frame.id.as_ref(), frame.title.as_ref());
                self.attach_data(&id, &frame)?;
                let name = frame.id.as_ref().map_or("", |t| js::trim(&t.text));
                self.db.block_mapping(name, "connector", self.span(frame.at, frame.at), self.span(sep, sep));
                return Ok(DocItem::Text(id));
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
        let mut open: Vec<Frame> = vec![];
        let mut list = vec![];
        let mut collect = false;
        loop {
            let t = self.kind();
            if t == END {
                break;
            }
            if t == END_KW {
                let Some(frame) = open.pop() else {
                    break;
                };
                let at = self.i;
                self.i += 1;
                if frame.global {
                    self.db.add_global(&list);
                    list = frame.list;
                    collect = frame.collect;
                    continue;
                }
                let id = self.db.add_subgraph(frame.id.as_ref(), &list, frame.title.as_ref(), false);
                self.attach_data(&id, &frame)?;
                let name = frame.id.as_ref().map_or("", |t| t.text.as_str());
                self.db.block_mapping(name, "subgraph", self.span(frame.at, frame.at), self.span(at, at));
                list = frame.list;
                collect = frame.collect;
                if collect {
                    list.push(DocItem::Text(id));
                }
            } else if matches!(t, SEMI | NEWLINE | SPACE | EOF) {
                let text = self.take();
                if collect {
                    list.push(DocItem::Text(text));
                }
            } else if t == COMMENT {
                self.i += 1;
            } else if t == FLOW_KW || t == GLOBAL_KW {
                let at = self.i;
                self.i += 1;
                let mut frame;
                if t == GLOBAL_KW || matches!(self.kind(), COMMENT | SEMI | NEWLINE | EOF) {
                    frame = Frame { global: t == GLOBAL_KW, at, ..Default::default() };
                    self.separator()?;
                } else {
                    frame = self.header(at)?;
                    self.i += 1;
                }
                frame.list = list;
                frame.collect = collect;
                open.push(frame);
                list = vec![];
                collect = true;
            } else {
                let item = self.statement()?;
                if collect && !matches!(&item,DocItem::Nodes(v)if v.is_empty()) {
                    list.push(item);
                }
            }
        }
        if !open.is_empty() {
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
                break;
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
pub fn parse(source: &str, db: &mut AgentflowModel, raw: bool) -> Result<(), Error> {
    let src = if raw { source.to_owned() } else { brace_trailing(source) };
    let t = tokenize(&src);
    let mut p = Parser {
        src: &src,
        types: t.types,
        texts: t.texts,
        locs: t.locs,
        data_from: 0,
        data_to: 0,
        starts: t.starts,
        i: 0,
        db,
        pending: None,
    };
    while matches!(p.kind(), SPACE | NEWLINE | COMMENT) {
        p.i += 1;
    }
    p.expect(GRAPH, "'SPACE', 'NEWLINE', 'COMMENT'")?;
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
        } else if !matches!(p.kind(), SEMI | NEWLINE | COMMENT) {
            return Err(p.fail("'COMMENT', 'SEMI', 'NEWLINE', 'SPACE'"));
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
