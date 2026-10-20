//! Block diagram parser and the database that builds the block tree from its statements.
//! Ported from Pele `src/diagrams/block/parser.ts` and `db.ts`.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::lexer::{token_name, tokenize, Tokens, T};
use super::{Block, BlockClassDef, BlockModel};
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

const STATEMENT_START: &str = "'NODE_ID', 'COLUMNS', 'SPACE_BLOCK', 'BLOCK_DIAGRAM_KEY', 'id-block', 'classDef', 'class', 'style'";

const MAX_SPACES: f64 = 100000.0;

pub(super) fn type_str_to_type(type_str: Option<&str>) -> &'static str {
    match type_str {
        Some("[]") => "square",
        Some("()") => "round",
        Some("(())") => "circle",
        Some(">]") => "rect_left_inv_arrow",
        Some("{}") => "diamond",
        Some("{{}}") => "hexagon",
        Some("([])") => "stadium",
        Some("[[]]") => "subroutine",
        Some("[()]") => "cylinder",
        Some("((()))") => "doublecircle",
        Some("[//]") => "lean_right",
        Some("[\\\\]") => "lean_left",
        Some("[/\\]") => "trapezoid",
        Some("[\\/]") => "inv_trapezoid",
        Some("<[]>") => "block_arrow",
        _ => "na",
    }
}

fn edge_str_to_edge_data(type_str: &str) -> &'static str {
    match js::trim(type_str).chars().next_back() {
        Some('x') => "arrow_cross",
        Some('o') => "arrow_circle",
        Some('>') => "arrow_point",
        _ => "",
    }
}

fn edge_str_to_edge_start_data(type_str: &str) -> &'static str {
    match js::trim(type_str).chars().next() {
        Some('x') => "arrow_cross",
        Some('o') => "arrow_circle",
        Some('<') => "arrow_point",
        _ => "arrow_open",
    }
}

fn edge_str_to_thickness(type_str: &str) -> &'static str {
    if type_str.contains("==") {
        "thick"
    } else {
        "normal"
    }
}

fn edge_str_to_pattern(type_str: &str) -> &'static str {
    if type_str.contains(".-") {
        "dotted"
    } else {
        "solid"
    }
}

struct NodeInfo<'a> {
    id: &'a str,
    label: Option<&'a str>,
    type_str: Option<String>,
    directions: Option<Vec<String>>,
}

/// A statement: one block, or a list of blocks and edges from a link statement.
enum Stmt {
    One(usize),
    Chain(Vec<usize>),
}

struct OpenFrame {
    list: Vec<usize>,
    header: Option<Stmt>,
}

struct Db {
    blocks: Vec<Block>,
    edges: Vec<usize>,
    classes: IndexMap<String, BlockClassDef>,
    class_lists: Vec<Vec<String>>,
    // Mermaid keeps the root in here too, which makes a block called `root` vanish. Pele keeps
    // it out.
    by_id: HashMap<String, usize>,
    // Ids that a class or style statement named before any block had them.
    pending: HashSet<String>,
    edge_count: HashMap<String, usize>,
    ids: usize,
    colors: usize,
    spaces: f64,
}

impl Db {
    fn new() -> Self {
        let root = Block { id: Some("root".into()), kind: "composite".into(), columns: Some(-1.0), ..Default::default() };
        Db {
            blocks: vec![root],
            edges: Vec::new(),
            classes: IndexMap::new(),
            class_lists: Vec::new(),
            by_id: HashMap::new(),
            pending: HashSet::new(),
            edge_count: HashMap::new(),
            ids: 0,
            colors: 0,
            spaces: 0.0,
        }
    }

    fn add(&mut self, block: Block) -> usize {
        self.blocks.push(block);
        self.blocks.len() - 1
    }

    /// Mermaid's ids are random. Pele counts them, so the same text always gives the same
    /// output.
    fn generate_id(&mut self) -> String {
        self.ids += 1;
        format!("id-{}", self.ids)
    }

    fn add_style_class(&mut self, id: &str, style_attributes: &str) {
        let found = self.classes.entry(id.to_string()).or_insert_with(|| BlockClassDef { id: id.to_string(), ..Default::default() });
        for attribute in style_attributes.split(',') {
            let fixed = match attribute.find(';') {
                None => js::trim(attribute).to_string(),
                Some(semi) => js::trim(&format!("{}{}", &attribute[..semi], &attribute[semi + 1..])).to_string(),
            };
            if attribute.contains("color") {
                found.text_styles.push(fixed.replacen("fill", "bgFill", 1).replacen("color", "fill", 1));
            }
            found.styles.push(fixed);
        }
    }

