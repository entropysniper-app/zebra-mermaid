//! Sankey layout. Ported from Pele `src/diagrams/sankey/layout.ts`.

use super::SankeyModel;
use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::{js, math};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alignment {
    Justify,
    Left,
    Right,
    Center,
}

pub(crate) struct LayoutOptions {
    pub width: f64,
    pub height: f64,
    pub node_width: f64,
    pub node_padding: f64,
    /// Height of a label line; thin nodes are kept at least this far apart where there is room.
    pub label_height: f64,
    pub alignment: Alignment,
}

#[derive(Debug, Clone)]
pub(crate) struct PlacedNode {
    pub layer: usize,
    /// Total flow through the node, in the diagram's units.
    pub value: f64,
    pub x: f64,
    pub y: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct PlacedLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub thickness: f64,
    /// Vertical centers of the band where it leaves its source and where it enters its target.
    pub source_y: f64,
    pub target_y: f64,
}

const PASSES: i32 = 6;

/// `x || y` for a comparator result: a zero or NaN difference falls through to the next key.
#[inline]
fn or(first: f64, then: f64) -> f64 {
    if first != 0.0 && !first.is_nan() {
        first
    } else {
        then
    }
}

/// Pele's `sort`: an insertion sort for short lists ("the built-in sort is slow to start on short
/// lists"), the built-in sort otherwise. Every comparator here ends in a difference of indexes,
/// so no two items tie and both give the one sorted order. The long case is a stable merge sort
/// that, unlike `slice::sort_by`, cannot panic should NaN centers make the order inconsistent.
fn sort(list: &mut [usize], compare: &impl Fn(usize, usize) -> f64) {
    if list.len() > 8 {
        merge_sort(list, compare);
        return;
    }
    for i in 1..list.len() {
        let item = list[i];
        let mut j = i;
        while j > 0 && compare(list[j - 1], item) > 0.0 {
            list[j] = list[j - 1];
            j -= 1;
        }
        list[j] = item;
    }
}

fn merge_sort(list: &mut [usize], compare: &impl Fn(usize, usize) -> f64) {
    let n = list.len();
    let mut merged = vec![0usize; n];
    let mut width = 1;
    while width < n {
        let mut start = 0;
        while start < n {
            let mid = (start + width).min(n);
            let end = (start + 2 * width).min(n);
            let (mut a, mut b) = (start, mid);
            for slot in &mut merged[start..end] {
                // Take from the right run only when it must come first, which keeps ties stable.
                if a < mid && (b >= end || compare(list[a], list[b]) <= 0.0) {
                    *slot = list[a];
                    a += 1;
                } else {
                    *slot = list[b];
                    b += 1;
                }
            }
            start = end;
        }
        list.copy_from_slice(&merged);
        width *= 2;
    }
}

