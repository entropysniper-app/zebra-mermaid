//! JavaScript number and string semantics that the ported code depends on.
//!
//! Pele is written against ECMAScript: `String(n)`, `Math.round`, `toFixed`, `Number(s)`,
//! `parseFloat`, `parseInt`, and string lengths counted in UTF-16 code units. Rust's own
//! operations differ in small ways (half-way rounding, exponent notation, which characters
//! count as white space), and those differences show up in SVG output. Everything here follows
//! the ECMAScript specification, not Rust's defaults.

use super::chars::is_space;
use std::fmt::{self, Write};

/// Scratch storage for Rust's decimal formatter. Normal f64 output and up to 100
/// significant digits fit on the stack; larger caller-supplied precisions can spill.
struct DecimalBuffer<const N: usize> {
    bytes: [u8; N],
    len: usize,
    heap: Option<String>,
}

impl<const N: usize> DecimalBuffer<N> {
    fn new() -> Self {
        Self { bytes: [0; N], len: 0, heap: None }
    }

    fn as_str(&self) -> &str {
        self.heap.as_deref().unwrap_or_else(|| std::str::from_utf8(&self.bytes[..self.len]).expect("decimal ASCII"))
    }

    fn truncate(&mut self, len: usize) {
        if let Some(heap) = &mut self.heap {
            heap.truncate(len);
        } else {
            self.len = self.len.min(len);
        }
    }

    /// Keeps the mantissa's digits in the same buffer and returns its exponent.
    fn scientific_digits(&mut self) -> i32 {
        let at = self.as_str().find('e').expect("exponent");
        let exponent = self.as_str()[at + 1..].parse().expect("exponent digits");
        if let Some(heap) = self.heap.take() {
            let mut bytes = heap.into_bytes();
            bytes.truncate(at);
            bytes.retain(u8::is_ascii_digit);
            self.heap = Some(String::from_utf8(bytes).expect("decimal ASCII"));
        } else {
            let mut len = 0;
            for i in 0..at {
                if self.bytes[i].is_ascii_digit() {
                    self.bytes[len] = self.bytes[i];
                    len += 1;
                }
            }
            self.len = len;
        }
        exponent
    }

    /// Adds one to a run of significant digits, keeping its length. A carry past
    /// the first digit changes 999 to 100 and is returned to adjust the exponent.
    fn increment(&mut self) -> bool {
        fn increment(bytes: &mut [u8]) -> bool {
            for digit in bytes.iter_mut().rev() {
                if *digit != b'9' {
                    *digit += 1;
                    return false;
                }
                *digit = b'0';
            }
            bytes[0] = b'1';
            true
        }
        if let Some(heap) = self.heap.take() {
            let mut bytes = heap.into_bytes();
            let carry = increment(&mut bytes);
            self.heap = Some(String::from_utf8(bytes).expect("decimal ASCII"));
            carry
        } else {
            increment(&mut self.bytes[..self.len])
        }
    }
}

impl<const N: usize> fmt::Write for DecimalBuffer<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if let Some(heap) = &mut self.heap {
            heap.push_str(text);
        } else if text.len() <= N - self.len {
            self.bytes[self.len..self.len + text.len()].copy_from_slice(text.as_bytes());
            self.len += text.len();
        } else {
            let mut heap = String::with_capacity(self.len + text.len());
            heap.push_str(self.as_str());
            heap.push_str(text);
            self.heap = Some(heap);
        }
        Ok(())
    }
}

/// `Math.round`: rounds half-way values up, towards positive infinity.
#[inline]
pub fn round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

/// `Math.sign`, which keeps NaN and the sign of zero.
#[inline]
pub fn sign(x: f64) -> f64 {
    if x.is_nan() || x == 0.0 {
        x
    } else if x > 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// `Math.max` over two values: NaN if either is NaN, and +0 over -0.
#[inline]
pub fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == b {
        if a == 0.0 && (a.is_sign_positive() || b.is_sign_positive()) {
            0.0
        } else {
            a
        }
    } else if a > b {
        a
    } else {
        b
    }
}

/// `Math.min` over two values: NaN if either is NaN, and -0 over +0.
#[inline]
pub fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == b {
        if a == 0.0 && (a.is_sign_negative() || b.is_sign_negative()) {
            -0.0
        } else {
            a
        }
    } else if a < b {
        a
    } else {
        b
    }
}

