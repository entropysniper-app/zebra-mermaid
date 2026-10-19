//! Gantt lexer. Ported from Pele `src/diagrams/gantt/lexer.ts` and `tokens.ts`, which follow
//! the states and rules of Mermaid's Jison grammar.
//!
//! Offsets are bytes. Every delimiter is ASCII; where the original tests a UTF-16 code unit
//! that can be outside ASCII (white space), the whole character is decoded, which gives the
//! same answer because no character it accepts is outside the Basic Multilingual Plane.

use crate::util::chars::{char_at, is_space, is_word};

/// Token types, in the order of `tokens.ts`.
pub(crate) mod t {
    pub const END: u8 = 0;
    pub const ERROR: u8 = 1;
    pub const OPEN_DIRECTIVE: u8 = 2;
    pub const ACC_TITLE: u8 = 3;
    pub const ACC_TITLE_VALUE: u8 = 4;
    pub const ACC_DESCR: u8 = 5;
    pub const ACC_DESCR_VALUE: u8 = 6;
    pub const ACC_DESCR_MULTILINE_VALUE: u8 = 7;
    pub const NL: u8 = 8;
    pub const HREF: u8 = 9;
    pub const CALLBACKNAME: u8 = 10;
    pub const CALLBACKARGS: u8 = 11;
    pub const CLICK: u8 = 12;
    pub const GANTT: u8 = 13;
    pub const DATE_FORMAT: u8 = 14;
    pub const INCLUSIVE_END_DATES: u8 = 15;
    pub const TOP_AXIS: u8 = 16;
    pub const AXIS_FORMAT: u8 = 17;
    pub const TICK_INTERVAL: u8 = 18;
    pub const INCLUDES: u8 = 19;
    pub const EXCLUDES: u8 = 20;
    pub const TODAY_MARKER: u8 = 21;
    pub const WEEKDAY_MONDAY: u8 = 22;
    pub const WEEKDAY_SUNDAY: u8 = 28;
    pub const WEEKEND_FRIDAY: u8 = 29;
    pub const WEEKEND_SATURDAY: u8 = 30;
    pub const DATE: u8 = 31;
    pub const TITLE: u8 = 32;
    pub const ACC_DESCRIPTION: u8 = 33;
    pub const SECTION: u8 = 34;
    pub const TASK_TXT: u8 = 35;
    pub const TASK_DATA: u8 = 36;
    pub const COLON: u8 = 37;
    pub const EOF: u8 = 38;

    pub const NAMES: [&str; 39] = [
        "$end",
        "INVALID",
        "open_directive",
        "acc_title",
        "acc_title_value",
        "acc_descr",
        "acc_descr_value",
        "acc_descr_multiline_value",
        "NL",
        "href",
        "callbackname",
        "callbackargs",
        "click",
        "gantt",
        "dateFormat",
        "inclusiveEndDates",
        "topAxis",
        "axisFormat",
        "tickInterval",
        "includes",
        "excludes",
        "todayMarker",
        "weekday_monday",
        "weekday_tuesday",
        "weekday_wednesday",
        "weekday_thursday",
        "weekday_friday",
        "weekday_saturday",
        "weekday_sunday",
        "weekend_friday",
        "weekend_saturday",
        "date",
        "title",
        "accDescription",
        "section",
        "taskTxt",
        "taskData",
        ":",
        "EOF",
    ];
}

pub(crate) struct Token {
    pub kind: u8,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Initial,
    AccTitle,
    AccDescr,
    AccDescrMultiline,
    Href,
    Callbackname,
    Callbackargs,
    Click,
}

const DAYS: [&str; 7] = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];

/// Whether `src` has `word` at `p`, ignoring ASCII case (the `i` flag without `u`).
fn word_at(src: &str, p: usize, word: &str) -> bool {
    src.as_bytes().get(p..p + word.len()).is_some_and(|b| b.eq_ignore_ascii_case(word.as_bytes()))
}

/// The end of a run of `\s` from `p`.
fn spaces(src: &str, mut p: usize) -> usize {
    while let Some(c) = char_at(src, p).filter(|&c| is_space(c)) {
        p += c.len_utf8();
    }
    p
}

/// `\s` at `p`: the end of that one character.
fn space(src: &str, p: usize) -> Option<usize> {
    char_at(src, p).filter(|&c| is_space(c)).map(|c| p + c.len_utf8())
}

/// `\b` after a word that ends at `p`: what follows is not a word character.
fn boundary(src: &str, p: usize) -> bool {
    !char_at(src, p).is_some_and(is_word)
}

