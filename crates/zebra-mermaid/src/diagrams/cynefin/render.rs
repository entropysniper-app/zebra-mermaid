//! Cynefin rendering. Ported from Pele `src/diagrams/cynefin/render.ts`.

use crate::svg::concat::{svg_format, svg_write};
use std::collections::HashMap;

use super::boundaries::{backward, cliff, curve_path, ellipse_path, forward, resolve_seed, wave, WaveOptions, FOLD, HORIZON};
use super::CynefinModel;
use crate::config::{Config, ConfigValue};
use crate::diagrams::{RenderCtx, Rendered};
use crate::error::Error;
use crate::svg::builder::{esc_text, num, render_label_svg as label_svg};
use crate::svg::edges::{marker, marker_trim};
use crate::svg::root::{svg_document, Accessible};
use crate::svg::theme::RADIUS;
use crate::text::{layout_render_label as layout_label, style, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, math};

/// (domain, column, row, series color)
const QUADRANTS: [(&str, f64, f64, i64); 4] =
    [("complex", 0.0, 0.0, 0), ("complicated", 1.0, 0.0, 1), ("chaotic", 0.0, 1.0, 3), ("clear", 1.0, 1.0, 2)];
const CONFUSION_COLOR: i64 = 4;

/// (name, decision model, kind of practice)
fn meta(name: &str) -> (&'static str, &'static str, &'static str) {
    match name {
        "complex" => ("Complex", "Probe → Sense → Respond", "Emergent Practices"),
        "complicated" => ("Complicated", "Sense → Analyse → Respond", "Good Practices"),
        "clear" => ("Clear", "Sense → Categorise → Respond", "Best Practices"),
        "chaotic" => ("Chaotic", "Act → Sense → Respond", "Novel Practices"),
        _ => ("Confusion", "", "Disorder"),
    }
}

const MAX_CONFUSION_ITEMS: usize = 3;
const ITEM_MAX_WIDTH: f64 = 200.0;
const ITEM_PAD_X: f64 = 10.0;
const ITEM_PAD_Y: f64 = 4.0;
const ITEM_GAP: f64 = 6.0;
const HEAD_GAP: f64 = 10.0;
const MARGIN: f64 = 20.0;
const CLEARANCE: f64 = 10.0;

struct Badge {
    text: String,
    label: Label,
    w: f64,
    h: f64,
    overflow: bool,
}

struct Block {
    name: &'static str,
    title: Label,
    subtitles: Vec<Label>,
    badges: Vec<Badge>,
    w: f64,
    h: f64,
    cx: f64,
    cy: f64,
}

/// `config.cynefin?.[key]` when it is a number within the bounds, else the fallback.
fn setting(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("cynefin", key) {
        Some(ConfigValue::Number(v)) if *v >= min && *v <= max => *v,
        _ => fallback,
    }
}

/// `x ** 2`. fdlibm's pow, which V8 and the `libm` crate share, answers `x * x` for this.
fn square(x: f64) -> f64 {
    math::pow(x, 2.0)
}

