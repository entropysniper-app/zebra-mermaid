//! Time axis support. Ported from Pele `src/diagrams/gantt/axis.ts`, which follows d3-time and
//! d3-time-format (English locale, local time) as Mermaid uses them: the `axisFormat`
//! specifiers, tick intervals with d3's `every` rule, and its choice of automatic ticks.

use super::dates::{MONTHS, WEEKDAYS};
use super::js_date::{JsDate, Zone};
use crate::util::{js, math};

const SECOND: f64 = 1000.0;
const MINUTE: f64 = 60.0 * SECOND;
const HOUR: f64 = 60.0 * MINUTE;
const DAY: f64 = 24.0 * HOUR;
const WEEK: f64 = 7.0 * DAY;
const MONTH: f64 = 30.0 * DAY;
const YEAR: f64 = 365.0 * DAY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TickUnit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// How an interval is stepped: the plain unit, or d3's `every(step)` variants.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Every {
    Base,
    /// Milliseconds: multiples of the step since the epoch.
    Millis(f64),
    /// Years: multiples of the step.
    Years(f64),
    /// Weeks: every step-th week counted from the week of the epoch.
    Weeks(f64, JsDate),
    /// The other units: where the unit's field is a multiple of the step.
    Field(f64),
}

/// A d3 time interval.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Calendar {
    unit: TickUnit,
    /// The first day of the week, for weeks (0 for Sunday).
    week_start: f64,
    every: Every,
}

fn start_of_day(d: &mut JsDate) {
    d.set_hours(0.0, 0.0, 0.0, 0.0);
}

impl Calendar {
    fn base(unit: TickUnit, week_start: f64) -> Calendar {
        Calendar { unit, week_start, every: Every::Base }
    }

    fn week(week_start: f64) -> Calendar {
        Calendar::base(TickUnit::Week, week_start)
    }

    /// The base interval's `floor`.
    fn base_floor(&self, d: &mut JsDate) {
        match self.unit {
            TickUnit::Millisecond => {}
            TickUnit::Second => d.set_time(d.time() - d.milliseconds()),
            TickUnit::Minute => d.set_time(d.time() - d.milliseconds() - d.seconds() * SECOND),
            TickUnit::Hour => d.set_time(d.time() - d.milliseconds() - d.seconds() * SECOND - d.minutes() * MINUTE),
            TickUnit::Day => start_of_day(d),
            TickUnit::Week => {
                d.set_date(d.date() - (d.day() + 7.0 - self.week_start) % 7.0);
                start_of_day(d);
            }
            TickUnit::Month => {
                d.set_date(1.0);
                start_of_day(d);
            }
            TickUnit::Year => {
                d.set_month(0.0, Some(1.0));
                start_of_day(d);
            }
        }
    }

    /// The base interval's `offset`.
    fn base_offset(&self, d: &mut JsDate, step: f64) {
        match self.unit {
            TickUnit::Millisecond => d.set_time(d.time() + step),
            TickUnit::Second => d.set_time(d.time() + step * SECOND),
            TickUnit::Minute => d.set_time(d.time() + step * MINUTE),
            TickUnit::Hour => d.set_time(d.time() + step * HOUR),
            TickUnit::Day => d.set_date(d.date() + step),
            TickUnit::Week => d.set_date(d.date() + step * 7.0),
            TickUnit::Month => d.set_month(d.month() + step, None),
            TickUnit::Year => d.set_full_year(d.full_year() + step),
        }
    }

    /// The value d3 tests against a step: a tick falls where it is a multiple of the step.
    fn field(&self, d: &JsDate) -> f64 {
        match self.unit {
            TickUnit::Second => d.utc_seconds(),
            TickUnit::Minute => d.minutes(),
            TickUnit::Hour => d.hours(),
            TickUnit::Day => d.date() - 1.0,
            TickUnit::Month => d.month(),
            _ => f64::NAN,
        }
    }

    /// How many values `field` cycles through, which bounds how far apart filtered ticks can be.
    fn cycle(&self) -> f64 {
        match self.unit {
            TickUnit::Second | TickUnit::Minute => 60.0,
            TickUnit::Hour => 24.0,
            TickUnit::Day => 31.0,
            TickUnit::Month => 12.0,
            _ => f64::INFINITY,
        }
    }

    fn ms(&self) -> f64 {
        match self.unit {
            TickUnit::Millisecond => 1.0,
            TickUnit::Second => SECOND,
            TickUnit::Minute => MINUTE,
            TickUnit::Hour => HOUR,
            TickUnit::Day => DAY,
            TickUnit::Week => WEEK,
            TickUnit::Month => MONTH,
            TickUnit::Year => YEAR,
        }
    }

