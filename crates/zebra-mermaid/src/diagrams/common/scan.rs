//! Lexer rules that Mermaid's journey and timeline grammars have in common, ported from the
//! compiled Jison rules: comments, whitespace, and the accTitle and accDescr statements.
//! Ported from Pele `src/diagrams/common/scan.ts`. Offsets are bytes.

use crate::util::chars::{char_at, is_space};

#[derive(Debug, Clone, Default)]
pub struct Tokens {
    pub types: Vec<usize>,
    pub texts: Vec<String>,
    pub starts: Vec<usize>,
}

pub fn line_end(src: &str, from: usize) -> usize {
    src[from.min(src.len())..].find('\n').map_or(src.len(), |k| from + k)
}

/// Whether `word` (lower-case ASCII letters) is at `p`; the grammars are case-insensitive.
pub fn keyword_at(src: &str, p: usize, word: &str) -> bool {
    let s = src.as_bytes();
    word.bytes().enumerate().all(|(i, w)| s.get(p + i).is_some_and(|&c| (c | 32) == w))
}

fn byte(src: &str, p: usize) -> Option<u8> {
    src.as_bytes().get(p).copied()
}

/// Skips comments and white space. Stops at a line break that starts a token, which is a
/// NEWLINE. Jison compiles `\%%` to a single `%`, so one percent sign starts a comment, and the
/// second comment rule takes any character before `%%` with it, a line break included.
pub fn skip_trivia(src: &str, from: usize) -> usize {
    let n = src.len();
    let mut p = from;
    while p < n {
        let c = src.as_bytes()[p];
        if c == b'%' && byte(src, p + 1) != Some(b'{') {
            p = line_end(src, p);
        } else if let Some(after) = percent_pair_after(src, p) {
            p = line_end(src, after);
        } else if c == b'\n' {
            return p;
        } else if char_at(src, p).is_some_and(is_space) {
            while let Some(ch) = char_at(src, p) {
                if !is_space(ch) {
                    break;
                }
                p += ch.len_utf8();
            }
        } else if c == b'#' {
            p = line_end(src, p);
        } else {
            return p;
        }
    }
    p
}

/// `[^}]%%` at `p`, with the character before the percent signs one UTF-16 unit: where it
/// ends, else `None`. A character outside the BMP is two units in JavaScript, so `%%` after it
/// is not one unit later and this rule does not apply.
fn percent_pair_after(src: &str, p: usize) -> Option<usize> {
    let c = char_at(src, p)?;
    if c == '}' || c.len_utf16() != 1 {
        return None;
    }
    let q = p + c.len_utf8();
    (byte(src, q) == Some(b'%') && byte(src, q + 1) == Some(b'%')).then_some(q + 2)
}

fn skip_space(src: &str, from: usize) -> usize {
    let mut p = from;
    while let Some(ch) = char_at(src, p) {
        if !is_space(ch) {
            break;
        }
        p += ch.len_utf8();
    }
    p
}

/// Lexes an accTitle or accDescr statement at `p` and returns where it ends, or `p` if there
/// is none. The token types acc_title, acc_title_value, acc_descr, acc_descr_value and
/// acc_descr_multiline_value must be numbered consecutively from `first`.
pub fn accessibility(src: &str, p: usize, out: &mut Tokens, first: usize) -> usize {
    let descr = keyword_at(src, p, "accdescr");
    if !descr && !keyword_at(src, p, "acctitle") {
        return p;
    }
    let n = src.len();
    let mut q = skip_space(src, p + 8);
    let c = byte(src, q);
    let mut emit = |t: usize, s: usize, e: usize| {
        out.types.push(t);
        out.texts.push(src[s..e].to_string());
        out.starts.push(s);
    };
    if c == Some(b':') {
        q = skip_space(src, q + 1);
        let end = line_end(src, q);
        emit(if descr { first + 2 } else { first }, p, q);
        emit(if descr { first + 3 } else { first + 1 }, q, end);
        return end;
    }
    if !descr || c != Some(b'{') {
        return p;
    }
    q = skip_space(src, q + 1);
    while byte(src, q) != Some(b'}') {
        let end = src[q..].find('}').map_or(n, |k| q + k);
        emit(first + 4, q, end);
        if q == n {
            return n;
        }
        q = end;
    }
    q + 1
}
