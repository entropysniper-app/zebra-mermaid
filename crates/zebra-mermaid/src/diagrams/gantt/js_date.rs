//! The parts of JavaScript's `Date` that Pele's gantt code uses, as V8 implements them.
//!
//! Pele schedules tasks and draws its time axis with `Date` objects in the host's local time
//! zone. Here local time is a fixed offset from UTC (`RenderOptions::timezone_offset_minutes`,
//! 0 by default, which is how the fixtures were made: Node with `TZ=UTC`). Daylight saving
//! time and other changes of a real zone's offset are not modeled: every instant has the same
//! offset, as in the `Etc/GMT±N` zones.
//!
//! The arithmetic follows V8 (`builtins-date.cc`, `date.cc`): `MakeDay` with V8's year and
//! month bounds, `MakeTime`, `MakeDate`, the local-time bound checked before converting to
//! UTC, and `TimeClip`. A date whose time value is NaN is an invalid date, and every getter of
//! it gives NaN, as in JavaScript.

/// A zone at a fixed offset: local time is UTC plus this many milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Zone {
    offset: f64,
}

pub(crate) const MS_PER_SECOND: f64 = 1000.0;
pub(crate) const MS_PER_MINUTE: f64 = 60_000.0;
pub(crate) const MS_PER_HOUR: f64 = 3_600_000.0;
pub(crate) const MS_PER_DAY: f64 = 86_400_000.0;

/// `DateCache::kMaxTimeInMs`: the largest time value a date holds.
const MAX_TIME: f64 = 8.64e15;
/// `DateCache::kMaxTimeBeforeUTCInMs`: the bound on a local time before its conversion to UTC.
const MAX_TIME_BEFORE_UTC: f64 = MAX_TIME + 30.0 * MS_PER_DAY;

impl Zone {
    pub(crate) const UTC: Zone = Zone { offset: 0.0 };

    /// A zone `minutes` ahead of UTC (negative: behind it).
    pub(crate) fn from_minutes(minutes: i32) -> Zone {
        Zone { offset: minutes as f64 * MS_PER_MINUTE }
    }

    /// `new Date(value)` for a number: the time value, clipped.
    pub(crate) fn at(self, value: f64) -> JsDate {
        JsDate { t: time_clip(value), zone: self }
    }

    /// `new Date(year, month, day, hours, minutes, seconds, ms)`, in local time. As in
    /// JavaScript, a year from 0 to 99 means 1900 to 1999.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn local(self, year: f64, month: f64, day: f64, hours: f64, minutes: f64, seconds: f64, ms: f64) -> JsDate {
        let year = full_year(year);
        let value = make_date(make_day(year, month, day), make_time(hours, minutes, seconds, ms));
        let mut d = JsDate { t: f64::NAN, zone: self };
        d.set_local(value);
        d
    }

    /// `Date.UTC(year, month, day, hours, minutes, seconds, ms)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn utc(self, year: f64, month: f64, day: f64, hours: f64, minutes: f64, seconds: f64, ms: f64) -> f64 {
        let year = full_year(year);
        time_clip(make_date(make_day(year, month, day), make_time(hours, minutes, seconds, ms)))
    }

    fn to_local(self, t: f64) -> f64 {
        t + self.offset
    }

    /// The UTC time value of a local time value (`DateCache::ToUTC`).
    pub(crate) fn at_local(self, local: f64) -> f64 {
        local - self.offset
    }
}

/// A JavaScript `Date`: a time value in milliseconds since the epoch (NaN when invalid) and
/// the zone its local-time methods use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct JsDate {
    t: f64,
    zone: Zone,
}

/// A local time taken apart.
struct Parts {
    year: f64,
    /// 0 to 11.
    month: f64,
    /// 1 to 31.
    date: f64,
    /// Days since the epoch.
    days: f64,
    /// Milliseconds since midnight.
    in_day: f64,
}

