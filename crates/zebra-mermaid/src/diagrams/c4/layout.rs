//! The C4 grid layout. Ported from Pele `src/diagrams/c4/layout.ts`.

use super::route::{Bx, HEAD, LABEL_PAD, LANE, NARROW};
use super::{C4Model, C4Shape};
use crate::text::{layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::util::js;

pub(super) const SHAPE_W: f64 = 216.0;
const SHAPE_MIN_H: f64 = 64.0;
const PAD_X: f64 = 14.0;
const PAD_Y: f64 = 12.0;
const GAP_X: f64 = NARROW + LABEL_PAD + 3.0 * crate::svg::edges::ARROW;
const GAP_Y: f64 = 88.0;
/// Room for a relation to run along the inside of a boundary.
pub(super) const GROUP_PAD: f64 = LANE + 8.0;
const GROUP_GAP: f64 = 48.0;
/// Where a boundary's heading starts, from its top-left corner.
pub(super) const HEAD_LEFT: f64 = 16.0;
pub(super) const HEAD_TOP: f64 = 10.0;

/// Shape keywords Mermaid accepts in $shape, $sprite or $tags, as names of the shared shapes.
fn form_named(name: &str) -> Option<&'static str> {
    Some(match name {
        "person" => "person",
        "box" | "rounded" | "component" => "rect",
        "cylinder" | "database" | "db" => "cyl",
        "queue" | "pipe" => "h-cyl",
        _ => return None,
    })
}

pub(super) struct Node {
    pub shape: usize,
    /// Type, name, technology, description.
    pub labels: [Label; 4],
    pub text_h: f64,
    pub bx: Bx,
}

pub(super) struct Group {
    pub boundary: usize,
    pub kids: Vec<usize>,
    pub nodes: Vec<Node>,
    /// Name, type, description.
    pub labels: Vec<Label>,
    pub head: f64,
    pub head_w: f64,
    pub head_x: f64,
    pub bx: Bx,
}

pub(super) struct Layout {
    /// One group per boundary, in the model's order.
    pub groups: Vec<Group>,
    /// The boundaries that are drawn, parents before children. The first is the unnamed top
    /// level.
    pub order: Vec<usize>,
    /// The box of each shape that is drawn: its group and its place there.
    pub shape_box: Vec<Option<(usize, usize)>>,
    /// Whether each boundary is drawn (the top level is not a box).
    pub boundary_box: Vec<bool>,
}

impl Layout {
    pub fn shape_bx(&self, shape: usize) -> Option<Bx> {
        self.shape_box[shape].map(|(g, k)| self.groups[g].nodes[k].bx)
    }

    pub fn boundary_bx(&self, boundary: usize) -> Option<Bx> {
        self.boundary_box[boundary].then(|| self.groups[boundary].bx)
    }
}

fn form_of(shape: &C4Shape) -> &'static str {
    let el = &shape.element;
    let explicit = form_named(&el.shape.as_deref().unwrap_or("").to_lowercase())
        .or_else(|| form_named(&el.sprite.as_deref().unwrap_or("").to_lowercase()));
    if let Some(form) = explicit {
        return form;
    }
    if let Some(tags) = el.tags.as_deref().filter(|t| !t.is_empty()) {
        for tag in tags.split(',') {
            if let Some(form) = form_named(&js::trim(tag).to_lowercase()) {
                return form;
            }
        }
    }
    let kind = shape.kind.as_str();
    if kind.ends_with("person") {
        "person"
    } else if kind.ends_with("_db") {
        "cyl"
    } else if kind.ends_with("_queue") {
        "h-cyl"
    } else {
        "rect"
    }
}

