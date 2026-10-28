//! The newlib libm functions that the engine calls, as bit-exact ports of the implementations **in the shipped
//! v1.71 binary** (its newlib differs from the one of GCC 11.3; see docs/ARCHITECTURE.md). Never use the host
//! libm or `f32::exp` etc.
//!
//! Called from linked engine code of the reference ELF: `expf`, `logf` (Compressor_process), `exp`
//! (Compressor_setAttack / Compressor_setRelease), `fmod` (Distortion_process; `fmod` reaches
//! `__ieee754_fmod`). No other libm function is reached from the engine (checked on the call graph of the ELF:
//! the only other libm-internal callees are the `__math_*` error helpers and `__errno`).
//! `sinf` is only used by `sinCos.c`, which is not linked (Reverb is not part of the firmware); it is listed
//! for completeness. `sqrtf`/`fabsf`/`fabs`/`fmaf` compile to `vsqrt`/`vabs`/`vfma` instructions (use
//! `f32::sqrt`, `f32::abs`, `crate::fpu::fma`).
//!
//! # Where the shipped functions are
//!
//! The shipped binary has no symbols. The functions were found through the BL call sites of matched functions
//! (`function_match_v1.71_vs_gcc11.json`): Compressor_setAttack (0x08012294) → `exp` at 0x08029648,
//! Compressor_process (0x0801243c, 0x0801247a) → `logf` at 0x08029970 and `expf` at 0x08029888,
//! Distortion_process (0x08013568) → `fmod` at 0x08029600, which calls `__ieee754_fmod` at 0x08029ad0.
//!
//! | function          | shipped v1.71 | GCC 11.3 ELF |
//! |-------------------|---------------|--------------|
//! | `fmod`            | 0x08029600    | 0x08029a48   |
//! | `exp`             | 0x08029648    | 0x08029648   |
//! | `expf`            | 0x08029888    | 0x08029898   |
//! | `logf`            | 0x08029970    | 0x0802997c   |
//! | `__ieee754_fmod`  | 0x08029ad0    | 0x08029b38   |
//!
//! # Algorithms
//!
//! `exp`, `expf` and `logf` are the Arm Optimized Routines implementations (newlib `libm/common/exp.c`,
//! `sf_exp.c`, `sf_log.c`: EXP_TABLE_BITS = 7 / EXP_POLY_ORDER = 5, EXP2F_TABLE_BITS = 5, LOGF_TABLE_BITS = 4),
//! with every `a * b + c` contracted to `vfma.f64` exactly as below. `fmod` is the fdlibm `w_fmod.c` wrapper and
//! `e_fmod.c` (integer shift-subtract; the final `x *= one` of the subnormal path is not present in the machine
//! code). The shipped and GCC 11.3 machine code differ only in instruction scheduling and register allocation:
//! the sequence of floating point operations, the fusing, the branch conditions, the tables (`__exp_data`,
//! `__exp2f_data`, `__logf_data`, `Zero[]`) and the literal pool constants are identical (the tables and
//! constants are generated from the shipped binary by `tools/gen_libm_tables.py`, which also checks them
//! against the ELF). So one implementation serves both; no ELF-specific variant is needed.
//!
//! # Side effects
//!
//! The overflow / underflow / domain paths store `ERANGE` (34) or `EDOM` (33) into `errno`
//! (`_impure_ptr->_errno`, via `__errno`). The firmware never reads `errno` (only `syscalls.c` writes it), so
//! errno is not modelled. The floating point exception flags (FPSCR cumulative bits) are not modelled either.
//!
//! # Floating point environment
//!
//! NaN results follow the Cortex-M7 with FPSCR.DN = 0 (the firmware never sets DN): an input NaN is returned
//! quieted, invalid operations give the default NaN `0x7fc00000` / `0x7ff8000000000000` (positive, unlike the
//! x86 default NaN). These are produced explicitly, not by host arithmetic.
//!
//! Flush-to-zero: the engine runs with FPSCR = 0 (FZ off; docs/ARCHITECTURE.md), which is the mode that
//! matters. The functions nevertheless also reproduce FZ = 1 when called under `crate::fpu::FtzGuard`
//! (both modes are difftested).
//! The cases where the host would differ from ARM FZ (ARM flushes a result that is tiny *before* rounding)
//! are handled explicitly: the final `vcvt.f32.f64` of `expf` and the subnormal result path of `exp`.