    fn add_style_to_node(&mut self, id: &str, styles: &str) {
        let k = self.named(id);
        self.blocks[k].styles = Some(styles.split(',').map(str::to_string).collect());
    }

    // Mermaid looks each id up before trimming it, so `class a, b hot` misses b. Pele trims it
    // first.
    fn set_css_class(&mut self, item_ids: &str, css_class_name: &str) {
        for id in item_ids.split(',') {
            let k = self.named(js::trim(id));
            let list = match self.blocks[k].classes {
                Some(list) => list,
                None => {
                    self.class_lists.push(Vec::new());
                    let list = self.class_lists.len() - 1;
                    self.blocks[k].classes = Some(list);
                    list
                }
            };
            self.class_lists[list].push(css_class_name.to_string());
        }
    }

    // Mermaid throws on a style for an unknown id, and a class for one makes the block of that
    // id disappear when it is declared later. Pele makes both wait for the block.
    fn named(&mut self, id: &str) -> usize {
        if let Some(&k) = self.by_id.get(id) {
            return k;
        }
        let k = self.add(Block { id: Some(id.to_string()), kind: "na".into(), ..Default::default() });
        self.by_id.insert(id.to_string(), k);
        self.pending.insert(id.to_string());
        k
    }

    fn set_hierarchy(&mut self, statements: Vec<usize>) -> Result<(), Error> {
        struct Frame {
            list: Vec<usize>,
            at: usize,
            parent: usize,
            children: Vec<usize>,
        }
        // Pele warns here when a block is wider than the columns of its container; it has no
        // warning callback when it renders, so nothing is reported.
        let mut stack = vec![Frame { list: statements, at: 0, parent: BlockModel::ROOT, children: Vec::new() }];
        while let Some(f) = stack.last_mut() {
            if f.at == f.list.len() {
                let Some(f) = stack.pop() else { break };
                self.blocks[f.parent].children = f.children;
                continue;
            }
            let bi = f.list[f.at];
            f.at += 1;
            let parent = f.parent;
            match self.blocks[bi].kind.as_str() {
                "classDef" => {
                    let id = self.blocks[bi].id.clone().unwrap_or_default();
                    let css = self.blocks[bi].css.clone().unwrap_or_default();
                    self.add_style_class(&id, &css);
                    continue;
                }
                "applyClass" => {
                    let id = self.blocks[bi].id.clone().unwrap_or_default();
                    let class = self.blocks[bi].style_class.clone().unwrap_or_default();
                    self.set_css_class(&id, &class);
                    continue;
                }
                "applyStyles" => {
                    if let Some(styles) = self.blocks[bi].styles_str.clone().filter(|s| !s.is_empty()) {
                        let id = self.blocks[bi].id.clone().unwrap_or_default();
                        self.add_style_to_node(&id, &styles);
                    }
                    continue;
                }
                "column-setting" => {
                    self.blocks[parent].columns = Some(self.blocks[bi].columns.unwrap_or(-1.0));
                    continue;
                }
                "edge" => {
                    let id = self.blocks[bi].id.clone().unwrap_or_default();
                    let count = self.edge_count.get(&id).copied().unwrap_or(0) + 1;
                    self.edge_count.insert(id.clone(), count);
                    self.blocks[bi].id = Some(format!("{}-{}", count, id));
                    self.edges.push(bi);
                    continue;
                }
                _ => {}
            }

            // A composite whose header was a link statement has no id of its own.
            if self.blocks[bi].id.is_none() {
                let id = self.generate_id();
                self.blocks[bi].id = Some(id);
            }
            if self.blocks[bi].label.as_deref().is_none_or(str::is_empty) {
                let label =
                    if self.blocks[bi].kind == "composite" { String::new() } else { self.blocks[bi].id.clone().unwrap_or_default() };
                self.blocks[bi].label = Some(label);
            }
            let known = self.by_id.get(self.blocks[bi].id.as_deref().unwrap_or("")).copied();
            let fresh = known.is_none() || self.pending.remove(self.blocks[bi].id.as_deref().unwrap_or(""));
            if !fresh {
                if let Some(k) = known {
                    if self.blocks[bi].kind != "na" {
                        self.blocks[k].kind = self.blocks[bi].kind.clone();
                    }
                    if self.blocks[bi].label.as_deref() != self.blocks[bi].id.as_deref() {
                        self.blocks[k].label = self.blocks[bi].label.clone();
                    }
                }
            } else {
                if let Some(k) = known {
                    self.blocks[bi].classes = self.blocks[k].classes;
                    self.blocks[bi].styles = self.blocks[k].styles.clone();
                }
                if self.blocks[bi].kind == "composite" {
                    self.blocks[bi].color_index = Some(self.colors);
                    self.colors += 1;
                }
                self.by_id.insert(self.blocks[bi].id.clone().unwrap_or_default(), bi);
            }

            let inner = std::mem::take(&mut self.blocks[bi].statements);
            self.blocks[bi].children = Vec::new();
            if self.blocks[bi].kind == "space" {
                let id = self.blocks[bi].id.clone().unwrap_or_default();
                let width = self.blocks[bi].width.unwrap_or(1.0);
                if !(width <= MAX_SPACES - self.spaces) {
                    return Err(Error::limit(format!("A block diagram can hold at most {} space blocks.", js::to_string(MAX_SPACES)))
                        .with_type(DiagramType::Block));
                }
                self.spaces += js::max(width, 0.0);
                let mut j = 0.0;
                while j < width {
                    let copy_id = format!("{}-{}", id, js::to_string(j));
                    let copy = Block { id: Some(copy_id.clone()), children: Vec::new(), ..self.blocks[bi].clone() };
                    let k = self.add(copy);
                    self.by_id.insert(copy_id, k);
                    if let Some(f) = stack.last_mut() {
                        f.children.push(k);
                    }
                    j += 1.0;
                }
            } else if fresh {
                if let Some(f) = stack.last_mut() {
                    f.children.push(bi);
                }
            }
            if !inner.is_empty() {
                stack.push(Frame { list: inner, at: 0, parent: bi, children: Vec::new() });
            }
        }
        Ok(())
    }
}

