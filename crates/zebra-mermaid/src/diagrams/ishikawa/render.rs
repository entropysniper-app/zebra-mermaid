//! Ishikawa rendering. Ported from Pele `src/diagrams/ishikawa/render.ts`.

use super::IshikawaModel;
use crate::config::Config;
use crate::diagrams::common::fit_width::{title_room, turn_to_fit};
use crate::diagrams::common::literal_label::break_render_label as break_label;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::edges::{marker, ARROW};
use crate::svg::root::{svg_document, Accessible};
use crate::text::{layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math};

/// How far a bone moves toward the tail for each unit it rises from the spine.
const SLANT: f64 = 0.5;
const FIRST_ROW: f64 = 14.0;
const ROW_GAP: f64 = 6.0;
const BONE_END: f64 = 8.0;
const TWIG_STEP: f64 = 22.0;
const STUB: f64 = 18.0;
const TEXT_GAP: f64 = 5.0;
const PAIR_GAP: f64 = 26.0;
const HEAD_GAP: f64 = 34.0;
const TAIL: f64 = 28.0;
const BOX_PAD_X: f64 = 10.0;
const BOX_PAD_Y: f64 = 5.0;

/// `Math.hypot(SLANT, 1)`.
fn length() -> f64 {
    math::hypot(SLANT, 1.0)
}

/// A cause below a category. Bones at even depths run level, from the slanted bone above them
/// toward the tail; bones at odd depths slant away from the spine, from the level bone above
/// them.
struct Bone {
    label: Label,
    depth: usize,
    /// Index of the parent bone, `None` under the category itself.
    parent: Option<usize>,
    /// Its place among its siblings, counted from the spine outward.
    slot: usize,
    kids: usize,
    /// How far its row is from the spine.
    rise: f64,
    /// Where it meets its parent, measured level from the category's bone: 0 on it, negative
    /// toward the tail.
    at: f64,
    /// Where its text ends.
    text_end: f64,
}

struct Category {
    node: usize,
    label: Label,
    box_w: f64,
    box_h: f64,
    bones: Vec<Bone>,
    /// The length of its bone, as distance from the spine.
    rise: f64,
    /// How far its causes reach toward the tail, measured level from its bone.
    reach: f64,
}

struct Frame {
    node: usize,
    depth: usize,
    parent: Option<usize>,
    slot: usize,
    /// Set when the bone was made and only its row is still to come.
    bone: Option<usize>,
}

/// Gives every cause of a category a row of its own, so that no two texts can meet, and
/// measures how far the whole of it reaches. A level bone takes the row nearest the spine and
/// its twigs the rows beyond; a slanted twig takes the row beyond everything that hangs off it.
/// The outline may be nested as deep as its indentation goes, so the walk keeps a stack of its
/// own.
fn arrange(model: &IshikawaModel, node: usize, up: bool, m: &Measure, size: f64, small: f64) -> Category {
    let nodes = &model.nodes;
    let label = layout_label(Some(&nodes[node].text), false, m, size, 180.0, 0);
    let line_height = js::round(small * 1.5);
    let mut bones: Vec<Bone> = Vec::new();
    let mut rows: Vec<usize> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    // Above the spine the first cause goes farthest out, so that the causes read from the top
    // down.
    let push = |stack: &mut Vec<Frame>, children: &[usize], depth: usize, parent: Option<usize>| {
        let n = children.len();
        for slot in (0..n).rev() {
            stack.push(Frame { node: children[if up { n - 1 - slot } else { slot }], depth, parent, slot, bone: None });
        }
    };
    push(&mut stack, &nodes[node].children, 2, None);
    while let Some(mut frame) = stack.pop() {
        if let Some(bone) = frame.bone {
            rows.push(bone);
            continue;
        }
        let index = bones.len();
        bones.push(Bone {
            label: layout_label(Some(&nodes[frame.node].text), false, m, small, 200.0, 0),
            depth: frame.depth,
            parent: frame.parent,
            slot: frame.slot,
            kids: nodes[frame.node].children.len(),
            rise: 0.0,
            at: 0.0,
            text_end: 0.0,
        });
        let (child, depth) = (frame.node, frame.depth);
        if depth.is_multiple_of(2) {
            rows.push(index);
        } else {
            frame.bone = Some(index);
            stack.push(frame);
        }
        push(&mut stack, &nodes[child].children, depth + 1, Some(index));
    }

    let mut rise = FIRST_ROW;
    for &index in &rows {
        let height = js::max(bones[index].label.height, line_height) + ROW_GAP;
        bones[index].rise = rise + height / 2.0;
        rise += height;
    }
    let box_w = label.width + 2.0 * BOX_PAD_X;
    let mut reach = box_w / 2.0;
    for k in 0..bones.len() {
        let parent = bones[k].parent.map(|p| (bones[p].at, bones[p].kids));
        let bone = &mut bones[k];
        if bone.depth.is_multiple_of(2) {
            bone.at = parent.map_or(0.0, |(at, _)| at);
            let span = if bone.kids > 0 { (bone.kids as f64 + 1.0) * TWIG_STEP } else { STUB };
            bone.text_end = bone.at - span - TEXT_GAP;
        } else {
            // Twigs stand along their level bone, the one nearest the spine nearest the tail.
            let (at, kids) = parent.unwrap_or((0.0, 0));
            bone.at = at - (kids as f64 - bone.slot as f64) * TWIG_STEP;
            bone.text_end = bone.at - TEXT_GAP;
        }
        reach = js::max(reach, bone.label.width - bone.text_end);
    }
    let box_h = label.height + 2.0 * BOX_PAD_Y;
    Category { node, label, box_w, box_h, bones, rise: rise + BONE_END, reach }
}

