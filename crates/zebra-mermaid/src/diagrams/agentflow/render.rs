//! Agentflow layout and SVG, including collapsed flows and containment edges.
use super::{
    db::AgentflowModel,
    graph::{Edge, Node},
    shapes,
};
use crate::diagrams::{
    common::fit_width::{runs_across, turn_to_fit},
    flowchart::shapes::canonical,
    RenderCtx, Rendered,
};
use crate::layout::{
    compound::{cnode, compound_layout, direction, shift_layout, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg},
    cluster::{cluster_title_x, mark_crossings, Crossings},
    edges::{default_reach, loop_path, marker, marker_trim, render_edge_label_svg as edge_label_svg, route_path, EdgePath, LoopLabel},
    root::{svg_document, Accessible},
    shapes::{draw_shape, inset_route, shape_size, PAD_X, PAD_Y},
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
};
use crate::text::{entities::decode_entities, layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{
    js,
    url::{link_url, linked, rel_attr, sanitize_url, LinkInfo},
};
use crate::{Config, ConfigValue, Error};
use indexmap::IndexMap;
use std::fmt::Write as _;
struct View<'a> {
    node: &'a Node,
    style: ResolvedStyle,
    label: Label,
    shape: String,
    w: f64,
    h: f64,
    dy: f64,
    loops: usize,
}
fn option(config: &Config, key: &str, fallback: f64) -> f64 {
    config.lookup("agentflow", key).and_then(ConfigValue::as_f64).filter(|v| *v > 0.0).map_or(fallback, |v| js::min(v, 2000.0))
}
fn tint(slot: usize, wash: &str, style: &ResolvedStyle, ctx: &RenderCtx, group: bool) -> String {
    if ctx.resolved() && group {
        return svg_format!(" fill=\"", ctx.paint(Role::GroupFill, ""), "\" stroke=\"", ctx.paint(Role::GroupStroke, ""), "\"");
    }
    let color = ctx.series(slot as i64);
    svg_format!(
        " fill=\"",
        color,
        "\"",
        if style.shape.contains("\"fill:") || style.shape.contains(";fill:") {
            String::new()
        } else {
            svg_format!(" fill-opacity=\"", wash, "\"")
        },
        " stroke=\"",
        color,
        "\""
    )
}
fn flow_of(i: usize, nodes: &[CNode], root: Dir) -> Dir {
    let mut p = nodes[i].parent;
    for _ in 0..10000 {
        if p < 0 {
            break;
        }
        let c = &nodes[p as usize];
        if let Some(dir) = c.dir {
            return dir;
        }
        p = c.parent;
    }
    root
}
fn contains(nodes: &[CNode], depth: &[isize], group: usize, i: usize) -> bool {
    let mut j = i;
    for _ in depth[group]..depth[i] {
        let p = nodes[j].parent;
        if p < 0 {
            return false;
        }
        j = p as usize;
    }
    j == group && i != group
}
fn inset(route: &mut [f64], at: usize, v: &View, c: &CNode) {
    if route.len() < at + 2 {
        return;
    }
    inset_route(route, at, &v.shape, v.w, v.h, c.x, c.y, c.w, c.h, c.is_group);
}
fn dot(x: f64, y: f64) -> String {
    svg_format!("M", num(x - 1.75), ",", num(y), "a1.75,1.75 0 1 0 3.5,0a1.75,1.75 0 1 0 -3.5,0Z")
}
fn put(edge: &Edge, label: &Label, path: &EdgePath, x: f64, y: f64, ctx: &RenderCtx, edges: &mut String, labels: &mut String) {
    let st = if !edge.styles.is_empty() || !edge.compiled.is_empty() {
        let mut all = edge.compiled.clone();
        all.extend(edge.styles.iter().cloned());
        Some(resolve_style(&all))
    } else {
        None
    };
    let color = st
        .as_ref()
        .and_then(|s| s.stroke.as_ref())
        .filter(|s| !s.is_empty())
        .map_or_else(|| ctx.paint(Role::Arrow, "var(--_l)").to_owned(), |s| esc(s).into_owned());
    let kind = if edge.thickness.as_deref() == Some("dotted") {
        "reference"
    } else if edge.head == "arrow_cross" {
        "failure"
    } else {
        "sequence"
    };
    let classes =
        class_names(&edge.classes.replace("edge-thickness-normal", "").replace("edge-pattern-solid", "").replace("flowchart-link", ""));
    svg_write!(
        edges,
        "<g class=\"pele-edge pele-edge-",
        kind,
        classes,
        "\" data-id=\"",
        esc_text(&edge.id),
        "\"",
        st.as_ref().map_or("", |s| s.line.as_str()),
        "><path d=\"",
        path.d,
        "\"",
        if edge.thickness.as_deref() == Some("dotted") { " stroke-dasharray=\"3 4\"" } else { "" },
        "/>",
        marker(&edge.head, path.ex, path.ey, path.edx, path.edy, &color),
        "</g>"
    )
    .unwrap();
    if label.width > 0.0 {
        write!(
            labels,
            "{}",
            edge_label_svg(
                &edge.id,
                label,
                x,
                y,
                st.as_ref().map_or("", |s| s.text.as_str()),
                ctx.icons(),
                ctx.paint(Role::Background, "var(--_bg)"),
            )
        )
        .unwrap();
    }
}
pub(crate) fn render(db: &AgentflowModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if runs_across(db.direction.as_deref()) {
        turn_to_fit(ctx.options, |down| draw(db, config, ctx, down))
    } else {
        draw(db, config, ctx, false)
    }
}
fn draw(db: &AgentflowModel, config: &Config, ctx: &RenderCtx, turned: bool) -> Result<Rendered, Error> {
    let graph = &db.graph;
    let size = ctx.font_size();
    let edge_size = js::round(size * 0.875);
    let m = ctx.measure();
    let wrap = option(config, "wrappingWidth", 120.0);
    let min_width = option(config, "minNodeWidth", 120.0);
    let pad = ctx.padding();
    let icons = ctx.icons();
    let index: IndexMap<_, _> = graph.nodes.iter().enumerate().map(|(i, n)| (n.id.clone(), i)).collect();
    let mut views = vec![];
    let mut nodes = vec![];
    for node in &graph.nodes {
        let mut all = node.compiled.clone();
        all.extend(node.styles.iter().cloned());
        let st = resolve_style(&all);
        let base = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let fs = st.font_size.unwrap_or(size);
        let markdown = node.label_type == "markdown";
        let view;
        let mut c;
        if node.group {
            let label = layout_label(node.label.as_deref(), markdown, &m, fs, 4000.0, base);
            c = cnode(0.0, 0.0, -1);
            c.is_group = true;
            c.dir = node.dir.as_deref().filter(|s| !s.is_empty()).map(|s| direction(Some(s)));
            c.pad_x = 20.0;
            c.pad_top = if label.height > 0.0 { label.height + 16.0 } else { 20.0 };
            c.pad_bottom = 20.0;
            c.min_w = label.width + 2.0 * 20.0 - 12.0;
            view = View { node, style: st, label, shape: "group".into(), w: 0.0, h: 0.0, dy: 0.0, loops: 0 };
        } else if node.shape == "collapsedGroup" {
            let label = layout_label(node.label.as_deref(), markdown, &m, fs, wrap, base);
            let w = js::max(label.width, min_width) + 2.0 * PAD_X;
            let h = label.height + 2.0 * PAD_Y + 14.0;
            view = View { node, style: st, label, shape: "collapsed".into(), w, h, dy: -14.0 / 2.0, loops: 0 };
            c = cnode(w, h, -1);
        } else {
            let shape = canonical(&node.shape);
            let label = layout_label(node.label.as_deref(), markdown, &m, fs, wrap, base);
            let s = shape_size(shape, if shape == "diam" { label.width } else { js::max(label.width, min_width) }, label.height);
            view = View { node, style: st, label, shape: shape.into(), w: s.w, h: s.h, dy: s.dy, loops: 0 };
            c = cnode(s.w, s.h, -1);
        }
        c.seq = views.len() as f64;
        views.push(view);
        nodes.push(c);
    }
    for (i, n) in graph.nodes.iter().enumerate() {
        nodes[i].parent = n.parent.as_ref().and_then(|id| index.get(id)).map_or(-1, |&i| i as isize);
    }
    let root = if turned { Dir::TB } else { direction(db.direction.as_deref()) };
    let flow: Vec<_> = (0..nodes.len()).map(|i| flow_of(i, &nodes, root)).collect();
    let sideways: Vec<_> = flow.iter().map(|d| matches!(d, Dir::LR | Dir::RL)).collect();
    let mut depth = vec![-1isize; nodes.len()];
    for i in 0..nodes.len() {
        let mut chain = vec![];
        let mut j = i as isize;
        while j >= 0 && depth[j as usize] < 0 && chain.len() <= nodes.len() {
            chain.push(j as usize);
            j = nodes[j as usize].parent;
        }
        let mut d = if j >= 0 { depth[j as usize] + 1 } else { 0 };
        for &k in chain.iter().rev() {
            depth[k] = d;
            d += 1;
        }
    }
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    let mut held = vec![];
    for e in &graph.edges {
        let (Some(&src), Some(&dst)) = (index.get(&e.start), index.get(&e.end)) else {
            continue;
        };
        let label = layout_label(Some(&e.label), e.label_type == "markdown", &m, edge_size, wrap, 0);
        if contains(&nodes, &depth, src, dst) || contains(&nodes, &depth, dst, src) {
            held.push((e, label, src, dst));
            continue;
        }
        if src == dst {
            views[src].loops += 1;
            if sideways[src] {
                nodes[src].h += 2.0 * (26.0 + if label.height > 0.0 { label.height + 8.0 } else { 0.0 });
            } else {
                nodes[src].w += 2.0 * (26.0 + if label.width > 0.0 { label.width + 12.0 } else { 0.0 });
            }
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: e.minlen.unwrap_or(1.0),
            label_w: if label.width > 0.0 { label.width + 12.0 } else { 0.0 },
            label_h: if label.height > 0.0 { label.height + 4.0 } else { 0.0 },
            route: vec![],
            label_x: 0.0,
            label_y: 0.0,
        });
        drawn.push(e);
        labels.push(label);
    }
    let layout = compound_layout(
        &mut nodes,
        &mut edges,
        root,
        &LayeredOptions {
            node_sep: option(config, "nodeSpacing", 40.0),
            edge_sep: 16.0,
            rank_sep: option(config, "rankSpacing", 48.0),
            port_sep: 20.0,
            tracks: false,
            straight: false,
            apart: false,
            head_room: None,
        },
    );
    let title = layout_label(db.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let th = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(layout.width, title.width);
    shift_layout(&mut nodes, &mut edges, pad + (inner - layout.width) / 2.0, pad + th);
    let width = (inner + 2.0 * pad).ceil();
    let height = (layout.height + th + 2.0 * pad).ceil();
    let mut links: Vec<LinkInfo> = vec![];
    let mut held_paths = vec![];
    let mut held_labels = vec![];
    let mut held_routes = vec![];
    let careful = held.len() * nodes.len() < 2_000_000;
    for &(edge, _, src, dst) in &held {
        let out = contains(&nodes, &depth, dst, src);
        let outer = if out { dst } else { src };
        let i = if out { src } else { dst };
        let g = &nodes[outer];
        let n = &nodes[i];
        let end = match g.dir.unwrap_or(flow[outer]) {
            Dir::TB => 2,
            Dir::RL => 3,
            Dir::BT => 0,
            Dir::LR => 1,
        };
        let first = if out { end } else { (end + 2) & 3 };
        let toward = |side: usize| {
            let upright = side & 1 == 0;
            let sign = if side == 0 || side == 3 { -1.0 } else { 1.0 };
            let nx = if upright { n.x } else { n.x + sign * n.w / 2.0 };
            let ny = if upright { n.y + sign * n.h / 2.0 } else { n.y };
            let bx = if upright { n.x } else { g.x + sign * g.w / 2.0 };
            let by = if upright { g.y + sign * g.h / 2.0 } else { n.y };
            let axis = if upright { 0.0 } else { 1.0 };
            if out {
                vec![nx, ny, axis, bx, by, axis]
            } else {
                vec![bx, by, axis, nx, ny, axis]
            }
        };
        let blocked = |r: &[f64]| {
            nodes.iter().enumerate().any(|(k, c)| {
                k != i
                    && !c.is_group
                    && c.x + c.w / 2.0 > js::min(r[0], r[3]) - 1.0
                    && c.x - c.w / 2.0 < js::max(r[0], r[3]) + 1.0
                    && c.y + c.h / 2.0 > js::min(r[1], r[4]) - 1.0
                    && c.y - c.h / 2.0 < js::max(r[1], r[4]) + 1.0
            })
        };
        let mut route = toward(first);
        if careful && blocked(&route) {
            for side in [(first + 1) & 3, (first + 3) & 3, (first + 2) & 3] {
                let other = toward(side);
                if !blocked(&other) {
                    route = other;
                    break;
                }
            }
        }
        held_labels.push(((route[0] + route[3]) / 2.0, (route[1] + route[4]) / 2.0));
        held_routes.push(route.clone());
        inset(&mut route, if out { 0 } else { 3 }, &views[i], n);
        let trim = marker_trim(&edge.head);
        held_paths.push(route_path(&route, Some("linear"), 0.0, trim, 0.0, default_reach(trim)));
    }
    let mut crossings = Crossings::new();
    if nodes.iter().any(|c| c.is_group) {
        for e in &edges {
            mark_crossings(&mut crossings, &e.route);
        }
        for route in &held_routes {
            mark_crossings(&mut crossings, route);
        }
    }
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    for (i, v) in views.iter().enumerate() {
        let c = &nodes[i];
        let n = v.node;
        let (x, y) = (c.x, c.y);
        let classes = class_names(n.classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&n.classes));
        let id = esc_text(&n.id);
        if c.is_group {
            let tx = cluster_title_x(c.x, c.y, c.w, c.h, v.label.width, &crossings, 20.0);
            svg_write!(
                clusters,
                "<g class=\"pele-cluster pele-flow",
                classes,
                "\" data-id=\"",
                id,
                "\"><rect x=\"",
                num(x - c.w / 2.0),
                "\" y=\"",
                num(y - c.h / 2.0),
                "\" width=\"",
                num(c.w),
                "\" height=\"",
                num(c.h),
                "\" rx=\"",
                RADIUS,
                "\"",
                tint(n.color_index.unwrap_or(7), "0.06", &v.style, ctx, true),
                v.style.shape,
                "/>",
                label_svg(
                    &v.label,
                    tx,
                    y - c.h / 2.0 + 8.0 + v.label.height / 2.0,
                    &svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", v.style.text),
                    icons
                ),
                "</g>"
            )
            .unwrap();
            continue;
        }
        let (w, h) = (v.w, v.h);
        let shape;
        let mut kind = String::new();
        if v.shape == "collapsed" {
            let color = ctx.series(n.color_index.unwrap_or(7) as i64);
            let line = h / 2.0 - 14.0 - 2.0;
            let dots = line + (14.0 + 2.0) / 2.0;
            shape = svg_format!(
                "<rect x=\"",
                num(-w / 2.0),
                "\" y=\"",
                num(-h / 2.0),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(h),
                "\" rx=\"",
                RADIUS,
                "\"",
                tint(n.color_index.unwrap_or(7), "0.06", &v.style, ctx, true),
                v.style.shape,
                "/><path d=\"M",
                num(-w / 2.0 + 10.0),
                ",",
                num(line),
                "H",
                num(w / 2.0 - 10.0),
                "\" fill=\"none\" stroke=\"",
                color,
                "\" stroke-dasharray=\"3 3\"",
                v.style.line,
                "/><path class=\"pele-collapsed-mark\" d=\"",
                dot(-9.0, dots),
                dot(0.0, dots),
                dot(9.0, dots),
                "\" fill=\"",
                color,
                "\" stroke=\"none\"/>"
            );
        } else {
            let name = n.kind.as_deref().unwrap_or("task");
            let slot = shapes::kind_slot(name).unwrap_or(1);
            kind = format!(" pele-kind-{name}");
            shape = draw_shape(
                &v.shape,
                w,
                h,
                &(tint(slot, "0.14", &v.style, ctx, false) + &v.style.shape),
                &svg_format!(" stroke=\"", ctx.series(slot as i64), "\"", v.style.line),
                ctx.theming,
            );
        }
        let mut text = label_svg(&v.label, 0.0, v.dy, &svg_format!(" class=\"pele-label\"", v.style.text), icons).to_string();
        if classes.split(' ').any(|s| s == "internal-link")
            && n.label.as_ref().is_some_and(|s| !s.is_empty())
            && ctx.options.links != Some(false)
        {
            let name = decode_entities(n.label.as_ref().unwrap());
            let safe = link_url(&sanitize_url(&name), ctx.options).is_some();
            let target = esc(&name);
            text = svg_format!(
                "<a class=\"internal-link\"",
                if safe { svg_format!(" href=\"", target, "\"", rel_attr(ctx.options)) } else { String::new() },
                " data-href=\"",
                target,
                "\">",
                text,
                "</a>"
            );
            links.push(LinkInfo { id: n.id.clone(), href: name, internal: true });
        }
        let mut body = shape + &text;
        if let Some(t) = n.tooltip.as_ref().filter(|s| !s.is_empty()) {
            body = svg_format!("<title>", esc_text(t), "</title>", body);
        }
        if let Some(url) = n.link.as_ref().filter(|s| !s.is_empty()) {
            body = linked(
                &body,
                url,
                &n.id,
                &mut links,
                ctx.options,
                &n.link_target.as_ref().filter(|s| !s.is_empty()).map_or_else(String::new, |s| svg_format!(" target=\"", esc(s), "\"")),
            );
        }
        svg_write!(
            nodes_out,
            "<g class=\"pele-node pele-shape-",
            v.shape,
            kind,
            classes,
            "\" data-id=\"",
            id,
            "\" transform=\"translate(",
            num(x),
            ",",
            num(y),
            ")\">",
            body,
            "</g>"
        )
        .unwrap();
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loop_seen = IndexMap::<usize, usize>::new();
    for (i, ce) in edges.iter_mut().enumerate() {
        let e = drawn[i];
        let label = &labels[i];
        let trim = marker_trim(&e.head);
        let path;
        if ce.src == ce.dst {
            let v = &views[ce.src];
            let k = *loop_seen.get(&ce.src).unwrap_or(&0);
            loop_seen.insert(ce.src, k + 1);
            let mut l = LoopLabel { label_w: ce.label_w, label_h: ce.label_h, label_x: ce.label_x, label_y: ce.label_y };
            path = loop_path(nodes[ce.src].x, nodes[ce.src].y, v.w, v.h, k as f64, sideways[ce.src], 26.0, trim, &mut l);
            ce.label_x = l.label_x;
            ce.label_y = l.label_y;
        } else {
            let mut route = ce.route.clone();
            let last = route.len().saturating_sub(3);
            inset(&mut route, 0, &views[ce.src], &nodes[ce.src]);
            inset(&mut route, last, &views[ce.dst], &nodes[ce.dst]);
            path = route_path(&route, e.curve.as_ref().and_then(ConfigValue::as_str), 0.0, trim, 0.0, default_reach(trim));
        }
        put(e, label, &path, ce.label_x, ce.label_y, ctx, &mut edges_out, &mut labels_out);
    }
    for (i, (e, label, _, _)) in held.iter().enumerate() {
        put(e, label, &held_paths[i], held_labels[i].0, held_labels[i].1, ctx, &mut edges_out, &mut labels_out);
    }
    let mut content =
        label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
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
    if !labels_out.is_empty() {
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", js::to_string(edge_size), "\">", labels_out, "</g>").unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    Ok(Rendered {
        svg: svg_document(
            "agentflow",
            width,
            height,
            size,
            ctx,
            Accessible { acc_title: db.acc_title.as_deref(), acc_descr: db.acc_descr.as_deref() },
            &content,
        ),
        width,
        height,
        links,
    })
}