#![allow(non_snake_case)]

mod tables;
use tables::*;

use crate::fpu::vfma64;
use core::hint::black_box;

/// ARM default NaN (FPSCR.DN = 0 operations that are invalid), single precision.
const DEFAULT_NAN_F32: u32 = 0x7fc0_0000;
/// ARM default NaN, double precision.
const DEFAULT_NAN_F64: u64 = 0x7ff8_0000_0000_0000;

/// True if the host currently flushes subnormal results to zero (`FtzGuard` active), i.e. the firmware code
/// being reproduced runs with FPSCR.FZ = 1.
#[inline(never)]
fn host_fz() -> bool {
    black_box(f64::MIN_POSITIVE) * black_box(0.5f64) == 0.0
}

#[inline(always)]
fn d(bits: u64) -> f64 {
    f64::from_bits(bits)
}

// ---------------------------------------------------------------------------------------------------------
// __math_* error helpers (math_err.c / math_errf.c). The `errno` stores are not modelled (see module doc).
// The multiplications are done at run time (black_box) like the machine code, so that the result follows the
// FZ mode (e.g. `__math_may_uflowf` gives 0x1p-149 with FZ = 0 and +0 with FZ = 1).
// ---------------------------------------------------------------------------------------------------------

/// `xflow(sign, y)`: `(sign ? -y : y) * y`.
#[inline(never)]
fn xflow(sign: u32, y: f64) -> f64 {
    let t = if sign != 0 { -y } else { y };
    black_box(t) * black_box(y)
}

/// `__math_uflow(sign)`.
fn __math_uflow(sign: u32) -> f64 {
    xflow(sign, MATH_UFLOW)
}

/// `__math_oflow(sign)`.
fn __math_oflow(sign: u32) -> f64 {
    xflow(sign, MATH_OFLOW)
}

/// `__math_check_uflow(y)`: returns `y` (errno only).
#[inline(always)]
fn __math_check_uflow(y: f64) -> f64 {
    y
}

/// `__math_check_oflow(y)`: returns `y` (errno only when `|y| > MATH_CHECK_OFLOW_MAX`).
#[inline(always)]
fn __math_check_oflow(y: f64) -> f64 {
    let _ = MATH_CHECK_OFLOW_MAX;
    y
}

/// `xflowf(sign, y)`: `(sign ? -y : y) * y`.
#[inline(never)]
fn xflowf(sign: u32, y: f32) -> f32 {
    let t = if sign != 0 { -y } else { y };
    black_box(t) * black_box(y)
}

fn __math_uflowf(sign: u32) -> f32 {
    xflowf(sign, MATH_UFLOWF)
}

fn __math_may_uflowf(sign: u32) -> f32 {
    xflowf(sign, MATH_MAY_UFLOWF)
}

fn __math_oflowf(sign: u32) -> f32 {
    xflowf(sign, MATH_OFLOWF)
}

/// `__math_divzerof(sign)`: `(sign ? -1.0f : 1.0f) / 0.0f` = ∓inf.
fn __math_divzerof(sign: u32) -> f32 {
    let _ = MATH_DIVZEROF_ZERO;
    if sign != 0 {
        f32::NEG_INFINITY
    } else {
        f32::INFINITY
    }
}

/// `__math_invalidf(x)`: `(x - x) / (x - x)`: the quieted input NaN, or the default NaN.
fn __math_invalidf(x: f32) -> f32 {
    let ix = x.to_bits();
    if (ix & 0x7fff_ffff) > 0x7f80_0000 {
        f32::from_bits(ix | 0x0040_0000)
    } else {
        f32::from_bits(DEFAULT_NAN_F32)
    }
}

// ---------------------------------------------------------------------------------------------------------
// exp (double), shipped 0x08029648.
// __exp_data word indices (byte offset / 8): 0 invln2N, 1 shift, 2 negln2hiN, 3 negln2loN, 4..7 poly C2..C5,
// 14.. tab[2*N] (tail, sbits pairs).
// ---------------------------------------------------------------------------------------------------------

const EXP_TABLE_BITS: u32 = 7;
const EXP_N: u64 = 1 << EXP_TABLE_BITS;
const EXP_TAB: usize = 14;

#[inline(always)]
fn top12(x: f64) -> u32 {
    (x.to_bits() >> 52) as u32
}

