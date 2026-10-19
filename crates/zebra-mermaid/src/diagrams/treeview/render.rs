//! TreeView rendering. Ported from Pele `src/diagrams/treeview/render.ts`.

use super::icons::{get_node_icon, glyph, icon_config, BUILTIN_PACK};
use super::{NodeType, TreeViewModel};
use crate::config::{Config, ConfigValue};
use crate::diagrams::common::fit_width::{tighten, title_room};
use crate::diagrams::common::literal_label::{left_label_svg, literal_label};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc, esc_text, label_svg, num};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::{class_names, RADIUS};
use crate::text::entities::decode_entities;
use crate::text::{layout_label, style, Label};
use crate::theme::Role;
use crate::util::chars::is_space;
use crate::util::js;
use std::fmt::Write as _;

const ICON_GAP: f64 = 6.0;
const DESC_GAP: f64 = 16.0;
const GUIDE_INSET: f64 = 7.0;
const ELBOW: f64 = 4.0;
const HIGHLIGHT_OVERHANG: f64 = 4.0;

/// A number from the `treeView` section of the configuration, used only when it is in range.
fn setting(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("treeView", key).and_then(ConfigValue::as_f64) {
        Some(value) if value >= min && value <= max => value,
        _ => fallback,
    }
}

/// Mermaid's layout options, with Pele's defaults.
struct TreeSettings {
    padding_x: f64,
    padding_y: f64,
    row_indent: f64,
    line_thickness: f64,
}

fn tree_settings(config: &Config) -> TreeSettings {
    TreeSettings {
        padding_x: setting(config, "paddingX", 5.0, 0.0, 100.0),
        padding_y: setting(config, "paddingY", 2.0, 0.0, 100.0),
        row_indent: setting(config, "rowIndent", 15.0, 0.0, 400.0),
        line_thickness: setting(config, "lineThickness", 1.0, 0.25, 12.0),
    }
}

struct Row {
    node: usize,
    x: f64,
    cy: f64,
    icon: Option<String>,
    text: String,
    label_x: f64,
    highlighted: bool,
    label: Option<Label>,
    description: Option<Label>,
    top: f64,
    height: f64,
}

pub(crate) fn render(model: &TreeViewModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    tighten(ctx.options, |tight| Ok(draw(model, config, ctx, tight)))
}

/// `cssClass.split(/\s+/).includes('highlight')`
fn has_highlight(css_class: &str) -> bool {
    css_class.split(is_space).any(|part| part == "highlight")
}

