// The following is adapted from fdlibm (http://www.netlib.org/fdlibm), by way of V8's
// `src/base/ieee754.cc` (V8 14.6.202.34, as bundled in Node.js v26.3.0).
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunSoft, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================
//
// The original source code covered by the above license above has been
// modified significantly by Google Inc.
// Copyright 2016 the V8 project authors. All rights reserved.
// (V8 is distributed under a BSD-style license; see THIRD_PARTY_NOTICES.md.)
//
// `log2` (with `k_log1p`) follows FreeBSD msun's e_log2.c and k_log.h as copied into the
// same V8 file; those files carry the same Sun Microsystems notice.
//
// `scalbn`, which V8 takes from the C library, follows musl's src/math/scalbn.c:
// Copyright © 2005-2020 Rich Felker, et al. (MIT license; see THIRD_PARTY_NOTICES.md).

//! fdlibm as V8 compiles it, for bit-exact `Math.*` results.
//!
//! This is a line-by-line port of the functions V8 uses for `Math.sin`, `cos`, `tan`, `asin`,
//! `acos`, `atan`, `atan2`, `exp`, `log`, `log2` and `log10`. Node.js builds V8 without
//! `V8_USE_LIBM_TRIG_FUNCTIONS`, so `sin`/`cos` are fdlibm too, not glibc.
//!
//! Node's clang build compiles this C++ with the default `-ffp-contract=on`, which fuses a
//! multiplication into the addition or subtraction that consumes it within one expression
//! (`a * b + c` becomes `fma(a, b, c)`, the left product winning when both operands are
//! products). The results depend on it, so every such fusion is written here as an explicit
//! [`fma`] call, and every unfused operation is written as plain `f64` arithmetic, which Rust
//! never contracts. The fused sites were read from clang's IR for `ieee754.cc`.

#![allow(clippy::approx_constant, clippy::excessive_precision, clippy::unreadable_literal)]

/// A multiply-add with one rounding, where clang contracted the C++ expression.
#[inline(always)]
fn fma(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}

/// `c[0] + x * (c[1] + x * (c[2] + ... + x * c[n-1]))`, every step fused as clang contracts
/// it: the innermost `c[n-2] + x * c[n-1]` first, then outwards.
#[inline(always)]
fn horner(x: f64, c: &[f64]) -> f64 {
    let mut acc = c[c.len() - 1];
    for &k in c[..c.len() - 1].iter().rev() {
        acc = fma(x, acc, k);
    }
    acc
}

