//! Git graph rendering. Ported from Pele `src/diagrams/git/render.ts`.
//!
//! Colors in resolved mode follow Zebra's `apply_theme`: each branch takes one color of
//! Zebra's `chart_series` for its lane, its connectors, its commits and its name tag, and the
//! name reads on that color; commit ids are text on the node fill; tags are text on the
//! secondary surface with a faint outline.

use std::collections::HashMap;

use super::{commit_type, GitCommit, GitDirection, GitModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit::plain_label;
use crate::diagrams::common::fit_width::turn_to_fit;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::{layout_label, style, Label};
use crate::theme::Role;
use crate::util::js;

const DOT: f64 = 7.0;
const STEP: f64 = 44.0;
const LANE: f64 = 44.0;
// Shorter than a step, so a curve between lanes never runs through a commit on a lane it crosses.
const CURVE: f64 = 36.0;
const GAP: f64 = 8.0;
const LABEL_GAP: f64 = 20.0;
const TAIL: f64 = 20.0;
const NOTCH: f64 = 7.0;
const TAG_GAP: f64 = 6.0;
const HALF: f64 = std::f64::consts::FRAC_1_SQRT_2;
const CLASSES: [&str; 5] = ["normal", "reverse", "highlight", "merge", "cherry-pick"];

struct View<'a> {
    commit: &'a GitCommit,
    lane: usize,
    m: f64,
    label: Option<Label>,
    tags: Vec<Label>,
}

fn flag(config: &Config, key: &str, fallback: bool) -> bool {
    config.lookup("gitGraph", key).and_then(ConfigValue::as_bool).unwrap_or(fallback)
}

fn tag_width(tag: &Label) -> f64 {
    NOTCH + 4.0 + tag.width + 6.0
}

/// The color of branch `lane`: Pele's series color in CSS mode, Zebra's branch color (its
/// `chart_series`) in resolved mode.
fn branch_color(ctx: &RenderCtx, lane: usize) -> String {
    ctx.theming.series_line(lane as i64)
}

fn symbol(ctx: &RenderCtx, kind: u8, x: f64, y: f64, color: &str) -> String {
    let at = format!("cx=\"{}\" cy=\"{}\"", num(x), num(y));
    if kind == commit_type::HIGHLIGHT {
        return format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{}\"/>",
            num(x - DOT - 1.0),
            num(y - DOT - 1.0),
            js::to_string(2.0 * DOT + 2.0),
            js::to_string(2.0 * DOT + 2.0),
            color
        );
    }
    if kind == commit_type::MERGE {
        // A ring: its middle shows the canvas.
        return format!(
            "<circle {} r=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/><circle {} r=\"2.5\" fill=\"{}\"/>",
            at,
            js::to_string(DOT - 1.0),
            ctx.paint(Role::Background, "var(--_bg)"),
            color,
            at,
            color
        );
    }
    let dot = format!("<circle {} r=\"{}\" fill=\"{}\"/>", at, js::to_string(DOT), color);
    // The marks of a reverse or cherry-pick are drawn on the branch color, so in resolved mode
    // they take the color that reads on it, as the branch name does.
    let mark = ctx.theming.text_on(color, "var(--_bg)");
    if kind == commit_type::REVERSE {
        return dot
            + &format!(
                "<path d=\"M{},{}l6,6M{},{}l6,-6\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\" stroke-linecap=\"round\"/>",
                num(x - 3.0),
                num(y - 3.0),
                num(x - 3.0),
                num(y + 3.0),
                mark
            );
    }
    if kind == commit_type::CHERRY_PICK {
        return dot
            + &format!(
                "<path d=\"M{},{}L{},{}L{},{}\" fill=\"none\" stroke=\"{}\" stroke-linejoin=\"round\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.8\" fill=\"{}\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.8\" fill=\"{}\"/>",
                num(x - 2.6),
                num(y + 1.0),
                num(x + 0.8),
                num(y - 4.2),
                num(x + 2.6),
                num(y + 1.0),
                mark,
                num(x - 2.6),
                num(y + 2.2),
                mark,
                num(x + 2.6),
                num(y + 2.2),
                mark
            );
    }
    dot
}

pub(crate) fn render(model: &GitModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    if model.direction != GitDirection::LR {
        return Ok(draw(model, config, ctx, model.direction));
    }
    turn_to_fit(ctx.options, |down| Ok(draw(model, config, ctx, if down { GitDirection::TB } else { GitDirection::LR })))
}

