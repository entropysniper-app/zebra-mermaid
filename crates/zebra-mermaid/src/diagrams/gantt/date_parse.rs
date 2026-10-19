//! `new Date(string)`: V8's date parser (`src/date/dateparser*.{h,cc}`), which Pele falls back
//! on when a task's start is not in the chart's date format.
//!
//! V8 first reads the ES5 date-time format (`[+-yy]yyyy[-MM[-DD]][THH:mm[:ss[.sss]][Z|±hh:mm]]`)
//! and, where that does not account for the whole text, continues with its legacy parser:
//! numbers become day or time fields, month names and a few zone names are read from their
//! first three letters, words before the first number are skipped, parenthesized text is
//! ignored, and so on. The text is read in UTF-16 code units, and a NUL code unit ends it.

use super::js_date::{make_date, make_day, make_time, time_clip, Zone, MS_PER_DAY};

/// `kNone`.
const NONE: i32 = i32::MAX;
/// `kMaxSignificantDigits`.
const MAX_SIGNIFICANT_DIGITS: usize = 9;

/// `DateParser::KeywordType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keyword {
    Invalid,
    MonthName,
    TimeZoneName,
    TimeSeparator,
    AmPm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Invalid,
    Unknown,
    Number { value: i32, length: usize },
    Symbol(u8),
    WhiteSpace,
    EndOfInput,
    Keyword { kind: Keyword, value: i32, length: usize },
}

impl Token {
    fn is_number(self) -> bool {
        matches!(self, Token::Number { .. })
    }

    fn number(self) -> i32 {
        match self {
            Token::Number { value, .. } => value,
            _ => 0,
        }
    }

    fn is_fixed_length_number(self, n: usize) -> bool {
        matches!(self, Token::Number { length, .. } if length == n)
    }

    fn is_symbol(self, c: u8) -> bool {
        self == Token::Symbol(c)
    }

    fn is_ascii_sign(self) -> bool {
        matches!(self, Token::Symbol(b'+') | Token::Symbol(b'-'))
    }

    /// 1 for `+`, -1 for `-`.
    fn ascii_sign(self) -> i32 {
        match self {
            Token::Symbol(c) => 44 - c as i32,
            _ => 0,
        }
    }

    fn is_keyword_type(self, k: Keyword) -> bool {
        matches!(self, Token::Keyword { kind, .. } if kind == k)
    }

    fn is_keyword_z(self) -> bool {
        matches!(self, Token::Keyword { kind: Keyword::TimeZoneName, value: 0, length: 1 })
    }
}

/// `KeywordTable::array`: a three-letter prefix, its type and value.
const KEYWORDS: [([u8; 3], Keyword, i32); 27] = [
    (*b"jan", Keyword::MonthName, 1),
    (*b"feb", Keyword::MonthName, 2),
    (*b"mar", Keyword::MonthName, 3),
    (*b"apr", Keyword::MonthName, 4),
    (*b"may", Keyword::MonthName, 5),
    (*b"jun", Keyword::MonthName, 6),
    (*b"jul", Keyword::MonthName, 7),
    (*b"aug", Keyword::MonthName, 8),
    (*b"sep", Keyword::MonthName, 9),
    (*b"oct", Keyword::MonthName, 10),
    (*b"nov", Keyword::MonthName, 11),
    (*b"dec", Keyword::MonthName, 12),
    (*b"am\0", Keyword::AmPm, 0),
    (*b"pm\0", Keyword::AmPm, 12),
    (*b"ut\0", Keyword::TimeZoneName, 0),
    (*b"utc", Keyword::TimeZoneName, 0),
    (*b"z\0\0", Keyword::TimeZoneName, 0),
    (*b"gmt", Keyword::TimeZoneName, 0),
    (*b"cdt", Keyword::TimeZoneName, -5),
    (*b"cst", Keyword::TimeZoneName, -6),
    (*b"edt", Keyword::TimeZoneName, -4),
    (*b"est", Keyword::TimeZoneName, -5),
    (*b"mdt", Keyword::TimeZoneName, -6),
    (*b"mst", Keyword::TimeZoneName, -7),
    (*b"pdt", Keyword::TimeZoneName, -7),
    (*b"pst", Keyword::TimeZoneName, -8),
    (*b"t\0\0", Keyword::TimeSeparator, 0),
];