fn measure(shape: &C4Shape, index: usize, id: usize, m: &Measure, size: f64, small: f64, width: f64) -> Node {
    let form = form_of(shape);
    let cap_w = if form == "h-cyl" { 36.0 } else { 0.0 };
    let cap_h = if form == "person" {
        HEAD
    } else if form == "cyl" {
        24.0
    } else {
        0.0
    };
    let inner = width - 2.0 * PAD_X - cap_w;
    let el = &shape.element;
    let techn = el.techn.as_deref().filter(|t| !t.is_empty()).map(|t| format!("[{}]", t));
    let labels = [
        layout_label(Some(&format!("«{}»", shape.kind.replace('_', " "))), false, m, small - 1.0, inner, 0),
        layout_label(Some(&el.label), false, m, size, inner, style::BOLD),
        layout_label(techn.as_deref(), false, m, small, inner, 0),
        layout_label(el.descr.as_deref(), false, m, small, inner, 0),
    ];
    let mut tw: f64 = 0.0;
    let mut text_h = if labels[3].height > 0.0 { 4.0 } else { 0.0 };
    for label in &labels {
        tw = js::max(tw, label.width);
        text_h += label.height;
    }
    Node {
        shape: index,
        bx: Bx {
            id,
            x: 0.0,
            y: 0.0,
            w: js::max(width, tw + 2.0 * PAD_X + cap_w),
            h: js::max(SHAPE_MIN_H, text_h + 2.0 * PAD_Y + cap_h),
            form,
        },
        labels,
        text_h,
    }
}

/// Places boxes left to right in rows of `per_row`, from (0, 0), and returns the width and
/// height the rows take. Shapes are stretched to the height of their row; boundaries keep
/// their own.
fn rows(boxes: &mut [&mut Bx], per_row: f64, gap_x: f64, gap_y: f64, stretch: bool) -> (f64, f64) {
    let mut width: f64 = 0.0;
    let mut y = 0.0;
    let len = boxes.len();
    // `per_row` is an integer of at least 1, perhaps a huge one.
    let step = if per_row >= len as f64 { len.max(1) } else { per_row as usize };
    let mut from = 0;
    while from < len {
        let to = len.min(from + step);
        let mut x = 0.0;
        let mut row_h: f64 = 0.0;
        for b in boxes[from..to].iter() {
            row_h = js::max(row_h, b.h);
        }
        for b in boxes[from..to].iter_mut() {
            b.x = x;
            b.y = y;
            if stretch {
                b.h = row_h;
            }
            x += b.w + gap_x;
        }
        width = js::max(width, x - gap_x);
        y += row_h + gap_y;
        from += step;
    }
    (width, if len > 0 { y - gap_y } else { 0.0 })
}

