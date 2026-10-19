//! The grid of a block diagram. Ported from Pele `src/diagrams/block/layout.ts`.
//!
//! Cells are kept in one list and refer to their children by index; cell 0 is the root.

use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::js;

#[derive(Debug, Clone, Default)]
pub(super) struct Cell {
    pub children: Vec<usize>,
    /// Grid width of this container: a positive count, or -1 to put every child on one row.
    pub columns: f64,
    pub span: f64,
    pub space: bool,
    /// Height reserved above a container's grid for its title.
    pub head: f64,
    /// Size the content needs. Set by the caller for leaves, computed here for containers.
    pub nw: f64,
    pub nh: f64,
    // Placement: grid position, then center and size in pixels.
    pub col: f64,
    pub row: f64,
    pub cols: f64,
    pub heights: Vec<f64>,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

const MAX_SPAN: f64 = 1000.0;

/// `calculateBlockPosition`: the column and row of grid position `position`.
pub(super) fn calculate_block_position(columns: f64, position: f64) -> Result<(f64, f64), Error> {
    if columns == 0.0 || !js::is_integer(columns) {
        return Err(Error::semantic("Columns must be an integer !== 0.").with_type(DiagramType::Block));
    }
    if position < 0.0 || !js::is_integer(position) {
        return Err(
            Error::semantic(format!("Position must be a non-negative integer.{}", js::to_string(position))).with_type(DiagramType::Block)
        );
    }
    if columns < 0.0 {
        return Ok((position, 0.0));
    }
    Ok((js::rem(position, columns), (position / columns).floor()))
}

// Puts each child in its grid cell and sizes the container. Every column is as wide as the
// widest block needs. Every row that holds a plain block is as tall as the tallest block in the
// container, as in Mermaid; a row of nothing but containers is only as tall as they need.
fn measure(cells: &mut [Cell], b: usize, gap: f64, inset: f64) -> Result<(), Error> {
    let columns = cells[b].columns;
    let children = std::mem::take(&mut cells[b].children);
    let mut heights: Vec<f64> = Vec::new();
    let mut position = 0.0;
    let mut cols = 0.0;
    let mut cw = 0.0;
    let mut tallest = 0.0;
    for &c in &children {
        let (px, py) = match calculate_block_position(columns, position) {
            Ok(at) => at,
            Err(e) => {
                cells[b].children = children;
                return Err(e);
            }
        };
        let child = &mut cells[c];
        // Mermaid lets a block wider than the rest of its row stick out of the grid. Pele stops
        // it at the last column.
        let mut span = if child.span >= 1.0 { js::min(child.span, MAX_SPAN) } else { 1.0 };
        if columns > 0.0 && span > columns - px {
            span = columns - px;
        }
        child.span = span;
        child.col = px;
        child.row = py;
        position += span;
        if px + span > cols {
            cols = px + span;
        }
        let row = py as usize;
        // A row is at most one past the last: a child never spans past the last column.
        while row >= heights.len() {
            heights.push(0.0);
        }
        if child.children.is_empty() {
            heights[row] = -1.0;
        } else if heights[row] >= 0.0 && child.nh > heights[row] {
            heights[row] = child.nh;
        }
        if child.space {
            continue;
        }
        let width = (child.nw - gap * (span - 1.0)) / span;
        if width > cw {
            cw = width;
        }
        if child.nh > tallest {
            tallest = child.nh;
        }
    }
    let mut height = (heights.len() as f64 - 1.0) * gap;
    for h in heights.iter_mut() {
        if *h < 0.0 {
            *h = tallest;
        }
        height += *h;
    }
    let bx = &mut cells[b];
    bx.children = children;
    bx.cols = cols;
    bx.heights = heights;
    bx.nw = js::max(bx.nw, cols * cw + (cols - 1.0) * gap + 2.0 * inset);
    bx.nh = height + 2.0 * inset + bx.head;
    Ok(())
}

// Gives each child its cell, sharing out any room the container has beyond what it needs.
fn arrange(cells: &mut [Cell], b: usize, gap: f64, inset: f64) {
    let cols = cells[b].cols;
    let heights = std::mem::take(&mut cells[b].heights);
    let children = std::mem::take(&mut cells[b].children);
    let (bx, by, bw, bh, head) = (cells[b].x, cells[b].y, cells[b].w, cells[b].h, cells[b].head);
    let rows = heights.len() as f64;
    let cw = (bw - 2.0 * inset - (cols - 1.0) * gap) / cols;
    let room = bh - 2.0 * inset - head - (rows - 1.0) * gap;
    let mut needed = 0.0;
    for &height in &heights {
        needed += height;
    }
    let scale = if needed > 0.0 { room / needed } else { 0.0 };
    let share = if needed > 0.0 { 0.0 } else { room / rows };
    let left = bx - bw / 2.0 + inset;
    let mut top = by - bh / 2.0 + inset + head;
    let mut row = 0usize;
    for &c in &children {
        let child = &mut cells[c];
        while (row as f64) < child.row {
            top += heights[row] * scale + share + gap;
            row += 1;
        }
        child.w = cw * child.span + gap * (child.span - 1.0);
        child.h = heights[row] * scale + share;
        child.x = left + child.col * (cw + gap) + child.w / 2.0;
        child.y = top + child.h / 2.0;
    }
    cells[b].heights = heights;
    cells[b].children = children;
}

/// Lays out a tree of cells in place. The root has no border, so its grid starts at the
/// origin. Gives the size of the whole.
pub(super) fn layout(cells: &mut [Cell], gap: f64) -> Result<(f64, f64), Error> {
    let mut order = vec![0usize];
    let mut i = 0;
    while i < order.len() {
        let children = cells[order[i]].children.clone();
        order.extend(children);
        i += 1;
    }
    for i in (0..order.len()).rev() {
        let b = order[i];
        if !cells[b].children.is_empty() {
            measure(cells, b, gap, if i == 0 { 0.0 } else { gap })?;
        }
    }
    let root = &mut cells[0];
    root.w = root.nw;
    root.h = root.nh;
    root.x = root.w / 2.0;
    root.y = root.h / 2.0;
    for (i, &b) in order.iter().enumerate() {
        if !cells[b].children.is_empty() {
            arrange(cells, b, gap, if i == 0 { 0.0 } else { gap });
        }
    }
    Ok((cells[0].w, cells[0].h))
}