/// `KeywordTable::Lookup`: a word longer than its keyword only matches a month name.
fn lookup(prefix: [u32; 3], length: usize) -> (Keyword, i32) {
    for (word, kind, value) in KEYWORDS {
        if (0..3).all(|j| prefix[j] == word[j] as u32) && (length <= 3 || kind == Keyword::MonthName) {
            return (kind, value);
        }
    }
    (Keyword::Invalid, 0)
}

/// `IsWhiteSpace`: ECMAScript white space, without line terminators.
fn is_white_space(c: u32) -> bool {
    matches!(c, 0x09 | 0x0b | 0x0c | 0x20 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x202f | 0x205f | 0x3000 | 0xfeff)
}

/// `IsWhiteSpaceOrLineTerminator`.
fn is_white_space_or_line_terminator(c: u32) -> bool {
    is_white_space(c) || matches!(c, 0x0a | 0x0d | 0x2028 | 0x2029)
}

/// `DateParser::InputReader`.
struct Reader<'a> {
    text: &'a [u16],
    index: usize,
    ch: u32,
}

impl<'a> Reader<'a> {
    fn new(text: &'a [u16]) -> Self {
        let mut r = Reader { text, index: 0, ch: 0 };
        r.next();
        r
    }

    fn position(&self) -> usize {
        self.index
    }

    fn next(&mut self) {
        self.ch = self.text.get(self.index).map_or(0, |&c| c as u32);
        self.index += 1;
    }

    /// Reads digits, keeping the first nine significant ones.
    fn read_unsigned_numeral(&mut self) -> i32 {
        let mut n: i32 = 0;
        let mut i = 0;
        while self.ch == b'0' as u32 {
            self.next();
        }
        while self.is_ascii_digit() {
            if i < MAX_SIGNIFICANT_DIGITS {
                n = n * 10 + (self.ch - b'0' as u32) as i32;
            }
            i += 1;
            self.next();
        }
        n
    }

    /// Reads a word (characters from `A` up, not white space) and its lower-cased prefix.
    fn read_word(&mut self) -> ([u32; 3], usize) {
        let mut prefix = [0u32; 3];
        let mut len = 0;
        while self.is_ascii_alpha_or_above() && !self.is_white_space_char() {
            if len < 3 {
                prefix[len] = self.ch | 0x20;
            }
            len += 1;
            self.next();
        }
        (prefix, len)
    }

    fn skip(&mut self, c: u8) -> bool {
        if self.ch == c as u32 {
            self.next();
            return true;
        }
        false
    }

    fn skip_white_space(&mut self) -> bool {
        if is_white_space_or_line_terminator(self.ch) {
            self.next();
            return true;
        }
        false
    }

    fn skip_parentheses(&mut self) -> bool {
        if self.ch != b'(' as u32 {
            return false;
        }
        let mut balance = 0;
        loop {
            if self.ch == b')' as u32 {
                balance -= 1;
            } else if self.ch == b'(' as u32 {
                balance += 1;
            }
            self.next();
            if !(balance > 0 && self.ch != 0) {
                break;
            }
        }
        true
    }

    fn is_end(&self) -> bool {
        self.ch == 0
    }

    fn is_ascii_digit(&self) -> bool {
        (b'0' as u32..=b'9' as u32).contains(&self.ch)
    }

    fn is_ascii_alpha_or_above(&self) -> bool {
        self.ch >= b'A' as u32
    }

    fn is_white_space_char(&self) -> bool {
        is_white_space(self.ch)
    }
}

/// `DateParser::DateStringTokenizer`.
struct Tokenizer<'a> {
    input: Reader<'a>,
    next: Token,
}

impl<'a> Tokenizer<'a> {
    fn new(text: &'a [u16]) -> Self {
        let mut input = Reader::new(text);
        let next = scan(&mut input);
        Tokenizer { input, next }
    }

