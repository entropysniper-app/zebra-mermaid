//! The sequence lexer. Ported from Pele `src/diagrams/sequence/lexer.ts`.
//!
//! Pele reads the text in UTF-16 code units with `charCodeAt`, and several rules look one or
//! two units ahead of a character that may not be ASCII. The lexer here works on the same
//! code units, so every look-ahead and every position is the one Pele uses. Token boundaries
//! only ever fall on ASCII delimiters or white space, never inside a surrogate pair, so each
//! token's text is a whole string again.

use super::tokens::*;
use crate::util::js;

pub(super) struct Tokens {
    pub types: Vec<u8>,
    pub texts: Vec<String>,
    /// Where each token starts, in UTF-16 code units.
    pub starts: Vec<usize>,
}

// Lexer states.
const S_INITIAL: u8 = 0;
const S_ID: u8 = 1;
const S_ALIAS: u8 = 2;
const S_LINE: u8 = 3;
const S_CONFIG: u8 = 4;
const S_ACC_TITLE: u8 = 5;
const S_ACC_DESCR: u8 = 6;
const S_ACC_DESCR_MULTILINE: u8 = 7;

// Character classes of Mermaid's lexer rules, for ASCII.
const C_ACTOR_START: u8 = 1;
const C_ACTOR_REST: u8 = 2;
const C_ID: u8 = 4;
const C_ID_WIDE: u8 = 8;
const C_LINE: u8 = 16;
const C_MENU: u8 = 32;

const fn contains(set: &[u8], c: u8) -> bool {
    let mut i = 0;
    while i < set.len() {
        if set[i] == c {
            return true;
        }
        i += 1;
    }
    false
}

const CLASS: [u8; 128] = {
    let mut table = [0u8; 128];
    let mut c = 0;
    while c < 128 {
        let ch = c as u8;
        let ws = c == 32 || (c >= 9 && c <= 13);
        let mut bits = 0u8;
        if !contains(b"/\\+()<->:\n,;", ch) {
            bits |= C_ACTOR_START;
        }
        if !contains(b"+<->:\n,;", ch) {
            bits |= C_ACTOR_REST;
        }
        if !contains(b"<>:\n,;@", ch) {
            bits |= C_ID_WIDE | if ws { 0 } else { C_ID };
        }
        if !contains(b"#\n;", ch) {
            bits |= C_LINE;
        }
        if !contains(b"/\\+()<>:\n,;-", ch) && !ws {
            bits |= C_MENU;
        }
        table[c] = bits;
        c += 1;
    }
    table
};

/// Pele's `isSpace` on a code unit; -1 stands for the NaN past the end.
#[inline]
fn is_space(c: i32) -> bool {
    if c < 128 {
        return c == 32 || (9..=13).contains(&c);
    }
    c == 0xa0
        || c == 0x1680
        || (0x2000..=0x200a).contains(&c)
        || c == 0x2028
        || c == 0x2029
        || c == 0x202f
        || c == 0x205f
        || c == 0x3000
        || c == 0xfeff
}

#[inline]
fn is_digit(c: i32) -> bool {
    (48..=57).contains(&c)
}

#[inline]
fn is_word(c: i32) -> bool {
    (48..=57).contains(&c) || (65..=90).contains(&c) || (97..=122).contains(&c) || c == 95
}

struct Lexer<'a> {
    src: &'a [u16],
    n: usize,
    types: Vec<u8>,
    texts: Vec<String>,
    starts: Vec<usize>,
    stack: Vec<u8>,
    state: u8,
    p: usize,
    next_nl: i64,
}

impl<'a> Lexer<'a> {
    /// `src.charCodeAt(q)`, with -1 for NaN past the end. No comparison below tells the two
    /// apart: -1 is no character, and `(-1 | 32)` is no lowercase letter, as `(NaN | 32)` is not.
    #[inline]
    fn at(&self, q: usize) -> i32 {
        if q < self.n {
            self.src[q] as i32
        } else {
            -1
        }
    }

    fn slice(&self, s: usize, e: usize) -> String {
        String::from_utf16_lossy(&self.src[s.min(self.n)..e.min(self.n).max(s.min(self.n))])
    }

    fn emit(&mut self, kind: u8, s: usize, e: usize) {
        let text = self.slice(s, e);
        self.types.push(kind);
        self.texts.push(text);
        self.starts.push(s);
        self.p = e;
    }

