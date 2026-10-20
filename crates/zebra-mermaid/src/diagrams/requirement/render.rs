//! Requirement tables and relation markers using Pele's compound graph layout.
use super::{RequirementModel, RequirementNode};
use crate::diagrams::common::fit_width::{runs_across, turn_to_fit};
use crate::diagrams::{RenderCtx, Rendered};
use crate::layout::{
    compound::{cnode, compound_layout, direction, CEdge, CNode, Dir},
    layered::LayeredOptions,
};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg, render_shared_style as shared_style, span_style, text_style},
    edges::{default_reach, render_edge_label_svg as edge_label_svg, route_path, EdgePath},
    root::{svg_document, Accessible},
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
};
use crate::text::{layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::js;
use crate::{Config, Error};
use indexmap::IndexMap;
struct Row {
    key: Label,
    value: Label,
    height: f64,
}
struct View<'a> {
    node: &'a RequirementNode,
    requirement: bool,
    style: ResolvedStyle,
    kind: Label,
    name: Label,
    rows: Vec<Row>,
    key_width: f64,
    head_height: f64,
    w: f64,
    h: f64,
}
fn restyle(mut label: Label, bits: u8, m: &Measure) -> Label {
    label.restyle(bits, m);
    label
}
fn start_text(label: &Label, x: f64, top: f64, attrs: &str) -> String {
    let mut y = top + label.line_height / 2.0 + label.size * 0.35;
    let shared = shared_style(label);
    let mut body = String::new();
    for line in &label.lines() {
        let mut first = true;
        for span in line {
            if span.icon.is_some() || span.text.is_empty() {
                continue;
            }
            svg_write!(
                body,
                "<tspan",
                if first { svg_format!(" x=\"", num(x), "\" y=\"", num(y), "\"") } else { String::new() },
                span_style(span.style & !shared),
                ">",
                esc(&span.text),
                "</tspan>"
            )
            .unwrap();
            first = false;
        }
        y += label.line_height;
    }
    if body.is_empty() {
        body
    } else {
        svg_format!("<text", text_style(attrs, shared), " text-anchor=\"start\" xml:space=\"preserve\">", body, "</text>")
    }
}
fn contains_marker(x: f64, y: f64, dx: f64, dy: f64) -> String {
    let cx = x - dx * 6.0;
    let cy = y - dy * 6.0;
    svg_format!(
        "<circle class=\"pele-marker\" cx=\"",
        num(cx),
        "\" cy=\"",
        num(cy),
        "\" r=\"6\"/><path class=\"pele-marker\" d=\"M",
        num(cx - dx * 6.0),
        ",",
        num(cy - dy * 6.0),
        "L",
        num(cx + dx * 6.0),
        ",",
        num(cy + dy * 6.0),
        "M",
        num(cx + dy * 6.0),
        ",",
        num(cy - dx * 6.0),
        "L",
        num(cx - dy * 6.0),
        ",",
        num(cy + dx * 6.0),
        "\"/>"
    )
}
fn arrow(x: f64, y: f64, dx: f64, dy: f64) -> String {
    let bx = x - dx * 9.0;
    let by = y - dy * 9.0;
    svg_format!(
        "<path class=\"pele-marker\" d=\"M",
        num(bx - dy * 4.5),
        ",",
        num(by + dx * 4.5),
        "L",
        num(x),
        ",",
        num(y),
        "L",
        num(bx + dy * 4.5),
        ",",
        num(by - dx * 4.5),
        "\" stroke-linejoin=\"round\"/>"
    )
}
fn spread(nodes: &[CNode], edges: &mut [CEdge], views: &[View]) {
    let mut sides = IndexMap::<usize, Vec<(usize, usize, f64)>>::new();
    for (i, edge) in edges.iter().enumerate() {
        let route = &edge.route;
        if edge.src == edge.dst || route.len() < 6 {
            continue;
        }
        for at in [0, route.len() - 3] {
            let node = if at == 0 { edge.src } else { edge.dst };
            let c = &nodes[node];
            let dx = route[at] - c.x;
            let dy = route[at + 1] - c.y;
            let next = if at == 0 { 3 } else { at - 3 };
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
            sides.entry(node * 4 + side).or_default().push((i, at, route[if side % 2 == 0 { next } else { next + 1 }]));
        }
    }
    for (key, mut list) in sides {
        if list.len() < 2 {
            continue;
        }
        let node = key >> 2;
        let vertical = key % 2 == 1;
        let span = if vertical { views[node].h } else { views[node].w } - 28.0;
        let step = js::max(0.0, js::min(28.0, span / (list.len() - 1) as f64));
        list.sort_by(|a, b| a.2.total_cmp(&b.2));
        for (j, &(i, at, _)) in list.iter().enumerate() {
            let offset = (j as f64 - (list.len() - 1) as f64 / 2.0) * step;
            edges[i].route[at + usize::from(vertical)] = if vertical { nodes[node].y } else { nodes[node].x } + offset;
        }
    }
}
pub(crate) fn render(model: &RequirementModel, _config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if runs_across(Some(&model.direction)) {
        turn_to_fit(ctx.options, |down| draw(model, ctx, down))
    } else {
        draw(model, ctx, false)
    }
}
fn draw(model: &RequirementModel, ctx: &RenderCtx, turned: bool) -> Result<Rendered, Error> {
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let mut index = IndexMap::new();
    let mut views = vec![];
    let mut nodes = vec![];
    for (node, is_req) in model.requirements.values().map(|n| (n, true)).chain(model.elements.values().map(|n| (n, false))) {
        let st = resolve_style(&node.styles);
        let base = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let name_size = st.font_size.unwrap_or(size);
        let body_size = st.font_size.unwrap_or(small);
        let kind = restyle(
            layout_label(Some(&format!("«{}»", if is_req { node.kind.as_str() } else { "Element" })), false, &m, body_size, 4000.0, 0),
            base,
            &m,
        );
        let name = restyle(layout_label(Some(&node.name), true, &m, name_size, 280.0, 0), base | style::BOLD, &m);
        let fields = if is_req {
            vec![("ID", &node.id), ("Text", &node.text), ("Risk", &node.risk), ("Verification", &node.verify)]
        } else {
            vec![("Type", &node.kind), ("Doc Ref", &node.doc_ref)]
        };
        let mut rows = vec![];
        let mut key_width = 0.0;
        let mut value_width = 0.0;
        let mut body_height = 0.0;
        for (label, text) in fields {
            if text.is_empty() {
                continue;
            }
            let key = restyle(layout_label(Some(label), false, &m, body_size, 4000.0, 0), base, &m);
            let value = restyle(layout_label(Some(text), true, &m, body_size, 240.0, 0), base, &m);
            let height = js::max(key.height, value.height);
            key_width = js::max(key_width, key.width);
            value_width = js::max(value_width, value.width);
            body_height += height + 2.0;
            rows.push(Row { key, value, height });
        }
        if !rows.is_empty() {
            body_height += 18.0;
        }
        let head_height = 20.0 + kind.height + name.height;
        let w = js::max(
            120.0,
            js::max(js::max(kind.width, name.width) + 28.0, if rows.is_empty() { 0.0 } else { key_width + 12.0 + value_width + 28.0 }),
        );
        index.entry(node.name.clone()).or_insert(views.len());
        let mut c = cnode(w, head_height + body_height, -1);
        c.seq = views.len() as f64;
        views.push(View {
            node,
            requirement: is_req,
            style: st,
            kind,
            name,
            rows,
            key_width,
            head_height,
            w,
            h: head_height + body_height,
        });
        nodes.push(c);
    }
    let dir = if turned { Dir::TB } else { direction(Some(&model.direction)) };
    let sideways = matches!(dir, Dir::LR | Dir::RL);
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    for (i, r) in model.relations.iter().enumerate() {
        let (Some(&src), Some(&dst)) = (index.get(&r.src), index.get(&r.dst)) else {
            continue;
        };
        let label = layout_label(Some(&format!("«{}»", r.kind)), false, &m, small, 4000.0, 0);
        if src == dst {
            if sideways {
                nodes[src].h += 2.0 * (26.0 + label.height + 8.0);
            } else {
                nodes[src].w += 2.0 * (26.0 + label.width + 12.0);
            }
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: 1.0,
            label_w: label.width + 12.0,
            label_h: label.height + 4.0,
            route: vec![],
            label_x: 0.0,
            label_y: 0.0,
        });
        drawn.push((format!("{}-{}-{i}", r.src, r.dst), r.kind == "contains"));
        labels.push(label);
    }
    let opt = LayeredOptions {
        node_sep: 40.0,
        edge_sep: 16.0,
        rank_sep: 56.0,
        port_sep: 20.0,
        tracks: false,
        straight: false,
        apart: false,
        head_room: None,
    };
    let layout = compound_layout(&mut nodes, &mut edges, dir, &opt);
    spread(&nodes, &mut edges, &views);
    let title = layout_label(model.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(layout.width, title.width);
    let ox = pad + (inner - layout.width) / 2.0;
    let oy = pad + title_h;
    let width = (inner + 2.0 * pad).ceil();
    let height = (layout.height + title_h + 2.0 * pad).ceil();
    let mut node_svg = String::new();
    for (i, v) in views.iter().enumerate() {
        let left = -v.w / 2.0;
        let top = -v.h / 2.0;
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
            "/>",
            label_svg(
                &v.kind,
                0.0,
                top + 10.0 + v.kind.height / 2.0,
                &svg_format!(
                    " class=\"pele-stereotype\" font-size=\"",
                    js::to_string(v.kind.size),
                    "\" fill=\"",
                    ctx.paint(Role::SecondaryText, "var(--_m)"),
                    "\"",
                    st.text
                ),
                None
            ),
            label_svg(
                &v.name,
                0.0,
                top + 10.0 + v.kind.height + v.name.height / 2.0,
                &svg_format!(" class=\"pele-label\" font-size=\"", js::to_string(v.name.size), "\" font-weight=\"var(--_hw)\"", st.text),
                None
            )
        );
        if !v.rows.is_empty() {
            let divider = top + v.head_height;
            svg_write!(
                body,
                "<path class=\"pele-divider\" d=\"M",
                num(left),
                ",",
                num(divider),
                "H",
                num(-left),
                "\" stroke=\"",
                ctx.paint(Role::InnerStroke, "var(--_b)"),
                "\"",
                st.line,
                "/>"
            )
            .unwrap();
            let mut y = divider + 10.0;
            let mut row_svg = String::new();
            for row in &v.rows {
                row_svg.push_str(&start_text(
                    &row.key,
                    left + 14.0,
                    y,
                    &svg_format!(" class=\"pele-field\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", st.text),
                ));
                row_svg.push_str(&start_text(
                    &row.value,
                    left + 14.0 + v.key_width + 12.0,
                    y,
                    &svg_format!(" class=\"pele-value\"", st.text),
                ));
                y += row.height + 2.0;
            }
            svg_write!(body, "<g class=\"pele-fields\" font-size=\"", js::to_string(v.rows[0].key.size), "\">", row_svg, "</g>").unwrap();
        }
        let classes = v.node.classes.join(" ");
        let classes = classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&classes);
        svg_write!(
            node_svg,
            "<g class=\"pele-node ",
            if v.requirement { "pele-requirement" } else { "pele-element" },
            class_names(classes),
            "\" data-id=\"",
            esc_text(&v.node.name),
            "\" transform=\"translate(",
            num(nodes[i].x + ox),
            ",",
            num(nodes[i].y + oy),
            ")\">",
            body,
            "</g>"
        )
        .unwrap();
    }
    let mut edge_svg = String::new();
    let mut labels_svg = String::new();
    let mut loops = IndexMap::<usize, usize>::new();
    for (i, e) in edges.iter().enumerate() {
        let (id, contains) = &drawn[i];
        let label = &labels[i];
        let start_trim = if *contains { 12.0 } else { 0.0 };
        let mut lx = e.label_x + ox;
        let mut ly = e.label_y + oy;
        let path = if e.src == e.dst {
            let v = &views[e.src];
            let cx = nodes[e.src].x + ox;
            let cy = nodes[e.src].y + oy;
            let k = loops.entry(e.src).or_default();
            let half = if sideways { v.w } else { v.h } / 2.0;
            let spread = js::min(half - 4.0, 12.0 + *k as f64 * 6.0);
            let reach = 26.0 + *k as f64 * 8.0;
            *k += 1;
            let at = |out, along| {
                if sideways {
                    format!("{},{}", num(cx + along), num(cy + v.h / 2.0 + out))
                } else {
                    format!("{},{}", num(cx + v.w / 2.0 + out), num(cy + along))
                }
            };
            let pull = reach / 0.75;
            if sideways {
                lx = cx;
                ly = cy + v.h / 2.0 + reach + 4.0 + e.label_h / 2.0;
            } else {
                lx = cx + v.w / 2.0 + reach + 4.0 + e.label_w / 2.0;
                ly = cy;
            }
            EdgePath {
                d: svg_format!("M", at(start_trim, -spread), "C", at(pull, -spread), " ", at(pull, spread), " ", at(0.0, spread)),
                sx: if sideways { cx - spread } else { cx + v.w / 2.0 },
                sy: if sideways { cy + v.h / 2.0 } else { cy - spread },
                sdx: if sideways { 0.0 } else { -1.0 },
                sdy: if sideways { -1.0 } else { 0.0 },
                ex: if sideways { cx + spread } else { cx + v.w / 2.0 },
                ey: if sideways { cy + v.h / 2.0 } else { cy + spread },
                edx: if sideways { 0.0 } else { -1.0 },
                edy: if sideways { -1.0 } else { 0.0 },
            }
        } else {
            let mut route = e.route.clone();
            for p in route.chunks_exact_mut(3) {
                p[0] += ox;
                p[1] += oy;
            }
            route_path(&route, None, start_trim, 0.0, default_reach(start_trim), default_reach(0.0))
        };
        svg_write!(
            edge_svg,
            "<g class=\"pele-edge pele-relationship-",
            if *contains { "contains" } else { "arrow" },
            "\" data-id=\"",
            esc_text(id),
            "\"><path d=\"",
            path.d,
            "\"",
            if *contains { "" } else { " stroke-dasharray=\"5 4\"" },
            "/>",
            if *contains { contains_marker(path.sx, path.sy, path.sdx, path.sdy) } else { arrow(path.ex, path.ey, path.edx, path.edy) },
            "</g>"
        )
        .unwrap();
        svg_write!(labels_svg, edge_label_svg(id, label, lx, ly, "", None, ctx.paint(Role::Background, "var(--_bg)"))).unwrap();
    }
    let mut content =
        label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
    if !edge_svg.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-edges\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\" stroke-linecap=\"round\">",
            edge_svg,
            "</g>"
        )
        .unwrap();
    }
    if !labels_svg.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-edge-labels\" font-size=\"",
            js::to_string(small),
            "\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            labels_svg,
            "</g>"
        )
        .unwrap();
    }
    svg_write!(content, "<g class=\"pele-nodes\">", node_svg, "</g>").unwrap();
    Ok(Rendered {
        svg: svg_document(
            "requirement",
            width,
            height,
            size,
            ctx,
            Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() },
            &content,
        ),
        width,
        height,
        links: vec![],
    })
}