/// The shortest decimal digits that read back as `x` (positive and finite), with the exponent
/// `n` such that `x = 0.d1d2d3... × 10^n`.
fn shortest_digits(x: f64) -> (DecimalBuffer<32>, i32) {
    let mut buffer = zmij::Buffer::new();
    let printed = buffer.format_finite(x);
    let (mantissa, exponent) =
        printed.split_once('e').map_or((printed, 0), |(digits, exponent)| (digits, exponent.parse::<i32>().expect("exponent digits")));
    let point = mantissa.find('.').unwrap_or(mantissa.len());
    let mut digits = DecimalBuffer::new();
    let mut leading = 0;
    for byte in mantissa.bytes().filter(u8::is_ascii_digit) {
        if digits.len == 0 && byte == b'0' {
            leading += 1;
        } else {
            digits.bytes[digits.len] = byte;
            digits.len += 1;
        }
    }
    digits.truncate(digits.as_str().trim_end_matches('0').len());
    (digits, point as i32 - leading + exponent)
}

/// `String(x)` / `Number.prototype.toString()` in base 10.
pub fn to_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x == 0.0 {
        return "0".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    // Every integer in the safe-integer range has its exact decimal spelling in JS.
    // Dimensions, font sizes and ticks usually take this path; avoid constructing and
    // splitting an exponential string before reconstructing the same integer.
    if x.abs() <= 9_007_199_254_740_991.0 && x.fract() == 0.0 {
        return (x as i64).to_string();
    }
    let negative = x < 0.0;
    let (digits, n) = shortest_digits(x.abs());
    let digits = digits.as_str();
    let k = digits.len() as i32;
    let capacity = if k <= n && n <= 21 {
        n as usize
    } else if 0 < n && n <= 21 {
        digits.len() + 1
    } else if -6 < n && n <= 0 {
        2 + (-n) as usize + digits.len()
    } else {
        digits.len() + 7
    };
    let mut out = String::with_capacity(capacity + usize::from(negative));
    if negative {
        out.push('-');
    }
    if k <= n && n <= 21 {
        out.push_str(digits);
        for _ in 0..(n - k) {
            out.push('0');
        }
    } else if 0 < n && n <= 21 {
        out.push_str(&digits[..n as usize]);
        out.push('.');
        out.push_str(&digits[n as usize..]);
    } else if -6 < n && n <= 0 {
        out.push_str("0.");
        for _ in 0..(-n) {
            out.push('0');
        }
        out.push_str(digits);
    } else {
        let e = n - 1;
        out.push_str(&digits[..1]);
        if k > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        write!(out, "e{e:+}").unwrap();
    }
    out
}

/// Writes an integer-valued double the way `String(n)` does, without the general formatter.
pub fn int_to_string(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        to_string(x)
    }
}

/// Whether `x * 10^places` is exactly halfway between integers, and whether its
/// lower integer is even. All calculations are exact on the binary significand.
fn decimal_tie(x: f64, places: i32) -> Option<bool> {
    if x == 0.0 || !x.is_finite() {
        return None;
    }
    // For nonnegative places, 5^places is 1 modulo 4. For negative places,
    // the odd significand must contain the corresponding factor of 5 first.
    let bits = x.abs().to_bits();
    let exp_bits = ((bits >> 52) & 0x7ff) as i64;
    let frac = bits & ((1u64 << 52) - 1);
    let (mut m, mut e) = if exp_bits == 0 { (frac, -1074i64) } else { (frac | (1u64 << 52), exp_bits - 1075) };
    if m == 0 {
        return None;
    }
    let tz = m.trailing_zeros();
    m >>= tz;
    e += tz as i64;
    if e + places as i64 != -1 {
        return None;
    }
    if places < 0 {
        let count = places.unsigned_abs();
        // An odd binary64 significand is below 2^53, and 5^23 exceeds that.
        if count > 22 {
            return None;
        }
        let factor = 5_u64.pow(count);
        if !m.is_multiple_of(factor) {
            return None;
        }
        m /= factor;
    }
    Some(m & 3 == 1)
}

/// `Number.prototype.toFixed(digits)`. A value exactly half-way rounds away from zero, as the
/// specification picks the larger of two equally near integers for the magnitude.
pub fn to_fixed(x: f64, digits: u32) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.abs() >= 1e21 || x.is_infinite() {
        return to_string(x);
    }
    let negative = x < 0.0;
    let a = x.abs();
    let mut s = format!("{:.*}", digits as usize, a);
    if decimal_tie(a, digits as i32) == Some(true) {
        // Rust rounds an exact tie to even; ECMAScript takes the larger candidate.
        s = increment_decimal(&s);
    }
    // `(-0.001).toFixed(2)` is "-0.00": the sign comes from x < 0 alone.
    if negative {
        format!("-{}", s)
    } else {
        s
    }
}

