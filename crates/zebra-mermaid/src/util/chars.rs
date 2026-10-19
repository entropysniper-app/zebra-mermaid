//! Character tests that the ported lexers use. Pele reads UTF-16 code units with
//! `charCodeAt`; these take a `char` (or a byte, for the ASCII-only tests). Every character that
//! these tests accept outside ASCII lies in the Basic Multilingual Plane, so testing a whole
//! character gives the answer the code unit test gives.

/// White space as JavaScript's `\s` and `String.prototype.trim` see it. This is not Rust's
/// `char::is_whitespace`: JavaScript counts U+FEFF and not U+0085.
#[inline]
pub fn is_space(c: char) -> bool {
    let c = c as u32;
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

/// `is_space` for a UTF-16 code unit or code point number; false past the end (`None`).
#[inline]
pub fn is_space_code(c: Option<u32>) -> bool {
    match c.and_then(char::from_u32) {
        Some(ch) => is_space(ch),
        None => false,
    }
}

#[inline]
pub fn is_blank(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// A line terminator: what `.` does not match.
#[inline]
pub fn is_line_end(c: char) -> bool {
    c == '\n' || c == '\r' || c == '\u{2028}' || c == '\u{2029}'
}

#[inline]
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[inline]
pub fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[inline]
pub fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// A character of `\w`: ASCII letters, digits and the underscore.
#[inline]
pub fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[inline]
pub fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[inline]
pub fn is_space_byte(b: u8) -> bool {
    b == b' ' || (9..=13).contains(&b)
}

/// The character that starts at byte `i` of `s`, if `i` is in range and on a boundary.
#[inline]
pub fn char_at(s: &str, i: usize) -> Option<char> {
    s.get(i..).and_then(|rest| rest.chars().next())
}

/// The character that ends at byte `i` of `s`.
#[inline]
pub fn char_before(s: &str, i: usize) -> Option<char> {
    s.get(..i).and_then(|head| head.chars().next_back())
}

/// The byte after the character that starts at `i`.
#[inline]
pub fn next_boundary(s: &str, i: usize) -> usize {
    match char_at(s, i) {
        Some(c) => i + c.len_utf8(),
        None => i + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_matches_javascript() {
        for c in [
            '\t', '\n', '\u{b}', '\u{c}', '\r', ' ', '\u{a0}', '\u{1680}', '\u{2000}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}',
            '\u{205f}', '\u{3000}', '\u{feff}',
        ] {
            assert!(is_space(c), "{:?}", c);
        }
        for c in ['\u{85}', '\u{200b}', 'a', '\u{180e}'] {
            assert!(!is_space(c), "{:?}", c);
        }
    }
}
/// Mermaid's accessibility normalization: `/\n\s+/g` collapses the whole
/// whitespace run after a newline, including additional empty lines.
pub fn normalize_accessibility_description(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '\n' {
            while chars.peek().is_some_and(|&next| is_space(next)) {
                chars.next();
            }
        }
    }
    out
}