/// `specialcase(tmp, sbits, ki)` of exp.c (|x| > 512 or result subnormal).
fn exp_specialcase(tmp: f64, sbits: u64, ki: u64) -> f64 {
    if ki & 0x8000_0000 == 0 {
        // k > 0, the exponent of scale might have overflowed by <= 460.
        let sbits = sbits.wrapping_sub(1009u64 << 52);
        let scale = d(sbits);
        let y = EXP_SPECIAL_POS_SCALE * vfma64(scale, tmp, scale);
        return __math_check_oflow(y);
    }
    // k < 0, need special care in the subnormal range.
    let sbits = sbits.wrapping_add(1022u64 << 52);
    let scale = d(sbits);
    let p = tmp * scale; // vmul (not fused)
    let mut y = scale + p;
    if y < 1.0 {
        // Round y to the right precision before scaling it into the subnormal range to avoid double rounding.
        let mut lo = scale - y;
        lo = lo + p;
        let hi = y + 1.0;
        let mut t = 1.0 - hi;
        t = t + y;
        t = t + lo;
        t = t + hi;
        y = t - 1.0;
        if y == 0.0 {
            y = EXP_SPECIAL_ZERO;
        }
        // force_eval(0x1p-1022 * 0x1p-1022) only raises flags.
        if host_fz() {
            // ARM FZ: 0x1p-1022 * y with 0 <= y < 1 is tiny before rounding and is flushed to +0.
            return __math_check_uflow(0.0);
        }
    }
    let y = EXP_SPECIAL_NEG_SCALE * y;
    __math_check_uflow(y)
}

/// newlib `double exp(double)` (Arm Optimized Routines, shipped v1.71 at 0x08029648).
pub fn exp(x: f64) -> f64 {
    let ix = x.to_bits();
    let mut abstop = top12(x) & 0x7ff;
    if abstop.wrapping_sub(0x3c9) >= 0x3f {
        if (abstop.wrapping_sub(0x3c9) as i32) < 0 {
            // Avoid spurious underflow for tiny x. Note: 0 is common input.
            return 1.0 + x;
        }
        if abstop >= 0x409 {
            if ix == f64::NEG_INFINITY.to_bits() {
                return EXP_SPECIAL_ZERO;
            }
            if abstop >= 0x7ff {
                return exp_nan_inf_add(x);
            }
            if ix >> 63 != 0 {
                return __math_uflow(0);
            } else {
                return __math_oflow(0);
            }
        }
        // Large x is special cased below.
        abstop = 0;
    }
    let T = &EXP_DATA;
    // exp(x) = 2^(k/N) * exp(r), with exp(r) in [2^(-1/2N),2^(1/2N)].
    let kd = vfma64(d(T[1]), d(T[0]), x);
    let ki = kd.to_bits();
    let kd = kd - d(T[1]);
    let r = vfma64(x, d(T[2]), kd);
    let r = vfma64(r, d(T[3]), kd);
    // 2^(k/N) ~= scale * (1 + tail).
    let idx = 2 * (ki % EXP_N) as usize;
    let top = ki << (52 - EXP_TABLE_BITS);
    let tail = d(T[EXP_TAB + idx]);
    // This is only a valid scale when -1023*N < k < 1024*N.
    let sbits = T[EXP_TAB + idx + 1].wrapping_add(top);
    let r2 = r * r;
    let mut tmp = r + tail;
    tmp = vfma64(tmp, vfma64(d(T[4]), d(T[5]), r), r2);
    let r4 = r2 * r2;
    tmp = vfma64(tmp, r4, vfma64(d(T[6]), d(T[7]), r));
    if abstop == 0 {
        return exp_specialcase(tmp, sbits, ki);
    }
    let scale = d(sbits);
    vfma64(scale, tmp, scale)
}

/// `1.0 + x` for x = +inf or NaN (ARM: a NaN input is returned quieted).
#[inline(always)]
fn exp_nan_inf_add(x: f64) -> f64 {
    if x.is_nan() {
        d(x.to_bits() | 0x0008_0000_0000_0000)
    } else {
        x
    }
}

// ---------------------------------------------------------------------------------------------------------
// expf, shipped 0x08029888.
// __exp2f_data word indices: 0..31 tab, 32 shift_scaled, 33..35 poly, 36 shift, 37 invln2_scaled,
// 38..40 poly_scaled.
// ---------------------------------------------------------------------------------------------------------