/// Places nodes in columns by their distance from a source, sizes them by their flow, and then
/// lets each column settle next to its neighbours so that the bands cross less.
pub(crate) fn layout_sankey(model: &SankeyModel, options: &LayoutOptions) -> Result<(Vec<PlacedNode>, Vec<PlacedLink>), Error> {
    let n = model.nodes.len();
    let mut links: Vec<PlacedLink> = model
        .links
        .iter()
        .map(|link| PlacedLink {
            source: link.source,
            target: link.target,
            value: if link.value.is_finite() && link.value > 0.0 { link.value } else { 0.0 },
            thickness: 0.0,
            source_y: 0.0,
            target_y: 0.0,
        })
        .collect();
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut incoming: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (k, link) in links.iter().enumerate() {
        outgoing[link.source].push(k);
        incoming[link.target].push(k);
    }

    // Longest distance from a source, in topological order; what is left over lies on a cycle.
    let mut depth = vec![0i32; n];
    let mut pending = vec![0i32; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);
    for i in 0..n {
        pending[i] = incoming[i].len() as i32;
        if pending[i] == 0 {
            order.push(i);
        }
    }
    let mut q = 0;
    while q < order.len() {
        let i = order[q];
        for &k in &outgoing[i] {
            let t = links[k].target;
            if depth[t] < depth[i] + 1 {
                depth[t] = depth[i] + 1;
            }
            pending[t] -= 1;
            if pending[t] == 0 {
                order.push(t);
            }
        }
        q += 1;
    }
    if order.len() < n {
        // Every node left over has a predecessor that is left over too; walking back finds the loop.
        let mut stuck = 0;
        while pending[stuck] == 0 {
            stuck += 1;
        }
        let mut seen = vec![false; n];
        while !seen[stuck] {
            seen[stuck] = true;
            for &k in &incoming[stuck] {
                if pending[links[k].source] > 0 {
                    stuck = links[k].source;
                    break;
                }
            }
        }
        return Err(Error::semantic(format!(
            "Sankey diagrams cannot have circular links, and \"{}\" is part of one.",
            model.nodes[stuck].id
        ))
        .with_type(DiagramType::Sankey));
    }
    let mut layers: i32 = 1;
    for &d in &depth {
        if d + 1 > layers {
            layers = d + 1;
        }
    }
    let mut reach = vec![0i32; n];
    for q in (0..n).rev() {
        let i = order[q];
        for &k in &outgoing[i] {
            if reach[i] < reach[links[k].target] + 1 {
                reach[i] = reach[links[k].target] + 1;
            }
        }
    }

    // Flows are scaled to the largest one, so that sums of huge values stay finite.
    let mut largest = 0.0;
    for link in &links {
        if link.value > largest {
            largest = link.value;
        }
    }
    let unit = if largest > 0.0 { 1.0 / largest } else { 0.0 };

    let mut nodes: Vec<PlacedNode> = Vec::with_capacity(n);
    let mut flow = vec![0.0f64; n];
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); layers as usize];
    let step = if layers > 1 { (options.width - options.node_width) / f64::from(layers - 1) } else { 0.0 };
    for i in 0..n {
        let layer: i32 = match options.alignment {
            Alignment::Left => depth[i],
            Alignment::Right => layers - 1 - reach[i],
            Alignment::Center => {
                let mut layer = depth[i];
                if incoming[i].is_empty() && !outgoing[i].is_empty() {
                    layer = layers;
                    for &k in &outgoing[i] {
                        if depth[links[k].target] - 1 < layer {
                            layer = depth[links[k].target] - 1;
                        }
                    }
                }
                layer
            }
            Alignment::Justify => {
                if !outgoing[i].is_empty() {
                    depth[i]
                } else {
                    layers - 1
                }
            }
        };
        let mut into = 0.0;
        let mut out = 0.0;
        let mut value = 0.0;
        let mut value_out = 0.0;
        for &k in &incoming[i] {
            into += links[k].value * unit;
            value += links[k].value;
        }
        for &k in &outgoing[i] {
            out += links[k].value * unit;
            value_out += links[k].value;
        }
        flow[i] = js::max(into, out);
        nodes.push(PlacedNode { layer: layer as usize, value: js::max(value, value_out), x: f64::from(layer) * step, y: 0.0, height: 0.0 });
        columns[layer as usize].push(i);
    }

    let mut tallest = 1;
    for column in &columns {
        if column.len() > tallest {
            tallest = column.len();
        }
    }
    let gap = if tallest > 1 { js::min(options.node_padding, options.height / (tallest as f64 - 1.0)) } else { options.node_padding };
    let mut scale = f64::INFINITY;
    for column in &columns {
        let mut sum = 0.0;
        for &i in column {
            sum += flow[i];
        }
        if sum > 0.0 {
            scale = js::min(scale, (options.height - (column.len() as f64 - 1.0) * gap) / sum);
        }
    }
    if !scale.is_finite() || scale < 0.0 {
        scale = 0.0;
    }

    // A thin node still needs room for its label. Where every column can afford it, each node
    // claims at least a label's height, and the flows are scaled down to make that fit.
    let mut claim = js::max(0.0, options.label_height - gap);
    let fits = |s: f64, claim: f64| -> bool {
        for column in &columns {
            let mut extent = (column.len() as f64 - 1.0) * gap;
            for &i in column {
                extent += js::max(flow[i] * s, claim);
            }
            if extent > options.height + 1e-6 {
                return false;
            }
        }
        true
    };
    if claim > 0.0 && !fits(scale, claim) {
        if fits(0.0, claim) {
            let mut low = 0.0;
            for _ in 0..40 {
                let mid = (low + scale) / 2.0;
                if fits(mid, claim) {
                    low = mid;
                } else {
                    scale = mid;
                }
            }
            scale = low;
        } else {
            claim = 0.0;
        }
    }

    // The room a node keeps clear above and below itself.
    let mut margin = vec![0.0f64; n];
    for link in &mut links {
        link.thickness = link.value * unit * scale;
    }
    for column in &columns {
        let mut y = 0.0;
        for &i in column {
            nodes[i].height = flow[i] * scale;
            margin[i] = js::max(0.0, claim - nodes[i].height) / 2.0;
            nodes[i].y = y + margin[i];
            y += nodes[i].height + 2.0 * margin[i] + gap;
        }
        // Spare room is shared out between the nodes of a column.
        let spare = (options.height - y + gap) / (column.len() as f64 + 1.0);
        for (k, &i) in column.iter().enumerate() {
            nodes[i].y += spare * (k as f64 + 1.0);
        }
    }

    // Bands leave and enter a node in the order of where they go, so they do not cross at it.
    // These are the offsets of each band's middle from the top of the node at either end.
    let mut source_offset = vec![0.0f64; links.len()];
    let mut target_offset = vec![0.0f64; links.len()];
    let mut state = Settle { nodes: &mut nodes, links: &links, margin: &margin, gap, height: options.height };
    state.stack_all(&mut outgoing, &mut incoming, &mut source_offset, &mut target_offset);
    for pass in 0..PASSES {
        let pull = math::pow(0.99, f64::from(pass));
        // From the second last column back to the first, then from the second to the last.
        for column in columns[..(layers - 1) as usize].iter_mut().rev() {
            state.settle(column, pull, false, &mut outgoing, &mut incoming, &mut source_offset, &mut target_offset);
        }
        state.stack_all(&mut outgoing, &mut incoming, &mut source_offset, &mut target_offset);
        for column in columns.iter_mut().skip(1) {
            state.settle(column, pull, true, &mut outgoing, &mut incoming, &mut source_offset, &mut target_offset);
        }
        state.stack_all(&mut outgoing, &mut incoming, &mut source_offset, &mut target_offset);
    }
    for k in 0..links.len() {
        links[k].source_y = nodes[links[k].source].y + source_offset[k];
        links[k].target_y = nodes[links[k].target].y + target_offset[k];
    }

    Ok((nodes, links))
}