/// Adds one unit in the last place of a decimal string of digits and at most one point.
fn increment_decimal(s: &str) -> String {
    let mut bytes: Vec<u8> = s.as_bytes().to_vec();
    let mut i = bytes.len();
    loop {
        if i == 0 {
            bytes.insert(0, b'1');
            break;
        }
        i -= 1;
        match bytes[i] {
            b'.' => continue,
            b'9' => bytes[i] = b'0',
            d => {
                bytes[i] = d + 1;
                break;
            }
        }
    }
    String::from_utf8(bytes).expect("ascii")
}

/// `Number.prototype.toPrecision(precision)` for 1..=100 digits.
pub fn to_precision(x: f64, precision: u32) -> String {
    if !x.is_finite() {
        return to_string(x);
    }
    if x == 0.0 {
        let mut s = String::from("0");
        if precision > 1 {
            s.push('.');
            for _ in 1..precision {
                s.push('0');
            }
        }
        return s;
    }
    let negative = x < 0.0;
    let a = x.abs();
    // Digits rounded to `precision` significant places, with ties away from zero as the
    // specification picks the larger n.
    let mut digits: DecimalBuffer<128> = DecimalBuffer::new();
    write!(digits, "{:.*e}", (precision - 1) as usize, a).unwrap();
    let mut e = digits.scientific_digits();
    // Ties: check whether a lies exactly half-way at this precision.
    let shift = precision as i32 - 1 - e;
    if decimal_tie(a, shift) == Some(true) && digits.increment() {
        e += 1;
    }
    let digits = digits.as_str();
    let p = precision as i32;
    let capacity = if e < -6 || e >= p {
        digits.len() + 7
    } else if e == p - 1 {
        digits.len()
    } else if e >= 0 {
        digits.len() + 1
    } else {
        2 + (-(e + 1)) as usize + digits.len()
    };
    let mut out = String::with_capacity(capacity + usize::from(negative));
    if negative {
        out.push('-');
    }
    if e < -6 || e >= p {
        out.push_str(&digits[..1]);
        if p > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        write!(out, "e{e:+}").unwrap();
        return out;
    }
    if e == p - 1 {
        out.push_str(digits);
        return out;
    }
    if e >= 0 {
        out.push_str(&digits[..(e + 1) as usize]);
        out.push('.');
        out.push_str(&digits[(e + 1) as usize..]);
    } else {
        out.push_str("0.");
        for _ in 0..(-(e + 1)) {
            out.push('0');
        }
        out.push_str(digits);
    }
    out
}

/// Trims what `String.prototype.trim` trims: `\s` white space and line terminators.
#[inline]
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_space)
}

#[inline]
pub fn trim_start(s: &str) -> &str {
    s.trim_start_matches(is_space)
}

