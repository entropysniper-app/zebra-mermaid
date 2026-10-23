//! Time-axis and task rendering, ported from Pele gantt/render.ts.
use super::{
    axis::{auto_ticks, interval_ticks, time_format},
    dates::{add, add_days, Unit},
    db::{GanttModel, Schedule, Times},
    js_date::{self, JsDate, Zone},
};
use crate::diagrams::common::fit_width::cap_width;
use crate::diagrams::common::str_entry::or_insert_with as str_entry;
use crate::diagrams::{RenderCtx, Rendered};
use crate::svg::concat::{svg_format, svg_write};
use crate::svg::{
    builder::{esc, esc_text, num, render_label_svg as label_svg},
    root::{svg_document, Accessible},
    theme::{resolve_style, CRITICAL},
};
use crate::text::{layout_render_label as layout_label, style, Measure, RenderLabel as Label};
use crate::theme::Role;
use crate::util::{js, url::linked};
use crate::{Config, ConfigValue, Error};
use indexmap::IndexMap;
use std::fmt::Write as _;

fn setting(c: &Config, key: &str, min: f64, max: f64) -> Option<f64> {
    c.lookup("gantt", key).and_then(ConfigValue::as_f64).filter(|n| n.is_finite()).map(|n| n.clamp(min, max))
}
fn text<'a>(c: &'a Config, key: &str) -> Option<&'a str> {
    c.lookup("gantt", key).and_then(ConfigValue::as_str)
}
fn scheduled(t: &Times) -> bool {
    t.start.is_some_and(|d| d.time().is_finite()) && t.end.is_some_and(|d| d.time().is_finite())
}
fn one_line(raw: &str, m: &Measure, size: f64) -> Label {
    let lower = raw.to_ascii_lowercase();
    let mut cooked = String::new();
    let mut from = 0;
    let mut search = 0;
    while let Some(off) = lower[search..].find("<br") {
        let at = search + off;
        let mut end = at + 3;
        while end < raw.len() && crate::util::chars::char_at(raw, end).is_some_and(crate::util::chars::is_space) {
            end += raw[end..].chars().next().unwrap().len_utf8();
        }
        if raw.as_bytes().get(end) == Some(&b'/') {
            end += 1;
        }
        if raw.as_bytes().get(end) == Some(&b'>') {
            cooked.push_str(&raw[from..at]);
            cooked.push(' ');
            from = end + 1;
        }
        search = end.max(at + 3);
    }
    cooked.push_str(&raw[from..]);
    let mut label = layout_label(Some(&cooked), false, m, size, 4000.0, 0);
    label.keep_first_line();
    label
}
struct Bar {
    task: usize,
    label: Label,
    row: usize,
    x0: f64,
    x1: f64,
    label_x: f64,
    inside: bool,
}
pub(crate) fn render(model: &GanttModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = ctx.font_size();
    let m = ctx.measure();
    let pad = ctx.padding();
    let now = options.now.filter(|n| n.is_finite()).unwrap_or_else(js_date::now);
    let zone = options.timezone_offset_minutes.map_or(Zone::UTC, Zone::from_minutes);
    let label_size = setting(config, "fontSize", 4.0, 72.0).unwrap_or(js::round(size * 0.8125));
    let section_size = setting(config, "sectionFontSize", 4.0, 72.0).unwrap_or(label_size);
    let tick_size = js::round(size * 0.75);
    let bar_h = setting(config, "barHeight", 4.0, 400.0).unwrap_or(js::round(label_size * 1.85));
    let bar_gap = setting(config, "barGap", 0.0, 400.0).unwrap_or(js::round(bar_h / 4.0));
    let row_h = bar_h + bar_gap;
    let section_styles = js::round(setting(config, "numberSectionStyles", 1.0, 1000.0).unwrap_or(4.0)) as usize;
    let top_axis = model.top_axis || config.lookup("gantt", "topAxis").and_then(ConfigValue::as_bool) == Some(true);
    let mut schedule = Schedule::new(model, now, zone)?;
    let times = schedule.times.clone();
    let row_tasks: Vec<usize> = model.tasks.iter().enumerate().filter_map(|(i, t)| (!t.vert).then_some(i)).collect();
    let mut verts: Vec<usize> = model.tasks.iter().enumerate().filter_map(|(i, t)| (t.vert && scheduled(&times[i])).then_some(i)).collect();
    let mut categories = IndexMap::<String, usize>::new();
    for &i in &row_tasks {
        let len = categories.len();
        str_entry(&mut categories, &model.tasks[i].section, || len);
    }
    let mut row_of = vec![0; model.tasks.len()];
    let mut row_section: Vec<String> = vec![];
    if model.display_mode == "compact" {
        let mut groups = IndexMap::<String, Vec<usize>>::new();
        for &i in &row_tasks {
            str_entry(&mut groups, &model.tasks[i].section, Default::default).push(i);
        }
        for (section, group) in groups {
            let offset = row_section.len();
            let mut sorted: Vec<_> = group.iter().copied().filter(|&i| scheduled(&times[i])).collect();
            sorted.sort_by(|&a, &b| {
                times[a]
                    .start
                    .unwrap()
                    .time()
                    .total_cmp(&times[b].start.unwrap().time())
                    .then(model.tasks[a].order.cmp(&model.tasks[b].order))
            });
            let mut free: Vec<f64> = vec![];
            for i in sorted {
                let start = times[i].start.unwrap().time();
                let row = free.iter().position(|&t| start >= t).unwrap_or(free.len());
                if row == free.len() {
                    free.push(times[i].end.unwrap().time());
                } else {
                    free[row] = times[i].end.unwrap().time();
                }
                row_of[i] = offset + row;
            }
            let mut rows = free.len();
            for i in group {
                if !scheduled(&times[i]) {
                    row_of[i] = offset + rows;
                    rows += 1;
                }
            }
            row_section.extend(std::iter::repeat_n(section, rows.max(1)));
        }
    } else {
        for &i in &row_tasks {
            row_of[i] = row_section.len();
            row_section.push(model.tasks[i].section.clone());
        }
    }
    let rows = row_section.len();
    let mut start = f64::INFINITY;
    let mut stop = f64::NEG_INFINITY;
    for t in &times {
        if scheduled(t) {
            start = js::min(start, t.start.unwrap().time());
            stop = js::max(stop, t.end.unwrap().time());
        }
    }
    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, 4000.0, style::BOLD);
    let title_h = if title.height > 0.0 { title.height + 16.0 } else { 0.0 };
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    if start > stop {
        let width = (js::max(title.width, 1.0) + 2.0 * pad).ceil();
        let height = (js::max(title.height, 1.0) + 2.0 * pad).ceil();
        let content =
            svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
        return Ok(Rendered { svg: svg_document("gantt", width, height, size, ctx, acc, &content), width, height, links: vec![] });
    }
    let mut section_labels = IndexMap::new();
    let mut section_width = 0.0;
    for section in categories.keys() {
        let label = layout_label(Some(section), false, &m, section_size, 180.0, 0);
        section_width = js::max(section_width, label.width);
        section_labels.insert(section.clone(), label);
    }
    let axis_format = if !model.axis_format.is_empty() {
        &model.axis_format
    } else if model.date_format == "D" {
        "%d"
    } else {
        text(config, "axisFormat").unwrap_or("%Y-%m-%d")
    };
    let tick_label = |t| time_format(axis_format, &zone.at(t));
    let sample = js::max(m.width(&tick_label(start), tick_size, 0), m.width(&tick_label(stop), tick_size, 0));
    let left = pad
        + setting(config, "leftPadding", 0.0, 2000.0)
            .unwrap_or(js::max(if section_width > 0.0 { section_width + 24.0 } else { 0.0 }, sample / 2.0));
    let right_pad = setting(config, "rightPadding", 0.0, 2000.0).unwrap_or(js::max(16.0, sample / 2.0));
    let target = cap_width(options, setting(config, "useWidth", 100.0, 20000.0).unwrap_or(800.0), 280.0, 0.0);
    let plot_width = js::max(240.0, target - left - right_pad - pad);
    let right = left + plot_width;
    let scale = if stop > start { plot_width / (stop - start) } else { 0.0 };
    let x = |t: f64| if scale == 0.0 { left + plot_width / 2.0 } else { left + (t - start) * scale };
    let tick_h = js::round(tick_size * 1.75);
    let top_pad = setting(config, "topPadding", 0.0, 2000.0);
    let grid_start = setting(config, "gridLineStartPadding", 0.0, 2000.0);
    let head_h = pad + title_h + if top_axis { tick_h } else { 0.0 };
    let plot_top = js::max(head_h, top_pad.unwrap_or(0.0));
    let grid_lead = if let (Some(a), Some(b)) = (top_pad, grid_start) { js::max(0.0, js::min(a - b, plot_top - head_h)) } else { 0.0 };
    let plot_height = rows.max(1) as f64 * row_h;
    let plot_bottom = plot_top + plot_height;
    let mut bars = vec![];
    let mut extent = right + right_pad;
    for &i in &row_tasks {
        let task = &model.tasks[i];
        let label = one_line(&task.task, &m, label_size);
        let row = row_of[i];
        if !scheduled(&times[i]) {
            let lx = left + 8.0 + label.width / 2.0;
            extent = js::max(extent, lx + label.width / 2.0);
            bars.push(Bar { task: i, label, row, x0: f64::NAN, x1: f64::NAN, label_x: lx, inside: false });
            continue;
        }
        let mut x0 = x(times[i].start.unwrap().time());
        let mut x1 = js::max(x(times[i].render_end.or(times[i].end).unwrap().time()), x0 + if task.milestone { 0.0 } else { 2.0 });
        if task.milestone {
            let mid = x0 + (x(times[i].end.unwrap().time()) - x0) / 2.0;
            x0 = mid - bar_h / 2.0;
            x1 = mid + bar_h / 2.0;
        }
        let inside = !task.milestone && label.width + 16.0 <= x1 - x0;
        let mut lx = (x0 + x1) / 2.0;
        if !inside {
            let fits_right = x1 + 8.0 + label.width <= right + right_pad;
            let fits_left = x0 - 8.0 - label.width >= left;
            lx = if fits_right || !fits_left { x1 + 8.0 + label.width / 2.0 } else { x0 - 8.0 - label.width / 2.0 };
            extent = js::max(extent, lx + label.width / 2.0);
        }
        bars.push(Bar { task: i, label, row, x0, x1, label_x: lx, inside });
    }
    let interval = model.tick_interval.as_deref().filter(|s| !s.is_empty()).or_else(|| text(config, "tickInterval")).unwrap_or("");
    let weekday = model.weekday.as_deref().or_else(|| text(config, "weekday")).unwrap_or("sunday");
    let count = js::max(2.0, js::min(10.0, (plot_width / (sample + 24.0)).floor()));
    let ticks = interval_ticks(interval, weekday, start, stop, (plot_width / 3.0).ceil(), zone)
        .unwrap_or_else(|| auto_ticks(start, stop, count, zone));
    let (mut grid, mut axis_bottom, mut axis_top) = (String::new(), String::new(), String::new());
    let mut last_right = f64::NEG_INFINITY;
    let mut last_label = String::new();
    let bottom_y = plot_bottom + tick_h / 2.0 + 2.0;
    let top_y = plot_top - grid_lead - tick_h / 2.0 - 2.0;
    for t in ticks {
        let tx = x(t);
        svg_write!(grid, "M", num(tx), ",", num(plot_top - grid_lead), "V", num(plot_bottom)).unwrap();
        let label = tick_label(t);
        let w = m.width(&label, tick_size, 0);
        if tx - w / 2.0 < last_right + 8.0 || label == last_label {
            continue;
        }
        last_right = tx + w / 2.0;
        last_label = label.clone();
        extent = js::max(extent, last_right);
        let body = svg_format!("text-anchor=\"middle\">", esc(&label), "</text>");
        svg_write!(axis_bottom, "<text x=\"", num(tx), "\" y=\"", num(bottom_y + tick_size * 0.35), "\" ", body).unwrap();
        if top_axis {
            svg_write!(axis_top, "<text x=\"", num(tx), "\" y=\"", num(top_y + tick_size * 0.35), "\" ", body).unwrap();
        }
    }
    let (mut vert_lines, mut vert_labels) = (String::new(), String::new());
    let mut vert_ends: Vec<f64> = vec![];
    let vert_top = plot_bottom + tick_h + 4.0;
    let vert_line = js::round(label_size * 1.5);
    verts.sort_by(|&a, &b| times[a].start.unwrap().time().total_cmp(&times[b].start.unwrap().time()));
    for i in verts {
        let task = &model.tasks[i];
        let vx = x(times[i].start.unwrap().time());
        let label = one_line(&task.task, &m, label_size);
        let cx = js::max(pad + label.width / 2.0, vx);
        let line = vert_ends.iter().position(|&v| cx - label.width / 2.0 >= v + 8.0).unwrap_or(vert_ends.len());
        let end = cx + label.width / 2.0;
        if line == vert_ends.len() {
            vert_ends.push(end);
        } else {
            vert_ends[line] = end;
        }
        extent = js::max(extent, end);
        svg_write!(
            vert_lines,
            "<path class=\"pele-vert\" data-id=\"",
            esc_text(&task.id),
            "\" d=\"M",
            num(vx),
            ",",
            num(plot_top - grid_lead),
            "V",
            num(plot_bottom),
            "\"/>"
        )
        .unwrap();
        svg_write!(
            vert_labels,
            label_svg(
                &label,
                cx,
                vert_top + line as f64 * vert_line + vert_line / 2.0,
                &svg_format!(" class=\"pele-vert-label\" data-id=\"", esc_text(&task.id), "\""),
                None,
            )
        )
        .unwrap();
    }
    let width = (js::max(extent, pad + title.width) + pad).ceil();
    let height = (plot_bottom + tick_h + if vert_ends.is_empty() { 0.0 } else { 4.0 + vert_ends.len() as f64 * vert_line } + pad).ceil();
    let (mut bands, mut section_out) = (String::new(), String::new());
    let mut row = 0;
    while row < rows {
        let section = &row_section[row];
        let mut end = row + 1;
        while end < rows && row_section[end] == *section {
            end += 1;
        }
        let index = categories[section] % section_styles;
        let y = plot_top + row as f64 * row_h;
        let h = (end - row) as f64 * row_h;
        if categories.len() > 1 || !section.is_empty() {
            svg_write!(
                bands,
                "<rect class=\"pele-section pele-section-",
                index,
                "\" x=\"0\" y=\"",
                num(y),
                "\" width=\"",
                js::number_text(width),
                "\" height=\"",
                num(h),
                "\" fill=\"",
                if index % 2 == 0 { ctx.paint(Role::NodeFill, "var(--_s)") } else { "none" },
                "\"/>"
            )
            .unwrap();
        }
        let label = &section_labels[section];
        svg_write!(
            section_out,
            label_svg(
                label,
                pad + 8.0 + label.width / 2.0,
                y + h / 2.0,
                &svg_format!(" class=\"pele-section-label\" data-id=\"", esc_text(section), "\""),
                ctx.icons(),
            )
        )
        .unwrap();
        row = end;
    }
    let mut excluded = String::new();
    if (!model.excludes.is_empty() || !model.includes.is_empty()) && add(&zone.at(start), 6.0, Unit::Year).time() > stop {
        let mut range: Option<(JsDate, JsDate)> = None;
        let mut day = zone.at(start);
        let flush = |range: &mut Option<(JsDate, JsDate)>, out: &mut String| {
            if let Some((mut from, mut to)) = range.take() {
                from.set_hours(0.0, 0.0, 0.0, 0.0);
                to.set_hours(23.0, 59.0, 59.0, 999.0);
                let ex0 = js::max(left, x(from.time()));
                let ex1 = js::min(right, x(to.time()));
                if ex1 > ex0 {
                    svg_write!(
                        out,
                        "<rect x=\"",
                        num(ex0),
                        "\" y=\"",
                        num(plot_top),
                        "\" width=\"",
                        num(ex1 - ex0),
                        "\" height=\"",
                        num(plot_height),
                        "\"/>"
                    )
                    .unwrap();
                }
            }
        };
        while day.time() <= stop {
            if schedule.is_invalid_date(&day)? {
                if let Some(r) = &mut range {
                    r.1 = day;
                } else {
                    range = Some((day, day));
                }
            } else {
                flush(&mut range, &mut excluded);
            }
            let next = add_days(&day, 1.0);
            if next.time() <= day.time() {
                break;
            }
            day = next;
        }
        flush(&mut range, &mut excluded);
    }
    let mut links = vec![];
    let mut tasks_out = String::new();
    for bar in bars {
        let task = &model.tasks[bar.task];
        let cy = plot_top + bar.row as f64 * row_h + row_h / 2.0;
        let state = if task.active {
            "active"
        } else if task.done {
            "done"
        } else {
            ""
        };
        let mut classes = "pele-task".to_string();
        if !state.is_empty() {
            write!(classes, " pele-{state}").unwrap();
        }
        if task.crit {
            classes.push_str(" pele-crit");
        }
        if task.milestone {
            classes.push_str(" pele-milestone");
        }
        if task.classes.iter().any(|c| c == "clickable") {
            classes.push_str(" pele-clickable");
        }
        let mut shape = String::new();
        let mut text_fill = String::new();
        if bar.x0.is_nan() {
            classes.push_str(" pele-unscheduled");
            text_fill = svg_format!(" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"");
        } else {
            let color = if task.crit {
                ctx.paint(Role::DangerText, CRITICAL).to_owned()
            } else if ctx.resolved() {
                ctx.series(categories[&task.section] as i64).into_owned()
            } else {
                "var(--_c)".into()
            };
            let fill = if state == "done" {
                ctx.paint(Role::SuccessBackground, "var(--_a)").to_owned()
            } else if state == "active" && ctx.resolved() {
                ctx.paint(Role::Selection, "var(--_c)").to_owned()
            } else {
                color.clone()
            };
            let paint =
                svg_format!("fill=\"", fill, "\"", if state == "active" && !ctx.resolved() { " fill-opacity=\"0.25\"" } else { "" });
            let stroke = if state == "done" && !task.crit { ctx.paint(Role::NodeStroke, "var(--_b)") } else { &color };
            if task.milestone {
                let cx = (bar.x0 + bar.x1) / 2.0;
                let r = bar_h / 2.0;
                shape = svg_format!(
                    "<path d=\"M",
                    num(cx),
                    ",",
                    num(cy - r),
                    "L",
                    num(cx + r),
                    ",",
                    num(cy),
                    "L",
                    num(cx),
                    ",",
                    num(cy + r),
                    "L",
                    num(cx - r),
                    ",",
                    num(cy),
                    "Z\" ",
                    paint,
                    " stroke=\"",
                    stroke,
                    "\" stroke-linejoin=\"round\"/>"
                );
            } else {
                shape = svg_format!(
                    "<rect x=\"",
                    num(bar.x0),
                    "\" y=\"",
                    num(cy - bar_h / 2.0),
                    "\" width=\"",
                    num(bar.x1 - bar.x0),
                    "\" height=\"",
                    num(bar_h),
                    "\"",
                    crate::svg::theme::rounded(&svg_format!(" ", paint, " stroke=\"", stroke, "\"")),
                    "/>"
                );
            }
            if bar.inside {
                if ctx.resolved() {
                    text_fill = svg_format!(" fill=\"", ctx.theming.text_on(&fill, "var(--_bg)"), "\"");
                } else if state.is_empty() {
                    text_fill = " fill=\"var(--_bg)\"".into();
                }
            } else if state == "done" {
                text_fill = svg_format!(" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\"");
            }
        }
        let mut body = shape + &svg_format!(label_svg(&bar.label, bar.label_x, cy, &svg_format!(" class=\"pele-label\"", text_fill), None));
        if let Some(url) = model.links.get(&task.id) {
            body = linked(&body, url, &task.id, &mut links, options, "");
        }
        svg_write!(tasks_out, "<g class=\"", classes, "\" data-id=\"", esc_text(&task.id), "\">", body, "</g>").unwrap();
    }
    let mut today = String::new();
    if model.today_marker != "off" && now >= start && now <= stop && stop > start {
        let tx = x(now);
        let top = plot_top - grid_lead;
        let st = if model.today_marker.is_empty() {
            String::new()
        } else {
            resolve_style(&model.today_marker.split(',').collect::<Vec<_>>()).line
        };
        let accent = ctx.paint(Role::Accent, "var(--_c)");
        today = svg_format!(
            "<g class=\"pele-today\"><path d=\"M",
            num(tx),
            ",",
            num(top),
            "V",
            num(plot_bottom),
            "\" fill=\"none\" stroke=\"",
            accent,
            "\" stroke-width=\"2\"",
            st,
            "/>",
            if st.is_empty() {
                svg_format!(
                    "<path d=\"M",
                    num(tx - 4.0),
                    ",",
                    num(top - 5.0),
                    "H",
                    num(tx + 4.0),
                    "L",
                    num(tx),
                    ",",
                    num(top + 1.0),
                    "Z\" fill=\"",
                    accent,
                    "\"/>"
                )
            } else {
                String::new()
            },
            "</g>"
        );
    }
    let axis_line = svg_format!(
        "M",
        num(left),
        ",",
        num(plot_bottom),
        "H",
        num(right),
        if top_axis { svg_format!("M", num(left), ",", num(plot_top - grid_lead), "H", num(right)) } else { String::new() }
    );
    let mut content =
        svg_format!(label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None));
    if !bands.is_empty() {
        svg_write!(content, "<g class=\"pele-sections\">", bands, "</g>").unwrap();
    }
    if !excluded.is_empty() {
        svg_write!(content, "<g class=\"pele-excluded\" fill=\"", ctx.paint(Role::GroupHeader, "var(--_a)"), "\">", excluded, "</g>")
            .unwrap();
    }
    svg_write!(
        content,
        "<g class=\"pele-grid\" fill=\"none\"><path d=\"",
        grid,
        "\" stroke=\"",
        ctx.paint(Role::InnerStroke, "var(--_b)"),
        "\" stroke-dasharray=\"1 3\"/><path d=\"",
        axis_line,
        "\" stroke=\"",
        ctx.paint(Role::Line, "var(--_b)"),
        "\"/></g>"
    )
    .unwrap();
    if !vert_lines.is_empty() {
        svg_write!(
            content,
            "<g class=\"pele-verts\" fill=\"none\" stroke=\"",
            ctx.paint(Role::Line, "var(--_l)"),
            "\" stroke-width=\"1.5\" stroke-dasharray=\"4 3\">",
            vert_lines,
            "</g>"
        )
        .unwrap();
    }
    svg_write!(
        content,
        today,
        "<g class=\"pele-tasks\" font-size=\"",
        num(label_size),
        "\">",
        tasks_out,
        "</g><g class=\"pele-section-labels\" font-size=\"",
        num(section_size),
        "\">",
        section_out,
        "</g><g class=\"pele-axis\" font-size=\"",
        js::number_text(tick_size),
        "\" fill=\"",
        ctx.paint(Role::SecondaryText, "var(--_m)"),
        "\">",
        axis_bottom,
        axis_top,
        "</g>"
    )
    .unwrap();
    if !vert_labels.is_empty() {
        svg_write!(content, "<g class=\"pele-vert-labels\" font-size=\"", num(label_size), "\">", vert_labels, "</g>").unwrap();
    }
    Ok(Rendered { svg: svg_document("gantt", width, height, size, ctx, acc, &content), width, height, links })
}