/// What the settling passes read and move.
struct Settle<'a> {
    nodes: &'a mut [PlacedNode],
    links: &'a [PlacedLink],
    margin: &'a [f64],
    gap: f64,
    height: f64,
}

impl Settle<'_> {
    #[inline]
    fn center(&self, i: usize) -> f64 {
        self.nodes[i].y + self.nodes[i].height / 2.0
    }

    fn stack_out(&self, list: &mut [usize], source_offset: &mut [f64]) {
        let by_target = |a: usize, b: usize| or(self.center(self.links[a].target) - self.center(self.links[b].target), a as f64 - b as f64);
        sort(list, &by_target);
        let mut y = 0.0;
        for &k in list.iter() {
            source_offset[k] = y + self.links[k].thickness / 2.0;
            y += self.links[k].thickness;
        }
    }

    fn stack_in(&self, list: &mut [usize], target_offset: &mut [f64]) {
        let by_source = |a: usize, b: usize| or(self.center(self.links[a].source) - self.center(self.links[b].source), a as f64 - b as f64);
        sort(list, &by_source);
        let mut y = 0.0;
        for &k in list.iter() {
            target_offset[k] = y + self.links[k].thickness / 2.0;
            y += self.links[k].thickness;
        }
    }

    fn stack_all(&self, outgoing: &mut [Vec<usize>], incoming: &mut [Vec<usize>], source_offset: &mut [f64], target_offset: &mut [f64]) {
        for i in 0..self.nodes.len() {
            self.stack_out(&mut outgoing[i], source_offset);
            self.stack_in(&mut incoming[i], target_offset);
        }
    }

    /// Moves each node of a column towards where its bands would run level, then restores the
    /// order and the gaps. `forward` looks at the bands coming in, otherwise at those going out.
    #[allow(clippy::too_many_arguments)]
    fn settle(
        &mut self,
        column: &mut [usize],
        pull: f64,
        forward: bool,
        outgoing: &mut [Vec<usize>],
        incoming: &mut [Vec<usize>],
        source_offset: &mut [f64],
        target_offset: &mut [f64],
    ) {
        for &i in column.iter() {
            let list = if forward { &mut incoming[i] } else { &mut outgoing[i] };
            if list.is_empty() {
                continue;
            }
            if forward {
                self.stack_in(list, target_offset);
            } else {
                self.stack_out(list, source_offset);
            }
            let mut weighted = 0.0;
            let mut weight = 0.0;
            let mut plain = 0.0;
            for &k in list.iter() {
                let link = &self.links[k];
                let level = if forward {
                    self.nodes[link.source].y + source_offset[k] - target_offset[k]
                } else {
                    self.nodes[link.target].y + target_offset[k] - source_offset[k]
                };
                weighted += level * link.thickness;
                weight += link.thickness;
                plain += self.center(if forward { link.source } else { link.target }) - self.nodes[i].height / 2.0;
            }
            let wanted = if weight > 0.0 { weighted / weight } else { plain / list.len() as f64 };
            self.nodes[i].y += (wanted - self.nodes[i].y) * pull;
        }
        let by_center = |a: usize, b: usize| or(self.center(a) - self.center(b), a as f64 - b as f64);
        sort(column, &by_center);
        let mut y = 0.0;
        for &i in column.iter() {
            if self.nodes[i].y - self.margin[i] < y {
                self.nodes[i].y = y + self.margin[i];
            }
            y = self.nodes[i].y + self.nodes[i].height + self.margin[i] + self.gap;
        }
        y = self.height;
        for &i in column.iter().rev() {
            if self.nodes[i].y + self.nodes[i].height + self.margin[i] > y {
                self.nodes[i].y = y - self.nodes[i].height - self.margin[i];
            }
            y = self.nodes[i].y - self.margin[i] - self.gap;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sorts_agree() {
        let keys = [5.0, 1.0, 3.0, 1.0, 9.0, 0.0, 3.0, 7.0, 2.0, 2.0, 8.0, 1.0];
        let compare = |a: usize, b: usize| or(keys[a] - keys[b], a as f64 - b as f64);
        let mut long: Vec<usize> = (0..keys.len()).rev().collect();
        sort(&mut long, &compare);
        assert_eq!(long, vec![5, 1, 3, 11, 8, 9, 2, 6, 0, 7, 10, 4]);
        let mut short: Vec<usize> = vec![6, 2, 0, 3, 1];
        sort(&mut short, &compare);
        assert_eq!(short, vec![1, 3, 2, 6, 0]);
    }
}