/// `GET_HIGH_WORD`: the sign, exponent and top 20 mantissa bits.
#[inline(always)]
fn high_word(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

/// `GET_LOW_WORD`: the low 32 mantissa bits.
#[inline(always)]
fn low_word(x: f64) -> u32 {
    x.to_bits() as u32
}

/// `INSERT_WORDS`.
#[inline(always)]
fn from_words(hi: u32, lo: u32) -> f64 {
    f64::from_bits(((hi as u64) << 32) | lo as u64)
}

/// `SET_HIGH_WORD`.
#[inline(always)]
fn with_high_word(x: f64, hi: u32) -> f64 {
    f64::from_bits((x.to_bits() & 0xFFFF_FFFF) | ((hi as u64) << 32))
}

/// `SET_LOW_WORD`.
#[inline(always)]
fn with_low_word(x: f64, lo: u32) -> f64 {
    f64::from_bits((x.to_bits() & 0xFFFF_FFFF_0000_0000) | lo as u64)
}

/// `scalbn(x, n)`: `x * 2^n` with one rounding, as musl computes it. V8 calls the C library's
/// `scalbn` here; its results are exact for the normal values this module passes.
fn scalbn(x: f64, mut n: i32) -> f64 {
    let x1p1023 = f64::from_bits(0x7FE0_0000_0000_0000);
    let x1p53 = f64::from_bits(0x4340_0000_0000_0000);
    let x1p_1022 = f64::from_bits(0x0010_0000_0000_0000);
    let mut y = x;
    if n > 1023 {
        y *= x1p1023;
        n -= 1023;
        if n > 1023 {
            y *= x1p1023;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        y *= x1p_1022 * x1p53;
        n += 1022 - 53;
        if n < -1022 {
            y *= x1p_1022 * x1p53;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    y * f64::from_bits(((0x3FF + n) as u64) << 52)
}

/// Table of constants for 2/pi, 396 hex digits (476 decimal) of 2/pi.
const TWO_OVER_PI: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163, 0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2,
    0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129, 0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353,
    0x39F49C, 0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF, 0x27CB09, 0xB74F46, 0x3F669E,
    0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292, 0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20,
    0x9AF436, 0x1DA9E3, 0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA, 0x73A8C9, 0x60E27B,
    0xC08C6B,
];

/// `__ieee754_rem_pio2(x, y)`: returns `n` and the remainder of `x` rem pi/2 as `y[0] + y[1]`.
fn rem_pio2(x: f64) -> (i32, f64, f64) {
    const NPIO2_HW: [i32; 32] = [
        0x3FF921FB, 0x400921FB, 0x4012D97C, 0x401921FB, 0x401F6A7A, 0x4022D97C, 0x4025FDBB, 0x402921FB, 0x402C463A, 0x402F6A7A, 0x4031475C,
        0x4032D97C, 0x40346B9C, 0x4035FDBB, 0x40378FDB, 0x403921FB, 0x403AB41B, 0x403C463A, 0x403DD85A, 0x403F6A7A, 0x40407E4C, 0x4041475C,
        0x4042106C, 0x4042D97C, 0x4043A28C, 0x40446B9C, 0x404534AC, 0x4045FDBB, 0x4046C6CB, 0x40478FDB, 0x404858EB, 0x404921FB,
    ];
    // invpio2:  53 bits of 2/pi
    // pio2_1:   first  33 bit of pi/2
    // pio2_1t:  pi/2 - pio2_1
    // pio2_2:   second 33 bit of pi/2
    // pio2_2t:  pi/2 - (pio2_1+pio2_2)
    // pio2_3:   third  33 bit of pi/2
    // pio2_3t:  pi/2 - (pio2_1+pio2_2+pio2_3)
    const ZERO: f64 = 0.00000000000000000000e+00; // 0x00000000, 0x00000000
    const HALF: f64 = 5.00000000000000000000e-01; // 0x3FE00000, 0x00000000
    const TWO24: f64 = 1.67772160000000000000e+07; // 0x41700000, 0x00000000
    const INVPIO2: f64 = 6.36619772367581382433e-01; // 0x3FE45F30, 0x6DC9C883
    const PIO2_1: f64 = 1.57079632673412561417e+00; // 0x3FF921FB, 0x54400000
    const PIO2_1T: f64 = 6.07710050650619224932e-11; // 0x3DD0B461, 0x1A626331
    const PIO2_2: f64 = 6.07710050630396597660e-11; // 0x3DD0B461, 0x1A600000
    const PIO2_2T: f64 = 2.02226624879595063154e-21; // 0x3BA3198A, 0x2E037073
    const PIO2_3: f64 = 2.02226624871116645580e-21; // 0x3BA3198A, 0x2E000000
    const PIO2_3T: f64 = 8.47842766036889956997e-32; // 0x397B839A, 0x252049C1

    let hx = high_word(x);
    let ix = hx & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        // |x| ~<= pi/4, no need for reduction
        return (0, x, 0.0);
    }
    if ix < 0x4002D97C {
        // |x| < 3pi/4, special case with n=+-1
        if hx > 0 {
            let mut z = x - PIO2_1;
            if ix != 0x3FF921FB {
                // 33+53 bit pi is good enough
                let y0 = z - PIO2_1T;
                let y1 = (z - y0) - PIO2_1T;
                return (1, y0, y1);
            }
            // near pi/2, use 33+33+53 bit pi
            z -= PIO2_2;
            let y0 = z - PIO2_2T;
            let y1 = (z - y0) - PIO2_2T;
            return (1, y0, y1);
        }
        // negative x
        let mut z = x + PIO2_1;
        if ix != 0x3FF921FB {
            let y0 = z + PIO2_1T;
            let y1 = (z - y0) + PIO2_1T;
            return (-1, y0, y1);
        }
        z += PIO2_2;
        let y0 = z + PIO2_2T;
        let y1 = (z - y0) + PIO2_2T;
        return (-1, y0, y1);
    }
    if ix <= 0x413921FB {
        // |x| ~<= 2^19*(pi/2), medium size
        let t = x.abs();
        let n = fma(t, INVPIO2, HALF) as i32;
        let fnum = n as f64;
        let mut r = fma(-fnum, PIO2_1, t);
        let mut w = fnum * PIO2_1T; // 1st round good to 85 bit
        let mut y0;
        if n < 32 && ix != NPIO2_HW[(n - 1) as usize] {
            y0 = r - w; // quick check no cancellation
        } else {
            let j = ix >> 20;
            y0 = r - w;
            let high = high_word(y0) as u32;
            let i = j - ((high >> 20) & 0x7FF) as i32;
            if i > 16 {
                // 2nd iteration needed, good to 118
                let t = r;
                w = fnum * PIO2_2;
                r = t - w;
                w = fma(fnum, PIO2_2T, -((t - r) - w));
                y0 = r - w;
                let high = high_word(y0) as u32;
                let i = j - ((high >> 20) & 0x7FF) as i32;
                if i > 49 {
                    // 3rd iteration need, 151 bits acc
                    let t = r; // will cover all possible cases
                    w = fnum * PIO2_3;
                    r = t - w;
                    w = fma(fnum, PIO2_3T, -((t - r) - w));
                    y0 = r - w;
                }
            }
        }
        let y1 = (r - y0) - w;
        if hx < 0 {
            return (-n, -y0, -y1);
        }
        return (n, y0, y1);
    }
    // all other (large) arguments
    if ix >= 0x7FF00000 {
        // x is inf or NaN
        // y[0] = y[1] = x - x, which is NaN for both.
        return (0, f64::NAN, f64::NAN);
    }
    // set z = scalbn(|x|,ilogb(x)-23)
    let low = low_word(x);
    let e0 = (ix >> 20) - 1046; // e0 = ilogb(z)-23;
    let mut z = from_words(ix.wrapping_sub(((e0 as u32) << 20) as i32) as u32, low);
    let mut tx = [0.0f64; 3];
    for t in tx.iter_mut().take(2) {
        *t = (z as i32) as f64;
        z = (z - *t) * TWO24;
    }
    tx[2] = z;
    let mut nx = 3;
    while tx[nx - 1] == ZERO {
        nx -= 1; // skip zero term
    }
    let mut y = [0.0f64; 2];
    let n = kernel_rem_pio2(&tx[..nx], &mut y, e0, &TWO_OVER_PI);
    if hx < 0 {
        return (-n, -y[0], -y[1]);
    }
    (n, y[0], y[1])
}

/// `__kernel_rem_pio2(x, y, e0, nx, prec = 2, ipio2)`: the last three bits of `N` with
/// `y = x - N*pi/2`, for `x` split into 24-bit pieces with exponent `e0`.
fn kernel_rem_pio2(x: &[f64], y: &mut [f64; 2], e0: i32, ipio2: &[i32]) -> i32 {
    const PIO2: [f64; 8] = [
        1.57079625129699707031e+00, // 0x3FF921FB, 0x40000000
        7.54978941586159635335e-08, // 0x3E74442D, 0x00000000
        5.39030252995776476554e-15, // 0x3CF84698, 0x80000000
        3.28200341580791294123e-22, // 0x3B78CC51, 0x60000000
        1.27065575308067607349e-29, // 0x39F01B83, 0x80000000
        1.22933308981111328932e-36, // 0x387A2520, 0x40000000
        2.73370053816464559624e-44, // 0x36E38222, 0x80000000
        2.16741683877804819444e-51, // 0x3569F31D, 0x00000000
    ];
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
    const TWO24: f64 = 1.67772160000000000000e+07; // 0x41700000, 0x00000000
    const TWON24: f64 = 5.96046447753906250000e-08; // 0x3E700000, 0x00000000

    // init_jk[prec] with prec = 2 (64-bit precision).
    let jk: i32 = 4;
    let jp = jk;
    let nx = x.len() as i32;

    let mut iq = [0i32; 20];
    let mut f = [0.0f64; 20];
    let mut fq = [0.0f64; 20];
    let mut q = [0.0f64; 20];
    let u = |i: i32| i as usize;

    // determine jx,jv,q0, note that 3>q0
    let jx = nx - 1;
    let mut jv = (e0 - 3) / 24;
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 24 * (jv + 1);

    // set up f[0] to f[jx+jk] where f[jx+jk] = ipio2[jv+jk]
    let m = jx + jk;
    for (i, j) in (0..=m).zip(jv - jx..) {
        f[u(i)] = if j < 0 { ZERO } else { ipio2[u(j)] as f64 };
    }

    // compute q[0],q[1],...q[jk]
    for i in 0..=jk {
        let mut fw = 0.0;
        for j in 0..=jx {
            fw = fma(x[u(j)], f[u(jx + i - j)], fw);
        }
        q[u(i)] = fw;
    }

    let mut jz = jk;
    let mut z;
    let mut n;
    let mut ih;
    loop {
        // distill q[] into iq[] reversingly
        let mut i = 0;
        let mut j = jz;
        z = q[u(jz)];
        while j > 0 {
            let fw = ((TWON24 * z) as i32) as f64;
            iq[u(i)] = fma(-TWO24, fw, z) as i32;
            z = q[u(j - 1)] + fw;
            i += 1;
            j -= 1;
        }

        // compute n
        z = scalbn(z, q0); // actual value of z
        z = fma(-8.0, (z * 0.125).floor(), z); // trim off integer >= 8
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0 {
            // need iq[jz-1] to determine n
            let i = iq[u(jz - 1)] >> (24 - q0);
            n += i;
            iq[u(jz - 1)] -= i << (24 - q0);
            ih = iq[u(jz - 1)] >> (23 - q0);
        } else if q0 == 0 {
            ih = iq[u(jz - 1)] >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }

        if ih > 0 {
            // q > 0.5
            n += 1;
            let mut carry = 0;
            for i in 0..jz {
                // compute 1-q
                let j = iq[u(i)];
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        iq[u(i)] = 0x1000000 - j;
                    }
                } else {
                    iq[u(i)] = 0xFFFFFF - j;
                }
            }
            if q0 > 0 {
                // rare case: chance is 1 in 12
                match q0 {
                    1 => iq[u(jz - 1)] &= 0x7FFFFF,
                    2 => iq[u(jz - 1)] &= 0x3FFFFF,
                    _ => {}
                }
            }
            if ih == 2 {
                z = ONE - z;
                if carry != 0 {
                    z -= scalbn(ONE, q0);
                }
            }
        }

        // check if recomputation is needed
        if z == ZERO {
            let mut j = 0;
            let mut i = jz - 1;
            while i >= jk {
                j |= iq[u(i)];
                i -= 1;
            }
            if j == 0 {
                // need recomputation
                let mut k = 1;
                while jk >= k && iq[u(jk - k)] == 0 {
                    k += 1; // k = no. of terms needed
                }
                for i in (jz + 1)..=(jz + k) {
                    // add q[jz+1] to q[jz+k]
                    f[u(jx + i)] = ipio2[u(jv + i)] as f64;
                    let mut fw = 0.0;
                    for j in 0..=jx {
                        fw = fma(x[u(j)], f[u(jx + i - j)], fw);
                    }
                    q[u(i)] = fw;
                }
                jz += k;
                continue;
            }
        }
        break;
    }

    // chop off zero terms
    if z == 0.0 {
        jz -= 1;
        q0 -= 24;
        while iq[u(jz)] == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        // break z into 24-bit if necessary
        z = scalbn(z, -q0);
        if z >= TWO24 {
            let fw = ((TWON24 * z) as i32) as f64;
            iq[u(jz)] = fma(-TWO24, fw, z) as i32;
            jz += 1;
            q0 += 24;
            iq[u(jz)] = fw as i32;
        } else {
            iq[u(jz)] = z as i32;
        }
    }

    // convert integer "bit" chunk to floating-point value
    let mut fw = scalbn(ONE, q0);
    let mut i = jz;
    while i >= 0 {
        q[u(i)] = fw * iq[u(i)] as f64;
        fw *= TWON24;
        i -= 1;
    }

    // compute PIo2[0,...,jp]*q[jz,...,0]
    let mut i = jz;
    while i >= 0 {
        let mut fw = 0.0;
        let mut k = 0;
        while k <= jp && k <= jz - i {
            fw = fma(PIO2[u(k)], q[u(i + k)], fw);
            k += 1;
        }
        fq[u(jz - i)] = fw;
        i -= 1;
    }

    // compress fq[] into y[] (prec 2)
    let mut fw = 0.0;
    let mut i = jz;
    while i >= 0 {
        fw += fq[u(i)];
        i -= 1;
    }
    y[0] = if ih == 0 { fw } else { -fw };
    fw = fq[0] - fw;
    for i in 1..=jz {
        fw += fq[u(i)];
    }
    y[1] = if ih == 0 { fw } else { -fw };
    n & 7
}

