//! Event modeling lanes and narrow-screen cards, ported from Pele render.ts.
use super::{EntityKind, EventModelingModel};
use crate::diagrams::common::{
    fit_width::turn_to_fit,
    literal_label::{break_render_label as break_label, left_label_markup as left_label_svg, literal_render_label as literal_label},
};
use crate::diagrams::{RenderCtx, Rendered};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg},
    edges::{marker, marker_trim},
    root::{svg_document, Accessible},
    theme::{FONT_MONO, RADIUS},
};
use crate::text::{entities::decode_entities, layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::js;
use crate::{Config, Error};
use indexmap::{IndexMap, IndexSet};
use std::fmt::Write as _;

fn series(kind: EntityKind) -> Option<i64> {
    match kind {
        EntityKind::Command => Some(0),
        EntityKind::Event => Some(1),
        EntityKind::Readmodel => Some(2),
        EntityKind::Processor => Some(6),
        EntityKind::Ui => None,
    }
}
fn hard_wrap(text: &str, columns: usize) -> Vec<String> {
    let text = decode_entities(text);
    let mut out = vec![];
    for line in text.split('\n') {
        let len = js::len16(line);
        if len <= columns {
            out.push(line.to_owned());
        } else {
            for i in (0..len).step_by(columns) {
                out.push(js::slice16(line, i, (i + columns).min(len)).to_owned());
            }
        }
    }
    out
}
fn lines(rows: &[String], x: f64, y: f64, step: f64, attrs: &str) -> String {
    let mut body = String::new();
    for (i, row) in rows.iter().enumerate() {
        if !row.is_empty() {
            svg_write!(body, "<tspan x=\"", num(x), "\" y=\"", num(y + i as f64 * step), "\">", esc(row), "</tspan>").unwrap();
        }
    }
    if body.is_empty() {
        body
    } else {
        svg_format!("<text", attrs, " xml:space=\"preserve\">", body, "</text>")
    }
}
fn widest(rows: &[String], m: &Measure, size: f64, st: u8) -> f64 {
    rows.iter().fold(0.0, |w, r| js::max(w, m.width(r, size, st)))
}
fn title_for(heading: &str, frame: &str, names: &IndexMap<&str, &str>) -> String {
    format!("{heading} · {frame}{}", names.get(frame).map_or_else(String::new, |s| format!(" {s}")))
}
fn specification_rows(given: &[String], when: &[String], then: &[String]) -> Vec<String> {
    let mut rows = vec![format!("Given  {}", given.join(", "))];
    if !when.is_empty() {
        rows.push(format!("When  {}", when.join(", ")));
    }
    rows.push(format!("Then  {}", then.join(", ")));
    rows
}
fn output(model: &EventModelingModel, ctx: &RenderCtx, width: f64, height: f64, content: &str) -> Rendered {
    Rendered {
        svg: svg_document(
            "eventmodeling",
            width,
            height,
            ctx.font_size(),
            ctx,
            Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() },
            content,
        ),
        width,
        height,
        links: vec![],
    }
}
pub(crate) fn render(model: &EventModelingModel, _config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    turn_to_fit(ctx.options, |down| if down { vertical(model, ctx) } else { horizontal(model, ctx) })
}
struct Band {
    label: Label,
    right: f64,
    y: f64,
    h: f64,
}
struct Placed {
    name: Label,
    data: Vec<String>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
fn horizontal(model: &EventModelingModel, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let size = ctx.font_size();
    let name_size = js::round(size * 0.875);
    let small = js::round(size * 0.8125);
    let mono = js::round(size * 0.75);
    let mono_step = js::round(mono * 1.4);
    let small_step = js::round(small * 1.5);
    let m = ctx.measure();
    let pad = ctx.padding();
    let mut label_column = 0.0;
    let mut bands: Vec<Band> = model
        .lanes
        .iter()
        .map(|lane| {
            let label = layout_label(Some(&lane.label), false, &m, small, 4000.0, style::BOLD);
            label_column = js::max(label_column, label.width);
            Band { label, right: f64::NEG_INFINITY, y: 0.0, h: 0.0 }
        })
        .collect();
    label_column += 24.0;
    let mut placed: Vec<Placed> = vec![];
    for (i, b) in model.boxes.iter().enumerate() {
        let band = &mut bands[b.lane];
        let name = layout_label(Some(&b.name), false, &m, name_size, 4000.0, style::BOLD);
        let data = b.data.as_ref().filter(|s| !s.is_empty()).map_or_else(Vec::new, |s| hard_wrap(s, 48));
        let w = js::max(96.0, js::max(name.width, widest(&data, &m, mono, style::MONO)) + 24.0);
        let h = 20.0 + name.height + if data.is_empty() { 0.0 } else { 6.0 + data.len() as f64 * mono_step };
        let x = if let Some(prev) = placed.last() {
            let stagger = prev.x + js::max(prev.w / 2.0, prev.w - 80.0);
            if model.boxes[i - 1].lane == b.lane {
                band.right + 16.0
            } else {
                js::max(stagger, band.right + 16.0)
            }
        } else {
            0.0
        };
        band.right = x + w;
        band.h = js::max(band.h, h);
        placed.push(Placed { name, data, x, y: 0.0, w, h });
    }
    let content_right = placed.iter().fold(0.0, |w, p| js::max(w, p.x + p.w));
    let lane_width = label_column + content_right + 12.0;
    let title = layout_label(model.title.as_deref(), false, &m, size, 4000.0, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let left = pad;
    let top = pad + title_h;
    let mut y = top;
    let mut lanes = String::new();
    for (i, band) in bands.iter_mut().enumerate() {
        band.y = y;
        band.h += 24.0;
        write!(lanes, "<g class=\"pele-cluster pele-em-lane\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"{}\"/>{}</g>",esc_text(&model.lanes[i].label),num(left),num(y),num(lane_width),num(band.h),ctx.paint(Role::GroupFill,"var(--_s)"),label_svg(&band.label,left+12.0+band.label.width/2.0,y+band.h/2.0,&svg_format!(" class=\"pele-cluster-label\" font-size=\"", js::to_string(small), "\" font-weight=\"var(--_hw)\" fill=\"", ctx.paint(Role::SecondaryText,"var(--_m)"), "\""),None)).unwrap();
        y += band.h + 8.0;
    }
    let mut bottom = if bands.is_empty() { top } else { y - 8.0 };
    let mut nodes = String::new();
    for (i, item) in placed.iter_mut().enumerate() {
        let b = &model.boxes[i];
        let band = &bands[b.lane];
        item.x += left + label_column;
        item.y = band.y + (band.h - item.h) / 2.0;
        let frame = svg_format!(
            "x=\"",
            num(item.x),
            "\" y=\"",
            num(item.y),
            "\" width=\"",
            num(item.w),
            "\" height=\"",
            num(item.h),
            "\" rx=\"",
            RADIUS,
            "\""
        );
        let shape = match series(b.kind) {
            None => svg_format!(
                "<rect ",
                frame,
                " fill=\"",
                ctx.paint(Role::Background, "var(--_bg)"),
                "\" stroke=\"",
                ctx.paint(Role::NodeStroke, "var(--_b)"),
                "\"/>"
            ),
            Some(i) => {
                let color = ctx.series(i);
                svg_format!(
                    "<rect ",
                    frame,
                    " fill=\"",
                    ctx.paint(Role::Background, "var(--_bg)"),
                    "\"/><rect ",
                    frame,
                    " fill=\"",
                    color,
                    "\" fill-opacity=\"0.22\" stroke=\"",
                    color,
                    "\"/>"
                )
            }
        };
        let text = svg_format!(label_svg(
            &item.name,
            item.x + item.w / 2.0,
            item.y + 10.0 + item.name.height / 2.0,
            &svg_format!(" class=\"pele-label\" font-size=\"", js::to_string(name_size), "\" font-weight=\"var(--_hw)\""),
            None,
        )) + &lines(
            &item.data,
            item.x + 12.0,
            item.y + 10.0 + item.name.height + 6.0 + mono_step / 2.0 + mono * 0.35,
            mono_step,
            &svg_format!(" class=\"pele-em-data\" font-family=\"", FONT_MONO, "\" font-size=\"", js::to_string(mono), "\""),
        );
        svg_write!(
            nodes,
            "<g class=\"pele-node pele-em-",
            b.kind.as_str(),
            if b.reset { " pele-em-reset" } else { "" },
            "\" data-id=\"",
            esc_text(&b.id),
            "\">",
            shape,
            text,
            "</g>"
        )
        .unwrap();
    }
    let mut edges = String::new();
    let trim = marker_trim("arrow_point");
    for r in &model.relations {
        let (Some(s), Some(t)) = (placed.get(r.source), placed.get(r.target)) else {
            continue;
        };
        if r.source == r.target {
            continue;
        }
        let (d, head) = if model.boxes[r.source].lane == model.boxes[r.target].lane {
            let forward = t.x >= s.x;
            let sx = if forward { s.x + s.w } else { s.x };
            let tx = if forward { t.x } else { t.x + t.w };
            let sy = s.y + s.h / 2.0;
            let ty = t.y + t.h / 2.0;
            let dir = if tx >= sx { 1.0 } else { -1.0 };
            (
                svg_format!("M", num(sx), ",", num(sy), "L", num(tx - dir * trim), ",", num(ty)),
                marker("arrow_point", tx, ty, dir, 0.0, ctx.paint(Role::Arrow, "var(--_l)")),
            )
        } else {
            let down = t.y > s.y;
            let from = js::max(s.x, t.x);
            let to = js::min(s.x + s.w, t.x + t.w);
            let sx = if to - from >= 16.0 {
                (from + to) / 2.0
            } else if t.x > s.x {
                s.x + s.w - js::min(24.0, s.w / 4.0)
            } else {
                s.x + js::min(24.0, s.w / 4.0)
            };
            let tx = if to - from >= 16.0 {
                sx
            } else if t.x > s.x {
                t.x + js::min(24.0, t.w / 4.0)
            } else {
                t.x + t.w - js::min(24.0, t.w / 4.0)
            };
            let sy = if down { s.y + s.h } else { s.y };
            let ty = if down { t.y } else { t.y + t.h };
            let dir = if down { 1.0 } else { -1.0 };
            let end = ty - dir * trim;
            let mid = (sy + end) / 2.0;
            let d = if sx == tx {
                svg_format!("M", num(sx), ",", num(sy), "V", num(end))
            } else {
                svg_format!(
                    "M",
                    num(sx),
                    ",",
                    num(sy),
                    "C",
                    num(sx),
                    ",",
                    num(mid),
                    " ",
                    num(tx),
                    ",",
                    num(mid),
                    " ",
                    num(tx),
                    ",",
                    num(end)
                )
            };
            (d, marker("arrow_point", tx, ty, 0.0, dir, ctx.paint(Role::Arrow, "var(--_l)")))
        };
        svg_write!(
            edges,
            "<g class=\"pele-edge\" data-id=\"",
            esc_text(&model.boxes[r.source].id),
            "-",
            esc_text(&model.boxes[r.target].id),
            "\"><path d=\"",
            d,
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\"/>",
            head,
            "</g>"
        )
        .unwrap();
    }
    let names: IndexMap<&str, &str> = model.boxes.iter().map(|b| (b.id.as_str(), b.name.as_str())).collect();
    let mut cards = String::new();
    let mut cards_right = 0.0;
    let mut card = |kind: &str, heading: &str, frame: &str, rows: Vec<String>| {
        let head = layout_label(Some(&title_for(heading, frame, &names)), false, &m, small, 4000.0, style::BOLD);
        let w = js::max(head.width, widest(&rows, &m, small, 0)) + 24.0;
        let h = 20.0 + head.height + rows.len() as f64 * small_step;
        let cy = bottom + if cards.is_empty() && bands.is_empty() { 0.0 } else { 8.0 };
        svg_write!(
            cards,
            "<g class=\"pele-em-",
            kind,
            "\" data-id=\"",
            esc_text(frame),
            "\"><rect x=\"",
            num(left),
            "\" y=\"",
            num(cy),
            "\" width=\"",
            num(w),
            "\" height=\"",
            num(h),
            "\" rx=\"",
            RADIUS,
            "\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\"/>",
            label_svg(
                &head,
                left + 12.0 + head.width / 2.0,
                cy + 10.0 + head.height / 2.0,
                &svg_format!(" font-size=\"", js::to_string(small), "\" font-weight=\"var(--_hw)\""),
                None
            ),
            lines(
                &rows,
                left + 12.0,
                cy + 10.0 + head.height + small_step / 2.0 + small * 0.35,
                small_step,
                &svg_format!(" font-size=\"", js::to_string(small), "\"")
            ),
            "</g>"
        )
        .unwrap();
        bottom = cy + h;
        cards_right = js::max(cards_right, w);
    };
    for note in &model.notes {
        card("note", "Note", &note.frame, hard_wrap(&note.text, 80));
    }
    for spec in &model.specifications {
        card(
            "specification",
            "Given / When / Then",
            &spec.frame,
            specification_rows(&spec.given, &spec.when, &spec.then).iter().flat_map(|s| hard_wrap(s, 80)).collect(),
        );
    }
    let width = (js::max(js::max(if bands.is_empty() { 0.0 } else { lane_width }, cards_right), title.width) + 2.0 * pad).ceil();
    let height = (bottom + pad).ceil();
    let content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None))
            + &lanes
            + &edges
            + &nodes
            + &cards;
    Ok(output(model, ctx, width, height, &content))
}
struct VerticalPlaced {
    y: f64,
    h: f64,
    w: f64,
    lane: f64,
}
fn label_rows(label: &Label) -> Vec<String> {
    label.lines().iter().map(|line| line.iter().map(|span| span.text.as_str()).collect::<String>()).collect()
}
fn vertical(model: &EventModelingModel, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let size = ctx.font_size();
    let small = js::round(size * 0.875);
    let mono = js::round(size * 0.75);
    let pad = ctx.padding();
    let m = ctx.measure();
    let room = js::max(160.0, ctx.options.max_width.unwrap_or(320.0) - 2.0 * pad);
    let mut runs = IndexMap::<usize, (usize, usize)>::new();
    let mut passed = IndexSet::new();
    for r in &model.relations {
        let (s, t) = (r.source, r.target);
        if s == t || s >= model.boxes.len() || t >= model.boxes.len() {
            continue;
        }
        if s.abs_diff(t) < 2 {
            passed.insert(s.max(t));
            continue;
        }
        if let Some(run) = runs.get_mut(&t) {
            run.0 = run.0.min(s);
            run.1 = run.1.max(s);
        } else {
            runs.insert(t, (s.min(t), s.max(t)));
        }
    }
    let mut ordered: Vec<_> = runs.into_iter().collect();
    ordered.sort_by_key(|(_, r)| r.0);
    let mut rail_of = IndexMap::<usize, usize>::new();
    let mut ends = vec![];
    for (target, (from, to)) in ordered {
        let rail = ends.iter().position(|&end| end < from).unwrap_or(ends.len());
        if rail == ends.len() {
            ends.push(to);
        } else {
            ends[rail] = to;
        }
        rail_of.insert(target, rail);
    }
    let gutter = 24.0 + 8.0 * ends.len().saturating_sub(1) as f64;
    let card_width = js::max(120.0, room - gutter);
    let text_room = card_width - 24.0;
    let title = layout_label(model.title.as_deref(), false, &m, size, room, style::BOLD);
    let mut y = pad + if title.height > 0.0 { title.height + 12.0 } else { 0.0 };
    let mut bottom = pad + title.height;
    let mut width = js::max(title.width, card_width + gutter);
    let (mut nodes, mut edges, mut cards) = (String::new(), String::new(), String::new());
    let mut placed = vec![];
    let literal = |s: &str, size, style| literal_label(&decode_entities(s), &m, size, text_room, style);
    for (i, b) in model.boxes.iter().enumerate() {
        let lane_room = if passed.contains(&i) { text_room - 20.0 } else { text_room };
        let lane = break_label(&layout_label(Some(&model.lanes[b.lane].label), false, &m, small, lane_room, style::BOLD), &m, lane_room);
        let name = layout_label(Some(&b.name), false, &m, small, text_room - 36.0, style::BOLD);
        let data = b.data.as_ref().filter(|s| !s.is_empty()).map(|s| literal(s, mono, style::MONO));
        let w = js::max(
            js::max(card_width, lane.width + 24.0),
            js::max(name.width + 24.0 + 36.0, data.as_ref().map_or(0.0, |d| d.width) + 24.0),
        );
        let h = 20.0 + name.height + data.as_ref().map_or(0.0, |d| d.height + 6.0);
        let cy = y + lane.height + 12.0;
        let frame =
            svg_format!("x=\"", num(pad), "\" y=\"", num(cy), "\" width=\"", num(w), "\" height=\"", num(h), "\" rx=\"", RADIUS, "\"");
        svg_write!(
            nodes,
            "<g class=\"pele-cluster pele-em-lane\" data-id=\"",
            esc_text(&model.lanes[b.lane].label),
            "\">",
            left_label_svg(
                &lane,
                pad + 12.0,
                y,
                &svg_format!(
                    " class=\"pele-cluster-label\" font-size=\"",
                    js::to_string(small),
                    "\" font-weight=\"var(--_hw)\" fill=\"",
                    ctx.paint(Role::SecondaryText, "var(--_m)"),
                    "\""
                )
            ),
            "<g class=\"pele-node pele-em-",
            b.kind.as_str(),
            if b.reset { " pele-em-reset" } else { "" },
            "\" data-id=\"",
            esc_text(&b.id),
            "\"><rect ",
            frame,
            " fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\"/>",
            series(b.kind).map_or_else(String::new, |i| {
                let c = ctx.series(i);
                svg_format!("<rect ", frame, " fill=\"", c, "\" fill-opacity=\"0.22\" stroke=\"", c, "\"/>")
            }),
            label_svg(
                &name,
                pad + 12.0 + name.width / 2.0,
                cy + 10.0 + name.height / 2.0,
                &svg_format!(" class=\"pele-label\" font-size=\"", js::to_string(small), "\" font-weight=\"var(--_hw)\""),
                None
            ),
            "<text class=\"pele-em-frame\" x=\"",
            num(pad + w - 12.0),
            "\" y=\"",
            num(cy + 10.0 + small),
            "\" text-anchor=\"end\" font-size=\"",
            js::to_string(mono),
            "\" fill=\"",
            ctx.paint(Role::SecondaryText, "var(--_m)"),
            "\">",
            esc(&b.id),
            "</text>",
            data.as_ref().map_or_else(String::new, |d| lines(
                &label_rows(d),
                pad + 12.0,
                cy + 10.0 + name.height + 6.0 + d.line_height / 2.0 + mono * 0.35,
                d.line_height,
                &svg_format!(" class=\"pele-em-data\" font-family=\"", FONT_MONO, "\" font-size=\"", js::to_string(mono), "\"")
            )),
            "</g></g>"
        )
        .unwrap();
        placed.push(VerticalPlaced { y: cy, h, w, lane: lane.width });
        width = js::max(width, w + gutter);
        bottom = cy + h;
        y = bottom + 28.0;
    }
    for r in &model.relations {
        let (Some(s), Some(t)) = (placed.get(r.source), placed.get(r.target)) else {
            continue;
        };
        if r.source == r.target {
            continue;
        }
        let down = t.y > s.y;
        let dir = if down { 1.0 } else { -1.0 };
        let sy = if down { s.y + s.h } else { s.y };
        let ty = if down { t.y } else { t.y + t.h };
        let (d, head) = if r.source.abs_diff(r.target) == 1 {
            let cx = pad + js::max(js::min(s.w, t.w) / 2.0, 12.0 + if down { t.lane } else { s.lane } + 16.0);
            (
                svg_format!("M", num(cx), ",", num(sy), "V", num(ty - dir * marker_trim("arrow_point"))),
                marker("arrow_point", cx, ty, 0.0, dir, ctx.paint(Role::Arrow, "var(--_l)")),
            )
        } else {
            let rail = pad + width - gutter + 12.0 + 8.0 * rail_of.get(&r.target).copied().unwrap_or(0) as f64;
            let from = s.y + s.h / 2.0 - 5.0;
            let to = t.y + t.h / 2.0 + 5.0;
            (
                svg_format!(
                    "M",
                    num(pad + s.w),
                    ",",
                    num(from),
                    "H",
                    num(rail),
                    "V",
                    num(to),
                    "H",
                    num(pad + t.w + marker_trim("arrow_point"))
                ),
                marker("arrow_point", pad + t.w, to, -1.0, 0.0, ctx.paint(Role::Arrow, "var(--_l)")),
            )
        };
        svg_write!(
            edges,
            "<g class=\"pele-edge\" data-id=\"",
            esc_text(&model.boxes[r.source].id),
            "-",
            esc_text(&model.boxes[r.target].id),
            "\"><path d=\"",
            d,
            "\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\"/>",
            head,
            "</g>"
        )
        .unwrap();
    }
    let names: IndexMap<&str, &str> = model.boxes.iter().map(|b| (b.id.as_str(), b.name.as_str())).collect();
    let mut card = |kind: &str, heading: &str, frame: &str, raw: &str| {
        let head = layout_label(Some(&title_for(heading, frame, &names)), false, &m, small, text_room, style::BOLD);
        let body = literal(raw, small, 0);
        let w = js::max(card_width, js::max(head.width + 24.0, body.width + 24.0));
        let h = 20.0 + head.height + body.height;
        svg_write!(
            cards,
            "<g class=\"pele-em-",
            kind,
            "\" data-id=\"",
            esc_text(frame),
            "\"><rect x=\"",
            num(pad),
            "\" y=\"",
            num(y),
            "\" width=\"",
            num(w),
            "\" height=\"",
            num(h),
            "\" rx=\"",
            RADIUS,
            "\" fill=\"",
            ctx.paint(Role::Background, "var(--_bg)"),
            "\" stroke=\"",
            ctx.paint(Role::NodeStroke, "var(--_b)"),
            "\"/>",
            label_svg(
                &head,
                pad + 12.0 + head.width / 2.0,
                y + 10.0 + head.height / 2.0,
                &svg_format!(" font-size=\"", js::to_string(small), "\" font-weight=\"var(--_hw)\""),
                None
            ),
            lines(
                &label_rows(&body),
                pad + 12.0,
                y + 10.0 + head.height + body.line_height / 2.0 + small * 0.35,
                body.line_height,
                &svg_format!(" font-size=\"", js::to_string(small), "\"")
            ),
            "</g>"
        )
        .unwrap();
        bottom = y + h;
        y = bottom + 8.0;
        width = js::max(width, w);
    };
    for note in &model.notes {
        card("note", "Note", &note.frame, &note.text);
    }
    for spec in &model.specifications {
        card("specification", "Given / When / Then", &spec.frame, &specification_rows(&spec.given, &spec.when, &spec.then).join("\n"));
    }
    let total_width = (width + 2.0 * pad).ceil();
    let height = (bottom + pad).ceil();
    let content = svg_format!(label_svg(
        &title,
        total_width / 2.0,
        pad + title.height / 2.0,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        None
    )) + &edges
        + &nodes
        + &cards;
    Ok(output(model, ctx, total_width, height, &content))
}
