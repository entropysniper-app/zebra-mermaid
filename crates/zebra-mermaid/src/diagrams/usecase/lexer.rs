//! Ordered Chevrotain-compatible use-case lexer. Offsets stay in UTF-16 units.
use super::tokens::*;
use crate::{DiagramType, Error, ErrorCode};
#[derive(Clone, Debug)]
pub struct Source {
    pub units: Vec<u16>,
}
impl Source {
    pub fn new(s: &str) -> Self {
        Self { units: s.encode_utf16().collect() }
    }
    pub fn len(&self) -> usize {
        self.units.len()
    }
    pub fn code(&self, i: usize) -> u16 {
        self.units.get(i).copied().unwrap_or(u16::MAX)
    }
    pub fn starts(&self, i: usize, s: &str) -> bool {
        self.units.get(i..).is_some_and(|rest| rest.starts_with(&s.encode_utf16().collect::<Vec<_>>()))
    }
    pub fn slice(&self, a: usize, b: usize) -> String {
        String::from_utf16_lossy(self.units.get(a..b).unwrap_or_default())
    }
}
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: u8,
    pub start: usize,
    pub end: usize,
}
fn word(c: u16) -> bool {
    c < 128 && ((c as u8).is_ascii_alphanumeric() || c == 95)
}
fn digit(c: u16) -> bool {
    (48..=57).contains(&c)
}
fn letter(c: u16) -> bool {
    (65..=90).contains(&c) || (97..=122).contains(&c)
}
fn blank(c: u16) -> bool {
    c == 32 || c == 9
}
fn hex(c: u16) -> bool {
    digit(c) || (65..=70).contains(&c) || (97..=102).contains(&c)
}
fn no_case(s: &Source, at: usize, w: &str) -> bool {
    w.bytes().enumerate().all(|(i, b)| s.code(at + i) | 32 == b as u16)
}
fn keyword(s: &Source, at: usize, c: u16) -> u8 {
    match c {
        97 => {
            if s.starts(at, "actor") {
                ACTOR
            } else {
                0
            }
        }
        115 => {
            if s.starts(at, "systemBoundary") {
                SYSTEM_BOUNDARY
            } else if s.starts(at, "style") {
                STYLE
            } else {
                0
            }
        }
        101 => {
            if s.starts(at, "end") {
                END
            } else if no_case(s, at, "extend") {
                EXTEND
            } else {
                0
            }
        }
        69 => {
            if no_case(s, at, "extend") {
                EXTEND
            } else {
                0
            }
        }
        100 => {
            if s.starts(at, "direction") {
                DIRECTION
            } else {
                0
            }
        }
        84 => {
            if s.starts(at, "TD") {
                TD
            } else if s.starts(at, "TB") {
                TB
            } else {
                0
            }
        }
        66 => {
            if s.starts(at, "BT") {
                BT
            } else {
                0
            }
        }
        76 => {
            if s.starts(at, "LR") {
                LR
            } else {
                0
            }
        }
        82 => {
            if s.starts(at, "RL") {
                RL
            } else {
                0
            }
        }
        110 => {
            if s.starts(at, "note") {
                NOTE
            } else {
                0
            }
        }
        102 => {
            if s.starts(at, "for") {
                FOR
            } else if s.starts(at, "false") {
                FALSE
            } else {
                0
            }
        }
        106 => {
            if s.starts(at, "json") {
                JSON
            } else {
                0
            }
        }
        99 => {
            if s.starts(at, "classDef") {
                CLASS_DEF
            } else if s.starts(at, "class") {
                CLASS
            } else {
                0
            }
        }
        105 | 73 => {
            if no_case(s, at, "include") {
                INCLUDE
            } else {
                0
            }
        }
        116 => {
            if s.starts(at, "true") {
                TRUE
            } else {
                0
            }
        }
        117 => {
            if s.starts(at, "usecase-beta") {
                USECASE
            } else {
                0
            }
        }
        _ => 0,
    }
}
pub fn line_column(s: &Source, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut start = 0;
    let mut i = 0;
    while i < offset {
        let c = s.code(i);
        if c == 10 || c == 13 {
            if c == 13 && i + 1 < offset && s.code(i + 1) == 10 {
                i += 1;
            }
            line += 1;
            start = i + 1;
        }
        i += 1;
    }
    (line, offset - start + 1)
}
pub fn location_text(s: &Source, start: usize, end: usize) -> String {
    let (l, c) = line_column(s, start);
    format!("line {l}, column {c} [{start},{end})")
}
fn string_end(s: &Source, at: usize, q: u16) -> Option<usize> {
    for k in at + 1..s.len() {
        let c = s.code(k);
        if c == q {
            return Some(k + 1);
        }
        if c == 10 || c == 13 {
            return None;
        }
    }
    None
}
fn starts_token(s: &Source, at: usize) -> bool {
    let c = s.code(at);
    if c != 34 && c != 39 {
        return true;
    }
    (c == 34 && s.code(at + 1) == 96) || string_end(s, at, c).is_some()
}
fn error(s: &Source, at: usize, n: usize) -> Error {
    let (line, column) = line_column(s, at);
    let mut e=Error::new(format!("Error lexing usecase diagram: unexpected character: ->{}<- at offset: {at}, skipped {n} characters. at line {line}, column {column} [{at},{})",s.slice(at,at+1),at+n),ErrorCode::Syntax).with_type(DiagramType::Usecase);
    e.line = line;
    e.column = column;
    e
}
pub fn tokenize(s: &Source) -> Result<Vec<Token>, Error> {
    const LENGTH: [usize; 21] = [0, 12, 5, 14, 3, 9, 2, 2, 2, 2, 2, 4, 3, 4, 8, 5, 5, 7, 6, 4, 5];
    let n = s.len();
    let mut i = 0;
    let mut out = vec![];
    let mut line_start = true;
    let mut no_brace = false;
    while i < n {
        let c = s.code(i);
        if blank(c) {
            i += 1;
            continue;
        }
        let mut kind;
        let mut end = i + 1;
        let mut fresh = false;
        if word(c) {
            kind = 0;
            if c == 97 && line_start && s.starts(i, "acc") {
                let descr = s.starts(i + 3, "Descr");
                if descr || s.starts(i + 3, "Title") {
                    let mut j = i + 8;
                    while blank(s.code(j)) {
                        j += 1;
                    }
                    let next = s.code(j);
                    if next == 58 {
                        kind = if descr { ACC_DESCR_LINE } else { ACC_TITLE_LINE };
                        end = j + 1;
                        while end < n && !matches!(s.code(end), 10 | 13) {
                            end += 1;
                        }
                    } else if next == 123 && descr && !no_brace {
                        if let Some(k) = (j + 1..n).find(|&k| s.code(k) == 125) {
                            kind = ACC_DESCR_BLOCK;
                            end = k + 1;
                        } else {
                            no_brace = true;
                        }
                    }
                }
            } else if c == 106 && s.starts(i, "json") && blank(s.code(i + 4)) {
                let mut j = i + 5;
                while blank(s.code(j)) {
                    j += 1;
                }
                let id = j;
                while word(s.code(j)) {
                    j += 1;
                }
                if j > id {
                    while blank(s.code(j)) {
                        j += 1;
                    }
                    if s.code(j) == 64 {
                        j += 1;
                        while blank(s.code(j)) {
                            j += 1;
                        }
                        if s.code(j) == 123 {
                            out.push(Token { kind: JSON_DECLARATION_START, start: i, end: j });
                            let mut depth = 0i64;
                            let mut quoted = false;
                            let mut escaped = false;
                            let mut k = j;
                            while k < n {
                                let ch = s.code(k);
                                if quoted {
                                    if escaped {
                                        escaped = false;
                                    } else if ch == 92 {
                                        escaped = true;
                                    } else if ch == 34 {
                                        quoted = false;
                                    }
                                } else if ch == 34 {
                                    quoted = true;
                                } else if ch == 123 {
                                    depth += 1;
                                } else if ch == 125 {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                k += 1;
                            }
                            i = if k < n { k + 1 } else { n };
                            out.push(Token {
                                kind: if k < n { JSON_OBJECT_LITERAL } else { UNCLOSED_JSON_OBJECT_LITERAL },
                                start: j,
                                end: i,
                            });
                            line_start = false;
                            continue;
                        }
                    }
                }
            }
            if kind == 0 {
                while end < n && word(s.code(end)) {
                    end += 1;
                }
                let key = keyword(s, i, c);
                if key != 0 {
                    if key == USECASE || end - i == LENGTH[key as usize] {
                        kind = key;
                        end = i + LENGTH[key as usize];
                    } else {
                        kind = IDENTIFIER;
                    }
                } else if matches!(c, 111 | 120) && s.code(i + 1) == 45 && s.code(i + 2) == 45 {
                    kind = if c == 111 { BACKWARD_CIRCLE } else { BACKWARD_CROSS };
                    end = i + 3;
                } else if digit(c) {
                    let mut digits = i + 1;
                    while digit(s.code(digits)) {
                        digits += 1;
                    }
                    if s.code(digits) == 46 && digit(s.code(digits + 1)) {
                        kind = NUMBER;
                        end = digits + 2;
                        while digit(s.code(end)) {
                            end += 1;
                        }
                        while letter(s.code(end)) {
                            end += 1;
                        }
                    } else {
                        kind = IDENTIFIER;
                    }
                } else {
                    kind = IDENTIFIER;
                    while s.code(end) == 45 && word(s.code(end + 1)) {
                        kind = CSS_IDENTIFIER;
                        end += 2;
                        while word(s.code(end)) {
                            end += 1;
                        }
                    }
                }
            }
        } else if c == 10 {
            kind = NEWLINE;
            fresh = true;
        } else if c == 13 {
            kind = NEWLINE;
            fresh = true;
            if s.code(end) == 10 {
                end += 1;
            }
        } else if c == 34 || c == 39 {
            if c == 34 && s.code(i + 1) == 96 {
                if let Some(close) = (i + 2..n).find(|&k| s.code(k) == 96 && s.code(k + 1) == 34) {
                    kind = MARKDOWN_STRING;
                    end = close + 2;
                } else {
                    kind = UNCLOSED_MARKDOWN_STRING;
                    end = n;
                }
            } else {
                kind = PLAIN_STRING;
                if let Some(k) = string_end(s, i, c) {
                    end = k;
                } else {
                    let mut resume = i + 1;
                    while resume < n && !starts_token(s, resume) {
                        resume += 1;
                    }
                    return Err(error(s, i, resume - i));
                }
            }
        } else if c == 37 {
            if line_start && s.code(end) == 37 {
                kind = COMMENT;
                while end < n && !matches!(s.code(end), 10 | 13) {
                    end += 1;
                }
            } else {
                kind = PERCENT;
            }
        } else if c == 45 {
            if s.code(end) != 45 {
                kind = DASH;
            } else {
                while s.code(end) == 45 {
                    end += 1;
                }
                let next = s.code(end);
                let two = end - i == 2;
                if two && next == 124 && s.code(end + 1) == 62 {
                    kind = GENERALIZATION;
                    end += 2;
                } else if next == 62 {
                    kind = FORWARD_SOLID;
                    end += 1;
                } else if two && matches!(next, 111 | 120) {
                    kind = if next == 111 { FORWARD_CIRCLE } else { FORWARD_CROSS };
                    end += 1;
                } else {
                    kind = MARKERLESS_SOLID;
                }
            }
        } else if c == 46 {
            let next = s.code(end);
            if next == 46 && s.code(end + 1) == 62 {
                kind = DEPENDENCY_ARROW;
                end += 2;
            } else if digit(next) {
                kind = NUMBER;
                while digit(s.code(end)) {
                    end += 1;
                }
                while letter(s.code(end)) {
                    end += 1;
                }
            } else {
                kind = DOT;
            }
        } else if c == 60 {
            if s.code(end) == 60 {
                out.push(Token { kind: STEREOTYPE_START, start: i, end: i + 2 });
                i += 2;
                if i >= n {
                    break;
                }
                if s.code(i) == 62 && s.code(i + 1) == 62 {
                    kind = STEREOTYPE_END;
                    end = i + 2;
                } else {
                    let mut close = i;
                    let mut found = false;
                    while close < n {
                        let ch = s.code(close);
                        if matches!(ch, 10 | 13) {
                            break;
                        }
                        if ch == 62 && s.code(close + 1) == 62 {
                            found = true;
                            break;
                        }
                        close += 1;
                    }
                    if found && !crate::util::js::trim(&s.slice(i, close)).is_empty() {
                        out.push(Token { kind: STEREOTYPE_TEXT, start: i, end: close });
                        kind = STEREOTYPE_END;
                        i = close;
                        end = close + 2;
                    } else {
                        while close < n && !matches!(s.code(close), 10 | 13) {
                            close += 1;
                        }
                        if close == i {
                            while close < n && matches!(s.code(close), 10 | 13) {
                                close += 1;
                            }
                            return Err(error(s, i, close - i));
                        }
                        kind = UNCLOSED_STEREOTYPE_TEXT;
                        end = close;
                    }
                }
            } else if s.code(end) == 45 && s.code(end + 1) == 45 {
                kind = BACKWARD_SOLID;
                end += 2;
                while s.code(end) == 45 {
                    end += 1;
                }
            } else {
                kind = LABEL_PUNCTUATION;
            }
        } else if c == 58 {
            if s.code(end) == 58 && s.code(end + 1) == 58 {
                kind = CLASS_SEPARATOR;
                end += 2;
            } else {
                kind = COLON;
            }
        } else if c == 64 {
            if s.code(end) == 123 {
                kind = METADATA_START;
                end += 1;
            } else {
                kind = AT;
            }
        } else if c == 123 {
            kind = LBRACE;
        } else if c == 125 {
            kind = RBRACE;
        } else if c == 91 {
            kind = LBRACKET;
        } else if c == 93 {
            kind = RBRACKET;
        } else if c == 40 {
            kind = LPAREN;
        } else if c == 41 {
            kind = RPAREN;
        } else if c == 44 {
            kind = COMMA;
        } else if c == 35 {
            while hex(s.code(end)) {
                end += 1;
            }
            kind = if end > i + 1 { HASH_COLOR } else { CSS_PUNCTUATION };
        } else if c == 92 {
            if s.code(end) == 44 {
                kind = CSS_ESCAPED_COMMA;
                end += 1;
            } else {
                kind = LABEL_PUNCTUATION;
            }
        } else if c < 127 && c > 32 {
            kind = if b"!#$&*+/=?^|~".contains(&(c as u8)) { CSS_PUNCTUATION } else { LABEL_PUNCTUATION };
        } else {
            kind = LABEL_SYMBOL;
            while end < n {
                let ch = s.code(end);
                if (32..127).contains(&ch) || matches!(ch, 9 | 10 | 13) {
                    break;
                }
                end += 1;
            }
        }
        out.push(Token { kind, start: i, end });
        i = end;
        line_start = fresh;
    }
    for _ in 0..3 {
        out.push(Token { kind: EOF, start: n, end: n });
    }
    Ok(out)
}