/// `__kernel_cos(x, y)`: cos on [-pi/4, pi/4], `y` the tail of `x`.
fn k_cos(x: f64, y: f64) -> f64 {
    const ONE: f64 = 1.00000000000000000000e+00; // 0x3FF00000, 0x00000000
    const C1: f64 = 4.16666666666666019037e-02; // 0x3FA55555, 0x5555554C
    const C2: f64 = -1.38888888888741095749e-03; // 0xBF56C16C, 0x16C15177
    const C3: f64 = 2.48015872894767294178e-05; // 0x3EFA01A0, 0x19CB1590
    const C4: f64 = -2.75573143513906633035e-07; // 0xBE927E4F, 0x809C52AD
    const C5: f64 = 2.08757232129817482790e-09; // 0x3E21EE9E, 0xBDB4B1C4
    const C6: f64 = -1.13596475577881948265e-11; // 0xBDA8FAE9, 0xBE8838D4

    let ix = high_word(x) & 0x7FFFFFFF; // ix = |x|'s high word
    if ix < 0x3E400000 {
        // if x < 2**27; (int)x == 0 always holds here
        return ONE;
    }
    let z = x * x;
    let r = z * horner(z, &[C1, C2, C3, C4, C5, C6]);
    if ix < 0x3FD33333 {
        // if |x| < 0.3
        ONE - fma(0.5, z, -fma(z, r, -(x * y)))
    } else {
        let qx = if ix > 0x3FE90000 {
            // x > 0.78125
            0.28125
        } else {
            from_words((ix - 0x00200000) as u32, 0) // x/4
        };
        let iz = fma(0.5, z, -qx);
        let a = ONE - qx;
        a - (iz - fma(z, r, -(x * y)))
    }
}

