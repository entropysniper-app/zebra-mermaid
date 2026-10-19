//! The parts of dayjs 1.11 that Mermaid's gantt code relies on. Ported from Pele
//! `src/diagrams/gantt/dates.ts`: strict parsing against a format, formatting, and calendar
//! arithmetic, all in local time, with the custom parse format, advanced format and ISO week
//! plugins and the English locale. Quirks are kept, because a date is accepted only when
//! formatting the parsed result gives back the input.
//!
//! The regular expressions of the original are written out as scanners; each is quoted above
//! the code that replaces it. Parsing reads the input in UTF-16 code units, as the original
//! slices and searches it by code unit.

use super::js_date::{JsDate, MS_PER_HOUR, MS_PER_MINUTE, MS_PER_SECOND};
use crate::util::chars::is_space;
use crate::util::js;

pub(crate) const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
pub(crate) const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

const DAY: f64 = 86_400_000.0;
const WEEK: f64 = 7.0 * DAY;

/// The units of a duration such as `3d`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// `pad(value, length)`: zeros before `String(value)`, even before a minus sign.
fn pad(value: f64, length: usize) -> String {
    let s = js::to_string(value);
    let n = s.len();
    if n >= length {
        s
    } else {
        "0".repeat(length - n) + &s
    }
}

/// `n + (s[(v - 20) % 10] || s[v] || s[0])` with `s = ['th', 'st', 'nd', 'rd']`.
fn ordinal(n: f64) -> String {
    let s = ["th", "st", "nd", "rd"];
    let v = n % 100.0;
    let pick = |i: f64| -> Option<&str> {
        if i >= 0.0 && i < 4.0 && i.fract() == 0.0 {
            Some(s[i as usize])
        } else {
            None
        }
    };
    let suffix = pick((v - 20.0) % 10.0).or_else(|| pick(v)).unwrap_or(s[0]);
    format!("{}{}", js::to_string(n), suffix)
}

/// `-Math.round(date.getTimezoneOffset() / 15) * 15`, with -0 as 0.
fn utc_offset(date: &JsDate) -> f64 {
    let offset = -js::round(date.timezone_offset() / 15.0) * 15.0;
    if offset == 0.0 {
        0.0
    } else {
        offset
    }
}

fn zone(date: &JsDate) -> String {
    let offset = utc_offset(date);
    let minutes = offset.abs();
    format!("{}{}:{}", if offset >= 0.0 { '+' } else { '-' }, pad((minutes / 60.0).floor(), 2), pad(minutes % 60.0, 2))
}

/// `date.getDay() || 7`.
pub(crate) fn iso_weekday(date: &JsDate) -> f64 {
    let d = date.day();
    if d == 0.0 || d.is_nan() {
        7.0
    } else {
        d
    }
}

pub(crate) fn add_days(date: &JsDate, days: f64) -> JsDate {
    let mut next = *date;
    next.set_date(next.date() + days);
    next
}