/// Mermaid's grid: in each boundary, its shapes in rows of c4ShapeInRow, then the boundaries
/// inside it in rows of c4BoundaryInRow, each sized around its own content.
pub(super) fn layout_c4(db: &C4Model, m: &Measure, size: f64, small: f64, room: Option<f64>, gap: f64) -> Layout {
    let group_pad = if room.is_none() { GROUP_PAD } else { 12.0 };
    let nb = db.boundaries.len();
    let mut groups: Vec<Group> = Vec::with_capacity(nb);
    for (index, boundary) in db.boundaries.iter().enumerate() {
        let label_room = match room {
            None => 4000.0,
            Some(room) => js::max(80.0, room - 2.0 * HEAD_LEFT - 2.0 * GROUP_PAD * 2.0),
        };
        let el = &boundary.element;
        let labels = if el.parent.is_some() {
            let kind = el.kind_type.as_deref().filter(|t| !t.is_empty()).map(|t| format!("[{}]", t));
            vec![
                layout_label(Some(&el.label), false, m, size, label_room, style::BOLD),
                layout_label(kind.as_deref(), false, m, small, label_room, 0),
                layout_label(el.descr.as_deref(), false, m, small, label_room, 0),
            ]
        } else {
            Vec::new()
        };
        let mut head = 0.0;
        let mut head_w: f64 = 0.0;
        for label in &labels {
            head += label.height;
            head_w = js::max(head_w, label.width);
        }
        groups.push(Group {
            boundary: index,
            kids: Vec::new(),
            nodes: Vec::new(),
            labels,
            head: if head > 0.0 { head + HEAD_TOP + 6.0 + LANE } else { group_pad },
            head_w,
            head_x: HEAD_LEFT,
            bx: Bx { id: index, x: 0.0, y: 0.0, w: 0.0, h: 0.0, form: "rect" },
        });
    }
    for g in 0..nb {
        if let Some(parent) = db.boundaries[g].element.parent {
            groups[parent].kids.push(g);
        }
    }
    // A boundary that ends up inside itself is not reachable from the top, as in Mermaid.
    let root = 0;
    let mut order: Vec<usize> = vec![root];
    let mut depths: Vec<Option<f64>> = vec![None; nb];
    depths[root] = Some(0.0);
    let mut i = 0;
    while i < order.len() {
        let g = order[i];
        for k in 0..groups[g].kids.len() {
            let kid = groups[g].kids[k];
            depths[kid] = Some(depths[g].unwrap_or(0.0) + 1.0);
            order.push(kid);
        }
        i += 1;
    }
    for (index, shape) in db.shapes.iter().enumerate() {
        if let Some(parent) = shape.element.parent {
            let depth = depths[parent].unwrap_or(0.0);
            let node_width = match room {
                None => SHAPE_W,
                Some(room) => js::max(100.0, js::min(SHAPE_W, room - 2.0 * depth * group_pad - 32.0)),
            };
            let node = measure(shape, index, nb + index, m, size, small, node_width);
            groups[parent].nodes.push(node);
        }
    }

    let per_row = js::max(1.0, db.shape_in_row);
    let groups_per_row = js::max(1.0, db.boundary_in_row);
    let mut kids_top: Vec<f64> = vec![0.0; nb];
    for &g in order.iter().rev() {
        let gap_x = if room.is_none() { GAP_X } else { gap };
        let count = |widths: &mut dyn Iterator<Item = f64>, limit: f64, gap: f64| -> f64 {
            let Some(room) = room else { return limit };
            let mut widest: f64 = 1.0;
            for w in widths {
                widest = js::max(widest, w);
            }
            js::max(1.0, js::min(limit, ((room + gap - 2.0 * group_pad) / (widest + gap)).floor()))
        };
        let node_count = count(&mut groups[g].nodes.iter().map(|n| n.bx.w), per_row, gap_x);
        let (grid_w, grid_h) = {
            let mut boxes: Vec<&mut Bx> = groups[g].nodes.iter_mut().map(|n| &mut n.bx).collect();
            rows(&mut boxes, node_count, gap_x, GAP_Y, true)
        };
        let kids = groups[g].kids.clone();
        let kid_count = count(&mut kids.iter().map(|&k| groups[k].bx.w), groups_per_row, GROUP_GAP);
        let mut kid_boxes: Vec<Bx> = kids.iter().map(|&k| groups[k].bx).collect();
        let (kids_w, kids_h) = {
            let mut boxes: Vec<&mut Bx> = kid_boxes.iter_mut().collect();
            rows(&mut boxes, kid_count, GROUP_GAP, GROUP_GAP, false)
        };
        for (k, &kid) in kids.iter().enumerate() {
            groups[kid].bx = kid_boxes[k];
        }
        let between = if grid_h > 0.0 && kids_h > 0.0 { GROUP_GAP } else { 0.0 };
        let frame = if g == root { 0.0 } else { group_pad };
        kids_top[g] = grid_h + between;
        let group = &mut groups[g];
        group.bx.w = js::max(js::max(grid_w, kids_w), group.head_w + 2.0 * HEAD_LEFT - 2.0 * frame) + 2.0 * frame;
        // `(frame && group.head)`
        let head = if frame != 0.0 { group.head } else { frame };
        group.bx.h = head + grid_h + between + kids_h + frame;
    }

    let mut shape_box: Vec<Option<(usize, usize)>> = vec![None; db.shapes.len()];
    let mut boundary_box = vec![false; nb];
    for &g in &order {
        let (left, top) = {
            let group = &groups[g];
            (group.bx.x + if g == root { 0.0 } else { group_pad }, group.bx.y + if g == root { 0.0 } else { group.head })
        };
        for (k, node) in groups[g].nodes.iter_mut().enumerate() {
            node.bx.x += left;
            node.bx.y += top;
            shape_box[node.shape] = Some((g, k));
        }
        for k in 0..groups[g].kids.len() {
            let kid = groups[g].kids[k];
            groups[kid].bx.x += left;
            groups[kid].bx.y += top + kids_top[g];
        }
        if g != root {
            boundary_box[g] = true;
        }
    }
    Layout { groups, order, shape_box, boundary_box }
}
