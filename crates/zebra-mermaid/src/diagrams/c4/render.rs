//! C4 rendering. Ported from Pele `src/diagrams/c4/render.ts`.
//!
//! In resolved mode the parts take the colors Zebra's native renderer gives C4 diagrams
//! (`apply_c4_colors`, the c4 block of `apply_layout_config` and `apply_layout` in its
//! `theme.rs`): people, systems, containers and components each have their own fill and
//! outline, boundaries are drawn as groups, and relation text is muted. Colors the diagram's
//! author gives (`UpdateElementStyle`, `UpdateRelStyle`, `$bgColor` and the like) are written
//! as Pele writes them, in `style` attributes, so they keep precedence in both modes.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::{HashMap, HashSet};

use super::layout::{layout_c4, HEAD_LEFT, HEAD_TOP};
use super::route::{crosses, cyl_ry, queue_rx, route, Bx, Lanes, Route, HEAD, LANE};
use super::{C4Model, ElementRef};
use crate::config::Config;
use crate::detect::DiagramType;
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::common::literal_label::{push_left_label_svg, render_left_label_svg as left_label_svg};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::options::RenderOptions;
use crate::svg::builder::{esc, esc_text, num, render_label_svg as label_svg};
use crate::svg::edges::{marker, marker_trim, ARROW};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::shapes::draw_shape;
use crate::svg::theme::{resolve_style, ResolvedStyle, RADIUS};
use crate::text::{layout_render_label as layout_label, style};
use crate::theme::color::mix_hex;
use crate::theme::Role;
use crate::util::js;
use crate::util::url::{linked, sanitize_url, LinkInfo};
use std::fmt::Write as _;

const PAIR_SHIFT: f64 = 7.0;
const NARROW_GAP: f64 = 36.0;
const WIDEN: f64 = 24.0;
const MAX_OFFSET: f64 = 2000.0;
/// A label may cover this much of a shape's edge, where there is no text.
const EDGE: f64 = 8.0;
/// Places along a relation to try for its label, nearest the middle first.
const SLIDE: [f64; 7] = [0.5, 0.4, 0.6, 0.3, 0.7, 0.2, 0.8];
/// Above this many box comparisons, labels stay at the middle and lines are not steered
/// around shapes.
const CARE_BUDGET: f64 = 2000000.0;

fn user_style(fill: Option<&str>, stroke: Option<&str>, color: Option<&str>) -> ResolvedStyle {
    let mut declarations: Vec<String> = Vec::new();
    if let Some(fill) = fill.filter(|s| !s.is_empty()) {
        declarations.push(format!("fill:{}", fill));
    }
    if let Some(stroke) = stroke.filter(|s| !s.is_empty()) {
        declarations.push(format!("stroke:{}", stroke));
    }
    if let Some(color) = color.filter(|s| !s.is_empty()) {
        declarations.push(format!("color:{}", color));
    }
    resolve_style(&declarations)
}

