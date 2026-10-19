//! Three-token lookahead parser, preserving Mermaid token diagnostics and occurrences.
use super::{
    db::resolve,
    json::parse_ordered_json,
    lexer::{line_column, tokenize, Source, Token},
    tokens::*,
    types::*,
};
use crate::{util::js, ConfigValue, DiagramType, Error, ErrorCode};
const LINE: u8 = 1;
const RELATION: u8 = 2;
const STYLE_PART: u8 = 4;
const FOLLOWS: u8 = 8;
const BOUNDARY_LINE: u8 = 16;
const CLOSES: u8 = 32;
const META_FIRST: u8 = 64;
const ARROWS:&str="DEPENDENCY_ARROW|GENERALIZATION|FORWARD_SOLID|BACKWARD_SOLID|MARKERLESS_SOLID|FORWARD_CIRCLE|BACKWARD_CIRCLE|FORWARD_CROSS|BACKWARD_CROSS";
const NAME_LIST: &str = "IDENTIFIER|PLAIN_STRING|MARKDOWN_STRING";
const STATEMENTS: &str =
    "ACC_TITLE_LINE|ACC_DESCR_LINE|ACC_DESCR_BLOCK|DIRECTION|ACTOR|SYSTEM_BOUNDARY|NOTE|JSON_DECLARATION_START|CLASS_DEF|CLASS|STYLE";