fn days_in_month(date: &JsDate) -> f64 {
    let mut last = date.zone().local(date.full_year(), date.month() + 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    last.set_hours(23.0, 59.0, 59.0, 999.0);
    last.date()
}

/// Sets the month or the year, keeping the day of the month unless the new month is shorter.
fn set_calendar(date: &JsDate, apply: impl FnOnce(&mut JsDate)) -> JsDate {
    let mut next = *date;
    next.set_date(1.0);
    apply(&mut next);
    let day = js::min(date.date(), days_in_month(&next));
    next.set_date(day);
    next
}

pub(crate) fn add(date: &JsDate, amount: f64, unit: Unit) -> JsDate {
    let zone = date.zone();
    match unit {
        Unit::Month => set_calendar(date, |d| d.set_month(date.month() + amount, None)),
        Unit::Year => set_calendar(date, |d| d.set_full_year(date.full_year() + amount)),
        Unit::Day => add_days(date, js::round(amount)),
        Unit::Week => add_days(date, js::round(7.0 * amount)),
        Unit::Hour => zone.at(date.time() + amount * MS_PER_HOUR),
        Unit::Minute => zone.at(date.time() + amount * MS_PER_MINUTE),
        Unit::Second => zone.at(date.time() + amount * MS_PER_SECOND),
        Unit::Millisecond => zone.at(date.time() + amount),
    }
}

fn week_thursday(date: &JsDate) -> JsDate {
    add_days(date, 4.0 - iso_weekday(date))
}

fn iso_week(date: &JsDate) -> f64 {
    let thursday = week_thursday(date);
    let jan1 = date.zone().local(thursday.full_year(), 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    let weekday = iso_weekday(&jan1);
    let first = add_days(&jan1, 4.0 - weekday + if weekday > 4.0 { 7.0 } else { 0.0 });
    let zone_delta = (utc_offset(&first) - utc_offset(&thursday)) * 60000.0;
    let weeks = (thursday.time() - first.time() - zone_delta) / WEEK;
    let whole = if weeks < 0.0 {
        let c = weeks.ceil();
        // `Math.ceil(weeks) || 0`: -0 becomes 0.
        if c == 0.0 {
            0.0
        } else {
            c
        }
    } else {
        weeks.floor()
    };
    whole + 1.0
}

/// `\[([^\]]+)]` at byte `i`: the end of the bracketed literal.
fn bracket_literal(s: &str, i: usize) -> Option<usize> {
    let b = s.as_bytes();
    if b.get(i) != Some(&b'[') || b.get(i + 1).is_none_or(|&c| c == b']') {
        return None;
    }
    s[i + 1..].find(']').map(|k| i + 1 + k + 1)
}

/// The length of the first of `alternatives` that `s` has at `i`.
fn literal_at(s: &str, i: usize, alternatives: &[&str]) -> Option<usize> {
    alternatives.iter().find(|a| s[i..].starts_with(*a)).map(|a| a.len())
}

/// The length of a run of `c` at `i`, at most `max`, if it is at least `min`.
fn run_at(s: &str, i: usize, c: u8, min: usize, max: usize) -> Option<usize> {
    let n = s.as_bytes()[i..].iter().take(max).take_while(|&&b| b == c).count();
    (n >= min).then_some(n)
}

/// Formats a date with a dayjs format. Returns `None` for the week-of-year and time zone name
/// tokens, which need dayjs plugins Mermaid does not load and so fail there.
pub(crate) fn format(date: &JsDate, template: &str) -> Option<String> {
    if !date.is_valid() {
        return Some("Invalid Date".to_string());
    }
    let s = if template.is_empty() { "YYYY-MM-DDTHH:mm:ssZ" } else { template };
    if s == "YYYY-MM-DD" {
        return Some(iso_date(date));
    }
    // `/\[([^\]]+)]|Q|wo|ww|w|WW|W|zzz|z|gggg|GGGG|Do|X|x|k{1,2}|S/g`
    let mut advanced = String::with_capacity(s.len());
    let mut unsupported = false;
    let mut i = 0;
    while i < s.len() {
        let len = bracket_literal(s, i)
            .map(|end| end - i)
            .or_else(|| literal_at(s, i, &["Q", "wo", "ww", "w", "WW", "W", "zzz", "z", "gggg", "GGGG", "Do", "X", "x"]))
            .or_else(|| run_at(s, i, b'k', 1, 2))
            .or_else(|| literal_at(s, i, &["S"]));
        let Some(len) = len else {
            let c = crate::util::chars::char_at(s, i).unwrap_or('\u{fffd}');
            advanced.push(c);
            i += c.len_utf8();
            continue;
        };
        let m = &s[i..i + len];
        match m {
            "Q" => advanced.push_str(&js::to_string(((date.month() + 1.0) / 3.0).ceil())),
            "Do" => {
                advanced.push('[');
                advanced.push_str(&ordinal(date.date()));
                advanced.push(']');
            }
            "GGGG" => advanced.push_str(&js::to_string(week_thursday(date).full_year())),
            "W" => advanced.push_str(&js::to_string(iso_week(date))),
            "WW" => advanced.push_str(&pad(iso_week(date), 2)),
            "k" => advanced.push_str(&js::to_string(hours_or_24(date))),
            "kk" => advanced.push_str(&pad(hours_or_24(date), 2)),
            "X" => advanced.push_str(&js::to_string((date.time() / 1000.0).floor())),
            "x" => advanced.push_str(&js::to_string(date.time())),
            "gggg" | "wo" | "w" | "ww" | "z" | "zzz" => {
                unsupported = true;
                advanced.push_str(m);
            }
            _ => advanced.push_str(m),
        }
        i += len;
    }
    if unsupported {
        return None;
    }
    let s = advanced.as_str();
    let hours = date.hours();
    // `/\[([^\]]+)]|YYYY|YY|M{1,4}|D{1,2}|d{1,4}|H{1,2}|h{1,2}|a|A|m{1,2}|s{1,2}|Z{1,2}|SSS/g`
    let mut out = String::with_capacity(s.len() + 8);
    let mut i = 0;
    while i < s.len() {
        if let Some(end) = bracket_literal(s, i) {
            out.push_str(&s[i + 1..end - 1]);
            i = end;
            continue;
        }
        let len = literal_at(s, i, &["YYYY", "YY"])
            .or_else(|| run_at(s, i, b'M', 1, 4))
            .or_else(|| run_at(s, i, b'D', 1, 2))
            .or_else(|| run_at(s, i, b'd', 1, 4))
            .or_else(|| run_at(s, i, b'H', 1, 2))
            .or_else(|| run_at(s, i, b'h', 1, 2))
            .or_else(|| literal_at(s, i, &["a", "A"]))
            .or_else(|| run_at(s, i, b'm', 1, 2))
            .or_else(|| run_at(s, i, b's', 1, 2))
            .or_else(|| run_at(s, i, b'Z', 1, 2))
            .or_else(|| literal_at(s, i, &["SSS"]));
        let Some(len) = len else {
            let c = crate::util::chars::char_at(s, i).unwrap_or('\u{fffd}');
            out.push(c);
            i += c.len_utf8();
            continue;
        };
        let month = date.month() as usize;
        let weekday = date.day() as usize;
        let piece = match &s[i..i + len] {
            "YY" => {
                let year = js::to_string(date.full_year());
                let n = year.len();
                year[n.saturating_sub(2)..].to_string()
            }
            "YYYY" => pad(date.full_year(), 4),
            "M" => js::to_string(date.month() + 1.0),
            "MM" => pad(date.month() + 1.0, 2),
            "MMM" => MONTHS[month][..3].to_string(),
            "MMMM" => MONTHS[month].to_string(),
            "D" => js::to_string(date.date()),
            "DD" => pad(date.date(), 2),
            "d" => js::to_string(date.day()),
            "dd" => WEEKDAYS[weekday][..2].to_string(),
            "ddd" => WEEKDAYS[weekday][..3].to_string(),
            "dddd" => WEEKDAYS[weekday].to_string(),
            "H" => js::to_string(hours),
            "HH" => pad(hours, 2),
            "h" => js::to_string(twelve(hours)),
            "hh" => pad(twelve(hours), 2),
            "a" => (if hours < 12.0 { "am" } else { "pm" }).to_string(),
            "A" => (if hours < 12.0 { "AM" } else { "PM" }).to_string(),
            "m" => js::to_string(date.minutes()),
            "mm" => pad(date.minutes(), 2),
            "s" => js::to_string(date.seconds()),
            "ss" => pad(date.seconds(), 2),
            "SSS" => pad(date.milliseconds(), 3),
            "Z" => zone(date),
            _ => zone(date).replacen(':', "", 1),
        };
        out.push_str(&piece);
        i += len;
    }
    Some(out)
}

fn hours_or_24(date: &JsDate) -> f64 {
    let h = date.hours();
    if h == 0.0 || h.is_nan() {
        24.0
    } else {
        h
    }
}

fn twelve(hours: f64) -> f64 {
    let h = hours % 12.0;
    if h == 0.0 || h.is_nan() {
        12.0
    } else {
        h
    }
}

/// `YYYY-MM-DD`, with the year padded to four digits when it is outside 1000 to 9999.
pub(crate) fn iso_date(date: &JsDate) -> String {
    let year = date.full_year();
    let month = date.month() + 1.0;
    let day = date.date();
    let y = if (1000.0..=9999.0).contains(&year) { js::to_string(year) } else { pad(year, 4) };
    format!(
        "{}{}{}{}{}",
        y,
        if month < 10.0 { "-0" } else { "-" },
        js::to_string(month),
        if day < 10.0 { "-0" } else { "-" },
        js::to_string(day)
    )
}

#[derive(Debug, Default)]
struct Fields {
    year: Option<f64>,
    month: Option<f64>,
    day: Option<f64>,
    day_given: bool,
    hours: Option<f64>,
    minutes: Option<f64>,
    seconds: Option<f64>,
    milliseconds: Option<f64>,
    week: Option<f64>,
    offset: Option<f64>,
    afternoon: Option<bool>,
}

/// The regular expression a reader matches with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Match {
    /// `/\d/`
    One,
    /// `/\d\d/`
    Two,
    /// `/\d{3}/`
    Three,
    /// `/\d{4}/`
    Four,
    /// `/\d\d?/`
    OneToTwo,
    /// `/[+-]?\d+/`
    Signed,
    /// `/[+-]\d\d:?(\d\d)?|Z/`
    Offset,
    /// `/\d*[^-_:/,()\s\d]+/`
    Word,
}

/// What a reader does with the text it matched.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Read {
    Afternoon,
    AfternoonLower,
    Quarter,
    Tenths,
    Hundredths,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Day,
    Ordinal,
    Week,
    Month,
    ShortMonthName,
    MonthName,
    Year,
    TwoDigitYear,
    Offset,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Step {
    Literal(usize),
    Reader(Match, Read),
}

fn reader(token: &str) -> Option<Step> {
    use Match as M;
    use Read as R;
    let (m, r) = match token {
        "A" => (M::Word, R::Afternoon),
        "a" => (M::Word, R::AfternoonLower),
        "Q" => (M::One, R::Quarter),
        "S" => (M::One, R::Tenths),
        "SS" => (M::Two, R::Hundredths),
        "SSS" => (M::Three, R::Milliseconds),
        "s" | "ss" => (M::OneToTwo, R::Seconds),
        "m" | "mm" => (M::OneToTwo, R::Minutes),
        "H" | "h" | "HH" | "hh" => (M::OneToTwo, R::Hours),
        "D" => (M::OneToTwo, R::Day),
        "DD" => (M::Two, R::Day),
        "Do" => (M::Word, R::Ordinal),
        "w" => (M::OneToTwo, R::Week),
        "ww" => (M::Two, R::Week),
        "M" => (M::OneToTwo, R::Month),
        "MM" => (M::Two, R::Month),
        "MMM" => (M::Word, R::ShortMonthName),
        "MMMM" => (M::Word, R::MonthName),
        "Y" => (M::Signed, R::Year),
        "YY" => (M::Two, R::TwoDigitYear),
        "YYYY" => (M::Four, R::Year),
        "Z" | "ZZ" => (M::Offset, R::Offset),
        _ => return None,
    };
    Some(Step::Reader(m, r))
}

/// `/[-_:/.,()\s]/` for one character.
fn is_separator(c: char) -> bool {
    matches!(c, '-' | '_' | ':' | '/' | '.' | ',' | '(' | ')') || is_space(c)
}

/// `template.match(RE_TOKENS)` with
/// `RE_TOKENS = /(\[[^[]*\])|([-_:/.,()\s]+)|(A|a|Q|YYYY|YY?|ww?|MM?M?M?|Do|DD?|hh?|HH?|mm?|ss?|S{1,3}|z|ZZ?)/g`.
/// Characters that start none of these are left out, as `match` leaves them.
fn tokens(template: &str) -> Vec<&str> {
    let b = template.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        // `\[[^[]*\]`: to the last `]` before the next `[`.
        if b[i] == b'[' {
            let stop = template[i + 1..].find('[').map_or(b.len(), |k| i + 1 + k);
            if let Some(k) = template[i + 1..stop].rfind(']') {
                let end = i + 1 + k + 1;
                out.push(&template[i..end]);
                i = end;
                continue;
            }
        }
        let c = crate::util::chars::char_at(template, i).unwrap_or('\u{fffd}');
        if is_separator(c) {
            let mut end = i;
            while let Some(d) = crate::util::chars::char_at(template, end).filter(|&d| is_separator(d)) {
                end += d.len_utf8();
            }
            out.push(&template[i..end]);
            i = end;
            continue;
        }
        let len = literal_at(template, i, &["A", "a", "Q", "YYYY"])
            .or_else(|| run_at(template, i, b'Y', 1, 2))
            .or_else(|| run_at(template, i, b'w', 1, 2))
            .or_else(|| run_at(template, i, b'M', 1, 4))
            .or_else(|| literal_at(template, i, &["Do"]))
            .or_else(|| run_at(template, i, b'D', 1, 2))
            .or_else(|| run_at(template, i, b'h', 1, 2))
            .or_else(|| run_at(template, i, b'H', 1, 2))
            .or_else(|| run_at(template, i, b'm', 1, 2))
            .or_else(|| run_at(template, i, b's', 1, 2))
            .or_else(|| run_at(template, i, b'S', 1, 3))
            .or_else(|| literal_at(template, i, &["z"]))
            .or_else(|| run_at(template, i, b'Z', 1, 2));
        match len {
            Some(len) => {
                out.push(&template[i..i + len]);
                i += len;
            }
            None => i += c.len_utf8(),
        }
    }
    out
}

