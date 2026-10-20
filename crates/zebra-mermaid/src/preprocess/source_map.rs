//! Maps byte offsets in preprocessed text back to the source the caller gave.
//!
//! Each preprocessing step that changes lengths records where each run of its output came
//! from. A run is either copied (offsets inside it map one to one) or written by the step (every
//! offset inside it maps to the start of the text it replaced). Mapping through the steps in
//! reverse order gives the offset in the original source.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
    out: usize,
    src: usize,
    exact: bool,
}

/// The map of one step, from its output back to its input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepMap {
    // Every completed step has a first mark. Identity and suffix steps need no heap
    // storage; only changes within the text add further marks.
    first: Option<Mark>,
    rest: Vec<Mark>,
    out_len: usize,
    src_len: usize,
}

impl StepMap {
    /// A step that kept the text as it was.
    pub fn identity(len: usize) -> Self {
        StepMap { first: Some(Mark { out: 0, src: 0, exact: true }), rest: Vec::new(), out_len: len, src_len: len }
    }

    /// A step that kept the text from byte `start` of its `len` bytes: what a [`Builder`] that
    /// copies that one run produces.
    pub fn suffix(len: usize, start: usize) -> Self {
        if start >= len {
            return StepMap { first: Some(Mark { out: 0, src: 0, exact: false }), rest: Vec::new(), out_len: 0, src_len: len };
        }
        StepMap { first: Some(Mark { out: 0, src: start, exact: true }), rest: Vec::new(), out_len: len - start, src_len: len }
    }

    pub fn to_src(&self, out: usize) -> usize {
        if out >= self.out_len {
            return self.src_len;
        }
        let m = match self.rest.binary_search_by(|m| m.out.cmp(&out)) {
            Ok(k) => self.rest[k],
            Err(0) => match self.first {
                Some(first) if out >= first.out => first,
                _ => return 0,
            },
            Err(k) => self.rest[k - 1],
        };
        if m.exact {
            (m.src + (out - m.out)).min(self.src_len)
        } else {
            m.src
        }
    }
}

/// Builds the output of a step together with its map.
pub struct Builder<'a> {
    src: &'a str,
    out: String,
    first: Option<Mark>,
    rest: Vec<Mark>,
}

impl<'a> Builder<'a> {
    pub fn new(src: &'a str) -> Self {
        Builder { src, out: String::with_capacity(src.len()), first: None, rest: Vec::new() }
    }

    fn mark(&mut self, src: usize, exact: bool) {
        let out = self.out.len();
        if let Some(last) = self.rest.last_mut().or(self.first.as_mut()) {
            if last.out == out {
                *last = Mark { out, src, exact };
                return;
            }
            // A copied run that continues the last one needs no new mark.
            if exact && last.exact && last.src + (out - last.out) == src {
                return;
            }
        }
        let mark = Mark { out, src, exact };
        if self.first.is_none() {
            self.first = Some(mark);
        } else {
            self.rest.push(mark);
        }
    }

    /// Copies `src[from..to]` to the output.
    pub fn copy(&mut self, from: usize, to: usize) {
        if from >= to {
            return;
        }
        self.mark(from, true);
        self.out.push_str(&self.src[from..to]);
    }

    /// Writes `text` in place of the source text that starts at `at`.
    pub fn write(&mut self, text: &str, at: usize) {
        if text.is_empty() {
            return;
        }
        self.mark(at, false);
        self.out.push_str(text);
    }

    pub fn finish(self) -> (String, StepMap) {
        let out_len = self.out.len();
        let first = self.first.or(Some(Mark { out: 0, src: 0, exact: false }));
        (self.out, StepMap { first, rest: self.rest, out_len, src_len: self.src.len() })
    }
}

/// The steps from a source to the text a parser reads, applied in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    steps: Vec<StepMap>,
}

impl SourceMap {
    /// A single preprocessing step, with room for the entity-encoding step appended by
    /// the parser. Avoid the general Vec growth capacity for these short maps.
    pub(crate) fn single(step: StepMap) -> Self {
        let mut steps = Vec::with_capacity(2);
        steps.push(step);
        Self { steps }
    }

    pub fn push(&mut self, step: StepMap) {
        self.steps.push(step);
    }

    /// Appends the steps of another map, which starts from the text this one ends with.
    pub fn extend(&mut self, other: SourceMap) {
        self.steps.extend(other.steps);
    }

    /// The byte offset in the original source of byte `offset` of the final text.
    pub fn to_source(&self, offset: usize) -> usize {
        let mut at = offset;
        for step in self.steps.iter().rev() {
            at = step.to_src(at);
        }
        at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_copied_and_written_runs() {
        let src = "ab#x;cd";
        let mut b = Builder::new(src);
        b.copy(0, 2);
        b.write("[X]", 2);
        b.copy(5, 7);
        let (out, map) = b.finish();
        assert_eq!(out, "ab[X]cd");
        assert_eq!(map.to_src(0), 0);
        assert_eq!(map.to_src(1), 1);
        assert_eq!(map.to_src(3), 2);
        assert_eq!(map.to_src(5), 5);
        assert_eq!(map.to_src(6), 6);
        assert_eq!(map.to_src(7), 7);
    }

    #[test]
    fn composes_steps() {
        // Step one drops "xx", step two drops the first character.
        let mut b = Builder::new("abxxcd");
        b.copy(0, 2);
        b.copy(4, 6);
        let (one, m1) = b.finish();
        let mut b = Builder::new(&one);
        b.copy(1, one.len());
        let (two, m2) = b.finish();
        assert_eq!(two, "bcd");
        let mut map = SourceMap::default();
        map.push(m1);
        map.push(m2);
        assert_eq!(map.to_source(0), 1);
        assert_eq!(map.to_source(1), 4);
        assert_eq!(map.to_source(2), 5);
        assert_eq!(map.to_source(3), 6);
    }

    #[test]
    fn preserves_offsets_for_inline_and_multiple_marks_and_empty_steps() {
        // An unbuilt default map and an explicit empty step have distinct equality,
        // although both clamp offsets to zero.
        assert_ne!(StepMap::default(), Builder::new("").finish().1);
        assert_ne!(StepMap::identity(0), Builder::new("").finish().1);
        let mut b = Builder::new("abcdef");
        b.copy(2, 4);
        b.write("XY", 4);
        b.copy(4, 5);
        b.copy(5, 6);
        let (text, step) = b.finish();
        assert_eq!(text, "cdXYef");
        let expected = [2, 3, 4, 4, 4, 5, 6, 6];
        for (at, expected) in expected.into_iter().enumerate() {
            assert_eq!(step.to_src(at), expected);
            assert_eq!(step.clone().to_src(at), expected);
        }
        assert_eq!(step.to_src(usize::MAX), 6);
        for len in [0, 1, 6] {
            let identity = StepMap::identity(len);
            for at in 0..=len + 1 {
                assert_eq!(identity.to_src(at), at.min(len));
            }
            let (_, empty) = Builder::new(&"abcdef"[..len]).finish();
            assert_eq!(empty.to_src(0), len);
            assert_eq!(empty.to_src(usize::MAX), len);
            assert_eq!(StepMap::suffix(len, len).to_src(0), len);
        }
        let mut map = SourceMap::default();
        map.push(StepMap::suffix(6, 2));
        map.push(StepMap::identity(4));
        for at in 0..=5 {
            assert_eq!(map.to_source(at), (at + 2).min(6));
        }
    }
}