/// `__kernel_sin(x, y, iy)`: sin on [-pi/4, pi/4], `y` the tail of `x` (0 when `iy == 0`).
fn k_sin(x: f64, y: f64, iy: i32) -> f64 {
    const HALF: f64 = 5.00000000000000000000e-01; // 0x3FE00000, 0x00000000
    const S1: f64 = -1.66666666666666324348e-01; // 0xBFC55555, 0x55555549
    const S2: f64 = 8.33333333332248946124e-03; // 0x3F811111, 0x1110F8A6
    const S3: f64 = -1.98412698298579493134e-04; // 0xBF2A01A0, 0x19C161D5
    const S4: f64 = 2.75573137070700676789e-06; // 0x3EC71DE3, 0x57B1FE7D
    const S5: f64 = -2.50507602534068634195e-08; // 0xBE5AE5E6, 0x8A2B9CEB
    const S6: f64 = 1.58969099521155010221e-10; // 0x3DE5D93A, 0x5ACFD57C

    let ix = high_word(x) & 0x7FFFFFFF; // high word of x
    if ix < 0x3E400000 {
        // |x| < 2**-27; (int)x == 0 always holds here
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = horner(z, &[S2, S3, S4, S5, S6]);
    if iy == 0 {
        fma(v, fma(z, r, S1), x)
    } else {
        x - fma(-v, S1, fma(z, fma(HALF, y, -(v * r)), -y))
    }
}

/// `__kernel_tan(x, y, iy)`: tan (`iy == 1`) or -1/tan (`iy == -1`) on [-pi/4, pi/4].
fn k_tan(mut x: f64, mut y: f64, iy: i32) -> f64 {
    const T: [f64; 13] = [
        3.33333333333334091986e-01,  // 3FD55555, 55555563
        1.33333333333201242699e-01,  // 3FC11111, 1110FE7A
        5.39682539762260521377e-02,  // 3FABA1BA, 1BB341FE
        2.18694882948595424599e-02,  // 3F9664F4, 8406D637
        8.86323982359930005737e-03,  // 3F8226E3, E96E8493
        3.59207910759131235356e-03,  // 3F6D6D22, C9560328
        1.45620945432529025516e-03,  // 3F57DBC8, FEE08315
        5.88041240820264096874e-04,  // 3F4344D8, F2F26501
        2.46463134818469906812e-04,  // 3F3026F7, 1A8D1068
        7.81794442939557092300e-05,  // 3F147E88, A03792A6
        7.14072491382608190305e-05,  // 3F12B80F, 32F0A7E9
        -1.85586374855275456654e-05, // BEF375CB, DB605373
        2.59073051863633712884e-05,  // 3EFB2A70, 74BF7AD4
    ];
    const ONE: f64 = 1.00000000000000000000e+00; // 3FF00000, 00000000
    const PIO4: f64 = 7.85398163397448278999e-01; // 3FE921FB, 54442D18
    const PIO4LO: f64 = 3.06161699786838301793e-17; // 3C81A626, 33145C07

    let hx = high_word(x); // high word of x
    let ix = hx & 0x7FFFFFFF; // high word of |x|
    if ix < 0x3E300000 {
        // x < 2**-28; (int)x == 0 always holds here
        let low = low_word(x);
        if ((ix as u32 | low) | (iy + 1) as u32) == 0 {
            return ONE / x.abs();
        } else if iy == 1 {
            return x;
        } else {
            // compute -1 / (x+y) carefully
            let w = x + y;
            let z = with_low_word(w, 0);
            let v = y - (z - x);
            let a = -ONE / w;
            let t = with_low_word(a, 0);
            let s = fma(t, z, ONE);
            return fma(a, fma(t, v, s), t);
        }
    }
    if ix >= 0x3FE59428 {
        // |x| >= 0.6744
        if hx < 0 {
            x = -x;
            y = -y;
        }
        let z = PIO4 - x;
        let w = PIO4LO - y;
        x = z + w;
        y = 0.0;
    }
    let z = x * x;
    let w = z * z;
    // Break x^5*(T[1]+x^2*T[2]+...) into
    // x^5(T[1]+x^4*T[3]+...+x^20*T[11]) +
    // x^5(x^2*(T[2]+x^4*T[4]+...+x^22*[T12]))
    let mut r = horner(w, &[T[1], T[3], T[5], T[7], T[9], T[11]]);
    let v = z * horner(w, &[T[2], T[4], T[6], T[8], T[10], T[12]]);
    let s = z * x;
    r = fma(z, fma(s, r + v, y), y);
    r = fma(T[0], s, r);
    let w = x + r;
    if ix >= 0x3FE59428 {
        let v = iy as f64;
        return ((1 - ((hx >> 30) & 2)) as f64) * fma(-2.0, x - (w * w / (w + v) - r), v);
    }
    if iy == 1 {
        return w;
    }
    // if allow error up to 2 ulp, simply return -1.0 / (x+r) here
    // compute -1.0 / (x+r) accurately
    let z = with_low_word(w, 0);
    let v = r - (z - x); // z+v = r+x
    let a = -1.0 / w; // a = -1.0/w
    let t = with_low_word(a, 0);
    let s = fma(t, z, 1.0);
    fma(a, fma(t, v, s), t)
}

/// The numerator of R(x^2), the rational approximation of (asin(x)-x)/x^3 that `asin` and
/// `acos` share, as `pS0..pS5`; the polynomial is `z * (pS0 + z * (pS1 + ...))`.
const ASIN_P: [f64; 6] = [
    1.66666666666666657415e-01,  // pS0 0x3FC55555, 0x55555555
    -3.25565818622400915405e-01, // pS1 0xBFD4D612, 0x03EB6F7D
    2.01212532134862925881e-01,  // pS2 0x3FC9C155, 0x0E884455
    -4.00555345006794114027e-02, // pS3 0xBFA48228, 0xB5688F3B
    7.91534994289814532176e-04,  // pS4 0x3F49EFE0, 0x7501B288
    3.47933107596021167570e-05,  // pS5 0x3F023DE1, 0x0DFDF709
];

/// The denominator of R(x^2): `one + z * (qS1 + z * (qS2 + z * (qS3 + z * qS4)))`.
const ASIN_Q: [f64; 5] = [
    1.00000000000000000000e+00,  // one
    -2.40339491173441421878e+00, // qS1 0xC0033A27, 0x1C8A2D4B
    2.02094576023350569471e+00,  // qS2 0x40002AE5, 0x9C598AC8
    -6.88283971605453293030e-01, // qS3 0xBFE6066C, 0x1B8D0159
    7.70381505559019352791e-02,  // qS4 0x3FB3B8C5, 0xB12E9282
];

/// `acos(x)`.
pub(crate) fn acos(x: f64) -> f64 {
    const ONE: f64 = 1.00000000000000000000e+00; // 0x3FF00000, 0x00000000
    const PI: f64 = 3.14159265358979311600e+00; // 0x400921FB, 0x54442D18
    const PIO2_HI: f64 = 1.57079632679489655800e+00; // 0x3FF921FB, 0x54442D18
    const PIO2_LO: f64 = 6.12323399573676603587e-17; // 0x3C91A626, 0x33145C07

    let hx = high_word(x);
    let ix = hx & 0x7FFFFFFF;
    if ix >= 0x3FF00000 {
        // |x| >= 1
        let lx = low_word(x);
        if ((ix - 0x3FF00000) as u32 | lx) == 0 {
            // |x|==1
            if hx > 0 {
                return 0.0; // acos(1) = 0
            }
            return PI + 2.0 * PIO2_LO; // acos(-1)= pi
        }
        return f64::NAN; // acos(|x|>1) is NaN
    }
    if ix < 0x3FE00000 {
        // |x| < 0.5
        if ix <= 0x3C600000 {
            return PIO2_HI + PIO2_LO; // if|x|<2**-57
        }
        let z = x * x;
        let p = z * horner(z, &ASIN_P);
        let q = horner(z, &ASIN_Q);
        let r = p / q;
        PIO2_HI - (x - fma(-x, r, PIO2_LO))
    } else if hx < 0 {
        // x < -0.5
        let z = (ONE + x) * 0.5;
        let p = z * horner(z, &ASIN_P);
        let q = horner(z, &ASIN_Q);
        let s = z.sqrt();
        let r = p / q;
        let w = fma(r, s, -PIO2_LO);
        fma(-2.0, s + w, PI)
    } else {
        // x > 0.5
        let z = (ONE - x) * 0.5;
        let s = z.sqrt();
        let df = with_low_word(s, 0);
        let c = fma(-df, df, z) / (s + df);
        let p = z * horner(z, &ASIN_P);
        let q = horner(z, &ASIN_Q);
        let r = p / q;
        let w = fma(r, s, c);
        2.0 * (df + w)
    }
}

/// `asin(x)`.
pub(crate) fn asin(x: f64) -> f64 {
    const ONE: f64 = 1.00000000000000000000e+00; // 0x3FF00000, 0x00000000
    const HUGE: f64 = 1.000e+300;
    const PIO2_HI: f64 = 1.57079632679489655800e+00; // 0x3FF921FB, 0x54442D18
    const PIO2_LO: f64 = 6.12323399573676603587e-17; // 0x3C91A626, 0x33145C07
    const PIO4_HI: f64 = 7.85398163397448278999e-01; // 0x3FE921FB, 0x54442D18

    let hx = high_word(x);
    let ix = hx & 0x7FFFFFFF;
    if ix >= 0x3FF00000 {
        // |x|>= 1
        let lx = low_word(x);
        if ((ix - 0x3FF00000) as u32 | lx) == 0 {
            // asin(1)=+-pi/2 with inexact
            return fma(x, PIO2_HI, x * PIO2_LO);
        }
        return f64::NAN; // asin(|x|>1) is NaN
    } else if ix < 0x3FE00000 {
        // |x|<0.5
        let mut t = 0.0;
        if ix < 0x3E400000 {
            // if |x| < 2**-27
            if HUGE + x > ONE {
                return x; // return x with inexact if x!=0
            }
        } else {
            t = x * x;
        }
        let p = t * horner(t, &ASIN_P);
        let q = horner(t, &ASIN_Q);
        let w = p / q;
        return fma(x, w, x);
    }
    // 1> |x|>= 0.5
    let w = ONE - x.abs();
    let t = w * 0.5;
    let p = t * horner(t, &ASIN_P);
    let q = horner(t, &ASIN_Q);
    let s = t.sqrt();
    let t = if ix >= 0x3FEF3333 {
        // if |x| > 0.975
        let w = p / q;
        PIO2_HI - fma(2.0, fma(s, w, s), -PIO2_LO)
    } else {
        let w = with_low_word(s, 0);
        let c = fma(-w, w, t) / (s + w);
        let r = p / q;
        let p = fma(2.0 * s, r, -fma(-2.0, c, PIO2_LO));
        let q = fma(-2.0, w, PIO4_HI);
        PIO4_HI - (p - q)
    };
    if hx > 0 {
        t
    } else {
        -t
    }
}

/// `atan(x)`.
pub(crate) fn atan(x: f64) -> f64 {
    const ATANHI: [f64; 4] = [
        4.63647609000806093515e-01, // atan(0.5)hi 0x3FDDAC67, 0x0561BB4F
        7.85398163397448278999e-01, // atan(1.0)hi 0x3FE921FB, 0x54442D18
        9.82793723247329054082e-01, // atan(1.5)hi 0x3FEF730B, 0xD281F69B
        1.57079632679489655800e+00, // atan(inf)hi 0x3FF921FB, 0x54442D18
    ];
    const ATANLO: [f64; 4] = [
        2.26987774529616870924e-17, // atan(0.5)lo 0x3C7A2B7F, 0x222F65E2
        3.06161699786838301793e-17, // atan(1.0)lo 0x3C81A626, 0x33145C07
        1.39033110312309984516e-17, // atan(1.5)lo 0x3C700788, 0x7AF0CBBD
        6.12323399573676603587e-17, // atan(inf)lo 0x3C91A626, 0x33145C07
    ];
    const AT: [f64; 11] = [
        3.33333333333329318027e-01,  // 0x3FD55555, 0x5555550D
        -1.99999999998764832476e-01, // 0xBFC99999, 0x9998EBC4
        1.42857142725034663711e-01,  // 0x3FC24924, 0x920083FF
        -1.11111104054623557880e-01, // 0xBFBC71C6, 0xFE231671
        9.09088713343650656196e-02,  // 0x3FB745CD, 0xC54C206E
        -7.69187620504482999495e-02, // 0xBFB3B0F2, 0xAF749A6D
        6.66107313738753120669e-02,  // 0x3FB10D66, 0xA0D03D51
        -5.83357013379057348645e-02, // 0xBFADDE2D, 0x52DEFD9A
        4.97687799461593236017e-02,  // 0x3FA97B4B, 0x24760DEB
        -3.65315727442169155270e-02, // 0xBFA2B444, 0x2C6A6C2F
        1.62858201153657823623e-02,  // 0x3F90AD3A, 0xE322DA11
    ];
    const ONE: f64 = 1.0;
    const HUGE: f64 = 1.0e300;

    let hx = high_word(x);
    let ix = hx & 0x7FFFFFFF;
    if ix >= 0x44100000 {
        // if |x| >= 2^66
        let low = low_word(x);
        if ix > 0x7FF00000 || (ix == 0x7FF00000 && low != 0) {
            return x + x; // NaN
        }
        if hx > 0 {
            return ATANHI[3] + ATANLO[3];
        }
        return -ATANHI[3] - ATANLO[3];
    }
    let mut x = x;
    let id: i32;
    if ix < 0x3FDC0000 {
        // |x| < 0.4375
        if ix < 0x3E400000 {
            // |x| < 2^-27
            if HUGE + x > ONE {
                return x; // raise inexact
            }
        }
        id = -1;
    } else {
        x = x.abs();
        if ix < 0x3FF30000 {
            // |x| < 1.1875
            if ix < 0x3FE60000 {
                // 7/16 <=|x|<11/16
                id = 0;
                x = fma(2.0, x, -ONE) / (2.0 + x);
            } else {
                // 11/16<=|x|< 19/16
                id = 1;
                x = (x - ONE) / (x + ONE);
            }
        } else if ix < 0x40038000 {
            // |x| < 2.4375
            id = 2;
            x = (x - 1.5) / fma(1.5, x, ONE);
        } else {
            // 2.4375 <= |x| < 2^66
            id = 3;
            x = -1.0 / x;
        }
    }
    // end of argument reduction
    let z = x * x;
    let w = z * z;
    // break sum from i=0 to 10 aT[i]z**(i+1) into odd and even poly
    let s1 = z * horner(w, &[AT[0], AT[2], AT[4], AT[6], AT[8], AT[10]]);
    let s2 = w * horner(w, &[AT[1], AT[3], AT[5], AT[7], AT[9]]);
    if id < 0 {
        return fma(-x, s1 + s2, x);
    }
    let id = id as usize;
    let z = ATANHI[id] - (fma(x, s1 + s2, -ATANLO[id]) - x);
    if hx < 0 {
        -z
    } else {
        z
    }
}

/// `atan2(y, x)`.
pub(crate) fn atan2(y: f64, x: f64) -> f64 {
    const TINY: f64 = 1.0e-300;
    const ZERO: f64 = 0.0;
    const PI_O_4: f64 = 7.8539816339744827900E-01; // 0x3FE921FB, 0x54442D18
    const PI_O_2: f64 = 1.5707963267948965580E+00; // 0x3FF921FB, 0x54442D18
    const PI: f64 = 3.1415926535897931160E+00; // 0x400921FB, 0x54442D18
    const PI_LO: f64 = 1.2246467991473531772E-16; // 0x3CA1A626, 0x33145C07

    let hx = high_word(x);
    let lx = low_word(x);
    let ix = hx & 0x7FFFFFFF;
    let hy = high_word(y);
    let ly = low_word(y);
    let iy = hy & 0x7FFFFFFF;
    if (ix as u32 | ((lx | (lx as i32).wrapping_neg() as u32) >> 31)) > 0x7FF00000
        || (iy as u32 | ((ly | (ly as i32).wrapping_neg() as u32) >> 31)) > 0x7FF00000
    {
        return x + y; // x or y is NaN
    }
    if (hx.wrapping_sub(0x3FF00000) as u32 | lx) == 0 {
        return atan(y); // x=1.0
    }
    let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2); // 2*sign(x)+sign(y)

    // when y = 0
    if (iy as u32 | ly) == 0 {
        match m {
            0 | 1 => return y,      // atan(+-0,+anything)=+-0
            2 => return PI + TINY,  // atan(+0,-anything) = pi
            _ => return -PI - TINY, // atan(-0,-anything) =-pi
        }
    }
    // when x = 0
    if (ix as u32 | lx) == 0 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }

    // when x is INF
    if ix == 0x7FF00000 {
        if iy == 0x7FF00000 {
            return match m {
                0 => PI_O_4 + TINY,        // atan(+INF,+INF)
                1 => -PI_O_4 - TINY,       // atan(-INF,+INF)
                2 => 3.0 * PI_O_4 + TINY,  // atan(+INF,-INF)
                _ => -3.0 * PI_O_4 - TINY, // atan(-INF,-INF)
            };
        }
        return match m {
            0 => ZERO,       // atan(+...,+INF)
            1 => -ZERO,      // atan(-...,+INF)
            2 => PI + TINY,  // atan(+...,-INF)
            _ => -PI - TINY, // atan(-...,-INF)
        };
    }
    // when y is INF
    if iy == 0x7FF00000 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }

    // compute y/x
    let k = (iy - ix) >> 20;
    let z = if k > 60 {
        // |y/x| >  2**60
        m &= 1;
        fma(0.5, PI_LO, PI_O_2)
    } else if hx < 0 && k < -60 {
        0.0 // 0 > |y|/x > -2**-60
    } else {
        atan((y / x).abs()) // safe to do y/x
    };
    match m {
        0 => z,                // atan(+,+)
        1 => -z,               // atan(-,+)
        2 => PI - (z - PI_LO), // atan(+,-)
        _ => (z - PI_LO) - PI, // atan(-,-)
    }
}