/// Whether `/[Ll]/` finds a letter outside the `\[[^\]]+]` literals of a template.
fn has_localized_token(template: &str) -> bool {
    let mut i = 0;
    while i < template.len() {
        if let Some(end) = bracket_literal(template, i) {
            i = end;
            continue;
        }
        let c = crate::util::chars::char_at(template, i).unwrap_or('\u{fffd}');
        if c == 'L' || c == 'l' {
            return true;
        }
        i += c.len_utf8();
    }
    false
}

/// The steps of a parse format, or `None` when dayjs cannot parse with it.
pub(crate) fn compile(template: &str) -> Option<Vec<Step>> {
    // dayjs expands localized tokens through a locale table the English locale does not have,
    // and fails.
    if has_localized_token(template) {
        return None;
    }
    let tokens = tokens(template);
    if tokens.is_empty() {
        return None;
    }
    Some(
        tokens
            .into_iter()
            .map(|token| {
                reader(token).unwrap_or_else(|| {
                    // `token.replace(/^\[|\]$/g, '')`
                    let t = token.strip_prefix('[').unwrap_or(token);
                    let t = t.strip_suffix(']').unwrap_or(t);
                    Step::Literal(js::len16(t))
                })
            })
            .collect(),
    )
}

fn is_digit16(c: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&c)
}

