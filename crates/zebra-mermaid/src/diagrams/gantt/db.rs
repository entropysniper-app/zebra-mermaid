//! The gantt model and its scheduling. Ported from Pele `src/diagrams/gantt/db.ts`.
//!
//! Parsing records the tasks as written ([`Builder`], then [`GanttModel`]); nothing there reads
//! a date or the clock. Pele resolves the tasks' times in `getTasks(now)`, which its renderer
//! calls; here that is [`Schedule::new`], called by the renderer with `RenderOptions::now`
//! and the zone of `RenderOptions::timezone_offset_minutes`.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::date_parse::parse_date;
use super::dates::{add, add_days, compile, format, iso_date, parse_duration, parse_strict, Step, WEEKDAYS};
use super::js_date::{JsDate, Zone};
use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::chars::{char_at, is_space, is_word};
use crate::util::js;
use crate::util::url::safe_url;

/// A task as written.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GanttTask {
    pub id: String,
    /// The task's label, under the name Mermaid gives it.
    pub task: String,
    pub section: String,
    pub active: bool,
    pub done: bool,
    pub crit: bool,
    pub milestone: bool,
    pub vert: bool,
    /// Row index in source order; -1 for vertical markers, which take no row.
    pub order: i64,
    pub classes: Vec<String>,
    pub prev_task_id: Option<String>,
    /// The task's data as written, after the label.
    pub raw_data: String,
    pub raw_start: Option<String>,
    pub raw_end: String,
}

/// A parsed gantt chart. Its tasks' times are resolved when it is rendered, against the
/// render's `now` and time zone.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GanttModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub date_format: String,
    pub axis_format: String,
    pub tick_interval: Option<String>,
    pub today_marker: String,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub inclusive_end_dates: bool,
    pub top_axis: bool,
    pub display_mode: String,
    /// The day week-based ticks start on, when the chart names one.
    pub weekday: Option<String>,
    pub weekend: String,
    pub sections: Vec<String>,
    pub links: IndexMap<String, String>,
    pub tasks: Vec<GanttTask>,
}

/// What the parser's actions build, under Mermaid's names.
#[derive(Debug)]
pub(crate) struct Builder {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub date_format: String,
    pub axis_format: String,
    pub tick_interval: Option<String>,
    pub today_marker: String,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub inclusive_end_dates: bool,
    pub top_axis: bool,
    pub display_mode: String,
    pub weekday: Option<String>,
    pub weekend: String,
    pub sections: Vec<String>,
    pub links: IndexMap<String, String>,
    pub tasks: Vec<GanttTask>,
    by_id: HashMap<String, usize>,
    current_section: String,
    task_count: u64,
    last_order: i64,
    last_task_id: Option<String>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            title: None,
            acc_title: None,
            acc_descr: None,
            date_format: String::new(),
            axis_format: String::new(),
            tick_interval: None,
            today_marker: String::new(),
            includes: Vec::new(),
            excludes: Vec::new(),
            inclusive_end_dates: false,
            top_axis: false,
            display_mode: String::new(),
            weekday: None,
            weekend: "saturday".to_string(),
            sections: Vec::new(),
            links: IndexMap::new(),
            tasks: Vec::new(),
            by_id: HashMap::new(),
            current_section: String::new(),
            task_count: 0,
            last_order: 0,
            last_task_id: None,
        }
    }
}

/// `[...new Set([...existing, ...txt.toLowerCase().split(/[\s,]+/)])]` without empty tokens.
fn merge_tokens(existing: &mut Vec<String>, txt: &str) {
    for token in txt.to_lowercase().split(|c: char| is_space(c) || c == ',') {
        if !token.is_empty() && !existing.iter().any(|e| e == token) {
            existing.push(token.to_string());
        }
    }
}

/// `/^\s*(active|done|crit|milestone|vert)\s*$/`
fn tag(item: &str) -> Option<&'static str> {
    ["active", "done", "crit", "milestone", "vert"].into_iter().find(|&t| js::trim(item) == t)
}

impl Builder {
    pub fn set_includes(&mut self, txt: &str) {
        merge_tokens(&mut self.includes, txt);
    }