/// `word\s*:\s*` and similar: `word`, white space, `sep`, white space.
fn word_sep(src: &str, p: usize, word: &str, sep: u8) -> Option<usize> {
    if !word_at(src, p, word) {
        return None;
    }
    let q = spaces(src, p + word.len());
    if src.as_bytes().get(q) != Some(&sep) {
        return None;
    }
    Some(spaces(src, q + 1))
}

/// `word\s+`: the end of the white space.
fn word_spaces(src: &str, p: usize, word: &str) -> Option<usize> {
    if !word_at(src, p, word) {
        return None;
    }
    let q = space(src, p + word.len())?;
    Some(spaces(src, q))
}

/// `word\s[^...]+`: a keyword, one white space character, then at least one character up to
/// the first of `stops`.
fn word_rest(src: &str, p: usize, word: &str, stops: &[u8]) -> Option<usize> {
    if !word_at(src, p, word) {
        return None;
    }
    let q = space(src, p + word.len())?;
    let b = src.as_bytes();
    let mut e = q;
    while e < b.len() && !stops.contains(&b[e]) {
        e += 1;
    }
    (e > q).then_some(e)
}

/// `word\b`.
fn word_boundary(src: &str, p: usize, word: &str) -> Option<usize> {
    (word_at(src, p, word) && boundary(src, p + word.len())).then_some(p + word.len())
}

/// `/weekday\s+(monday|...|sunday)\b/iy`: the day's index and the end.
fn weekday(src: &str, p: usize) -> Option<(usize, usize)> {
    let q = word_spaces(src, p, "weekday")?;
    DAYS.iter().enumerate().find_map(|(k, day)| word_boundary(src, q, day).map(|e| (k, e)))
}

/// `/weekend\s+(?:(friday)|saturday)\b/iy`: whether it is Friday, and the end.
fn weekend(src: &str, p: usize) -> Option<(bool, usize)> {
    let q = word_spaces(src, p, "weekend")?;
    if let Some(e) = word_boundary(src, q, "friday") {
        return Some((true, e));
    }
    word_boundary(src, q, "saturday").map(|e| (false, e))
}

/// `/\d\d\d\d-\d\d-\d\d\b/y`
fn date_at(src: &str, p: usize) -> bool {
    let b = src.as_bytes();
    let Some(s) = b.get(p..p + 10) else { return false };
    s.iter().enumerate().all(|(i, &c)| if i == 4 || i == 7 { c == b'-' } else { c.is_ascii_digit() }) && boundary(src, p + 10)
}

/// The statements that start with a keyword, by the keyword's first letter, in the grammar's
/// rule order: the keyword, what ends its text (or `None` for `\b`), and the token.
fn keyword_rule(src: &str, p: usize, c: u8) -> Option<(u8, usize)> {
    const NO_HASH: &[u8] = b"#\n;";
    let rules: &[(&str, Option<&[u8]>, u8)] = match c {
        b'a' => &[("axisFormat", Some(NO_HASH), t::AXIS_FORMAT), ("accDescription", Some(NO_HASH), t::ACC_DESCRIPTION)],
        b'g' => &[("gantt", None, t::GANTT)],
        b'd' => &[("dateFormat", Some(NO_HASH), t::DATE_FORMAT)],
        b'i' => &[("inclusiveEndDates", None, t::INCLUSIVE_END_DATES), ("includes", Some(NO_HASH), t::INCLUDES)],
        b't' => &[
            ("topAxis", None, t::TOP_AXIS),
            ("tickInterval", Some(NO_HASH), t::TICK_INTERVAL),
            ("todayMarker", Some(b"\n;"), t::TODAY_MARKER),
            ("title", Some(b"\n"), t::TITLE),
        ],
        b'e' => &[("excludes", Some(NO_HASH), t::EXCLUDES)],
        b's' => &[("section", Some(b"\n"), t::SECTION)],
        _ => &[],
    };
    for &(word, stops, kind) in rules {
        let end = match stops {
            None => word_boundary(src, p, word),
            Some(stops) => word_rest(src, p, word, stops),
        };
        if let Some(e) = end {
            return Some((kind, e));
        }
    }
    None
}