    fn next(&mut self) -> Token {
        let result = self.next;
        self.next = scan(&mut self.input);
        result
    }

    fn peek(&self) -> Token {
        self.next
    }

    fn skip_symbol(&mut self, c: u8) -> bool {
        if self.next.is_symbol(c) {
            self.next = scan(&mut self.input);
            return true;
        }
        false
    }
}

fn scan(input: &mut Reader) -> Token {
    let pre = input.position();
    if input.is_end() {
        return Token::EndOfInput;
    }
    if input.is_ascii_digit() {
        let value = input.read_unsigned_numeral();
        return Token::Number { value, length: input.position() - pre };
    }
    for c in [b':', b'-', b'+', b'.', b')'] {
        if input.skip(c) {
            return Token::Symbol(c);
        }
    }
    if input.is_ascii_alpha_or_above() && !input.is_white_space_char() {
        let (prefix, length) = input.read_word();
        let (kind, value) = lookup(prefix, length);
        return Token::Keyword { kind, value, length };
    }
    if input.skip_white_space() {
        return Token::WhiteSpace;
    }
    if input.skip_parentheses() {
        return Token::Unknown;
    }
    input.next();
    Token::Unknown
}

fn between(x: i32, lo: i32, hi: i32) -> bool {
    lo <= x && x <= hi
}

fn is_minute(x: i32) -> bool {
    between(x, 0, 59)
}

fn is_hour(x: i32) -> bool {
    between(x, 0, 23)
}

fn is_second(x: i32) -> bool {
    between(x, 0, 59)
}

fn is_month(x: i32) -> bool {
    between(x, 1, 12)
}

fn is_day(x: i32) -> bool {
    between(x, 1, 31)
}

/// `DateParser::TimeZoneComposer`.
struct TimeZone {
    sign: i32,
    hour: i32,
    minute: i32,
}

impl TimeZone {
    fn set(&mut self, offset_in_hours: i32) {
        self.sign = if offset_in_hours < 0 { -1 } else { 1 };
        self.hour = offset_in_hours * self.sign;
        self.minute = 0;
    }

    fn set_sign(&mut self, sign: i32) {
        self.sign = if sign < 0 { -1 } else { 1 };
    }

    fn is_expecting(&self, n: i32) -> bool {
        self.hour != NONE && self.minute == NONE && is_minute(n)
    }

    fn is_utc(&self) -> bool {
        self.hour == 0 && self.minute == 0
    }

    fn is_empty(&self) -> bool {
        self.hour == NONE
    }

    /// The offset in seconds, `Some(None)` for local time, or `None` when it is out of range.
    fn write(&mut self) -> Option<Option<f64>> {
        if self.sign == NONE {
            return Some(None);
        }
        if self.hour == NONE {
            self.hour = 0;
        }
        if self.minute == NONE {
            self.minute = 0;
        }
        let total = (self.hour as u32).wrapping_mul(3600).wrapping_add((self.minute as u32).wrapping_mul(60));
        // Smi::kMaxValue on 64-bit builds with pointer compression.
        if total > 0x3fff_ffff {
            return None;
        }
        let mut total = total as i32;
        if self.sign < 0 {
            total = -total;
        }
        Some(Some(total as f64))
    }
}

/// `DateParser::TimeComposer`.
struct Time {
    comp: [i32; 4],
    index: usize,
    hour_offset: i32,
}

impl Time {
    fn is_empty(&self) -> bool {
        self.index == 0
    }

    fn is_expecting(&self, n: i32) -> bool {
        (self.index == 1 && is_minute(n)) || (self.index == 2 && is_second(n)) || (self.index == 3 && between(n, 0, 999))
    }

