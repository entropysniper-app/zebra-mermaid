//! Squarified layout. Ported from Pele `src/diagrams/treemap/layout.ts`.

use super::TreemapNode;
use crate::util::js;

#[cfg(test)]
#[path = "../../../tests/upstream/treemap_layout_1_12.rs"]
mod upstream_1_12;

#[derive(Debug, Clone)]
pub struct Box {
    /// The node, by index into the model's nodes.
    pub node: usize,
    pub parent: Option<usize>,
    /// Position among the top-level nodes of the branch this box belongs to, which picks its
    /// color.
    pub series: usize,
    /// A leaf's own value, or the sum of the leaves under a section.
    pub value: f64,
    pub children: Vec<usize>,
    pub headed: bool,
    pub header: f64,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct Insets {
    pub gap: f64,
    /// Space a section keeps for its header, and around its children on the other three sides.
    pub header: f64,
    pub short: f64,
    pub side: f64,
}

fn make_box(nodes: &[TreemapNode], node: usize, parent: Option<usize>, series: usize) -> Box {
    let n = &nodes[node];
    let value = match (n.children.is_none(), n.value) {
        (true, Some(v)) if v.is_finite() && v > 0.0 => v,
        _ => 0.0,
    };
    Box { node, parent, series, value, children: Vec::new(), headed: false, header: 0.0, x: 0.0, y: 0.0, w: 0.0, h: 0.0 }
}

/// Lays out the boxes as one squarified treemap: rows are added along the shorter side, and a
/// row stops growing when another box would make its boxes less square.
#[allow(clippy::too_many_arguments)]
fn squarify(all: &mut [Box], boxes: &[usize], total: f64, mut x: f64, mut y: f64, mut w: f64, mut h: f64, gap: f64) {
    let mut rest = total;
    let mut from = 0;
    while from < boxes.len() {
        let scale = (w * h) / rest;
        let side = js::min(w, h);
        let mut sum = all[boxes[from]].value * scale;
        let largest = sum;
        let mut worst = js::max((side * side * largest) / (sum * sum), (sum * sum) / (side * side * largest));
        let mut to = from + 1;
        while to < boxes.len() {
            let smallest = all[boxes[to]].value * scale;
            let next = sum + smallest;
            let ratio = js::max((side * side * largest) / (next * next), (next * next) / (side * side * smallest));
            if ratio > worst {
                break;
            }
            worst = ratio;
            sum = next;
            to += 1;
        }

        let thickness = sum / side;
        let mut offset = 0.0;
        for i in from..to {
            let b = &mut all[boxes[i]];
            let length = if i == to - 1 { side - offset } else { (b.value * scale) / thickness };
            let bx = if w >= h { x } else { x + offset };
            let by = if w >= h { y + offset } else { y };
            let bw = if w >= h { thickness } else { length };
            let bh = if w >= h { length } else { thickness };
            let gx = if js::round(bx + bw - gap / 2.0) > js::round(bx + gap / 2.0) { gap / 2.0 } else { 0.0 };
            let gy = if js::round(by + bh - gap / 2.0) > js::round(by + gap / 2.0) { gap / 2.0 } else { 0.0 };
            b.x = js::round(bx + gx);
            b.y = js::round(by + gy);
            b.w = js::round(bx + bw - gx) - b.x;
            b.h = js::round(by + bh - gy) - b.y;
            offset += length;
            rest -= b.value;
        }
        if w >= h {
            x += thickness;
            w -= thickness;
        } else {
            y += thickness;
            h -= thickness;
        }
        from = to;
    }
}

/// Returns every box (all of them, for parents and colors) and the indices of those with a
/// size, parents before their children.
pub fn layout_treemap(nodes: &[TreemapNode], roots: &[usize], width: f64, height: f64, insets: &Insets) -> (Vec<Box>, Vec<usize>) {
    let mut all: Vec<Box> = roots.iter().enumerate().map(|(index, &node)| make_box(nodes, node, None, index)).collect();
    let top: Vec<usize> = (0..all.len()).collect();
    let mut i = 0;
    while i < all.len() {
        let node = all[i].node;
        let series = all[i].series;
        if let Some(children) = &nodes[node].children {
            for &child in children {
                let b = make_box(nodes, child, Some(i), series);
                let at = all.len();
                all.push(b);
                all[i].children.push(at);
            }
        }
        i += 1;
    }
    for i in (0..all.len()).rev() {
        if let Some(parent) = all[i].parent {
            let value = all[i].value;
            all[parent].value += value;
        }
    }

    let Insets { gap, header, short, side } = *insets;
    let mut placed: Vec<usize> = Vec::new();
    let place = |all: &mut Vec<Box>, placed: &mut Vec<usize>, children: &[usize], x: f64, y: f64, w: f64, h: f64| {
        let mut shown: Vec<usize> = children.iter().copied().filter(|&b| all[b].value > 0.0).collect();
        // A value that is not finite makes the total not finite, and nothing is placed. Leaving
        // early keeps the sort below to finite values, where it is a total order.
        if shown.iter().any(|&b| !all[b].value.is_finite()) {
            return;
        }
        // `.sort((a, b) => b.value - a.value)`, stable.
        shown.sort_by(|&a, &b| all[b].value.total_cmp(&all[a].value));
        let mut total = 0.0;
        for &b in &shown {
            total += all[b].value;
        }
        if shown.is_empty() || !total.is_finite() {
            return;
        }
        let w = js::max(0.0, w);
        let h = js::max(0.0, h);
        if w > 0.0 && h > 0.0 {
            squarify(all, &shown, total, x - gap / 2.0, y - gap / 2.0, w + gap, h + gap, gap);
        }
        for &b in &shown {
            let clamp = |v: f64, low: f64, high: f64| if v >= low { js::min(v, high) } else { low };
            let bx = &mut all[b];
            let x0 = clamp(bx.x, x, x + w);
            let y0 = clamp(bx.y, y, y + h);
            bx.w = clamp(bx.x + bx.w, x0, x + w) - x0;
            bx.h = clamp(bx.y + bx.h, y0, y + h) - y0;
            bx.x = x0;
            bx.y = y0;
            placed.push(b);
        }
    };

    place(&mut all, &mut placed, &top, 0.0, 0.0, width, height);
    let mut i = 0;
    while i < placed.len() {
        let b = placed[i];
        if !all[b].children.is_empty() {
            let children = all[b].children.clone();
            let (bx, by, bw, bh) = (all[b].x, all[b].y, all[b].w, all[b].h);
            let top = if bh - header - side >= 12.0 { header } else { short };
            all[b].headed = bw > js::max(24.0, 2.0 * side) && bh - top - side >= 8.0;
            all[b].header = if all[b].headed { top } else { 0.0 };
            if all[b].headed {
                place(&mut all, &mut placed, &children, bx + side, by + top, bw - 2.0 * side, bh - top - side);
            } else {
                place(&mut all, &mut placed, &children, bx, by, bw, bh);
            }
        }
        i += 1;
    }
    (all, placed)
}