/// `cos(x)`.
pub(crate) fn cos(x: f64) -> f64 {
    let z = 0.0;
    // |x| ~< pi/4
    let ix = high_word(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        k_cos(x, z)
    } else if ix >= 0x7FF00000 {
        // cos(Inf or NaN) is NaN
        f64::NAN // x - x
    } else {
        // argument reduction needed
        let (n, y0, y1) = rem_pio2(x);
        match n & 3 {
            0 => k_cos(y0, y1),
            1 => -k_sin(y0, y1, 1),
            2 => -k_cos(y0, y1),
            _ => k_sin(y0, y1, 1),
        }
    }
}

/// `sin(x)`.
pub(crate) fn sin(x: f64) -> f64 {
    let z = 0.0;
    // |x| ~< pi/4
    let ix = high_word(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        k_sin(x, z, 0)
    } else if ix >= 0x7FF00000 {
        // sin(Inf or NaN) is NaN
        f64::NAN // x - x
    } else {
        // argument reduction needed
        let (n, y0, y1) = rem_pio2(x);
        match n & 3 {
            0 => k_sin(y0, y1, 1),
            1 => k_cos(y0, y1),
            2 => -k_sin(y0, y1, 1),
            _ => -k_cos(y0, y1),
        }
    }
}

/// `tan(x)`.
pub(crate) fn tan(x: f64) -> f64 {
    let z = 0.0;
    // |x| ~< pi/4
    let ix = high_word(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        k_tan(x, z, 1)
    } else if ix >= 0x7FF00000 {
        // tan(Inf or NaN) is NaN
        f64::NAN // x - x
    } else {
        // argument reduction needed
        let (n, y0, y1) = rem_pio2(x);
        // 1 -> n even, -1 -> n odd
        k_tan(y0, y1, 1 - ((n & 1) << 1))
    }
}

