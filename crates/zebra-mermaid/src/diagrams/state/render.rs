//! State composites, regions, notes and transitions, ported from Pele state/render.ts.
use super::{graph::Node, StateModel};
use crate::diagrams::{
    common::fit_width::{runs_across, turn_to_fit},
    RenderCtx, Rendered,
};
use crate::layout::{
    compound::{cnode, compound_layout, direction, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use crate::svg::{
    builder::{esc, esc_text, label_svg, num},
    edges::{edge_label_svg, marker, marker_trim, route_path, side_reach, EdgePath},
    root::{svg_document, Accessible},
    shapes::end_side,
    theme::{class_names, resolve_style, ResolvedStyle},
    title::with_title,
};
use crate::text::{entities::decode_entities, layout_label, style, Label};
use crate::theme::Role;
use crate::util::{
    js, math,
    url::{linked, safe_url},
};
use crate::{Config, ConfigValue, Error};
use indexmap::IndexMap;
use std::collections::BTreeMap;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum K {
    State,
    Described,
    Start,
    End,
    Choice,
    Fork,
    Join,
    Note,
    Composite,
    Region,
    Hidden,
}
const CLASSES: [&str; 9] = [
    "pele-node pele-state",
    "pele-node pele-state",
    "pele-node pele-state-start",
    "pele-node pele-state-end",
    "pele-node pele-state-choice",
    "pele-node pele-state-fork",
    "pele-node pele-state-join",
    "pele-note",
    "pele-cluster pele-state-composite",
];
struct View<'a> {
    node: Option<&'a Node>,
    kind: K,
    style: Option<ResolvedStyle>,
    label: Label,
    lines: Vec<Label>,
    w: f64,
    h: f64,
    title_h: f64,
}
fn view<'a>(node: Option<&'a Node>, kind: K, empty: &Label) -> View<'a> {
    View { node, kind, style: None, label: empty.clone(), lines: vec![], w: 0.0, h: 0.0, title_h: 0.0 }
}
fn vertical(d: Option<Dir>) -> bool {
    matches!(d, Some(Dir::TB | Dir::BT))
}
fn option(c: &Config, key: &str, f: f64) -> f64 {
    c.lookup("state", key).and_then(ConfigValue::as_f64).filter(|v| *v > 0.0).map_or(f, |v| js::min(v, 2000.0))
}
fn clamp(v: f64, a: f64, b: f64) -> f64 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}
fn rect(x: f64, y: f64, w: f64, h: f64, rest: &str) -> String {
    format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rest}/>", num(x), num(y), num(w), num(h))
}
fn group(dir: Dir, x: f64, top: f64, bottom: f64) -> CNode {
    let mut c = cnode(0.0, 0.0, -1);
    c.is_group = true;
    c.dir = Some(dir);
    c.pad_x = x;
    c.pad_top = top;
    c.pad_bottom = bottom;
    c
}
fn settle(done: &mut usize, queue: &mut Vec<usize>, placed: &mut [bool], kids: &[Vec<usize>]) {
    while *done < queue.len() {
        for &kid in &kids[queue[*done]] {
            if !placed[kid] {
                placed[kid] = true;
                queue.push(kid);
            }
        }
        *done += 1;
    }
}
fn encloses(nodes: &[CNode], logical: &[isize], a: usize, b: usize) -> bool {
    if !nodes[a].is_group {
        return false;
    }
    let mut p = logical[b];
    while p >= 0 {
        if p as usize == a {
            return true;
        }
        p = logical[p as usize];
    }
    false
}
fn side(route: &[f64], at: usize, c: &CNode) -> i64 {
    if at + 1 >= route.len() {
        return -1;
    }
    end_side(route, at, c.x, c.y, c.w, c.h) as i64
}
fn attach(route: &mut [f64], at: usize, c: &CNode, v: &View, pad: f64) {
    if c.is_group || route.len() < 6 {
        return;
    }
    let bar = matches!(v.kind, K::Fork | K::Join);
    let other = if at == 0 { 3 } else { at - 3 };
    for axis in [1, 0] {
        let (extent, drawn, breadth, center, cross) =
            if axis == 1 { (c.h, v.h, v.w, c.y + pad, c.x + pad) } else { (c.w, v.w, v.h, c.x + pad, c.y + pad) };
        let delta = route[at + axis] - center;
        if (delta.abs() - extent / 2.0).abs() >= 0.5 {
            continue;
        }
        route[at + axis] -= js::sign(delta) * (extent - drawn) / 2.0;
        if bar && breadth > drawn {
            route[at + 1 - axis] = clamp(route[other + 1 - axis], cross - breadth / 2.0 + 6.0, cross + breadth / 2.0 - 6.0);
        }
        return;
    }
}
fn title_x(c: &CNode, w: f64, crossings: &BTreeMap<i64, Vec<f64>>) -> f64 {
    let Some(xs) = crossings.get(&(js::round(c.y - c.h / 2.0) as i64)) else {
        return c.x;
    };
    if w == 0.0 {
        return c.x;
    }
    let left = c.x - c.w / 2.0;
    let half = w / 2.0;
    let mut best = c.x;
    let mut best_gap = -1.0;
    for x in [c.x, left + 16.0 + half, left + c.w - 16.0 - half] {
        let mut gap = f64::INFINITY;
        for &cx in xs {
            if cx > left && cx < left + c.w {
                gap = js::min(gap, (cx - x).abs() - half);
            }
        }
        if gap >= 6.0 {
            return x;
        }
        if gap > best_gap + 0.5 {
            best_gap = gap;
            best = x;
        }
    }
    best
}
fn divide(items: &[usize], bounds: [f64; 4], nodes: &[CNode], views: &[View], pad: f64, out: &mut String) {
    let regions: Vec<_> = items.iter().copied().filter(|&i| views[i].kind == K::Region).collect();
    if regions.len() < 2 {
        return;
    }
    let [left, top, right, bottom] = bounds;
    let lo = |i: usize, x| if x { nodes[i].x - nodes[i].w / 2.0 } else { nodes[i].y - nodes[i].h / 2.0 };
    let hi = |i: usize, x| if x { nodes[i].x + nodes[i].w / 2.0 } else { nodes[i].y + nodes[i].h / 2.0 };
    for x in [true, false] {
        let mut sorted = regions.clone();
        sorted.sort_by(|&a, &b| lo(a, x).total_cmp(&lo(b, x)).then(a.cmp(&b)));
        if (1..sorted.len()).any(|k| hi(sorted[k - 1], x) > lo(sorted[k], x) + 0.5) {
            continue;
        }
        for k in 1..sorted.len() {
            let at = num((hi(sorted[k - 1], x) + lo(sorted[k], x)) / 2.0 + pad);
            let d = if x { format!("{at},{}V{}", num(top), num(bottom)) } else { format!("{},{at}H{}", num(left), num(right)) };
            out.push_str(&format!("<path class=\"pele-divider\" d=\"M{d}\"/>"));
        }
        return;
    }
    for i in regions {
        let c = &nodes[i];
        out.push_str(&rect(c.x - c.w / 2.0 + pad, c.y - c.h / 2.0 + pad, c.w, c.h, " class=\"pele-divider\" rx=\"10\""));
    }
}
pub(crate) fn render(db: &StateModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if runs_across(Some(&db.direction)) {
        turn_to_fit(ctx.options, |down| draw(db, config, ctx, down))
    } else {
        draw(db, config, ctx, false)
    }
}
fn draw(db: &StateModel, config: &Config, ctx: &RenderCtx, turned: bool) -> Result<Rendered, Error> {
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let wrap = option(config, "wrappingWidth", 200.0);
    let pad = ctx.padding();
    let icons = ctx.icons();
    let root_dir = if turned { Dir::TB } else { direction(Some(&db.direction)) };
    let empty = layout_label(None, false, &m, size, 0.0, 0);
    let text = |raw: Option<&str>, size, wrap, style| layout_label(raw, true, &m, size, wrap, style);
    let mut index = IndexMap::new();
    let mut views = vec![];
    let mut nodes = vec![];
    for n in &db.graph.nodes {
        if n.shape.as_deref() == Some("noteGroup") {
            continue;
        }
        let kind = if n.group {
            if n.shape.as_deref() == Some("divider") {
                K::Region
            } else {
                K::Composite
            }
        } else {
            match n.shape.as_deref() {
                Some("rectWithTitle") => K::Described,
                Some("stateStart") => K::Start,
                Some("stateEnd") => K::End,
                Some("choice") => K::Choice,
                Some("fork") => K::Fork,
                Some("join") => K::Join,
                Some("note") => K::Note,
                Some("divider") => K::Hidden,
                _ => K::State,
            }
        };
        index.insert(n.id.clone(), views.len());
        views.push(view(Some(n), kind, &empty));
    }
    let states = views.len();
    let mut logical: Vec<isize> = (0..states)
        .map(|i| {
            views[i]
                .node
                .unwrap()
                .parent
                .as_ref()
                .and_then(|p| index.get(p))
                .copied()
                .filter(|&p| p != i && views[p].node.unwrap().group)
                .map_or(-1, |p| p as isize)
        })
        .collect();
    {
        let mut kids = vec![vec![]; states];
        let mut queue = vec![];
        let mut placed = vec![false; states];
        for i in 0..states {
            if logical[i] < 0 {
                placed[i] = true;
                queue.push(i);
            } else {
                kids[logical[i] as usize].push(i);
            }
        }
        let mut done = 0;
        settle(&mut done, &mut queue, &mut placed, &kids);
        let mut seen = vec![-1isize; states];
        for i in 0..states {
            if placed[i] {
                continue;
            }
            let mut ring = i;
            while seen[ring] != i as isize {
                seen[ring] = i as isize;
                ring = logical[ring] as usize;
            }
            logical[ring] = -1;
            placed[ring] = true;
            queue.push(ring);
            settle(&mut done, &mut queue, &mut placed, &kids);
        }
    }
    let mut owners = vec![-1isize; states];
    for i in 0..states {
        if let Some(&owner) = views[i].node.unwrap().note_for.as_ref().and_then(|s| index.get(s)) {
            owners[i] = owner as isize;
            logical[i] = logical[owner];
        }
    }
    let inner: Vec<_> = views.iter().map(|v| direction(v.node.unwrap().dir.as_deref())).collect();
    let across: Vec<_> = (0..states).map(|i| vertical(Some(if logical[i] < 0 { root_dir } else { inner[logical[i] as usize] }))).collect();
    let mut degree = vec![0usize; states];
    let mut noted = vec![false; states];
    let mut appear = vec![f64::INFINITY; states];
    let mut appeared = 0.0;
    let mut counts = vec![0; states * 2];
    for e in &db.graph.edges {
        let (Some(&src), Some(&dst)) = (index.get(&e.start), index.get(&e.end)) else {
            continue;
        };
        if e.dashed {
            noted[if views[src].kind == K::Note { dst } else { src }] = true;
            continue;
        }
        counts[2 * src] += 1;
        degree[src] = degree[src].max(counts[2 * src]);
        counts[2 * dst + 1] += 1;
        degree[dst] = degree[dst].max(counts[2 * dst + 1]);
        if appear[src] == f64::INFINITY {
            appear[src] = appeared;
            appeared += 1.0;
        }
        if appear[dst] == f64::INFINITY {
            appear[dst] = appeared;
            appeared += 1.0;
        }
    }
    for i in 0..states {
        let v = &mut views[i];
        let n = v.node.unwrap();
        let mut styles = n.compiled.clone();
        styles.extend(n.styles.iter().cloned());
        let st = resolve_style(&styles);
        let base = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let fs = st.font_size.unwrap_or(size);
        v.style = Some(st);
        let mut compound = None;
        match v.kind {
            K::Composite => {
                v.label = text(n.label.as_deref(), fs, 4000.0, base);
                v.title_h = if v.label.height > 0.0 { v.label.height + 12.0 } else { 0.0 };
                let mut c = group(inner[i], 16.0, v.title_h + 16.0, 16.0);
                c.min_w = v.label.width + 2.0 * 16.0;
                compound = Some(c);
            }
            K::Region => compound = Some(group(inner[i], 10.0, 10.0, 10.0)),
            K::Start => {
                v.w = 14.0;
                v.h = 14.0;
            }
            K::End => {
                v.w = 18.0;
                v.h = 18.0;
            }
            K::Choice => {
                v.w = 30.0;
                v.h = 30.0;
            }
            K::Fork | K::Join => {
                let length = clamp(40.0 * degree[i] as f64 + 16.0, 72.0, 320.0);
                v.w = if across[i] { length } else { 8.0 };
                v.h = if across[i] { 8.0 } else { length };
            }
            K::Hidden => {}
            K::Note => {
                v.label = text(n.label.as_deref(), small, js::max(wrap, 320.0), 0);
                v.w = v.label.width + 24.0;
                v.h = v.label.height + 16.0;
            }
            _ => {
                v.label = text(n.label.as_deref(), fs, wrap, base);
                let mut w = v.label.width;
                let mut h = v.label.height + 2.0 * 10.0;
                if v.kind == K::Described {
                    h += 14.0;
                    for line in &n.description {
                        let label = text(Some(if line.is_empty() { " " } else { line }), small, wrap, base);
                        if label.width > w {
                            w = label.width;
                        }
                        h += label.height;
                        v.lines.push(label);
                    }
                }
                v.w = js::max(w + 2.0 * 16.0, 56.0);
                v.h = h;
            }
        }
        let mut c = compound.unwrap_or_else(|| cnode(v.w, v.h, -1));
        if matches!(v.kind, K::State | K::Described) {
            c.span = if across[i] { v.w } else { v.h } - 24.0;
        }
        nodes.push(c);
    }
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    let mut loop_count = vec![0usize; states];
    let mut loop_width = vec![0.0; states];
    let mut loop_height = vec![0.0; states];
    let mut anchors = IndexMap::new();
    for e in &db.graph.edges {
        let (Some(&a), Some(&b)) = (index.get(&e.start), index.get(&e.end)) else {
            continue;
        };
        let (mut src, mut dst) = (a, b);
        let label = text(Some(&e.label), small, wrap, 0);
        let lw = if label.width > 0.0 { label.width + 12.0 } else { 0.0 };
        if src == dst {
            loop_count[src] += 1;
            loop_width[src] = js::max(loop_width[src], lw);
            loop_height[src] += label.height;
        } else if encloses(&nodes, &logical, src, dst) {
            anchors.insert(edges.len(), src);
            appear.push(-1.0);
            let id = views.len();
            views.push(view(None, K::Hidden, &empty));
            nodes.push(cnode(0.0, 0.0, src as isize));
            src = id;
        } else if encloses(&nodes, &logical, dst, src) {
            anchors.insert(edges.len(), dst);
            appear.push(f64::INFINITY);
            let id = views.len();
            views.push(view(None, K::Hidden, &empty));
            nodes.push(cnode(0.0, 0.0, dst as isize));
            dst = id;
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: 1.0,
            label_w: lw,
            label_h: if label.height > 0.0 { label.height + 4.0 } else { 0.0 },
            route: vec![],
            label_x: 0.0,
            label_y: 0.0,
        });
        drawn.push(e);
        labels.push(label);
    }
    for i in 0..states {
        if owners[i] >= 0 {
            continue;
        }
        let mut room = 0.0;
        if loop_count[i] > 0 {
            let reach = 26.0 + (loop_count[i] - 1) as f64 * 8.0;
            room = if across[i] { reach + loop_width[i] } else { reach * 0.75 + 4.0 + loop_height[i] };
            if !nodes[i].is_group {
                if across[i] {
                    nodes[i].w += 2.0 * room;
                    nodes[i].h = js::max(nodes[i].h, loop_height[i]);
                } else {
                    nodes[i].h += 2.0 * room;
                    nodes[i].w = js::max(nodes[i].w, loop_width[i]);
                }
                room = 0.0;
            }
        }
        if !noted[i] && room == 0.0 {
            nodes[i].parent = logical[i];
            continue;
        }
        let mut c = if across[i] { group(Dir::LR, room, 0.0, 0.0) } else { group(Dir::TB, 0.0, room, room) };
        c.parent = logical[i];
        appear.push(appear[i]);
        nodes[i].parent = views.len() as isize;
        views.push(view(None, K::Hidden, &empty));
        nodes.push(c);
    }
    for i in 0..states {
        if owners[i] >= 0 {
            nodes[i].parent = nodes[owners[i] as usize].parent;
        }
    }
    {
        let mut members = vec![vec![]; nodes.len()];
        let mut top = vec![];
        for (i, c) in nodes.iter().enumerate() {
            if c.parent >= 0 {
                members[c.parent as usize].push(i);
            } else {
                top.push(i);
            }
        }
        let by_flow = |a: &usize, b: &usize| {
            (views[*a].kind != K::Start)
                .cmp(&(views[*b].kind != K::Start))
                .then_with(|| appear[*a].partial_cmp(&appear[*b]).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.cmp(b))
        };
        top.sort_by(by_flow);
        top.reverse();
        let mut seq = 0.0;
        while let Some(i) = top.pop() {
            nodes[i].seq = seq;
            seq += 1.0;
            members[i].sort_by(by_flow);
            top.extend(members[i].iter().rev().copied());
        }
    }
    let layout = compound_layout(
        &mut nodes,
        &mut edges,
        root_dir,
        &LayeredOptions {
            node_sep: option(config, "nodeSpacing", 40.0),
            edge_sep: 16.0,
            rank_sep: option(config, "rankSpacing", 44.0),
            port_sep: 20.0,
            head_room: Some(marker_trim("arrow_point")),
            tracks: false,
            straight: false,
            apart: false,
        },
    );
    let width = (layout.width + 2.0 * pad).ceil();
    let height = (layout.height + 2.0 * pad).ceil();
    let mut links = vec![];
    let mut crossings = BTreeMap::<i64, Vec<f64>>::new();
    for e in &edges {
        for k in (3..e.route.len().saturating_sub(3)).step_by(3) {
            crossings.entry(js::round(e.route[k + 1]) as i64).or_default().push(e.route[k]);
        }
    }
    let mut children = vec![vec![]; nodes.len()];
    let mut order = vec![];
    for (i, c) in nodes.iter().enumerate() {
        if c.parent >= 0 {
            children[c.parent as usize].push(i);
        } else {
            order.push(i);
        }
    }
    let roots = order.clone();
    let mut q = 0;
    while q < order.len() {
        order.extend(children[order[q]].iter().copied());
        q += 1;
    }
    let mut clusters = String::new();
    let mut dividers = String::new();
    let mut nodes_out = String::new();
    let mut notes_out = String::new();
    divide(&roots, [pad / 2.0, pad / 2.0, width - pad / 2.0, height - pad / 2.0], &nodes, &views, pad, &mut dividers);
    let surface = format!(" fill=\"{}\" stroke=\"{}\"", ctx.paint(Role::NodeFill, "var(--_s)"), ctx.paint(Role::NodeStroke, "var(--_b)"));
    let rule = format!(" fill=\"none\" stroke=\"{}\"", ctx.paint(Role::InnerStroke, "var(--_b)"));
    let line = ctx.paint(Role::Line, "var(--_l)");
    let bg = ctx.paint(Role::Background, "var(--_bg)");
    for i in order {
        let v = &views[i];
        let (Some(n), Some(st)) = (v.node, v.style.as_ref()) else {
            continue;
        };
        if v.kind >= K::Region {
            continue;
        }
        let c = &nodes[i];
        let (x, y, w, h) = (c.x + pad, c.y + pad, v.w, v.h);
        let label_attrs = format!(" class=\"pele-label\"{}", st.text);
        let mut body;
        match v.kind {
            K::Composite => {
                let left = x - c.w / 2.0;
                let top = y - c.h / 2.0;
                body = rect(
                    left,
                    top,
                    c.w,
                    c.h,
                    &format!(
                        " rx=\"10\" fill=\"{}\"{} stroke=\"{}\"{}",
                        ctx.paint(Role::GroupFill, "var(--_a)"),
                        if ctx.resolved() { "" } else { " fill-opacity=\"0.5\"" },
                        ctx.paint(Role::GroupStroke, "var(--_b)"),
                        st.shape
                    ),
                );
                if v.title_h > 0.0 {
                    body.push_str(&format!("<path d=\"M{},{}H{}\"{rule}{}/>", num(left), num(top + v.title_h), num(left + c.w), st.line));
                    body.push_str(&label_svg(
                        &v.label,
                        title_x(c, v.label.width, &crossings) + pad,
                        top + v.title_h / 2.0,
                        &format!(" class=\"pele-cluster-label\"{}", st.text),
                        icons,
                    ));
                }
                divide(&children[i], [left, top + v.title_h, left + c.w, top + c.h], &nodes, &views, pad, &mut dividers);
            }
            K::Start => body = format!("<circle r=\"7\" fill=\"{line}\"{}/>", st.shape),
            K::End => {
                body =
                    format!("<circle r=\"8.5\" fill=\"{bg}\" stroke=\"{line}\"{}/><circle r=\"5\" fill=\"{line}\"{}/>", st.line, st.shape)
            }
            K::Choice => {
                body = format!(
                    "<polygon points=\"0,{} {},0 0,{} {},0\"{surface}{}/>",
                    num(-h / 2.0),
                    num(w / 2.0),
                    num(h / 2.0),
                    num(-w / 2.0),
                    st.shape
                )
            }
            K::Fork | K::Join => body = rect(-w / 2.0, -h / 2.0, w, h, &format!(" rx=\"2\" fill=\"{line}\"{}", st.shape)),
            K::Note => {
                let r = num(w / 2.0);
                let b = num(h / 2.0);
                let fold = num(w / 2.0 - 8.0);
                let corner = num(8.0 - h / 2.0);
                body=format!("<path d=\"M-{r},-{b}H{fold}L{r},{corner}V{b}H-{r}Z\" fill=\"{}\" stroke=\"{}\"/><path d=\"M{fold},-{b}V{corner}H{r}\"{rule}/>{}",ctx.paint(Role::WarningBackground,"var(--_bg)"),ctx.paint(Role::NodeStroke,"var(--_b)"),label_svg(&v.label,0.0,0.0,&format!(" class=\"pele-label\" font-size=\"{}\" fill=\"{}\"",js::to_string(small),ctx.paint(Role::WarningText,"var(--_m)")),icons));
            }
            _ => {
                body = rect(-w / 2.0, -h / 2.0, w, h, &format!(" rx=\"{}\"{surface}{}", num(js::min(10.0, h / 2.0)), st.shape));
                if v.kind == K::State {
                    body.push_str(&label_svg(&v.label, 0.0, 0.0, &label_attrs, icons));
                } else {
                    let mut at = 10.0 - h / 2.0 + v.label.height;
                    body.push_str(&format!(
                        "<path d=\"M{},{}H{}\"{rule}{}/>{}",
                        num(-w / 2.0),
                        num(at + 6.0),
                        num(w / 2.0),
                        st.line,
                        label_svg(&v.label, 0.0, at - v.label.height / 2.0, &label_attrs, icons)
                    ));
                    at += 14.0;
                    for l in &v.lines {
                        body.push_str(&label_svg(
                            l,
                            0.0,
                            at + l.height / 2.0,
                            &format!(" class=\"pele-state-description\" font-size=\"{}\"{}", js::to_string(small), st.text),
                            icons,
                        ));
                        at += l.height;
                    }
                }
            }
        }
        if let Some((url, tooltip)) = db.links.get(&n.id) {
            let tip = decode_entities(tooltip.trim_matches('"'));
            if !tip.is_empty() {
                body = format!("<title>{}</title>{body}", esc(&tip));
            }
            body = linked(&body, &safe_url(url.trim_matches('"')), &n.id, &mut links, ctx.options, " target=\"_blank\"");
        }
        // The regex removes only space-delimited Mermaid bookkeeping classes.
        let classes = n
            .classes
            .split(' ')
            .filter(|s| !matches!(*s, "statediagram-state" | "statediagram-cluster-alt" | "statediagram-cluster" | "statediagram-note"))
            .collect::<Vec<_>>()
            .join(" ");
        let open = format!("<g class=\"{}{}\" data-id=\"{}\"", CLASSES[v.kind as usize], class_names(&classes), esc_text(&n.id));
        let complete = if v.kind == K::Composite {
            format!("{open}>{body}</g>")
        } else {
            format!("{open} transform=\"translate({},{})\">{body}</g>", num(x), num(y))
        };
        if v.kind == K::Composite {
            clusters.push_str(&complete);
        } else if v.kind == K::Note {
            notes_out.push_str(&complete);
        } else {
            nodes_out.push_str(&complete);
        }
    }
    let mut reach = side_reach();
    for (i, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 || drawn[i].dashed {
            continue;
        }
        reach.add(e.dst as i64, side(&e.route, e.route.len() - 3, &nodes[e.dst]), "arrow_point");
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loop_seen = vec![0usize; states];
    let mut loop_text = vec![0.0; states];
    for (i, ce) in edges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &labels[i];
        let note = edge.dashed;
        let end_type = if note { "none" } else { "arrow_point" };
        let path;
        if ce.src == ce.dst {
            let a = &nodes[ce.src];
            let v = &views[ce.src];
            let k = loop_seen[ce.src];
            loop_seen[ce.src] += 1;
            let side = across[ce.src];
            let w = if a.is_group { a.w } else { v.w };
            let h = if a.is_group { a.h } else { v.h };
            let out = if side { w } else { h } / 2.0;
            let spread = js::max(js::min(if side { h } else { w } / 2.0 - 4.0, 8.0 + k as f64 * 6.0), 2.0);
            let reach = 26.0 + k as f64 * 8.0;
            let len = math::hypot(reach, spread);
            let trim = marker_trim(end_type) / if len == 0.0 { 1.0 } else { len };
            let px = |dx: f64, dy: f64| a.x + pad + if side { out + dx } else { -dy };
            let py = |dx: f64, dy: f64| a.y + pad + if side { dy } else { out + dx };
            let at = |dx, dy| format!("{},{}", num(px(dx, dy)), num(py(dx, dy)));
            let mut p = EdgePath {
                d: format!(
                    "M{}C{} {} {}",
                    at(0.0, -spread),
                    at(reach, -2.0 * spread),
                    at(reach, 2.0 * spread),
                    at(reach * trim, spread + spread * trim)
                ),
                sx: 0.0,
                sy: 0.0,
                sdx: 0.0,
                sdy: 0.0,
                ex: px(0.0, spread),
                ey: py(0.0, spread),
                edx: px(-reach, -spread) - px(0.0, 0.0),
                edy: py(-reach, -spread) - py(0.0, 0.0),
            };
            let length = math::hypot(p.edx, p.edy);
            let length = if length == 0.0 { 1.0 } else { length };
            p.edx /= length;
            p.edy /= length;
            path = p;
            let furthest = 26.0 + (loop_count[ce.src] - 1) as f64 * 8.0;
            let below = loop_text[ce.src] + label.height / 2.0;
            loop_text[ce.src] += label.height;
            ce.label_x = if side { a.x + out + furthest + 4.0 + ce.label_w / 2.0 } else { a.x };
            ce.label_y = if side { a.y - loop_height[ce.src] / 2.0 + below } else { a.y + out + furthest * 0.75 + 4.0 + below };
        } else {
            let mut route = ce.route.clone();
            let last = route.len().saturating_sub(3);
            let leaves = reach.of(ce.src as i64, side(&route, 0, &nodes[ce.src]));
            let arrives = reach.of(ce.dst as i64, side(&route, last, &nodes[ce.dst]));
            for k in (0..route.len()).step_by(3) {
                route[k] += pad;
                route[k + 1] += pad;
            }
            attach(&mut route, 0, &nodes[ce.src], &views[ce.src], pad);
            attach(&mut route, last, &nodes[ce.dst], &views[ce.dst], pad);
            if let Some(&composite) = anchors.get(&i) {
                if route.len() >= 6 {
                    let g = &nodes[composite];
                    let inward = views[ce.src].kind == K::Hidden;
                    let at = if inward { 0 } else { route.len() - 3 };
                    let near = matches!(g.dir, Some(Dir::TB | Dir::LR)) == inward;
                    let point = if vertical(g.dir) {
                        [route[at], g.y + pad + if near { views[composite].title_h - g.h / 2.0 } else { g.h / 2.0 }, 0.0]
                    } else {
                        [g.x + pad + if near { -g.w / 2.0 } else { g.w / 2.0 }, route[at + 1], 1.0]
                    };
                    if inward {
                        route.splice(0..0, point);
                    } else {
                        route.extend(point);
                    }
                }
            }
            path = route_path(&route, None, 0.0, marker_trim(end_type), leaves, arrives);
        }
        let class = if note {
            format!("pele-note-edge\" stroke=\"{}\" stroke-dasharray=\"3 4", ctx.paint(Role::NodeStroke, "var(--_b)"))
        } else {
            "pele-transition".into()
        };
        edges_out.push_str(&format!(
            "<g class=\"pele-edge {class}\" data-id=\"{}\"><path d=\"{}\"/>{}</g>",
            esc_text(&edge.id),
            path.d,
            marker(end_type, path.ex, path.ey, path.edx, path.edy, ctx.paint(Role::Arrow, "var(--_l)"))
        ));
        if label.width > 0.0 {
            labels_out.push_str(&edge_label_svg(&edge.id, label, ce.label_x + pad, ce.label_y + pad, "", icons, bg));
        }
    }
    let mut content = String::new();
    if !clusters.is_empty() {
        content.push_str(&format!("<g class=\"pele-clusters\">{clusters}</g>"));
    }
    if !dividers.is_empty() {
        content.push_str(&format!("<g class=\"pele-dividers\" stroke-dasharray=\"4 4\"{rule}>{dividers}</g>"));
    }
    if !edges_out.is_empty() {
        content.push_str(&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"{line}\" stroke-linecap=\"round\">{edges_out}</g>"));
    }
    if !labels_out.is_empty() {
        content.push_str(&format!("<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>", js::to_string(small)));
    }
    content.push_str(&format!("<g class=\"pele-nodes\">{nodes_out}{notes_out}</g>"));
    let titled = with_title(db.title.as_deref(), &m, size, pad, width, height, content);
    Ok(Rendered {
        svg: svg_document(
            "state",
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
