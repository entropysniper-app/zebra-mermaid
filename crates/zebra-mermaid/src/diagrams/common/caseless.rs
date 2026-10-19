//! Ported from Pele `src/diagrams/common/caseless.ts`.

/// Whether `word`, written in lower case, is at byte `at` of `src`. Only ASCII letters match in
/// either case, as with the `i` flag on a regular expression without the `u` flag.
pub fn word_at(src: &str, at: usize, word: &str) -> bool {
    let s = src.as_bytes();
    for (k, &w) in word.as_bytes().iter().enumerate() {
        let Some(&c) = s.get(at + k) else { return false };
        if c != w && !(w.is_ascii_lowercase() && c == w - 32) {
            return false;
        }
    }
    true
}
