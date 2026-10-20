//! Mind map rendering. Ported from Pele `src/diagrams/mindmap/render.ts`.

use super::MindmapModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{title_room, turn_to_fit};
use crate::diagrams::common::icon::icon_svg;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::shapes::{draw_shape, shape_inset, shape_size};
use crate::svg::theme::class_names;
use crate::text::{layout_render_label as layout_label, style, Icons, RenderLabel as Label};
use crate::theme::color::mix_hex;
use crate::theme::Role;
use crate::util::js;

/// Indexed by Mermaid's node type. A node without delimiters is text on a line, except the root.
const SHAPES: [&str; 7] = ["", "rounded", "rect", "circle", "cloud", "bang", "hex"];
const ROOT_SHAPE: &str = "stadium";

const GAP_X: f64 = 36.0;
const GAP_Y: f64 = 8.0;
/// Between two subtrees, of which at least one has children of its own.
const GROUP_GAP: f64 = 18.0;
const BRANCH_GAP_X: f64 = 52.0;
const BRANCH_GAP_Y: f64 = 28.0;
const TEXT_PAD: f64 = 6.0;
const PILL_PAD: f64 = 12.0;
const TINT: &str = "0.22";
const ICON_GAP: f64 = 6.0;
/// Running down, each level is set in by this much, and links leave a node this far from its left edge.
const INDENT: f64 = 28.0;
const STEM: f64 = 12.0;
const BEND: f64 = 8.0;
const ROW_GAP: f64 = 10.0;

/// A label with each line starting at `left`, for an outline, where centered lines would look ragged.
fn left_label(label: &Label, left: f64, cy: f64, icons: Option<&Icons>) -> String {
    if label.lines().len() < 2 {
        return label_svg(label, left + label.width / 2.0, cy, " class=\"pele-label\"", icons).to_string();
    }
    let mut out = String::new();
    let count = label.lines().len() as f64;
    for (k, line) in label.lines().iter().enumerate() {
        let width = label.widths()[k];
        let one = Label::from_lines(vec![line.to_vec()], vec![width], width, label.line_height, label.size, label.line_height);
        svg_write!(
            out,
            label_svg(&one, left + width / 2.0, cy + (k as f64 - (count - 1.0) / 2.0) * label.line_height, " class=\"pele-label\"", icons,)
        )
        .unwrap();
    }
    out
}

/// Zebra's `tonal_scale`: `count` mixes of `foreground` into `background`, from `start` to `end`.
fn tonal(foreground: &str, background: &str, start: f32, end: f32, count: usize, index: usize) -> String {
    let position = if count <= 1 { 0.0 } else { index as f32 / (count - 1) as f32 };
    mix_hex(foreground, background, start + (end - start) * position)
}

/// The colors of one first-level branch.
struct Branch {
    /// Links of the branch.
    line: String,
    /// The attributes of a node's outline: fill and stroke.
    tint: String,
    /// The stroke of lines inside a node's shape.
    inner: String,
    /// Resolved mode: the fill and icon color the node's group gives its label.
    text: String,
}

/// In CSS mode a branch takes Pele's series color for its links, and a tint of it for its
/// nodes. In resolved mode it takes Zebra's mind map section colors (`apply_layout_config`):
/// links and outlines in the line color, nodes filled with the section color, which is the
/// chart palette when the caller gave series colors and a tonal scale of the accent otherwise,
/// and labels in a color that reads on that fill. Zebra fills sections opaquely, and its label
/// colors are chosen for an opaque fill, so the tint's opacity is left out there.
fn branch(ctx: &RenderCtx, section: i64) -> Branch {
    if !ctx.resolved() {
        let color = ctx.series(section).into_owned();
        return Branch {
            tint: svg_format!(" fill=\"", color, "\" fill-opacity=\"", TINT, "\" stroke=\"", color, "\""),
            inner: svg_format!(" stroke=\"", color, "\""),
            line: color,
            text: String::new(),
        };
    }
    let t = &ctx.theming.resolved;
    let line = t.color(Role::Line).to_string();
    let (fill, text) = if t.series.is_some() {
        let fill = ctx.series(section).into_owned();
        let text = ctx.theming.text_on(&fill, "").into_owned();
        (fill, text)
    } else {
        (tonal(&t.accent, &t.background, 0.04, 0.13, 12, section.rem_euclid(12) as usize), t.color(Role::Text).to_string())
    };
    Branch {
        tint: svg_format!(" fill=\"", fill, "\" stroke=\"", line, "\""),
        inner: svg_format!(" stroke=\"", line, "\""),
        line,
        text: svg_format!(" fill=\"", text, "\" color=\"", text, "\""),
    }
}