    pub fn set_excludes(&mut self, txt: &str) {
        merge_tokens(&mut self.excludes, txt);
    }

    pub fn set_acc_title(&mut self, txt: &str) {
        self.acc_title = Some(js::trim_start(txt).to_string());
    }

    pub fn set_acc_description(&mut self, txt: &str) {
        self.acc_descr = Some(crate::diagrams::squeeze_line_starts(txt));
    }

    pub fn add_section(&mut self, txt: &str) {
        self.current_section = txt.to_string();
        self.sections.push(txt.to_string());
    }

    pub fn add_task(&mut self, descr: &str, data_str: &str) -> Result<(), Error> {
        let body = data_str.strip_prefix(':').unwrap_or(data_str);
        let mut data: Vec<&str> = body.split(',').collect();
        let mut task = GanttTask {
            id: String::new(),
            task: descr.to_string(),
            section: self.current_section.clone(),
            active: false,
            done: false,
            crit: false,
            milestone: false,
            vert: false,
            order: -1,
            classes: Vec::new(),
            prev_task_id: self.last_task_id.clone(),
            raw_data: data_str.to_string(),
            raw_start: None,
            raw_end: String::new(),
        };
        while let Some(name) = data.first().and_then(|d| tag(d)) {
            match name {
                "active" => task.active = true,
                "done" => task.done = true,
                "crit" => task.crit = true,
                "milestone" => task.milestone = true,
                _ => task.vert = true,
            }
            data.remove(0);
        }
        // Mermaid fails with a TypeError on both of these.
        if data.is_empty() {
            return Err(
                Error::semantic(format!("Task \"{}\" needs a duration or an end date.", js::trim(descr))).with_type(DiagramType::Gantt)
            );
        }
        if data.len() > 3 {
            return Err(Error::semantic(format!(
                "Task \"{}\" has too many fields. A task takes an id, a start, and an end or duration.",
                js::trim(descr)
            ))
            .with_type(DiagramType::Gantt));
        }
        let data: Vec<&str> = data.into_iter().map(js::trim).collect();
        if data.len() == 3 {
            task.id = data[0].to_string();
        } else {
            self.task_count += 1;
            task.id = format!("task{}", self.task_count);
        }
        if data.len() > 1 {
            task.raw_start = Some(data[data.len() - 2].to_string());
        }
        task.raw_end = data[data.len() - 1].to_string();
        if !task.vert {
            task.order = self.last_order;
            self.last_order += 1;
        }
        self.last_task_id = Some(task.id.clone());
        self.by_id.insert(task.id.clone(), self.tasks.len());
        self.tasks.push(task);
        Ok(())
    }

    pub fn set_link(&mut self, ids: &str, link: &str) {
        let href = safe_url(link);
        for id in ids.split(',') {
            if self.by_id.contains_key(id) {
                self.links.insert(id.to_string(), href.clone());
            }
        }
        self.set_class(ids, "clickable");
    }

    /// Callbacks are recognized and never run; the task is only marked clickable, as in Mermaid.
    pub fn set_click_event(&mut self, ids: &str) {
        self.set_class(ids, "clickable");
    }

    fn set_class(&mut self, ids: &str, class_name: &str) {
        for id in ids.split(',') {
            if let Some(&i) = self.by_id.get(id) {
                self.tasks[i].classes.push(class_name.to_string());
            }
        }
    }

    pub fn finish(self) -> GanttModel {
        GanttModel {
            title: self.title,
            acc_title: self.acc_title,
            acc_descr: self.acc_descr,
            date_format: self.date_format,
            axis_format: self.axis_format,
            tick_interval: self.tick_interval,
            today_marker: self.today_marker,
            includes: self.includes,
            excludes: self.excludes,
            inclusive_end_dates: self.inclusive_end_dates,
            top_axis: self.top_axis,
            display_mode: self.display_mode,
            weekday: self.weekday,
            weekend: self.weekend,
            sections: self.sections,
            links: self.links,
            tasks: self.tasks,
        }
    }
}

