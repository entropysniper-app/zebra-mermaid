//! Venn rendering. Ported from Pele `src/diagrams/venn/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::db::normalize_text;
use super::layout::{find_spot, layout_circles, region_path, Circle, Spot};
use super::{VennModel, VennSubset};
use crate::config::Config;
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{resolve_style, ResolvedStyle};
use crate::text::{layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::color::mix_hex;
use crate::theme::Role;
use crate::util::{js, math};
use std::fmt::Write as _;

const RADIUS: f64 = 100.0;
const MAX_GROWTH: f64 = 1.6;
/// The opacity Pele fills a set's circle with.
const SET_OPACITY: f32 = 0.2;

struct Line {
    /// The id of a text; `None` for the region's own label.
    id: Option<String>,
    raw: String,
    size: f64,
    base: u8,
    label: Label,
}

struct Region {
    sets: Vec<String>,
    inside: Vec<usize>,
    spot: Spot,
    // What is written in the region: its label first, then its texts.
    lines: Vec<Line>,
    style: ResolvedStyle,
}

/// One key for a list of set ids. Pele writes a single id as itself and a list as its JSON; ids
/// cannot hold a double quote, so the list itself is as good a key.
fn key_of(ids: &[String]) -> Vec<String> {
    ids.to_vec()
}

pub(crate) fn render(model: &VennModel, _config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let tiny = js::round(size * 0.8125);
    let m = ctx.measure();
    let pad = ctx.padding();

    let circles = layout_circles(&model.subsets);
    let mut index: HashMap<&str, usize> = HashMap::new();
    for (i, c) in circles.iter().enumerate() {
        index.insert(&c.id, i);
    }

    // Styles for the same targets add up, later declarations winning.
    let mut styles: HashMap<Vec<String>, IndexMap<&str, &str>> = HashMap::new();
    for declared in &model.styles {
        let merged = styles.entry(key_of(&declared.targets)).or_default();
        for (property, value) in &declared.styles {
            merged.insert(property, value);
        }
    }
    let style_of = |key: &[String]| -> ResolvedStyle {
        match styles.get(key) {
            None => ResolvedStyle::default(),
            Some(declared) => {
                let list: Vec<String> = declared.iter().map(|(property, value)| format!("{}:{}", property, value)).collect();
                resolve_style(&list)
            }
        }
    };

    // Circles that touch each circle; only those can cut into a region of it.
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); circles.len()];
    for a in 0..circles.len() {
        for b in a + 1..circles.len() {
            if math::hypot(circles[a].x - circles[b].x, circles[a].y - circles[b].y) < circles[a].r + circles[b].r {
                neighbors[a].push(b);
                neighbors[b].push(a);
            }
        }
    }

    // One region for each set and each stated overlap; a later statement about the same sets replaces an earlier one.
    let mut stated: IndexMap<Vec<String>, &VennSubset> = IndexMap::new();
    for subset in &model.subsets {
        stated.insert(key_of(&subset.sets), subset);
    }
    let mut regions: IndexMap<Vec<String>, Region> = IndexMap::new();
    for (key, subset) in stated {
        let mut sets: Vec<String> = Vec::new();
        for id in &subset.sets {
            if !sets.contains(id) {
                sets.push(id.clone());
            }
        }
        // Every set named in a subset has a circle.
        let inside: Vec<usize> = sets.iter().filter_map(|id| index.get(id.as_str()).copied()).collect();
        let within: HashSet<usize> = inside.iter().copied().collect();
        let mut smallest = inside[0];
        for &i in &inside {
            if circles[i].r < circles[smallest].r {
                smallest = i;
            }
        }
        let style = style_of(&key);
        let text = match &subset.label {
            Some(label) => Some(label.as_str()),
            None if subset.sets.len() == 1 => Some(subset.sets[0].as_str()),
            None => None,
        };
        let base = (if style.bold { style::BOLD } else { 0 }) | (if style.italic { style::ITALIC } else { 0 });
        let outside: Vec<usize> = neighbors[smallest].iter().copied().filter(|i| !within.contains(i)).collect();
        let mut spot = find_spot(&circles, &inside, &outside);
        // A region that other circles cover, or all but cover, is written across them instead.
        if spot.margin < circles[smallest].r * 0.2 {
            spot = find_spot(&circles, &inside, &[]);
        }
        let mut region = Region { sets, inside, spot, lines: Vec::new(), style };
        if let Some(text) = text.filter(|t| !t.is_empty()) {
            let font_size = if region.sets.len() == 1 { size } else { small };
            region.lines.push(Line {
                id: None,
                raw: text.to_string(),
                size: font_size,
                base,
                label: layout_label(Some(text), false, &m, font_size, 4000.0, base),
            });
        }
        regions.insert(key, region);
    }
    for node in &model.text_nodes {
        let raw = node.label.as_deref().unwrap_or(&node.id);
        if let Some(region) = regions.get_mut(&key_of(&node.sets)) {
            region.lines.push(Line {
                id: Some(node.id.clone()),
                raw: raw.to_string(),
                size: tiny,
                base: 0,
                label: layout_label(Some(raw), false, &m, tiny, 4000.0, 0),
            });
        }
    }

    // Lengths so far are in units of the layout. The scale gives the largest circle a set radius,
    // and grows, within limits, until every region holds what is written in it.
    let mut largest = 0.0;
    for c in &circles {
        if c.r > largest {
            largest = c.r;
        }
    }
    let mut scale = if largest > 0.0 { RADIUS / largest } else { 1.0 };
    let mut growth: f64 = 1.0;
    for region in regions.values() {
        if region.spot.margin <= 0.0 {
            continue;
        }
        let mut width: f64 = 0.0;
        let mut height = 0.0;
        for line in &region.lines {
            width = js::max(width, line.label.width);
            height += line.label.height;
        }
        let need = math::hypot(width, height) / 2.0 + 4.0;
        growth = js::max(growth, js::min(need / (region.spot.margin * scale), MAX_GROWTH));
    }
    scale *= growth;

    // What is still too wide for its region is wrapped to it.
    for region in regions.values_mut() {
        let room = js::max(region.spot.margin * scale * 1.8, 72.0);
        for line in &mut region.lines {
            if line.label.width > room {
                line.label = layout_label(Some(&line.raw), false, &m, line.size, room, line.base);
            }
        }
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for c in &circles {
        min_x = js::min(min_x, (c.x - c.r) * scale);
        min_y = js::min(min_y, (c.y - c.r) * scale);
        max_x = js::max(max_x, (c.x + c.r) * scale);
        max_y = js::max(max_y, (c.y + c.r) * scale);
    }
    for region in regions.values() {
        let mut wide: f64 = 0.0;
        let mut tall = 0.0;
        for line in &region.lines {
            wide = js::max(wide, line.label.width);
            tall += line.label.height;
        }
        min_x = js::min(min_x, region.spot.x * scale - wide / 2.0);
        max_x = js::max(max_x, region.spot.x * scale + wide / 2.0);
        min_y = js::min(min_y, region.spot.y * scale - tall / 2.0);
        max_y = js::max(max_y, region.spot.y * scale + tall / 2.0);
    }
    if circles.is_empty() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 0.0;
        max_y = 0.0;
    }

    let title_text = model.title.as_deref().filter(|t| !t.is_empty()).map(normalize_text);
    let title = layout_label(title_text.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let inner = js::max(max_x - min_x, title.width);
    let ox = pad + (inner - (max_x - min_x)) / 2.0 - min_x;
    let oy = pad + title_height - min_y;
    let width = (inner + 2.0 * pad).ceil();
    let height = (max_y - min_y + title_height + 2.0 * pad).ceil();
    let placed: Vec<Circle> =
        circles.iter().map(|c| Circle { id: c.id.clone(), x: c.x * scale + ox, y: c.y * scale + oy, r: c.r * scale }).collect();

    let no_style = ResolvedStyle::default();
    let colors: Vec<String> = (0..placed.len()).map(|i| ctx.series(i as i64).into_owned()).collect();
    let mut sets = String::new();
    for (i, c) in placed.iter().enumerate() {
        let color = &colors[i];
        let style = regions.get(&vec![c.id.clone()]).map_or(&no_style, |r| &r.style);
        svg_write!(
            sets,
            "<circle class=\"pele-venn-set\" data-id=\"",
            esc_text(&c.id),
            "\" cx=\"",
            num(c.x),
            "\" cy=\"",
            num(c.y),
            "\" r=\"",
            num(c.r),
            "\" fill=\"",
            color,
            "\" fill-opacity=\"0.2\" stroke=\"",
            color,
            "\"",
            style.shape,
            "/>"
        )
        .unwrap();
    }

    // In resolved mode a region's own label takes a color that reads on what is drawn under it:
    // the canvas tinted by each of the region's circles at the circles' opacity. Pele lets the
    // label inherit the text color, which is what this picks unless the tint is strong. The
    // tints are laid in drawing order; a fill an author's style gives a set is not looked at,
    // and such a style can set the label's own color.
    let label_fill = |inside: &[usize]| -> String {
        if !ctx.resolved() {
            return String::new();
        }
        let t = &ctx.theming.resolved;
        let mut under = t.background.clone();
        let mut sorted = inside.to_vec();
        sorted.sort_unstable();
        for i in sorted {
            under = mix_hex(&colors[i], &under, SET_OPACITY);
        }
        svg_format!(" fill=\"", t.text_on(&under), "\"")
    };
    let text_fill = ctx.paint(Role::SecondaryText, "var(--_m)");

    let mut overlaps = String::new();
    let mut labels = String::new();
    for region in regions.values() {
        let id = esc_text(&region.sets.join("|")).into_owned();
        // An overlap is only drawn when a style gives it a look of its own; otherwise the circles show it.
        if region.sets.len() > 1 && !region.style.shape.is_empty() {
            let outline: Vec<&Circle> = region.inside.iter().map(|&i| &placed[i]).collect();
            let d = region_path(&outline, |v| num(v).to_string());
            if !d.is_empty() {
                svg_write!(
                    overlaps,
                    "<path class=\"pele-venn-overlap\" data-id=\"",
                    id,
                    "\" d=\"",
                    d,
                    "\" fill=\"none\"",
                    region.style.shape,
                    "/>"
                )
                .unwrap();
            }
        }
        let mut total = 0.0;
        for line in &region.lines {
            total += line.label.height;
        }
        if total == 0.0 {
            continue;
        }
        let x = region.spot.x * scale + ox;
        let mut y = region.spot.y * scale + oy - total / 2.0;
        let mut content = String::new();
        for line in &region.lines {
            let attrs = match &line.id {
                None => svg_format!(
                    " class=\"pele-label\"",
                    if line.size == size { String::new() } else { svg_format!(" font-size=\"", js::to_string(line.size), "\"") },
                    label_fill(&region.inside),
                    region.style.text
                ),
                Some(id) => svg_format!(
                    " class=\"pele-venn-text\" data-id=\"",
                    esc_text(id),
                    "\" font-size=\"",
                    js::to_string(tiny),
                    "\" fill=\"",
                    text_fill,
                    "\"",
                    style_of(std::slice::from_ref(id)).text
                ),
            };
            write!(content, "{}", label_svg(&line.label, x, y + line.label.height / 2.0, &attrs, None)).unwrap();
            y += line.label.height;
        }
        svg_write!(labels, "<g class=\"pele-venn-region\" data-id=\"", id, "\">", content, "</g>").unwrap();
    }

    let mut content =
        label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
    svg_write!(content, "<g class=\"pele-venn-sets\">", sets, "</g>").unwrap();
    if !overlaps.is_empty() {
        svg_write!(content, "<g class=\"pele-venn-overlaps\">", overlaps, "</g>").unwrap();
    }
    if !labels.is_empty() {
        svg_write!(content, "<g class=\"pele-venn-labels\">", labels, "</g>").unwrap();
    }
    Ok(Rendered { svg: svg_document("venn", width, height, size, ctx, Accessible::default(), &content), width, height, links: Vec::new() })
}