    fn floor(&self, d: &mut JsDate) {
        match self.every {
            Every::Base => self.base_floor(d),
            Every::Millis(step) => d.set_time((d.time() / step).floor() * step),
            Every::Years(step) => {
                d.set_full_year((d.full_year() / step).floor() * step);
                self.base_floor(d);
            }
            Every::Weeks(step, epoch) => {
                self.base_floor(d);
                self.base_offset(d, -modulo(span(&epoch, d, WEEK), step));
            }
            Every::Field(step) => {
                // Date overflow makes the field NaN, which can never match the interval.
                self.base_floor(d);
                while d.time().is_finite() && self.field(d) % step != 0.0 {
                    d.set_time(d.time() - 1.0);
                    self.base_floor(d);
                }
            }
        }
    }

    fn offset(&self, d: &mut JsDate, n: f64) {
        match self.every {
            Every::Base => self.base_offset(d, n),
            Every::Millis(step) => d.set_time(d.time() + n * step),
            Every::Years(step) => d.set_full_year(d.full_year() + n * step),
            Every::Weeks(step, _) => self.base_offset(d, n * step),
            Every::Field(step) => {
                let mut i = 0.0;
                while i < n && d.time().is_finite() {
                    loop {
                        self.base_offset(d, 1.0);
                        if !(d.time().is_finite() && self.field(d) % step != 0.0) {
                            break;
                        }
                    }
                    i += 1.0;
                }
            }
        }
    }
}

/// Whole units between two floored dates, discounting a change of UTC offset between them.
fn span(start: &JsDate, end: &JsDate, unit: f64) -> f64 {
    ((end.time() - start.time() - (end.timezone_offset() - start.timezone_offset()) * MINUTE) / unit).floor()
}

fn modulo(a: f64, b: f64) -> f64 {
    ((a % b) + b) % b
}

/// The interval d3's `unit.every(step)` gives.
fn every(unit: TickUnit, step: f64, week_start: f64, zone: Zone) -> Calendar {
    let base = Calendar::base(unit, week_start);
    if step <= 1.0 {
        return base;
    }
    let every = match unit {
        TickUnit::Millisecond => Every::Millis(step),
        TickUnit::Year => Every::Years(step),
        TickUnit::Week => {
            let mut epoch = zone.at(0.0);
            base.floor(&mut epoch);
            Every::Weeks(step, epoch)
        }
        _ => Every::Field(step),
    };
    Calendar { every, ..base }
}

/// Ticks from start to stop inclusive, or `None` when there would be more than `limit`.
fn range(calendar: &Calendar, start: f64, stop: f64, limit: f64, zone: Zone) -> Option<Vec<f64>> {
    let mut d = zone.at(start - 1.0);
    calendar.floor(&mut d);
    calendar.offset(&mut d, 1.0);
    calendar.floor(&mut d);
    let mut out = Vec::new();
    let mut previous = f64::NEG_INFINITY;
    while d.time() <= stop && d.time() > previous {
        if out.len() as f64 >= limit {
            return None;
        }
        previous = d.time();
        out.push(previous);
        calendar.offset(&mut d, 1.0);
        calendar.floor(&mut d);
    }
    Some(out)
}

fn tick_step(start: f64, stop: f64, count: f64) -> f64 {
    let step = (stop - start) / js::max(0.0, count);
    let power = math::log10(step).floor();
    let error = step / math::pow(10.0, power);
    let factor = if error >= math::sqrt(50.0) {
        10.0
    } else if error >= math::sqrt(10.0) {
        5.0
    } else if error >= std::f64::consts::SQRT_2 {
        2.0
    } else {
        1.0
    };
    math::pow(10.0, power) * factor
}

const AUTO: [(TickUnit, f64, f64); 18] = [
    (TickUnit::Second, 1.0, SECOND),
    (TickUnit::Second, 5.0, 5.0 * SECOND),
    (TickUnit::Second, 15.0, 15.0 * SECOND),
    (TickUnit::Second, 30.0, 30.0 * SECOND),
    (TickUnit::Minute, 1.0, MINUTE),
    (TickUnit::Minute, 5.0, 5.0 * MINUTE),
    (TickUnit::Minute, 15.0, 15.0 * MINUTE),
    (TickUnit::Minute, 30.0, 30.0 * MINUTE),
    (TickUnit::Hour, 1.0, HOUR),
    (TickUnit::Hour, 3.0, 3.0 * HOUR),
    (TickUnit::Hour, 6.0, 6.0 * HOUR),
    (TickUnit::Hour, 12.0, 12.0 * HOUR),
    (TickUnit::Day, 1.0, DAY),
    (TickUnit::Day, 2.0, 2.0 * DAY),
    (TickUnit::Week, 1.0, WEEK),
    (TickUnit::Month, 1.0, MONTH),
    (TickUnit::Month, 3.0, 3.0 * MONTH),
    (TickUnit::Year, 1.0, YEAR),
];