fn draw(model: &TreeViewModel, config: &Config, ctx: &RenderCtx, tight: bool) -> Rendered {
    let options = ctx.options;
    let size = ctx.font_size();
    let m = ctx.measure();
    let pad = ctx.padding();
    let icons = icon_config(config);
    let nodes = &model.nodes;
    let root = &nodes[0];

    let TreeSettings { padding_x, padding_y, row_indent, line_thickness } = tree_settings(config);
    let mut max_depth = 0.0;
    if tight {
        let mut depths: Vec<(usize, f64)> = root.children.iter().map(|&node| (node, 0.0)).collect();
        let mut i = 0;
        while i < depths.len() {
            let (node, depth) = depths[i];
            max_depth = js::max(max_depth, depth);
            for &child in &nodes[node].children {
                depths.push((child, depth + 1.0));
            }
            i += 1;
        }
    }
    let max_width = options.max_width.unwrap_or(320.0);
    let step = if tight {
        js::min(row_indent + padding_x, js::max(2.0, (max_width - 120.0) / js::max(1.0, max_depth)))
    } else {
        row_indent + padding_x
    };
    let row_height = js::round(size * 1.5) + 2.0 * padding_y;
    let icon_size = size;

    let title = layout_label(model.title.as_deref(), false, &m, size, if tight { title_room(options) } else { 4000.0 }, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };

    let node_icon = |index: usize| {
        let node = &nodes[index];
        get_node_icon(node.icon.as_deref(), &node.name, node.node_type, &icons)
    };
    // Siblings line up: when one entry of a folder has an icon, the others leave room for one.
    let has_icon = |list: &[usize]| list.iter().any(|&child| node_icon(child).is_some());
    let mut rows: Vec<Row> = Vec::new();
    let mut row_of: Vec<usize> = vec![usize::MAX; nodes.len()];
    let mut pending: Vec<(usize, f64, bool)> = Vec::new();
    let push = |pending: &mut Vec<(usize, f64, bool)>, list: &[usize], depth: f64| {
        let slot = has_icon(list);
        for &child in list.iter().rev() {
            pending.push((child, depth, slot));
        }
    };
    push(&mut pending, &root.children, 0.0);
    let mut label_right: f64 = 0.0;
    let mut any_highlight = false;
    let mut rows_height = 0.0;
    while let Some((index, depth, slot)) = pending.pop() {
        let node = &nodes[index];
        let directory = node.node_type == NodeType::Directory;
        let weight = if directory { style::BOLD } else { 0 };
        let x = depth * step;
        let text = decode_entities(&node.name);
        let label_x = x + if slot { icon_size + ICON_GAP } else { 0.0 };
        let room = js::max(40.0, max_width - 2.0 * pad - 2.0 * HIGHLIGHT_OVERHANG - label_x);
        let label = if tight { Some(literal_label(&text, &m, size, room, weight)) } else { None };
        let description = match &node.description {
            Some(d) if tight && !d.is_empty() => Some(literal_label(&decode_entities(d), &m, size, room, style::ITALIC)),
            _ => None,
        };
        let width = match &label {
            Some(label) => label.width,
            None => m.width(&text, size, weight),
        };
        if label_x + width > label_right {
            label_right = label_x + width;
        }
        let highlighted = node.css_class.as_deref().is_some_and(has_highlight);
        if highlighted {
            any_highlight = true;
        }
        let label_height = label.as_ref().map_or(0.0, |l| l.height);
        let height = if tight {
            js::max(row_height, label_height + description.as_ref().map_or(0.0, |d| d.height) + 2.0 * padding_y)
        } else {
            row_height
        };
        let cy = rows_height + if tight { padding_y + js::max(icon_size, label_height) / 2.0 } else { height / 2.0 };
        row_of[index] = rows.len();
        rows.push(Row {
            node: index,
            x,
            cy,
            icon: node_icon(index),
            text,
            label_x,
            highlighted,
            label,
            description,
            top: rows_height,
            height,
        });
        rows_height += height;
        push(&mut pending, &node.children, depth + 1.0);
    }

    let desc_x = label_right + DESC_GAP;
    let mut width = label_right;
    for row in &rows {
        let Some(description) = nodes[row.node].description.as_deref().filter(|d| !d.is_empty()) else { continue };
        let w = match &row.description {
            Some(d) => d.width,
            None => m.width(&decode_entities(description), size, style::ITALIC),
        };
        width = js::max(width, (if tight { row.label_x } else { desc_x }) + w);
    }
    width = js::max(width, title.width);
    let inset = pad + if any_highlight { HIGHLIGHT_OVERHANG } else { 0.0 };
    let top = pad + title_height;
    let total_width = (width + 2.0 * inset).ceil();
    let total_height = (top + rows_height + pad).ceil();

    let mut guides = String::new();
    for row in &rows {
        let children = &nodes[row.node].children;
        let Some(&last_child) = children.last() else { continue };
        let gx = inset + row.x + GUIDE_INSET;
        let tick = js::max(gx, inset + row.x + step - padding_x);
        let last = top + rows[row_of[last_child]].cy;
        let r = js::min(ELBOW, tick - gx);
        let mut d = format!("M{},{}V{}", num(gx), num(top + row.top + row.height - 2.0), num(last - r));
        if r > 0.0 {
            write!(d, "Q{},{} {},{}H{}", num(gx), num(last), num(gx + r), num(last), num(tick)).unwrap();
        }
        let mut i = 0;
        while i + 1 < children.len() && tick > gx {
            write!(d, "M{},{}H{}", num(gx), num(top + rows[row_of[children[i]]].cy), num(tick)).unwrap();
            i += 1;
        }
        write!(guides, "<path d=\"{}\"/>", d).unwrap();
    }

    let baseline = size * 0.35;
    let muted = ctx.paint(Role::SecondaryText, "var(--_m)");
    let mut body = String::new();
    for row in &rows {
        let node = &nodes[row.node];
        let cy = top + row.cy;
        let directory = node.node_type == NodeType::Directory;
        let mut inner = String::new();
        if row.highlighted {
            let x = inset + row.x - HIGHLIGHT_OVERHANG;
            // A highlighted row is a selected one. Pele tints the accent; resolved output fills
            // the row with the selection color, which is made to sit under text.
            let fill = if ctx.resolved() {
                format!("fill=\"{}\"", ctx.paint(Role::Selection, "var(--_c)"))
            } else {
                "fill=\"var(--_c)\" fill-opacity=\"0.14\"".to_string()
            };
            write!(inner, 
                "<rect class=\"pele-tree-highlight\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" {}/>",
                num(x),
                num(top + row.top + 1.0),
                num(total_width - inset + HIGHLIGHT_OVERHANG - x),
                num(row.height - 2.0),
                RADIUS,
                fill
            ).unwrap();
        }
        if let Some(icon) = &row.icon {
            let name = icon.strip_prefix(BUILTIN_PACK).and_then(|rest| rest.strip_prefix(':')).unwrap_or("");
            let outline = glyph(name).or_else(|| glyph(if directory { "folder" } else { "file" })).unwrap_or("");
            // `options.icons?.(row.icon)`. Pele's render hands diagrams a resolver that sees the
            // name with entity codes turned into characters, and keeps its answer when there is
            // one, even an empty one; `Icons::markup` cannot tell no answer from an empty one.
            let drawn = match options.icons.as_ref().and_then(|resolver| resolver.resolve(&decode_entities(icon))) {
                Some(markup) => markup,
                None => format!("<path d=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>", outline, muted),
            };
            write!(inner, 
                "<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
                esc_text(icon),
                num(inset + row.x),
                num(cy - icon_size / 2.0),
                num(icon_size),
                num(icon_size),
                drawn
            ).unwrap();
        }
        let weight = if directory { " font-weight=\"var(--_hw)\"" } else { "" };
        if let Some(label) = &row.label {
            inner.push_str(&left_label_svg(
                label,
                inset + row.label_x,
                top + row.top + padding_y,
                &format!(" class=\"pele-label\"{}", weight),
            ));
        } else if !row.text.is_empty() {
            write!(inner, 
                "<text class=\"pele-label\" x=\"{}\" y=\"{}\"{} xml:space=\"preserve\">{}</text>",
                num(inset + row.label_x),
                num(cy + baseline),
                weight,
                esc(&row.text)
            ).unwrap();
        }
        if let Some(description) = &row.description {
            inner.push_str(&left_label_svg(
                description,
                inset + row.label_x,
                top + row.top + padding_y + row.label.as_ref().map_or(0.0, |l| l.height),
                &format!(" class=\"pele-tree-description\" font-style=\"italic\" fill=\"{}\"", muted),
            ));
        } else if let Some(description) = node.description.as_deref().filter(|d| !d.is_empty()) {
            write!(inner, 
                "<text class=\"pele-tree-description\" x=\"{}\" y=\"{}\" font-style=\"italic\" fill=\"{}\" xml:space=\"preserve\">{}</text>",
                num(inset + desc_x),
                num(cy + baseline),
                muted,
                esc(&decode_entities(description))
            ).unwrap();
        }
        write!(body, 
            "<g class=\"pele-node pele-tree-{}{}\" data-id=\"{}\">{}</g>",
            if directory { "dir" } else { "file" },
            class_names(node.css_class.as_deref().unwrap_or("")),
            esc_text(&node.name),
            inner
        ).unwrap();
    }

    let mut content =
        label_svg(&title, total_width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", ctx.icons()).to_string();
    if !guides.is_empty() {
        // The guides connect each folder to its entries.
        write!(content, 
            "<g class=\"pele-tree-guides\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\">{}</g>",
            ctx.paint(Role::Line, "var(--_b)"),
            num(line_thickness),
            guides
        ).unwrap();
    }
    content.push_str(&body);

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Rendered {
        svg: svg_document("treeView", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    }
}
