//! Cluster titles that stay clear of the edges entering their cluster. Ported from Pele
//! `src/svg/cluster.ts`. Functions take a cluster's center and size instead of a layout node.

use crate::svg::concat::svg_format;
use std::collections::HashMap;

use super::builder::{num, view_label_svg};
use crate::text::render_label::{LabelRead, LabelView};
use crate::text::RenderLabel;
use crate::text::{Icons, Label};
use crate::theme::{Role, Theming};
use crate::util::js;

/// The x of every route point, by its rounded y: the edges that cross a cluster's top border.
pub type Crossings = HashMap<i64, Vec<f64>>;

pub fn mark_crossings(crossings: &mut Crossings, route: &[f64]) {
    let mut k = 0;
    while k + 1 < route.len() {
        let key = js::round(route[k + 1]) as i64;
        crossings.entry(key).or_default().push(route[k]);
        k += 3;
    }
}

/// Picks where a cluster title sits along the top edge so that edges entering there do not
/// cross it. The cluster is centered on (`cx`, `cy`) with size `cw` by `ch`.
pub fn cluster_title_x(cx: f64, cy: f64, cw: f64, ch: f64, width: f64, crossings: &Crossings, pad: f64) -> f64 {
    if width == 0.0 {
        return cx;
    }
    let top = cy - ch / 2.0;
    let left = cx - cw / 2.0;
    let half = width / 2.0;
    let candidates = [left + pad - 6.0 + half, cx, left + cw - pad + 6.0 - half];
    let Some(xs) = crossings.get(&(js::round(top) as i64)) else {
        return candidates[0];
    };
    let mut best = candidates[0];
    let mut best_gap = -1.0;
    for &x in &candidates {
        let mut gap = f64::INFINITY;
        for &c in xs {
            if c > left && c < left + cw {
                gap = js::min(gap, (c - x).abs() - half);
            }
        }
        if gap >= 6.0 {
            return x;
        }
        if gap > best_gap + 0.5 {
            best_gap = gap;
            best = x;
        }
    }
    best
}

/// Whether an edge that enters at the top runs through a title placed at `x`.
pub fn title_crossed(cx: f64, cy: f64, cw: f64, ch: f64, width: f64, x: f64, crossings: &Crossings) -> bool {
    if width <= 0.0 {
        return false;
    }
    let Some(xs) = crossings.get(&(js::round(cy - ch / 2.0) as i64)) else { return false };
    let left = cx - cw / 2.0;
    xs.iter().any(|&c| c > left && c < left + cw && (c - x).abs() < width / 2.0 + 4.0)
}

/// A title that an edge runs through, drawn over the edges: the text on an outline in the
/// color of the cluster behind it, so that only the line is seen to stop short of the letters.
/// `parent_of` and `index` find how many clusters deep the title is; in CSS mode each level
/// tints the background, as in Pele. In resolved mode the outline is the group fill, which
/// every level of a resolved cluster uses.
#[allow(clippy::too_many_arguments)]
pub fn struck_title(
    label: &Label,
    x: f64,
    y: f64,
    attrs: &str,
    id: &str,
    parent_of: impl Fn(usize) -> isize,
    index: usize,
    icons: Option<&Icons>,
    theming: &Theming,
) -> String {
    struck_title_view(label.view(), x, y, attrs, id, parent_of, index, icons, theming)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_struck_title(
    label: &RenderLabel,
    x: f64,
    y: f64,
    attrs: &str,
    id: &str,
    parent_of: impl Fn(usize) -> isize,
    index: usize,
    icons: Option<&Icons>,
    theming: &Theming,
) -> String {
    struck_title_view(label.view(), x, y, attrs, id, parent_of, index, icons, theming)
}

#[allow(clippy::too_many_arguments)]
fn struck_title_view(
    label: LabelView<'_>,
    x: f64,
    y: f64,
    attrs: &str,
    id: &str,
    parent_of: impl Fn(usize) -> isize,
    index: usize,
    icons: Option<&Icons>,
    theming: &Theming,
) -> String {
    let mut depth = 1;
    let mut p = parent_of(index);
    while p >= 0 && depth < 8 {
        depth += 1;
        p = parent_of(p as usize);
    }
    // Each cluster is half its fill over what is behind it.
    let css = format!("color-mix(in srgb,var(--_a) {}%,var(--_bg))", num(100.0 * (1.0 - crate::util::math::pow(0.5, depth as f64))));
    let fill = theming.paint(Role::GroupFill, &css).to_string();
    svg_format!(
        "<g class=\"pele-cluster-title\" data-id=\"",
        id,
        "\">",
        view_label_svg(
            label,
            x,
            y,
            &svg_format!(" fill=\"none\" stroke=\"", fill, "\" stroke-width=\"5\" stroke-linejoin=\"round\""),
            icons
        ),
        view_label_svg(label, x, y, attrs, icons),
        "</g>"
    )
}