/// `exp(x)`.
pub(crate) fn exp(x: f64) -> f64 {
    const ONE: f64 = 1.0;
    const HALF: [f64; 2] = [0.5, -0.5];
    const O_THRESHOLD: f64 = 7.09782712893383973096e+02; // 0x40862E42, 0xFEFA39EF
    const U_THRESHOLD: f64 = -7.45133219101941108420e+02; // 0xC0874910, 0xD52D3051
    const LN2HI: [f64; 2] = [
        6.93147180369123816490e-01,  // 0x3FE62E42, 0xFEE00000
        -6.93147180369123816490e-01, // 0xBFE62E42, 0xFEE00000
    ];
    const LN2LO: [f64; 2] = [
        1.90821492927058770002e-10,  // 0x3DEA39EF, 0x35793C76
        -1.90821492927058770002e-10, // 0xBDEA39EF, 0x35793C76
    ];
    const INVLN2: f64 = 1.44269504088896338700e+00; // 0x3FF71547, 0x652B82FE
    const P1: f64 = 1.66666666666666019037e-01; // 0x3FC55555, 0x5555553E
    const P2: f64 = -2.77777777770155933842e-03; // 0xBF66C16C, 0x16BEBD93
    const P3: f64 = 6.61375632143793436117e-05; // 0x3F11566A, 0xAF25DE2C
    const P4: f64 = -1.65339022054652515390e-06; // 0xBEBBBD41, 0xC5D26BF1
    const P5: f64 = 4.13813679705723846039e-08; // 0x3E663769, 0x72BEA4D0
    const E: f64 = 2.718281828459045; // 0x4005BF0A, 0x8B145769
    const HUGE: f64 = 1.0e+300;
    const TWOM1000: f64 = 9.33263618503218878990e-302; // 2**-1000=0x01700000,0
    const TWO1023: f64 = 8.988465674311579539e307; // 0x1p1023

    let mut x = x;
    let mut hi = 0.0;
    let mut lo = 0.0;
    let mut k: i32 = 0;
    let mut hx = high_word(x) as u32;
    let xsb = ((hx >> 31) & 1) as usize; // sign bit of x
    hx &= 0x7FFFFFFF; // high word of |x|

    // filter out non-finite argument
    if hx >= 0x40862E42 {
        // if |x|>=709.78...
        if hx >= 0x7FF00000 {
            let lx = low_word(x);
            if ((hx & 0xFFFFF) | lx) != 0 {
                return x + x; // NaN
            }
            return if xsb == 0 { x } else { 0.0 }; // exp(+-inf)={inf,0}
        }
        if x > O_THRESHOLD {
            return HUGE * HUGE; // overflow
        }
        if x < U_THRESHOLD {
            return TWOM1000 * TWOM1000; // underflow
        }
    }

    // argument reduction
    if hx > 0x3FD62E42 {
        // if  |x| > 0.5 ln2
        if hx < 0x3FF0A2B2 {
            // and |x| < 1.5 ln2
            // V8 special-cases exp(1) to return the correctly rounded E.
            if x == 1.0 {
                return E;
            }
            hi = x - LN2HI[xsb];
            lo = LN2LO[xsb];
            k = 1 - xsb as i32 - xsb as i32;
        } else {
            k = fma(INVLN2, x, HALF[xsb]) as i32;
            let t = k as f64;
            hi = fma(-t, LN2HI[0], x); // t*ln2HI is exact here
            lo = t * LN2LO[0];
        }
        x = hi - lo;
    } else if hx < 0x3E300000 {
        // when |x|<2**-28
        if HUGE + x > ONE {
            return ONE + x; // trigger inexact
        }
    } else {
        k = 0;
    }

    // x is now in primary range
    let t = x * x;
    let twopk = if k >= -1021 {
        from_words(0x3FF00000i32.wrapping_add(((k as u32) << 20) as i32) as u32, 0)
    } else {
        from_words(0x3FF00000u32.wrapping_add(((k + 1000) as u32) << 20), 0)
    };
    let c = fma(-t, horner(t, &[P1, P2, P3, P4, P5]), x);
    if k == 0 {
        return ONE - ((x * c) / (c - 2.0) - x);
    }
    let y = ONE - ((lo - (x * c) / (2.0 - c)) - hi);
    if k >= -1021 {
        if k == 1024 {
            return y * 2.0 * TWO1023;
        }
        return y * twopk;
    }
    y * twopk * TWOM1000
}

