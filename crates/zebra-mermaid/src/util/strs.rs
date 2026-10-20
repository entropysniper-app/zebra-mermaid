//! Substring search for short needles in short text.
//!
//! `str::find(&str)` builds a two-way searcher on every call, which costs more than the search
//! itself when the text is a label or an attribute of a few bytes. For such text these scan
//! for the needle's first character with the standard library's byte search and compare the
//! rest in place. Longer text uses the byte-search implementation in memchr, which skips ahead instead of
//! stopping at every occurrence of a common first character. Both return exactly what
//! `str::find` and `str::contains` return: every match of a valid UTF-8 needle in valid UTF-8
//! text starts on a character boundary, so searching by the first character finds the same
//! first match.

/// Text this long or longer is searched with the standard searcher.
const LONG: usize = 96;

/// The byte offset of the first occurrence of `needle` in `hay`.
#[inline]
pub fn find(hay: &str, needle: &str) -> Option<usize> {
    let n = needle.as_bytes();
    let Some(first) = needle.chars().next() else { return Some(0) };
    let h = hay.as_bytes();
    if h.len() < n.len() {
        return None;
    }
    if h.len() >= LONG {
        return memchr::memmem::find(h, n);
    }
    let head = first.len_utf8();
    let rest = &n[head..];
    let mut from = 0;
    while let Some(k) = hay[from..].find(first) {
        let at = from + k;
        if h.len() - at >= n.len() && &h[at + head..at + n.len()] == rest {
            return Some(at);
        }
        from = at + head;
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