/// `\s` for one UTF-16 code unit.
fn is_space16(c: u16) -> bool {
    char::from_u32(c as u32).is_some_and(is_space)
}

/// The first match of `m` in `s`, as `re.exec(s)` finds it: start and end.
fn find(m: Match, s: &[u16]) -> Option<(usize, usize)> {
    let digit = |i: usize| s.get(i).is_some_and(|&c| is_digit16(c));
    let digits_from = |mut i: usize| {
        while digit(i) {
            i += 1;
        }
        i
    };
    for i in 0..s.len() {
        let found = match m {
            Match::One => digit(i).then_some(i + 1),
            Match::Two => (digit(i) && digit(i + 1)).then_some(i + 2),
            Match::Three => (0..3).all(|k| digit(i + k)).then_some(i + 3),
            Match::Four => (0..4).all(|k| digit(i + k)).then_some(i + 4),
            Match::OneToTwo => digit(i).then(|| if digit(i + 1) { i + 2 } else { i + 1 }),
            Match::Signed => {
                if (s[i] == b'+' as u16 || s[i] == b'-' as u16) && digit(i + 1) {
                    Some(digits_from(i + 1))
                } else if digit(i) {
                    Some(digits_from(i))
                } else {
                    None
                }
            }
            Match::Offset => {
                if (s[i] == b'+' as u16 || s[i] == b'-' as u16) && digit(i + 1) && digit(i + 2) {
                    let mut j = i + 3;
                    if s.get(j) == Some(&(b':' as u16)) {
                        j += 1;
                    }
                    if digit(j) && digit(j + 1) {
                        j += 2;
                    }
                    Some(j)
                } else if s[i] == b'Z' as u16 {
                    Some(i + 1)
                } else {
                    None
                }
            }
            Match::Word => {
                let j = digits_from(i);
                let word = |k: usize| {
                    s.get(k).is_some_and(|&c| {
                        !is_digit16(c) && !is_space16(c) && ![b'-', b'_', b':', b'/', b',', b'(', b')'].iter().any(|&x| c == x as u16)
                    })
                };
                if word(j) {
                    let mut k = j;
                    while word(k) {
                        k += 1;
                    }
                    Some(k)
                } else {
                    None
                }
            }
        };
        if let Some(end) = found {
            return Some((i, end));
        }
    }
    None
}