/// `Lg1..Lg7`: the polynomial for log(1+f) that `log`, `log2` and `log10` share.
const LG1: f64 = 6.666666666666735130e-01; // 3FE55555 55555593
const LG2: f64 = 3.999999999940941908e-01; // 3FD99999 9997FA04
const LG3: f64 = 2.857142874366239149e-01; // 3FD24924 94229359
const LG4: f64 = 2.222219843214978396e-01; // 3FCC71C5 1D8E78AF
const LG5: f64 = 1.818357216161805012e-01; // 3FC74664 96CB03DE
const LG6: f64 = 1.531383769920937332e-01; // 3FC39A09 D078C69F
const LG7: f64 = 1.479819860511658591e-01; // 3FC2F112 DF3E5244

/// `log(x)`.
pub(crate) fn log(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01; // 3fe62e42 fee00000
    const LN2_LO: f64 = 1.90821492927058770002e-10; // 3dea39ef 35793c76
    const TWO54: f64 = 1.80143985094819840000e+16; // 43500000 00000000
    const ZERO: f64 = 0.0;

    let mut x = x;
    let mut hx = high_word(x);
    let lx = low_word(x);

    let mut k: i32 = 0;
    if hx < 0x00100000 {
        // x < 2**-1022
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY; // log(+-0)=-inf
        }
        if hx < 0 {
            return f64::NAN; // log(-#) = NaN
        }
        k -= 54;
        x *= TWO54; // subnormal number, scale up x
        hx = high_word(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    let mut i = (hx + 0x95F64) & 0x100000;
    x = with_high_word(x, (hx | (i ^ 0x3FF00000)) as u32); // normalize x or x/2
    k += i >> 20;
    let f = x - 1.0;
    if (0x000FFFFF & (2 + hx)) < 3 {
        // -2**-20 <= f < 2**-20
        if f == ZERO {
            if k == 0 {
                return ZERO;
            }
            let dk = k as f64;
            return fma(dk, LN2_HI, dk * LN2_LO);
        }
        let r = f * f * fma(-0.33333333333333333, f, 0.5);
        if k == 0 {
            return f - r;
        }
        let dk = k as f64;
        return fma(dk, LN2_HI, -(fma(-dk, LN2_LO, r) - f));
    }
    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    i = hx - 0x6147A;
    let w = z * z;
    let j = 0x6B851 - hx;
    let t1 = w * horner(w, &[LG2, LG4, LG6]);
    let t2 = z * horner(w, &[LG1, LG3, LG5, LG7]);
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            return f - fma(-s, hfsq + r, hfsq);
        }
        fma(dk, LN2_HI, -((hfsq - fma(s, hfsq + r, dk * LN2_LO)) - f))
    } else {
        if k == 0 {
            return fma(-s, f - r, f);
        }
        fma(dk, LN2_HI, -(fma(s, f - r, -(dk * LN2_LO)) - f))
    }
}