/// About `count` ticks at round times, chosen the way d3's time scale chooses them.
pub(crate) fn auto_ticks(start: f64, stop: f64, count: f64, zone: Zone) -> Vec<f64> {
    let target = (stop - start) / count;
    let mut i = 0;
    while i < AUTO.len() && AUTO[i].2 <= target {
        i += 1;
    }
    let calendar = if i == AUTO.len() {
        every(TickUnit::Year, js::max(1.0, tick_step(start / YEAR, stop / YEAR, count)), 0.0, zone)
    } else if i == 0 {
        every(TickUnit::Millisecond, js::max(tick_step(start, stop, count), 1.0), 0.0, zone)
    } else {
        let (unit, step, _) = AUTO[if target / AUTO[i - 1].2 < AUTO[i].2 / target { i - 1 } else { i }];
        every(unit, step, 0.0, zone)
    };
    range(&calendar, start, stop, 4.0 * count + 8.0, zone).unwrap_or_default()
}

const WEEK_STARTS: [&str; 7] = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];

/// `/^([1-9]\d*)(millisecond|second|minute|hour|day|week|month)$/`
fn parse_interval(interval: &str) -> Option<(f64, TickUnit)> {
    let b = interval.as_bytes();
    if b.first().is_none_or(|c| !(b'1'..=b'9').contains(c)) {
        return None;
    }
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    let unit = match &interval[digits..] {
        "millisecond" => TickUnit::Millisecond,
        "second" => TickUnit::Second,
        "minute" => TickUnit::Minute,
        "hour" => TickUnit::Hour,
        "day" => TickUnit::Day,
        "week" => TickUnit::Week,
        "month" => TickUnit::Month,
        _ => return None,
    };
    Some((js::parse_int(&interval[..digits], 10), unit))
}

/// Ticks for a `tickInterval` such as `1week`, or `None` when it is not one or asks for more
/// than `limit`.
pub(crate) fn interval_ticks(interval: &str, weekday: &str, start: f64, stop: f64, limit: f64, zone: Zone) -> Option<Vec<f64>> {
    let (step, unit) = parse_interval(interval)?;
    let week_start = WEEK_STARTS.iter().position(|d| *d == weekday).map_or(0.0, |k| k as f64);
    let calendar = every(unit, step, week_start, zone);
    if (stop - start) / (js::min(step, calendar.cycle()) * calendar.ms()) > limit {
        return None;
    }
    range(&calendar, start, stop, limit, zone)
}

/// `pad(value, fill, width)`: d3's padding, which keeps a minus sign first.
fn pad(value: f64, fill: &str, width: usize) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let digits = js::to_string(if sign.is_empty() { value } else { -value });
    if digits.len() < width {
        format!("{}{}{}", sign, fill.repeat(width - digits.len()), digits)
    } else {
        format!("{}{}", sign, digits)
    }
}

fn floored(date: &JsDate, calendar: &Calendar) -> JsDate {
    let mut d = *date;
    calendar.floor(&mut d);
    d
}

fn year_calendar() -> Calendar {
    Calendar::base(TickUnit::Year, 0.0)
}

fn day_calendar() -> Calendar {
    Calendar::base(TickUnit::Day, 0.0)
}

/// Weeks since the last `week_start` day before the year began, which is d3's %U and %W.
fn week_of_year(date: &JsDate, week_start: f64) -> f64 {
    let week = Calendar::week(week_start);
    let year_start = floored(date, &year_calendar());
    let before = date.zone().at(year_start.time() - 1.0);
    span(&floored(&before, &week), &floored(date, &week), WEEK)
}

/// The Thursday that decides which ISO week and ISO year a date belongs to.
fn iso_thursday(date: &JsDate) -> JsDate {
    let thursday = Calendar::week(4.0);
    let day = date.day();
    let late = day >= 4.0 || day == 0.0;
    let from = if late { *date } else { date.zone().at(date.time() - 1.0) };
    let mut d = floored(&from, &thursday);
    if !late {
        thursday.offset(&mut d, 1.0);
        thursday.floor(&mut d);
    }
    d
}