#[inline]
pub fn trim_end(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

/// The length of a string in UTF-16 code units, as `String.prototype.length` counts it.
#[inline]
pub fn len16(s: &str) -> usize {
    if s.is_ascii() {
        s.len()
    } else {
        s.chars().map(char::len_utf16).sum()
    }
}

/// The byte offset in `s` of UTF-16 offset `units`, clamped to the string. An offset that
/// falls inside a surrogate pair is moved to the start of the character.
pub fn byte_at16(s: &str, units: usize) -> usize {
    if s.is_ascii() {
        return units.min(s.len());
    }
    let mut count = 0;
    for (i, c) in s.char_indices() {
        if count >= units {
            return i;
        }
        count += c.len_utf16();
        if count > units {
            return i;
        }
    }
    s.len()
}

/// The UTF-16 offset of byte offset `byte` in `s`.
pub fn units_at(s: &str, byte: usize) -> usize {
    len16(&s[..byte.min(s.len())])
}

/// `s.slice(from, to)` with UTF-16 offsets, clamped. A cut through a surrogate pair keeps
/// the whole character on the side where it starts.
pub fn slice16(s: &str, from: usize, to: usize) -> &str {
    let a = byte_at16(s, from);
    let b = byte_at16(s, to).max(a);
    &s[a..b]
}

fn is_str_whitespace(c: char) -> bool {
    is_space(c)
}

/// `Number(s)`: the StringToNumber conversion.
pub fn number(s: &str) -> f64 {
    let t = s.trim_matches(is_str_whitespace);
    if t.is_empty() {
        return 0.0;
    }
    let b = t.as_bytes();
    if b.len() > 2 && b[0] == b'0' {
        let radix = match b[1] {
            b'x' | b'X' => 16,
            b'o' | b'O' => 8,
            b'b' | b'B' => 2,
            _ => 0,
        };
        if radix != 0 {
            let digits = &t[2..];
            if digits.chars().all(|c| c.is_digit(radix)) {
                return digits_value(digits, radix);
            }
            return f64::NAN;
        }
    }
    let (sign, body) = match b[0] {
        b'+' => (1.0, &t[1..]),
        b'-' => (-1.0, &t[1..]),
        _ => (1.0, t),
    };
    if body == "Infinity" {
        return sign * f64::INFINITY;
    }
    match decimal_prefix(body) {
        Some(end) if end == body.len() => sign * body.parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

fn digits_value(digits: &str, radix: u32) -> f64 {
    let mut v = 0.0f64;
    for c in digits.chars() {
        v = v * radix as f64 + c.to_digit(radix).unwrap_or(0) as f64;
    }
    // Exact for anything below 2^53; beyond that the specification rounds the full value,
    // which this sum approximates. Pele only reads small values this way.
    v
}

/// The length of the longest StrUnsignedDecimalLiteral (without Infinity) at the start of `s`.
fn decimal_prefix(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - int_start;
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        let mut j = i + 1;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        frac_digits = j - i - 1;
        if int_digits > 0 || frac_digits > 0 {
            i = j;
        }
    }
    if int_digits == 0 && frac_digits == 0 {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    Some(i)
}

/// `parseFloat(s)`.
pub fn parse_float(s: &str) -> f64 {
    let t = s.trim_start_matches(is_str_whitespace);
    let (sign, body) = match t.as_bytes().first() {
        Some(b'+') => (1.0, &t[1..]),
        Some(b'-') => (-1.0, &t[1..]),
        _ => (1.0, t),
    };
    if body.starts_with("Infinity") {
        return sign * f64::INFINITY;
    }
    match decimal_prefix(body) {
        Some(end) => {
            let text = &body[..end];
            // "5." and ".5" are valid prefixes for parseFloat; Rust parses both.
            sign * text.parse::<f64>().unwrap_or(f64::NAN)
        }
        None => f64::NAN,
    }
}

/// `parseInt(s, radix)` for a radix of 0 (detect) or 2..=36.
pub fn parse_int(s: &str, radix: u32) -> f64 {
    let t = s.trim_start_matches(is_str_whitespace);
    let (sign, mut body) = match t.as_bytes().first() {
        Some(b'+') => (1.0, &t[1..]),
        Some(b'-') => (-1.0, &t[1..]),
        _ => (1.0, t),
    };
    let mut radix = radix;
    let mut strip = true;
    if radix != 0 {
        if !(2..=36).contains(&radix) {
            return f64::NAN;
        }
        if radix != 16 {
            strip = false;
        }
    } else {
        radix = 10;
    }
    if strip && (body.starts_with("0x") || body.starts_with("0X")) {
        body = &body[2..];
        radix = 16;
    }
    let end = body.find(|c: char| !c.is_digit(radix)).unwrap_or(body.len());
    if end == 0 {
        return f64::NAN;
    }
    let digits = &body[..end];
    let value = if radix == 10 { digits.parse::<f64>().unwrap_or(f64::NAN) } else { digits_value(digits, radix) };
    sign * value
}

/// `Number.isInteger` / `Number.isSafeInteger`-style check for an integral, finite value.
#[inline]
pub fn is_integer(x: f64) -> bool {
    x.is_finite() && x.fract() == 0.0
}

/// ToInt32, as the bitwise operators apply it.
pub fn to_int32(x: f64) -> i32 {
    if !x.is_finite() || x == 0.0 {
        return 0;
    }
    let t = x.trunc();
    let m = t.rem_euclid(4294967296.0);
    let u = m as u64 as u32;
    u as i32
}

/// The `%` operator on doubles: the result takes the sign of the dividend.
#[inline]
pub fn rem(a: f64, b: f64) -> f64 {
    a % b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_of_numbers() {
        assert_eq!(to_string(0.0), "0");
        assert_eq!(to_string(-0.0), "0");
        assert_eq!(to_string(1.0), "1");
        assert_eq!(to_string(-1.5), "-1.5");
        assert_eq!(to_string(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(to_string(1e21), "1e+21");
        assert_eq!(to_string(1e20), "100000000000000000000");
        assert_eq!(to_string(123456789012345680000.0), "123456789012345680000");
        assert_eq!(to_string(1e-7), "1e-7");
        assert_eq!(to_string(1.5e-7), "1.5e-7");
        assert_eq!(to_string(0.000001), "0.000001");
        assert_eq!(to_string(f64::NAN), "NaN");
        assert_eq!(to_string(f64::INFINITY), "Infinity");
        assert_eq!(to_string(-f64::INFINITY), "-Infinity");
        assert_eq!(to_string(5e-324), "5e-324");
        assert_eq!(to_string(1.7976931348623157e308), "1.7976931348623157e+308");
        assert_eq!(to_string(255.0 / 7.0), "36.42857142857143");
    }

    #[test]
    fn math_round_rounds_halves_up() {
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -2.0);
        assert_eq!(round(-0.5), 0.0);
        assert_eq!(round(0.49999999999999994), 0.0);
        assert_eq!(round(1.4999), 1.0);
        assert!(round(f64::NAN).is_nan());
    }

    #[test]
    fn fixed_rounds_exact_ties_away_from_zero() {
        assert_eq!(to_fixed(0.5, 0), "1");
        assert_eq!(to_fixed(2.5, 0), "3");
        assert_eq!(to_fixed(-2.5, 0), "-3");
        assert_eq!(to_fixed(1.005, 2), "1.00");
        assert_eq!(to_fixed(1.125, 2), "1.13");
        assert_eq!(to_fixed(33.333333, 0), "33");
        assert_eq!(to_fixed(-0.001, 2), "-0.00");
        assert_eq!(to_fixed(99.5, 0), "100");
        assert_eq!(to_fixed(0.0, 2), "0.00");
        assert_eq!(to_fixed(1e21, 2), "1e+21");
    }

    #[test]
    fn precision() {
        assert_eq!(to_precision(123.456, 4), "123.5");
        assert_eq!(to_precision(0.000123, 2), "0.00012");
        assert_eq!(to_precision(123456.0, 2), "1.2e+5");
        assert_eq!(to_precision(2.5, 1), "3");
        assert_eq!(to_precision(1e-7, 3), "1.00e-7");
        assert_eq!(to_precision(0.0, 3), "0.00");
    }

    #[test]
    fn numbers_from_strings() {
        assert_eq!(number(""), 0.0);
        assert_eq!(number("  12  "), 12.0);
        assert_eq!(number("1e3"), 1000.0);
        assert_eq!(number(".5"), 0.5);
        assert_eq!(number("5."), 5.0);
        assert_eq!(number("0x1F"), 31.0);
        assert_eq!(number("-Infinity"), f64::NEG_INFINITY);
        assert!(number("inf").is_nan());
        assert!(number("NaN").is_nan());
        assert!(number("1e").is_nan());
        assert!(number("-0x10").is_nan());
        assert!(number("12px").is_nan());
        assert_eq!(number("\u{feff}7\u{3000}"), 7.0);
    }

    #[test]
    fn parse_float_takes_a_prefix() {
        assert_eq!(parse_float("12px"), 12.0);
        assert_eq!(parse_float("  -3.5e2x"), -350.0);
        assert_eq!(parse_float("1e"), 1.0);
        assert!(parse_float("px").is_nan());
        assert_eq!(parse_float("Infinityx"), f64::INFINITY);
        assert_eq!(parse_float(".5."), 0.5);
    }

    #[test]
    fn parse_int_reads_digits() {
        assert_eq!(parse_int("42abc", 10), 42.0);
        assert_eq!(parse_int("0x1f", 0), 31.0);
        assert_eq!(parse_int("ff", 16), 255.0);
        assert!(parse_int("z", 10).is_nan());
        assert_eq!(parse_int("-12", 10), -12.0);
    }

    #[test]
    fn utf16_lengths_and_slices() {
        assert_eq!(len16("abc"), 3);
        assert_eq!(len16("中文"), 2);
        assert_eq!(len16("😀"), 2);
        assert_eq!(len16("e\u{301}"), 2);
        assert_eq!(slice16("a😀b", 0, 1), "a");
        assert_eq!(slice16("a😀b", 1, 3), "😀");
        assert_eq!(slice16("a😀b", 3, 4), "b");
        assert_eq!(units_at("a😀b", 5), 3);
        assert_eq!(byte_at16("中文x", 2), 6);
    }

    #[test]
    fn js_trim_differs_from_rust_trim() {
        // U+FEFF is white space for JavaScript and not for Rust; U+0085 is the reverse.
        assert_eq!(trim("\u{feff} a \u{3000}"), "a");
        assert_eq!(trim("\u{85}a"), "\u{85}a");
    }

    #[test]
    fn int32() {
        assert_eq!(to_int32(4294967297.0), 1);
        assert_eq!(to_int32(-1.5), -1);
        assert_eq!(to_int32(2147483648.0), -2147483648);
        assert_eq!(to_int32(f64::NAN), 0);
    }
}