fn number16(s: &[u16]) -> f64 {
    js::number(&String::from_utf16_lossy(s))
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `input.match(/([+-]|\d\d)/g)`
fn offset_parts(s: &[u16]) -> Vec<&[u16]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'+' as u16 || s[i] == b'-' as u16 {
            out.push(&s[i..i + 1]);
            i += 1;
        } else if is_digit16(s[i]) && s.get(i + 1).is_some_and(|&c| is_digit16(c)) {
            out.push(&s[i..i + 2]);
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Applies what a reader read. Returns false where dayjs's reader fails.
fn read(f: &mut Fields, r: Read, input: &[u16]) -> bool {
    match r {
        Read::Afternoon => f.afternoon = Some(input == utf16("PM").as_slice()),
        Read::AfternoonLower => f.afternoon = Some(input == utf16("pm").as_slice()),
        Read::Quarter => f.month = Some((number16(input) - 1.0) * 3.0 + 1.0),
        Read::Tenths => f.milliseconds = Some(number16(input) * 100.0),
        Read::Hundredths => f.milliseconds = Some(number16(input) * 10.0),
        Read::Milliseconds => f.milliseconds = Some(number16(input)),
        Read::Seconds => f.seconds = Some(number16(input)),
        Read::Minutes => f.minutes = Some(number16(input)),
        Read::Hours => f.hours = Some(number16(input)),
        Read::Day => f.day = Some(number16(input)),
        Read::Week => f.week = Some(number16(input)),
        Read::Month => f.month = Some(number16(input)),
        Read::Year => f.year = Some(number16(input)),
        Read::TwoDigitYear => {
            let n = number16(input);
            f.year = Some(n + if n > 68.0 { 1900.0 } else { 2000.0 });
        }
        Read::Ordinal => {
            // `input.match(/\d+/)`
            let Some(start) = input.iter().position(|&c| is_digit16(c)) else { return false };
            let end = input[start..].iter().position(|&c| !is_digit16(c)).map_or(input.len(), |k| start + k);
            // dayjs keeps the digits as a string here, so even "0" counts as a day having been given.
            f.day = Some(number16(&input[start..end]));
            f.day_given = true;
            for i in 1..=31 {
                if utf16(&ordinal(i as f64)) == input {
                    f.day = Some(i as f64);
                }
            }
        }
        Read::ShortMonthName | Read::MonthName => {
            let index = MONTHS
                .iter()
                .position(|m| {
                    let name = if r == Read::ShortMonthName { &m[..3] } else { m };
                    utf16(name) == input
                })
                .map_or(0, |k| k + 1);
            if index < 1 {
                return false;
            }
            f.month = Some(if index % 12 == 0 { index as f64 } else { (index % 12) as f64 });
        }
        Read::Offset => {
            if input == [b'Z' as u16] {
                f.offset = Some(0.0);
                return true;
            }
            let parts = offset_parts(input);
            let hours = parts.get(1).map_or(f64::NAN, |p| number16(p));
            let mins = parts.get(2).map_or(f64::NAN, |p| number16(p));
            let minutes = hours * 60.0 + if mins == 0.0 || mins.is_nan() { 0.0 } else { mins };
            let plus = parts.first().is_some_and(|p| *p == [b'+' as u16]);
            f.offset = Some(if minutes == 0.0 {
                0.0
            } else if plus {
                -minutes
            } else {
                minutes
            });
        }
    }
    true
}

/// A JavaScript value used as a condition.
fn truthy(v: Option<f64>) -> bool {
    v.is_some_and(|x| x != 0.0 && !x.is_nan())
}

/// `input.replace(match, '')`: the first occurrence of `needle` in `rest` removed.
fn remove_first(rest: &mut Vec<u16>, needle: &[u16]) {
    if needle.is_empty() {
        return;
    }
    if let Some(at) = rest.windows(needle.len()).position(|w| w == needle) {
        rest.drain(at..at + needle.len());
    }
}

fn loose(input: &str, template: &str, steps: Option<&[Step]>, now: &JsDate) -> Option<JsDate> {
    let zone = now.zone();
    if template == "x" || template == "X" {
        let scale = if template == "X" { 1000.0 } else { 1.0 };
        return Some(zone.at(scale * js::number(input)));
    }
    let steps = steps?;
    let mut f = Fields::default();
    let mut rest: Vec<u16> = utf16(input);
    let mut start = 0usize;
    for step in steps {
        match *step {
            Step::Literal(len) => start += len,
            Step::Reader(m, r) => {
                let from = start.min(rest.len());
                let (a, b) = find(m, &rest[from..])?;
                let matched: Vec<u16> = rest[from + a..from + b].to_vec();
                if !read(&mut f, r, &matched) {
                    return None;
                }
                // dayjs drops the first occurrence of the matched text, wherever it is, and does
                // not move `start`.
                remove_first(&mut rest, &matched);
            }
        }
    }
    if let (Some(afternoon), Some(hours)) = (f.afternoon, f.hours) {
        if afternoon {
            if hours < 12.0 {
                f.hours = Some(hours + 12.0);
            }
        } else if hours == 12.0 {
            f.hours = Some(0.0);
        }
    }
    let day = if f.day_given || truthy(f.day) {
        f.day.unwrap_or(f64::NAN)
    } else if !truthy(f.year) && !truthy(f.month) {
        now.date()
    } else {
        1.0
    };
    let year = if truthy(f.year) { f.year.unwrap_or(f64::NAN) } else { now.full_year() };
    let mut month = 0.0;
    if !(truthy(f.year) && !truthy(f.month)) {
        month = match f.month {
            Some(m) if m > 0.0 => m - 1.0,
            _ => now.month(),
        };
    }
    let or0 = |v: Option<f64>| if truthy(v) { v.unwrap_or(0.0) } else { 0.0 };
    let hours = or0(f.hours);
    let minutes = or0(f.minutes);
    let seconds = or0(f.seconds);
    let ms = or0(f.milliseconds);
    if let Some(offset) = f.offset {
        return Some(zone.at(zone.utc(year, month, day, hours, minutes, seconds, ms + offset * 60000.0)));
    }
    // Applying a week number needs a dayjs plugin Mermaid does not load.
    if truthy(f.week) {
        return None;
    }
    Some(zone.local(year, month, day, hours, minutes, seconds, ms))
}

/// Parses the way `dayjs(input, format, true)` does, with the steps `compile` gave for the
/// format. `now` supplies the fields the format leaves out.
pub(crate) fn parse_strict(input: &str, template: &str, steps: Option<&[Step]>, now: &JsDate) -> Option<JsDate> {
    let date = loose(input, template, steps, now)?;
    if date.is_valid() && format(&date, template).as_deref() == Some(input) {
        Some(date)
    } else {
        None
    }
}

/// `/^(\d+(?:\.\d+)?)([Mdhmswy]|ms)$/` on the trimmed text: a duration such as `1.5d`.
pub(crate) fn parse_duration(text: &str) -> Option<(f64, Unit)> {
    let s = js::trim(text);
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    let unit = match &s[i..] {
        "M" => Unit::Month,
        "d" => Unit::Day,
        "h" => Unit::Hour,
        "m" => Unit::Minute,
        "s" => Unit::Second,
        "w" => Unit::Week,
        "y" => Unit::Year,
        "ms" => Unit::Millisecond,
        _ => return None,
    };
    Some((js::parse_float(&s[..i]), unit))
}

#[cfg(test)]
mod tests {
    use super::super::js_date::Zone;
    use super::*;

    fn day(y: f64, m: f64, d: f64) -> JsDate {
        Zone::UTC.local(y, m, d, 0.0, 0.0, 0.0, 0.0)
    }

    fn parse(input: &str, template: &str) -> Option<String> {
        let now = Zone::UTC.at(1705276800000.0);
        let steps = compile(template);
        parse_strict(input, template, steps.as_deref(), &now).map(|d| iso_date(&d))
    }

    #[test]
    fn calendar_arithmetic() {
        // Values from dayjs 1.11 in Node with TZ=UTC.
        let iso = |d: JsDate| iso_date(&d);
        assert_eq!(iso(add(&day(2024.0, 0.0, 31.0), 1.0, Unit::Month)), "2024-02-29");
        assert_eq!(iso(add(&day(2023.0, 0.0, 31.0), 1.0, Unit::Month)), "2023-02-28");
        assert_eq!(iso(add(&day(2024.0, 1.0, 29.0), 1.0, Unit::Year)), "2025-02-28");
        assert_eq!(iso(add(&day(2024.0, 1.0, 29.0), 4.0, Unit::Year)), "2028-02-29");
        assert_eq!(iso(add(&day(2024.0, 11.0, 31.0), 1.0, Unit::Day)), "2025-01-01");
        assert_eq!(iso(add(&day(2024.0, 11.0, 15.0), 1.5, Unit::Month)), "2025-01-15");
        assert_eq!(iso(add(&day(2024.0, 11.0, 31.0), 2.5, Unit::Day)), "2025-01-03");
        assert_eq!(iso(add(&day(2024.0, 2.0, 31.0), -1.0, Unit::Month)), "2024-02-29");
        assert_eq!(iso(add(&day(1900.0, 0.0, 31.0), 1.0, Unit::Month)), "1900-02-28");
        assert_eq!(iso(add(&day(9999.0, 11.0, 31.0), 1.0, Unit::Day)), "10000-01-01");
        // Years 0 to 99: the month length is taken from 1900 to 1999, as Pele's
        // `new Date(year, month + 1, 0)` does.
        let mut y4 = day(2000.0, 0.0, 31.0);
        y4.set_full_year(4.0);
        assert_eq!(iso(add(&y4, 1.0, Unit::Month)), "0004-02-29");
        let mut y0 = day(2000.0, 0.0, 31.0);
        y0.set_full_year(0.0);
        assert_eq!(iso(add(&y0, 1.0, Unit::Month)), "0000-02-28");
        // Negative years: `pad` puts its zeros before the sign.
        let mut neg = day(2000.0, 5.0, 1.0);
        neg.set_full_year(-5.0);
        assert_eq!(iso(neg), "00-5-06-01");
        assert_eq!(format(&neg, "YYYY/YY").as_deref(), Some("00-5/-5"));
    }

    #[test]
    fn formats() {
        let d = Zone::UTC.at(1704153600000.0 + 13.0 * 3_600_000.0 + 5.0 * 60_000.0 + 7_008.0);
        assert_eq!(format(&d, "YYYY-MM-DD HH:mm:ss.SSS").as_deref(), Some("2024-01-02 13:05:07.008"));
        assert_eq!(format(&d, "").as_deref(), Some("2024-01-02T13:05:07+00:00"));
        assert_eq!(format(&d, "Do MMM YY, dddd h a ZZ [at] Q W k").as_deref(), Some("2nd Jan 24, Tuesday 1 pm +0000 at 1 1 13"));
        assert_eq!(format(&d, "w"), None);
        assert_eq!(format(&day(2021.0, 0.0, 1.0), "GGGG-[W]WW").as_deref(), Some("2020-W53"));
        assert_eq!(format(&day(2024.0, 11.0, 30.0), "GGGG-[W]WW").as_deref(), Some("2025-W01"));
    }

    #[test]
    fn strict_parsing() {
        assert_eq!(parse("2024-02-29", "YYYY-MM-DD").as_deref(), Some("2024-02-29"));
        assert_eq!(parse("2023-02-29", "YYYY-MM-DD"), None);
        assert_eq!(parse("0001-01-01", "YYYY-MM-DD"), None);
        assert_eq!(parse("31/12/2024", "DD/MM/YYYY").as_deref(), Some("2024-12-31"));
        assert_eq!(parse("Mar 5 2024", "MMM D YYYY").as_deref(), Some("2024-03-05"));
        assert_eq!(parse("5th March", "Do MMMM").as_deref(), Some("2024-03-05"));
        assert_eq!(parse("2024-01-01T10:00+02:00", "YYYY-MM-DDTHH:mmZ"), None);
        assert_eq!(parse("1704067200", "X").as_deref(), Some("2024-01-01"));
        assert_eq!(parse("12", "LT"), None);
        assert_eq!(parse("2024", "YYYY").as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn durations() {
        assert_eq!(parse_duration(" 3d "), Some((3.0, Unit::Day)));
        assert_eq!(parse_duration("1.5ms"), Some((1.5, Unit::Millisecond)));
        assert_eq!(parse_duration("2M"), Some((2.0, Unit::Month)));
        assert_eq!(parse_duration("1.d"), None);
        assert_eq!(parse_duration("d"), None);
        assert_eq!(parse_duration("3 d"), None);
    }
}