// Scheduling walks tasks day by day and parses dates with a format the diagram supplies. This
// bounds the total, in rough character operations, so a hostile diagram fails instead of hanging.
const WORK_LIMIT: f64 = 4e8;
const DAY_STEP: f64 = 400.0;
const DATED_STEP: f64 = 200.0;
const FORMAT_STEP: f64 = 400.0;

/// When a task starts and ends, once scheduled. Unset when what it refers to never resolves.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Times {
    pub start: Option<JsDate>,
    pub end: Option<JsDate>,
    /// Where the bar ends when excluded days pushed the task's end out; `None` when the end
    /// was given as a date.
    pub render_end: Option<JsDate>,
    manual_end: bool,
    processed: bool,
}

/// The tasks of a chart scheduled against a time and zone: Pele's `getTasks(now)` and the state
/// it leaves for `isInvalidDate`.
pub(crate) struct Schedule<'a> {
    model: &'a GanttModel,
    by_id: HashMap<&'a str, usize>,
    now: f64,
    zone: Zone,
    work: f64,
    include_set: HashSet<String>,
    exclude_set: HashSet<String>,
    excluded_days: [bool; 7],
    dated_tokens: bool,
    template: &'a str,
    steps: Option<Vec<Step>>,
    iso_steps: Option<Vec<Step>>,
    pub times: Vec<Times>,
}

/// `/^[YMDHhmsSZXx\-/.:, T]*$/`
fn numeric_format(template: &str) -> bool {
    template.bytes().all(|b| b"YMDHhmsSZXx-/.:, T".contains(&b))
}

/// `/^after\s+([\d\w- ]+)/` and `/^until\s+([\d\w- ]+)/`: the list of ids.
fn reference_list<'s>(s: &'s str, keyword: &str) -> Option<&'s str> {
    if !s.starts_with(keyword) {
        return None;
    }
    let in_class = |c: char| is_word(c) || c == '-' || c == ' ';
    let mut ends = Vec::new();
    let mut q = keyword.len();
    while let Some(c) = char_at(s, q).filter(|&c| is_space(c)) {
        q += c.len_utf8();
        ends.push(q);
    }
    // `\s+` gives back characters until the list can start.
    for &start in ends.iter().rev() {
        let mut end = start;
        while let Some(c) = char_at(s, end).filter(|&c| in_class(c)) {
            end += c.len_utf8();
        }
        if end > start {
            return Some(&s[start..end]);
        }
    }
    None
}

fn later(a: Option<JsDate>, b: Option<JsDate>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if a.time() > b.time())
}

fn earlier(a: Option<JsDate>, b: Option<JsDate>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if a.time() < b.time())
}

fn semantic(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Gantt)
}

impl<'a> Schedule<'a> {
    /// Resolves every task's start and end. A task may refer to tasks defined after it, so,
    /// like Mermaid, this makes up to eleven passes, each of which schedules every task again.
    pub fn new(model: &'a GanttModel, now: f64, zone: Zone) -> Result<Self, Error> {
        let mut by_id = HashMap::new();
        for (i, task) in model.tasks.iter().enumerate() {
            by_id.insert(task.id.as_str(), i);
        }
        let weekend_start = match model.weekend.as_str() {
            "friday" => 5.0,
            "saturday" => 6.0,
            _ => f64::NAN,
        };
        let exclude_set: HashSet<String> = model.excludes.iter().cloned().collect();
        let weekends = exclude_set.contains("weekends");
        let excluded_days: [bool; 7] = std::array::from_fn(|day| {
            let iso = if day == 0 { 7.0 } else { day as f64 };
            (weekends && (iso == weekend_start || iso == weekend_start + 1.0)) || exclude_set.contains(&WEEKDAYS[day].to_lowercase())
        });
        let template = js::trim(&model.date_format);
        // A date written with only these tokens and separators can never read as a weekday
        // name, so when nothing else is excluded or included there is no need to format each day.
        let mut dated_tokens = !model.includes.is_empty() || !numeric_format(template);
        for token in &model.excludes {
            if token != "weekends" && !WEEKDAYS.iter().any(|d| d.to_lowercase() == *token) {
                dated_tokens = true;
            }
        }
        let mut s = Schedule {
            model,
            by_id,
            now,
            zone,
            work: 0.0,
            include_set: model.includes.iter().cloned().collect(),
            exclude_set,
            excluded_days,
            dated_tokens,
            template,
            steps: compile(template),
            iso_steps: compile("YYYY-MM-DD"),
            times: vec![Times::default(); model.tasks.len()],
        };
        let mut all_processed = s.compile_tasks()?;
        let mut i = 0;
        while !all_processed && i < 10 {
            all_processed = s.compile_tasks()?;
            i += 1;
        }
        Ok(s)
    }

