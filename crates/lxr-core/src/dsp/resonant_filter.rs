//! Port of `Core/Inc/Soundengine/DSPAudio/ResonantFilter.h`, `Core/Src/Soundengine/DSPAudio/ResonantFilter.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define ENABLE_NONLINEAR_INTEGRATORS 0` (Core/Inc/Soundengine/DSPAudio/ResonantFilter.h:32)
pub const ENABLE_NONLINEAR_INTEGRATORS: i32 = 0;
/// `#define FILTER_GAIN 0x7fff` (Core/Inc/Soundengine/DSPAudio/ResonantFilter.h:33)
pub const FILTER_GAIN: i32 = 32767;
/// `#define USE_SHAPER_NONLINEARITY 0` (Core/Inc/Soundengine/DSPAudio/ResonantFilter.h:35)
pub const USE_SHAPER_NONLINEARITY: i32 = 0;
/// `#define USE_FMAF 0` (Core/Src/Soundengine/DSPAudio/ResonantFilter.c:155)
pub const USE_FMAF: i32 = 0;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Src/Soundengine/DSPAudio/ResonantFilter.c:128: #define is_nan(x) ((x) != (x))

// ---- enums
/// C: `enum filterTypeEnum` (Core/Inc/Soundengine/DSPAudio/ResonantFilter.h:39), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type filterTypeEnum = u8;
pub const FILTER_LP: filterTypeEnum = 1;
pub const FILTER_HP: filterTypeEnum = 2;
pub const FILTER_BP: filterTypeEnum = 3;
pub const FILTER_UNITY_BP: filterTypeEnum = 4;
pub const FILTER_NOTCH: filterTypeEnum = 5;
pub const FILTER_PEAK: filterTypeEnum = 6;
pub const FILTER_NAIVE_2_POLE: filterTypeEnum = 7;
pub const FILTER_OFF: filterTypeEnum = 8;