/// The box that everything drawn covers.
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    fn grow(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        if x0 < self.min_x {
            self.min_x = x0;
        }
        if y0 < self.min_y {
            self.min_y = y0;
        }
        if x1 > self.max_x {
            self.max_x = x1;
        }
        if y1 > self.max_y {
            self.max_y = y1;
        }
    }
}

fn draw(model: &GitModel, config: &Config, ctx: &RenderCtx, direction: GitDirection) -> Rendered {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let small = js::round(size * 0.75);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);

    let show_branches = flag(config, "showBranches", true);
    let show_commit_label = flag(config, "showCommitLabel", true);
    let rotate = flag(config, "rotateCommitLabel", true);
    let parallel = flag(config, "parallelCommits", false);
    let vertical = direction != GitDirection::LR;
    let flip = direction == GitDirection::BT;

    let lane_of: HashMap<&str, usize> = model.lanes.iter().enumerate().map(|(i, name)| (name.as_str(), i)).collect();
    let lane_count = model.lanes.len();
    let medium = js::round(size * 0.875);
    // Mermaid writes branch names, tags, and commit ids as plain text, so tags in them show as
    // written.
    let names: Vec<Label> =
        model.lanes.iter().map(|name| plain_label(Some(if show_branches { name } else { "" }), &m, medium, 4000.0, 0)).collect();

    let mut sorted: Vec<&GitCommit> = model.commits.values().collect();
    sorted.sort_by_key(|commit| commit.seq);
    let mut views: Vec<View> = Vec::with_capacity(sorted.len());
    let mut by_id: HashMap<&str, usize> = HashMap::new();
    for commit in sorted {
        let labelled = show_commit_label
            && commit.kind != commit_type::CHERRY_PICK
            && (commit.kind != commit_type::MERGE || commit.custom_id == Some(true));
        let mut tags = Vec::new();
        for tag in &commit.tags {
            let label = plain_label(Some(tag), &m, small, 4000.0, 0);
            if !label.lines.is_empty() {
                tags.push(label);
            }
        }
        by_id.insert(&commit.id, views.len());
        views.push(View {
            commit,
            lane: lane_of.get(commit.branch.as_str()).copied().unwrap_or(0),
            m: 0.0,
            label: if labelled { Some(plain_label(Some(&commit.id), &m, small, 4000.0, 0)) } else { None },
            tags,
        });
    }
    let count = views.len();

    // Room each commit needs: along the main axis on either side, and across it towards the
    // previous lane (`lo`) and the next one (`hi`). Tags sit on one side of a lane and ids on
    // the other.
    let mut half = vec![0.0; count];
    let mut tag_half = vec![0.0; count];
    let mut lo = vec![DOT; lane_count];
    let mut hi = vec![DOT; lane_count];
    for (k, view) in views.iter().enumerate() {
        let mut tags_main: f64 = 0.0;
        let mut tags_cross: f64 = 0.0;
        for tag in &view.tags {
            let w = tag_width(tag);
            if vertical {
                tags_main = js::max(tags_main, tag.height);
                tags_cross += w + 4.0;
            } else {
                tags_main = js::max(tags_main, w);
                tags_cross += tag.height + 3.0;
            }
        }
        let mut label_main = 0.0;
        let mut label_cross = 0.0;
        if let Some(label) = view.label.as_ref().filter(|l| !l.lines.is_empty()) {
            if rotate {
                label_cross = 3.0 + HALF * (label.width + label.height);
            } else {
                label_main = if vertical { label.height } else { label.width };
                label_cross = 6.0 + if vertical { label.width } else { label.height };
            }
        }
        half[k] = js::max(js::max(DOT, tags_main / 2.0), label_main / 2.0);
        tag_half[k] = tags_main / 2.0;
        let tag_side = DOT + if !view.tags.is_empty() { TAG_GAP + tags_cross } else { 0.0 };
        let label_side = DOT + label_cross;
        let lane = view.lane;
        if lane < lane_count {
            lo[lane] = js::max(lo[lane], if vertical { label_side } else { tag_side });
            hi[lane] = js::max(hi[lane], if vertical { tag_side } else { label_side });
        }
    }

    // In a left-to-right graph the tags stand over their commit. A connector that comes down
    // from a lane above joins the commit's lane this far before the commit, clear of the tags,
    // and the commit is placed far enough along to leave room for that.
    let mut lead = vec![0.0; count];
    if !vertical {
        for (k, view) in views.iter().enumerate() {
            if tag_half[k] == 0.0 {
                continue;
            }
            for id in &view.commit.parents {
                if let Some(&p) = by_id.get(id.as_str()) {
                    if views[p].lane < view.lane {
                        lead[k] = tag_half[k] + 4.0;
                    }
                }
            }
        }
    }
    let space = |p: usize, k: usize| -> f64 {
        js::max(js::max(STEP, half[p] + half[k] + GAP), if lead[k] > 0.0 { half[p] + CURVE + lead[k] } else { 0.0 })
    };

    for k in 0..count {
        if !parallel {
            views[k].m = if k == 0 { 0.0 } else { views[k - 1].m + space(k - 1, k) };
            continue;
        }
        // Mermaid's `parallelCommits`: a commit sits one step after its furthest parent, so
        // branches that grow side by side share positions instead of taking turns.
        let mut at: f64 = 0.0;
        for id in &views[k].commit.parents {
            let Some(&p) = by_id.get(id.as_str()) else { continue };
            if p >= k {
                continue;
            }
            at = js::max(at, views[p].m + space(p, k));
        }
        views[k].m = at;
    }
    let mut end: f64 = 0.0;
    for view in &views {
        end = js::max(end, view.m);
    }

    let mut cross = vec![0.0; lane_count];
    for i in 0..lane_count {
        let name = &names[i];
        let reach = if name.lines.is_empty() {
            0.0
        } else if vertical {
            name.width / 2.0 + 8.0
        } else {
            name.height / 2.0 + 2.0
        };
        lo[i] = js::max(lo[i], reach);
        hi[i] = js::max(hi[i], reach);
        if i > 0 {
            cross[i] = cross[i - 1] + js::max(LANE, hi[i - 1] + lo[i] + GAP);
        }
    }

    let px = |m: f64, c: f64| -> f64 {
        if vertical {
            c
        } else {
            m
        }
    };
    let py = |m: f64, c: f64| -> f64 {
        if vertical {
            if flip {
                -m
            } else {
                m
            }
        } else {
            c
        }
    };
    let point = |m: f64, c: f64| -> String { format!("{},{}", num(px(m, c)), num(py(m, c))) };

    let mut b = Bounds { min_x: f64::INFINITY, min_y: f64::INFINITY, max_x: f64::NEG_INFINITY, max_y: f64::NEG_INFINITY };

    let mut branches = String::new();
    if show_branches {
        for i in 0..lane_count {
            let name = &names[i];
            let color = branch_color(ctx, i);
            let c = cross[i];
            let w = name.width + 16.0;
            let h = name.height + 4.0;
            let along = if vertical { h } else { w };
            let from = if name.lines.is_empty() { -LABEL_GAP } else { -LABEL_GAP - along };
            let cx = px(from + along / 2.0, c);
            let cy = py(from + along / 2.0, c);
            b.grow(px(-LABEL_GAP, c), py(-LABEL_GAP, c), px(-LABEL_GAP, c), py(-LABEL_GAP, c));
            b.grow(px(end + TAIL, c), py(end + TAIL, c), px(end + TAIL, c), py(end + TAIL, c));
            branches.push_str(&format!(
                "<g class=\"pele-branch\" data-id=\"{}\"><path class=\"pele-lane\" d=\"M{}L{}\" stroke=\"{}\" stroke-dasharray=\"2 4\"/>",
                esc_text(&model.lanes[i]),
                point(-LABEL_GAP, c),
                point(end + TAIL, c),
                color
            ));
            if !name.lines.is_empty() {
                b.grow(cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0);
                branches.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"/>",
                    num(cx - w / 2.0),
                    num(cy - h / 2.0),
                    num(w),
                    num(h),
                    RADIUS,
                    color
                ));
                let text = ctx.theming.text_on(&color, "var(--_bg)");
                branches.push_str(&label_svg(name, cx, cy, &format!(" class=\"pele-label\" fill=\"{}\"", text), None));
            }
            branches.push_str("</g>");
        }
    }

    // Positions of the commits on each lane, to tell whether a connector may run along it.
    let mut taken: Vec<Vec<f64>> = vec![Vec::new(); lane_count];
    for view in &views {
        if let Some(list) = taken.get_mut(view.lane) {
            list.push(view.m);
        }
    }
    if parallel {
        for list in &mut taken {
            list.sort_by(|a, b| super::compare(*a, *b));
        }
    }
    let busy = |lane: usize, from: f64, to: f64| -> bool {
        let list = &taken[lane];
        let mut low = 0;
        let mut high = list.len();
        while low < high {
            let mid = (low + high) >> 1;
            if list[mid] <= from {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low < list.len() && list[low] < to
    };

    let mut curves = String::new();
    let mut lines = String::new();
    for (index, view) in views.iter().enumerate() {
        let commit = view.commit;
        let c2 = cross[view.lane];
        let m2 = view.m;
        for (j, id) in commit.parents.iter().enumerate() {
            let Some(&p) = by_id.get(id.as_str()) else { continue };
            if p == index {
                continue;
            }
            let parent = &views[p];
            let c1 = cross[parent.lane];
            let m1 = parent.m;
            if parent.lane == view.lane {
                lines.push_str(&format!(
                    "<path class=\"pele-edge\" d=\"M{}L{}\" stroke=\"{}\"/>",
                    point(m1, c1),
                    point(m2, c2),
                    branch_color(ctx, view.lane)
                ));
                continue;
            }
            let merged = j > 0 && commit.kind == commit_type::MERGE;
            let picked = j > 0 && !merged;
            let mut d = format!("M{}", point(m1, c1));
            if m2 <= m1 {
                let mid = (m1 + m2) / 2.0;
                d.push_str(&format!("C{} {} {}", point(mid, c1), point(mid, c2), point(m2, c2)));
            } else if !merged && !busy(view.lane, m1, m2) {
                // Leaves the parent at once and runs along the child's lane.
                let s = js::min(CURVE, m2 - m1);
                d.push_str(&format!("C{} {} {}L{}", point(m1 + s / 2.0, c1), point(m1 + s / 2.0, c2), point(m1 + s, c2), point(m2, c2)));
            } else {
                // Turns in just before the child, or before its tags when it comes from their side.
                let to = if parent.lane < view.lane && m2 - lead[index] > m1 { m2 - lead[index] } else { m2 };
                let mut s = js::min(CURVE, to - m1);
                let mut along = c1;
                if !merged || busy(parent.lane, m1, m2) {
                    // Neither lane is free to run along. Between two lanes, past the ids of one
                    // and before the tags of the next, a strip stays empty, and the connector
                    // takes that.
                    along = if c1 > c2 { c2 + hi[view.lane] + GAP / 2.0 } else { c2 - lo[view.lane] - GAP / 2.0 };
                    s = js::min(CURVE, (to - m1) / 2.0);
                    d.push_str(&format!("C{} {} {}", point(m1 + s / 2.0, c1), point(m1 + s / 2.0, along), point(m1 + s, along)));
                }
                d.push_str(&format!(
                    "L{}C{} {} {}",
                    point(to - s, along),
                    point(to - s / 2.0, along),
                    point(to - s / 2.0, c2),
                    point(to, c2)
                ));
                if to < m2 {
                    d.push_str(&format!("L{}", point(m2, c2)));
                }
            }
            curves.push_str(&format!(
                "<path class=\"pele-edge{}\" d=\"{}\" stroke=\"{}\"{}/>",
                if merged {
                    " pele-edge-merge"
                } else if picked {
                    " pele-edge-cherry-pick"
                } else {
                    ""
                },
                d,
                branch_color(ctx, if merged { parent.lane } else { view.lane }),
                if picked { " stroke-dasharray=\"4 3\"" } else { "" }
            ));
        }
    }

    let mut commits = String::new();
    for view in &views {
        let commit = view.commit;
        let x = px(view.m, cross[view.lane]);
        let y = py(view.m, cross[view.lane]);
        let kind = commit.custom_type.unwrap_or(commit.kind);
        b.grow(x - DOT - 1.0, y - DOT - 1.0, x + DOT + 1.0, y + DOT + 1.0);
        // Merges and cherry-picks carry a message Mermaid makes up; only a written one is worth
        // showing.
        let written = commit.kind < commit_type::MERGE && !commit.message.is_empty();
        let mut body = if written { format!("<title>{}</title>", esc_text(&commit.message)) } else { String::new() };
        body.push_str(&symbol(ctx, kind, x, y, &branch_color(ctx, view.lane)));

        if let Some(label) = view.label.as_ref().filter(|l| !l.lines.is_empty()) {
            let w = label.width;
            let h = label.height;
            // The patch under a commit id: Zebra's commit label background.
            let back = |cx: f64, cy: f64| -> String {
                format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\"/>",
                    num(cx - w / 2.0 - 2.0),
                    num(cy - h / 2.0 + 2.0),
                    num(w + 4.0),
                    num(h - 4.0),
                    ctx.paint(Role::NodeFill, "var(--_bg)")
                )
            };
            if rotate {
                // The text ends beside the commit and trails back the way the history came.
                let k = (HALF * h) / 2.0;
                let ax = if vertical { x - DOT - 3.0 - k } else { x + 3.0 };
                let ay = if vertical { y } else { y + DOT + 3.0 + k };
                let reach = HALF * w + k;
                b.grow(ax - reach, if flip { ay - reach } else { ay - k }, ax + k, if flip { ay + k } else { ay + reach });
                body.push_str(&format!(
                    "<g class=\"pele-commit-label\" transform=\"rotate({} {} {})\">{}{}</g>",
                    if flip { 45 } else { -45 },
                    num(ax),
                    num(ay),
                    back(ax - w / 2.0, ay),
                    label_svg(label, ax - w / 2.0, ay, "", None)
                ));
            } else {
                let cx = if vertical { x - DOT - 6.0 - w / 2.0 } else { x };
                let cy = if vertical { y } else { y + DOT + 6.0 + h / 2.0 };
                b.grow(cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0);
                body.push_str(&format!("<g class=\"pele-commit-label\">{}{}</g>", back(cx, cy), label_svg(label, cx, cy, "", None)));
            }
        }

        // Beside the lane the tags form a row; above it they stack, the first one furthest out.
        let tags = &view.tags;
        let order: Vec<usize> = if vertical { (0..tags.len()).collect() } else { (0..tags.len()).rev().collect() };
        let mut offset = DOT + TAG_GAP;
        for t in order {
            let tag = &tags[t];
            let w = tag_width(tag);
            let h = tag.height;
            let left = if vertical { x + offset } else { x - w / 2.0 };
            let cy = if vertical { y } else { y - offset - h / 2.0 };
            offset += if vertical { w + 4.0 } else { h + 3.0 };
            b.grow(left, cy - h / 2.0, left + w, cy + h / 2.0);
            let outline = ctx.paint(Role::FaintText, "var(--_b)");
            body.push_str(&format!(
                "<g class=\"pele-tag\"><path d=\"M{},{}L{},{}H{}V{}H{}Z\" fill=\"{}\" stroke=\"{}\" stroke-linejoin=\"round\"/><circle cx=\"{}\" cy=\"{}\" r=\"1.5\" fill=\"{}\"/>{}</g>",
                num(left),
                num(cy),
                num(left + NOTCH),
                num(cy - h / 2.0),
                num(left + w),
                num(cy + h / 2.0),
                num(left + NOTCH),
                ctx.paint(Role::GroupHeader, "var(--_s)"),
                outline,
                num(left + NOTCH),
                num(cy),
                outline,
                label_svg(
                    tag,
                    left + NOTCH + 4.0 + tag.width / 2.0,
                    cy,
                    &format!(" fill=\"{}\"", ctx.paint(Role::Text, "var(--_fg)")),
                    None
                )
            ));
        }

        let merge = if commit.kind == commit_type::MERGE && kind != commit_type::MERGE { " pele-commit-merge" } else { "" };
        commits.push_str(&format!(
            "<g class=\"pele-commit pele-commit-{}{}\" data-id=\"{}\">{}</g>",
            CLASSES.get(kind as usize).unwrap_or(&CLASSES[0]),
            merge,
            esc_text(&commit.id),
            body
        ));
    }

    if b.min_x > b.max_x {
        b.grow(0.0, 0.0, 0.0, 0.0);
    }
    let title = layout_label(model.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let inner = js::max(b.max_x - b.min_x, title.width);
    let width = (inner + 2.0 * pad).ceil();
    let height = (b.max_y - b.min_y + title_height + 2.0 * pad).ceil();
    let dx = pad - b.min_x + (inner - (b.max_x - b.min_x)) / 2.0;
    let dy = pad + title_height - b.min_y;

    let mut content = label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None);
    content.push_str(&format!("<g transform=\"translate({},{})\">", num(dx), num(dy)));
    if !branches.is_empty() {
        content.push_str(&format!("<g class=\"pele-branches\" fill=\"none\" font-size=\"{}\">{}</g>", js::to_string(medium), branches));
    }
    if !curves.is_empty() || !lines.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-edges\" fill=\"none\" stroke-width=\"2\" stroke-linecap=\"round\">{}{}</g>",
            curves, lines
        ));
    }
    if !commits.is_empty() {
        // The group's fill is the color of the commit ids, the only text that inherits it.
        content.push_str(&format!(
            "<g class=\"pele-commits\" font-size=\"{}\" fill=\"{}\">{}</g>",
            js::to_string(small),
            ctx.paint(Role::Text, "var(--_m)"),
            commits
        ));
    }
    content.push_str("</g>");

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered { svg: svg_document("gitGraph", width, height, size, ctx, acc, &content), width, height, links: Vec::new() }
}
