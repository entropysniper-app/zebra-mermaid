//! Substring search for short needles in short text.
//!
//! `str::find(&str)` builds a two-way searcher on every call, which costs more than the search
//! itself when the text is a label or an attribute of a few bytes. For such text we search
//! for the needle's first byte and compare the remainder in place. Longer text uses memchr's
//! substring search. Both return exactly what `str::find` and `str::contains` return: the
//! first byte of a nonempty valid UTF-8 needle cannot be a continuation byte, so a complete
//! match in valid UTF-8 still starts on a character boundary.

/// Text this long or longer uses memchr substring search.
const LONG: usize = 96;

/// The byte offset of the first occurrence of `needle` in `hay`.
#[inline]
pub fn find(hay: &str, needle: &str) -> Option<usize> {
    let n = needle.as_bytes();
    let Some(&first) = n.first() else { return Some(0) };
    let h = hay.as_bytes();
    if h.len() < n.len() {
        return None;
    }
    if h.len() >= LONG {
        return memchr::memmem::find(h, n);
    }
    if n.len() == 1 {
        return memchr::memchr(first, h);
    }
    // Only positions with enough remaining bytes can contain a complete match.
    // All slicing here is on bytes; partial UTF-8 prefixes are never returned.
    let candidates = &h[..h.len() - n.len() + 1];
    let mut from = 0;
    while let Some(k) = memchr::memchr(first, &candidates[from..]) {
        let at = from + k;
        if h[at + 1..at + n.len()] == n[1..] {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// Whether `hay` contains `needle`.
#[inline]
pub fn contains(hay: &str, needle: &str) -> bool {
    find(hay, needle).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_unicode_search_matches_standard_at_length_boundaries() {
        let mut state = 0x5e17f1_u32;
        let mut next = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            state as usize
        };
        let alphabet = ['a', 'b', ':', '<', '\0', 'é', 'ê', '\u{80}', '\u{7ff}', '中', '\u{d7ff}', '\u{e000}', '😀', '\u{10ffff}'];
        for index in 0..4096 {
            let target = [0, 1, 2, 3, 7, 31, 95, 96, 97, 128][index % 10];
            let mut hay = String::new();
            while hay.len() < target {
                hay.push(alphabet[next() % alphabet.len()]);
            }
            let mut boundaries: Vec<_> = hay.char_indices().map(|(i, _)| i).collect();
            boundaries.push(hay.len());
            let start = next() % boundaries.len();
            let end = start + next() % (boundaries.len() - start);
            let selected = &hay[boundaries[start]..boundaries[end]];
            let absent = format!("{selected}missing");
            for needle in ["", "a", "é", "ê", "中😀", "\0", selected, absent.as_str()] {
                assert_eq!(find(&hay, needle), hay.find(needle), "case {index}, {hay:?}, {needle:?}");
                assert_eq!(contains(&hay, needle), hay.contains(needle), "case {index}");
            }
        }
    }

    #[test]
    fn agrees_with_str_find() {
        let cases = [
            ("", ""),
            ("abc", ""),
            ("", "a"),
            ("abc", "abc"),
            ("abc", "abcd"),
            ("aab", "ab"),
            ("xx:fa-car", ":fa-"),
            ("中文¶ß后", "¶ß"),
            ("¶x¶ß", "¶ß"),
            ("ﬂ°quot¶ß", "ﬂ°"),
            ("<rect x=\"1\"/><rectx/>", "<rect"),
            ("var(--_w) var(--_tw)", "var(--_tw)"),
            ("a--b---c", "---"),
            ("%%{init}%%", "%%{"),
            ("a long attribute list with many spaces ........................................ rx=\"v\" end", " rx=\"v\""),
            ("a long attribute list with many spaces ........................................ rx=\"v\" end", " rz=\"v\""),
        ];
        for (hay, needle) in cases {
            assert_eq!(find(hay, needle), hay.find(needle), "{hay:?} {needle:?}");
            assert_eq!(contains(hay, needle), hay.contains(needle), "{hay:?} {needle:?}");
        }
    }

    #[test]
    fn long_unicode_and_partial_prefixes_match_standard_search() {
        for count in [0, 1, 31, 95, 96, 97, 256, 1024] {
            for prefix in ["a", "中", "😀", "var(--_", "<rect"] {
                let hay = format!("{}{}", prefix.repeat(count), "中文😀var(--_hw)<rect rx=\"v\"/>");
                for needle in ["", "a", "中", "😀", "文😀", "var(--_h", "var(--_hw)", "var(--_w)", "<rect", "rectx", "rx=\"v\""] {
                    assert_eq!(find(&hay, needle), hay.find(needle), "count={count}, prefix={prefix}, needle={needle}");
                }
            }
        }
    }
}
