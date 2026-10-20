//! Private SVG concatenation. Static parts bypass `fmt::Arguments` dispatch while keeping
//! every argument alive and evaluating it before any output is written.

use std::borrow::Cow;
use std::fmt::{self, Write as _};

pub(crate) trait Part {
    fn append(&self, out: &mut String);
    fn size_hint(&self) -> usize {
        0
    }
}

impl<T: Part + ?Sized> Part for &T {
    #[inline]
    fn append(&self, out: &mut String) {
        (**self).append(out);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        (**self).size_hint()
    }
}

impl<T: Part + ?Sized> Part for &mut T {
    #[inline]
    fn append(&self, out: &mut String) {
        (**self).append(out);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        (**self).size_hint()
    }
}

impl Part for std::rc::Rc<str> {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(self);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Part for str {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(self);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Part for String {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(self);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Part for Cow<'_, str> {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(self);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Part for super::builder::Num {
    #[inline]
    fn append(&self, out: &mut String) {
        super::builder::push_num(out, self.0);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        8
    }
}

impl Part for crate::util::js::NumberText {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(self.as_str());
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

macro_rules! display_parts {
    ($($t:ty),* $(,)?) => {
        $(impl Part for $t {
            #[inline]
            fn append(&self, out: &mut String) {
                write!(out, "{}", self).unwrap();
            }
        })*
    };
}
// Preserve Rust's original Display semantics for ordinary numeric arguments. SVG coordinates
// use Num above; numeric conversion itself is unchanged by this concatenation layer.
display_parts!(f64, f32, usize, isize, u64, i64, u32, i32, u16, i16, u8, i8, u128, i128);

impl Part for char {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push(*self);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len_utf8()
    }
}

impl Part for bool {
    #[inline]
    fn append(&self, out: &mut String) {
        out.push_str(if *self { "true" } else { "false" });
    }
}

pub(crate) struct Pair<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H: Part, T: Part> Part for Pair<H, T> {
    #[inline]
    fn append(&self, out: &mut String) {
        self.head.append(out);
        self.tail.append(out);
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.head.size_hint().saturating_add(self.tail.size_hint())
    }
}

impl Part for () {
    #[inline]
    fn append(&self, _: &mut String) {}
}

#[inline]
pub(crate) fn format(parts: impl Part) -> String {
    let mut out = String::with_capacity(parts.size_hint());
    parts.append(&mut out);
    out
}

pub(crate) trait Sink {
    fn write_svg_parts(&mut self, parts: impl Part) -> fmt::Result;
}

impl Sink for String {
    #[inline]
    fn write_svg_parts(&mut self, parts: impl Part) -> fmt::Result {
        parts.append(self);
        Ok(())
    }
}

macro_rules! svg_parts {
    () => { () };
    ($head:expr $(, $tail:expr)* $(,)?) => {
        $crate::svg::concat::Pair {
            head: &$head,
            tail: $crate::svg::concat::svg_parts!($($tail),*),
        }
    };
}
pub(crate) use svg_parts;

macro_rules! svg_format {
    ($($part:expr),* $(,)?) => {
        $crate::svg::concat::format($crate::svg::concat::svg_parts!($($part),*))
    };
}
pub(crate) use svg_format;

macro_rules! svg_write {
    ($out:expr, $($part:expr),* $(,)?) => {
        {
            use $crate::svg::concat::Sink as _;
            $out.write_svg_parts($crate::svg::concat::svg_parts!($($part),*))
        }
    };
}
pub(crate) use svg_write;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn temporary_arguments_and_native_display_are_preserved() {
        for n in [0.0, -0.0, 1.005, -2.345, 1e-100, 1e100, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            let actual = svg_format!("<", &format!("{n}"), ':', super::super::builder::num(n), ">", n, 42_u64, false, "中😀");
            assert_eq!(actual, format!("<{n}:{}>{n}{}{}中😀", super::super::builder::num(n), 42_u64, false));
        }
        let mut out = String::from("prefix");
        svg_write!(out, &String::from("<&"), Cow::Borrowed("后"), Cow::Owned::<str>("tail".into())).unwrap();
        assert_eq!(out, "prefix<&后tail");
        fn append_to_borrow(out: &mut String) {
            svg_write!(out, out.len(), String::from("done")).unwrap();
        }
        let before = out.len();
        append_to_borrow(&mut out);
        assert_eq!(out, format!("prefix<&后tail{before}done"));
    }

    struct Observed<'a> {
        log: &'a RefCell<Vec<String>>,
        name: &'static str,
    }
    impl fmt::Display for Observed<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.log.borrow_mut().push(format!("write:{}", self.name));
            f.write_str(self.name)
        }
    }
    impl Part for Observed<'_> {
        fn append(&self, out: &mut String) {
            write!(out, "{}", self).unwrap();
        }
    }
    impl Drop for Observed<'_> {
        fn drop(&mut self) {
            self.log.borrow_mut().push(format!("drop:{}", self.name));
        }
    }
    fn observe<'a>(log: &'a RefCell<Vec<String>>, name: &'static str) -> Observed<'a> {
        log.borrow_mut().push(format!("eval:{name}"));
        Observed { log, name }
    }

    #[test]
    fn evaluation_writes_and_drops_keep_standard_format_order() {
        let expected = RefCell::new(Vec::new());
        let original = format!("{}{}", observe(&expected, "a"), observe(&expected, "b"));
        let actual = RefCell::new(Vec::new());
        let result = svg_format!(observe(&actual, "a"), observe(&actual, "b"));
        assert_eq!(result, original);
        assert_eq!(actual.into_inner(), expected.into_inner());
    }
}