pub(crate) fn render(model: &CynefinModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let small = js::round(size * 0.8125);
    let item_size = js::round(size * 0.875);
    let m = ctx.measure();
    let pad = ctx.padding();
    let icons = ctx.icons();
    let show_descriptions = !matches!(config.lookup("cynefin", "showDomainDescriptions"), Some(ConfigValue::Bool(false)));
    let amplitude = setting(config, "boundaryAmplitude", 8.0, 0.0, 50.0);
    let seed = resolve_seed(setting(config, "seed", 0.0, -1e15, 1e15), options.id_prefix.as_deref().unwrap_or("pele"));
    let item_line = js::round(item_size * 1.5);

    let badge = |text: String, overflow: bool| -> Badge {
        let label = layout_label(Some(&text), false, &m, item_size, ITEM_MAX_WIDTH, 0);
        let w = label.width + 2.0 * ITEM_PAD_X;
        let h = js::max(label.height, item_line) + 2.0 * ITEM_PAD_Y;
        Badge { text, label, w, h, overflow }
    };

    let block = |name: &'static str| -> Block {
        let (heading, decision, practice) = meta(name);
        let title = layout_label(Some(heading), false, &m, size, 4000.0, style::BOLD);
        let mut subtitles: Vec<Label> = Vec::new();
        if show_descriptions {
            for text in [decision, practice] {
                if !text.is_empty() {
                    subtitles.push(layout_label(Some(text), false, &m, small, 4000.0, style::ITALIC));
                }
            }
        }
        let items: &[super::CynefinItem] = model.domains.get(name).map_or(&[], |d| &d.items);
        let shown = if name == "confusion" && items.len() > MAX_CONFUSION_ITEMS { MAX_CONFUSION_ITEMS } else { items.len() };
        let mut badges: Vec<Badge> = items[..shown].iter().map(|item| badge(item.label.clone(), false)).collect();
        if shown < items.len() {
            badges.push(badge(format!("+{} more", items.len() - shown), true));
        }
        let mut w = title.width;
        let mut h = title.height;
        for s in &subtitles {
            w = js::max(w, s.width);
            h += s.height;
        }
        if !badges.is_empty() {
            h += HEAD_GAP - ITEM_GAP;
        }
        for b in &badges {
            w = js::max(w, b.w);
            h += ITEM_GAP + b.h;
        }
        Block { name, title, subtitles, badges, w, h, cx: 0.0, cy: 0.0 }
    };

    let mut quadrants: Vec<Block> = QUADRANTS.iter().map(|&(name, ..)| block(name)).collect();
    let mut confusion = block("confusion");
    let rx = js::max(88.0, confusion.w * 0.71 + 8.0);
    let ry = js::max(64.0, confusion.h * 0.71 + 6.0);

    let mut quad_w = setting(config, "width", 640.0, 100.0, 10000.0) / 2.0;
    let mut quad_h = setting(config, "height", 440.0, 100.0, 10000.0) / 2.0;
    for b in &quadrants {
        quad_w = js::max(quad_w, b.w + 2.0 * MARGIN);
        quad_h = js::max(quad_h, b.h + 2.0 * MARGIN);
    }
    // A block sits in the middle of its quadrant, so its inner corner must stay clear of the
    // ellipse.
    for b in &quadrants {
        let dx = (quad_w - b.w) / 2.0;
        let dy = (quad_h - b.h) / 2.0;
        if square(dx / (rx + CLEARANCE)) + square(dy / (ry + CLEARANCE)) >= 1.0 {
            continue;
        }
        let wider = b.w + 2.0 * (rx + CLEARANCE);
        let taller = b.h + 2.0 * (ry + CLEARANCE);
        if wider * quad_h <= quad_w * taller {
            quad_w = js::max(quad_w, wider);
        } else {
            quad_h = js::max(quad_h, taller);
        }
    }

    let title = layout_label(model.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let width = 2.0 * quad_w;
    let height = 2.0 * quad_h;
    let x0 = pad;
    let y0 = pad + title_height;
    let cx = x0 + quad_w;
    let cy = y0 + quad_h;
    let right = x0 + width;
    let bottom = y0 + height;

    let fold = wave(y0, cy, cx, seed, amplitude, WaveOptions { segments: 4, pin_end: true, ..FOLD });
    let west = wave(x0, cx, cy, seed + 100.0, amplitude, WaveOptions { segments: 4, pin_end: true, ..HORIZON });
    let east = wave(cx, right, cy, seed + 200.0, amplitude, WaveOptions { segments: 4, pin_start: true, ..HORIZON });
    let drop = cliff(cx, cy, bottom, width * 0.03);
    let at = |curve: &[f64], end: bool| -> String {
        if end {
            format!("{},{}", num(curve[curve.len() - 2]), num(curve[curve.len() - 1]))
        } else {
            format!("{},{}", num(curve[0]), num(curve[1]))
        }
    };

    let region = |name: &str| -> String {
        match name {
            "complex" => svg_format!("M", num(x0), ",", num(y0), "L", at(&fold, false), forward(&fold), backward(&west), "Z"),
            "complicated" => {
                svg_format!(
                    "M",
                    at(&fold, false),
                    "L",
                    num(right),
                    ",",
                    num(y0),
                    "L",
                    at(&east, true),
                    backward(&east),
                    backward(&fold),
                    "Z"
                )
            }
            "chaotic" => svg_format!("M", at(&west, false), forward(&west), forward(&drop), "L", num(x0), ",", num(bottom), "Z"),
            _ => svg_format!(
                "M",
                at(&east, false),
                forward(&east),
                "L",
                num(right),
                ",",
                num(bottom),
                "L",
                at(&drop, true),
                backward(&drop),
                "Z"
            ),
        }
    };

    let mut fills = String::new();
    for &(name, _, _, color) in &QUADRANTS {
        svg_write!(
            fills,
            "<path class=\"pele-cynefin-region\" data-id=\"",
            name,
            "\" d=\"",
            region(name),
            "\" fill=\"",
            ctx.series(color),
            "\" fill-opacity=\"0.14\"/>"
        )
        .unwrap();
    }
    // The wavy lines between domains divide one shape; the cliff is a drawn line.
    let divider = ctx.paint(Role::InnerStroke, "var(--_b)");
    let boundaries = svg_format!(
        "<g class=\"pele-cynefin-boundaries\" fill=\"none\" stroke=\"",
        divider,
        "\" stroke-dasharray=\"6 3\"><path d=\"",
        curve_path(&fold),
        "\"/><path d=\"",
        curve_path(&west),
        "\"/><path d=\"",
        curve_path(&east),
        "\"/><path class=\"pele-cynefin-cliff\" d=\"",
        curve_path(&drop),
        "\" stroke=\"",
        ctx.paint(Role::Line, "var(--_l)"),
        "\" stroke-width=\"2\" stroke-dasharray=\"none\" stroke-linecap=\"round\"/></g>"
    );
    let oval = ellipse_path(cx, cy, rx, ry);
    let ellipse = svg_format!(
        "<path d=\"",
        oval,
        "\" fill=\"",
        ctx.paint(Role::Background, "var(--_bg)"),
        "\"/><path class=\"pele-cynefin-region\" data-id=\"confusion\" d=\"",
        oval,
        "\" fill=\"",
        ctx.series(CONFUSION_COLOR),
        "\" fill-opacity=\"0.18\" stroke=\"",
        divider,
        "\" stroke-dasharray=\"4 2\"/>"
    );

    let secondary = ctx.paint(Role::SecondaryText, "var(--_m)");
    let item_fill = ctx.paint(Role::NodeFill, "var(--_bg)");
    let item_stroke = ctx.paint(Role::NodeStroke, "var(--_b)");
    let item_size_s = js::to_string(item_size);
    let small_s = js::to_string(small);
    let draw_block = |b: &Block| -> String {
        let mut y = b.cy - b.h / 2.0;
        let mut out =
            label_svg(&b.title, b.cx, y + b.title.height / 2.0, " class=\"pele-label\" font-weight=\"var(--_hw)\"", None).to_string();
        y += b.title.height;
        for s in &b.subtitles {
            svg_write!(
                out,
                label_svg(
                    s,
                    b.cx,
                    y + s.height / 2.0,
                    &svg_format!(
                        " class=\"pele-cynefin-subtitle\" font-size=\"",
                        small_s,
                        "\" font-style=\"italic\" fill=\"",
                        secondary,
                        "\""
                    ),
                    None,
                )
            )
            .unwrap();
            y += s.height;
        }
        y += HEAD_GAP - ITEM_GAP;
        for item in &b.badges {
            y += ITEM_GAP;
            svg_write!(
                out,
                "<g class=\"pele-node",
                if item.overflow { " pele-cynefin-overflow" } else { "" },
                "\"",
                if item.overflow { String::new() } else { svg_format!(" data-id=\"", esc_text(&item.text), "\"") },
                ">"
            )
            .unwrap();
            svg_write!(
                out,
                "<rect x=\"",
                num(b.cx - item.w / 2.0),
                "\" y=\"",
                num(y),
                "\" width=\"",
                num(item.w),
                "\" height=\"",
                num(item.h),
                "\" rx=\"",
                RADIUS,
                "\" fill=\"",
                item_fill,
                "\" stroke=\"",
                item_stroke,
                "\"",
                if item.overflow { " stroke-dasharray=\"3 2\"" } else { "" },
                "/>"
            )
            .unwrap();
            let attrs = svg_format!(
                " class=\"pele-label\" font-size=\"",
                item_size_s,
                "\"",
                if item.overflow { svg_format!(" fill=\"", secondary, "\"") } else { String::new() }
            );
            svg_write!(out, label_svg(&item.label, b.cx, y + item.h / 2.0, &attrs, icons)).unwrap();
            out.push_str("</g>");
            y += item.h;
        }
        svg_format!("<g class=\"pele-cynefin-domain\" data-id=\"", b.name, "\">", out, "</g>")
    };

    let mut domains = String::new();
    for (index, &(_, col, row, _)) in QUADRANTS.iter().enumerate() {
        let b = &mut quadrants[index];
        b.cx = x0 + quad_w * (col + 0.5);
        b.cy = y0 + quad_h * (row + 0.5);
        domains.push_str(&draw_block(b));
    }
    confusion.cx = cx;
    confusion.cy = cy;
    domains.push_str(&draw_block(&confusion));

    let find = |name: &str| -> &Block {
        match QUADRANTS.iter().position(|&(n, ..)| n == name) {
            Some(k) => &quadrants[k],
            None => &confusion,
        }
    };
    let inside = |b: &Block, (x, y): (f64, f64)| -> bool {
        if b.name == "confusion" {
            square((x - cx) / (rx + 4.0)) + square((y - cy) / (ry + 4.0)) <= 1.0
        } else {
            (x - b.cx).abs() <= b.w / 2.0 + 8.0 && (y - b.cy).abs() <= b.h / 2.0 + 8.0
        }
    };

    let line = ctx.paint(Role::Line, "var(--_l)");
    let arrow = ctx.paint(Role::Arrow, "var(--_l)");
    let mut edges = String::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for transition in &model.transitions {
        let from = find(&transition.from);
        let to = find(&transition.to);
        if from.name == to.name {
            continue;
        }
        let key = if transition.from < transition.to {
            format!("{} {}", transition.from, transition.to)
        } else {
            format!("{} {}", transition.to, transition.from)
        };
        let repeat = seen.get(&key).copied().unwrap_or(0);
        seen.insert(key, repeat + 1);

        let dx = to.cx - from.cx;
        let dy = to.cy - from.cy;
        let len = or_one(math::hypot(dx, dy));
        let mx = (from.cx + to.cx) / 2.0;
        let my = (from.cy + to.cy) / 2.0;
        // Curves bow away from the middle of the diagram, which keeps them off the ellipse.
        let outward = if -dy * (mx - cx) + dx * (my - cy) >= 0.0 { 1.0 } else { -1.0 };
        let nx = (-dy / len) * outward;
        let ny = (dx / len) * outward;
        // A diagonal runs through the middle, so it bows far enough to pass the ellipse.
        let diagonal = from.name != "confusion" && to.name != "confusion" && dx.abs() > 1.0 && dy.abs() > 1.0;
        let bow = (if diagonal { 2.0 * (math::hypot(rx * nx, ry * ny) + 16.0) } else { len * 0.15 }) + repeat as f64 * 56.0;
        let px = mx + nx * bow;
        let py = my + ny * bow;
        let point = |t: f64| -> (f64, f64) {
            let u = 1.0 - t;
            (u * u * from.cx + 2.0 * u * t * px + t * t * to.cx, u * u * from.cy + 2.0 * u * t * py + t * t * to.cy)
        };
        let crossing = |b: &Block, mut lo: f64, mut hi: f64, leaving: bool| -> f64 {
            for _ in 0..16 {
                let mid = (lo + hi) / 2.0;
                if inside(b, point(mid)) == leaving {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            (lo + hi) / 2.0
        };
        const STEPS: usize = 48;
        let steps = STEPS as f64;
        let mut i = 1;
        while i <= STEPS && inside(from, point(i as f64 / steps)) {
            i += 1;
        }
        if i > STEPS {
            continue;
        }
        let t0 = crossing(from, (i - 1) as f64 / steps, i as f64 / steps, true);
        while i <= STEPS && !inside(to, point(i as f64 / steps)) {
            i += 1;
        }
        let t1 = if i > STEPS { 1.0 } else { crossing(to, (i - 1) as f64 / steps, i as f64 / steps, false) };

        let (sx, sy) = point(t0);
        let (mut ex, mut ey) = point(t1);
        let blossom = |a: f64, b: f64, c: f64| (1.0 - t0) * (1.0 - t1) * a + ((1.0 - t0) * t1 + t0 * (1.0 - t1)) * b + t0 * t1 * c;
        let qx = blossom(from.cx, px, to.cx);
        let qy = blossom(from.cy, py, to.cy);
        let el = or_one(math::hypot(ex - qx, ey - qy));
        let ux = (ex - qx) / el;
        let uy = (ey - qy) / el;
        let head = marker("arrow_point", ex, ey, ux, uy, arrow).to_string();
        ex -= ux * marker_trim("arrow_point");
        ey -= uy * marker_trim("arrow_point");

        let mut text = String::new();
        if let Some(label) = transition.label.as_deref().filter(|l| !l.is_empty()) {
            let label = layout_label(Some(label), false, &m, small, ITEM_MAX_WIDTH, 0);
            let (lx, ly) = point((t0 + t1) / 2.0);
            let away = (nx.abs() * label.width) / 2.0 + (ny.abs() * label.height) / 2.0 + 6.0;
            let half = label.width / 2.0 + 4.0;
            let tx = js::min(js::max(lx + nx * away, x0 + half), js::max(right - half, x0 + half));
            text = label_svg(&label, tx, ly + ny * away, &svg_format!(" class=\"pele-edge-label\" font-size=\"", small_s, "\""), icons)
                .to_string();
        }
        svg_write!(
            edges,
            "<g class=\"pele-edge\" data-id=\"",
            transition.from,
            "-",
            transition.to,
            "\"><path d=\"M",
            num(sx),
            ",",
            num(sy),
            "Q",
            num(qx),
            ",",
            num(qy),
            " ",
            num(ex),
            ",",
            num(ey),
            "\" fill=\"none\" stroke=\"",
            line,
            "\"/>",
            head,
            text,
            "</g>"
        )
        .unwrap();
    }

    let total_width = (js::max(width, title.width) + 2.0 * pad).ceil();
    let total_height = (height + title_height + 2.0 * pad).ceil();
    let mut content =
        label_svg(&title, x0 + width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None).to_string();
    svg_write!(content, "<g class=\"pele-cynefin-regions\">", fills, "</g>").unwrap();
    content.push_str(&boundaries);
    content.push_str(&ellipse);
    content.push_str(&domains);
    if !edges.is_empty() {
        svg_write!(content, "<g class=\"pele-edges\">", edges, "</g>").unwrap();
    }

    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered {
        svg: svg_document("cynefin", total_width, total_height, size, ctx, acc, &content),
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}

/// `x || 1`: zero and NaN become one.
fn or_one(x: f64) -> f64 {
    if x == 0.0 || x.is_nan() {
        1.0
    } else {
        x
    }
}
