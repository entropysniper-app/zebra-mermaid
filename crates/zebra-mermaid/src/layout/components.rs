//! Move disconnected graph components into rows, preserving each component's layout.
//! Ported from Pele `src/layout/components.ts`.
use super::compound::{CEdge, CNode, CompoundResult};
use crate::util::js;
use indexmap::IndexMap;

struct Part {
    nodes: Vec<usize>,
    edges: Vec<usize>,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}
impl Part {
    fn include(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.left = js::min(self.left, x - w / 2.0);
        self.top = js::min(self.top, y - h / 2.0);
        self.right = js::max(self.right, x + w / 2.0);
        self.bottom = js::max(self.bottom, y + h / 2.0);
    }
}
pub fn wrap_components(nodes: &mut [CNode], edges: &mut [CEdge], width: f64, gap: f64) -> Option<CompoundResult> {
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for (i, n) in nodes.iter().enumerate() {
        if n.parent >= 0 {
            let a = root(&mut parent, i);
            let b = root(&mut parent, n.parent as usize);
            parent[a] = b;
        }
    }
    for e in edges.iter() {
        let a = root(&mut parent, e.src);
        let b = root(&mut parent, e.dst);
        parent[a] = b;
    }
    let mut parts = IndexMap::<usize, Part>::new();
    for (i, c) in nodes.iter().enumerate() {
        let id = root(&mut parent, i);
        let p = parts.entry(id).or_insert_with(|| Part {
            nodes: Vec::new(),
            edges: Vec::new(),
            left: f64::INFINITY,
            top: f64::INFINITY,
            right: f64::NEG_INFINITY,
            bottom: f64::NEG_INFINITY,
        });
        p.nodes.push(i);
        p.include(c.x, c.y, c.w, c.h);
    }
    if parts.len() < 2 {
        return None;
    }
    for (i, e) in edges.iter().enumerate() {
        let p = parts.get_mut(&root(&mut parent, e.src)).unwrap();
        p.edges.push(i);
        for k in (0..e.route.len()).step_by(3) {
            p.include(e.route[k], e.route[k + 1], 0.0, 0.0);
        }
        if e.label_w > 0.0 {
            p.include(e.label_x, e.label_y, e.label_w, e.label_h);
        }
    }
    let (mut x, mut y, mut row_height, mut right, mut from, mut to) = (0.0, 0.0, 0.0, 0.0, f64::INFINITY, f64::NEG_INFINITY);
    let mut shifts = Vec::new();
    for p in parts.values() {
        let (w, h) = (p.right - p.left, p.bottom - p.top);
        if x > 0.0 && x + w > width {
            y += row_height + gap;
            x = 0.0;
            row_height = 0.0;
        }
        shifts.push((x - p.left, y - p.top));
        from = js::min(from, p.left);
        to = js::max(to, p.right);
        right = js::max(right, x + w);
        row_height = js::max(row_height, h);
        x += w + gap;
    }
    if right >= to - from {
        return None;
    }
    for (p, (ox, oy)) in parts.values().zip(shifts) {
        for &i in &p.nodes {
            nodes[i].x += ox;
            nodes[i].y += oy;
        }
        for &i in &p.edges {
            let e = &mut edges[i];
            for k in (0..e.route.len()).step_by(3) {
                e.route[k] += ox;
                e.route[k + 1] += oy;
            }
            e.label_x += ox;
            e.label_y += oy;
        }
    }
    Some(CompoundResult { width: right, height: y + row_height })
}
