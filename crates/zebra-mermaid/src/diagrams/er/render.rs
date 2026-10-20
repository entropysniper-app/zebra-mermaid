//! Entity tables, subgraphs and crow's-foot paths from Pele er/render.ts.
use super::{Attribute, ErModel};
use crate::diagrams::common::{
    fit_width::{runs_across, turn_to_fit},
    generics::parse_generic_types,
};
use crate::diagrams::{RenderCtx, Rendered};
use crate::layout::{
    compound::{cnode, compound_layout, direction, shift_layout, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{
        esc, esc_text, joined, num, push_esc, push_num, render_label_svg as label_svg, render_shared_style as shared_style, span_style,
        text_style,
    },
    cluster::{cluster_title_x, mark_crossings, Crossings},
    edges::{default_reach, render_edge_label_svg as edge_label_svg, route_path},
    root::{svg_document, Accessible},
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
};
use crate::text::{entities::decode_entities, layout_render_label as layout_label, style, Icons, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math, url::*};
use crate::{Config, ConfigValue, Error};
use indexmap::{IndexMap, IndexSet};
use std::fmt::Write;
#[derive(Clone)]
struct GraphNode {
    id: String,
    label: String,
    alias: String,
    attrs: Vec<Attribute>,
    group: bool,
    parent: Option<String>,
    styles: Vec<String>,
    classes: String,
    dir: Option<String>,
}
struct GraphEdge {
    id: String,
    start: String,
    end: String,
    label: String,
    head: String,
    tail: String,
    dashed: bool,
}
struct Row {
    cells: [Label; 4],
    h: f64,
}
struct View {
    node: GraphNode,
    style: ResolvedStyle,
    label: Label,
    w: f64,
    h: f64,
    head_h: f64,
    rows: Vec<Row>,
    cols: Vec<f64>,
    row_size: f64,
    key_size: f64,
    below: bool,
}
#[derive(Default)]
struct Loop {
    band: f64,
    at: f64,
}
fn graph(db: &ErModel) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let compiled = |classes: Vec<&str>| {
        let mut out = vec![];
        for class in classes {
            if let Some(c) = db.classes.get(class) {
                out.extend(c.styles.iter().chain(&c.text_styles).map(|s| js::trim(s).to_owned()));
            }
        }
        out
    };
    let mut parents = IndexMap::new();
    let mut group_ids = IndexSet::new();
    for g in db.groups.iter().rev() {
        group_ids.insert(g.id.clone());
        for child in &g.nodes {
            parents.insert(child.clone(), g.id.clone());
        }
    }
    let mut nodes = vec![];
    for g in db.groups.iter().rev() {
        let mut styles = compiled(g.classes.iter().map(String::as_str).collect());
        styles.extend(g.styles.iter().cloned());
        nodes.push(GraphNode {
            id: g.id.clone(),
            label: g.title.clone(),
            alias: String::new(),
            attrs: vec![],
            group: true,
            parent: parents.get(&g.id).cloned(),
            styles,
            classes: g.classes.join(" "),
            dir: g.dir.clone(),
        });
    }
    let mut moved = IndexMap::new();
    for (name, e) in &db.entities {
        if group_ids.contains(name) {
            moved.insert(e.id.clone(), name.clone());
            continue;
        }
        let mut styles = compiled(e.classes.split(' ').collect());
        styles.extend(e.styles.iter().cloned());
        nodes.push(GraphNode {
            id: e.id.clone(),
            label: e.label.clone(),
            alias: e.alias.clone(),
            attrs: e.attributes.clone(),
            group: false,
            parent: parents.get(name).cloned(),
            styles,
            classes: e.classes.clone(),
            dir: None,
        });
    }
    let edges = db
        .relationships
        .iter()
        .enumerate()
        .map(|(i, r)| GraphEdge {
            id: format!("id_{}_{}_{i}", r.a, r.b),
            start: moved.get(&r.a).unwrap_or(&r.a).clone(),
            end: moved.get(&r.b).unwrap_or(&r.b).clone(),
            label: r.role.clone(),
            tail: r.card_b.clone(),
            head: r.card_a.clone(),
            dashed: !r.identifying,
        })
        .collect();
    (nodes, edges)
}
fn cell(raw: &str) -> String {
    let converted = parse_generic_types(raw);
    if converted == raw {
        converted
    } else {
        converted.replace('<', "&lt;").replace('>', "&gt;")
    }
}
/// Appends a label laid out from its left edge at `x`.
fn push_text_left(out: &mut String, label: &Label, x: f64, mut y: f64, attrs: &str, icons: Option<&Icons>) {
    if label.lines().is_empty() {
        return;
    }
    if label.lines().len() == 1 && label.lines()[0].len() == 1 && label.lines()[0][0].icon.is_none() && label.lines()[0][0].style == 0 {
        out.push_str("<text");
        out.push_str(attrs);
        out.push_str(" x=\"");
        push_num(out, x);
        out.push_str("\" y=\"");
        push_num(out, y);
        out.push_str("\">");
        push_esc(out, &label.lines()[0][0].text);
        out.push_str("</text>");
        return;
    }
    let shared = shared_style(label);
    svg_write!(out, "<text", text_style(attrs, shared), " xml:space=\"preserve\">").unwrap();
    // Icons are drawn after the text element.
    let mut extra = String::new();
    for line in &label.lines() {
        let has_icon = line.iter().any(|s| s.icon.is_some());
        let mut sx = x;
        let mut placed = false;
        for s in line {
            if let Some(icon) = &s.icon {
                let side = label.size * 1.1;
                svg_write!(extra, "<svg class=\"pele-icon\" data-icon=\"", esc_text(icon), "\" x=\"", num(sx), "\" y=\"", num(y-label.size*0.35-side/2.0), "\" width=\"", num(side), "\" height=\"", num(side), "\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">", icons.map_or_else(String::new,|r|r.markup(icon).to_string()), "</svg>").unwrap();
            } else if !s.text.is_empty() {
                out.push_str("<tspan");
                if has_icon || !placed {
                    out.push_str(" x=\"");
                    push_num(out, sx);
                    out.push_str("\" y=\"");
                    push_num(out, y);
                    out.push('"');
                }
                out.push_str(span_style(s.style & !shared));
                out.push('>');
                push_esc(out, &s.text);
                out.push_str("</tspan>");
                placed = true;
            }
            sx += s.width;
        }
        y += label.line_height;
    }
    out.push_str("</text>");
    out.push_str(&extra);
}
fn marker(kind: &str, x: f64, y: f64, dx: f64, dy: f64, bg: &str) -> String {
    if !matches!(kind, "only_one" | "zero_or_one" | "one_or_more" | "zero_or_more" | "md_parent") {
        return String::new();
    }
    // Write only the geometry this cardinality uses, directly into the final marker.
    let mut out = String::with_capacity(192 + bg.len());
    let at = |out: &mut String, command: char, back, side| {
        write!(out, "{command}{},{}", num(x - dx * back - dy * side), num(y - dy * back + dx * side)).unwrap();
    };
    let bar = |out: &mut String, back| {
        at(out, 'M', back, -5.0);
        at(out, 'L', back, 5.0);
    };
    out.push_str("<path class=\"pele-marker\" d=\"");
    match kind {
        "only_one" => {
            bar(&mut out, 6.0);
            bar(&mut out, 10.0);
        }
        "zero_or_one" => bar(&mut out, 6.0),
        "one_or_more" | "zero_or_more" => {
            at(&mut out, 'M', 0.0, -6.0);
            at(&mut out, 'L', 10.0, 0.0);
            at(&mut out, 'L', 0.0, 6.0);
            if kind == "one_or_more" {
                bar(&mut out, 13.5);
            }
        }
        "md_parent" => {
            at(&mut out, 'M', 0.0, 0.0);
            at(&mut out, 'L', 6.0, 4.0);
            at(&mut out, 'L', 12.0, 0.0);
            at(&mut out, 'L', 6.0, -4.0);
            svg_write!(out, "Z\" fill=\"", bg, "\"/>").unwrap();
            return out;
        }
        _ => unreachable!(),
    }
    out.push_str("\"/>");
    let ring = match kind {
        "zero_or_one" => Some(13.5),
        "zero_or_more" => Some(14.0),
        _ => None,
    };
    if let Some(back) = ring {
        svg_write!(
            out,
            "<circle class=\"pele-marker\" cx=\"",
            num(x - dx * back),
            "\" cy=\"",
            num(y - dy * back),
            "\" r=\"3.5\" fill=\"",
            bg,
            "\"/>"
        )
        .unwrap();
    }
    out
}
fn option(c: &Config, key: &str, fallback: f64) -> f64 {
    c.lookup("er", key).and_then(ConfigValue::as_f64).filter(|n| *n > 0.0).map_or(fallback, |n| js::min(n, 2000.0))
}
pub(crate) fn render(db: &ErModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if runs_across(Some(&db.direction)) {
        turn_to_fit(ctx.options, |down| draw(db, config, ctx, down))
    } else {
        draw(db, config, ctx, false)
    }
}
fn draw(db: &ErModel, config: &Config, ctx: &RenderCtx, turned: bool) -> Result<Rendered, Error> {
    let (graph, graph_edges) = graph(db);
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let icons = ctx.icons();
    let index: IndexMap<String, usize> = graph.iter().enumerate().map(|(i, n)| (n.id.clone(), i)).collect();
    let mut edges = vec![];
    let mut edge_labels = vec![];
    let mut drawn = vec![];
    let mut loops = IndexMap::<usize, Vec<usize>>::new();
    let mut loop_of: Vec<Option<Loop>> = vec![];
    for e in &graph_edges {
        let (Some(&src), Some(&dst)) = (index.get(&e.start), index.get(&e.end)) else {
            continue;
        };
        let label = layout_label(Some(&e.label), true, &m, small, 200.0, 0);
        let l = if src == dst {
            loops.entry(src).or_default().push(edges.len());
            Some(Loop::default())
        } else {
            None
        };
        loop_of.push(l);
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
        edge_labels.push(label);
        drawn.push(e);
    }
    let horizontal = |i: usize| {
        let mut at = i;
        for _ in 0..graph.len() {
            let Some(up) = graph[at].parent.as_ref().and_then(|s| index.get(s)).copied() else {
                break;
            };
            if let Some(d) = graph[up].dir.as_ref().filter(|s| !s.is_empty()) {
                return d == "LR" || d == "RL";
            }
            at = up;
        }
        db.direction == "LR" || db.direction == "RL"
    };
    let mut views = vec![];
    let mut nodes = vec![];
    for (i, node) in graph.iter().enumerate() {
        let st = resolve_style(&node.styles);
        let base = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let name_size = st.font_size.unwrap_or(size);
        let row_size = st.font_size.unwrap_or(small);
        let key_size = st.font_size.unwrap_or(js::round(size * 0.75));
        let label = layout_label(
            Some(if !node.group && !node.alias.is_empty() { &node.alias } else { &node.label }),
            true,
            &m,
            name_size,
            if node.group { 4000.0 } else { 240.0 },
            base,
        );
        let own = loops.get(&i).map_or(&[][..], |v| v.as_slice());
        let below = !own.is_empty() && !node.group && horizontal(i);
        let mut v = View {
            node: node.clone(),
            style: st,
            label,
            w: 0.0,
            h: 0.0,
            head_h: 0.0,
            rows: vec![],
            cols: vec![],
            row_size,
            key_size,
            below,
        };
        let mut reach = 0.0;
        let mut bands = 0.0;
        for &e in own {
            let l = &edge_labels[e];
            let band = js::max(38.0, if below { l.width + 12.0 } else { l.height + 6.0 });
            loop_of[e].as_mut().unwrap().band = band;
            bands += band;
            reach = js::max(reach, if below { l.height } else { l.width });
        }
        let min_w = js::max(96.0, if below { bands + 8.0 } else { 0.0 });
        let mut c = if node.group {
            let mut c = cnode(0.0, 0.0, -1);
            c.is_group = true;
            c.dir = node.dir.as_deref().filter(|s| !s.is_empty()).map(|d| direction(Some(d)));
            c.pad_x = 20.0;
            c.pad_top = if v.label.height > 0.0 { v.label.height + 16.0 } else { 20.0 };
            c.pad_bottom = 20.0;
            c.min_w = js::max(v.label.width + 40.0 - 12.0, 96.0);
            c
        } else {
            let name_h = js::max(v.label.height, js::round(name_size * 1.5));
            if node.attrs.is_empty() {
                v.w = js::max(v.label.width + 36.0, min_w);
                v.h = js::max(name_h + 24.0, if below { 0.0 } else { bands + 8.0 });
            } else {
                let row_line = js::round(row_size * 1.5);
                let mut widths = [0.0; 4];
                for attr in &node.attrs {
                    let cells = [
                        layout_label(Some(&cell(&attr.kind)), true, &m, row_size, 1e6, base),
                        layout_label(Some(&cell(&attr.name)), true, &m, row_size, 1e6, base),
                        layout_label(Some(&attr.keys.join(", ")), false, &m, key_size, 1e6, base),
                        layout_label(Some(&cell(&attr.comment)), true, &m, row_size, 260.0, base),
                    ];
                    let mut h = row_line;
                    for k in 0..4 {
                        widths[k] = js::max(widths[k], cells[k].width);
                        h = js::max(h, cells[k].height);
                    }
                    v.rows.push(Row { cells, h: h + 9.0 });
                    v.h += h + 9.0;
                }
                let mut columns = 0;
                let mut inner = 0.0;
                for w in widths {
                    if w == 0.0 {
                        continue;
                    }
                    inner += w + if columns > 0 { 16.0 } else { 0.0 };
                    columns += 1;
                }
                v.w = js::max(js::max(inner + 24.0, v.label.width + 36.0), min_w);
                let spare = if columns > 0 { (v.w - inner - 24.0) / columns as f64 } else { 0.0 };
                let mut x = 12.0;
                for w in widths {
                    v.cols.push(x);
                    if w > 0.0 {
                        x += w + spare + 16.0;
                    }
                }
                v.head_h = name_h + 18.0;
                v.h += v.head_h;
            }
            let room = if own.is_empty() { 0.0 } else { 2.0 * (18.0 + 28.0 * 0.75 + 10.0 + reach) };
            if below {
                cnode(v.w, v.h + room, -1)
            } else {
                cnode(v.w + room, v.h, -1)
            }
        };
        c.seq = i as f64;
        c.parent = node.parent.as_ref().and_then(|id| index.get(id)).map_or(-1, |&j| j as isize);
        views.push(v);
        nodes.push(c);
    }
    let opt = LayeredOptions {
        node_sep: option(config, "nodeSpacing", 48.0),
        edge_sep: 20.0,
        rank_sep: option(config, "rankSpacing", 64.0),
        port_sep: 24.0,
        tracks: false,
        straight: false,
        apart: false,
        head_room: None,
    };
    let layout = compound_layout(&mut nodes, &mut edges, if turned { Dir::TB } else { direction(Some(&db.direction)) }, &opt);
    spread(&mut edges, &nodes, &views);
    let mut layout_width = layout.width;
    for (&i, own) in &loops {
        let c = &nodes[i];
        if c.is_group {
            let reach = own.iter().fold(0.0, |v, &e| js::max(v, edge_labels[e].width));
            layout_width = js::max(layout_width, c.x + c.w / 2.0 + 18.0 + 28.0 * 0.75 + 10.0 + reach);
        }
    }
    let title = layout_label(db.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(layout_width, title.width);
    shift_layout(&mut nodes, &mut edges, pad + (inner - layout_width) / 2.0, pad + title_h);
    let width = (inner + 2.0 * pad).ceil();
    let height = (layout.height + title_h + 2.0 * pad).ceil();
    let mut links = vec![];
    let mut crossings = Crossings::new();
    if nodes.iter().any(|c| c.is_group) {
        for e in &edges {
            mark_crossings(&mut crossings, &e.route);
        }
    }
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    let box_attrs =
        svg_format!(" fill=\"", ctx.paint(Role::NodeFill, "var(--_s)"), "\" stroke=\"", ctx.paint(Role::NodeStroke, "var(--_b)"), "\"");
    for (i, v) in views.iter_mut().enumerate() {
        let c = &nodes[i];
        let node = &v.node;
        let classes = class_names(node.classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&node.classes));
        if c.is_group {
            v.w = c.w;
            v.h = c.h;
            let x = c.x - c.w / 2.0;
            let y = c.y - c.h / 2.0;
            let tx = cluster_title_x(c.x, c.y, c.w, c.h, v.label.width, &crossings, 20.0);
            svg_write!(
                clusters,
                "<g class=\"pele-cluster",
                classes,
                "\" data-id=\"",
                esc_text(&node.id),
                "\"><rect x=\"",
                num(x),
                "\" y=\"",
                num(y),
                "\" width=\"",
                num(c.w),
                "\" height=\"",
                num(c.h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                ctx.paint(Role::GroupFill, "var(--_a)"),
                "\"",
                if ctx.resolved() { "" } else { " fill-opacity=\"0.5\"" },
                " stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"",
                v.style.shape,
                "/>",
                label_svg(
                    &v.label,
                    tx,
                    y + 8.0 + v.label.height / 2.0,
                    &svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", v.style.text),
                    icons
                ),
                "</g>"
            )
            .unwrap();
            continue;
        }
        let x0 = -v.w / 2.0;
        let y0 = -v.h / 2.0;
        let text = &v.style.text;
        let shape =
            svg_format!("x=\"", num(x0), "\" y=\"", num(y0), "\" width=\"", num(v.w), "\" height=\"", num(v.h), "\" rx=\"", RADIUS, "\"");
        let mut name = svg_format!(label_svg(
            &v.label,
            0.0,
            if v.rows.is_empty() { 0.0 } else { y0 + v.head_h / 2.0 },
            &joined(" class=\"pele-label\"", text),
            icons,
        ));
        let display = decode_entities(if node.alias.is_empty() { &node.label } else { &node.alias });
        if classes.split_whitespace().any(|c| c == "internal-link") && !name.is_empty() && ctx.options.links != Some(false) {
            let safe = link_url(&sanitize_url(&display), ctx.options).is_some();
            let target = esc(&display);
            name = svg_format!(
                "<a class=\"internal-link\"",
                if safe { svg_format!(" href=\"", target, "\"", rel_attr(ctx.options)) } else { String::new() },
                " data-href=\"",
                target,
                "\">",
                name,
                "</a>"
            );
            links.push(LinkInfo { id: node.label.clone(), href: display, internal: true });
        }
        svg_write!(
            nodes_out,
            "<g class=\"pele-node pele-entity",
            classes,
            "\" data-id=\"",
            esc_text(&node.label),
            "\" transform=\"translate(",
            num(c.x),
            ",",
            num(c.y),
            ")\">"
        )
        .unwrap();
        if v.rows.is_empty() {
            svg_write!(nodes_out, "<rect ", shape, box_attrs, v.style.shape, "/>", name).unwrap();
        } else {
            let row_line = js::round(v.row_size * 1.5);
            let line_color =
                v.style.stroke.as_ref().map_or_else(|| ctx.paint(Role::InnerStroke, "var(--_b)").to_string(), |s| esc(s).into_owned());
            // The attributes of each column's text are the same on every row.
            let secondary = ctx.paint(Role::SecondaryText, "var(--_m)");
            let type_attrs = svg_format!(" class=\"pele-er-type\" fill=\"", secondary, "\"", text);
            let name_attrs = joined(" class=\"pele-er-name\"", text);
            let key_attrs =
                svg_format!(" class=\"pele-er-keys\" font-size=\"", js::to_string(v.key_size), "\" fill=\"", secondary, "\"", text);
            let comment_attrs = svg_format!(" class=\"pele-er-comment\" fill=\"", secondary, "\"", text);
            let mut stripes = String::new();
            let mut cells = String::new();
            let mut y = y0 + v.head_h;
            for (r, row) in v.rows.iter().enumerate() {
                if r % 2 == 0 {
                    let span = svg_format!("x=\"", num(x0), "\" width=\"", num(v.w), "\"");
                    if r == v.rows.len() - 1 {
                        svg_write!(
                            stripes,
                            "<rect ",
                            span,
                            " y=\"",
                            num(y),
                            "\" height=\"",
                            num(row.h),
                            "\" rx=\"",
                            RADIUS,
                            "\"/><rect ",
                            span,
                            " y=\"",
                            num(y),
                            "\" height=\"",
                            num(row.h / 2.0),
                            "\"/>"
                        )
                        .unwrap();
                    } else {
                        svg_write!(stripes, "<rect ", span, " y=\"", num(y), "\" height=\"", num(row.h), "\"/>").unwrap();
                    }
                }
                let baseline = y + 9.0 / 2.0 + row_line / 2.0 + v.row_size * 0.35;
                push_text_left(&mut cells, &row.cells[0], x0 + v.cols[0], baseline, &type_attrs, icons);
                push_text_left(&mut cells, &row.cells[1], x0 + v.cols[1], baseline, &name_attrs, icons);
                push_text_left(&mut cells, &row.cells[2], x0 + v.cols[2], baseline, &key_attrs, icons);
                push_text_left(&mut cells, &row.cells[3], x0 + v.cols[3], baseline, &comment_attrs, icons);
                y += row.h;
            }
            svg_write!(
                nodes_out,
                "<rect ",
                shape,
                box_attrs,
                v.style.shape,
                "/><g class=\"pele-er-stripes\" fill=\"",
                ctx.paint(Role::GroupHeader, "var(--_bg)"),
                "\"",
                if v.style.shape.contains("fill:") { " opacity=\"0.6\"" } else { "" },
                ">",
                stripes,
                "</g><path class=\"pele-er-divider\" d=\"M",
                num(x0),
                ",",
                num(y0 + v.head_h),
                "H",
                num(x0 + v.w),
                "\" stroke=\"",
                line_color,
                "\"/><rect ",
                shape,
                " fill=\"none\" stroke=\"",
                ctx.paint(Role::NodeStroke, "var(--_b)"),
                "\"",
                v.style.line,
                "/>",
                name,
                "<g class=\"pele-er-attributes\" font-size=\"",
                js::to_string(v.row_size),
                "\">",
                cells,
                "</g>"
            )
            .unwrap();
        }
        nodes_out.push_str("</g>");
    }
    for (&i, own) in &loops {
        let v = &views[i];
        let total = own.iter().map(|&e| loop_of[e].as_ref().unwrap().band).sum::<f64>();
        let scale = js::min(1.0, js::max(if v.below { v.w } else { v.h } - 8.0, 0.0) / total);
        let mut at = -total * scale / 2.0;
        for &e in own {
            let l = loop_of[e].as_mut().unwrap();
            l.band *= scale;
            l.at = at + l.band / 2.0;
            at += l.band;
        }
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let bg = ctx.paint(Role::Background, "var(--_bg)");
    for (i, e) in edges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &edge_labels[i];
        let (d, start, end) = if let Some(l) = &loop_of[i] {
            let c = &nodes[e.src];
            let v = &views[e.src];
            let spread = js::max(js::min(l.band / 2.0 - 6.0, 13.0), 1.0);
            let far = 18.0 + 28.0 * 0.75 + 6.0;
            if v.below {
                let x = c.x + l.at;
                let y = c.y + v.h / 2.0;
                let turn = num(y + 18.0);
                let out = num(y + 46.0);
                e.label_x = x;
                e.label_y = y + far + label.height / 2.0;
                (
                    svg_format!(
                        "M",
                        num(x - spread),
                        ",",
                        num(y),
                        "V",
                        turn,
                        "C",
                        num(x - spread),
                        ",",
                        out,
                        " ",
                        num(x + spread),
                        ",",
                        out,
                        " ",
                        num(x + spread),
                        ",",
                        turn,
                        "V",
                        num(y)
                    ),
                    marker(&edge.tail, x - spread, y, 0.0, -1.0, bg),
                    marker(&edge.head, x + spread, y, 0.0, -1.0, bg),
                )
            } else {
                let x = c.x + v.w / 2.0;
                let y = c.y + l.at;
                let turn = num(x + 18.0);
                let out = num(x + 46.0);
                e.label_x = x + far + label.width / 2.0;
                e.label_y = y;
                (
                    svg_format!(
                        "M",
                        num(x),
                        ",",
                        num(y - spread),
                        "H",
                        turn,
                        "C",
                        out,
                        ",",
                        num(y - spread),
                        " ",
                        out,
                        ",",
                        num(y + spread),
                        " ",
                        turn,
                        ",",
                        num(y + spread),
                        "H",
                        num(x)
                    ),
                    marker(&edge.tail, x, y - spread, -1.0, 0.0, bg),
                    marker(&edge.head, x, y + spread, -1.0, 0.0, bg),
                )
            }
        } else {
            let mut route = straighten(&e.route);
            if route.len() < 6 {
                continue;
            }
            let last = route.len() - 3;
            inset(&mut route, 0, &views[e.src], &nodes[e.src]);
            inset(&mut route, last, &views[e.dst], &nodes[e.dst]);
            let dx = (route[last] - route[0]).abs();
            let dy = (route[last + 1] - route[1]).abs();
            let direct = route.len() == 6 && (dx < 0.01 || dy < 0.01 || math::hypot(dx, dy) <= 54.0);
            let stub = if direct { 0.0 } else { 18.0 };
            let path = route_path(&route, None, stub, stub, default_reach(stub), default_reach(stub));
            let d = if stub > 0.0 {
                svg_format!("M", num(path.sx), ",", num(path.sy), "L", &path.d[1..], "L", num(path.ex), ",", num(path.ey))
            } else {
                path.d
            };
            (d, marker(&edge.tail, path.sx, path.sy, path.sdx, path.sdy, bg), marker(&edge.head, path.ex, path.ey, path.edx, path.edy, bg))
        };
        svg_write!(
            edges_out,
            "<g class=\"pele-edge pele-er-",
            if edge.dashed { "non-identifying" } else { "identifying" },
            "\" data-id=\"",
            esc_text(&edge.id),
            "\"><path d=\"",
            d,
            "\"",
            if edge.dashed { " stroke-dasharray=\"4 3\"" } else { "" },
            "/>",
            start,
            end,
            "</g>"
        )
        .unwrap();
        if label.width > 0.0 {
            svg_write!(labels_out, edge_label_svg(&edge.id, label, e.label_x, e.label_y, "", icons, bg)).unwrap();
        }
    }
    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
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
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", js::to_string(small), "\">", labels_out, "</g>").unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    Ok(Rendered {
        svg: svg_document(
            "er",
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
fn straighten(route: &[f64]) -> Vec<f64> {
    if route.len() < 3 {
        return route.to_vec();
    }
    let mut out = route[..3].to_vec();
    for k in (3..route.len()).step_by(3) {
        let prev = out.len() - 3;
        let flat = |axis: usize| (out[prev + axis] - route[k + axis]).abs() < 0.01 && (route[k + axis] - route[k + 3 + axis]).abs() < 0.01;
        if k + 3 < route.len() && (flat(0) || flat(1)) {
            continue;
        }
        out.extend_from_slice(&route[k..k + 3]);
    }
    out
}
fn spread(edges: &mut [CEdge], nodes: &[CNode], views: &[View]) {
    let mut sides = IndexMap::<usize, Vec<(usize, usize, f64)>>::new();
    for (i, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let last = e.route.len() - 3;
        for (node, at, next) in [(e.src, 0, 3), (e.dst, last, last - 3)] {
            let c = &nodes[node];
            let dx = e.route[at] - c.x;
            let dy = e.route[at + 1] - c.y;
            let side = if (dy.abs() - c.h / 2.0).abs() < 0.5 {
                if dy < 0.0 {
                    0
                } else {
                    2
                }
            } else if (dx.abs() - c.w / 2.0).abs() < 0.5 {
                if dx < 0.0 {
                    3
                } else {
                    1
                }
            } else {
                continue;
            };
            sides.entry(node * 4 + side).or_default().push((i, at, e.route[next + (side & 1)]));
        }
    }
    for (key, mut list) in sides {
        if list.len() < 2 {
            continue;
        }
        let i = key >> 2;
        let vertical = key & 1 == 1;
        let (w, h) = if nodes[i].is_group { (nodes[i].w, nodes[i].h) } else { (views[i].w, views[i].h) };
        let length = if vertical { h } else { w };
        let step = js::max(js::min(18.0, (length - 28.0) / (list.len() - 1) as f64), 0.0);
        let center = if vertical { nodes[i].y } else { nodes[i].x };
        list.sort_by(|a, b| a.2.total_cmp(&b.2));
        for (j, &(e, at, _)) in list.iter().enumerate() {
            edges[e].route[at + usize::from(vertical)] = center + (j as f64 - (list.len() - 1) as f64 / 2.0) * step;
        }
    }
}
fn inset(route: &mut [f64], at: usize, v: &View, c: &CNode) {
    if c.is_group {
        return;
    }
    let dx = route[at] - c.x;
    let dy = route[at + 1] - c.y;
    if c.w != v.w && (dx.abs() - c.w / 2.0).abs() < 0.5 {
        route[at] -= js::sign(dx) * (c.w - v.w) / 2.0;
    }
    if c.h != v.h && (dy.abs() - c.h / 2.0).abs() < 0.5 {
        route[at + 1] -= js::sign(dy) * (c.h - v.h) / 2.0;
    }
}