    /// Whether a day is excluded. A day matches a token by its weekday name, by its date as
    /// YYYY-MM-DD, or by its date written in the chart's own format.
    pub fn is_invalid_date(&mut self, date: &JsDate) -> Result<bool, Error> {
        let d = date.day();
        let weekday = !d.is_nan() && self.excluded_days[d as usize];
        if !self.dated_tokens {
            return Ok(weekday);
        }
        let date_only = iso_date(date);
        let mut formatted = Some(date_only.clone());
        self.spend(DATED_STEP)?;
        if self.template != "YYYY-MM-DD" {
            self.spend(FORMAT_STEP + js::len16(self.template) as f64 * 8.0)?;
            formatted = format(date, self.template);
        }
        let listed = |set: &HashSet<String>| set.contains(&date_only) || formatted.as_ref().is_some_and(|f| set.contains(f));
        if listed(&self.include_set) {
            return Ok(false);
        }
        Ok(weekday || listed(&self.exclude_set))
    }

    fn spend(&mut self, units: f64) -> Result<(), Error> {
        self.work += units;
        if self.work > WORK_LIMIT {
            return Err(Error::limit("The gantt chart takes too much work to schedule.").with_type(DiagramType::Gantt));
        }
        Ok(())
    }

    fn parse(&mut self, s: &str, iso: bool) -> Result<Option<JsDate>, Error> {
        let template = if iso { "YYYY-MM-DD" } else { self.template };
        self.spend((js::len16(template) as f64 + 1.0) * (js::len16(s) as f64 + 1.0))?;
        let steps = if iso { self.iso_steps.as_deref() } else { self.steps.as_deref() };
        Ok(parse_strict(s, template, steps, &self.zone.at(self.now)))
    }

    fn today(&self) -> JsDate {
        let mut today = self.zone.at(self.now);
        today.set_hours(0.0, 0.0, 0.0, 0.0);
        today
    }

    fn compile_tasks(&mut self) -> Result<bool, Error> {
        let mut all_processed = true;
        for i in 0..self.model.tasks.len() {
            self.compile_task(i)?;
            all_processed = all_processed && self.times[i].processed;
        }
        Ok(all_processed)
    }

    fn compile_task(&mut self, i: usize) -> Result<(), Error> {
        let task = &self.model.tasks[i];
        match &task.raw_start {
            None => {
                // Mermaid fails with a TypeError when the first task has no start. It starts
                // today here.
                let previous = task.prev_task_id.as_deref().and_then(|id| self.by_id.get(id).copied());
                self.times[i].start = match previous {
                    Some(p) => self.times[p].end,
                    None => Some(self.today()),
                };
            }
            Some(start) => {
                if let Some(start_time) = self.get_start_date(start)? {
                    self.times[i].start = Some(start_time);
                }
            }
        }
        let Some(start) = self.times[i].start else { return Ok(()) };
        self.times[i].end = self.get_end_date(&start, &task.raw_end)?;
        if self.times[i].end.is_none() {
            return Ok(());
        }
        self.times[i].processed = true;
        self.times[i].manual_end = self.parse(&task.raw_end, true)?.is_some();
        self.check_task_dates(i)
    }

    fn references(&self, list: &str) -> Vec<usize> {
        list.split(' ').filter(|id| !id.is_empty()).filter_map(|id| self.by_id.get(id).copied()).collect()
    }