pub(crate) fn render(model: &MindmapModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, down)))
}

/// Stacks subtrees top to bottom and returns the height at which their parent's anchor
/// belongs, setting each subtree's offset `rel` and the height `stacked` they take.
#[allow(clippy::too_many_arguments)]
fn stack(
    model: &MindmapModel,
    list: &[usize],
    from: usize,
    to: usize,
    gap: f64,
    extent: &[f64],
    own: &[f64],
    anchor: &[f64],
    rel: &mut [f64],
    stacked: &mut f64,
) -> f64 {
    let nodes = &model.nodes;
    let mut y = 0.0;
    let mut last = 0.0;
    for k in from..to {
        let c = list[k];
        rel[c] = y;
        let grouped = !nodes[c].children.is_empty() || (k + 1 < to && !nodes[list[k + 1]].children.is_empty());
        last = if grouped { js::max(gap, GROUP_GAP) } else { gap };
        y += extent[c] + last;
    }
    *stacked = y - last;
    let a = list[from];
    let b = list[to - 1];
    (rel[a] + own[a] + anchor[a] + rel[b] + own[b] + anchor[b]) / 2.0
}

/// `trunk[p] || fallback`
fn or(value: f64, fallback: f64) -> f64 {
    if value != 0.0 && !value.is_nan() {
        value
    } else {
        fallback
    }
}