    fn add(&mut self, n: i32) -> bool {
        if self.index < 4 {
            self.comp[self.index] = n;
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn add_final(&mut self, n: i32) -> bool {
        if !self.add(n) {
            return false;
        }
        while self.index < 4 {
            self.comp[self.index] = 0;
            self.index += 1;
        }
        true
    }

    fn write(&mut self) -> Option<[i32; 4]> {
        while self.index < 4 {
            self.comp[self.index] = 0;
            self.index += 1;
        }
        let [mut hour, minute, second, millisecond] = self.comp;
        if self.hour_offset != NONE {
            if !between(hour, 0, 12) {
                return None;
            }
            hour %= 12;
            hour += self.hour_offset;
        }
        if !is_hour(hour) || !is_minute(minute) || !is_second(second) || !between(millisecond, 0, 999) {
            // A 24th hour is allowed if minutes, seconds and milliseconds are 0.
            if hour != 24 || minute != 0 || second != 0 || millisecond != 0 {
                return None;
            }
        }
        Some([hour, minute, second, millisecond])
    }
}

/// `DateParser::DayComposer`.
struct Day {
    comp: [i32; 3],
    index: usize,
    named_month: i32,
    is_iso_date: bool,
}

impl Day {
    fn is_empty(&self) -> bool {
        self.index == 0
    }

    fn add(&mut self, n: i32) -> bool {
        if self.index < 3 {
            self.comp[self.index] = n;
            self.index += 1;
            true
        } else {
            false
        }
    }

    /// Year, month (0-based) and day.
    fn write(&mut self) -> Option<[i32; 3]> {
        if self.index < 1 {
            return None;
        }
        // Day and month default to 1.
        while self.index < 3 {
            self.comp[self.index] = 1;
            self.index += 1;
        }
        let mut year = 0; // Default year is 0 (=> 2000) for KJS compatibility.
        let month;
        let day;
        if self.named_month == NONE {
            if self.is_iso_date || (self.index == 3 && !is_day(self.comp[0])) {
                // YMD
                year = self.comp[0];
                month = self.comp[1];
                day = self.comp[2];
            } else {
                // MD(Y)
                month = self.comp[0];
                day = self.comp[1];
                if self.index == 3 {
                    year = self.comp[2];
                }
            }
        } else {
            month = self.named_month;
            if self.index == 1 {
                // MD or DM
                day = self.comp[0];
            } else if !is_day(self.comp[0]) {
                // YMD, MYD, or YDM
                year = self.comp[0];
                day = self.comp[1];
            } else {
                // DMY, MDY, or DYM
                day = self.comp[0];
                year = self.comp[1];
            }
        }
        if !self.is_iso_date {
            if between(year, 0, 49) {
                year += 2000;
            } else if between(year, 50, 99) {
                year += 1900;
            }
        }
        // Smi::IsValid with 31-bit Smis.
        if !between(year, -(1 << 30), (1 << 30) - 1) || !is_month(month) || !is_day(day) {
            return None;
        }
        Some([year, month - 1, day])
    }
}

/// `DateParser::ReadMilliseconds`: the first three significant digits of a numeral.
fn read_milliseconds(token: Token) -> i32 {
    let (mut number, mut length) = match token {
        Token::Number { value, length } => (value, length),
        _ => (0, 0),
    };
    if length < 3 {
        if length == 1 {
            number *= 100;
        } else if length == 2 {
            number *= 10;
        }
    } else if length > 3 {
        if length > MAX_SIGNIFICANT_DIGITS {
            length = MAX_SIGNIFICANT_DIGITS;
        }
        let mut factor = 1;
        loop {
            factor *= 10;
            length -= 1;
            if length <= 3 {
                break;
            }
        }
        number /= factor;
    }
    number
}

/// `DateParser::ParseES5DateTime`: returns the first token it did not account for.
fn parse_es5(scanner: &mut Tokenizer, day: &mut Day, time: &mut Time, tz: &mut TimeZone) -> Token {
    // Parse mandatory date: [('-'|'+')yy]yyyy[':'MM[':'DD]]
    if scanner.peek().is_ascii_sign() {
        // Keep the sign token, so invalid dates can be detected.
        let sign_token = scanner.next();
        if !scanner.peek().is_fixed_length_number(6) {
            return sign_token;
        }
        let sign = sign_token.ascii_sign();
        let year = scanner.next().number();
        if sign < 0 && year == 0 {
            return sign_token;
        }
        day.add(sign * year);
    } else if scanner.peek().is_fixed_length_number(4) {
        let year = scanner.next().number();
        day.add(year);
    } else {
        return scanner.next();
    }
    if scanner.skip_symbol(b'-') {
        if !scanner.peek().is_fixed_length_number(2) || !is_month(scanner.peek().number()) {
            return scanner.next();
        }
        let month = scanner.next().number();
        day.add(month);
        if scanner.skip_symbol(b'-') {
            if !scanner.peek().is_fixed_length_number(2) || !is_day(scanner.peek().number()) {
                return scanner.next();
            }
            let d = scanner.next().number();
            day.add(d);
        }
    }
    // Check for optional time: 'T'HH':'mm[':'ss['.'sss]]Z
    if !scanner.peek().is_keyword_type(Keyword::TimeSeparator) {
        if scanner.peek() != Token::EndOfInput {
            return scanner.next();
        }
    } else {
        // ES5 Date Time String time part is present.
        scanner.next();
        if !scanner.peek().is_fixed_length_number(2) || !between(scanner.peek().number(), 0, 24) {
            return Token::Invalid;
        }
        // Allow 24:00[:00[.000]], but no other time starting with 24.
        let hour_is_24 = scanner.peek().number() == 24;
        let hour = scanner.next().number();
        time.add(hour);
        if !scanner.skip_symbol(b':') {
            return Token::Invalid;
        }
        if !scanner.peek().is_fixed_length_number(2) || !is_minute(scanner.peek().number()) || (hour_is_24 && scanner.peek().number() > 0) {
            return Token::Invalid;
        }
        let minute = scanner.next().number();
        time.add(minute);
        if scanner.skip_symbol(b':') {
            if !scanner.peek().is_fixed_length_number(2)
                || !is_second(scanner.peek().number())
                || (hour_is_24 && scanner.peek().number() > 0)
            {
                return Token::Invalid;
            }
            let second = scanner.next().number();
            time.add(second);
            if scanner.skip_symbol(b'.') {
                if !scanner.peek().is_number() || (hour_is_24 && scanner.peek().number() > 0) {
                    return Token::Invalid;
                }
                // Allow more or less than the mandated three digits.
                let ms = read_milliseconds(scanner.next());
                time.add(ms);
            }
        }
        // Check for optional timezone designation: 'Z' | ('+'|'-')hh':'mm
        if scanner.peek().is_keyword_z() {
            scanner.next();
            tz.set(0);
        } else if scanner.peek().is_symbol(b'+') || scanner.peek().is_symbol(b'-') {
            let sign = if scanner.next().is_symbol(b'+') { 1 } else { -1 };
            tz.set_sign(sign);
            if scanner.peek().is_fixed_length_number(4) {
                // hhmm extension syntax.
                let hourmin = scanner.next().number();
                let hour = hourmin / 100;
                let min = hourmin % 100;
                if !is_hour(hour) || !is_minute(min) {
                    return Token::Invalid;
                }
                tz.hour = hour;
                tz.minute = min;
            } else {
                if !scanner.peek().is_fixed_length_number(2) || !is_hour(scanner.peek().number()) {
                    return Token::Invalid;
                }
                tz.hour = scanner.next().number();
                if !scanner.skip_symbol(b':') {
                    return Token::Invalid;
                }
                if !scanner.peek().is_fixed_length_number(2) || !is_minute(scanner.peek().number()) {
                    return Token::Invalid;
                }
                tz.minute = scanner.next().number();
            }
        }
        if scanner.peek() != Token::EndOfInput {
            return Token::Invalid;
        }
    }
    // Successfully parsed ES5 Date Time String. Date-only forms without a zone are UTC,
    // date-time forms local time.
    if tz.is_empty() && time.is_empty() {
        tz.set(0);
    }
    day.is_iso_date = true;
    Token::EndOfInput
}

/// The fields `DateParser::Parse` writes: year, month, day, hour, minute, second,
/// millisecond, and the offset from UTC in seconds (`None` for local time).
fn parse_fields(text: &[u16]) -> Option<([i32; 3], [i32; 4], Option<f64>)> {
    let mut scanner = Tokenizer::new(text);
    let mut tz = TimeZone { sign: NONE, hour: NONE, minute: NONE };
    let mut time = Time { comp: [0; 4], index: 0, hour_offset: NONE };
    let mut day = Day { comp: [0; 3], index: 0, named_month: NONE, is_iso_date: false };

    let next_unhandled = parse_es5(&mut scanner, &mut day, &mut time, &mut tz);
    if next_unhandled == Token::Invalid {
        return None;
    }
    let mut has_read_number = !day.is_empty();
    let mut token = next_unhandled;
    while token != Token::EndOfInput {
        match token {
            Token::Number { value: n, .. } => {
                has_read_number = true;
                if scanner.skip_symbol(b':') {
                    if scanner.skip_symbol(b':') {
                        // n + "::"
                        if !time.is_empty() {
                            return None;
                        }
                        time.add(n);
                        time.add(0);
                    } else {
                        // n + ":"
                        if !time.add(n) {
                            return None;
                        }
                        if scanner.peek().is_symbol(b'.') {
                            scanner.next();
                        }
                    }
                } else if scanner.skip_symbol(b'.') && time.is_expecting(n) {
                    time.add(n);
                    if !scanner.peek().is_number() {
                        return None;
                    }
                    let ms = read_milliseconds(scanner.next());
                    if ms < 0 {
                        return None;
                    }
                    time.add_final(ms);
                } else if tz.is_expecting(n) {
                    tz.minute = n;
                } else if time.is_expecting(n) {
                    time.add_final(n);
                    // Require end, white space, "Z", "+" or "-" immediately after finalizing time.
                    let peek = scanner.peek();
                    if peek != Token::EndOfInput && peek != Token::WhiteSpace && !peek.is_keyword_z() && !peek.is_ascii_sign() {
                        return None;
                    }
                } else {
                    if !day.add(n) {
                        return None;
                    }
                    scanner.skip_symbol(b'-');
                }
            }
            Token::Keyword { kind, value, .. } => {
                if kind == Keyword::AmPm && !time.is_empty() {
                    time.hour_offset = value;
                } else if kind == Keyword::MonthName {
                    day.named_month = value;
                    scanner.skip_symbol(b'-');
                } else if kind == Keyword::TimeZoneName && has_read_number {
                    tz.set(value);
                } else {
                    // Garbage words are illegal if a number has been read.
                    if has_read_number {
                        return None;
                    }
                    // The first number has to be separated from garbage words by white space
                    // or other separators.
                    if scanner.peek().is_number() {
                        return None;
                    }
                }
            }
            _ if token.is_ascii_sign() && (tz.is_utc() || !time.is_empty()) => {
                // Parse UTC offset (only after UTC or time).
                tz.set_sign(token.ascii_sign());
                // The following number may be empty.
                let mut n = 0;
                let mut length = 0;
                if scanner.peek().is_number() {
                    if let Token::Number { value, length: l } = scanner.next() {
                        n = value;
                        length = l;
                    }
                }
                has_read_number = true;
                if scanner.peek().is_symbol(b':') {
                    tz.hour = n;
                    tz.minute = NONE;
                } else if length == 2 || length == 1 {
                    // Time zones like GMT-8.
                    tz.hour = n;
                    tz.minute = 0;
                } else if length == 4 || length == 3 {
                    // Time zones like GMT-0800.
                    tz.hour = n / 100;
                    tz.minute = n % 100;
                } else {
                    // No need to accept time zones like GMT-12345.
                    return None;
                }
            }
            _ if (token.is_ascii_sign() || token.is_symbol(b')')) && has_read_number => {
                // Extra sign or ')' is illegal if a number has been read.
                return None;
            }
            _ => {
                // Ignore other characters and white space.
            }
        }
        token = scanner.next();
    }
    let date = day.write()?;
    let clock = time.write()?;
    let offset = tz.write()?;
    Some((date, clock, offset))
}

/// `new Date(text).getTime()`: the time value V8's `ParseDateTimeString` gives, or NaN.
pub(crate) fn parse_date(text: &str, zone: Zone) -> f64 {
    let units: Vec<u16> = text.encode_utf16().collect();
    let Some(([year, month, day], [hour, minute, second, ms], offset)) = parse_fields(&units) else {
        return f64::NAN;
    };
    let day = make_day(year as f64, month as f64, day as f64);
    let time = make_time(hour as f64, minute as f64, second as f64, ms as f64);
    let date = make_date(day, time);
    // DateCache::kMaxTimeInMs and kMaxTimeBeforeUTCInMs.
    let max = 8.64e15;
    match offset {
        None => {
            if !(-(max + 30.0 * MS_PER_DAY)..=(max + 30.0 * MS_PER_DAY)).contains(&date) {
                return f64::NAN;
            }
            time_clip(zone.at_local(date.trunc()))
        }
        Some(seconds) => {
            let date = date - seconds * 1000.0;
            if !(-max..=max).contains(&date) {
                return f64::NAN;
            }
            time_clip(date)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> f64 {
        parse_date(s, Zone::UTC)
    }

    #[test]
    fn iso_forms() {
        assert_eq!(parse("2014-01-01"), 1388534400000.0);
        assert_eq!(parse("2014-02"), 1391212800000.0);
        assert_eq!(parse("2014"), 1388534400000.0);
        assert_eq!(parse("2014-01-01T10:20:30.5Z"), 1388571630500.0);
        assert_eq!(parse("2014-01-01T10:20+0130"), 1388566200000.0);
        assert_eq!(parse("2014-01-01T24:00"), 1388620800000.0);
        assert!(parse("2014-01-01T24:01").is_nan());
        assert_eq!(parse("+002014-01-01"), 1388534400000.0);
        // V8 falls back to its legacy date parser without a time component.
        assert_eq!(parse("-000000-01-01"), 978307200000.0);
        assert!(parse("-000000-01-01T00:00:00Z").is_nan());
        assert_eq!(parse("0001-01-01"), -62135596800000.0);
    }

    #[test]
    fn legacy_forms() {
        // From Pele's test inputs, values from Node 26 (V8 14.6) with TZ=UTC.
        for (text, value) in [
            ("2014-011-20", 1416441600000.0),
            ("2014-1-01", 1388534400000.0),
            ("mm- 2014-01-01", 1388534400000.0),
            ("teFormat yyyy-mm-d 0", 946684800000.0),
            ("201-01-01", -55824249600000.0),
            ("///etc/passwd: 0", 946684800000.0),
            ("alert(1)nother :2014-01-12", 1389484800000.0),
            ("2014-02-", 1391212800000.0),
            ("58", -378691200000.0),
            ("0", 946684800000.0),
            ("10", 1001894400000.0),
            ("192", -56108246400000.0),
            ("191 8", -56121465600000.0),
            ("49", 2493072000000.0),
            ("4-02-10", 1270166400000.0),
            ("javascript:alert(1)2014-01-01", 1388534400000.0),
            ("--><script>alert(1)</script> 2024-01-01", 1704067200000.0),
            ("2014-001", 1388534400000.0),
            ("1", 978307200000.0),
            ("url(//evil.example/x) 1910", -1893456000000.0),
            ("]]><script>alert(1)</script>2014-01-01", 1388534400000.0),
        ] {
            assert_eq!(parse(text), value, "{text}");
        }
        assert!(parse("tomorrow").is_nan());
        assert!(parse("2014-01-01 x").is_nan());
        assert!(parse("").is_nan());
    }

    #[test]
    fn local_forms_take_the_zone() {
        let zone = Zone::from_minutes(-300);
        // A date-only ISO form is UTC; a date-time form and legacy forms are local time.
        assert_eq!(parse_date("2014-01-01", zone), 1388534400000.0);
        assert_eq!(parse_date("2014-01-01T00:00", zone), 1388552400000.0);
        assert_eq!(parse_date("Jan 1 2014", zone), 1388552400000.0);
        assert_eq!(parse_date("Jan 1 2014 GMT+2", zone), 1388527200000.0);
    }
}
