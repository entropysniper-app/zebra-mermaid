//! `Math` functions with the results V8 gives.
//!
//! The reference is Node.js v26.3.0 (V8 14.6.202.34). V8 computes `sin`, `cos`, `tan`, `asin`,
//! `acos`, `atan`, `atan2`, `exp`, `log`, `log2` and `log10` with its fdlibm port in
//! `src/base/ieee754.cc`; [`super::fdlibm`] reproduces that code as clang compiles it, so these
//! match bit for bit on every platform. `Math.pow` is V8's `math::pow` (`src/numbers/ieee754.cc`),
//! which applies the ECMAScript special cases and then calls the C library's `pow`; see [`pow`].
//! `Math.hypot` is V8's own summation, written out below. `tests/compat_units.rs` checks these
//! against values recorded from Node.

use super::fdlibm;

/// `Math.sin` (fdlibm `s_sin.c`).
#[inline]
pub fn sin(x: f64) -> f64 {
    fdlibm::sin(x)
}

/// `Math.cos` (fdlibm `s_cos.c`).
#[inline]
pub fn cos(x: f64) -> f64 {
    fdlibm::cos(x)
}

/// `Math.tan` (fdlibm `s_tan.c`).
#[inline]
pub fn tan(x: f64) -> f64 {
    fdlibm::tan(x)
}

/// `Math.atan2` (fdlibm `e_atan2.c`).
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    fdlibm::atan2(y, x)
}

/// `Math.atan` (fdlibm `s_atan.c`).
#[inline]
pub fn atan(x: f64) -> f64 {
    fdlibm::atan(x)
}

/// `Math.acos` (fdlibm `e_acos.c`).
#[inline]
pub fn acos(x: f64) -> f64 {
    fdlibm::acos(x)
}

/// `Math.asin` (fdlibm `e_asin.c`).
#[inline]
pub fn asin(x: f64) -> f64 {
    fdlibm::asin(x)
}

/// `Math.exp` (fdlibm `e_exp.c`).
#[inline]
pub fn exp(x: f64) -> f64 {
    fdlibm::exp(x)
}

/// `Math.log` (fdlibm `e_log.c`).
#[inline]
pub fn log(x: f64) -> f64 {
    fdlibm::log(x)
}

/// `Math.log10` (fdlibm `e_log10.c`).
#[inline]
pub fn log10(x: f64) -> f64 {
    fdlibm::log10(x)
}

/// `Math.log2` (FreeBSD msun `e_log2.c`).
#[inline]
pub fn log2(x: f64) -> f64 {
    fdlibm::log2(x)
}

/// `Math.pow` and `**`, as V8's `math::pow` computes them.
///
/// V8 runs with `--use-std-math-pow` (the default except on AIX): after the ECMAScript special
/// cases it calls the C library's `pow`, the same function `f64::powf` calls. The result is
/// therefore exactly Node's on the same operating system and architecture, and it is not
/// portable: Apple's `pow`, glibc's and Rust's wasm fallback round differently in rare cases.
pub fn pow(x: f64, y: f64) -> f64 {
    if y.is_nan() {
        // 1. If exponent is NaN, return NaN.
        return f64::NAN;
    }
    if y.is_infinite() && (x == 1.0 || x == -1.0) {
        // ECMAScript returns NaN for a base of ±1 and an infinite exponent, where C returns 1.
        return f64::NAN;
    }
    // C distinguishes quiet and signaling NaN; JS does not.
    let x = if x.is_nan() { f64::NAN } else { x };
    // V8 matches its optimizing compilers, which do not call `pow` for these exponents.
    if y == 2.0 {
        return x * x;
    }
    if y == 0.5 {
        if x.is_infinite() {
            return f64::INFINITY;
        }
        // The +0 gives +0 rather than -0 for (-0) ** 0.5.
        return (x + 0.0).sqrt();
    }
    // `black_box` keeps LLVM from rewriting a constant base, e.g. pow(2, y) into exp2(y) or
    // pow(10, y) into exp10(y), which V8's runtime call never does.
    std::hint::black_box(x).powf(std::hint::black_box(y))
}

/// `Math.sqrt`, which IEEE 754 rounds exactly everywhere.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// `Math.hypot` over two values, as V8's builtin computes it: scaled by the largest magnitude,
/// with a compensated sum.
pub fn hypot(a: f64, b: f64) -> f64 {
    hypot_all(&[a, b])
}

/// `Math.hypot` over any number of values.
pub fn hypot_all(values: &[f64]) -> f64 {
    let mut max: f64 = 0.0;
    let mut nan = false;
    for &v in values {
        let a = v.abs();
        if a.is_nan() {
            nan = true;
        } else if a > max {
            max = a;
        }
    }
    if max == f64::INFINITY {
        return f64::INFINITY;
    }
    if nan {
        return f64::NAN;
    }
    if max == 0.0 {
        return 0.0;
    }
    // The magnitudes are read again rather than kept: `abs` is exact, and a buffer for two
    // values would cost an allocation on every edge drawn.
    let mut sum = 0.0;
    let mut compensation = 0.0;
    for &v in values {
        let n = v.abs() / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// `Math.imul`
#[inline]
pub fn imul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}