    fn get_start_date(&mut self, raw: &str) -> Result<Option<JsDate>, Error> {
        let s = js::trim(raw);
        let template = self.template;
        if (template == "x" || template == "X") && !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            let date = self.zone.at(js::number(s));
            if !date.is_valid() {
                return Err(semantic(format!("Invalid date:{}", s)));
            }
            return Ok(Some(date));
        }
        if let Some(list) = reference_list(s, "after") {
            let mut latest: Option<usize> = None;
            for t in self.references(list) {
                if latest.is_none_or(|l| later(self.times[t].end, self.times[l].end)) {
                    latest = Some(t);
                }
            }
            return Ok(match latest {
                Some(l) => self.times[l].end,
                None => Some(self.today()),
            });
        }
        if let Some(parsed) = self.parse(s, false)? {
            return Ok(Some(parsed));
        }
        // What does not fit the format gets a second chance with the JavaScript date parser,
        // as in Mermaid.
        let date = self.zone.at(parse_date(s, self.zone));
        if !date.is_valid() || date.full_year() < -10000.0 || date.full_year() > 10000.0 {
            return Err(semantic(format!("Invalid date:{}", s)));
        }
        Ok(Some(date))
    }

    fn get_end_date(&mut self, start: &JsDate, raw: &str) -> Result<Option<JsDate>, Error> {
        let s = js::trim(raw);
        if let Some(list) = reference_list(s, "until") {
            let mut earliest: Option<usize> = None;
            for t in self.references(list) {
                if earliest.is_none_or(|e| earlier(self.times[t].start, self.times[e].start)) {
                    earliest = Some(t);
                }
            }
            return Ok(match earliest {
                Some(e) => self.times[e].start,
                None => Some(self.today()),
            });
        }
        if let Some(parsed) = self.parse(s, false)? {
            return Ok(Some(if self.model.inclusive_end_dates { add_days(&parsed, 1.0) } else { parsed }));
        }
        let Some((value, unit)) = parse_duration(s) else {
            // Pele warns that the text is neither a date nor a duration; the task gets no duration.
            return Ok(Some(*start));
        };
        let end = add(start, value, unit);
        Ok(Some(if end.is_valid() { end } else { *start }))
    }

    /// Pushes the end of a task out by one day for every excluded day it covers.
    fn check_task_dates(&mut self, i: usize) -> Result<(), Error> {
        let times = self.times[i];
        if self.model.excludes.is_empty() || times.manual_end {
            return Ok(());
        }
        let (Some(start), Some(end)) = (times.start, times.end) else { return Ok(()) };
        let mut cursor = add_days(&start, 1.0);
        let mut end_time = end;
        let max_end_time = add_days(&end_time, 10000.0).time();
        let mut invalid = false;
        let mut render_end: Option<f64> = None;
        while cursor.time() <= end_time.time() {
            self.spend(DAY_STEP)?;
            if !invalid {
                render_end = Some(end_time.time());
            }
            invalid = self.is_invalid_date(&cursor)?;
            if invalid {
                end_time.set_date(end_time.date() + 1.0);
                if end_time.time() > max_end_time {
                    return Err(semantic(
                        "Failed to find a valid date that was not excluded by `excludes` after 10,000 iterations.".to_string(),
                    ));
                }
            }
            cursor.set_date(cursor.date() + 1.0);
        }
        self.times[i].end = Some(end_time);
        self.times[i].render_end = render_end.map(|t| self.zone.at(t));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_lists() {
        assert_eq!(reference_list("after a b", "after"), Some("a b"));
        // The greedy whitespace matcher backtracks, leaving a space in the ID class.
        assert_eq!(reference_list("after  \t.x", "after"), Some(" "));
        assert_eq!(reference_list("after  .x", "after"), Some(" "));
        assert_eq!(reference_list("afterwards", "after"), None);
        assert_eq!(reference_list("until x-1 y", "until"), Some("x-1 y"));
    }

    #[test]
    fn tokens_merge() {
        let mut v = vec!["weekends".to_string()];
        merge_tokens(&mut v, "Sunday, 2024-01-01  weekends");
        assert_eq!(v, vec!["weekends", "sunday", "2024-01-01"]);
    }
}