impl JsDate {
    /// `+date`, `date.getTime()`.
    #[inline]
    pub(crate) fn time(&self) -> f64 {
        self.t
    }

    #[inline]
    pub(crate) fn zone(&self) -> Zone {
        self.zone
    }

    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        !self.t.is_nan()
    }

    fn local_time(&self) -> f64 {
        self.zone.to_local(self.t)
    }

    fn parts(&self) -> Option<Parts> {
        if self.t.is_nan() {
            return None;
        }
        let local = self.local_time();
        let days = (local / MS_PER_DAY).floor();
        let in_day = local - days * MS_PER_DAY;
        let (year, month, date) = civil_from_days(days as i64);
        Some(Parts { year: year as f64, month: (month - 1) as f64, date: date as f64, days, in_day })
    }

    fn in_day(&self) -> f64 {
        if self.t.is_nan() {
            return f64::NAN;
        }
        let local = self.local_time();
        local - (local / MS_PER_DAY).floor() * MS_PER_DAY
    }

    pub(crate) fn full_year(&self) -> f64 {
        self.parts().map_or(f64::NAN, |p| p.year)
    }

    pub(crate) fn month(&self) -> f64 {
        self.parts().map_or(f64::NAN, |p| p.month)
    }

    /// `getDate()`: the day of the month.
    pub(crate) fn date(&self) -> f64 {
        self.parts().map_or(f64::NAN, |p| p.date)
    }

    /// `getDay()`: the day of the week, 0 for Sunday.
    pub(crate) fn day(&self) -> f64 {
        if self.t.is_nan() {
            return f64::NAN;
        }
        let days = (self.local_time() / MS_PER_DAY).floor();
        (days + 4.0).rem_euclid(7.0)
    }

    pub(crate) fn hours(&self) -> f64 {
        (self.in_day() / MS_PER_HOUR).floor()
    }

    pub(crate) fn minutes(&self) -> f64 {
        ((self.in_day() / MS_PER_MINUTE).floor()) % 60.0
    }

    pub(crate) fn seconds(&self) -> f64 {
        ((self.in_day() / MS_PER_SECOND).floor()) % 60.0
    }

    pub(crate) fn milliseconds(&self) -> f64 {
        self.in_day() % 1000.0
    }

    /// `getUTCSeconds()`.
    pub(crate) fn utc_seconds(&self) -> f64 {
        if self.t.is_nan() {
            return f64::NAN;
        }
        let in_day = self.t - (self.t / MS_PER_DAY).floor() * MS_PER_DAY;
        ((in_day / MS_PER_SECOND).floor()) % 60.0
    }

    /// `getTimezoneOffset()`: minutes from local time to UTC.
    pub(crate) fn timezone_offset(&self) -> f64 {
        if self.t.is_nan() {
            return f64::NAN;
        }
        // V8 computes `(time - local) / msPerMinute` in integers, so this is never -0.
        let minutes = -self.zone.offset / MS_PER_MINUTE;
        minutes + 0.0
    }

    /// `SetLocalDateValue`: a local time value checked, converted to UTC and clipped.
    fn set_local(&mut self, value: f64) {
        self.t = if (-MAX_TIME_BEFORE_UTC..=MAX_TIME_BEFORE_UTC).contains(&value) {
            time_clip(value.trunc() - self.zone.offset)
        } else {
            f64::NAN
        };
    }

    /// `setTime(value)`.
    pub(crate) fn set_time(&mut self, value: f64) {
        self.t = time_clip(value);
    }

    /// `setDate(day)`.
    pub(crate) fn set_date(&mut self, day: f64) {
        let value = match self.parts() {
            Some(p) => make_date(make_day(p.year, p.month, day), p.in_day),
            None => f64::NAN,
        };
        self.set_local(value);
    }

    /// `setMonth(month)`, or `setMonth(month, day)` when `day` is given.
    pub(crate) fn set_month(&mut self, month: f64, day: Option<f64>) {
        let value = match self.parts() {
            Some(p) => make_date(make_day(p.year, month, day.unwrap_or(p.date)), p.in_day),
            None => f64::NAN,
        };
        self.set_local(value);
    }

    /// `setFullYear(year)`. An invalid date counts as local midnight of 1 January 1970.
    pub(crate) fn set_full_year(&mut self, year: f64) {
        let (month, day, in_day) = match self.parts() {
            Some(p) => (p.month, p.date, p.in_day),
            None => (0.0, 1.0, 0.0),
        };
        self.set_local(make_date(make_day(year, month, day), in_day));
    }

    /// `setHours(hours, minutes, seconds, ms)`.
    pub(crate) fn set_hours(&mut self, hours: f64, minutes: f64, seconds: f64, ms: f64) {
        let value = match self.parts() {
            Some(p) => make_date(p.days, make_time(hours, minutes, seconds, ms)),
            None => f64::NAN,
        };
        self.set_local(value);
    }
}