pub(crate) fn render(model: &IshikawaModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, down)))
}

fn draw(model: &IshikawaModel, _config: &Config, ctx: &RenderCtx, down: bool) -> Rendered {
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    if down {
        return vertical_causes(model, ctx);
    }
    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, 4000.0, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let length = length();
    let line = ctx.paint(Role::Line, "var(--_l)");
    let arrow = ctx.paint(Role::Arrow, "var(--_l)");
    let node_fill = ctx.paint(Role::NodeFill, "var(--_s)");
    let node_stroke = ctx.paint(Role::NodeStroke, "var(--_b)");

    let mut body = String::new();
    let mut min_x: f64 = 0.0;
    let mut max_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    if let Some(root) = model.root() {
        let head = layout_label(Some(&root.text), false, &m, size, 200.0, style::BOLD);
        let head_h = js::max(head.height + 24.0, 48.0);
        let head_w = head.width + 24.0 + head_h / 2.0;
        let mut categories: Vec<Category> =
            root.children.iter().enumerate().map(|(i, &node)| arrange(model, node, i.is_multiple_of(2), &m, size, small)).collect();
        // Even a category without causes has a bone long enough to carry its name clear of the
        // head.
        for category in &mut categories {
            category.rise = js::max(category.rise, head_h / 2.0 + 14.0);
        }

        // Categories meet the spine in pairs, one above and one below, each pair as far toward
        // the tail from the one before as the wider of the two needs.
        let mut xs: Vec<f64> = Vec::new();
        let mut x = -HEAD_GAP;
        let mut i = 0;
        while i < categories.len() {
            if i > 0 {
                let mut room: f64 = 0.0;
                for side in 0..2 {
                    if let Some(before) = categories.get(i - 2 + side) {
                        let next = categories.get(i + side).map_or(0.0, |next| next.box_w / 2.0);
                        room = js::max(room, before.reach + next);
                    }
                }
                x -= room + PAIR_GAP;
            }
            xs.push(x);
            xs.push(x);
            i += 2;
        }
        let tail = if !categories.is_empty() { x - TAIL } else { -HEAD_GAP };

        max_x = head_w;
        min_x = tail;
        min_y = -head_h / 2.0;
        max_y = head_h / 2.0;
        let mut groups = String::new();
        for (i, category) in categories.iter().enumerate() {
            let sign: f64 = if i.is_multiple_of(2) { -1.0 } else { 1.0 };
            let ax = xs[i];
            let px = |at: f64, rise: f64| ax + at - rise * SLANT;
            let end_x = px(0.0, category.rise);
            let end_y = sign * category.rise;
            // The bone stops short of the spine to leave its arrowhead a clean point.
            let trim = ARROW - 1.0;
            let mut d =
                svg_format!("M", num(ax - (SLANT / length) * trim), ",", num((sign * trim) / length), "L", num(end_x), ",", num(end_y));
            let mut texts = String::new();
            for bone in &category.bones {
                let y = sign * bone.rise;
                if bone.depth.is_multiple_of(2) {
                    svg_write!(d, "M", num(px(bone.at, bone.rise)), ",", num(y), "H", num(px(bone.text_end + TEXT_GAP, bone.rise)))
                        .unwrap();
                } else {
                    let from = bone.parent.map_or(0.0, |p| category.bones[p].rise);
                    svg_write!(d, "M", num(px(bone.at, from)), ",", num(sign * from), "L", num(px(bone.at, bone.rise)), ",", num(y))
                        .unwrap();
                }
                let right = px(bone.text_end, bone.rise);
                // Causes below the first level are secondary.
                let fill =
                    if bone.depth > 2 { svg_format!(" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"") } else { String::new() };
                svg_write!(texts, label_svg(&bone.label, right - bone.label.width / 2.0, y, &fill, None)).unwrap();
                min_x = js::min(min_x, right - bone.label.width);
            }
            let box_y = end_y + sign * (category.box_h / 2.0);
            min_x = js::min(min_x, end_x - category.box_w / 2.0);
            max_x = js::max(max_x, end_x + category.box_w / 2.0);
            min_y = js::min(min_y, box_y - category.box_h / 2.0);
            max_y = js::max(max_y, box_y + category.box_h / 2.0);
            svg_write!(groups, "<g class=\"pele-ishikawa-category\" data-id=\"", esc_text(&model.nodes[category.node].text), "\">")
                .unwrap();
            svg_write!(groups, "<path class=\"pele-ishikawa-bones\" d=\"", d, "\" fill=\"none\" stroke=\"", line, "\"/>").unwrap();
            svg_write!(groups, marker("arrow_point", ax, 0.0, SLANT / length, -sign / length, arrow)).unwrap();
            svg_write!(
                groups,
                "<rect x=\"",
                num(end_x - category.box_w / 2.0),
                "\" y=\"",
                num(box_y - category.box_h / 2.0),
                "\" width=\"",
                num(category.box_w),
                "\" height=\"",
                num(category.box_h),
                "\"",
                crate::svg::theme::rounded(&svg_format!(" fill=\"", node_fill, "\" stroke=\"", node_stroke, "\"")),
                "/>"
            )
            .unwrap();
            svg_write!(groups, label_svg(&category.label, end_x, box_y, " class=\"pele-label\"", None)).unwrap();
            if !texts.is_empty() {
                svg_write!(groups, "<g class=\"pele-ishikawa-causes\" font-size=\"", js::number_text(small), "\">", texts, "</g>").unwrap();
            }
            groups.push_str("</g>");
        }

        let r = head_h / 2.0;
        body = svg_format!(
            "<path class=\"pele-ishikawa-spine\" d=\"M",
            num(tail),
            ",0H",
            num(-(ARROW - 1.0)),
            "\" fill=\"none\" stroke=\"",
            line,
            "\"/>",
            marker("arrow_point", 0.0, 0.0, 1.0, 0.0, arrow),
            groups
        );
        svg_write!(body, "<g class=\"pele-ishikawa-head\" data-id=\"", esc_text(&root.text), "\">").unwrap();
        svg_write!(
            body,
            "<path d=\"M0,",
            num(-r),
            "H",
            num(head_w - r),
            "A",
            num(r),
            ",",
            num(r),
            " 0 0 1 ",
            num(head_w - r),
            ",",
            num(r),
            "H0Z\" fill=\"",
            node_fill,
            "\" stroke=\"",
            node_stroke,
            "\" stroke-linejoin=\"round\"/>"
        )
        .unwrap();
        svg_write!(body, label_svg(&head, 12.0 + head.width / 2.0, 0.0, " class=\"pele-label\" font-weight=\"var(--_hw)\"", None)).unwrap();
        body.push_str("</g>");
    }

    let inner = js::max(max_x - min_x, title.width);
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();
    let ox = pad + (inner - (max_x - min_x)) / 2.0 - min_x;
    let oy = pad + title_height - min_y;
    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
    if !body.is_empty() {
        svg_write!(content, "<g transform=\"translate(", num(ox), ",", num(oy), ")\">", body, "</g>").unwrap();
    }
    Rendered { svg: svg_document("ishikawa", width, height, size, ctx, Accessible::default(), &content), width, height, links: Vec::new() }
}

struct Entry {
    node: usize,
    depth: usize,
    parent: Option<usize>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// The outline drawn down the page, for a host too narrow for the fishbone.
fn vertical_causes(model: &IshikawaModel, ctx: &RenderCtx) -> Rendered {
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let room = js::max(140.0, options.max_width.unwrap_or(320.0) - 2.0 * pad);
    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, title_room(options), style::BOLD);
    let mut order: Vec<Entry> = Vec::new();
    let mut pending: Vec<(usize, usize, Option<usize>)> = if model.nodes.is_empty() { Vec::new() } else { vec![(0, 0, None)] };
    let mut depth = 0;
    while let Some((node, entry_depth, parent)) = pending.pop() {
        let index = order.len();
        order.push(Entry { node, depth: entry_depth, parent, x: 0.0, y: 0.0, w: 0.0, h: 0.0 });
        depth = depth.max(entry_depth);
        for &child in model.nodes[node].children.iter().rev() {
            pending.push((child, entry_depth + 1, Some(index)));
        }
    }
    let indent = js::min(20.0, js::max(2.0, (room - 120.0) / depth.max(1) as f64));
    // `title.height ? … : 0` is falsy for 0 and NaN.
    let mut y = pad + if title.height != 0.0 && !title.height.is_nan() { title.height + 12.0 } else { 0.0 };
    let mut width = title.width;
    let mut nodes = String::new();
    let mut edges = String::new();
    let edge_color = ctx.paint(Role::Line, "var(--_b)");
    let node_fill = ctx.paint(Role::NodeFill, "var(--_s)");
    let node_stroke = ctx.paint(Role::NodeStroke, "var(--_b)");
    for k in 0..order.len() {
        let entry_depth = order[k].depth;
        let text = &model.nodes[order[k].node].text;
        let x = pad + entry_depth as f64 * indent;
        let label_room = js::max(60.0, room - entry_depth as f64 * indent - 20.0);
        let label = break_label(
            &layout_label(
                Some(text),
                false,
                &m,
                if entry_depth < 2 { size } else { small },
                label_room,
                if entry_depth < 2 { style::BOLD } else { 0 },
            ),
            &m,
            label_room,
        );
        let w = label.width + 20.0;
        let h = label.height + 12.0;
        let top = y;
        y += h + 12.0;
        width = js::max(width, x - pad + w);
        let entry = &mut order[k];
        entry.x = x;
        entry.y = top;
        entry.w = w;
        entry.h = h;
        let framed = entry_depth < 2;
        let class = match entry_depth {
            0 => "pele-ishikawa-head",
            1 => "pele-ishikawa-category",
            _ => "pele-ishikawa-cause",
        };
        svg_write!(nodes, "<g class=\"pele-node ", class, "\" data-id=\"", esc_text(text), "\">").unwrap();
        if framed {
            svg_write!(
                nodes,
                "<rect x=\"",
                num(x),
                "\" y=\"",
                num(top),
                "\" width=\"",
                num(w),
                "\" height=\"",
                num(h),
                "\" rx=\"",
                if entry_depth == 0 { 12 } else { 4 },
                "\" fill=\"",
                node_fill,
                "\" stroke=\"",
                node_stroke,
                "\"/>"
            )
            .unwrap();
        }
        let attrs = svg_format!(
            " class=\"pele-label\"",
            if entry_depth < 2 { " font-weight=\"var(--_hw)\"" } else { "" },
            if entry_depth > 1 { svg_format!(" font-size=\"", js::number_text(small), "\"") } else { String::new() }
        );
        svg_write!(nodes, label_svg(&label, x + w / 2.0, top + h / 2.0, &attrs, None)).unwrap();
        nodes.push_str("</g>");
        if let Some(p) = order[k].parent {
            let parent = &order[p];
            let gx = parent.x + indent / 2.0;
            svg_write!(
                edges,
                "<path class=\"pele-edge\" d=\"M",
                num(gx),
                ",",
                num(parent.y + parent.h),
                "V",
                num(top + h / 2.0),
                "H",
                num(x),
                "\" fill=\"none\" stroke=\"",
                edge_color,
                "\"/>"
            )
            .unwrap();
        }
    }
    let total_width = (width + 2.0 * pad).ceil();
    let height = (js::max(pad + title.height, y - 12.0) + pad).ceil();
    let content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        None
    )) + &edges
        + &nodes;
    Rendered {
        svg: svg_document("ishikawa", total_width, height, size, ctx, Accessible::default(), &content),
        width: total_width,
        height,
        links: Vec::new(),
    }
}
