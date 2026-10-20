//! SVG flowcharts, ported from Pele flowchart/render.ts. Layout remains independent of paint.
use super::{
    graph::{self, GraphEdge, GraphNode},
    model::FlowchartModel,
    shapes::canonical,
};
use crate::diagrams::common::fit_width::{runs_across, tighten, title_room, turn_to_fit};
use crate::diagrams::{RenderCtx, Rendered};
use crate::layout::compound::{cnode, compound_layout, direction, shift_layout, CEdge, Dir};
use crate::layout::{lanes::lane_layout, layered::LayeredOptions};
use crate::svg::cluster::render_struck_title as struck_title;
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::edges::render_push_edge_label_svg as push_edge_label_svg;
use crate::svg::{
    builder::{esc, esc_text, joined, num, push_num, render_label_svg as label_svg, render_push_label_svg as push_label_svg},
    cluster::*,
    edges::*,
    root::{svg_document, Accessible},
    shapes::*,
    theme::{class_names, resolve_style, ResolvedStyle, RADIUS},
};
use crate::text::entities::decode_entities;
use crate::text::{layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, url::*};
use crate::{Config, ConfigMap, ConfigValue, Error};
use indexmap::IndexMap;

struct View<'a> {
    node: &'a GraphNode,
    style: ResolvedStyle,
    label: Label,
    shape: String,
    w: f64,
    h: f64,
    dy: f64,
}
fn positive(config: &ConfigMap, key: &str, fallback: f64) -> f64 {
    config.get(key).and_then(ConfigValue::as_f64).filter(|n| *n > 0.0).map_or(fallback, |n| js::min(n, 2000.0))
}
fn asset(value: Option<f64>, fallback: f64) -> f64 {
    value.filter(|n| *n > 0.0 && *n <= 4000.0).unwrap_or(fallback)
}
pub(crate) fn render(model: &FlowchartModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    render_variant(model, config, ctx, false)
}
pub(crate) fn render_variant(model: &FlowchartModel, config: &Config, ctx: &RenderCtx, swimlane: bool) -> Result<Rendered, Error> {
    let fitted = |down| tighten(ctx.options, |tight| draw(model, config, ctx, swimlane, down, tight));
    if runs_across(model.direction.as_deref()) {
        turn_to_fit(ctx.options, fitted)
    } else {
        fitted(false)
    }
}
fn draw(db: &FlowchartModel, config: &Config, ctx: &RenderCtx, swimlane: bool, turned: bool, tight: bool) -> Result<Rendered, Error> {
    let options = ctx.options;
    let empty = ConfigMap::new();
    let group_pad = if tight { 12.0 } else { 20.0 };
    let flow = if swimlane { config.section("swimlane").or_else(|| config.section("flowchart")) } else { config.section("flowchart") }
        .unwrap_or(&empty);
    let mut graph = graph::build(db, flow.get("curve").and_then(ConfigValue::as_str));
    if swimlane {
        let mut loose = false;
        for node in &mut graph.nodes {
            if node.is_group {
                node.dir = None;
            } else if node.parent.is_none() {
                node.parent = Some("__swimlane_default__".into());
                loose = true;
            }
        }
        if loose {
            graph.nodes.push(GraphNode {
                id: "__swimlane_default__".into(),
                label_type: "text".into(),
                shape: "rect".into(),
                is_group: true,
                ..Default::default()
            });
        }
    }
    let size = ctx.font_size();
    let edge_size = js::round(size * 0.875);
    let measure = ctx.measure();
    let wrap = js::min(positive(flow, "wrappingWidth", 200.0), if swimlane && tight { 100.0 } else { f64::INFINITY });
    let pad = ctx.padding();
    let icons = ctx.icons();
    let index: IndexMap<&str, usize> = graph.nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let mut views = Vec::with_capacity(graph.nodes.len());
    let mut cnodes = Vec::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        let mut styles = node.compiled.clone();
        styles.extend(node.styles.iter().cloned());
        let st = resolve_style(&styles);
        let text_style = if st.bold { style::BOLD } else { 0 } | if st.italic { style::ITALIC } else { 0 };
        let font_size = st.font_size.unwrap_or(size);
        let (label, shape, w, h, dy, mut c) = if node.is_group {
            let label = layout_label(
                node.label.as_deref(),
                node.label_type == "markdown",
                &measure,
                font_size,
                if swimlane && tight { js::max(80.0, title_room(options) / 2.0) } else { 4000.0 },
                text_style,
            );
            let mut c = cnode(0.0, 0.0, -1);
            c.is_group = true;
            c.dir = node.dir.as_deref().filter(|s| !s.is_empty()).map(|s| direction(Some(s)));
            c.pad_x = group_pad;
            c.pad_top = if label.height > 0.0 { label.height + 16.0 } else { 20.0 };
            c.pad_bottom = group_pad;
            c.min_w = label.width + 2.0 * group_pad - 12.0;
            (label, "group".to_owned(), 0.0, 0.0, 0.0, c)
        } else {
            let shape = canonical(&node.shape).to_owned();
            let text = if node.img.is_some() || node.icon.is_some() || shape_has_label(&shape) { node.label.as_deref() } else { None };
            let label = layout_label(text, node.label_type == "markdown", &measure, font_size, wrap, text_style);
            let (w, h, dy) = if node.img.is_some() || node.icon.is_some() {
                let box_size = if node.img.is_some() { 80.0 } else { 48.0 };
                let aw = asset(node.asset_width, box_size);
                let ah = asset(node.asset_height, box_size);
                (
                    js::max(aw, label.width + 8.0),
                    ah + if label.height > 0.0 { label.height + 6.0 } else { 0.0 },
                    if label.height == 0.0 {
                        0.0
                    } else if node.pos.as_deref() == Some("t") {
                        -ah / 2.0
                    } else {
                        ah / 2.0
                    },
                )
            } else {
                let s = shape_size(&shape, label.width, label.height);
                (s.w, s.h, s.dy)
            };
            (label, shape, w, h, dy, cnode(w, h, -1))
        };
        c.seq = views.len() as f64;
        views.push(View { node, style: st, label, shape, w, h, dy });
        cnodes.push(c);
    }
    for (i, node) in graph.nodes.iter().enumerate() {
        cnodes[i].parent = node.parent.as_deref().and_then(|p| index.get(p)).map_or(-1, |i| *i as isize);
    }
    let root_dir = if turned { Dir::TB } else { direction(db.direction.as_deref()) };
    let sideways: Vec<bool> = (0..cnodes.len())
        .map(|i| {
            let mut p = cnodes[i].parent;
            let mut d = None;
            let mut hops = 0;
            while p >= 0 && d.is_none() && hops < 10000 {
                d = cnodes[p as usize].dir;
                p = cnodes[p as usize].parent;
                hops += 1;
            }
            matches!(d.unwrap_or(root_dir), Dir::LR | Dir::RL)
        })
        .collect();
    for (i, v) in views.iter().enumerate() {
        if v.node.is_group {
            cnodes[i].span = f64::INFINITY;
        } else if v.node.img.is_none() && v.node.icon.is_none() {
            cnodes[i].span = shape_span(&v.shape, v.w, v.h, sideways[i]);
        }
    }
    let mut cedges = Vec::new();
    let mut labels = Vec::new();
    let mut drawn: Vec<&GraphEdge> = Vec::new();
    for edge in &graph.edges {
        let (Some(&src), Some(&dst)) = (index.get(edge.start.as_str()), index.get(edge.end.as_str())) else {
            continue;
        };
        let label = layout_label(Some(&edge.label), edge.label_type == "markdown", &measure, edge_size, wrap, 0);
        if src == dst {
            if sideways[src] {
                cnodes[src].h += 2.0 * (26.0 + if label.height > 0.0 { label.height + 8.0 } else { 0.0 });
            } else {
                cnodes[src].w += 2.0 * (26.0 + if label.width > 0.0 { label.width + 12.0 } else { 0.0 });
            }
        }
        cedges.push(CEdge {
            src,
            dst,
            minlen: edge.minlen.unwrap_or(1.0),
            label_w: if label.width > 0.0 { label.width + 12.0 } else { 0.0 },
            label_h: if label.height > 0.0 { label.height + 4.0 } else { 0.0 },
            route: vec![],
            label_x: 0.0,
            label_y: 0.0,
        });
        labels.push(label);
        drawn.push(edge);
    }
    let curve = flow.get("curve").and_then(ConfigValue::as_str);
    let opt = LayeredOptions {
        node_sep: positive(flow, "nodeSpacing", 40.0) * if tight { 0.5 } else { 1.0 },
        edge_sep: 16.0,
        rank_sep: positive(flow, "rankSpacing", 48.0) * if tight { 0.75 } else { 1.0 },
        port_sep: 20.0,
        tracks: curve.is_some_and(|s| s.starts_with("step")),
        straight: curve == Some("linear"),
        apart: false,
        head_room: Some(marker_trim("arrow_point")),
    };
    let layout = if swimlane {
        lane_layout(&mut cnodes, &mut cedges, root_dir, &opt, if tight { 12.0 } else { crate::layout::lanes::LANE_PAD })
    } else {
        compound_layout(&mut cnodes, &mut cedges, root_dir, &opt)
    };
    let title = layout_label(db.title.as_deref(), false, &measure, size, if tight { title_room(options) } else { 4000.0 }, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(layout.width, title.width);
    shift_layout(&mut cnodes, &mut cedges, pad + (inner - layout.width) / 2.0, pad + title_h);
    let width = (inner + 2.0 * pad).ceil();
    let height = (layout.height + title_h + 2.0 * pad).ceil();
    let mut links = vec![];
    let mut crossings = Crossings::new();
    if cnodes.iter().any(|c| c.is_group) {
        for edge in &cedges {
            mark_crossings(&mut crossings, &edge.route);
        }
    }
    let mut clusters = String::new();
    let mut struck = String::new();
    let mut nodes_out = String::new();
    let shape_attrs =
        svg_format!(" fill=\"", ctx.paint(Role::NodeFill, "var(--_s)"), "\" stroke=\"", ctx.paint(Role::NodeStroke, "var(--_b)"), "\"");
    let line_attrs = svg_format!(" stroke=\"", ctx.paint(Role::InnerStroke, "var(--_b)"), "\"");
    for (i, v) in views.iter().enumerate() {
        let c = &cnodes[i];
        let node = &v.node;
        let class_source = node.classes.strip_prefix("default").map(|s| s.strip_prefix(' ').unwrap_or(s)).unwrap_or(&node.classes);
        let classes = class_names(class_source);
        let id = esc_text(&node.id);
        if c.is_group && swimlane && c.parent < 0 {
            let left = c.x - c.w / 2.0;
            let top = c.y - c.h / 2.0;
            let across = matches!(root_dir, Dir::LR | Dir::RL);
            let cx = if across { left + c.pad_top / 2.0 } else { c.x };
            let cy = if across { c.y } else { top + c.pad_top / 2.0 };
            let text = svg_format!(label_svg(
                &v.label,
                cx,
                cy,
                &svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", v.style.text),
                icons,
            ));
            svg_write!(
                clusters,
                "<g class=\"pele-cluster pele-lane",
                classes,
                "\" data-id=\"",
                id,
                "\"><rect x=\"",
                num(left),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(c.w),
                "\" height=\"",
                num(c.h),
                "\" fill=\"none\" stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"/><rect x=\"",
                num(left),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(if across { c.pad_top } else { c.w }),
                "\" height=\"",
                num(if across { c.h } else { c.pad_top }),
                "\" fill=\"",
                ctx.paint(Role::GroupHeader, "var(--_a)"),
                "\" stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"",
                v.style.shape,
                "/>",
                if across && !text.is_empty() {
                    svg_format!("<g transform=\"rotate(-90 ", num(cx), " ", num(cy), ")\">", text, "</g>")
                } else {
                    text
                },
                "</g>"
            )
            .unwrap();
            continue;
        }
        if c.is_group {
            let tx = cluster_title_x(c.x, c.y, c.w, c.h, v.label.width, &crossings, group_pad);
            let ty = c.y - c.h / 2.0 + 8.0 + v.label.height / 2.0;
            let attrs =
                svg_format!(" class=\"pele-cluster-label\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"", v.style.text);
            let crossed = title_crossed(c.x, c.y, c.w, c.h, v.label.width, tx, &crossings);
            if crossed {
                struck.push_str(&struck_title(&v.label, tx, ty, &attrs, &id, |j| cnodes[j].parent, i, icons, ctx.theming));
            }
            svg_write!(
                clusters,
                "<g class=\"pele-cluster",
                classes,
                "\" data-id=\"",
                id,
                "\"><rect x=\"",
                num(c.x - c.w / 2.0),
                "\" y=\"",
                num(c.y - c.h / 2.0),
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
                if crossed { String::new() } else { svg_format!(label_svg(&v.label, tx, ty, &attrs, icons)) },
                "</g>"
            )
            .unwrap();
            continue;
        }
        let inner = if let Some(img) = node.img.as_ref().filter(|s| !s.is_empty()) {
            let aw = asset(node.asset_width, 80.0);
            let ah = asset(node.asset_height, 80.0);
            let top = if v.label.height > 0.0 {
                if node.pos.as_deref() == Some("t") {
                    v.h / 2.0 - ah
                } else {
                    -v.h / 2.0
                }
            } else {
                -ah / 2.0
            };
            image_url(&safe_url(img), options).map_or_else(String::new, |src| {
                svg_format!(
                    "<image href=\"",
                    esc(&src),
                    "\" x=\"",
                    num(-aw / 2.0),
                    "\" y=\"",
                    num(top),
                    "\" width=\"",
                    num(aw),
                    "\" height=\"",
                    num(ah),
                    "\" preserveAspectRatio=\"",
                    if node.constraint.as_deref() == Some("on") { "xMidYMid meet" } else { "none" },
                    "\"/>"
                )
            })
        } else if let Some(icon) = node.icon.as_ref().filter(|s| !s.is_empty()) {
            let b = asset(node.asset_height, 48.0);
            let top = if v.label.height > 0.0 {
                if node.pos.as_deref() == Some("t") {
                    v.h / 2.0 - b
                } else {
                    -v.h / 2.0
                }
            } else {
                -b / 2.0
            };
            let frame = if node.shape == "icon" {
                String::new()
            } else if node.shape == "iconCircle" {
                svg_format!("<circle cy=\"", num(top + b / 2.0), "\" r=\"", num(b / 2.0), "\"", shape_attrs, v.style.shape, "/>")
            } else {
                svg_format!(
                    "<rect x=\"",
                    num(-b / 2.0),
                    "\" y=\"",
                    num(top),
                    "\" width=\"",
                    num(b),
                    "\" height=\"",
                    num(b),
                    "\" rx=\"",
                    if node.shape == "iconRounded" { 8 } else { 0 },
                    "\"",
                    shape_attrs,
                    v.style.shape,
                    "/>"
                )
            };
            let glyph = b * 0.6;
            svg_format!(frame, "<svg class=\"pele-icon\" data-icon=\"", esc_text(icon), "\" x=\"", num(-glyph/2.0), "\" y=\"", num(top+(b-glyph)/2.0), "\" width=\"", num(glyph), "\" height=\"", num(glyph), "\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">", icons.map_or_else(String::new,|i|i.markup(icon).to_string()), "</svg>")
        } else {
            draw_shape(&v.shape, v.w, v.h, &joined(&shape_attrs, &v.style.shape), &joined(&line_attrs, &v.style.line), ctx.theming)
        };
        let label_attrs = joined(" class=\"pele-label\"", &v.style.text);
        let internal = classes.split_whitespace().any(|s| s == "internal-link")
            && node.label.as_ref().is_some_and(|s| !s.is_empty())
            && options.links != Some(false);
        let tooltip = node.tooltip.as_ref().filter(|s| !s.is_empty());
        let link = node.link.as_ref().filter(|s| !s.is_empty());
        svg_write!(nodes_out, "<g class=\"pele-node pele-shape-", v.shape, classes, "\" data-id=\"", id, "\" transform=\"translate(")
            .unwrap();
        push_num(&mut nodes_out, c.x);
        nodes_out.push(',');
        push_num(&mut nodes_out, c.y);
        nodes_out.push_str(")\">");
        if !internal && tooltip.is_none() && link.is_none() {
            // The usual node: its shape and label go straight into the output.
            nodes_out.push_str(&inner);
            push_label_svg(&mut nodes_out, &v.label, 0.0, v.dy, &label_attrs, icons);
            nodes_out.push_str("</g>");
            continue;
        }
        let mut text = svg_format!(label_svg(&v.label, 0.0, v.dy, &label_attrs, icons));
        if internal {
            let name = decode_entities(node.label.as_deref().unwrap());
            let allowed = link_url(&sanitize_url(&name), options).is_some();
            let target = esc(&name);
            text = svg_format!(
                "<a class=\"internal-link\"",
                if allowed { svg_format!(" href=\"", target, "\"", rel_attr(options)) } else { String::new() },
                " data-href=\"",
                target,
                "\">",
                text,
                "</a>"
            );
            links.push(LinkInfo { id: node.id.clone(), href: name, internal: true });
        }
        let mut body = inner + &text;
        if let Some(t) = tooltip {
            body = svg_format!("<title>", esc_text(t), "</title>", body);
        }
        if let Some(url) = link {
            body = linked(
                &body,
                url,
                &node.id,
                &mut links,
                options,
                &node.link_target.as_ref().filter(|s| !s.is_empty()).map_or_else(String::new, |t| svg_format!(" target=\"", esc(t), "\"")),
            );
        }
        nodes_out.push_str(&body);
        nodes_out.push_str("</g>");
    }
    let mut reach = side_reach();
    for (i, e) in cedges.iter().enumerate() {
        if e.src == e.dst || drawn[i].thickness == "invisible" || e.route.len() < 6 {
            continue;
        }
        let s = &cnodes[e.src];
        let t = &cnodes[e.dst];
        reach.add(e.src as i64, end_side(&e.route, 0, s.x, s.y, s.w, s.h) as i64, &drawn[i].arrow_start);
        reach.add(e.dst as i64, end_side(&e.route, e.route.len() - 3, t.x, t.y, t.w, t.h) as i64, &drawn[i].arrow_end);
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loop_seen = IndexMap::<usize, usize>::new();
    for (i, e) in cedges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &labels[i];
        let st = if !edge.styles.is_empty() || !edge.compiled.is_empty() {
            let mut s = edge.compiled.clone();
            s.extend(edge.styles.iter().cloned());
            Some(resolve_style(&s))
        } else {
            None
        };
        if edge.thickness != "invisible" {
            let s = &cnodes[e.src];
            let t = &cnodes[e.dst];
            let path = if e.src == e.dst {
                let k = loop_seen.entry(e.src).or_default();
                let mut ll = LoopLabel { label_w: e.label_w, label_h: e.label_h, label_x: e.label_x, label_y: e.label_y };
                let p = loop_path(
                    s.x,
                    s.y,
                    views[e.src].w,
                    views[e.src].h,
                    *k as f64,
                    sideways[e.src],
                    26.0,
                    marker_trim(&edge.arrow_end),
                    &mut ll,
                );
                *k += 1;
                e.label_x = ll.label_x;
                e.label_y = ll.label_y;
                p
            } else {
                let mut route = e.route.clone();
                let last = route.len() - 3;
                let leaves = reach.of(e.src as i64, end_side(&route, 0, s.x, s.y, s.w, s.h) as i64);
                let arrives = reach.of(e.dst as i64, end_side(&route, last, t.x, t.y, t.w, t.h) as i64);
                inset_route(&mut route, 0, &views[e.src].shape, views[e.src].w, views[e.src].h, s.x, s.y, s.w, s.h, s.is_group);
                inset_route(&mut route, last, &views[e.dst].shape, views[e.dst].w, views[e.dst].h, t.x, t.y, t.w, t.h, t.is_group);
                route_path(&route, edge.curve.as_deref(), marker_trim(&edge.arrow_start), marker_trim(&edge.arrow_end), leaves, arrives)
            };
            let color = st
                .as_ref()
                .and_then(|s| s.stroke.as_ref())
                .map_or_else(|| ctx.paint(Role::Arrow, "var(--_l)").to_string(), |s| esc(s).into_owned());
            let attrs = match edge.thickness.as_str() {
                "thick" => " stroke-width=\"2.5\"",
                "dotted" => " stroke-dasharray=\"3 4\"",
                _ => "",
            };
            let classes = class_names(
                &edge.classes.replace("edge-thickness-normal", "").replace("edge-pattern-solid", "").replace("flowchart-link", ""),
            );
            svg_write!(
                edges_out,
                "<g class=\"pele-edge",
                classes,
                "\" data-id=\"",
                esc_text(&edge.id),
                "\"",
                st.as_ref().map_or("", |s| s.line.as_str()),
                "><path d=\"",
                path.d,
                "\"",
                attrs,
                "/>"
            )
            .unwrap();
            push_marker(&mut edges_out, &edge.arrow_start, path.sx, path.sy, path.sdx, path.sdy, &color);
            push_marker(&mut edges_out, &edge.arrow_end, path.ex, path.ey, path.edx, path.edy, &color);
            edges_out.push_str("</g>");
        }
        if label.width > 0.0 {
            push_edge_label_svg(
                &mut labels_out,
                &edge.id,
                label,
                e.label_x,
                e.label_y,
                st.as_ref().map_or("", |s| s.text.as_str()),
                icons,
                ctx.paint(Role::Background, "var(--_bg)"),
            );
        }
    }
    let mut content = String::with_capacity(clusters.len() + edges_out.len() + struck.len() + labels_out.len() + nodes_out.len() + 256);
    push_label_svg(&mut content, &title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None);
    if !clusters.is_empty() {
        content.push_str("<g class=\"pele-clusters\">");
        content.push_str(&clusters);
        content.push_str("</g>");
    }
    if !edges_out.is_empty() {
        content.push_str("<g class=\"pele-edges\" fill=\"none\" stroke=\"");
        content.push_str(ctx.paint(Role::Line, "var(--_l)"));
        content.push_str("\" stroke-linecap=\"round\">");
        content.push_str(&edges_out);
        content.push_str("</g>");
    }
    if !struck.is_empty() {
        content.push_str("<g class=\"pele-cluster-titles\">");
        content.push_str(&struck);
        content.push_str("</g>");
    }
    if !labels_out.is_empty() {
        content.push_str("<g class=\"pele-edge-labels\" font-size=\"");
        content.push_str(&js::to_string(edge_size));
        content.push_str("\">");
        content.push_str(&labels_out);
        content.push_str("</g>");
    }
    content.push_str("<g class=\"pele-nodes\">");
    content.push_str(&nodes_out);
    content.push_str("</g>");
    let acc = Accessible { acc_title: db.acc_title.as_deref(), acc_descr: db.acc_descr.as_deref() };
    Ok(Rendered {
        svg: svg_document(if swimlane { "swimlane" } else { "flowchart" }, width, height, size, ctx, acc, &content),
        width,
        height,
        links,
    })
}