/// `TimeClip`: NaN beyond ±8.64e15 ms, otherwise the integer part (never -0).
pub(crate) fn time_clip(t: f64) -> f64 {
    if !t.is_finite() || t.abs() > MAX_TIME {
        return f64::NAN;
    }
    t.trunc() + 0.0
}

/// `DoubleToInteger`: truncation, with NaN as 0.
fn to_integer(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x.trunc()
    }
}

/// The Date constructor and `Date.UTC` read a year from 0 to 99 as 1900 to 1999.
fn full_year(year: f64) -> f64 {
    if year.is_nan() {
        return year;
    }
    let y = to_integer(year);
    if (0.0..=99.0).contains(&y) {
        1900.0 + y
    } else {
        year
    }
}

/// V8's `MakeDay`: the day number of a year, month (any integer, carried into the year) and day
/// of the month. Years beyond ±1,000,000 and months beyond ±10,000,000 give NaN, as in V8;
/// every such date is past the range a date can hold anyway.
pub(crate) fn make_day(year: f64, month: f64, date: f64) -> f64 {
    if !((-1_000_000.0..=1_000_000.0).contains(&year) && (-10_000_000.0..=10_000_000.0).contains(&month) && date.is_finite()) {
        return f64::NAN;
    }
    let mut y = year.trunc() as i64;
    let mut m = month.trunc() as i64;
    y += m / 12;
    m %= 12;
    if m < 0 {
        m += 12;
        y -= 1;
    }
    let first = days_from_civil(y, m + 1, 1);
    (first - 1) as f64 + to_integer(date)
}

/// `MakeTime`.
pub(crate) fn make_time(hours: f64, minutes: f64, seconds: f64, ms: f64) -> f64 {
    if !(hours.is_finite() && minutes.is_finite() && seconds.is_finite() && ms.is_finite()) {
        return f64::NAN;
    }
    to_integer(hours) * MS_PER_HOUR + to_integer(minutes) * MS_PER_MINUTE + to_integer(seconds) * MS_PER_SECOND + to_integer(ms)
}

/// `MakeDate`.
pub(crate) fn make_date(day: f64, time: f64) -> f64 {
    if !(day.is_finite() && time.is_finite()) {
        return f64::NAN;
    }
    if time == 0.0 && day != 0.0 {
        return day * MS_PER_DAY;
    }
    time + day * MS_PER_DAY
}