pub(crate) fn tokenize(src: &str) -> Vec<Token> {
    let b = src.as_bytes();
    let n = b.len();
    let mut out: Vec<Token> = Vec::new();
    let mut state = State::Initial;
    let mut p = 0;
    let until = |ch: u8, from: usize| -> usize { b.get(from..).and_then(|s| s.iter().position(|&c| c == ch)).map_or(n, |k| from + k) };

    loop {
        if p >= n {
            let kind = match state {
                State::Initial => t::EOF,
                State::AccTitle => t::ACC_TITLE_VALUE,
                State::AccDescr => t::ACC_DESCR_VALUE,
                State::AccDescrMultiline => t::ACC_DESCR_MULTILINE_VALUE,
                State::Href => t::HREF,
                State::Callbackname => t::CALLBACKNAME,
                State::Callbackargs => t::CALLBACKARGS,
                State::Click => t::CLICK,
            };
            out.push(Token { kind, start: n, end: n });
            break;
        }
        let c = b[p];
        match state {
            State::Initial => {
                let ch = char_at(src, p).unwrap_or('\u{fffd}');
                // The code unit after this one: a character outside the Basic Multilingual Plane
                // is followed by its low surrogate, which is never `%`.
                let after = p + ch.len_utf8();
                let d = if ch.len_utf16() == 1 { b.get(after).copied() } else { None };
                if c == b'%' {
                    if d != Some(b'%') {
                        let mut e = until(b'\n', p + 1);
                        while b.get(e) == Some(&b'\n') {
                            e += 1;
                        }
                        p = e;
                    } else if b.get(p + 2) == Some(&b'{') {
                        // The grammar enters a lexer state it never defines, so nothing can follow.
                        out.push(Token { kind: t::OPEN_DIRECTIVE, start: p, end: p + 3 });
                        out.push(Token { kind: t::ERROR, start: p, end: p });
                        return out;
                    } else {
                        p = until(b'\n', p + 2);
                    }
                } else if c != b'}' && d == Some(b'%') {
                    // Any character but `}` followed by `%` starts a comment that runs to the end
                    // of the line.
                    p = until(b'\n', after + 1);
                } else if c == b'\n' {
                    let mut e = p + 1;
                    while b.get(e) == Some(&b'\n') {
                        e += 1;
                    }
                    out.push(Token { kind: t::NL, start: p, end: e });
                    p = e;
                } else if is_space(ch) {
                    p = spaces(src, p);
                } else if let Some((kind, e, next)) = keyword(src, p, c) {
                    if let Some(kind) = kind {
                        out.push(Token { kind, start: p, end: e });
                    }
                    if let Some(next) = next {
                        state = next;
                    }
                    p = e;
                } else if c.is_ascii_digit() && date_at(src, p) {
                    out.push(Token { kind: t::DATE, start: p, end: p + 10 });
                    p += 10;
                } else if c != b':' {
                    let mut e = after;
                    while e < n && b[e] != b':' && b[e] != b'\n' {
                        e += 1;
                    }
                    out.push(Token { kind: t::TASK_TXT, start: p, end: e });
                    p = e;
                } else {
                    let mut e = p + 1;
                    while e < n && b[e] != b'#' && b[e] != b'\n' && b[e] != b';' {
                        e += 1;
                    }
                    out.push(Token { kind: if e > p + 1 { t::TASK_DATA } else { t::COLON }, start: p, end: e });
                    p = e;
                }
            }
            State::AccTitle | State::AccDescr => {
                let kind = if state == State::AccTitle { t::ACC_TITLE_VALUE } else { t::ACC_DESCR_VALUE };
                state = State::Initial;
                let e = until(b'\n', p);
                out.push(Token { kind, start: p, end: e });
                p = e;
            }
            State::AccDescrMultiline => {
                if c == b'}' {
                    state = State::Initial;
                    p += 1;
                } else {
                    let e = until(b'}', p);
                    out.push(Token { kind: t::ACC_DESCR_MULTILINE_VALUE, start: p, end: e });
                    p = e;
                }
            }
            State::Href => {
                if c == b'"' {
                    state = State::Initial;
                    p += 1;
                } else {
                    let e = until(b'"', p);
                    out.push(Token { kind: t::HREF, start: p, end: e });
                    p = e;
                }
            }
            State::Callbackname => {
                if c == b'(' {
                    // `/\(\s*\)/y`
                    let q = spaces(src, p + 1);
                    if b.get(q) == Some(&b')') {
                        state = State::Initial;
                        p = q + 1;
                    } else {
                        state = State::Callbackargs;
                        p += 1;
                    }
                } else {
                    let e = until(b'(', p);
                    out.push(Token { kind: t::CALLBACKNAME, start: p, end: e });
                    p = e;
                }
            }
            State::Callbackargs => {
                if c == b')' {
                    state = State::Initial;
                    p += 1;
                } else {
                    let e = until(b')', p);
                    out.push(Token { kind: t::CALLBACKARGS, start: p, end: e });
                    p = e;
                }
            }
            State::Click => {
                let ch = char_at(src, p).unwrap_or('\u{fffd}');
                if is_space(ch) {
                    state = State::Initial;
                    p += ch.len_utf8();
                } else {
                    let mut e = p + ch.len_utf8();
                    while let Some(x) = char_at(src, e).filter(|&x| !is_space(x)) {
                        e += x.len_utf8();
                    }
                    out.push(Token { kind: t::CLICK, start: p, end: e });
                    p = e;
                }
            }
        }
    }
    out.push(Token { kind: t::END, start: n, end: n });
    out
}