    fn emit_text(&mut self, kind: u8, s: usize, e: usize, text: String) {
        self.types.push(kind);
        self.texts.push(text);
        self.starts.push(s);
        self.p = e;
    }

    fn push(&mut self, s: u8) {
        self.stack.push(self.state);
        self.state = s;
    }

    fn pop(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.state = s;
        }
    }

    fn eol(&mut self, q: usize) -> usize {
        if self.next_nl < q as i64 {
            self.next_nl = match self.src[q.min(self.n)..].iter().position(|&c| c == 10) {
                Some(k) => (q + k) as i64,
                None => self.n as i64,
            };
        }
        self.next_nl as usize
    }

    #[inline]
    fn has(&self, q: usize, bit: u8) -> bool {
        let c = self.at(q);
        if (0..128).contains(&c) {
            return CLASS[c as usize] & bit != 0;
        }
        if bit == C_ID || bit == C_MENU {
            !is_space(c)
        } else {
            true
        }
    }

    /// Matches a lowercase literal without regard to the case of the source.
    fn ci(&self, q: usize, word: &str) -> bool {
        for (k, &w) in word.as_bytes().iter().enumerate() {
            let w = w as i32;
            let c = self.at(q + k);
            if (97..=122).contains(&w) {
                if (c | 32) != w {
                    return false;
                }
            } else if c != w {
                return false;
            }
        }
        true
    }

    fn word(&self, lit: &str) -> bool {
        self.ci(self.p, lit) && !is_word(self.at(self.p + lit.len()))
    }

    fn skip_ws(&self, mut q: usize) -> usize {
        while q < self.n && is_space(self.at(q)) {
            q += 1;
        }
        q
    }

    fn skip_same_line(&mut self) -> bool {
        let mut q = self.p;
        while q < self.n {
            let c = self.at(q);
            if c == 10 || !is_space(c) {
                break;
            }
            q += 1;
        }
        if q == self.p {
            return false;
        }
        self.p = q;
        true
    }

    fn line_run(&self, mut q: usize) -> usize {
        while q < self.n && self.has(q, C_LINE) {
            q += 1;
        }
        q
    }

    fn begin(&mut self, kind: u8, len: usize, next: u8) {
        self.emit(kind, self.p, self.p + len);
        self.push(next);
    }

    fn keyword(&mut self, lit: &str, kind: u8, next: Option<u8>) -> bool {
        if !self.word(lit) {
            return false;
        }
        self.emit(kind, self.p, self.p + lit.len());
        if let Some(next) = next {
            self.push(next);
        }
        true
    }

    /// The actor menu keywords count only when spaces and the start of a name follow.
    fn menu(&mut self, lit: &str, kind: u8) -> bool {
        if !self.ci(self.p, lit) {
            return false;
        }
        let mut q = self.p + lit.len();
        let mut c = self.at(q);
        if c != 32 && c != 9 {
            return false;
        }
        while c == 32 || c == 9 {
            q += 1;
            c = self.at(q);
        }
        if q >= self.n || !self.has(q, C_MENU) {
            return false;
        }
        self.emit(kind, self.p, self.p + lit.len());
        true
    }

    fn arrow_ahead(&self, q: usize) -> bool {
        let c = self.at(q);
        let d = self.at(q + 1);
        if c == 45 {
            if d == 120 || d == 88 || d == 41 || d == 92 || d == 47 || d == 45 {
                return true;
            }
            let e = self.at(q + 2);
            return d == 124 && (e == 92 || e == 47);
        }
        if c == 47 || c == 92 {
            return (d == 124 || d == c) && self.at(q + 2) == 45;
        }
        c == 40 && d == 41
    }

    fn actor(&mut self) {
        let p = self.p;
        let mut q = p + 1;
        while q < self.n && self.has(q, C_ACTOR_START) {
            q += 1;
        }
        while q < self.n && !self.arrow_ahead(q) {
            let mut r = q;
            while self.at(r) == 45 {
                r += 1;
            }
            if r >= self.n || !self.has(r, C_ACTOR_REST) {
                break;
            }
            r += 1;
            while r < self.n && self.has(r, C_ACTOR_REST) {
                r += 1;
            }
            q = r;
        }
        let text = js::trim(&self.slice(p, q)).to_string();
        self.emit_text(ACTOR, p, q, text);
    }

    fn number(&mut self) -> bool {
        let p = self.p;
        let mut q = p;
        while is_digit(self.at(q)) {
            q += 1;
        }
        let whole = q > p;
        if self.at(q) == 46 && is_digit(self.at(q + 1)) {
            let r = if is_digit(self.at(q + 2)) { q + 3 } else { q + 2 };
            let after = self.at(r);
            if after == 32 || after == 10 {
                self.emit(NUM, p, r);
                return true;
            }
        }
        let after = self.at(q);
        if whole && (after == 32 || after == 10) {
            self.emit(NUM, p, q);
            return true;
        }
        false
    }

    fn dash(&mut self) {
        let p = self.p;
        let d = self.at(p + 1);
        let e = self.at(p + 2);
        if d == 62 {
            if e == 62 {
                self.emit(SOLID_ARROW, p, p + 3);
            } else {
                self.emit(SOLID_OPEN_ARROW, p, p + 2);
            }
        } else if d == 120 || d == 88 {
            self.emit(SOLID_CROSS, p, p + 2);
        } else if d == 41 {
            self.emit(SOLID_POINT, p, p + 2);
        } else if d == 124 && e == 92 {
            self.emit(SOLID_ARROW_TOP, p, p + 3);
        } else if d == 124 && e == 47 {
            self.emit(SOLID_ARROW_BOTTOM, p, p + 3);
        } else if d == 92 && e == 92 {
            self.emit(STICK_ARROW_TOP, p, p + 3);
        } else if d == 47 && e == 47 {
            self.emit(STICK_ARROW_BOTTOM, p, p + 3);
        } else if d == 45 {
            let f = self.at(p + 3);
            if e == 62 {
                if f == 62 {
                    self.emit(DOTTED_ARROW, p, p + 4);
                } else {
                    self.emit(DOTTED_OPEN_ARROW, p, p + 3);
                }
            } else if e == 120 || e == 88 {
                self.emit(DOTTED_CROSS, p, p + 3);
            } else if e == 41 {
                self.emit(DOTTED_POINT, p, p + 3);
            } else if e == 124 && f == 92 {
                self.emit(SOLID_ARROW_TOP_DOTTED, p, p + 4);
            } else if e == 124 && f == 47 {
                self.emit(SOLID_ARROW_BOTTOM_DOTTED, p, p + 4);
            } else if e == 92 && f == 92 {
                self.emit(STICK_ARROW_TOP_DOTTED, p, p + 4);
            } else if e == 47 && f == 47 {
                self.emit(STICK_ARROW_BOTTOM_DOTTED, p, p + 4);
            } else {
                self.emit(MINUS, p, p + 1);
            }
        } else {
            self.emit(MINUS, p, p + 1);
        }
    }

    fn reverse(&mut self, c: i32) -> bool {
        let p = self.p;
        let d = self.at(p + 1);
        if (d != 124 && d != c) || self.at(p + 2) != 45 {
            return false;
        }
        let dotted = self.at(p + 3) == 45;
        let top = c == 47;
        let kind = if d == 124 {
            match (dotted, top) {
                (true, true) => SOLID_ARROW_TOP_REVERSE_DOTTED,
                (true, false) => SOLID_ARROW_BOTTOM_REVERSE_DOTTED,
                (false, true) => SOLID_ARROW_TOP_REVERSE,
                (false, false) => SOLID_ARROW_BOTTOM_REVERSE,
            }
        } else {
            match (dotted, top) {
                (true, true) => STICK_ARROW_TOP_REVERSE_DOTTED,
                (true, false) => STICK_ARROW_BOTTOM_REVERSE_DOTTED,
                (false, true) => STICK_ARROW_TOP_REVERSE,
                (false, false) => STICK_ARROW_BOTTOM_REVERSE,
            }
        };
        self.emit(kind, p, p + if dotted { 4 } else { 3 });
        true
    }

    fn title(&mut self) -> bool {
        if !self.ci(self.p, "title") {
            return false;
        }
        let mut q = self.p + 5;
        let legacy = self.at(q) == 58;
        if legacy {
            q += 1;
        }
        if !is_space(self.at(q)) || q + 1 >= self.n || !self.has(q + 1, C_LINE) {
            return false;
        }
        let end = self.line_run(q + 1);
        self.emit(if legacy { LEGACY_TITLE } else { TITLE }, self.p, end);
        true
    }

    fn acc(&mut self) -> bool {
        let descr = self.ci(self.p, "accdescr");
        if !descr && !self.ci(self.p, "acctitle") {
            return false;
        }
        let q = self.skip_ws(self.p + 8);
        let c = self.at(q);
        if c == 58 {
            let end = self.skip_ws(q + 1);
            self.emit(if descr { ACC_DESCR } else { ACC_TITLE }, self.p, end);
            self.push(if descr { S_ACC_DESCR } else { S_ACC_TITLE });
            return true;
        }
        if descr && c == 123 {
            self.p = self.skip_ws(q + 1);
            self.push(S_ACC_DESCR_MULTILINE);
            return true;
        }
        false
    }

    fn keywords(&mut self, c: i32) -> bool {
        match c | 32 {
            97 => {
                self.keyword("actor", PARTICIPANT_ACTOR, Some(S_ID))
                    || self.keyword("alt", ALT, Some(S_LINE))
                    || self.keyword("and", AND, Some(S_LINE))
                    || self.keyword("activate", ACTIVATE, Some(S_ID))
                    || self.acc()
                    || self.keyword("autonumber", AUTONUMBER, None)
            }
            98 => self.keyword("box", BOX, Some(S_LINE)) || self.keyword("break", BREAK, Some(S_LINE)),
            99 => self.keyword("create", CREATE, None) || self.keyword("critical", CRITICAL, Some(S_LINE)),
            100 => {
                self.keyword("destroy", DESTROY, Some(S_ID))
                    || self.menu("details", DETAILS)
                    || self.keyword("deactivate", DEACTIVATE, Some(S_ID))
            }
            101 => self.keyword("else", ELSE, Some(S_LINE)) || self.keyword("end", KW_END, None),
            108 => {
                self.keyword("loop", LOOP, Some(S_LINE))
                    || self.keyword("left of", LEFT_OF, None)
                    || self.menu("links", LINKS)
                    || self.menu("link", LINK)
            }
            110 => self.keyword("note", NOTE, None),
            111 => {
                self.keyword("opt", OPT, Some(S_LINE))
                    || self.keyword("option", OPTION, Some(S_LINE))
                    || self.keyword("over", OVER, None)
                    || self.keyword("off", OFF, None)
            }
            112 => {
                self.keyword("participant", PARTICIPANT, Some(S_ID))
                    || self.keyword("par", PAR, Some(S_LINE))
                    || self.keyword("par_over", PAR_OVER, Some(S_LINE))
                    || self.menu("properties", PROPERTIES)
            }
            114 => self.keyword("rect", RECT, Some(S_LINE)) || self.keyword("right of", RIGHT_OF, None),
            115 => self.keyword("sequencediagram", SD, None),
            116 => self.title(),
            _ => false,
        }
    }

    fn initial(&mut self) {
        let p = self.p;
        let c = self.at(p);
        if c == 10 {
            let mut q = p + 1;
            while self.at(q) == 10 {
                q += 1;
            }
            self.emit(NEWLINE, p, q);
            return;
        }
        if is_space(c) {
            self.p = self.skip_ws(p + 1);
            return;
        }
        if c == 35 || (c == 37 && self.at(p + 1) != 123) {
            self.p = self.eol(p);
            return;
        }
        if c != 125 && self.at(p + 1) == 37 && self.at(p + 2) == 37 {
            self.p = self.eol(p);
            return;
        }
        if (is_digit(c) || c == 46) && self.number() {
            return;
        }
        if c < 128 && (97..=122).contains(&(c | 32)) && self.keywords(c) {
            return;
        }
        if c == 44 {
            self.emit(COMMA, p, p + 1);
        } else if c == 59 {
            self.emit(NEWLINE, p, p + 1);
        } else if self.has(p, C_ACTOR_START) {
            self.actor();
        } else if c == 45 {
            self.dash();
        } else if c == 60 && self.starts_with(p, "<<->>") {
            self.emit(BIDIRECTIONAL_SOLID_ARROW, p, p + 5);
        } else if c == 60 && self.starts_with(p, "<<-->>") {
            self.emit(BIDIRECTIONAL_DOTTED_ARROW, p, p + 6);
        } else if (c == 47 || c == 92) && self.reverse(c) {
        } else if c == 58 {
            let end = self.line_run(p + 1);
            self.emit(TXT, p, end);
        } else if c == 43 {
            self.emit(PLUS, p, p + 1);
        } else if c == 40 && self.at(p + 1) == 41 {
            self.emit(CENTRAL, p, p + 2);
        } else {
            self.emit(INVALID, p, p + 1);
        }
    }

    fn starts_with(&self, q: usize, lit: &str) -> bool {
        lit.bytes().enumerate().all(|(k, b)| self.at(q + k) == b as i32)
    }

    /// A participant name, up to an alias, a config object, or the end of the statement.
    fn id(&mut self) -> bool {
        if self.skip_same_line() {
            return true;
        }
        let p = self.p;
        let c = self.at(p);
        if c == 35 {
            self.p = self.eol(p);
            return true;
        }
        if c == 64 && self.at(p + 1) == 123 {
            self.begin(CONFIG_START, 2, S_CONFIG);
            return true;
        }
        let mut a = p;
        while a < self.n && self.has(a, C_ID) {
            a += 1;
        }
        if a > p {
            let w = self.skip_ws(a);
            if self.at(w) == 64 && self.at(w + 1) == 123 {
                self.emit(ACTOR, p, a);
                return true;
            }
            if w > a && self.ci(w, "as") && is_space(self.at(w + 2)) {
                self.emit(ACTOR, p, a);
                self.push(S_ALIAS);
                return true;
            }
        }
        let mut b = a;
        while b < self.n && self.has(b, C_ID_WIDE) {
            b += 1;
        }
        if b > p {
            let end = self.at(b);
            // Before any other delimiter the name must be followed by a comment, so it ends at
            // the last one.
            let mut cut = b;
            if b < self.n && end != 10 && end != 59 {
                cut -= 1;
                while cut > p && self.at(cut) != 35 {
                    cut -= 1;
                }
            }
            if cut > p {
                let text = js::trim(&self.slice(p, cut)).to_string();
                self.emit_text(ACTOR, p, cut, text);
                self.pop();
                return true;
            }
        }
        if self.at(b) == 60 {
            let e = self.eol(p);
            self.emit(INVALID, p, e);
            self.pop();
            return true;
        }
        if c != 10 {
            let e = self.eol(p);
            let text = js::trim(&self.slice(p, e)).to_string();
            self.emit_text(INVALID, p, e, text);
            self.pop();
            return true;
        }
        false
    }

    fn step(&mut self) -> bool {
        match self.state {
            S_INITIAL => {
                self.initial();
                true
            }
            S_ID => self.id(),
            S_ALIAS => {
                if self.skip_same_line() {
                    return true;
                }
                if self.at(self.p) == 35 {
                    self.p = self.eol(self.p);
                    return true;
                }
                self.pop();
                self.pop();
                if self.word("as") {
                    self.begin(AS, 2, S_LINE);
                } else {
                    self.emit(NEWLINE, self.p, self.p);
                }
                true
            }
            S_LINE => {
                if self.skip_same_line() {
                    return true;
                }
                if self.at(self.p) == 35 {
                    self.p = self.eol(self.p);
                    return true;
                }
                let end = self.line_run(self.p);
                self.emit(REST_OF_LINE, self.p, end);
                self.pop();
                true
            }
            S_CONFIG => {
                let p = self.p;
                if self.at(p) != 125 {
                    let q = self.src[p..].iter().position(|&c| c == 125).map_or(self.n, |k| p + k);
                    self.emit(CONFIG_CONTENT, p, q);
                } else {
                    let w = self.skip_ws(p + 1);
                    let alias = w > p + 1 && self.ci(w, "as") && is_space(self.at(w + 2));
                    self.emit(CONFIG_END, p, p + 1);
                    self.pop();
                    if alias {
                        self.push(S_ALIAS);
                    } else {
                        self.pop();
                    }
                }
                true
            }
            S_ACC_TITLE | S_ACC_DESCR => {
                let kind = if self.state == S_ACC_TITLE { ACC_TITLE_VALUE } else { ACC_DESCR_VALUE };
                let e = self.eol(self.p);
                self.emit(kind, self.p, e);
                self.pop();
                true
            }
            _ => {
                let p = self.p;
                if self.at(p) == 125 {
                    self.p += 1;
                    self.pop();
                } else {
                    let q = self.src[p..].iter().position(|&c| c == 125).map_or(self.n, |k| p + k);
                    self.emit(ACC_DESCR_MULTILINE_VALUE, p, q);
                }
                true
            }
        }
    }
}