const EXP2F_TABLE_BITS: u32 = 5;
const EXP2F_N: u64 = 1 << EXP2F_TABLE_BITS;

/// newlib `float expf(float)` (Arm Optimized Routines, shipped v1.71 at 0x08029888).
pub fn expf(x: f32) -> f32 {
    let ix = x.to_bits();
    let xd = x as f64; // vcvt.f64.f32 (a subnormal x is flushed under FZ, like the host with FtzGuard)
    let abstop = (ix >> 20) & 0x7ff;
    if abstop > 0x42a {
        // |x| >= 88 or x is nan.
        if ix == f32::NEG_INFINITY.to_bits() {
            return EXPF_ZERO;
        }
        if abstop >= 0x7f8 {
            // x + x: inf, or the quieted NaN.
            return if x.is_nan() { f32::from_bits(ix | 0x0040_0000) } else { x };
        }
        if x > EXPF_OFLOW_BOUND {
            return __math_oflowf(0);
        }
        if x < EXPF_UFLOW_BOUND {
            return __math_uflowf(0);
        }
        if x < EXPF_MAY_UFLOW_BOUND {
            return __math_may_uflowf(0);
        }
    }
    let T = &EXP2F_DATA;
    let invln2 = d(T[37]);
    let shift = d(T[36]);
    // x*N/Ln2 = k + r with r in [-1/2, 1/2] and int k.
    let kd = vfma64(shift, invln2, xd);
    let ki = kd.to_bits();
    let kd = kd - shift;
    let r = vfma64(-kd, invln2, xd); // vfnms: z - kd with z = InvLn2N * xd fused
    // exp(x) = 2^(k/N) * 2^(r/N) ~= s * (C0*r^3 + C1*r^2 + C2*r + 1)
    let t = T[(ki % EXP2F_N) as usize].wrapping_add((ki as u32 as u64) << (52 - EXP2F_TABLE_BITS));
    let s = d(t);
    let z = vfma64(d(T[39]), d(T[38]), r);
    let r2 = r * r;
    let y = vfma64(1.0, d(T[40]), r);
    let y = vfma64(y, z, r2);
    let y = y * s;
    cvt_f32_f64(y)
}

/// `vcvt.f32.f64` of a positive value with the FZ semantics of the Cortex-M7: with FZ = 1 a result that is
/// below the smallest normal *before* rounding is flushed to +0 (hosts may round it up to 0x1p-126 first).
#[inline(always)]
fn cvt_f32_f64(y: f64) -> f32 {
    if y < d(0x3810_0000_0000_0000) && host_fz() {
        return 0.0;
    }
    y as f32
}

// ---------------------------------------------------------------------------------------------------------
// logf, shipped 0x08029970.
// __logf_data word indices: 0..31 tab {invc, logc}, 32 ln2, 33..35 poly A0..A2.
// ---------------------------------------------------------------------------------------------------------

const LOGF_TABLE_BITS: u32 = 4;
const LOGF_N: u32 = 1 << LOGF_TABLE_BITS;
const LOGF_OFF: u32 = 0x3f33_0000;

/// newlib `float logf(float)` (Arm Optimized Routines, shipped v1.71 at 0x08029970).
pub fn logf(x: f32) -> f32 {
    let mut ix = x.to_bits();
    // Fix sign of zero with downward rounding when x==1.
    if ix == 0x3f80_0000 {
        return LOGF_ZERO;
    }
    if ix.wrapping_sub(0x0080_0000) >= 0x7f80_0000 - 0x0080_0000 {
        // x < 0x1p-126 or inf or nan.
        if ix.wrapping_mul(2) == 0 {
            return __math_divzerof(1);
        }
        if ix == 0x7f80_0000 {
            // log(inf) == inf.
            return x;
        }
        if ix & 0x8000_0000 != 0 || ix.wrapping_mul(2) >= 0xff00_0000 {
            return __math_invalidf(x);
        }
        // x is subnormal, normalize it (under FZ the product is +0, as on the Cortex-M7).
        ix = (x * LOGF_SUBNORMAL_SCALE).to_bits();
        ix = ix.wrapping_sub(23 << 23);
    }
    let T = &LOGF_DATA;
    // x = 2^k z; where z is in range [OFF,2*OFF] and exact.
    let tmp = ix.wrapping_sub(LOGF_OFF);
    let i = ((tmp >> (23 - LOGF_TABLE_BITS)) % LOGF_N) as usize;
    let k = (tmp as i32) >> 23; // arithmetic shift
    let iz = ix.wrapping_sub(tmp & 0xff80_0000);
    let invc = d(T[2 * i]);
    let logc = d(T[2 * i + 1]);
    let z = f32::from_bits(iz) as f64;
    // log(x) = log1p(z/c-1) + log(c) + k*Ln2
    let r = vfma64(-1.0, invc, z);
    let y0 = vfma64(logc, k as f64, d(T[32]));
    // Pipelined polynomial evaluation to approximate log1p(r).
    let r2 = r * r;
    let y = vfma64(d(T[35]), d(T[34]), r);
    let y = vfma64(y, d(T[33]), r2);
    let p = r + y0;
    let y = vfma64(p, r2, y);
    y as f32 // |y| >= ~1e-7 or 0: never subnormal
}