/// `down` draws the map as an outline: the root at the top, and each level set in under its parent.
/// It is far narrower than branches spreading to both sides, and as tall as it needs to be.
fn draw(model: &MindmapModel, config: &Config, ctx: &RenderCtx, down: bool) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);
    let icons = ctx.icons();
    let wrap = match config.lookup("mindmap", "maxNodeWidth") {
        Some(ConfigValue::Number(w)) if *w > 0.0 => js::min(*w, 2000.0),
        _ => 200.0,
    };
    let glyph = size * 1.25;

    let nodes = &model.nodes;
    let n = nodes.len();
    let mut parent = vec![0usize; n];
    let mut depth = vec![0usize; n];
    for i in 0..n {
        for &child in &nodes[i].children {
            parent[child] = i;
            depth[child] = depth[i] + 1;
        }
    }
    // An outline has the width the host gives it, less what its level is set in.
    let room = match options.max_width {
        Some(a) if down && a > 0.0 => a - 2.0 * pad,
        _ => f64::INFINITY,
    };
    let mut labels: Vec<Label> = Vec::with_capacity(n);
    let mut shapes: Vec<&str> = Vec::with_capacity(n);
    let mut w = vec![0.0; n];
    let mut h = vec![0.0; n];
    let mut dy = vec![0.0; n];
    // Where links meet the node, measured down from its top: its middle.
    let mut anchor = vec![0.0; n];
    // Whether the host has the node's icon. One it does not have takes no room.
    let mut drawn = vec![false; n];

    for i in 0..n {
        let node = &nodes[i];
        drawn[i] = node.icon.as_deref().is_some_and(|icon| !m.icon_markup(icon).is_empty());
        let fit =
            js::min(wrap, js::max(72.0, room - depth[i] as f64 * INDENT - 2.0 * TEXT_PAD - if drawn[i] { glyph + ICON_GAP } else { 0.0 }));
        let label = layout_label(Some(&node.descr), true, &m, size, fit, 0);
        let tw = label.width + if drawn[i] { glyph + if label.width > 0.0 { ICON_GAP } else { 0.0 } } else { 0.0 };
        let th = js::max(label.height, js::round(size * 1.5));
        let mut shape = match SHAPES.get(node.kind as usize).copied().unwrap_or("") {
            "" if i == 0 => ROOT_SHAPE,
            s => s,
        };
        // In an outline the root heads a column, and the trunk drops from under its left end. A circle
        // or a cloud would stand wide of that, so the root is a pill there unless it is a box already.
        if down && i == 0 && shape != "rect" && shape != "rounded" {
            shape = ROOT_SHAPE;
        }
        shapes.push(shape);
        if shape.is_empty() {
            // A node with no shape of its own is plain text when it ends a branch, and text in a
            // tinted pill when the branch goes on from it, which sets the levels apart.
            let pill = !node.children.is_empty();
            w[i] = tw + 2.0 * if pill { PILL_PAD } else { TEXT_PAD };
            h[i] = th + if pill { 6.0 } else { 0.0 };
            anchor[i] = h[i] / 2.0;
        } else {
            let s = shape_size(shape, tw, th);
            w[i] = s.w;
            h[i] = s.h;
            dy[i] = s.dy;
            anchor[i] = s.h / 2.0;
        }
        labels.push(label);
    }

    // Each subtree takes a band of height `extent`. `rel` is a subtree's offset among its siblings,
    // `own` the node's offset inside its band, and `kids` the offset of its children's bands.
    let mut extent = vec![0.0; n];
    let mut rel = vec![0.0; n];
    let mut own = vec![0.0; n];
    let mut kids = vec![0.0; n];
    let mut stacked = 0.0;

    // Nodes are in depth-first order, so going backwards visits children before their parent.
    for i in (1..n).rev() {
        let children = &nodes[i].children;
        if children.is_empty() {
            extent[i] = h[i];
            continue;
        }
        let top = stack(model, children, 0, children.len(), GAP_Y, &extent, &own, &anchor, &mut rel, &mut stacked) - anchor[i];
        let low = js::min(0.0, top);
        extent[i] = js::max(stacked, top + h[i]) - low;
        own[i] = top - low;
        kids[i] = -low;
    }

    let mut x = vec![0.0; n];
    let mut y = vec![0.0; n];
    let mut band = vec![0.0; n];
    let mut side = vec![0.0f64; n];
    // Running down: how far the trunk under each node has been drawn.
    let mut trunk = vec![0.0; n];

    if down {
        // Nodes are in depth-first order, which is the order of an outline's rows.
        let mut row = 0.0;
        for i in 0..n {
            if i > 0 && parent[i] == 0 {
                row += ROW_GAP;
            }
            x[i] = depth[i] as f64 * INDENT + w[i] / 2.0;
            y[i] = row;
            row += h[i] + ROW_GAP;
        }
    } else if n > 0 {
        // The first branches go right and the rest left, split where the two sides are closest in height.
        let branches = &nodes[0].children;
        let count = branches.len();
        let mut total = 0.0;
        for &b in branches {
            total += extent[b];
        }
        let mut split = count;
        let mut best = f64::INFINITY;
        let mut right = 0.0;
        for k in 1..count {
            right += extent[branches[k - 1]];
            let diff = (2.0 * right - total + (2.0 * k as f64 - count as f64) * BRANCH_GAP_Y).abs();
            if diff < best {
                best = diff;
                split = k;
            }
        }
        let (mut from, mut to, mut dir) = (0, split, 1.0);
        while from < count {
            let middle = stack(model, branches, from, to, BRANCH_GAP_Y, &extent, &own, &anchor, &mut rel, &mut stacked);
            for &c in &branches[from..to] {
                band[c] = rel[c] - middle;
                side[c] = dir;
            }
            from = to;
            to = count;
            dir = -1.0;
        }
        y[0] = -h[0] / 2.0;
        for i in 0..n {
            if i > 0 {
                y[i] = band[i] + own[i];
            }
            for &c in &nodes[i].children {
                if i > 0 {
                    side[c] = side[i];
                    band[c] = band[i] + kids[i] + rel[c];
                }
                let gap = if i > 0 { GAP_X } else { BRANCH_GAP_X };
                x[c] = x[i] + side[c] * (w[i] / 2.0 + gap + w[c] / 2.0);
            }
        }
    }

    let resolved = ctx.resolved();
    let mut min_x: f64 = 0.0;
    let mut max_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    let mut out = String::new();
    let mut edges = String::new();
    let mut body = String::new();
    let mut colors: Option<Branch> = None;
    let flush = |out: &mut String, edges: &mut String, body: &mut String, colors: &Option<Branch>, section: Option<usize>| {
        if body.is_empty() {
            return;
        }
        let line = colors.as_ref().map_or("", |c| c.line.as_str());
        svg_write!(
            out,
            "<g class=\"pele-branch pele-section-",
            section.unwrap_or(0),
            "\"><g class=\"pele-edges\" fill=\"none\" stroke=\"",
            line,
            "\" stroke-width=\"1.5\">",
            edges,
            "</g>",
            body,
            "</g>"
        )
        .unwrap();
        edges.clear();
        body.clear();
    };

    // The root is drawn last, on top of the links that leave it.
    for k in 1..=n {
        let i = k % n;
        let node = &nodes[i];
        let shape = shapes[i];
        let label = &labels[i];
        let cx = x[i];
        let top = y[i];
        min_x = js::min(min_x, cx - w[i] / 2.0);
        max_x = js::max(max_x, cx + w[i] / 2.0);
        min_y = js::min(min_y, top);
        max_y = js::max(max_y, top + h[i]);

        if i == 0 || parent[i] == 0 {
            flush(&mut out, &mut edges, &mut body, &colors, nodes[k - 1].section);
        }
        if i > 0 && down {
            if parent[i] == 0 {
                colors = Some(branch(ctx, node.section.unwrap_or(0) as i64));
            }
            let p = parent[i];
            // The link drops from under its parent and turns in to the node's left. It starts where
            // the link to the sibling above turned in, so each stretch of a shared trunk has one color.
            // Under the root's rounded end, the trunk starts where that end has come down to the bottom.
            let stem = if p == 0 { js::min(js::max(STEM, h[0] / 2.0), INDENT - BEND) } else { STEM };
            let x1 = x[p] - w[p] / 2.0 + js::min(stem, w[p] / 2.0);
            let y1 = or(trunk[p], y[p] + h[p]);
            let left = cx - w[i] / 2.0;
            let x2 = left + if !shape.is_empty() { shape_inset(shape, w[i], h[i], 3, 0.0) } else { 0.0 };
            let y2 = top + anchor[i];
            let bend = js::max(0.0, js::min(js::min(BEND, x2 - x1), y2 - y1));
            // The next link carries on from where this one leaves the trunk, so the trunk has no gap.
            trunk[p] = y2 - bend;
            svg_write!(
                edges,
                "<path class=\"pele-edge\" d=\"M",
                num(x1),
                ",",
                num(y1),
                "V",
                num(y2 - bend),
                "Q",
                num(x1),
                ",",
                num(y2),
                " ",
                num(x1 + bend),
                ",",
                num(y2),
                "H",
                num(x2),
                "\"/>"
            )
            .unwrap();
        } else if i > 0 {
            if parent[i] == 0 {
                colors = Some(branch(ctx, node.section.unwrap_or(0) as i64));
            }
            let p = parent[i];
            let dir = side[i];
            let from = if dir > 0.0 { 1 } else { 3 };
            let x1 = x[p] + dir * (w[p] / 2.0 - if !shapes[p].is_empty() { shape_inset(shapes[p], w[p], h[p], from, 0.0) } else { 0.0 });
            let y1 = y[p] + anchor[p];
            let near = cx - (dir * w[i]) / 2.0;
            let x2 = near + if !shape.is_empty() { dir * shape_inset(shape, w[i], h[i], 4 - from, 0.0) } else { 0.0 };
            let y2 = top + anchor[i];
            let mid = num((x1 + x2) / 2.0);
            svg_write!(
                edges,
                "<path class=\"pele-edge\" d=\"M",
                num(x1),
                ",",
                num(y1),
                "C",
                mid,
                ",",
                num(y1),
                " ",
                mid,
                ",",
                num(y2),
                " ",
                num(x2),
                ",",
                num(y2),
                "\"/>"
            )
            .unwrap();
        }

        let pill = shape.is_empty() && !node.children.is_empty();
        // Every node off the root is filled with a tint of its branch's color and outlined in it,
        // whether its shape was given in the source or is the pill of a node that branches.
        // The root is neutral.
        let empty = String::new();
        let (tint, inner_line, text) = match &colors {
            Some(c) if i > 0 => (&c.tint, &c.inner, &c.text),
            _ => (&empty, &empty, &empty),
        };
        let mut inner = String::new();
        // In resolved mode the group of a filled node gives its label and icon their color.
        let mut paint = String::new();
        if !shape.is_empty() {
            if i > 0 {
                inner = draw_shape(shape, w[i], h[i], tint, inner_line, ctx.theming);
                if resolved {
                    paint = text.clone();
                }
            } else {
                // Zebra fills the root with a light mix of the accent and writes its text in the
                // text color.
                let fill = if resolved {
                    let t = &ctx.theming.resolved;
                    mix_hex(&t.accent, &t.background, 0.16)
                } else {
                    "var(--_s)".to_string()
                };
                let stroke = ctx.paint(Role::NodeStroke, "var(--_b)");
                inner = draw_shape(
                    shape,
                    w[i],
                    h[i],
                    &svg_format!(" fill=\"", fill, "\" stroke=\"", stroke, "\""),
                    &svg_format!(" stroke=\"", stroke, "\""),
                    ctx.theming,
                );
                if resolved {
                    let color = ctx.paint(Role::Text, "");
                    paint = svg_format!(" fill=\"", color, "\" color=\"", color, "\"");
                }
            }
        } else if pill {
            inner = svg_format!(
                "<rect x=\"",
                num(-w[i] / 2.0),
                "\" y=\"",
                num(-h[i] / 2.0),
                "\" width=\"",
                num(w[i]),
                "\" height=\"",
                num(h[i]),
                "\" rx=\"",
                num(h[i] / 2.0),
                "\"",
                tint,
                "/>"
            );
            if resolved {
                paint = text.clone();
            }
        }
        let mut shift = 0.0;
        if let Some(icon) = node.icon.as_deref().filter(|_| drawn[i]) {
            shift = (glyph + if label.width > 0.0 { ICON_GAP } else { 0.0 }) / 2.0;
            inner.push_str(&icon_svg(icon, -shift - label.width / 2.0, dy[i] - glyph / 2.0, glyph, icons));
        }
        let kind = if !shape.is_empty() {
            shape
        } else if pill {
            "branch"
        } else {
            "text"
        };
        let group = svg_format!(
            "<g class=\"pele-node pele-shape-",
            kind,
            if i > 0 { "" } else { " pele-root" },
            class_names(node.class.as_deref().unwrap_or("")),
            "\" data-id=\"",
            esc_text(&node.node_id),
            "\" transform=\"translate(",
            num(cx),
            ",",
            num(top + h[i] / 2.0),
            ")\"",
            paint,
            ">",
            inner,
            if down && shape.is_empty() {
                left_label(label, shift - label.width / 2.0, dy[i], icons)
            } else {
                label_svg(label, shift, dy[i], " class=\"pele-label\"", icons).to_string()
            },
            "</g>"
        );
        if i > 0 {
            body.push_str(&group);
        } else {
            out.push_str(&group);
        }
    }

    let title = layout_label(model.title.as_deref(), false, &m, size, title_room(options), style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let width = js::max(max_x - min_x, title.width);
    let total_width = (width + 2.0 * pad).ceil();
    let total_height = (max_y - min_y + title_height + 2.0 * pad).ceil();
    let mut content =
        label_svg(&title, total_width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None)
            .to_string();
    svg_write!(
        content,
        "<g transform=\"translate(",
        num(pad - min_x + (width - max_x + min_x) / 2.0),
        ",",
        num(pad + title_height - min_y),
        ")\">",
        out,
        "</g>"
    )
    .unwrap();

    Rendered {
        svg: svg_document("mindmap", total_width, total_height, size, ctx, Accessible::default(), &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