/// Days from 1970-01-01 to a date of the proleptic Gregorian calendar (month 1 to 12).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The year, month (1 to 12) and day of a day number.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The current time, as `Date.now()` gives it.
pub(crate) fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as f64,
        Err(e) => -(e.duration().as_millis() as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(y: f64, m: f64, d: f64) -> JsDate {
        Zone::UTC.local(y, m, d, 0.0, 0.0, 0.0, 0.0)
    }

    #[test]
    fn calendar_round_trips() {
        for z in [-800_000_000i64, -719_468, -1, 0, 1, 59, 365, 11_016, 2_932_896, 100_000_000] {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(-719_528), (0, 1, 1));
    }

    #[test]
    fn constructor_and_getters() {
        // new Date(2024, 1, 29) in UTC, values from Node with TZ=UTC.
        let d = utc(2024.0, 1.0, 29.0);
        assert_eq!(d.time(), 1709164800000.0);
        assert_eq!((d.full_year(), d.month(), d.date(), d.day()), (2024.0, 1.0, 29.0, 4.0));
        // Month overflow carries into the year; day 0 is the last day of the previous month.
        assert_eq!(utc(2023.0, 12.0, 1.0).time(), 1704067200000.0);
        assert_eq!(utc(2024.0, 2.0, 0.0).date(), 29.0);
        assert_eq!(utc(2023.0, 2.0, 0.0).date(), 28.0);
        assert_eq!(utc(2100.0, 2.0, 0.0).date(), 28.0);
        assert_eq!(utc(2000.0, 2.0, 0.0).date(), 29.0);
        // Years 0 to 99 are read as 1900 to 1999.
        assert_eq!(utc(5.0, 0.0, 1.0).full_year(), 1905.0);
        assert_eq!(utc(-5.0, 0.0, 1.0).time(), -62324985600000.0);
        assert_eq!(utc(100.0, 0.0, 1.0).time(), -59011459200000.0);
        // The range of a date is ±8.64e15 ms.
        assert_eq!(utc(275760.0, 8.0, 13.0).time(), 8.64e15);
        assert!(!utc(275760.0, 8.0, 14.0).is_valid());
        assert_eq!(utc(-271821.0, 3.0, 20.0).time(), -8.64e15);
        assert!(!utc(-271821.0, 3.0, 19.0).is_valid());
        let invalid = Zone::UTC.at(f64::NAN);
        assert!(invalid.full_year().is_nan() && invalid.day().is_nan() && invalid.timezone_offset().is_nan());
    }

    #[test]
    fn setters() {
        // Jan 31 + 1 month by setMonth overflows into March, as in JavaScript.
        let mut d = utc(2023.0, 0.0, 31.0);
        d.set_month(1.0, None);
        assert_eq!((d.month(), d.date()), (2.0, 3.0));
        let mut d = utc(2024.0, 11.0, 31.0);
        d.set_date(32.0);
        assert_eq!((d.full_year(), d.month(), d.date()), (2025.0, 0.0, 1.0));
        let mut d = utc(2024.0, 1.0, 29.0);
        d.set_full_year(2025.0);
        assert_eq!((d.month(), d.date()), (2.0, 1.0));
        let mut d = Zone::UTC.at(f64::NAN);
        d.set_full_year(2020.0);
        assert_eq!(d.time(), 1577836800000.0);
        let mut d = Zone::UTC.at(1700000000123.0);
        d.set_hours(23.0, 59.0, 59.0, 999.0);
        assert_eq!(d.time(), 1700006399999.0);
        let mut d = Zone::UTC.at(f64::NAN);
        d.set_date(1.0);
        assert!(!d.is_valid());
    }

    #[test]
    fn fixed_offsets() {
        // TZ=Etc/GMT-5 (five hours ahead of UTC): new Date(2024, 0, 1) is 2023-12-31T19:00Z.
        let east = Zone::from_minutes(300);
        let d = east.local(2024.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(d.time(), 1704049200000.0);
        assert_eq!((d.date(), d.hours(), d.timezone_offset()), (1.0, 0.0, -300.0));
        // TZ=Etc/GMT+8.
        let west = Zone::from_minutes(-480);
        let d = west.at(1704067200000.0);
        assert_eq!((d.full_year(), d.month(), d.date(), d.hours(), d.day()), (2023.0, 11.0, 31.0, 16.0, 0.0));
        assert_eq!(d.timezone_offset(), 480.0);
    }
}
