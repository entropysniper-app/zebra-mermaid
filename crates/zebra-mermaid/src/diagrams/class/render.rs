//! UML compartments, namespaces and cardinalities using Pele's layout policy.
use super::{
    graph::{self, Shape},
    markers, ClassModel, ClassNode, Member,
};
use crate::diagrams::common::{
    fit_width::{runs_across, tighten, turn_to_fit},
    generics::parse_generic_types,
};
use crate::diagrams::{RenderCtx, Rendered};
use crate::layout::{
    compound::{cnode, compound_layout, direction, CEdge, Dir},
    layered::LayeredOptions,
};
use crate::svg::cluster::render_struck_title as struck_title;
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg},
    cluster::*,
    edges::{default_reach, render_edge_label_svg as edge_label_svg, route_path, EdgePath},
    root::{svg_document, Accessible},
    shapes::end_side,
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
    title::with_title,
};
use crate::text::{entities::decode_entities, layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{
    js,
    url::{linked, LinkInfo},
};
use crate::{Config, ConfigMap, ConfigValue, Error};
use indexmap::IndexMap;
use std::collections::HashMap;
struct Line {
    text: String,
    width: f64,
    classifier: String,
}
struct View {
    style: ResolvedStyle,
    label: Label,
    w: f64,
    h: f64,
    annotations: Vec<String>,
    attributes: Vec<Line>,
    methods: Vec<Line>,
    title_h: f64,
    attributes_h: f64,
    font_size: f64,
}
impl View {
    fn base(label: Label, w: f64, h: f64, font_size: f64) -> Self {
        Self {
            style: ResolvedStyle::default(),
            label,
            w,
            h,
            font_size,
            annotations: vec![],
            attributes: vec![],
            methods: vec![],
            title_h: 0.0,
            attributes_h: 0.0,
        }
    }
}
#[derive(Clone, Copy)]
struct End {
    edge: usize,
    at: usize,
    toward: f64,
    order: usize,
    room: f64,
    side: i32,
    after: bool,
}
fn number(c: &ConfigMap, key: &str, fallback: f64) -> f64 {
    c.get(key).and_then(ConfigValue::as_f64).filter(|n| *n > 0.0).map_or(fallback, |n| js::min(n, 2000.0))
}
// Match Pele's per-draw declaration and CSS-class caches. Borrow declarations from the
// immutable model; property slots preserve first-insertion order while replacing the last
// effective value without allocating or hashing a property name on every repeated style.
#[derive(Default)]
struct ClassStyles<'a> {
    effects: HashMap<&'a str, Option<usize>>,
    properties: HashMap<String, usize>,
    from_defs: HashMap<&'a str, Vec<(usize, &'a str)>>,
    kept: Vec<(usize, &'a str)>,
    slots: Vec<Option<usize>>,
}
impl<'a> ClassStyles<'a> {
    fn keep(&mut self, declaration: &'a str) {
        let property = if let Some(property) = self.effects.get(declaration) {
            *property
        } else {
            let style = resolve_style(&[declaration]);
            let property = if style.shape.is_empty() && style.text.is_empty() {
                None
            } else {
                declaration.split_once(':').map(|(name, _)| {
                    let name = js::trim(name).to_lowercase();
                    let next = self.properties.len();
                    let id = *self.properties.entry(name).or_insert(next);
                    if id == self.slots.len() {
                        self.slots.push(None);
                    }
                    id
                })
            };
            self.effects.insert(declaration, property);
            property
        };
        if let Some(property) = property {
            if let Some(index) = self.slots[property] {
                self.kept[index].1 = declaration;
            } else {
                self.slots[property] = Some(self.kept.len());
                self.kept.push((property, declaration));
            }
        }
    }
    fn resolve(&mut self, model: &'a ClassModel, class: &'a ClassNode) -> ResolvedStyle {
        self.kept.clear();
        self.slots.fill(None);
        if let Some(base) = self.from_defs.get(class.css_classes.as_str()) {
            self.kept.extend_from_slice(base);
            for (index, (property, _)) in self.kept.iter().enumerate() {
                self.slots[*property] = Some(index);
            }
        } else {
            for name in class.css_classes.split(' ') {
                if let Some(styles) = model.style_classes.get(name) {
                    for declaration in styles {
                        for part in declaration.split(',') {
                            self.keep(part);
                        }
                    }
                }
            }
            self.from_defs.insert(&class.css_classes, self.kept.clone());
        }
        for declaration in &class.styles {
            self.keep(declaration);
        }
        resolve_style(&self.kept.iter().map(|(_, declaration)| *declaration).collect::<Vec<_>>())
    }
}
pub(crate) fn render(db: &ClassModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let fitted = |down| tighten(ctx.options, |tight| draw(db, config, ctx, down, tight));
    if runs_across(Some(&db.direction)) {
        turn_to_fit(ctx.options, fitted)
    } else {
        fitted(false)
    }
}
fn draw(db: &ClassModel, config: &Config, ctx: &RenderCtx, turned: bool, tight: bool) -> Result<Rendered, Error> {
    let empty = ConfigMap::new();
    let conf = config.section("class").unwrap_or(&empty);
    let (graph, graph_edges) = graph::build(db, conf.get("hierarchicalNamespaces").and_then(ConfigValue::as_bool) != Some(false));
    let hide_empty = conf.get("hideEmptyMembersBox").and_then(ConfigValue::as_bool) == Some(true);
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.8);
    let tiny_h = js::round(tiny * 1.4);
    let m = ctx.measure();
    let pad = ctx.padding();
    let dir = if turned { Dir::TB } else { direction(Some(&db.direction)) };
    let mut children = vec![0; graph.len()];
    for n in &graph {
        if n.parent >= 0 {
            children[n.parent as usize] += 1;
        }
    }
    let mut crossed = vec![false; graph.len()];
    for e in &graph_edges {
        if e.source < 0 || e.target < 0 {
            continue;
        }
        let a = graph[e.source as usize].parent;
        let b = graph[e.target as usize].parent;
        if a == b {
            continue;
        }
        for start in [a, b] {
            let mut g = start;
            while g >= 0 && !crossed[g as usize] {
                crossed[g as usize] = true;
                g = graph[g as usize].parent;
            }
        }
    }
    let mut views = vec![];
    let mut styles = ClassStyles::default();
    let mut nodes = vec![];
    for (i, node) in graph.iter().enumerate() {
        let (v, mut c) = if node.group {
            let label = layout_label(Some(&node.label), false, &m, size, 4000.0, 0);
            let mut v = View::base(label, 0.0, 0.0, size);
            let c = if children[i] > 0 {
                let mut c = cnode(0.0, 0.0, -1);
                c.is_group = true;
                c.pad_x = 20.0;
                c.pad_top = if v.label.height > 0.0 { v.label.height + 16.0 } else { 20.0 };
                c.pad_bottom = 20.0;
                c.min_w = v.label.width + 40.0 - 12.0 + if crossed[i] && !tight { js::min(v.label.width + 24.0, 240.0) } else { 0.0 };
                c
            } else {
                v.w = js::max(v.label.width + 40.0, 80.0);
                v.h = v.label.height + 40.0;
                cnode(v.w, v.h, -1)
            };
            (v, c)
        } else if let Some(cls) = node.class {
            let st = styles.resolve(db, cls);
            let font_size = st.font_size.unwrap_or(size);
            let member_size = js::round(font_size * 0.875);
            let line_h = js::round(member_size * 1.5);
            let generic = if cls.kind.is_empty() {
                String::new()
            } else {
                format!("&lt;{}&gt;", parse_generic_types(&cls.kind).replace('<', "&lt;").replace('>', "&gt;"))
            };
            let label = layout_label(
                Some(&(cls.label.clone() + &generic)),
                false,
                &m,
                font_size,
                4000.0,
                style::BOLD | if st.italic { style::ITALIC } else { 0 },
            );
            let annotations: Vec<_> = cls.annotations.iter().map(|a| format!("«{}»", js::trim(&decode_entities(a)))).collect();
            let line = |member: &Member| {
                let text = js::trim(&decode_entities(&member.display())).to_owned();
                Line {
                    width: m.width(&text, member_size, if member.classifier == "*" { style::ITALIC } else { 0 }),
                    text,
                    classifier: member.classifier.clone(),
                }
            };
            let attributes: Vec<_> = cls.members.iter().map(line).collect();
            let methods: Vec<_> = cls.methods.iter().map(line).collect();
            let mut inner = label.width;
            for a in &annotations {
                inner = js::max(inner, m.width(a, tiny, 0));
            }
            for l in attributes.iter().chain(&methods) {
                inner = js::max(inner, l.width);
            }
            let sections = !(hide_empty && attributes.is_empty() && methods.is_empty());
            let section = |count: usize| {
                if !sections {
                    0.0
                } else if count > 0 {
                    12.0 + count as f64 * line_h
                } else {
                    10.0
                }
            };
            let title_h = 16.0 + annotations.len() as f64 * tiny_h + js::max(label.height, js::round(font_size * 1.5));
            let attributes_h = section(attributes.len());
            let w = js::max(inner + 24.0, 64.0);
            let h = title_h + attributes_h + section(methods.len());
            (View { style: st, label, w, h, annotations, attributes, methods, title_h, attributes_h, font_size }, cnode(w, h, -1))
        } else if node.shape == Shape::Note {
            let label = layout_label(Some(&node.label), true, &m, small, 260.0, 0);
            let w = js::max(label.width + 24.0, 40.0);
            let h = label.height + 16.0;
            (View::base(label, w, h, small), cnode(w, h, -1))
        } else {
            let label = layout_label(Some(&node.label), false, &m, small, 200.0, 0);
            let w = js::max(label.width, 12.0) + 4.0;
            let h = js::max(label.height, 12.0);
            (View::base(label, w, h, small), cnode(w, h, -1))
        };
        c.parent = node.parent;
        c.seq = i as f64;
        views.push(v);
        nodes.push(c);
    }
    let mut edges = vec![];
    let mut drawn = vec![];
    let mut labels = vec![];
    let mut start_room = vec![];
    let mut end_room = vec![];
    let mut crowd = vec![0.0; views.len() * 4];
    for e in &graph_edges {
        if e.source < 0 || e.target < 0 {
            continue;
        }
        let src = e.source as usize;
        let dst = e.target as usize;
        let label = layout_label(e.label.as_deref(), true, &m, small, 200.0, 0);
        let sw = if e.tail_label.is_empty() { 0.0 } else { m.width(&decode_entities(&e.tail_label), tiny, 0) };
        let ew = if e.head_label.is_empty() { 0.0 } else { m.width(&decode_entities(&e.head_label), tiny, 0) };
        if src == dst {
            nodes[src].w += 2.0 * (26.0 + if label.width > 0.0 { label.width + 12.0 } else { 0.0 });
        } else {
            for (item, slot, room) in [(src, 0, sw), (dst, 2, ew)] {
                crowd[item * 4 + slot] += 1.0;
                crowd[item * 4 + slot + 1] = js::max(crowd[item * 4 + slot + 1], room);
            }
        }
        start_room.push(sw);
        end_room.push(ew);
        drawn.push(e);
        edges.push(CEdge {
            src,
            dst,
            minlen: 1.0,
            label_w: if label.width > 0.0 { label.width + 12.0 } else { 0.0 },
            label_h: if label.height > 0.0 { label.height + 4.0 } else { 0.0 },
            route: vec![],
            label_x: 0.0,
            label_y: 0.0,
        });
        labels.push(label);
    }
    if matches!(dir, Dir::TB | Dir::BT) {
        for i in 0..views.len() {
            if graph[i].class.is_none() {
                continue;
            }
            let mut need = 0.0;
            for slot in [0, 2] {
                let count = crowd[i * 4 + slot];
                let room = crowd[i * 4 + slot + 1];
                if count > 1.0 {
                    need = js::max(need, (count - 1.0) * if room > 0.0 { room + 24.0 } else { 14.0 } + 24.0);
                }
            }
            need = js::min(need, 480.0);
            if need > views[i].w {
                nodes[i].w += need - views[i].w;
                views[i].w = need;
            }
        }
    }
    let lettered = start_room.iter().chain(&end_room).any(|&r| r > 0.0);
    let opt = LayeredOptions {
        node_sep: number(conf, "nodeSpacing", 40.0),
        edge_sep: 16.0,
        rank_sep: number(conf, "rankSpacing", if lettered { 72.0 } else { 56.0 }),
        port_sep: 20.0,
        tracks: false,
        straight: false,
        apart: true,
        head_room: None,
    };
    let layout = compound_layout(&mut nodes, &mut edges, dir, &opt);
    let mut sides = IndexMap::<usize, Vec<End>>::new();
    let mut starts = vec![None; edges.len()];
    let mut ends = vec![None; edges.len()];
    for (i, e) in edges.iter_mut().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let last = e.route.len() - 3;
        for (at, item, room, first) in [(0, e.src, start_room[i], true), (last, e.dst, end_room[i], false)] {
            let c = &nodes[item];
            let side = end_side(&e.route, at, c.x, c.y, c.w, c.h);
            if side < 0 {
                continue;
            }
            let slack = (c.w - views[item].w) / 2.0;
            if side == 1 {
                e.route[at] -= slack;
            } else if side == 3 {
                e.route[at] += slack;
            }
            let across = (side & 1) as usize;
            let toward = e.route[(if at == 0 { 3 } else { at - 3 }) + across];
            let bend = toward - e.route[at + across];
            let end = End { edge: i, at, toward, order: i, room, side, after: if across != 0 { bend > 0.0 } else { bend >= 0.0 } };
            sides.entry(item * 4 + side as usize).or_default().push(end);
            if first {
                starts[i] = Some(end);
            } else {
                ends[i] = Some(end);
            }
        }
    }
    let on_border = |item: usize, x: f64, y: f64| {
        let mut g = nodes[item].parent;
        for _ in 0..10000 {
            if g < 0 {
                break;
            }
            let c = &nodes[g as usize];
            let dx = (x - c.x).abs();
            let dy = (y - c.y).abs();
            if ((dy - c.h / 2.0).abs() < 0.5 && dx <= c.w / 2.0 + 0.5) || ((dx - c.w / 2.0).abs() < 0.5 && dy <= c.h / 2.0 + 0.5) {
                return true;
            }
            g = c.parent;
        }
        false
    };
    for (key, mut list) in sides {
        if list.len() < 2 {
            continue;
        }
        let item = key >> 2;
        let across = key & 1;
        let length = if across != 0 { views[item].h } else { views[item].w };
        list.sort_by(|a, b| a.toward.total_cmp(&b.toward).then(a.order.cmp(&b.order)));
        let mut gaps = vec![];
        let mut total = 0.0;
        for pair in list.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let mut gap = if across != 0 { 20.0 } else { 24.0 };
            if a.room > 0.0 && a.after {
                gap += if across != 0 { tiny_h + 6.0 } else { a.room };
            }
            if b.room > 0.0 && !b.after {
                gap += if across != 0 { tiny_h + 6.0 } else { b.room };
            }
            gaps.push(gap);
            total += gap;
        }
        let scale = js::min(1.0, (length - 24.0) / total);
        if scale <= 0.0 {
            continue;
        }
        let mut position = if across != 0 { nodes[item].y } else { nodes[item].x } - total * scale / 2.0;
        for (j, end) in list.iter().enumerate() {
            let route = &mut edges[end.edge].route;
            let step = if end.at == 0 { 3isize } else { -3 };
            let first = (end.at as isize + step) as usize;
            let mut border = None;
            let mut k = first as isize;
            while k >= 0 && (k as usize) < route.len() && (route[k as usize + across] - route[first + across]).abs() < 0.5 {
                if on_border(item, route[k as usize], route[k as usize + 1]) {
                    border = Some(k);
                    break;
                }
                k += step;
            }
            if let Some(border) = border {
                if (route[first + across] - position).abs() <= 24.0 {
                    let mut k = first as isize;
                    while k != border + step {
                        route[k as usize + across] = position;
                        k += step;
                    }
                }
            }
            route[end.at + across] = position;
            if let Some(gap) = gaps.get(j) {
                position += gap * scale;
            }
        }
    }
    for i in 0..edges.len() {
        for (end, room, first) in [(starts[i], start_room[i], true), (ends[i], end_room[i], false)] {
            let Some(end) = end.filter(|_| room > 0.0) else {
                continue;
            };
            let route = &mut edges[i].route;
            let across = (end.side & 1) as usize;
            let at = if first { 0 } else { route.len() - 3 };
            let along = at + 1 - across;
            let reach = (route[(if first { 3 } else { at - 3 }) + 1 - across] - route[along]).abs();
            let length = js::min(if across != 0 { room + 10.0 } else { tiny_h + 6.0 }, reach * 0.35);
            if length < 4.0 {
                continue;
            }
            let mut point = [route[at], route[at + 1], route[at + 2]];
            point[1 - across] += if end.side == 0 || end.side == 3 { -length } else { length };
            route.splice(if first { 3..3 } else { at..at }, point);
        }
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0.0, 0.0, layout.width, layout.height);
    let mut cards = vec![];
    let mut add_card = |raw: &str, w: f64, x: f64, y: f64, side: i32, after: bool| {
        if raw.is_empty() {
            return;
        }
        let (mut cx, mut cy) = (x, y);
        if side & 1 != 0 {
            cx += if side == 3 { -1.0 } else { 1.0 } * (5.0 + w / 2.0);
            cy += if after { 1.0 } else { -1.0 } * (7.0 + tiny_h / 2.0);
        } else {
            cx += if after { 1.0 } else { -1.0 } * (9.0 + w / 2.0);
            cy += if side == 0 { -1.0 } else { 1.0 } * (3.0 + tiny_h / 2.0);
        }
        cards.push((decode_entities(raw), cx, cy));
        min_x = js::min(min_x, cx - w / 2.0);
        max_x = js::max(max_x, cx + w / 2.0);
        min_y = js::min(min_y, cy - tiny_h / 2.0);
        max_y = js::max(max_y, cy + tiny_h / 2.0);
    };
    let mut loop_seen = IndexMap::<usize, usize>::new();
    let mut loop_at = vec![0; edges.len()];
    for (i, e) in edges.iter().enumerate() {
        let edge = drawn[i];
        if e.src == e.dst {
            let k = loop_seen.entry(e.src).or_default();
            loop_at[i] = *k;
            let c = &nodes[e.src];
            let v = &views[e.src];
            let x = c.x + v.w / 2.0;
            let spread = js::min(v.h / 2.0 - 4.0, 8.0 + *k as f64 * 6.0);
            *k += 1;
            add_card(&edge.tail_label, start_room[i], x, c.y - spread, 1, false);
            add_card(&edge.head_label, end_room[i], x, c.y + spread, 1, true);
        } else if e.route.len() >= 6 {
            let last = e.route.len() - 3;
            add_card(
                &edge.tail_label,
                start_room[i],
                e.route[0],
                e.route[1],
                starts[i].map_or(2, |e| e.side),
                starts[i].is_none_or(|e| e.after),
            );
            add_card(
                &edge.head_label,
                end_room[i],
                e.route[last],
                e.route[last + 1],
                ends[i].map_or(0, |e| e.side),
                ends[i].is_none_or(|e| e.after),
            );
        }
    }
    let ox = pad - min_x;
    let oy = pad - min_y;
    let width = (max_x - min_x + 2.0 * pad).ceil();
    let height = (max_y - min_y + 2.0 * pad).ceil();
    let mut links = vec![];
    let mut crossings = Crossings::new();
    if graph.first().is_some_and(|n| n.group) {
        for e in &edges {
            mark_crossings(&mut crossings, &e.route);
        }
    }
    let (mut clusters, mut struck_titles, mut nodes_out) = (String::new(), String::new(), String::new());
    for (i, v) in views.iter().enumerate() {
        let c = &nodes[i];
        let node = &graph[i];
        let x = c.x + ox;
        let y = c.y + oy;
        let id = esc_text(&node.id);
        let w = if c.is_group { c.w } else { v.w };
        let h = if c.is_group { c.h } else { v.h };
        if node.group {
            let tx = if c.is_group { cluster_title_x(c.x, c.y, c.w, c.h, v.label.width, &crossings, 20.0) } else { c.x };
            let ty = y - h / 2.0 + 8.0 + v.label.height / 2.0;
            let attrs = svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"");
            let struck = c.is_group && title_crossed(c.x, c.y, c.w, c.h, v.label.width, tx, &crossings);
            if struck {
                struck_titles.push_str(&struck_title(&v.label, tx + ox, ty, &attrs, &id, |j| graph[j].parent, i, None, ctx.theming));
            }
            svg_write!(
                clusters,
                "<g class=\"pele-cluster pele-namespace\" data-id=\"",
                id,
                "\"><rect x=\"",
                num(x - w / 2.0),
                "\" y=\"",
                num(y - h / 2.0),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                ctx.paint(Role::GroupFill, "var(--_a)"),
                "\"",
                if ctx.resolved() { "" } else { " fill-opacity=\"0.5\"" },
                " stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"/>",
                if struck { String::new() } else { svg_format!(label_svg(&v.label, tx + ox, ty, &attrs, None)) },
                "</g>"
            )
            .unwrap();
            continue;
        }
        let at = svg_format!(" transform=\"translate(", num(x), ",", num(y), ")\"");
        if let Some(cls) = node.class {
            nodes_out.push_str(&class_svg(v, cls, &id, &at, size, tiny, tiny_h, ctx, &mut links));
        } else if node.shape == Shape::Note {
            let r = w / 2.0;
            let t = -h / 2.0;
            let stroke = ctx.paint(Role::WarningText, "var(--_b)");
            svg_write!(
                nodes_out,
                "<g class=\"pele-node pele-note\" data-id=\"",
                id,
                "\"",
                at,
                "><path d=\"M",
                num(-r),
                ",",
                num(t),
                "H",
                num(r - 8.0),
                "L",
                num(r),
                ",",
                num(t + 8.0),
                "V",
                num(-t),
                "H",
                num(-r),
                "Z\" fill=\"",
                ctx.paint(Role::WarningBackground, "var(--_a)"),
                "\" stroke=\"",
                stroke,
                "\" stroke-linejoin=\"round\"/><path d=\"M",
                num(r - 8.0),
                ",",
                num(t),
                "V",
                num(t + 8.0),
                "H",
                num(r),
                "\" fill=\"none\" stroke=\"",
                stroke,
                "\" stroke-linejoin=\"round\"/>",
                label_svg(
                    &v.label,
                    0.0,
                    0.0,
                    &svg_format!(
                        " class=\"pele-label\" font-size=\"",
                        js::to_string(small),
                        "\"",
                        if ctx.resolved() {
                            svg_format!(" fill=\"", ctx.paint(Role::WarningText, "var(--_fg)"), "\"")
                        } else {
                            String::new()
                        }
                    ),
                    ctx.icons()
                ),
                "</g>"
            )
            .unwrap();
        } else {
            svg_write!(
                nodes_out,
                "<g class=\"pele-node pele-interface\" data-id=\"",
                id,
                "\"",
                at,
                ">",
                label_svg(&v.label, 0.0, 0.0, &svg_format!(" class=\"pele-label\" font-size=\"", js::to_string(small), "\""), ctx.icons()),
                "</g>"
            )
            .unwrap();
        }
    }
    let (mut edges_out, mut labels_out) = (String::new(), String::new());
    for (i, e) in edges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &labels[i];
        let st = markers::trim(edge.tail);
        let et = markers::trim(edge.head);
        let path = if e.src == e.dst {
            let c = &nodes[e.src];
            let v = &views[e.src];
            let k = loop_at[i] as f64;
            let x = c.x + ox + v.w / 2.0;
            let y = c.y + oy;
            let spread = js::min(v.h / 2.0 - 4.0, 8.0 + k * 6.0);
            let reach = 26.0 + k * 8.0;
            e.label_x = c.x + v.w / 2.0 + reach + 4.0 + e.label_w / 2.0;
            e.label_y = c.y;
            EdgePath {
                d: svg_format!(
                    "M",
                    num(x + st),
                    ",",
                    num(y - spread),
                    "C",
                    num(x + reach + st),
                    ",",
                    num(y - spread),
                    " ",
                    num(x + reach + et),
                    ",",
                    num(y + spread),
                    " ",
                    num(x + et),
                    ",",
                    num(y + spread)
                ),
                sx: x,
                sy: y - spread,
                sdx: -1.0,
                sdy: 0.0,
                ex: x,
                ey: y + spread,
                edx: -1.0,
                edy: 0.0,
            }
        } else {
            for point in e.route.chunks_exact_mut(3) {
                point[0] += ox;
                point[1] += oy;
            }
            route_path(&e.route, None, st, et, default_reach(st), default_reach(et))
        };
        let attrs = match edge.pattern {
            "dashed" => " stroke-dasharray=\"6 4\"",
            "dotted" => " stroke-dasharray=\"2 4\"",
            _ => "",
        };
        let color = ctx.paint(Role::Arrow, "var(--_l)");
        svg_write!(
            edges_out,
            "<g class=\"pele-edge ",
            if edge.pattern == "dotted" { "pele-note-edge" } else { "pele-relation" },
            "\" data-id=\"",
            esc_text(&edge.id),
            "\"><path d=\"",
            path.d,
            "\"",
            attrs,
            "/>",
            markers::draw(edge.tail, path.sx, path.sy, path.sdx, path.sdy, color),
            markers::draw(edge.head, path.ex, path.ey, path.edx, path.edy, color),
            "</g>"
        )
        .unwrap();
        if label.width > 0.0 {
            svg_write!(
                labels_out,
                edge_label_svg(&edge.id, label, e.label_x + ox, e.label_y + oy, "", ctx.icons(), ctx.paint(Role::Background, "var(--_bg)"),)
            )
            .unwrap();
        }
    }
    let mut cards_out = String::new();
    for (text, x, y) in cards {
        svg_write!(
            cards_out,
            "<text class=\"pele-cardinality\" x=\"",
            num(x + ox),
            "\" y=\"",
            num(y + oy + tiny * 0.35),
            "\" text-anchor=\"middle\">",
            esc(&text),
            "</text>"
        )
        .unwrap();
    }
    let mut content = String::new();
    if !clusters.is_empty() {
        svg_write!(content, "<g class=\"pele-clusters\">", clusters, "</g>").unwrap();
    }
    if !edges_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-edges\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\" stroke-linecap=\"round\">",
            edges_out,
            "</g>"
        )
        .unwrap();
    }
    if !struck_titles.is_empty() {
        svg_write!(content, "<g class=\"pele-cluster-titles\">", struck_titles, "</g>").unwrap();
    }
    if !labels_out.is_empty() {
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", js::to_string(small), "\">", labels_out, "</g>").unwrap();
    }
    if !cards_out.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-cardinalities\" font-size=\"",
            js::to_string(tiny),
            "\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            cards_out,
            "</g>"
        )
        .unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    let titled = with_title(db.title.as_deref(), &m, size, pad, width, height, content);
    Ok(Rendered {
        svg: svg_document(
            "class",
            titled.width,
            titled.height,
            size,
            ctx,
            Accessible { acc_title: db.acc_title.as_deref(), acc_descr: db.acc_descr.as_deref() },
            &titled.content,
        ),
        width: titled.width,
        height: titled.height,
        links,
    })
}
#[allow(clippy::too_many_arguments)]
fn class_svg(
    v: &View,
    cls: &ClassNode,
    id: &str,
    at: &str,
    size: f64,
    tiny: f64,
    tiny_h: f64,
    ctx: &RenderCtx,
    links: &mut Vec<LinkInfo>,
) -> String {
    let left = -v.w / 2.0;
    let top = -v.h / 2.0;
    let member_size = js::round(v.font_size * 0.875);
    let line_h = js::round(member_size * 1.5);
    let st = &v.style;
    let mut body = svg_format!(
        "<rect x=\"",
        num(left),
        "\" y=\"",
        num(top),
        "\" width=\"",
        num(v.w),
        "\" height=\"",
        num(v.h),
        "\" rx=\"",
        RADIUS,
        "\" fill=\"",
        ctx.paint(Role::NodeFill, "var(--_s)"),
        "\" stroke=\"",
        ctx.paint(Role::NodeStroke, "var(--_b)"),
        "\"",
        st.shape,
        "/>"
    );
    let first = top + v.title_h;
    let second = first + v.attributes_h;
    if v.h > v.title_h {
        let stroke = st.stroke.as_ref().map_or_else(String::new, |s| svg_format!(" style=\"stroke:", esc(s), "\""));
        svg_write!(
            body,
            "<path class=\"pele-divider\" d=\"M",
            num(left),
            ",",
            num(first),
            "H",
            num(-left),
            "M",
            num(left),
            ",",
            num(second),
            "H",
            num(-left),
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::InnerStroke, "var(--_b)"),
            "\"",
            stroke,
            "/>"
        )
        .unwrap();
    }
    let mut y = top + 8.0;
    for annotation in &v.annotations {
        svg_write!(
            body,
            "<text class=\"pele-annotation\" x=\"0\" y=\"",
            num(y + tiny_h / 2.0 + tiny * 0.35),
            "\" text-anchor=\"middle\" font-size=\"",
            js::to_string(tiny),
            "\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            esc(annotation),
            "</text>"
        )
        .unwrap();
        y += tiny_h;
    }
    let title_size = if v.font_size == size { String::new() } else { svg_format!(" font-size=\"", num(v.font_size), "\"") };
    svg_write!(
        body,
        label_svg(
            &v.label,
            0.0,
            (y + first - 8.0) / 2.0,
            &svg_format!(" class=\"pele-label\"", title_size, " font-weight=\"var(--_hw)\"", st.text),
            ctx.icons(),
        )
    )
    .unwrap();
    let lines = |list: &[Line], kind: &str, from: f64| {
        let mut out = String::new();
        let mut baseline = from + 6.0 + line_h / 2.0 + member_size * 0.35;
        for line in list {
            let mark = if line.classifier == "$" {
                format!(
                    " pele-static\"{}",
                    if st.text.is_empty() {
                        " style=\"text-decoration:underline\"".into()
                    } else {
                        format!("{}text-decoration:underline\"", &st.text[..st.text.len() - 1])
                    }
                )
            } else if line.classifier == "*" {
                svg_format!(" pele-abstract\" font-style=\"italic\"", st.text)
            } else {
                format!("\"{}", st.text)
            };
            svg_write!(
                out,
                "<text class=\"pele-member pele-",
                kind,
                mark,
                " x=\"",
                num(left + 12.0),
                "\" y=\"",
                num(baseline),
                "\">",
                esc(&line.text),
                "</text>"
            )
            .unwrap();
            baseline += line_h;
        }
        out
    };
    if !v.attributes.is_empty() || !v.methods.is_empty() {
        svg_write!(
            body,
            "<g class=\"pele-members\" font-size=\"",
            js::to_string(member_size),
            "\">",
            lines(&v.attributes, "attribute", first),
            lines(&v.methods, "method", second),
            "</g>"
        )
        .unwrap();
    }
    if let Some(t) = cls.tooltip.as_ref().filter(|s| !s.is_empty()) {
        body = svg_format!("<title>", esc_text(t), "</title>", body);
    }
    if let Some(url) = cls.link.as_ref().filter(|s| !s.is_empty()) {
        let attrs = cls
            .link_target
            .as_deref()
            .filter(|s| matches!(*s, "_self" | "_blank" | "_parent" | "_top"))
            .map_or_else(String::new, |s| svg_format!(" target=\"", s, "\""));
        body = linked(&body, url, &cls.id, links, ctx.options, &attrs);
    }
    svg_format!(
        "<g class=\"pele-node pele-class",
        class_names(cls.css_classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&cls.css_classes)),
        "\" data-id=\"",
        id,
        "\"",
        at,
        ">",
        body,
        "</g>"
    )
}