pub(super) fn tokenize(src: &[u16]) -> Tokens {
    let n = src.len();
    let mut lx =
        Lexer { src, n, types: Vec::new(), texts: Vec::new(), starts: Vec::new(), stack: Vec::new(), state: S_INITIAL, p: 0, next_nl: -1 };
    while lx.p < n {
        if !lx.step() {
            lx.types.push(ERROR);
            lx.texts.push(String::new());
            lx.starts.push(lx.p);
            return Tokens { types: lx.types, texts: lx.texts, starts: lx.starts };
        }
    }
    // Mermaid's lexer tries its rules once more on the empty remainder before reporting the end.
    match lx.state {
        S_INITIAL | S_ALIAS => lx.emit(NEWLINE, n, n),
        S_LINE => lx.emit(REST_OF_LINE, n, n),
        S_ACC_TITLE => lx.emit(ACC_TITLE_VALUE, n, n),
        S_ACC_DESCR => lx.emit(ACC_DESCR_VALUE, n, n),
        S_ACC_DESCR_MULTILINE => lx.emit(ACC_DESCR_MULTILINE_VALUE, n, n),
        _ => {}
    }
    lx.types.push(END);
    lx.texts.push(String::new());
    lx.starts.push(n);
    Tokens { types: lx.types, texts: lx.texts, starts: lx.starts }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(u8, String)> {
        let units: Vec<u16> = src.encode_utf16().collect();
        let t = tokenize(&units);
        t.types.into_iter().zip(t.texts).collect()
    }

    #[test]
    fn classes_match_pele() {
        assert_eq!(CLASS[b'a' as usize], C_ACTOR_START | C_ACTOR_REST | C_ID | C_ID_WIDE | C_LINE | C_MENU);
        assert_eq!(CLASS[b' ' as usize], C_ACTOR_START | C_ACTOR_REST | C_ID_WIDE | C_LINE);
        assert_eq!(CLASS[b'-' as usize], C_ID | C_ID_WIDE | C_LINE);
        assert_eq!(CLASS[b'#' as usize], C_ACTOR_START | C_ACTOR_REST | C_ID | C_ID_WIDE | C_MENU);
        assert_eq!(CLASS[b'@' as usize], C_ACTOR_START | C_ACTOR_REST | C_LINE | C_MENU);
    }

    #[test]
    fn messages_and_arrows() {
        let t = kinds("sequenceDiagram\nAlice->>John: Hi\nA-)B: x\nA<<-->>B:y\n");
        let types: Vec<u8> = t.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            types,
            vec![
                SD,
                NEWLINE,
                ACTOR,
                SOLID_ARROW,
                ACTOR,
                TXT,
                NEWLINE,
                ACTOR,
                SOLID_POINT,
                ACTOR,
                TXT,
                NEWLINE,
                ACTOR,
                BIDIRECTIONAL_DOTTED_ARROW,
                ACTOR,
                TXT,
                NEWLINE,
                NEWLINE,
                END
            ]
        );
        assert_eq!(t[2].1, "Alice");
        assert_eq!(t[5].1, ": Hi");
    }

    #[test]
    fn names_with_dashes_and_non_ascii() {
        let t = kinds("a-b-c->>中文😀 x: y\n");
        assert_eq!(t[0], (ACTOR, "a-b-c".to_string()));
        assert_eq!(t[1].0, SOLID_ARROW);
        assert_eq!(t[2], (ACTOR, "中文😀 x".to_string()));
        // A comment marker after a non-ASCII character starts a comment, as it does in Pele,
        // which looks one code unit ahead.
        let t = kinds("é%% c\n");
        assert_eq!(t[0].0, NEWLINE);
    }

    #[test]
    fn participant_alias_and_config() {
        let t = kinds("participant A as The A\nparticipant B@{ \"type\": \"database\" } as Bee\n");
        let types: Vec<u8> = t.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            types,
            vec![
                PARTICIPANT,
                ACTOR,
                AS,
                REST_OF_LINE,
                NEWLINE,
                PARTICIPANT,
                ACTOR,
                CONFIG_START,
                CONFIG_CONTENT,
                CONFIG_END,
                AS,
                REST_OF_LINE,
                NEWLINE,
                NEWLINE,
                END
            ]
        );
        assert_eq!(t[3].1, "The A");
        assert_eq!(t[8].1, " \"type\": \"database\" ");
    }
}