/// `.replace(/ﬂ°°?/g, '#')`
fn restore_hashes(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    let mut rest = url;
    while let Some(k) = rest.find("\u{fb02}\u{b0}") {
        out.push_str(&rest[..k]);
        out.push('#');
        rest = &rest[k + "\u{fb02}\u{b0}".len()..];
        if let Some(after) = rest.strip_prefix('\u{b0}') {
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn link(body: String, url: Option<&str>, id: &str, links: &mut Vec<LinkInfo>, options: &RenderOptions) -> String {
    let Some(url) = url.filter(|u| !u.is_empty()) else { return body };
    // Entity encoding has hidden any `#name;` in the address. Put it back, so that the check
    // sees what a browser would.
    let url = if url.contains("\u{b6}\u{df}") { restore_hashes(url).replace("\u{b6}\u{df}", ";") } else { url.to_string() };
    linked(&body, &sanitize_url(&url), id, links, options, "")
}

pub(crate) fn render(model: &C4Model, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| {
        if !tight {
            return Ok(draw(model, config, ctx, false, NARROW_GAP)?.0);
        }
        let (mut best, mut squeezed) = draw(model, config, ctx, true, NARROW_GAP)?;
        let mut gap = NARROW_GAP + WIDEN;
        while squeezed > 0 && gap <= NARROW_GAP + 3.0 * WIDEN {
            let (wider, left) = draw(model, config, ctx, true, gap)?;
            if left < squeezed {
                best = wider;
                squeezed = left;
            }
            gap += WIDEN;
        }
        Ok(best)
    })
}

/// The fill and outline of a shape in resolved mode, by kind, as Zebra's `apply_c4_colors`
/// sets them.
fn shape_colors(ctx: &RenderCtx, kind: &str, external: bool) -> (String, String) {
    let t = &ctx.theming.resolved;
    if external {
        return (t.group_header.clone(), t.node_stroke.clone());
    }
    if kind == "person" {
        (mix_hex(&t.accent, &t.background, 0.16), t.accent.clone())
    } else if kind.starts_with("system") {
        (mix_hex(&t.accent, &t.background, 0.09), t.accent.clone())
    } else if kind.starts_with("component") {
        (mix_hex(&t.text, &t.background, 0.03), t.node_stroke.clone())
    } else {
        (t.node_fill.clone(), t.node_stroke.clone())
    }
}

fn draw(db: &C4Model, _config: &Config, ctx: &RenderCtx, tight: bool, gap: f64) -> Result<(Rendered, usize), Error> {
    let options = ctx.options;
    let theming = ctx.theming;
    let resolved = ctx.resolved();
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.8125);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let mut links: Vec<LinkInfo> = Vec::new();
    let room = if tight { Some(js::max(80.0, options.max_width.unwrap_or(320.0) - 2.0 * pad)) } else { None };
    let mut layout = layout_c4(db, &m, size, small, room, gap);
    let root = layout.order[0];

    let mut min_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_x = layout.groups[root].bx.w;
    let mut max_y = layout.groups[root].bx.h;
    let mut grow = |x0: f64, y0: f64, x1: f64, y1: f64| {
        min_x = js::min(min_x, x0);
        min_y = js::min(min_y, y0);
        max_x = js::max(max_x, x1);
        max_y = js::max(max_y, y1);
    };

    let box_of = |r: Option<ElementRef>| -> Option<Bx> {
        match r? {
            ElementRef::Shape(i) => layout.shape_bx(i),
            ElementRef::Boundary(i) => layout.boundary_bx(i),
        }
    };
    let mut ends: Vec<Bx> = Vec::with_capacity(db.rels.len() * 2);
    let mut pairs: HashMap<usize, HashSet<usize>> = HashMap::new();
    for rel in &db.rels {
        let from = if rel.from_attr.is_some() { None } else { db.element(&rel.from) };
        let to = if rel.to_attr.is_some() { None } else { db.element(&rel.to) };
        let (Some(a), Some(b)) = (box_of(from), box_of(to)) else {
            return Err(Error::semantic(format!("C4 rel \"{}\" -> \"{}\" references an unknown shape or boundary", rel.from, rel.to))
                .with_type(DiagramType::C4));
        };
        ends.push(a);
        ends.push(b);
        pairs.entry(a.id).or_default().insert(b.id);
    }

    // What a relation's label should not cover: shapes, arrowheads, boundary headings, and the
    // labels placed before it.
    let mut nodes: Vec<Bx> = Vec::new();
    let mut taken: Vec<f64> = Vec::new();
    for &g in &layout.order {
        for node in &layout.groups[g].nodes {
            let b = node.bx;
            nodes.push(b);
            taken.extend_from_slice(&[b.x + EDGE, b.y + EDGE, b.x + b.w - EDGE, b.y + b.h - EDGE]);
        }
    }
    let rel_count = db.rels.len() as f64;
    let careful = rel_count * (nodes.len() as f64 + layout.order.len() as f64 + 3.0 * rel_count) <= CARE_BUDGET;
    let covered = |taken: &[f64], x0: f64, y0: f64, x1: f64, y1: f64| -> f64 {
        let mut area = 0.0;
        let mut i = 0;
        while i < taken.len() {
            let w = js::min(x1, taken[i + 2]) - js::max(x0, taken[i]);
            if w > 0.0 {
                let h = js::min(y1, taken[i + 3]) - js::max(y0, taken[i + 1]);
                if h > 0.0 {
                    area += w * h;
                }
            }
            i += 4;
        }
        area
    };
    let offset = |value: Option<f64>| -> f64 {
        match value {
            Some(v) if v != 0.0 && !v.is_nan() => js::max(-MAX_OFFSET, js::min(MAX_OFFSET, v)),
            _ => 0.0,
        }
    };

    // Every line is drawn before any label is placed, so that labels can keep off the
    // arrowheads.
    let mut edges_out = String::new();
    let trim = marker_trim("arrow_point");
    let mut lanes = Lanes::default();
    for &g in &layout.order {
        if g != root {
            lanes.frame(&layout.groups[g].bx);
        }
    }
    let mut routes: Vec<Route> = Vec::with_capacity(db.rels.len());
    for (index, rel) in db.rels.iter().enumerate() {
        let a = ends[index * 2];
        let b = ends[index * 2 + 1];
        let st = user_style(None, rel.line_color.as_deref(), None);
        let color = match &st.stroke {
            Some(stroke) => esc(stroke).into_owned(),
            None => ctx.paint(Role::Arrow, "var(--_l)").to_string(),
        };
        let start_arrow = rel.rel_type == "birel" || rel.rel_type == "rel_b";
        let end_arrow = rel.rel_type != "rel_b";
        let reverse = a.id != b.id && pairs.get(&b.id).is_some_and(|s| s.contains(&a.id));
        let r = route(
            &a,
            &b,
            if reverse { PAIR_SHIFT } else { 0.0 },
            if careful { Some(nodes.as_slice()) } else { None },
            &mut lanes,
            if start_arrow { trim } else { 0.0 },
            if end_arrow { trim } else { 0.0 },
        );
        grow(js::min(r.x0, r.x1), js::min(r.y0, r.y1), js::max(r.x0, r.x1), js::max(r.y0, r.y1));
        if start_arrow && careful {
            taken.extend_from_slice(&[r.sx - ARROW, r.sy - ARROW, r.sx + ARROW, r.sy + ARROW]);
        }
        if end_arrow && careful {
            taken.extend_from_slice(&[r.ex - ARROW, r.ey - ARROW, r.ex + ARROW, r.ey + ARROW]);
        }
        svg_write!(
            edges_out,
            "<g class=\"pele-edge\" data-id=\"",
            esc_text(&format!("{}-{}", rel.from, rel.to)),
            "\"",
            st.line,
            "><path d=\"",
            r.d,
            "\"/>"
        )
        .unwrap();
        if start_arrow {
            svg_write!(edges_out, marker("arrow_point", r.sx, r.sy, r.sdx, r.sdy, &color)).unwrap();
        }
        if end_arrow {
            svg_write!(edges_out, marker("arrow_point", r.ex, r.ey, r.edx, r.edy, &color)).unwrap();
        }
        edges_out.push_str("</g>");
        routes.push(r);
    }

    // A boundary's heading goes at the left, the middle or the right of its top edge: the
    // first of them no line crosses.
    for k in 0..layout.order.len() {
        let g = layout.order[k];
        let group = &mut layout.groups[g];
        if !careful || group.head_w == 0.0 {
            continue;
        }
        let top = group.bx.y + HEAD_TOP;
        let bottom = group.bx.y + group.head - LANE;
        let mut fewest = f64::INFINITY;
        for x in [HEAD_LEFT, (group.bx.w - group.head_w) / 2.0, group.bx.w - HEAD_LEFT - group.head_w] {
            let mut count = 0.0;
            for r in &routes {
                if crosses(&r.points, group.bx.x + x - 4.0, top, group.bx.x + x + group.head_w + 4.0, bottom) {
                    count += 1.0;
                }
            }
            if count < fewest {
                fewest = count;
                group.head_x = x;
            }
            if count == 0.0 {
                break;
            }
        }
        taken.extend_from_slice(&[group.bx.x + group.head_x - 4.0, top, group.bx.x + group.head_x + group.head_w + 4.0, bottom]);
    }

    // Relation text is muted in Zebra; Pele lets the label inherit the text color.
    let rel_text_fill = if resolved { svg_format!(" fill=\"", theming.resolved.color(Role::MutedText), "\"") } else { String::new() };
    let mut labels_out = String::new();
    for (index, rel) in db.rels.iter().enumerate() {
        let r = &routes[index];
        let id = format!("{}-{}", rel.from, rel.to);
        let st = user_style(None, None, rel.text_color.as_deref());
        let text = if db.c4_type.as_deref() == Some("C4Dynamic") { format!("{}: {}", index + 1, rel.label) } else { rel.label.clone() };
        let label = layout_label(Some(&text), false, &m, small, r.room, 0);
        let techn_text = rel.techn.as_deref().filter(|t| !t.is_empty()).map(|t| format!("[{}]", t));
        let techn = layout_label(techn_text.as_deref(), false, &m, small, r.room, style::ITALIC);
        let lw = js::max(label.width, techn.width) + 8.0;
        let lh = label.height + techn.height;
        if lh == 0.0 {
            continue;
        }

        // The label sits on the line, or just beside it when another relation runs the
        // opposite way.
        let beside = r.nx.abs() * (lw / 2.0) + r.ny.abs() * (lh / 2.0) + 2.0;
        let aside = if r.lean != 0.0 && !r.lean.is_nan() { js::max(0.0, js::min(r.lean, beside - 14.0)) } else { beside };
        // Manual offsets are tuned to the original spacing. Reposition compact labels beside
        // their routed arrows so an offset cannot leave them floating outside a boundary.
        let mut dx = r.nx * aside + if tight { 0.0 } else { offset(rel.offset_x) };
        let mut dy = r.ny * aside + if tight { 0.0 } else { offset(rel.offset_y) };
        let mut at = 0.5;
        if careful {
            // Slide along the line to the spot that covers least. Offsets from the diagram are
            // kept unless a spot on the line itself covers less than any they lead to: they are
            // often tuned for Mermaid's spacing.
            let mut least = f64::INFINITY;
            let moved = dx != r.nx * aside || dy != r.ny * aside;
            let passes = if moved { 2 } else { 1 };
            let mut pass = 0;
            while pass < passes && least > 0.0 {
                let px = if pass != 0 { r.nx * aside } else { dx };
                let py = if pass != 0 { r.ny * aside } else { dy };
                for t in SLIDE {
                    let cx = r.x0 + (r.x1 - r.x0) * t + px;
                    let cy = r.y0 + (r.y1 - r.y0) * t + py;
                    let area = covered(&taken, cx - lw / 2.0, cy - lh / 2.0, cx + lw / 2.0, cy + lh / 2.0);
                    if area < least {
                        least = area;
                        at = t;
                        dx = px;
                        dy = py;
                    }
                    if area == 0.0 {
                        break;
                    }
                }
                pass += 1;
            }
        }
        let cx = r.x0 + (r.x1 - r.x0) * at + dx;
        let top = r.y0 + (r.y1 - r.y0) * at + dy - lh / 2.0;
        if careful {
            taken.extend_from_slice(&[cx - lw / 2.0, top, cx + lw / 2.0, top + lh]);
        }
        grow(cx - lw / 2.0, top, cx + lw / 2.0, top + lh);
        svg_write!(labels_out, "<g class=\"pele-edge-label\" data-id=\"", esc_text(&id), "\">").unwrap();
        if let Some(descr) = rel.descr.as_deref().filter(|d| !d.is_empty()) {
            svg_write!(labels_out, "<title>", esc_text(descr), "</title>").unwrap();
        }
        svg_write!(
            labels_out,
            "<rect x=\"",
            num(cx - lw / 2.0),
            "\" y=\"",
            num(top),
            "\" width=\"",
            num(lw),
            "\" height=\"",
            num(lh),
            "\" rx=\"3\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\"/>"
        )
        .unwrap();
        let body = label_svg(&label, cx, top + label.height / 2.0, &format!("{}{}", rel_text_fill, st.text), None).to_string()
            + &label_svg(
                &techn,
                cx,
                top + label.height + techn.height / 2.0,
                &svg_format!(" class=\"pele-c4-techn\" fill=\"", ctx.paint(Role::MutedText, "var(--_m)"), "\"", st.text),
                None,
            )
            .to_string();
        labels_out.push_str(&link(body, rel.link.as_deref(), &id, &mut links, options));
        labels_out.push_str("</g>");
    }

    let mut clusters = String::new();
    let mut nodes_out = String::new();
    // Boundary titles are secondary text in Zebra; Pele lets the name inherit the text color.
    let heading_fill = if resolved { svg_format!(" fill=\"", theming.resolved.color(Role::SecondaryText), "\"") } else { String::new() };
    let small_s = js::to_string(small);
    let type_size = js::to_string(small - 1.0);
    for &g in &layout.order {
        let group = &layout.groups[g];
        if g != root {
            let boundary = &db.boundaries[group.boundary];
            let el = &boundary.element;
            let st = user_style(el.bg_color.as_deref(), el.border_color.as_deref(), el.font_color.as_deref());
            let muted = ctx.paint(Role::SecondaryText, "var(--_m)");
            let x = group.bx.x + group.head_x;
            let mut y = group.bx.y + HEAD_TOP;
            let mut text = left_label_svg(
                &group.labels[0],
                x,
                y,
                &svg_format!(" class=\"pele-cluster-label\" font-weight=\"var(--_hw)\"", heading_fill, st.text),
            );
            y += group.labels[0].height;
            push_left_label_svg(
                &mut text,
                &group.labels[1],
                x,
                y,
                &svg_format!(" class=\"pele-c4-type\" font-size=\"", small_s, "\" fill=\"", muted, "\"", st.text),
            );
            y += group.labels[1].height;
            push_left_label_svg(
                &mut text,
                &group.labels[2],
                x,
                y,
                &svg_format!(" class=\"pele-c4-descr\" font-size=\"", small_s, "\" fill=\"", muted, "\"", st.text),
            );
            let node = boundary.node_type.as_deref().is_some_and(|t| !t.is_empty());
            svg_write!(
                clusters,
                "<g class=\"pele-cluster pele-c4-",
                if node { "node" } else { "boundary" },
                "\" data-id=\"",
                esc_text(&el.alias),
                "\"><rect x=\"",
                num(group.bx.x),
                "\" y=\"",
                num(group.bx.y),
                "\" width=\"",
                num(group.bx.w),
                "\" height=\"",
                num(group.bx.h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                ctx.paint(Role::GroupFill, "none"),
                "\" stroke=\"",
                ctx.paint(Role::GroupStroke, "var(--_b)"),
                "\"",
                if node { "" } else { " stroke-dasharray=\"6 4\"" },
                st.shape,
                "/>"
            )
            .unwrap();
            clusters.push_str(&link(text, el.link.as_deref(), &el.alias, &mut links, options));
            clusters.push_str("</g>");
        }
        for node in &group.nodes {
            let shape = &db.shapes[node.shape];
            let el = &shape.element;
            let external = shape.kind.starts_with("external");
            let st = user_style(el.bg_color.as_deref(), el.border_color.as_deref(), el.font_color.as_deref());
            let (fill, stroke_color) = if resolved {
                shape_colors(ctx, &shape.kind, external)
            } else {
                ((if external { "var(--_bg)" } else { "var(--_s)" }).to_string(), "var(--_b)".to_string())
            };
            let stroke = svg_format!(" stroke=\"", stroke_color, "\"", if external { " stroke-dasharray=\"4 3\"" } else { "" });
            let Bx { w, h, form, .. } = node.bx;
            let labels = &node.labels;
            let dx = if form == "h-cyl" { -queue_rx(h) / 2.0 } else { 0.0 };
            let dy = if form == "person" {
                HEAD / 2.0
            } else if form == "cyl" {
                cyl_ry(w) / 2.0
            } else {
                0.0
            };
            // Zebra writes all of a shape's text in the text color.
            let muted = svg_format!(" fill=\"", ctx.paint(Role::Text, "var(--_m)"), "\"", st.text);
            let mut y = dy - node.text_h / 2.0;
            let mut text = label_svg(
                &labels[0],
                dx,
                y + labels[0].height / 2.0,
                &svg_format!(" class=\"pele-c4-type\" font-size=\"", type_size, "\"", muted),
                None,
            )
            .to_string();
            y += labels[0].height;
            write!(
                text,
                "{}",
                label_svg(
                    &labels[1],
                    dx,
                    y + labels[1].height / 2.0,
                    &svg_format!(" class=\"pele-label\" font-weight=\"var(--_hw)\"", st.text),
                    None,
                )
            )
            .unwrap();
            y += labels[1].height;
            svg_write!(
                text,
                label_svg(
                    &labels[2],
                    dx,
                    y + labels[2].height / 2.0,
                    &svg_format!(" class=\"pele-c4-techn\" font-size=\"", small_s, "\"", muted),
                    None,
                )
            )
            .unwrap();
            y += labels[2].height + 4.0;
            write!(
                text,
                "{}",
                label_svg(
                    &labels[3],
                    dx,
                    y + labels[3].height / 2.0,
                    &svg_format!(" class=\"pele-c4-descr\" font-size=\"", small_s, "\"", st.text),
                    None,
                )
            )
            .unwrap();
            let outline =
                draw_shape(form, w, h, &svg_format!(" fill=\"", fill, "\"", stroke, st.shape), &format!("{}{}", stroke, st.line), theming);
            svg_write!(
                nodes_out,
                "<g class=\"pele-node pele-shape-",
                form,
                " pele-c4-",
                shape.kind,
                if external { " pele-c4-external" } else { "" },
                "\" data-id=\"",
                esc_text(&el.alias),
                "\" transform=\"translate(",
                num(node.bx.x + w / 2.0),
                ",",
                num(node.bx.y + h / 2.0),
                ")\">"
            )
            .unwrap();
            nodes_out.push_str(&link(outline + &text, el.link.as_deref(), &el.alias, &mut links, options));
            nodes_out.push_str("</g>");
        }
    }

    let title = layout_label(db.title.as_deref(), false, &m, size, if tight { title_room(options) } else { 4000.0 }, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(max_x - min_x, title.width);
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();
    let ox = pad - min_x + (inner - (max_x - min_x)) / 2.0;
    let oy = pad + title_height - min_y;

    let mut content =
        label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
    svg_write!(content, "<g transform=\"translate(", num(ox), ",", num(oy), ")\">").unwrap();
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
    if !nodes_out.is_empty() {
        svg_write!(content, "<g class=\"pele-nodes\">", nodes_out, "</g>").unwrap();
    }
    if !labels_out.is_empty() {
        svg_write!(content, "<g class=\"pele-edge-labels\" font-size=\"", small_s, "\">", labels_out, "</g>").unwrap();
    }
    content.push_str("</g>");

    let acc = Accessible { acc_title: db.acc_title.as_deref(), acc_descr: db.acc_descr.as_deref() };
    let svg = svg_document("c4", width, height, size, ctx, acc, &content);
    Ok((Rendered { svg, width, height, links }, lanes.squeezed))
}