struct Parser<'a> {
    src: &'a str,
    types: Vec<T>,
    texts: Vec<&'a str>,
    starts: Vec<usize>,
    i: usize,
}

impl<'a> Parser<'a> {
    fn kind(&self) -> T {
        self.types.get(self.i).copied().unwrap_or(T::End)
    }

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.starts.get(self.i).copied().unwrap_or(self.src.len());
        if t == T::Error {
            return syntax_error(DiagramType::Block, self.src, at, "", true);
        }
        syntax_error(DiagramType::Block, self.src, at, &format!("Expecting {}, got '{}'", expected, token_name(t)), false)
    }

    fn expect(&mut self, ty: T) -> Result<&'a str, Error> {
        if self.kind() != ty {
            return Err(self.fail(&format!("'{}'", token_name(ty))));
        }
        let text = self.texts[self.i];
        self.i += 1;
        Ok(text)
    }

    fn node(&mut self) -> Result<NodeInfo<'a>, Error> {
        let id = self.expect(T::NodeId)?;
        if self.kind() == T::NodeDStart {
            let open = self.texts[self.i];
            self.i += 1;
            let label = self.expect(T::Str)?;
            let close = self.expect(T::NodeDEnd)?;
            return Ok(NodeInfo { id, label: Some(label), type_str: Some(format!("{}{}", open, close)), directions: None });
        }
        if self.kind() == T::BlockArrowStart {
            let open = self.texts[self.i];
            self.i += 1;
            let label = self.expect(T::Str)?;
            let mut directions = vec![self.expect(T::Dir)?.to_string()];
            while self.kind() == T::Dir {
                directions.push(self.texts[self.i].to_string());
                self.i += 1;
            }
            let close = self.expect(T::BlockArrowEnd)?;
            return Ok(NodeInfo { id, label: Some(label), type_str: Some(format!("{}{}", open, close)), directions: Some(directions) });
        }
        Ok(NodeInfo { id, label: None, type_str: None, directions: None })
    }

    fn node_statement(&mut self, db: &mut Db) -> Result<Stmt, Error> {
        let first = self.node()?;
        let size = if self.kind() == T::Size {
            let s = js::parse_int(self.texts[self.i], 10);
            self.i += 1;
            s
        } else {
            1.0
        };
        let mut from = Block {
            id: Some(first.id.to_string()),
            label: first.label.map(str::to_string),
            kind: type_str_to_type(first.type_str.as_deref()).to_string(),
            directions: first.directions,
            width_in_columns: Some(size),
            ..Default::default()
        };
        let mut chain: Option<Vec<usize>> = None;
        while matches!(self.kind(), T::Link | T::StartLink) {
            let mut label = "";
            let started = self.kind() == T::StartLink;
            self.i += 1;
            if started {
                self.expect(T::LinkLabel)?;
                label = self.expect(T::Str)?;
                self.expect(T::Link)?;
            }
            let link = self.texts[self.i - 1];
            let to = self.node()?;
            let arrow_type_end = edge_str_to_edge_data(link);
            let arrow_type_start = edge_str_to_edge_start_data(link);
            let thickness = edge_str_to_thickness(link);
            let pattern = edge_str_to_pattern(link);
            // Mermaid keeps only the last link of `A --> B --> C` and loses its start. Pele
            // keeps every link.
            let list = chain.get_or_insert_with(|| {
                vec![db.add(Block {
                    id: from.id.clone(),
                    label: from.label.clone(),
                    kind: from.kind.clone(),
                    directions: from.directions.clone(),
                    ..Default::default()
                })]
            });
            from = Block {
                id: Some(to.id.to_string()),
                label: to.label.map(str::to_string),
                kind: type_str_to_type(to.type_str.as_deref()).to_string(),
                directions: to.directions.clone(),
                ..Default::default()
            };
            let last = list.last().copied().unwrap_or(0);
            let last_id = db.blocks[last].id.clone().unwrap_or_default();
            let edge = db.add(Block {
                id: Some(format!("{}-{}", last_id, to.id)),
                start: Some(last_id),
                end: Some(to.id.to_string()),
                label: Some(label.to_string()),
                kind: "edge".into(),
                thickness: Some(thickness.into()),
                pattern: Some(pattern.into()),
                directions: to.directions,
                arrow_type_end: Some(arrow_type_end.into()),
                arrow_type_start: Some(arrow_type_start.into()),
                ..Default::default()
            });
            let next = db.add(from.clone());
            list.push(edge);
            list.push(next);
        }
        Ok(match chain {
            Some(list) => Stmt::Chain(list),
            None => Stmt::One(db.add(from)),
        })
    }
}

