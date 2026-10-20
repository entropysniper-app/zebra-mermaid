//! Use-case actors, ovals, JSON tables, boundaries and relationship routing.
use super::{
    graph::{build, Node},
    types::UsecaseModel,
};
use crate::diagrams::{
    common::fit_width::{runs_across, turn_to_fit},
    RenderCtx, Rendered,
};
use crate::layout::{
    compound::{cnode, compound_layout, direction, shift_layout, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg},
    cluster::{mark_crossings, Crossings},
    edges::{default_reach, marker, marker_trim, render_edge_label_svg as edge_label_svg, route_path, EdgePath},
    root::{svg_document, Accessible},
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
};
use crate::text::{layout_render_label as layout_label, style, Icons, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math};
use crate::{Config, ConfigValue, Error};
use indexmap::IndexMap;
use std::fmt::Write as _;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Actor,
    Ellipse,
    Box,
    Json,
    Boundary,
}
struct View<'a> {
    node: &'a Node,
    kind: Kind,
    style: ResolvedStyle,
    label: Label,
    stereotype: Label,
    w: f64,
    h: f64,
    rows: Vec<(Label, Label)>,
    key_w: f64,
    head_h: f64,
    loops: usize,
}
fn option(config: &Config, key: &str, fallback: f64) -> f64 {
    config.lookup("usecase", key).and_then(ConfigValue::as_f64).filter(|v| *v > 0.0).map_or(fallback, |v| js::min(v, 2000.0))
}
fn literal(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if matches!(c, '&' | '<' | '>' | '\\') {
            write!(out, "&#{};", c as u32).unwrap();
        } else {
            out.push(c);
        }
    }
    out
}
fn text(raw: Option<&str>, markdown: bool, m: &Measure, size: f64, wrap: f64, base: u8) -> Label {
    let converted = raw.filter(|s| !s.is_empty() && !markdown).map(literal);
    layout_label(converted.as_deref().or(raw), markdown, m, size, wrap, base)
}
fn anchor(v: &View, c: &CNode, side: usize, along: f64) -> (f64, f64) {
    let sign = if side == 1 || side == 2 { 1.0 } else { -1.0 };
    let upright = side == 1 || side == 3;
    let mut reach = if upright { v.w } else { v.h } / 2.0;
    let mut middle = 0.0;
    if v.kind == Kind::Actor {
        if upright {
            reach = 40.0 / 2.0 + 3.0;
            middle = (60.0 - v.h) / 2.0;
        } else {
            reach += 2.0;
        }
    } else if v.kind == Kind::Ellipse {
        let span = if upright { v.h } else { v.w } / 2.0;
        let ratio = along / span;
        reach *= js::max(1.0 - ratio * ratio, 0.0).sqrt();
    }
    if upright {
        (c.x + sign * reach, c.y + middle + along)
    } else {
        (c.x + along, c.y + sign * reach)
    }
}
fn side_length(v: &View, side: usize) -> f64 {
    let upright = side == 1 || side == 3;
    if v.kind == Kind::Actor {
        return if upright { 60.0 * 0.5 } else { 40.0 * 0.6 };
    }
    let length = if upright { v.h } else { v.w };
    if v.kind == Kind::Ellipse {
        length * 0.6
    } else {
        length - 16.0
    }
}
fn attach_ends(edges: &mut [CEdge], nodes: &[CNode], views: &[View]) {
    let mut sides = IndexMap::<usize, Vec<(usize, usize, f64)>>::new();
    for (i, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let last = e.route.len() - 3;
        for (index, at, next) in [(e.src, 0, 3), (e.dst, last, last - 3)] {
            let c = &nodes[index];
            if c.is_group {
                continue;
            }
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
            sides.entry(index * 4 + side).or_default().push((i, at, e.route[next + (side & 1)]));
        }
    }
    for (key, mut ends) in sides {
        let index = key >> 2;
        let side = key & 3;
        let v = &views[index];
        let step = if ends.len() > 1 { js::min(14.0, side_length(v, side) / (ends.len() - 1) as f64) } else { 0.0 };
        ends.sort_by(|a, b| a.2.total_cmp(&b.2));
        for (i, &(edge, at, _)) in ends.iter().enumerate() {
            let (x, y) = anchor(v, &nodes[index], side, (i as f64 - (ends.len() - 1) as f64 / 2.0) * step);
            edges[edge].route[at] = x;
            edges[edge].route[at + 1] = y;
        }
    }
}
fn trim(kind: &str) -> f64 {
    if kind == "extension" {
        12.0
    } else {
        marker_trim(kind)
    }
}
fn mark(kind: &str, x: f64, y: f64, dx: f64, dy: f64, color: &str, bg: &str) -> String {
    if kind != "extension" {
        return svg_format!(marker(kind, x, y, dx, dy, color));
    }
    let bx = x - dx * 12.0;
    let by = y - dy * 12.0;
    let half = 12.0 * 0.5;
    svg_format!(
        "<path class=\"pele-marker\" d=\"M",
        num(x),
        ",",
        num(y),
        "L",
        num(bx - dy * half),
        ",",
        num(by + dx * half),
        "L",
        num(bx + dy * half),
        ",",
        num(by - dx * half),
        "Z\" fill=\"",
        bg,
        "\" stroke=\"",
        color,
        "\" stroke-linejoin=\"round\"/>"
    )
}
fn figure(v: &View, ctx: &RenderCtx, surface: &str) -> String {
    let n = v.node;
    let shape = &v.style.shape;
    let border = ctx.paint(Role::NodeStroke, "var(--_b)");
    let bg = ctx.paint(Role::Background, "var(--_bg)");
    let line = svg_format!(" fill=\"none\" stroke=\"", border, "\"", v.style.line);
    let mut head = -21.0;
    let mut radius = 9.0;
    let mut out=match n.actor_type.as_deref(){Some("hollow")=>{radius=8.0;svg_format!("<circle cy=\"-21\" r=\"8\" fill=\"", bg, "\" stroke=\"", border, "\"", shape, "/><path d=\"M-20,-9H20V0H5.5L20,15.5L12,25.5L0,12L-12,25.5L-20,15.5L-5.5,0H-20Z\" fill=\"", bg, "\" stroke=\"", border, "\" stroke-linejoin=\"round\"", shape, "/>")},Some("awesome")=>svg_format!("<path d=\"M0,-30A11,11 0 1 1 0,-8A11,11 0 1 1 0,-30ZM-20,25C-20,9 -12,-3 0,-3C12,-3 20,9 20,25C20,28 18,30 15,30H-15C-18,30 -20,28 -20,25Z\" fill=\"", border, "\"", shape, "/>"),Some("icon")=>{head = -14.0;radius=12.0;let icon=n.icon.as_deref().unwrap_or("");svg_format!("<rect x=\"-22\" y=\"-26\" width=\"44\" height=\"44\" rx=\"", RADIUS, "\"", surface, shape, "/><svg class=\"pele-icon\" data-icon=\"", esc_text(icon), "\" x=\"-14\" y=\"-18\" width=\"28\" height=\"28\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">", ctx.icons().map_or_else(String::new,|i|i.markup(icon).to_string()), "</svg>")},_=>svg_format!("<circle cy=\"-21\" r=\"9\"", surface, shape, "/><path d=\"M0,-12V10M-14,-3H14M0,10L-12,30M0,10L12,30\"", line, "/>")};
    if n.business {
        svg_write!(
            out,
            "<path class=\"pele-business\" d=\"M",
            num(0.12 * radius),
            ",",
            num(head + 0.99 * radius),
            "L",
            num(0.92 * radius),
            ",",
            num(head - 0.39 * radius),
            "\"",
            line,
            "/>"
        )
        .unwrap();
    }
    out
}
fn title_x(c: &CNode, width: f64, crossings: &Crossings) -> f64 {
    let Some(xs) = crossings.get(&(js::round(c.y - c.h / 2.0) as i64)) else {
        return c.x;
    };
    if width == 0.0 {
        return c.x;
    }
    let left = c.x - c.w / 2.0 + 6.0;
    let right = c.x + c.w / 2.0 - 6.0;
    let mut cuts: Vec<_> = xs.iter().copied().filter(|&x| x > left && x < right).collect();
    cuts.sort_by(f64::total_cmp);
    cuts.push(right);
    let need = width / 2.0 + 6.0;
    let mut best = c.x;
    let mut best_off = f64::INFINITY;
    let mut widest = 0.0;
    let mut from = left;
    for cut in cuts {
        if cut - from >= 2.0 * need {
            let x = js::min(js::max(c.x, from + need), cut - need);
            if (x - c.x).abs() < best_off {
                best_off = (x - c.x).abs();
                best = x;
            }
        } else if best_off == f64::INFINITY && cut - from > widest {
            widest = cut - from;
            best = (from + cut) / 2.0;
        }
        from = cut;
    }
    best
}
fn left_label(label: &Label, x: f64, cy: f64, attrs: &str, icons: Option<&Icons>) -> String {
    if label.lines().len() < 2 {
        return svg_format!(label_svg(label, x + label.width / 2.0, cy, attrs, icons));
    }
    let mut out = String::new();
    for i in 0..label.lines().len() {
        let line = Label::from_lines(
            vec![label.lines()[i].to_vec()],
            vec![label.widths()[i]],
            label.width,
            label.line_height,
            label.size,
            label.line_height,
        );
        svg_write!(
            out,
            label_svg(&line, x + label.widths()[i] / 2.0, cy - label.height / 2.0 + (i as f64 + 0.5) * label.line_height, attrs, icons,)
        )
        .unwrap();
    }
    out
}
pub(crate) fn render(model: &UsecaseModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if runs_across(Some(&model.direction)) {
        turn_to_fit(ctx.options, |down| draw(model, config, ctx, down))
    } else {
        draw(model, config, ctx, false)
    }
}
fn draw(model: &UsecaseModel, config: &Config, ctx: &RenderCtx, turned: bool) -> Result<Rendered, Error> {
    let (graph, graph_edges) = build(model);
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.8125);
    let m = ctx.measure();
    let pad = ctx.padding();
    let icons = ctx.icons();
    let root_dir = if turned { Dir::TB } else { direction(Some(&model.direction)) };
    let sideways = matches!(root_dir, Dir::LR | Dir::RL);
    let index: IndexMap<_, _> = graph.iter().enumerate().map(|(i, n)| (n.id.clone(), i)).collect();
    let mut views = vec![];
    let mut nodes = vec![];
    for node in &graph {
        let mut styles = node.compiled.clone();
        styles.extend(node.styles.iter().cloned());
        let st = resolve_style(&styles);
        let base = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let fs = st.font_size.unwrap_or(size);
        let markdown = node.label_type == "markdown";
        let raw_stereo = node.stereotype.as_ref().filter(|s| !s.is_empty()).map(|s| format!("«{s}»"));
        let stereotype = text(raw_stereo.as_deref(), false, &m, tiny, 200.0, base);
        let mut v = View {
            node,
            kind: Kind::Box,
            style: st,
            label: stereotype.clone(),
            stereotype,
            w: 0.0,
            h: 0.0,
            rows: vec![],
            key_w: 0.0,
            head_h: 0.0,
            loops: 0,
        };
        let mut c;
        if node.group {
            v.kind = Kind::Boundary;
            v.label = text(Some(&node.label), markdown, &m, fs, 4000.0, base);
            let th = v.label.height;
            c = cnode(0.0, 0.0, -1);
            c.is_group = true;
            c.pad_x = 20.0;
            c.pad_bottom = 20.0;
            c.pad_top = if node.boundary_type.as_deref() == Some("package") {
                th + 2.0 * 6.0 + 16.0
            } else if th > 0.0 {
                th + 16.0
            } else {
                20.0
            };
            c.min_w = v.label.width + 2.0 * 20.0;
        } else {
            if let Some(rows) = &node.json_rows {
                v.kind = Kind::Json;
                v.label = text(Some(&node.label), false, &m, fs, 4000.0, base | style::BOLD);
                let mut value_w = 0.0;
                for row in rows {
                    let key = text(Some(&row.key), false, &m, small, 4000.0, base);
                    let value = text(Some(&row.value), false, &m, small, 260.0, base);
                    v.key_w = js::max(v.key_w, key.width);
                    value_w = js::max(value_w, value.width);
                    v.h += js::max(value.height, js::round(small * 1.5)) + 8.0;
                    v.rows.push((key, value));
                }
                if v.key_w > 0.0 {
                    v.key_w += 2.0 * 10.0;
                }
                v.head_h = js::max(v.label.height, js::round(fs * 1.5)) + 8.0;
                v.w = js::max(
                    js::max(v.label.width + 2.0 * 10.0, if !v.rows.is_empty() { v.key_w + value_w + 2.0 * 10.0 } else { 0.0 }),
                    72.0,
                );
                v.h += v.head_h;
            } else if node.actor_type.is_some() {
                v.kind = Kind::Actor;
                v.label = text(Some(&node.label), markdown, &m, fs, 140.0, base);
                v.w = js::max(js::max(40.0, v.label.width), v.stereotype.width);
                v.h = 60.0 + 6.0 + v.stereotype.height + v.label.height;
            } else {
                v.label = text(Some(&node.label), markdown, &m, if node.shape == "note" { small } else { fs }, 180.0, base);
                let tw = js::max(v.label.width, v.stereotype.width);
                let th = v.label.height + v.stereotype.height;
                if matches!(node.shape.as_str(), "usecaseEllipse" | "usecaseBusiness") {
                    v.kind = Kind::Ellipse;
                    v.w = js::max(tw + 2.0 * node.padding + if node.business { 16.0 } else { 4.0 }, 88.0);
                    v.h = js::max(th + 28.0, 48.0);
                } else if node.shape == "note" {
                    v.w = tw + 2.0 * node.padding + 6.0;
                    v.h = th + 18.0;
                } else {
                    v.w = js::max(tw + 2.0 * node.padding + 12.0, 80.0);
                    v.h = js::max(th + 24.0, 44.0);
                }
            }
            c = cnode(v.w, v.h, -1);
        }
        c.seq = views.len() as f64;
        views.push(v);
        nodes.push(c);
    }
    for (i, n) in graph.iter().enumerate() {
        nodes[i].parent = n.parent.as_ref().and_then(|id| index.get(id)).map_or(-1, |&i| i as isize);
    }
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    for e in &graph_edges {
        let (Some(&src), Some(&dst)) = (index.get(&e.start), index.get(&e.end)) else {
            continue;
        };
        let semantic = matches!(e.kind.as_str(), "include" | "extend");
        let value = if semantic { Some(format!("«{}»", e.label.as_deref().unwrap_or("undefined"))) } else { e.label.clone() };
        let label = text(value.as_deref(), e.label_type.as_deref() == Some("markdown"), &m, small, 200.0, 0);
        if src == dst {
            views[src].loops += 1;
            if sideways {
                nodes[src].h += 2.0 * (26.0 + if label.height > 0.0 { label.height + 8.0 } else { 0.0 });
            } else {
                nodes[src].w += 2.0 * (26.0 + if label.width > 0.0 { label.width + 12.0 } else { 0.0 });
            }
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: e.minlen,
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
        root_dir,
        &LayeredOptions {
            node_sep: option(config, "nodeSpacing", 32.0),
            edge_sep: 16.0,
            rank_sep: option(config, "rankSpacing", 72.0),
            port_sep: 20.0,
            head_room: None,
            tracks: false,
            straight: false,
            apart: false,
        },
    );
    let title = layout_label(model.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(layout.width, title.width);
    shift_layout(&mut nodes, &mut edges, pad + (inner - layout.width) / 2.0, pad + title_h);
    let width = (inner + 2.0 * pad).ceil();
    let height = (layout.height + title_h + 2.0 * pad).ceil();
    let mut crossings = Crossings::new();
    for e in &edges {
        mark_crossings(&mut crossings, &e.route);
    }
    attach_ends(&mut edges, &nodes, &views);
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    let surface =
        svg_format!(" fill=\"", ctx.paint(Role::NodeFill, "var(--_s)"), "\" stroke=\"", ctx.paint(Role::NodeStroke, "var(--_b)"), "\"");
    let border = ctx.paint(Role::NodeStroke, "var(--_b)");
    let rule = ctx.paint(Role::InnerStroke, "var(--_b)");
    let muted = ctx.paint(Role::SecondaryText, "var(--_m)");
    let bg = ctx.paint(Role::Background, "var(--_bg)");
    for (i, v) in views.iter().enumerate() {
        let c = &nodes[i];
        let n = v.node;
        let classes = class_names(n.classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&n.classes));
        let id = esc_text(&n.id);
        let text_attrs = &v.style.text;
        let shape = &v.style.shape;
        let (w, h) = (v.w, v.h);
        if v.kind == Kind::Boundary {
            let x = c.x - c.w / 2.0;
            let y = c.y - c.h / 2.0;
            let box_attrs = svg_format!(
                " fill=\"",
                ctx.paint(Role::GroupFill, "var(--_a)"),
                "\"",
                if ctx.resolved() { "" } else { " fill-opacity=\"0.5\"" },
                " stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"",
                shape
            );
            let title_attrs = svg_format!(" class=\"pele-cluster-label\" fill=\"", muted, "\"", text_attrs);
            let body = if n.boundary_type.as_deref() == Some("package") {
                let tw = js::min(v.label.width + 2.0 * 6.0 + 8.0, c.w);
                let th = v.label.height + 2.0 * 6.0;
                svg_format!(
                    "<rect class=\"pele-boundary-tab\" x=\"",
                    num(x),
                    "\" y=\"",
                    num(y),
                    "\" width=\"",
                    num(tw),
                    "\" height=\"",
                    num(th),
                    "\"",
                    box_attrs,
                    "/><rect x=\"",
                    num(x),
                    "\" y=\"",
                    num(y + th),
                    "\" width=\"",
                    num(c.w),
                    "\" height=\"",
                    num(c.h - th),
                    "\"",
                    box_attrs,
                    "/>",
                    label_svg(&v.label, x + tw / 2.0, y + th / 2.0, &title_attrs, icons)
                )
            } else {
                svg_format!(
                    "<rect x=\"",
                    num(x),
                    "\" y=\"",
                    num(y),
                    "\" width=\"",
                    num(c.w),
                    "\" height=\"",
                    num(c.h),
                    "\" rx=\"",
                    RADIUS,
                    "\"",
                    box_attrs,
                    "/>",
                    label_svg(&v.label, title_x(c, v.label.width, &crossings), y + 8.0 + v.label.height / 2.0, &title_attrs, icons)
                )
            };
            svg_write!(clusters, "<g class=\"pele-cluster pele-boundary", classes, "\" data-id=\"", id, "\">", body, "</g>").unwrap();
            continue;
        }
        let body;
        let kind;
        if v.kind == Kind::Actor {
            kind = "pele-actor";
            let top = -h / 2.0;
            let sy = top + 60.0 + 6.0 + v.stereotype.height / 2.0;
            body = svg_format!(
                "<g class=\"pele-actor-figure\" transform=\"translate(0,",
                num(top + 60.0 / 2.0),
                ")\">",
                figure(v, ctx, &surface),
                "</g>",
                label_svg(
                    &v.stereotype,
                    0.0,
                    sy,
                    &svg_format!(" class=\"pele-stereotype\" font-size=\"", js::number_text(tiny), "\" fill=\"", muted, "\""),
                    icons
                ),
                label_svg(&v.label, 0.0, h / 2.0 - v.label.height / 2.0, &svg_format!(" class=\"pele-label\"", text_attrs), icons)
            );
        } else if v.kind == Kind::Json {
            kind = "pele-json";
            let x0 = -w / 2.0;
            let mut y = -h / 2.0 + v.head_h;
            let mut cells = String::new();
            let mut lines = svg_format!("M", num(x0), ",", num(y), "H", num(x0 + w));
            if v.key_w > 0.0 {
                svg_write!(lines, "M", num(x0 + v.key_w), ",", num(y), "V", num(h / 2.0)).unwrap();
            }
            for (key, value) in &v.rows {
                let rh = js::max(value.height, js::round(small * 1.5)) + 8.0;
                svg_write!(
                    cells,
                    label_svg(
                        key,
                        x0 + 10.0 + key.width / 2.0,
                        y + rh / 2.0,
                        &svg_format!(" class=\"pele-json-key\" fill=\"", muted, "\""),
                        icons,
                    )
                )
                .unwrap();
                cells.push_str(&left_label(
                    value,
                    x0 + v.key_w + 10.0,
                    y + rh / 2.0,
                    &svg_format!(" class=\"pele-json-value\"", text_attrs),
                    icons,
                ));
                y += rh;
            }
            body = svg_format!(
                "<rect x=\"",
                num(x0),
                "\" y=\"",
                num(-h / 2.0),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(h),
                "\" rx=\"",
                RADIUS,
                "\"",
                surface,
                shape,
                "/><path class=\"pele-json-lines\" d=\"",
                lines,
                "\" fill=\"none\" stroke=\"",
                rule,
                "\"",
                v.style.line,
                "/>",
                label_svg(
                    &v.label,
                    0.0,
                    -h / 2.0 + v.head_h / 2.0,
                    &svg_format!(" class=\"pele-label\" font-weight=\"var(--_hw)\"", text_attrs),
                    icons
                ),
                "<g font-size=\"",
                js::number_text(small),
                "\">",
                cells,
                "</g>"
            );
        } else {
            let label_y = v.stereotype.height / 2.0;
            let mut outline;
            if v.kind == Kind::Ellipse {
                kind = "pele-usecase";
                outline = svg_format!("<ellipse rx=\"", num(w / 2.0), "\" ry=\"", num(h / 2.0), "\"", surface, shape, "/>");
                if n.business {
                    let from = w / 2.0 - 15.0;
                    let to = w / 2.0 - 5.0;
                    let arc = |at: f64| {
                        let q = at / (w / 2.0);
                        h / 2.0 * (1.0 - q * q).sqrt()
                    };
                    svg_write!(
                        outline,
                        "<path class=\"pele-business\" d=\"M",
                        num(from),
                        ",",
                        num(arc(from)),
                        "L",
                        num(to),
                        ",",
                        num(-arc(to)),
                        "\" fill=\"none\" stroke=\"",
                        border,
                        "\"",
                        v.style.line,
                        "/>"
                    )
                    .unwrap();
                }
            } else if n.shape == "note" {
                kind = "pele-note";
                let (r, b) = (w / 2.0, h / 2.0);
                outline = svg_format!(
                    "<path d=\"M",
                    num(-r),
                    ",",
                    num(-b),
                    "H",
                    num(r - 9.0),
                    "L",
                    num(r),
                    ",",
                    num(9.0 - b),
                    "V",
                    num(b),
                    "H",
                    num(-r),
                    "Z\" fill=\"",
                    ctx.paint(Role::WarningBackground, "var(--_a)"),
                    "\" stroke=\"",
                    border,
                    "\" stroke-linejoin=\"round\"",
                    shape,
                    "/><path d=\"M",
                    num(r - 9.0),
                    ",",
                    num(-b),
                    "V",
                    num(9.0 - b),
                    "H",
                    num(r),
                    "\" fill=\"none\" stroke=\"",
                    border,
                    "\" stroke-linejoin=\"round\"",
                    v.style.line,
                    "/>"
                );
            } else {
                kind = "pele-usecase";
                outline = svg_format!(
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
                    surface,
                    shape,
                    "/>"
                );
            }
            body = format!(
                "{outline}{}{}",
                label_svg(
                    &v.stereotype,
                    0.0,
                    -v.label.height / 2.0,
                    &svg_format!(" class=\"pele-stereotype\" font-size=\"", js::number_text(tiny), "\" fill=\"", muted, "\""),
                    icons
                ),
                label_svg(
                    &v.label,
                    0.0,
                    label_y,
                    &svg_format!(
                        " class=\"pele-label\"",
                        if n.shape == "note" { svg_format!(" font-size=\"", js::number_text(small), "\"") } else { String::new() },
                        text_attrs
                    ),
                    icons
                )
            );
        }
        svg_write!(
            nodes_out,
            "<g class=\"pele-node ",
            kind,
            classes,
            "\" data-id=\"",
            id,
            "\" transform=\"translate(",
            num(c.x),
            ",",
            num(c.y),
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
        let st = if !e.styles.is_empty() || !e.compiled.is_empty() {
            let mut all = e.compiled.clone();
            all.extend(e.styles.iter().cloned());
            Some(resolve_style(&all))
        } else {
            None
        };
        let path;
        if ce.src == ce.dst {
            let c = &nodes[ce.src];
            let v = &views[ce.src];
            let k = *loop_seen.get(&ce.src).unwrap_or(&0);
            loop_seen.insert(ce.src, k + 1);
            let side = if sideways { 2 } else { 1 };
            let spread = js::min(side_length(v, side) / 2.0, 8.0 + k as f64 * 6.0);
            let reach = 26.0 + k as f64 * 8.0;
            let (ax, ay) = anchor(v, c, side, -spread);
            let (bx, by) = anchor(v, c, side, spread);
            let len = math::hypot(reach, spread);
            let len = if len == 0.0 { 1.0 } else { len };
            let ux = if sideways { spread } else { reach } / len;
            let uy = if sideways { reach } else { spread } / len;
            let (st, et) = (trim(&e.tail), trim(&e.head));
            let out = |x: f64, y: f64, sign: f64, far: f64| {
                if sideways {
                    format!("{},{}", num(x + sign * spread * if far != 0.0 { 1.0 } else { 0.0 }), num(y + far))
                } else {
                    format!("{},{}", num(x + far), num(y + sign * spread * if far != 0.0 { 1.0 } else { 0.0 }))
                }
            };
            let sx = if sideways { ax - ux * st } else { ax + ux * st };
            let sy = if sideways { ay + uy * st } else { ay - uy * st };
            let ex = bx + ux * et;
            let ey = by + uy * et;
            path = EdgePath {
                d: svg_format!(
                    "M",
                    num(sx),
                    ",",
                    num(sy),
                    "C",
                    out(ax, ay, -1.0, reach),
                    " ",
                    out(bx, by, 1.0, reach),
                    " ",
                    num(ex),
                    ",",
                    num(ey)
                ),
                sx: ax,
                sy: ay,
                sdx: if sideways { ux } else { -ux },
                sdy: if sideways { -uy } else { uy },
                ex: bx,
                ey: by,
                edx: -ux,
                edy: -uy,
            };
            if sideways {
                ce.label_x = c.x;
                ce.label_y = js::max(ay, by) + reach * 0.75 + 6.0 + label.height / 2.0;
            } else {
                ce.label_x = js::max(ax, bx) + reach * 0.75 + 8.0 + label.width / 2.0;
                ce.label_y = c.y;
            }
        } else {
            if ce.route.len() < 6 {
                continue;
            }
            let (st, et) = (trim(&e.tail), trim(&e.head));
            path = route_path(&ce.route, None, st, et, default_reach(st), default_reach(et));
        }
        let color = st
            .as_ref()
            .and_then(|s| s.stroke.as_ref())
            .filter(|s| !s.is_empty())
            .map_or_else(|| ctx.paint(Role::Arrow, "var(--_l)").to_string(), |s| esc(s).into_owned());
        let dashes = if e.internal {
            "2 3"
        } else if e.dotted {
            "5 4"
        } else if e.animate {
            "9 5"
        } else {
            ""
        };
        let clean = e
            .classes
            .strip_prefix("default relationship relationship-")
            .map(|s| s.split_once(' ').map_or("", |(_, rest)| rest))
            .unwrap_or(&e.classes);
        svg_write!(
            edges_out,
            "<g class=\"pele-edge pele-",
            e.kind,
            class_names(clean),
            "\" data-id=\"",
            esc_text(&e.id),
            "\"",
            st.as_ref().map_or("", |s| s.line.as_str()),
            "><path d=\"",
            path.d,
            "\"",
            if dashes.is_empty() { String::new() } else { svg_format!(" stroke-dasharray=\"", dashes, "\"") },
            "/>",
            mark(&e.tail, path.sx, path.sy, path.sdx, path.sdy, &color, bg),
            mark(&e.head, path.ex, path.ey, path.edx, path.edy, &color, bg),
            "</g>"
        )
        .unwrap();
        if label.width > 0.0 {
            let semantic = matches!(e.kind.as_str(), "include" | "extend");
            let attrs = format!(
                "{}{}",
                if semantic { svg_format!(" fill=\"", muted, "\"") } else { String::new() },
                st.as_ref().map_or("", |s| s.text.as_str())
            );
            svg_write!(labels_out, edge_label_svg(&e.id, label, ce.label_x, ce.label_y, &attrs, icons, bg)).unwrap();
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
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", js::number_text(small), "\">", labels_out, "</g>").unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    Ok(Rendered {
        svg: svg_document(
            "usecase",
            width,
            height,
            size,
            ctx,
            Accessible { acc_title: Some(&model.acc_title), acc_descr: Some(&model.acc_descr) },
            &content,
        ),
        width,
        height,
        links: vec![],
    })
}