// ---------------------------------------------------------------------------------------------------------
// fmod: wrapper (w_fmod.c) shipped 0x08029600, __ieee754_fmod (e_fmod.c) shipped 0x08029ad0.
// ---------------------------------------------------------------------------------------------------------

/// The NaN that `x * y` produces on ARM (FPSCR.DN = 0) when an operand is a NaN: the first signalling NaN,
/// else the first quiet NaN, quieted.
fn arm_nan_operand(x: u64, y: u64) -> Option<u64> {
    let is_nan = |v: u64| (v & 0x7fff_ffff_ffff_ffff) > 0x7ff0_0000_0000_0000;
    let is_snan = |v: u64| is_nan(v) && v & 0x0008_0000_0000_0000 == 0;
    let q = |v: u64| v | 0x0008_0000_0000_0000;
    if is_snan(x) {
        Some(q(x))
    } else if is_snan(y) {
        Some(q(y))
    } else if is_nan(x) {
        Some(x)
    } else if is_nan(y) {
        Some(y)
    } else {
        None
    }
}

/// `__ieee754_fmod(x, y)` (fdlibm e_fmod.c), shipped v1.71 at 0x08029ad0. Exact; integer arithmetic only.
pub fn __ieee754_fmod(x: f64, y: f64) -> f64 {
    let xb = x.to_bits();
    let yb = y.to_bits();
    let mut hx = (xb >> 32) as i32;
    let mut lx = xb as u32;
    let mut hy = (yb >> 32) as i32;
    let mut ly = yb as u32;
    let sx = (hx as u32 & 0x8000_0000) as i32;
    hx ^= sx; // |x|
    hy &= 0x7fff_ffff; // |y|

    // Purge off exception values: y = 0, or x not finite, or y is NaN: (x*y)/(x*y).
    if (hy as u32 | ly) == 0
        || hx >= 0x7ff0_0000
        || ((hy as u32) | ((ly | ly.wrapping_neg()) >> 31)) > 0x7ff0_0000
    {
        // x*y is a NaN operand (quieted), or ±0 / ±inf, and p/p of those is that NaN or the default NaN.
        return d(arm_nan_operand(xb, yb).unwrap_or(DEFAULT_NAN_F64));
    }
    if hx <= hy {
        if hx < hy || lx < ly {
            return x; // |x| < |y|: return x
        }
        if lx == ly {
            return d(FMOD_ZERO[(sx as u32 >> 31) as usize]); // |x| = |y|: return x*0
        }
    }

    // Determine ix = ilogb(x).
    let mut ix: i32;
    if hx < 0x0010_0000 {
        // subnormal x
        if hx == 0 {
            ix = -1043;
            let mut i = lx as i32;
            while i > 0 {
                ix -= 1;
                i = i.wrapping_shl(1);
            }
        } else {
            ix = -1022;
            let mut i = hx.wrapping_shl(11);
            while i > 0 {
                ix -= 1;
                i = i.wrapping_shl(1);
            }
        }
    } else {
        ix = (hx >> 20) - 1023;
    }

    // Determine iy = ilogb(y).
    let mut iy: i32;
    if hy < 0x0010_0000 {
        // subnormal y
        if hy == 0 {
            iy = -1043;
            let mut i = ly as i32;
            while i > 0 {
                iy -= 1;
                i = i.wrapping_shl(1);
            }
        } else {
            iy = -1022;
            let mut i = hy.wrapping_shl(11);
            while i > 0 {
                iy -= 1;
                i = i.wrapping_shl(1);
            }
        }
    } else {
        iy = (hy >> 20) - 1023;
    }

    // Set up {hx,lx}, {hy,ly} and align y to x.
    if ix >= -1022 {
        hx = 0x0010_0000 | (0x000f_ffff & hx);
    } else {
        // subnormal x, shift x to normal
        let n = -1022 - ix;
        if n <= 31 {
            hx = (hx.wrapping_shl(n as u32)) | (lx.wrapping_shr((32 - n) as u32)) as i32;
            lx = lx.wrapping_shl(n as u32);
        } else {
            hx = lx.wrapping_shl((n - 32) as u32) as i32;
            lx = 0;
        }
    }
    if iy >= -1022 {
        hy = 0x0010_0000 | (0x000f_ffff & hy);
    } else {
        // subnormal y, shift y to normal
        let n = -1022 - iy;
        if n <= 31 {
            hy = (hy.wrapping_shl(n as u32)) | (ly.wrapping_shr((32 - n) as u32)) as i32;
            ly = ly.wrapping_shl(n as u32);
        } else {
            hy = ly.wrapping_shl((n - 32) as u32) as i32;
            ly = 0;
        }
    }

    // Fix point fmod.
    let mut n = ix - iy;
    while n != 0 {
        n -= 1;
        let mut hz = hx.wrapping_sub(hy);
        let lz = lx.wrapping_sub(ly);
        if lx < ly {
            hz = hz.wrapping_sub(1);
        }
        if hz < 0 {
            hx = hx.wrapping_add(hx).wrapping_add((lx >> 31) as i32);
            lx = lx.wrapping_add(lx);
        } else {
            if (hz as u32 | lz) == 0 {
                // return sign(x)*0
                return d(FMOD_ZERO[(sx as u32 >> 31) as usize]);
            }
            hx = hz.wrapping_add(hz).wrapping_add((lz >> 31) as i32);
            lx = lz.wrapping_add(lz);
        }
    }
    let mut hz = hx.wrapping_sub(hy);
    let lz = lx.wrapping_sub(ly);
    if lx < ly {
        hz = hz.wrapping_sub(1);
    }
    if hz >= 0 {
        hx = hz;
        lx = lz;
    }

    // Convert back to floating value and restore the sign.
    if (hx as u32 | lx) == 0 {
        return d(FMOD_ZERO[(sx as u32 >> 31) as usize]);
    }
    while hx < 0x0010_0000 {
        // normalize x
        hx = hx.wrapping_add(hx).wrapping_add((lx >> 31) as i32);
        lx = lx.wrapping_add(lx);
        iy -= 1;
    }
    if iy >= -1022 {
        // normalize output
        hx = hx.wrapping_sub(0x0010_0000) | ((iy + 1023) << 20);
        d((((hx | sx) as u32 as u64) << 32) | lx as u64)
    } else {
        // subnormal output (the `x *= one` of the C source is not in the machine code)
        let n = -1022 - iy;
        if n <= 20 {
            lx = (lx >> n) | ((hx as u32).wrapping_shl((32 - n) as u32));
            hx >>= n;
        } else if n <= 31 {
            lx = ((hx as u32).wrapping_shl((32 - n) as u32)) | (lx >> n);
            hx = sx;
        } else {
            lx = (hx >> (n - 32)) as u32;
            hx = sx;
        }
        d((((hx | sx) as u32 as u64) << 32) | lx as u64)
    }
}

/// newlib `double fmod(double, double)` (w_fmod.c wrapper + `__ieee754_fmod`), shipped v1.71 at 0x08029600.
pub fn fmod(x: f64, y: f64) -> f64 {
    let z = __ieee754_fmod(x, y);
    if x.is_nan() || y.is_nan() {
        return z;
    }
    // `y == 0.0` is a vcmp.f64: under FZ a subnormal y compares equal to 0 (host DAZ/FZ does the same).
    if y == FMOD_WRAPPER_ZERO {
        // errno = EDOM; return 0.0/0.0 (default NaN)
        return d(DEFAULT_NAN_F64);
    }
    z
}

/// newlib `float sinf(float)`. Only used by `sinCos.c`, which is not linked in the firmware.
pub fn sinf(x: f32) -> f32 {
    todo!("port: sinf (not linked in the firmware; only needed if VerbTools/sinCos is ported)")
}