fn push_statement(list: &mut Vec<usize>, statement: Stmt) {
    match statement {
        Stmt::One(k) => list.push(k),
        Stmt::Chain(items) => list.extend(items),
    }
}

/// Parses a block diagram and builds its tree. `title` is the front matter title.
pub(super) fn parse_block(src: &str, title: Option<&str>) -> Result<BlockModel, Error> {
    let Tokens { types, texts, starts } = tokenize(src);
    let mut p = Parser { src, types, texts, starts, i: 0 };
    let mut db = Db::new();

    p.expect(T::Key)?;
    // Composites nest without recursion: each open one keeps the statement list of its parent.
    // Pele's lists hold statements, a link statement being a list itself; they are flattened
    // here as Pele flattens them when it builds the tree.
    let mut open: Vec<OpenFrame> = Vec::new();
    let mut list: Vec<usize> = Vec::new();
    loop {
        match p.kind() {
            T::NodeId => {
                let statement = p.node_statement(&mut db)?;
                push_statement(&mut list, statement);
            }
            T::Columns => {
                let columns = js::parse_int(p.texts[p.i], 10);
                p.i += 1;
                list.push(db.add(Block { kind: "column-setting".into(), columns: Some(columns), ..Default::default() }));
            }
            T::SpaceBlock => {
                let width = js::parse_int(p.texts[p.i], 10);
                p.i += 1;
                let id = db.generate_id();
                list.push(db.add(Block {
                    id: Some(id),
                    kind: "space".into(),
                    label: Some(String::new()),
                    width: Some(width),
                    ..Default::default()
                }));
            }
            T::Key => {
                p.i += 1;
                open.push(OpenFrame { list: std::mem::take(&mut list), header: None });
            }
            T::IdBlock => {
                p.i += 1;
                if p.kind() != T::NodeId {
                    return Err(p.fail("'NODE_ID'"));
                }
                let header = p.node_statement(&mut db)?;
                open.push(OpenFrame { list: std::mem::take(&mut list), header: Some(header) });
            }
            T::EndKeyword => {
                let frame = open.pop();
                let Some(frame) = frame.filter(|_| !list.is_empty()) else {
                    return Err(p.fail(STATEMENT_START));
                };
                p.i += 1;
                let id = db.generate_id();
                let children = std::mem::replace(&mut list, frame.list);
                let composite = match frame.header {
                    None => Block {
                        id: Some(id),
                        kind: "composite".into(),
                        label: Some(String::new()),
                        statements: children,
                        ..Default::default()
                    },
                    Some(Stmt::One(h)) => {
                        Block { kind: "composite".into(), children: Vec::new(), statements: children, ..db.blocks[h].clone() }
                    }
                    // `{ ...list, type, children }` keeps none of the header's blocks.
                    Some(Stmt::Chain(_)) => Block { kind: "composite".into(), statements: children, ..Default::default() },
                };
                list.push(db.add(composite));
            }
            T::ClassDef => {
                p.i += 1;
                let id = js::trim(p.expect(T::ClassDefId)?).to_string();
                let css = js::trim(p.expect(T::ClassDefStyleOpts)?).to_string();
                list.push(db.add(Block { kind: "classDef".into(), id: Some(id), css: Some(css), ..Default::default() }));
            }
            T::Class => {
                p.i += 1;
                let id = js::trim(p.expect(T::ClassEntityIds)?).to_string();
                let class = js::trim(p.expect(T::StyleClass)?).to_string();
                list.push(db.add(Block { kind: "applyClass".into(), id: Some(id), style_class: Some(class), ..Default::default() }));
            }
            T::Style => {
                p.i += 1;
                let id = js::trim(p.expect(T::StyleEntityIds)?).to_string();
                let styles = js::trim(p.expect(T::StyleDefinitionData)?).to_string();
                list.push(db.add(Block { kind: "applyStyles".into(), id: Some(id), styles_str: Some(styles), ..Default::default() }));
            }
            T::Eof => {
                if !open.is_empty() {
                    return Err(p.fail(&format!("{}, 'end'", STATEMENT_START)));
                }
                if list.is_empty() {
                    return Err(p.fail(STATEMENT_START));
                }
                db.set_hierarchy(list)?;
                return Ok(BlockModel {
                    title: title.map(str::to_string),
                    blocks: db.blocks,
                    edges: db.edges,
                    classes: db.classes,
                    class_lists: db.class_lists,
                });
            }
            _ => {
                let expected = if list.is_empty() {
                    STATEMENT_START.to_string()
                } else {
                    format!("{}{}", STATEMENT_START, if open.is_empty() { ", 'EOF'" } else { ", 'end'" })
                };
                return Err(p.fail(&expected));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_tree() {
        let m = parse_block("block\n columns 2\n a b:2\n block:g\n  c --> d\n end\n space:2\n", None).unwrap();
        let root = &m.blocks[BlockModel::ROOT];
        assert_eq!(root.columns, Some(2.0));
        let ids: Vec<&str> = root.children.iter().map(|&k| m.blocks[k].id.as_deref().unwrap()).collect();
        assert_eq!(ids, ["a", "b", "g", "id-2-0", "id-2-1"]);
        assert_eq!(m.blocks[root.children[1]].width_in_columns, Some(2.0));
        let g = &m.blocks[root.children[2]];
        assert_eq!(g.kind, "composite");
        let inner: Vec<&str> = g.children.iter().map(|&k| m.blocks[k].id.as_deref().unwrap()).collect();
        assert_eq!(inner, ["c", "d"]);
        assert_eq!(m.edges.len(), 1);
        assert_eq!(m.blocks[m.edges[0]].id.as_deref(), Some("1-c-d"));
    }

    #[test]
    fn messages() {
        let e = parse_block("block\n end\n", None).unwrap_err();
        assert!(e.message.ends_with(&format!("Expecting {}, got 'end'", STATEMENT_START)), "{}", e.message);
        let e = parse_block("block\n a[b]\n", None).unwrap_err();
        assert!(e.message.starts_with("Lexical error on line 2."), "{}", e.message);
    }
}