/// `k_log1p(f)`: `log(1+f) - f + f*f/2` for `1+f` in [sqrt(2)/2, sqrt(2)].
#[inline(always)]
fn k_log1p(f: f64) -> f64 {
    let s = f / (2.0 + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * horner(w, &[LG2, LG4, LG6]);
    let t2 = z * horner(w, &[LG1, LG3, LG5, LG7]);
    let r = t2 + t1;
    let hfsq = 0.5 * f * f;
    s * (hfsq + r)
}

/// `log2(x)`.
pub(crate) fn log2(x: f64) -> f64 {
    const TWO54: f64 = 1.80143985094819840000e+16; // 0x43500000, 0x00000000
    const IVLN2HI: f64 = 1.44269504072144627571e+00; // 0x3FF71547, 0x65200000
    const IVLN2LO: f64 = 1.67517131648865118353e-10; // 0x3DE705FC, 0x2EEFA200

    let mut x = x;
    let mut hx = high_word(x);
    let lx = low_word(x);

    let mut k: i32 = 0;
    if hx < 0x00100000 {
        // x < 2**-1022
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY; // log(+-0)=-inf
        }
        if hx < 0 {
            return f64::NAN; // log(-#) = NaN
        }
        k -= 54;
        x *= TWO54; // subnormal number, scale up x
        hx = high_word(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    if hx == 0x3FF00000 && lx == 0 {
        return 0.0; // log(1) = +0
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    let i = (hx + 0x95F64) & 0x100000;
    x = with_high_word(x, (hx | (i ^ 0x3FF00000)) as u32); // normalize x or x/2
    k += i >> 20;
    let y = k as f64;
    let f = x - 1.0;
    let hfsq = 0.5 * f * f;
    let r = k_log1p(f);

    // f-hfsq must (for args near 1) be evaluated in extra precision to avoid a large
    // cancellation when x is near sqrt(2) or 1/sqrt(2); see e_log2.c.
    let hi = with_low_word(f - hfsq, 0);
    let lo = (f - hi) - hfsq + r;
    let mut val_hi = hi * IVLN2HI;
    let mut val_lo = fma(lo + hi, IVLN2LO, lo * IVLN2HI);

    // spadd(val_hi, val_lo, y), except for not using double_t:
    let w = y + val_hi;
    val_lo += (y - w) + val_hi;
    val_hi = w;

    val_lo + val_hi
}

/// `log10(x)`.
pub(crate) fn log10(x: f64) -> f64 {
    const TWO54: f64 = 1.80143985094819840000e+16; // 0x43500000, 0x00000000
    const IVLN10: f64 = 4.34294481903251816668e-01;
    const LOG10_2HI: f64 = 3.01029995663611771306e-01; // 0x3FD34413, 0x509F6000
    const LOG10_2LO: f64 = 3.69423907715893078616e-13; // 0x3D59FEF3, 0x11F12B36

    let mut x = x;
    let mut hx = high_word(x);
    let mut lx = low_word(x);

    let mut k: i32 = 0;
    if hx < 0x00100000 {
        // x < 2**-1022
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0 {
            return f64::NEG_INFINITY; // log(+-0)=-inf
        }
        if hx < 0 {
            return f64::NAN; // log(-#) = NaN
        }
        k -= 54;
        x *= TWO54; // subnormal number, scale up x
        hx = high_word(x);
        lx = low_word(x);
    }
    if hx >= 0x7FF00000 {
        return x + x;
    }
    if hx == 0x3FF00000 && lx == 0 {
        return 0.0; // log(1) = +0
    }
    k += (hx >> 20) - 1023;

    let i = ((k as u32 & 0x80000000) >> 31) as i32;
    hx = (hx & 0x000FFFFF) | ((0x3FF - i) << 20);
    let y = (k + i) as f64;
    x = from_words(hx as u32, lx);

    let z = fma(y, LOG10_2LO, IVLN10 * log(x));
    fma(y, LOG10_2HI, z)
}