// ---- types
/// C: `struct ResoFilterStruct` (Core/Inc/Soundengine/DSPAudio/ResonantFilter.h:51), 32 bytes.
/// C typedef name `ResonantFilter`; the struct tag `ResoFilterStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResonantFilter {
    /// `float f; / **< cutoff frequency as integer from 0 to 0xffff where 0xffff = SR* /`
    pub f: f32,
    /// `float g; / **< embedded integrator gain (Fig 3.11), wc == wa* /`
    pub g: f32,
    /// `float q; / **< q value calculated from setReso()* /`
    pub q: f32,
    /// `float s1;`
    pub s1: f32,
    /// `float s2;`
    pub s2: f32,
    /// `float a,b;`
    pub a: f32,
    /// `float a,b;`
    pub b: f32,
    /// `float drive;`
    pub drive: f32,
}
/// C struct tag of `ResonantFilter`.
pub type ResoFilterStruct = ResonantFilter;
impl ResonantFilter {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { f: 0.0, g: 0.0, q: 0.0, s1: 0.0, s2: 0.0, a: 0.0, b: 0.0, drive: 0.0 };
}
impl Default for ResonantFilter {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `IN_ITCM_RAM void SVF_setReso(ResonantFilter* filter, float feedback)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:23`. Reference build: out-of-line in ITCM. Group P1.
pub fn SVF_setReso(filter: &mut ResonantFilter, feedback: f32) {
    // compiled as: feedback > 0.9f ? 0.02f : 1 - feedback
    if feedback > f32::from_bits(0x3f666666) {
        filter.q = f32::from_bits(0x3ca3d70a);
    } else {
        filter.q = 1.0 - feedback;
    }
}

/// C: `void SVF_init(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:29`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::resonant_filter::SVF_directSetFilterValue`.
pub fn SVF_init(filter: &mut ResonantFilter) {
    filter.s1 = 0.0;
    filter.s2 = 0.0;
    filter.a = 0.0;
    filter.b = 0.0;
    filter.q = f32::from_bits(0x3f666666); // 0.9f
    filter.drive = f32::from_bits(0x3ea147ae); // 0.315f
    // SVF_directSetFilterValue(filter, 0.25f), constant folded by GCC: values from the literal pool
    filter.f = f32::from_bits(0x3de66666);
    filter.g = f32::from_bits(0x3ebce2f3);
}

/// C: `void SVF_reset(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:48`. Reference build: out-of-line in flash. Group P1.
pub fn SVF_reset(filter: &mut ResonantFilter) {
    filter.s1 = 0.0;
    filter.s2 = 0.0;
    filter.a = 0.0;
    filter.b = 0.0;
}

/// C: `IN_ITCM_RAM static float fastTanh(float var)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:60`. Reference build: inlined into its callers only. Group P1.
pub fn fastTanh(var: f32) -> f32 {
    // as inlined in the LP output of SVF_calcBlock*: compares with -1.95f / 1.95f, then 4.15f*var/vfma(4.29f, var, var)
    if var < f32::from_bits(0xbff9999a) {
        -1.0
    } else if var > f32::from_bits(0x3ff9999a) {
        1.0
    } else {
        let den = vfma(f32::from_bits(0x408947ae), var, var);
        (var * f32::from_bits(0x4084cccd)) / den
    }
}

/// C: `IN_ITCM_RAM float fastTan(float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:67`. Reference build: out-of-line in ITCM. Group P1.
pub fn fastTan(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x * f32::from_bits(0x3eaaaaab); // x * (1/3.f)
    let b = vfma(-5.0, x2, 2.0);
    let a = (x2 - 15.0) * x3;
    a / b
}

/// C: `IN_ITCM_RAM void SVF_recalcFreq(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:81`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTan`.
pub fn SVF_recalcFreq(filter: &mut ResonantFilter) {
    // fastTan(M_PI * f): the product is computed in double and narrowed
    let arg = (filter.f as f64 * core::f64::consts::PI) as f32;
    filter.g = fastTan(arg);
}

/// C: `IN_ITCM_RAM void SVF_setDrive(ResonantFilter* filter,float drive)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:89`. Reference build: out-of-line in ITCM. Group P1.
pub fn SVF_setDrive(filter: &mut ResonantFilter, drive: f32) {
    filter.drive = vfma(f32::from_bits(0x3ecccccd), drive * drive, 6.0);
}

/// C: `IN_ITCM_RAM void SVF_directSetFilterValue(ResonantFilter* filter, float val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:100`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTan`.
pub fn SVF_directSetFilterValue(filter: &mut ResonantFilter, val: f32) {
    filter.f = val * f32::from_bits(0x3ee66666); // 0.5f*0.90f
    let arg = (filter.f as f64 * core::f64::consts::PI) as f32;
    filter.g = fastTan(arg);
}

/// C: `IN_ITCM_RAM float tanhXdX(float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:108`. Reference build: out-of-line in ITCM. Group P1.
pub fn tanhXdX(x: f32) -> f32 {
    let a = x * x;
    let den = vfma(945.0, vfma(420.0, a, 15.0), a);
    let num = vfma(945.0, a + 105.0, a);
    num / den
}

/// C: `IN_ITCM_RAM float softClipTwo(float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:116`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::tanhXdX`.
pub fn softClipTwo(r#in: f32) -> f32 {
    // 0.5*in is computed in single precision
    tanhXdX(r#in * 0.5) * r#in
}

/// C: `float removeMaliciousSample(float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:130`. Reference build: out-of-line in flash. Group P1.
pub fn removeMaliciousSample(r#in: f32) -> f32 {
    if r#in != r#in {
        0.0
    } else {
        r#in
    }
}

/// C: `IN_ITCM_RAM void SVF_calcBlockZDF(ResonantFilter* filter, const uint8_t type, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:157`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTanh`, `dsp::resonant_filter::removeMaliciousSample`, `dsp::resonant_filter::softClipTwo`.
pub fn SVF_calcBlockZDF(filter: &mut ResonantFilter, r#type: u8, buf: &mut [i16], size: u8) {
    SVF_calcBlock_impl(filter, r#type, buf, size, true);
}

/// C: `IN_ITCM_RAM void SVF_calcBlockCheapZDF(ResonantFilter* filter, const uint8_t type, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:378`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTanh`, `dsp::resonant_filter::softClipTwo`.
pub fn SVF_calcBlockCheapZDF(filter: &mut ResonantFilter, r#type: u8, buf: &mut [i16], size: u8) {
    SVF_calcBlock_impl(filter, r#type, buf, size, false);
}

/// `__SSAT(vcvt.s32.f32(v), 16)`: saturating float to int conversion, then saturation to 16 bits.
#[inline(always)]
fn ssat16_f(v: f32) -> i16 {
    (v as i32).clamp(-32768, 32767) as i16
}

/// Shared body of `SVF_calcBlockZDF` and `SVF_calcBlockCheapZDF`. GCC compiles both to the same sequence of
/// floating point operations (ITCM disassembly); the only difference is that the ZDF version first replaces
/// NaN states with 0 (`removeMaliciousSample`).
///
/// Notes from the machine code:
/// * `R = f >= 0.4499f ? 1 : q` is `!(f < 0.4499f)`: a NaN `f` selects 1.
/// * the input scaling `(buf/32767.f)*drive` is reassociated to `buf * (drive * (1/32767.f))`.
/// * `softClipTwo(x) - a` (naive) and `softClipTwo(s1) + ...` (ZDF) fuse the final multiplication of
///   `softClipTwo` into the following `vfma`/`vfnms`.
/// * `1/(f1+1)` is computed once per block and multiplied.
/// * for an unknown type (0, 9..255) the first sample's state is stored and the function returns
///   without writing the buffer.
fn SVF_calcBlock_impl(filter: &mut ResonantFilter, r#type: u8, buf: &mut [i16], size: u8, remove_nan: bool) {
    const INV_32767: f32 = f32::from_bits(0x38000100);
    const GAIN: f32 = f32::from_bits(0x46fffe00); // 32767.f
    let fq = filter.f;
    let g = filter.g;
    let R = if fq < f32::from_bits(0x3ee6594b) { filter.q } else { 1.0 };

    if r#type == FILTER_OFF {
        return;
    }
    if r#type == FILTER_NAIVE_2_POLE {
        let f_lp2 = fq * f32::from_bits(0x400d70a4); // 2.21f
        if size == 0 {
            return;
        }
        let one_minus_q = 1.0 - filter.q;
        let d9 = one_minus_q as f64;
        let d3 = d9 / (1.0 - f_lp2 as f64);
        let q = fpu::vfma64(d3, d9, f64::from_bits(0x3ff6666666666666)) as f32; // (1-q)*1.4 + (1-q)/(1-f_lp2)
        let dk = filter.drive * INV_32767;
        let mut a = filter.a;
        let mut b = filter.b;
        for i in 0..size as usize {
            let xin = buf[i] as i32 as f32 * dk;
            let ab = a - b;
            let h = xin * 0.5;
            let h2 = h * h;
            let den = vfma(945.0, vfma(420.0, h2, 15.0), h2);
            let num = vfma(945.0, h2 + 105.0, h2);
            let t = num / den;
            // (x - a) with x = xin * t fused
            let mut tmp = vfnms(a, xin, t);
            tmp = vfma(tmp, ab, q);
            a = vfma(a, tmp, f_lp2);
            if a > 1.0 {
                a = 1.0;
            } else if a < -1.0 {
                a = -1.0;
            }
            filter.a = a;
            b = vfma(b, a - b, f_lp2);
            let out;
            if b > 1.0 {
                b = 1.0;
                out = 32767;
            } else if b < -1.0 {
                b = -1.0;
                out = -32767;
            } else {
                out = ssat16_f(b * GAIN);
            }
            filter.b = b;
            buf[i] = out;
        }
        return;
    }

    if remove_nan {
        filter.s1 = removeMaliciousSample(filter.s1);
        filter.s2 = removeMaliciousSample(filter.s2);
    }
    if size == 0 {
        return;
    }
    let g2 = g + g;
    let ff = g * g;
    let dk = filter.drive * INV_32767;
    let R2 = R + R;
    let g0 = 1.0 / vfma(1.0, R, g2);
    let f1 = ff * g0;
    let gg0 = g * g0;
    let inv_f1p1 = 1.0 / (f1 + 1.0);
    let mut s1 = filter.s1;
    let mut s2 = filter.s2;
    for i in 0..size as usize {
        let xin = buf[i] as i32 as f32 * dk;
        // tanhXdX(0.5*xin) and tanhXdX(0.5*s1)
        let hx = xin * 0.5;
        let hx = hx * hx;
        let hs = s1 * 0.5;
        let hs = hs * hs;
        let tx = vfma(945.0, hx, hx + 105.0) / vfma(945.0, hx, vfma(420.0, hx, 15.0));
        let ts = vfma(945.0, hs + 105.0, hs) / vfma(945.0, vfma(420.0, hs, 15.0), hs);
        let x = tx * xin;
        let y1 = vfma(vfma(s2, f1, x), gg0, s1) * inv_f1p1;
        let xx = x - y1;
        let y0 = vfma(g * xx, ts, s1) * g0; // (softClipTwo(s1) + f*xx) * g0
        let ugb = y0 * R2;
        s2 = vfma(s2, y0, g2);
        let hp = xx - ugb;
        s1 = vfma(hp * g2, ts, s1); // softClipTwo(s1) + 2f*(xx - 2R*y0)
        let out = match r#type {
            FILTER_LP => {
                if y1 < f32::from_bits(0xbff9999a) {
                    -32767
                } else if y1 > f32::from_bits(0x3ff9999a) {
                    32767
                } else {
                    let den = vfma(f32::from_bits(0x408947ae), y1, y1);
                    ssat16_f(((y1 * f32::from_bits(0x4084cccd)) / den) * GAIN)
                }
            }
            FILTER_HP => ssat16_f(hp * GAIN),
            FILTER_BP => ssat16_f(y0 * GAIN),
            FILTER_UNITY_BP => ssat16_f(ugb * GAIN),
            FILTER_NOTCH => ssat16_f((x - ugb) * GAIN),
            FILTER_PEAK => ssat16_f((ugb + vfnms(x, y1, 2.0)) * GAIN),
            _ => {
                filter.s1 = s1;
                filter.s2 = s2;
                return;
            }
        };
        buf[i] = out;
    }
    filter.s1 = s1;
    filter.s2 = s2;
}