const STYLE_PARTS:&str="WORD|PLAIN_STRING|NUMBER|HASH_COLOR|CSS_IDENTIFIER|CSS_ESCAPED_COMMA|DASH|DOT|PERCENT|CSS_PUNCTUATION|COLON|LPAREN|RPAREN|LBRACKET|RBRACKET|LBRACE|RBRACE|AT|MARKERLESS_SOLID";
fn word(k: u8) -> bool {
    (USECASE..=IDENTIFIER).contains(&k)
}
fn label_text(k: u8) -> bool {
    (USECASE..=LABEL_SYMBOL).contains(&k)
}
fn arrow(k: u8) -> bool {
    matches!(
        k,
        DEPENDENCY_ARROW
            | GENERALIZATION
            | FORWARD_SOLID
            | BACKWARD_SOLID
            | MARKERLESS_SOLID
            | FORWARD_CIRCLE
            | BACKWARD_CIRCLE
            | FORWARD_CROSS
            | BACKWARD_CROSS
    )
}
fn flags(k: u8) -> u8 {
    let mut f = 0;
    if matches!(
        k,
        NEWLINE
            | COMMENT
            | ACC_TITLE_LINE
            | ACC_DESCR_LINE
            | ACC_DESCR_BLOCK
            | DIRECTION
            | ACTOR
            | SYSTEM_BOUNDARY
            | NOTE
            | JSON_DECLARATION_START
            | CLASS_DEF
            | CLASS
            | STYLE
            | IDENTIFIER
            | PLAIN_STRING
            | MARKDOWN_STRING
    ) {
        f |= LINE;
    }
    if k == IDENTIFIER || arrow(k) {
        f |= RELATION;
    }
    if word(k)
        || matches!(
            k,
            PLAIN_STRING
                | NUMBER
                | HASH_COLOR
                | CSS_IDENTIFIER
                | CSS_ESCAPED_COMMA
                | DASH
                | DOT
                | PERCENT
                | CSS_PUNCTUATION
                | COLON
                | LPAREN
                | RPAREN
                | LBRACKET
                | RBRACKET
                | LBRACE
                | RBRACE
                | AT
                | MARKERLESS_SOLID
        )
    {
        f |= STYLE_PART;
    }
    if arrow(k) || matches!(k, STEREOTYPE_START | CLASS_SEPARATOR | IDENTIFIER | NEWLINE | EOF) {
        f |= FOLLOWS;
    }
    if matches!(k, NEWLINE | COMMENT | ACTOR | IDENTIFIER | PLAIN_STRING | MARKDOWN_STRING) {
        f |= BOUNDARY_LINE;
    }
    if matches!(k, FORWARD_SOLID | MARKERLESS_SOLID | FORWARD_CIRCLE | FORWARD_CROSS) {
        f |= CLOSES;
    }
    if matches!(k, NEWLINE | IDENTIFIER | PLAIN_STRING | RBRACE) {
        f |= META_FIRST;
    }
    f
}
#[derive(Clone, Default)]
struct Item {
    named: Named,
    shape: Option<String>,
    metadata: Option<Vec<Property>>,
    stereotype: Option<String>,
    stereotype_span: Option<Span>,
    classes: Vec<String>,
    class_spans: Vec<Span>,
    explicit: bool,
    node_span: Span,
}
struct Tail {
    id: Option<String>,
    id_span: Option<Span>,
    kind: String,
    arrow_type: u8,
    label: Option<Label>,
    minlen: f64,
    target: Item,
    span: Span,
}
struct Parser {
    src: Source,
    t: Vec<Token>,
    p: usize,
    last_end: usize,
    drafts: Drafts,
    parent: Option<usize>,
    pending: Vec<(usize, usize)>,
    edge_count: usize,
    note_count: usize,
    statements: Vec<Statement>,
}
fn generated_id(s: &str) -> String {
    s.encode_utf16().map(|c| if c < 128 && ((c as u8).is_ascii_alphanumeric() || c == 95) { c as u8 as char } else { '_' }).collect()
}
fn is_key(k: u8) -> bool {
    k == IDENTIFIER || k == PLAIN_STRING
}
fn occurrences(props: &[Property]) -> Vec<MetadataOccurrence> {
    props.iter().map(|p| p.occurrence.clone()).collect()
}
fn sequences() -> String {
    let names: Vec<_> = NAME_LIST.split('|').collect();
    let meta: Vec<_> = names
        .iter()
        .flat_map(|name| ["NEWLINE", "IDENTIFIER", "PLAIN_STRING", "RBRACE"].map(|next| format!("{name}, METADATA_START, {next}")))
        .collect();
    let follows = format!("STEREOTYPE_START|CLASS_SEPARATOR|IDENTIFIER|{ARROWS}|NEWLINE|EOF");
    let mut list = vec![STATEMENTS.to_owned()];
    list.extend(meta.clone());
    for (i, name) in names.iter().enumerate() {
        if i == 0 {
            list.push(format!("{name}, LPAREN"));
            list.push(format!("{name}, LBRACKET"));
        }
        for next in follows.split('|') {
            list.push(format!("{name}, {next}"));
        }
    }
    list.extend(meta);
    list.join("|")
}
impl Parser {
    fn kind(&self) -> u8 {
        self.t[self.p].kind
    }
    fn image(&self, k: usize) -> String {
        self.src.slice(self.t[k].start, self.t[k].end)
    }
    fn span(&self, k: usize) -> Span {
        [self.t[k].start, self.t[k].end]
    }
    fn fail(&self, detail: String) -> Error {
        let t = &self.t[self.p];
        let (line, column) = line_column(&self.src, t.start);
        let mut e = Error::new(
            format!("Error parsing usecase diagram: {detail} at line {line}, column {column} [{},{})", t.start, t.end),
            ErrorCode::Syntax,
        )
        .with_type(DiagramType::Usecase);
        e.line = line;
        e.column = column;
        e
    }
    fn mismatch(&self, name: &str) -> Error {
        self.fail(format!("Expecting token of type --> {name} <-- but found --> '{}' <--", self.image(self.p)))
    }
    fn expect(&mut self, k: u8) -> Result<usize, Error> {
        if self.kind() != k {
            return Err(self.mismatch(NAMES[k as usize]));
        }
        let at = self.p;
        self.p += 1;
        Ok(at)
    }
    fn no_alt(&self, expected: &str) -> Error {
        let list = expected.split('|').enumerate().map(|(i, s)| format!("  {}. [{s}]", i + 1)).collect::<Vec<_>>().join("\n");
        self.fail(format!("Expecting: one of these possible Token sequences:\n{list}\nbut found: '{}'", self.image(self.p)))
    }
    fn line_end(&mut self) -> Result<(), Error> {
        self.last_end = self.t[self.p - 1].end;
        if self.kind() == NEWLINE {
            self.p += 1;
        } else if self.kind() != EOF {
            return Err(self.no_alt("NEWLINE|EOF"));
        }
        Ok(())
    }
    fn content(&self, k: usize) -> Span {
        let trim = if self.t[k].kind == MARKDOWN_STRING {
            2
        } else if self.t[k].kind == PLAIN_STRING {
            1
        } else {
            0
        };
        [self.t[k].start + trim, self.t[k].end - trim]
    }
    fn token_label(&self, k: usize) -> Label {
        let at = self.content(k);
        Label {
            text: self.src.slice(at[0], at[1]),
            kind: if self.t[k].kind == MARKDOWN_STRING { "markdown" } else { "text" }.into(),
            span: at,
        }
    }
    fn label(&mut self) -> Result<Label, Error> {
        if matches!(self.kind(), PLAIN_STRING | MARKDOWN_STRING) {
            let out = self.token_label(self.p);
            self.p += 1;
            return Ok(out);
        }
        if !label_text(self.kind()) {
            return Err(self.no_alt("PLAIN_STRING|MARKDOWN_STRING|LABEL_TEXT"));
        }
        let first = self.p;
        while label_text(self.kind()) {
            self.p += 1;
        }
        let at = [self.t[first].start, self.t[self.p - 1].end];
        Ok(Label { text: self.src.slice(at[0], at[1]), kind: "text".into(), span: at })
    }
    fn property(&mut self) -> Result<Property, Error> {
        let key = self.p;
        if !is_key(self.kind()) {
            return Err(self.no_alt("IDENTIFIER|PLAIN_STRING"));
        }
        self.p += 1;
        self.expect(COLON)?;
        let value = self.p;
        let kind = self.kind();
        if !is_key(kind) && kind != TRUE && kind != FALSE {
            return Err(self.no_alt("IDENTIFIER|PLAIN_STRING|TRUE|FALSE"));
        }
        self.p += 1;
        let ks = self.content(key);
        let vs = self.content(value);
        Ok(Property {
            value: if kind == TRUE {
                ConfigValue::Bool(true)
            } else if kind == FALSE {
                ConfigValue::Bool(false)
            } else {
                ConfigValue::String(self.src.slice(vs[0], vs[1]))
            },
            occurrence: MetadataOccurrence {
                key: self.src.slice(ks[0], ks[1]),
                span: [self.t[key].start, self.t[value].end],
                key_span: ks,
                value_span: vs,
            },
        })
    }
    fn separator_ahead(&self) -> bool {
        let (a, b, c) = (self.kind(), self.t[self.p + 1].kind, self.t[self.p + 2].kind);
        if a == COMMA {
            return is_key(b) || (b == NEWLINE && (c == NEWLINE || is_key(c)));
        }
        if a != NEWLINE {
            return false;
        }
        b == COMMA || is_key(b) || (b == NEWLINE && (c == NEWLINE || c == COMMA || is_key(c)))
    }
    fn metadata(&mut self) -> Result<Vec<Property>, Error> {
        self.expect(METADATA_START)?;
        let mut props = vec![];
        while self.kind() == NEWLINE {
            self.p += 1;
        }
        if is_key(self.kind()) {
            props.push(self.property()?);
            while self.separator_ahead() {
                if self.kind() == COMMA {
                    self.p += 1;
                } else {
                    while self.kind() == NEWLINE {
                        self.p += 1;
                    }
                    if self.kind() == COMMA {
                        self.p += 1;
                    }
                }
                while self.kind() == NEWLINE {
                    self.p += 1;
                }
                props.push(self.property()?);
            }
            if self.kind() == COMMA {
                self.p += 1;
            }
            while self.kind() == NEWLINE {
                self.p += 1;
            }
        }
        self.expect(RBRACE)?;
        Ok(props)
    }
    fn class_suffix(&mut self, classes: &mut Vec<String>, spans: &mut Vec<Span>) -> Result<(), Error> {
        loop {
            self.p += 1;
            let id = self.expect(IDENTIFIER)?;
            classes.push(self.image(id));
            spans.push(self.span(id));
            if self.kind() != COMMA {
                break;
            }
        }
        Ok(())
    }
    fn name(&mut self, brackets: bool) -> Result<Item, Error> {
        let k = self.p;
        let kind = self.kind();
        let mut item =
            Item { named: Named { span: self.content(k), generated: kind != IDENTIFIER, ..Default::default() }, ..Default::default() };
        if kind == IDENTIFIER {
            self.p += 1;
            item.named.id = self.image(k);
            let open = self.kind();
            if open == LPAREN || (brackets && open == LBRACKET) {
                self.p += 1;
                item.named.label = self.label()?;
                self.expect(if open == LPAREN { RPAREN } else { RBRACKET })?;
                item.shape = Some(if open == LPAREN { "ellipse" } else { "rect" }.into());
            } else {
                item.named.label = self.token_label(k);
            }
        } else if matches!(kind, PLAIN_STRING | MARKDOWN_STRING) {
            self.p += 1;
            item.named.label = self.token_label(k);
            item.named.id = generated_id(&item.named.label.text);
        } else {
            return Err(self.no_alt(NAME_LIST));
        }
        Ok(item)
    }
    fn element(&mut self, actor: bool) -> Result<Item, Error> {
        let first = self.p;
        let mut item = self.name(!actor)?;
        if actor {
            item.shape = None;
        }
        if self.kind() == METADATA_START {
            item.metadata = Some(self.metadata()?);
        }
        if self.kind() == STEREOTYPE_START {
            self.p += 1;
            let text = self.expect(STEREOTYPE_TEXT)?;
            self.expect(STEREOTYPE_END)?;
            item.stereotype = Some(js::trim(&self.image(text)).into());
            item.stereotype_span = Some(self.span(text));
        }
        if self.kind() == CLASS_SEPARATOR {
            self.class_suffix(&mut item.classes, &mut item.class_spans)?;
        }
        item.explicit = item.shape.is_some() || item.metadata.is_some() || item.stereotype.as_ref().is_some_and(|s| !s.is_empty());
        item.node_span = [self.t[first].start, self.t[self.p - 1].end];
        Ok(item)
    }
    fn draft(&self, item: &Item, kind: &str) -> DraftElement {
        DraftElement {
            named: item.named.clone(),
            kind: kind.into(),
            parent: self.parent,
            shape: item.shape.clone(),
            metadata: item.metadata.clone(),
            stereotype: item.stereotype.clone(),
            classes: item.classes.clone(),
        }
    }
    fn endpoint(item: &Item, declares: bool) -> Endpoint {
        Endpoint { named: item.named.clone(), declares, classed: !item.classes.is_empty() }
    }
    fn occurrence(item: &Item, defines: bool) -> NodeOccurrence {
        NodeOccurrence {
            id: item.named.id.clone(),
            span: item.node_span,
            id_span: item.named.span,
            label_span: Some(item.named.label.span),
            defines,
            stereotype_span: item.stereotype_span,
            metadata: item.metadata.as_ref().map(|p| occurrences(p)),
            class_spans: item.class_spans.clone(),
        }
    }
    fn labeled_right(&self, closers: u8) -> bool {
        if !(USECASE..=MARKDOWN_STRING).contains(&self.kind()) {
            return false;
        }
        let mut k = self.p + 1;
        loop {
            let kind = self.t[k].kind;
            if kind == MARKERLESS_SOLID || flags(kind) & closers != 0 {
                return true;
            }
            if !(USECASE..=MARKDOWN_STRING).contains(&kind) {
                return false;
            }
            k += 1;
        }
    }
    fn minlen(&self, k: usize) -> f64 {
        let dashes = (self.t[k].end - self.t[k].start) as f64 - if self.t[k].kind == MARKERLESS_SOLID { 0.0 } else { 1.0 };
        js::max(1.0, dashes - 1.0)
    }
    fn relation_tail(&mut self) -> Result<Tail, Error> {
        let first = self.p;
        let mut tail = Tail {
            id: None,
            id_span: None,
            kind: "association".into(),
            arrow_type: 0,
            label: None,
            minlen: 1.0,
            target: Item::default(),
            span: [0, 0],
        };
        if self.kind() == IDENTIFIER && self.t[self.p + 1].kind == AT {
            tail.id = Some(self.image(self.p));
            tail.id_span = Some(self.span(self.p));
            self.p += 2;
        }
        let k = self.p;
        let kind = self.kind();
        self.p += 1;
        match kind {
            DEPENDENCY_ARROW => {
                self.expect(COLON)?;
                if !matches!(self.kind(), INCLUDE | EXTEND) {
                    return Err(self.no_alt("INCLUDE|EXTEND"));
                }
                tail.kind = if self.kind() == INCLUDE { "include" } else { "extend" }.into();
                tail.label = Some(Label { text: tail.kind.clone(), kind: "text".into(), span: self.span(self.p) });
                self.p += 1;
            }
            GENERALIZATION => tail.kind = "generalization".into(),
            FORWARD_SOLID => tail.minlen = self.minlen(k),
            BACKWARD_SOLID => {
                tail.arrow_type = 1;
                if self.t[k].end - self.t[k].start == 3 && self.labeled_right(0) {
                    tail.label = Some(self.label()?);
                    let close = self.expect(MARKERLESS_SOLID)?;
                    tail.minlen = self.minlen(close);
                } else {
                    tail.minlen = self.minlen(k);
                }
            }
            MARKERLESS_SOLID => {
                tail.arrow_type = 2;
                if self.t[k].end - self.t[k].start == 2 && self.labeled_right(CLOSES) {
                    tail.label = Some(self.label()?);
                    tail.arrow_type = match self.kind() {
                        FORWARD_SOLID => 0,
                        FORWARD_CIRCLE => 3,
                        FORWARD_CROSS => 4,
                        MARKERLESS_SOLID => 2,
                        _ => return Err(self.no_alt("FORWARD_SOLID|MARKERLESS_SOLID|FORWARD_CIRCLE|FORWARD_CROSS")),
                    };
                    if tail.arrow_type == 0 || tail.arrow_type == 2 {
                        tail.minlen = self.minlen(self.p);
                    }
                    self.p += 1;
                } else {
                    tail.minlen = self.minlen(k);
                }
            }
            FORWARD_CIRCLE => tail.arrow_type = 3,
            FORWARD_CROSS => tail.arrow_type = 4,
            BACKWARD_CIRCLE | BACKWARD_CROSS => {
                tail.arrow_type = if kind == BACKWARD_CIRCLE { 5 } else { 6 };
                if self.labeled_right(0) {
                    tail.label = Some(self.label()?);
                    self.expect(MARKERLESS_SOLID)?;
                }
            }
            _ => {
                self.p -= 1;
                return Err(self.no_alt(ARROWS));
            }
        }
        tail.target = self.element(false)?;
        tail.span = [self.t[first].start, self.t[self.p - 1].end];
        Ok(tail)
    }
    fn add(&mut self, kind: &str) -> usize {
        let i = self.statements.len();
        self.statements.push(Statement { kind: kind.into(), ..Default::default() });
        i
    }
    fn relationship(&mut self, source: &Item, declaration: bool, tail: Tail, mut nodes: Vec<NodeOccurrence>) -> usize {
        let target = &tail.target;
        if target.explicit {
            self.drafts.elements.push(self.draft(target, "usecase"));
        }
        self.drafts.relationships.push(DraftRelationship {
            source: Self::endpoint(source, declaration),
            target: Self::endpoint(target, target.explicit),
            span: tail.span,
            id: tail.id.clone(),
            id_span: tail.id_span,
            kind: tail.kind,
            arrow_type: tail.arrow_type,
            label: tail.label.clone(),
            minlen: tail.minlen,
        });
        nodes.push(Self::occurrence(target, target.explicit));
        let out = self.add("edge");
        let id = tail.id.unwrap_or_else(|| {
            let id = format!("edge-{}", self.edge_count);
            self.edge_count += 1;
            id
        });
        self.statements[out].nodes = Some(nodes);
        self.statements[out].edges =
            Some(vec![EdgeOccurrence { id, id_span: tail.id_span, label_span: tail.label.map(|l| l.span), ..Default::default() }]);
        out
    }
    fn declaration(&mut self, nodes: Vec<NodeOccurrence>) -> usize {
        let out = self.add("node");
        self.statements[out].nodes = Some(nodes);
        out
    }
    fn actors(&mut self, top: bool) -> Result<usize, Error> {
        self.p += 1;
        let mut items = vec![self.element(true)?];
        let mut tail = None;
        if self.kind() == COMMA {
            while self.kind() == COMMA {
                self.p += 1;
                items.push(self.element(true)?);
            }
        } else if top && flags(self.kind()) & RELATION != 0 {
            tail = Some(self.relation_tail()?);
        }
        self.line_end()?;
        for item in &items {
            self.drafts.elements.push(self.draft(item, "actor"));
        }
        let nodes = items.iter().map(|item| Self::occurrence(item, true)).collect();
        Ok(if let Some(tail) = tail { self.relationship(&items[0], true, tail, nodes) } else { self.declaration(nodes) })
    }
    fn entity(&mut self, top: bool) -> Result<usize, Error> {
        let mut source = self.element(false)?;
        let tail = if top && flags(self.kind()) & RELATION != 0 { Some(self.relation_tail()?) } else { None };
        self.line_end()?;
        if tail.is_none() {
            source.explicit = true;
            source.shape.get_or_insert_with(|| "ellipse".into());
        }
        if source.explicit {
            self.drafts.elements.push(self.draft(&source, "usecase"));
        }
        let nodes = vec![Self::occurrence(&source, source.explicit)];
        Ok(if let Some(tail) = tail { self.relationship(&source, source.explicit, tail, nodes) } else { self.declaration(nodes) })
    }
    fn boundary(&mut self) -> Result<usize, Error> {
        self.p += 1;
        let mut named = self.name(true)?;
        let inline = if self.kind() == METADATA_START { Some(self.metadata()?) } else { None };
        if self.kind() == CLASS_SEPARATOR {
            self.class_suffix(&mut named.classes, &mut named.class_spans)?;
        }
        self.line_end()?;
        let group = self.drafts.boundaries.len();
        self.drafts.boundaries.push(DraftBoundary { named: named.named.clone(), classes: named.classes.clone() });
        self.parent = Some(group);
        let mut children = vec![];
        while flags(self.kind()) & BOUNDARY_LINE != 0 {
            children.push(self.line(false)?);
        }
        self.parent = None;
        let end = self.expect(END)?;
        self.line_end()?;
        let out = self.add("group");
        let end_span = self.span(end);
        let stmt = &mut self.statements[out];
        stmt.group = Some(named.named.id.clone());
        stmt.id_span = Some(named.named.span);
        stmt.title_span = Some(named.named.label.span);
        stmt.end_span = Some(end_span);
        stmt.class_spans = named.class_spans;
        stmt.children = children;
        if let Some(inline) = inline {
            stmt.metadata = Some(occurrences(&inline));
            self.drafts.metadata.push(DraftMetadata { target: named.named.id, span: named.named.span, properties: inline, statement: out });
        }
        Ok(out)
    }
    fn style(&mut self) -> Result<String, Error> {
        let first = self.p;
        let kind = self.kind();
        if word(kind) || kind == CSS_IDENTIFIER {
            self.p += 1;
        } else if kind == MARKERLESS_SOLID {
            self.p += 1;
            if !word(self.kind()) {
                return Err(self.mismatch("WORD"));
            }
            self.p += 1;
        } else {
            return Err(self.no_alt("WORD|CSS_IDENTIFIER|MARKERLESS_SOLID"));
        }
        self.expect(COLON)?;
        if flags(self.kind()) & STYLE_PART == 0 {
            return Err(self.fail(format!("Expecting: expecting at least one iteration which starts with one of these possible Token sequences::\n  <[{}]>\nbut found: '{}'",STYLE_PARTS.replace('|',"] ,["),self.image(self.p))));
        }
        while flags(self.kind()) & STYLE_PART != 0 {
            self.p += 1;
        }
        Ok(self.src.slice(self.t[first].start, self.t[self.p - 1].end).replace("\\,", ","))
    }
    fn styles(&mut self) -> Result<Vec<String>, Error> {
        let mut out = vec![self.style()?];
        while self.kind() == COMMA {
            self.p += 1;
            out.push(self.style()?);
        }
        Ok(out)
    }
    fn identifiers(&mut self) -> Result<Vec<usize>, Error> {
        let mut out = vec![self.expect(IDENTIFIER)?];
        while self.kind() == COMMA {
            self.p += 1;
            out.push(self.expect(IDENTIFIER)?);
        }
        Ok(out)
    }
    fn reference(&self, k: usize) -> NodeOccurrence {
        let at = self.content(k);
        NodeOccurrence { id: self.image(k), span: at, id_span: at, ..Default::default() }
    }
    fn top_statement(&mut self) -> Result<usize, Error> {
        let k = self.p;
        let kind = self.kind();
        let out;
        if matches!(kind, ACC_TITLE_LINE | ACC_DESCR_LINE | ACC_DESCR_BLOCK) {
            self.p += 1;
            let text = self.image(k);
            if kind == ACC_DESCR_BLOCK {
                self.drafts.acc_description = js::trim(&text[text.find('{').unwrap() + 1..text.rfind('}').unwrap()]).into();
            } else if kind == ACC_DESCR_LINE {
                self.drafts.acc_description = js::trim(&text[text.find(':').unwrap() + 1..]).into();
            } else {
                self.drafts.acc_title = js::trim(&text[text.find(':').unwrap() + 1..]).into();
            }
            out = self.add(if kind == ACC_TITLE_LINE { "accTitle" } else { "accDescr" });
        } else if kind == DIRECTION {
            self.p += 1;
            let value = self.kind();
            if !(TD..=RL).contains(&value) {
                return Err(self.no_alt("TD|TB|BT|LR|RL"));
            }
            self.drafts.direction = Some(if value == TD { "TB".into() } else { self.image(self.p) });
            self.p += 1;
            out = self.add("direction");
        } else if kind == ACTOR {
            return self.actors(true);
        } else if kind == SYSTEM_BOUNDARY {
            return self.boundary();
        } else if kind == NOTE {
            self.p += 1;
            self.expect(FOR)?;
            let target = self.expect(IDENTIFIER)?;
            let text = self.label()?;
            self.drafts.notes.push(DraftNote { target: self.image(target), span: self.span(target), label: text.clone() });
            out = self.add("note");
            let node = self.reference(target);
            let stmt = &mut self.statements[out];
            stmt.reference = Some(format!("note-{}", self.note_count));
            self.note_count += 1;
            stmt.ref_span = Some(text.span);
            stmt.nodes = Some(vec![node]);
        } else if kind == JSON_DECLARATION_START {
            self.p += 1;
            let literal = self.expect(JSON_OBJECT_LITERAL)?;
            let (mut classes, mut class_spans) = (vec![], vec![]);
            if self.kind() == CLASS_SEPARATOR {
                self.class_suffix(&mut classes, &mut class_spans)?;
            }
            let mut from = self.t[k].start + 4;
            while matches!(self.src.code(from), 32 | 9) {
                from += 1;
            }
            let mut to = from;
            while self.src.code(to) < 128 && ((self.src.code(to) as u8).is_ascii_alphanumeric() || self.src.code(to) == 95) {
                to += 1;
            }
            let id = self.src.slice(from, to);
            let index = self.drafts.jsons.len();
            self.drafts.jsons.push(DraftJson { id: id.clone(), value: OrderedJson::default(), span: [from, to], classes });
            self.pending.push((index, literal));
            out = self.add("json");
            self.statements[out].nodes = Some(vec![NodeOccurrence {
                id,
                span: [from, to],
                id_span: [from, to],
                defines: true,
                class_spans: class_spans.clone(),
                ..Default::default()
            }]);
            self.statements[out].class_spans = class_spans;
        } else if kind == CLASS_DEF {
            self.p += 1;
            let ids = self.identifiers()?;
            let styles = self.styles()?;
            self.drafts.class_defs.push((ids.iter().map(|&k| self.image(k)).collect(), styles));
            out = self.add("classDef");
            self.statements[out].reference = Some(self.image(ids[0]));
            self.statements[out].ref_span = Some(self.span(ids[0]));
        } else if kind == CLASS {
            self.p += 1;
            let targets = self.identifiers()?;
            let classes = self.identifiers()?;
            self.drafts.classes.push(ClassAssign {
                targets: targets.iter().map(|&k| (self.image(k), self.span(k))).collect(),
                classes: classes.iter().map(|&k| self.image(k)).collect(),
            });
            out = self.add("classAssign");
            self.statements[out].reference = Some(self.image(classes[0]));
            self.statements[out].ref_span = Some(self.span(classes[0]));
            self.statements[out].nodes = Some(targets.iter().map(|&k| self.reference(k)).collect());
        } else if kind == STYLE {
            self.p += 1;
            let target = self.expect(IDENTIFIER)?;
            let styles = self.styles()?;
            self.drafts.styles.push(StyleAssign { target: self.image(target), span: self.span(target), styles });
            out = self.add("style");
            self.statements[out].nodes = Some(vec![self.reference(target)]);
        } else {
            let next = self.t[self.p + 1].kind;
            if next == METADATA_START {
                if flags(self.t[self.p + 2].kind) & META_FIRST == 0 {
                    return Err(self.no_alt(&sequences()));
                }
                self.p += 1;
                let mut target = self.reference(k);
                if kind != IDENTIFIER {
                    target.id = generated_id(&self.src.slice(target.span[0], target.span[1]));
                }
                let assigned = self.metadata()?;
                out = self.add("metadata");
                self.statements[out].nodes = Some(vec![target.clone()]);
                self.statements[out].metadata = Some(occurrences(&assigned));
                self.drafts.metadata.push(DraftMetadata { target: target.id, span: target.span, properties: assigned, statement: out });
            } else {
                let follows = flags(next) & FOLLOWS != 0 || (kind == IDENTIFIER && matches!(next, LPAREN | LBRACKET));
                if !follows
                    || (kind == IDENTIFIER
                        && matches!(
                            self.image(k).to_lowercase().as_str(),
                            "allowmixing" | "newpage" | "package" | "rectangle" | "skinparam"
                        ))
                {
                    return Err(self.no_alt(&sequences()));
                }
                return self.entity(true);
            }
        }
        self.line_end()?;
        Ok(out)
    }
    fn line(&mut self, top: bool) -> Result<usize, Error> {
        let k = self.p;
        let kind = self.kind();
        if kind == NEWLINE {
            self.p += 1;
            let out = self.add("blank");
            self.statements[out].span = self.span(k);
            return Ok(out);
        }
        if kind == COMMENT {
            self.p += 1;
            self.line_end()?;
            let out = self.add("comment");
            self.statements[out].span = self.span(k);
            return Ok(out);
        }
        let out = if top {
            self.top_statement()?
        } else if kind == ACTOR {
            self.actors(false)?
        } else {
            self.entity(false)?
        };
        let span = [self.t[k].start, self.last_end];
        self.statements[out].span = span;
        if let Some(edges) = &mut self.statements[out].edges {
            for e in edges {
                e.span = span;
            }
        }
        Ok(out)
    }
}
pub fn parse(source: &str, title: Option<&str>) -> Result<UsecaseModel, Error> {
    let src = Source::new(source);
    let tokens = tokenize(&src)?;
    let mut p = Parser {
        src,
        t: tokens,
        p: 0,
        last_end: 0,
        drafts: Drafts::default(),
        parent: None,
        pending: vec![],
        edge_count: 0,
        note_count: 0,
        statements: vec![],
    };
    let header = p.expect(USECASE)?;
    p.line_end()?;
    let mut roots = vec![];
    while flags(p.kind()) & LINE != 0 {
        roots.push(p.line(true)?);
    }
    if p.kind() != EOF {
        return Err(p.fail(format!("Redundant input, expecting EOF but found: {}", p.image(p.p))));
    }
    for &(json, literal) in &p.pending {
        p.drafts.jsons[json].value = parse_ordered_json(&p.image(literal), line_column(&p.src, p.t[literal].start))?;
    }
    let span = p.span(header);
    resolve(source, &p.src, p.drafts, span, p.statements, roots, title)
}