/// A statement that starts with a keyword at `p`: the token it emits (if any), where it ends,
/// and the state it enters (if any).
fn keyword(src: &str, p: usize, c: u8) -> Option<(Option<u8>, usize, Option<State>)> {
    let lower = c | 32;
    match lower {
        b'a' => {
            if let Some(e) = word_sep(src, p, "accTitle", b':') {
                return Some((Some(t::ACC_TITLE), e, Some(State::AccTitle)));
            }
            if let Some(e) = word_sep(src, p, "accDescr", b':') {
                return Some((Some(t::ACC_DESCR), e, Some(State::AccDescr)));
            }
            if let Some(e) = word_sep(src, p, "accDescr", b'{') {
                return Some((None, e, Some(State::AccDescrMultiline)));
            }
        }
        b'h' => {
            // `/href\s+"/iy`
            if let Some(q) = word_spaces(src, p, "href") {
                if src.as_bytes().get(q) == Some(&b'"') {
                    return Some((None, q + 1, Some(State::Href)));
                }
            }
        }
        b'c' => {
            if let Some(e) = word_spaces(src, p, "call") {
                return Some((None, e, Some(State::Callbackname)));
            }
            if let Some(e) = word_spaces(src, p, "click") {
                return Some((None, e, Some(State::Click)));
            }
        }
        b'w' => {
            if let Some((day, e)) = weekday(src, p) {
                return Some((Some(t::WEEKDAY_MONDAY + day as u8), e, None));
            }
            if let Some((friday, e)) = weekend(src, p) {
                return Some((Some(if friday { t::WEEKEND_FRIDAY } else { t::WEEKEND_SATURDAY }), e, None));
            }
        }
        _ => {}
    }
    keyword_rule(src, p, lower).map(|(kind, e)| (Some(kind), e, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(u8, &str)> {
        tokenize(src).into_iter().map(|tk| (tk.kind, &src[tk.start..tk.end])).collect()
    }

    #[test]
    fn statements() {
        assert_eq!(
            kinds("gantt\n  dateFormat YYYY-MM-DD\n  A task :a1, 2014-01-01, 30d\n"),
            vec![
                (t::GANTT, "gantt"),
                (t::NL, "\n"),
                (t::DATE_FORMAT, "dateFormat YYYY-MM-DD"),
                (t::NL, "\n"),
                (t::TASK_TXT, "A task "),
                (t::TASK_DATA, ":a1, 2014-01-01, 30d"),
                (t::NL, "\n"),
                (t::EOF, ""),
                (t::END, ""),
            ]
        );
    }

    #[test]
    fn quirks() {
        // A keyword's one white space character may be a line break.
        assert_eq!(kinds("axisFormat\nx\n")[0], (t::AXIS_FORMAT, "axisFormat\nx"));
        // A single `%` comments out the rest of the line and the line breaks after it.
        assert_eq!(kinds("% x\n\nA")[0], (t::TASK_TXT, "A"));
        // Any character but `}` before `%` starts a comment.
        assert_eq!(kinds("a% x\nb")[0], (t::NL, "\n"));
        assert_eq!(kinds("WEEKDAY  Friday")[0], (t::WEEKDAY_MONDAY + 4, "WEEKDAY  Friday"));
        assert_eq!(kinds("weekend fridays")[0].0, t::TASK_TXT);
        assert_eq!(kinds("click a href \"u\"")[..3].iter().map(|k| k.0).collect::<Vec<_>>(), vec![t::CLICK, t::HREF, t::EOF]);
        assert_eq!(kinds("%%{init}")[1].0, t::ERROR);
    }
}