/// Formats a date with a d3 time format specifier such as `%Y-%m-%d`.
pub(crate) fn time_format(specifier: &str, date: &JsDate) -> String {
    let mut out = String::new();
    let mut chars = specifier.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut c = chars.next();
        let fill = match c {
            Some('-') => Some(""),
            Some('_') => Some(" "),
            Some('0') => Some("0"),
            _ => None,
        };
        let fill = match fill {
            Some(f) => {
                c = chars.next();
                f
            }
            None if c == Some('e') => " ",
            None => "0",
        };
        out.push_str(&directive(c, date, fill));
    }
    out
}

fn directive(c: Option<char>, d: &JsDate, p: &str) -> String {
    let Some(c) = c else { return String::new() };
    let weekday = d.day() as usize;
    let month = d.month() as usize;
    match c {
        'a' => WEEKDAYS[weekday][..3].to_string(),
        'A' => WEEKDAYS[weekday].to_string(),
        'b' => MONTHS[month][..3].to_string(),
        'B' => MONTHS[month].to_string(),
        'c' => time_format("%x, %X", d),
        'd' | 'e' => pad(d.date(), p, 2),
        'f' => pad(d.milliseconds(), p, 3) + "000",
        'g' => pad(iso_thursday(d).full_year() % 100.0, p, 2),
        'G' => pad(iso_thursday(d).full_year() % 10000.0, p, 4),
        'H' => pad(d.hours(), p, 2),
        'I' => pad(twelve(d.hours()), p, 2),
        'j' => pad(1.0 + span(&floored(d, &year_calendar()), &floored(d, &day_calendar()), DAY), p, 3),
        'L' => pad(d.milliseconds(), p, 3),
        'm' => pad(d.month() + 1.0, p, 2),
        'M' => pad(d.minutes(), p, 2),
        'p' => (if d.hours() >= 12.0 { "PM" } else { "AM" }).to_string(),
        'q' => js::to_string(1.0 + (d.month() / 3.0).floor()),
        'Q' => js::to_string(d.time()),
        's' => js::to_string((d.time() / 1000.0).floor()),
        'S' => pad(d.seconds(), p, 2),
        'u' => js::to_string(if d.day() == 0.0 { 7.0 } else { d.day() }),
        'U' => pad(week_of_year(d, 0.0), p, 2),
        'V' => {
            let thursday = iso_thursday(d);
            let year_start = floored(&thursday, &year_calendar());
            let first = floored(&year_start, &Calendar::week(4.0));
            pad(span(&first, &thursday, WEEK) + if year_start.day() == 4.0 { 1.0 } else { 0.0 }, p, 2)
        }
        'w' => js::to_string(d.day()),
        'W' => pad(week_of_year(d, 1.0), p, 2),
        'x' => time_format("%-m/%-d/%Y", d),
        'X' => time_format("%-I:%M:%S %p", d),
        'y' => pad(d.full_year() % 100.0, p, 2),
        'Y' => pad(d.full_year() % 10000.0, p, 4),
        'Z' => {
            let z = d.timezone_offset();
            let abs = z.abs();
            format!("{}{}{}", if z > 0.0 { '-' } else { '+' }, pad((abs / 60.0).floor(), "0", 2), pad(abs % 60.0, "0", 2))
        }
        '%' => "%".to_string(),
        c => c.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        // 2024-01-02T13:05:07.008Z, from d3-time-format in Node with TZ=UTC.
        let d = Zone::UTC.at(1704200707008.0);
        assert_eq!(time_format("%Y-%m-%d", &d), "2024-01-02");
        assert_eq!(time_format("%a %A %b %B %e %-d %_m %j %U %W %V %G %u %w", &d), "Tue Tuesday Jan January  2 2  1 002 00 01 01 2024 2 2");
        assert_eq!(
            time_format("%c|%I %p|%f|%L|%Q|%s|%Z|%%|%q|%y|%", &d),
            "1/2/2024, 1:05:07 PM|01 PM|008000|008|1704200707008|1704200707|+0000|%|1|24|"
        );
    }

    #[test]
    fn ticks() {
        let start = 1704067200000.0; // 2024-01-01
        let stop = start + 30.0 * DAY;
        let weeks = interval_ticks("1week", "monday", start, stop, 100.0, Zone::UTC).unwrap();
        assert_eq!(weeks, vec![1704067200000.0, 1704672000000.0, 1705276800000.0, 1705881600000.0, 1706486400000.0]);
        assert_eq!(interval_ticks("0day", "sunday", start, stop, 100.0, Zone::UTC), None);
        assert_eq!(interval_ticks("1day", "sunday", start, stop, 10.0, Zone::UTC), None);
        let auto = auto_ticks(start, stop, 8.0, Zone::UTC);
        assert_eq!(auto, vec![1704585600000.0, 1705190400000.0, 1705795200000.0, 1706400000000.0]);
    }
}
