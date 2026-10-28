//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Delay.h`, `Core/Src/Soundengine/DSPAudio/FX/Delay.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define LOW_MIN SECONDS_TO_SAMPLES(0.0006)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:35) = 26.483999999999998
pub const LOW_MIN: f64 = f64::from_bits(0x403a7be76c8b4395);
/// `#define LOW_MAX SECONDS_TO_SAMPLES(0.02)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:36) = 882.8000000000001
pub const LOW_MAX: f64 = f64::from_bits(0x408b966666666667);
/// `#define HI_MIN SECONDS_TO_SAMPLES(0.02)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:38) = 882.8000000000001
pub const HI_MIN: f64 = f64::from_bits(0x408b966666666667);
/// `#define HI_MAX FX_BUFFER_SIZE` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:39)
pub const HI_MAX: i32 = 32767;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/Soundengine/DSPAudio/FX/Delay.h:33: #define SECONDS_TO_SAMPLES(s) (s*REAL_FS)

// ---- enums
/// C: `enum DelayTypesEnum` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:41), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type DelayTypesEnum = u8;
pub const DELAY_NORMAL: DelayTypesEnum = 0;
pub const DELAY_PINGPONG: DelayTypesEnum = 1;

// ---- types
/// C: `struct DelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:47), 96 bytes.
/// C typedef name `Delay`; the struct tag `DelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delay {
    /// `uint32_t oldRead;`
    pub oldRead: u32,
    /// `uint32_t newRead;`
    pub newRead: u32,
    /// `uint32_t write;`
    pub write: u32,
    /// `float fracReadNew,fracReadOld;`
    pub fracReadNew: f32,
    /// `float fracReadNew,fracReadOld;`
    pub fracReadOld: f32,
    /// `float interpValue;`
    pub interpValue: f32,
    /// `float interpStep;`
    pub interpStep: f32,
    /// `int16_t tmpBuffer[2];`
    pub tmpBuffer: [i16; 2],
    /// `int16_t tmpBuffer2[2]; //used for stereo or pingpong delay`
    pub tmpBuffer2: [i16; 2],
    /// `float feedback;`
    pub feedback: f32,
    /// `float range;`
    pub range: f32,
    /// `float delayTime;`
    pub delayTime: f32,
    /// `float pendingDelayTime;`
    pub pendingDelayTime: f32,
    /// `uint8_t timePending;`
    pub timePending: u8,
    /// `OnePoleFilter tone;`
    pub tone: OnePoleFilter,
    /// `OnePoleFilter tone2;`
    pub tone2: OnePoleFilter,
    /// `uint8_t delayType;`
    pub delayType: u8,
}
/// C struct tag of `Delay`.
pub type DelayStruct = Delay;
impl Delay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { oldRead: 0, newRead: 0, write: 0, fracReadNew: 0.0, fracReadOld: 0.0, interpValue: 0.0, interpStep: 0.0, tmpBuffer: [0; 2], tmpBuffer2: [0; 2], feedback: 0.0, range: 0.0, delayTime: 0.0, pendingDelayTime: 0.0, timePending: 0, tone: OnePoleFilter::ZERO, tone2: OnePoleFilter::ZERO, delayType: 0 };
}
impl Default for Delay {
    fn default() -> Self {
        Self::ZERO
    }
}

use super::{ld32, pkhbt, ssat16, st32, sx};
use crate::cmsis::__QADD16;
use crate::dsp::one_pole_lp::{calcOnePole, initOnePole, setOnePoleCoef};
use crate::fpu::{f16_bits_to_f32, f32_to_f16_bits};

/// `1/32767.f` as GCC folded it (`itg/32767.f` → `itg * 0x38000100`).
const INV_32767: f32 = f32::from_bits(0x3800_0100);
/// `HI_MAX` as float (32767.0).
const HI_MAX_F: f32 = f32::from_bits(0x46ff_fe00);
/// `HI_MAX*0.48f` (15728.16).
const HI_MAX_PP: f32 = f32::from_bits(0x4675_c0a3);
/// `(float)LOW_MAX` = `(float)HI_MIN` (882.8).
const LOW_MAX_F: f32 = f32::from_bits(0x445c_b333);
/// `(float)LOW_MIN` (26.484).
const LOW_MIN_F: f32 = f32::from_bits(0x41d3_df3b);
/// `1.0f / (REAL_FS * 0.01f)`.
const INTERP_STEP: f32 = f32::from_bits(0x3b14_7919);
/// `(__fp16)-1.f`: GCC replaces the conversion of the clamped minimum (`itg == -32768`) by this constant.
const F16_MINUS_ONE: u16 = 0xbc00;

// ---- functions without access to globals
/// C: `void Delay_init(Delay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:18`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::one_pole_lp::initOnePole`.
pub fn Delay_init(d: &mut Delay) {
    d.timePending = 0;
    d.pendingDelayTime = 0.0;
    d.oldRead = 0;
    d.newRead = 0;
    d.fracReadOld = 0.0;
    d.fracReadNew = 0.0;
    d.write = (FX_BUFFER_SIZE / 20 - 10) as u32;
    d.feedback = f32::from_bits(0x3f33_3333);
    d.interpValue = 0.0;
    d.interpStep = 0.0;
    initOnePole(&mut d.tone);
    initOnePole(&mut d.tone2);
}

/// C: `float interpolate_s16(float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:40`. Reference build: inlined into its callers only. Group P2.
/// Inlined form: `vmul b*x; vfma (+= a*(1-x))`.
pub fn interpolate_s16(a: f32, b: f32, x: f32) -> f32 {
    vfma(b * x, a, 1.0 - x)
}

/// C: `float interpolate(float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:272`. Reference build: inlined into its callers only. Group P2.
/// Inlined form (in `calcDelayTime`): `vmul b*x; vfma (+= a*(1-x))`.
pub fn interpolate(a: f32, b: f32, x: f32) -> f32 {
    vfma(b * x, a, 1.0 - x)
}

/// C: `float calcDelayTime(Delay* d, float timeCv)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:276`. Reference build: inlined into its callers only. Group P2.
/// Calls: `dsp::fx::delay::interpolate`.
pub fn calcDelayTime(d: &mut Delay, timeCv: f32) -> f32 {
    let range = d.range;
    let hi = if d.delayType == DELAY_NORMAL { HI_MAX_F } else { HI_MAX_PP };
    let one_m_range = 1.0 - range;
    let max = vfma(range * hi, one_m_range, LOW_MAX_F);
    let min = vfma(range * LOW_MAX_F, one_m_range, LOW_MIN_F);
    vfma(timeCv * max, min, 1.0 - timeCv)
}

/// C: `void Delay_setDelayTime(Delay* d, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:293`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::fx::delay::calcDelayTime`.
pub fn Delay_setDelayTime(d: &mut Delay, t: f32) {
    // vcmpe interpValue, #0; bls: ordered and <= 0 -> set, otherwise (> 0 or NaN) pending
    if d.interpValue <= 0.0 {
        d.delayTime = t;
        let offset = calcDelayTime(d, t * d.range);
        let offItg = offset as u32; // vcvt.u32.f32 (truncating, saturating)
        d.fracReadNew = offset - offItg as f32;
        let nr = d.write.wrapping_sub(1).wrapping_sub(offItg);
        d.newRead = if d.delayType == DELAY_NORMAL { nr & FX_BUFFER_SIZE as u32 } else { nr & (FX_BUFFER_SIZE >> 1) as u32 };
        d.interpValue = 1.0;
        d.interpStep = INTERP_STEP;
    } else {
        d.pendingDelayTime = t;
        d.timePending = 1;
    }
}

/// C: `void Delay_setFeedback(Delay* d, float fb)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:349`. Reference build: out-of-line in flash. Group P2.
pub fn Delay_setFeedback(d: &mut Delay, fb: f32) {
    d.feedback = fb;
}

/// C: `void Delay_setRange(Delay* d, float r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:353`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::fx::delay::Delay_setDelayTime`.
pub fn Delay_setRange(d: &mut Delay, r: f32) {
    d.range = r;
    let t = d.delayTime;
    Delay_setDelayTime(d, t);
}

/// C: `void Delay_setFilter(Delay* d, float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:359`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::one_pole_lp::setOnePoleCoef`.
pub fn Delay_setFilter(d: &mut Delay, f: f32) {
    let ff = (f * f) as f64;
    setOnePoleCoef(&mut d.tone, ff);
    setOnePoleCoef(&mut d.tone2, ff);
}

/// `FX_BUFFER[i]` for the delay line reads. Out of range only in one firmware situation: after
/// `fxBus.delay.delayType` changes from normal to ping-pong (MIDI `CC2_FX_DELAY_TYPE` writes it directly) while a
/// read pointer is above `FX_BUFFER_SIZE>>1`, the first ping-pong read of that pointer is
/// `FX_BUFFER[(FX_BUFFER_SIZE>>1) + read]`, past the end of `FX_BUFFER` (C out-of-bounds read of the DTCM globals
/// that follow it). That memory is not modelled: such reads give +0.0 here (NOT bit-exact; see PORTING_STATUS).
#[inline(always)]
fn fx_at(buf: &[u16; 32768], i: u32) -> u16 {
    buf.get(i as usize).copied().unwrap_or(0)
}

/// One interpolated delay line read (`interpolate_s16(...)*32767.f` stored to an `int16_t`).
#[inline(always)]
fn read_interp(buf: &[u16; 32768], i0: u32, i1: u32, frac: f32) -> i16 {
    let a = f16_bits_to_f32(fx_at(buf, i0));
    let b = f16_bits_to_f32(fx_at(buf, i1));
    ((interpolate_s16(a, b, frac) * 32767.0) as i32) as i16
}

/// Crossfade of the two read pointers: `tmp[0] * interpValue + tmp[1] * (1.0 - interpValue)`.
/// GCC: `(double)((float)tmp0 * iv)` + fused `(double)tmp1 * (1.0 - (double)iv)`, `vcvt.s32.f64`, `sxth`.
#[inline(always)]
fn crossfade(t0: i16, t1: i16, iv: f32) -> i16 {
    let p = (t0 as f32 * iv) as f64;
    (fpu::vfma64(p, t1 as f64, 1.0 - iv as f64) as i32) as i16
}

/// Delay line input sample: `(__fp16)clamp(itg/32767.f)` as compiled.
#[inline(always)]
fn to_line(itg: i16) -> u16 {
    if itg == -32768 {
        F16_MINUS_ONE
    } else {
        f32_to_f16_bits(itg as f32 * INV_32767)
    }
}

/// Dry/wet mix of one stereo pair as compiled:
/// `__PKHBT(__SSAT(fx1 + __SSAT((wet - fx1) * dryWet,16),16), __SSAT(fx2 + __SSAT((wet2 - fx2) * dryWet,16),16), 16)`.
#[inline(always)]
fn mix_sat(fx1: i16, fx2: i16, wet: i16, wet2: i16, dryWet: f32) -> u32 {
    let a = ssat16((((wet as i32 - fx1 as i32) as f32) * dryWet) as i32);
    let b = ssat16((((wet2 as i32 - fx2 as i32) as f32) * dryWet) as i32);
    pkhbt(ssat16(fx1 as i32 + a), ssat16(fx2 as i32 + b))
}

/// Adds the packed pair `x` to the output (stereo: both halves; mono: low half only, `strh`).
#[inline(always)]
fn add_out(out: &mut [i16], k: usize, x: u32, channels: u8) {
    if channels == 2 {
        let w = __QADD16(ld32(out, k), x);
        st32(out, k, w);
    } else {
        // only the low halfword of the 32 bit load is stored; it depends only on out[k] and x's low half
        out[k] = __QADD16(out[k] as u16 as u32, x) as i16;
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Delay_calcBufferPingPong(Delay* d, uint8_t bufferIdx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:45`. Reference build: inlined into its callers only. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `dsp::fx::delay::interpolate_s16`.
    pub fn Delay_calcBufferPingPong(&mut self, d: &mut Delay, bufferIdx: u8) {
        let hs = (FX_BUFFER_SIZE >> 1) as u32;
        let fb = &*self.FX_BUFFER;
        if bufferIdx == 0 {
            let r = d.oldRead;
            let pos2 = (r + 1) & hs;
            d.tmpBuffer[0] = read_interp(fb, r, pos2, d.fracReadOld);
            d.tmpBuffer2[0] = read_interp(fb, hs + r, hs + pos2, d.fracReadOld);
            d.oldRead = pos2;
        } else {
            let r = d.newRead;
            let pos2 = (r + 1) & hs;
            d.tmpBuffer[1] = read_interp(fb, r, pos2, d.fracReadNew);
            d.tmpBuffer2[1] = read_interp(fb, hs + r, hs + pos2, d.fracReadNew);
            d.newRead = pos2;
        }
    }

    /// C: `void Delay_calcBuffer(Delay* d, uint8_t bufferIdx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:71`. Reference build: inlined into its callers only. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `dsp::fx::delay::interpolate_s16`.
    pub fn Delay_calcBuffer(&mut self, d: &mut Delay, bufferIdx: u8) {
        let mask = FX_BUFFER_SIZE as u32;
        let fb = &*self.FX_BUFFER;
        if bufferIdx == 0 {
            let r = d.oldRead;
            let pos2 = (r + 1) & mask;
            d.tmpBuffer[0] = read_interp(fb, r, pos2, d.fracReadOld);
            d.oldRead = pos2;
        } else {
            let r = d.newRead;
            let pos2 = (r + 1) & mask;
            d.tmpBuffer[1] = read_interp(fb, r, pos2, d.fracReadNew);
            d.newRead = pos2;
        }
    }

    /// C: `void Delay_processPingPong(Delay *d, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:93`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `Lxr::Delay_calcBufferPingPong`, `dsp::fx::delay::Delay_setDelayTime`, `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`.
    ///
    /// `outL` is the interleaved output starting at the C `outL` pointer (the C code writes through
    /// `(int32_t*)outL` only; `outR` is not used and therefore not a parameter).
    pub fn Delay_processPingPong(&mut self, d: &mut Delay, fxBuffer: &[i16], outL: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        let hs = (FX_BUFFER_SIZE >> 1) as u32;
        let mut i: u8 = 0;
        let mut k: usize = 0; // stereoOut position (in int16 units)
        if size == 0 {
            return;
        }
        loop {
            let idx1 = i.wrapping_mul(2) as usize;
            let idx2 = i.wrapping_mul(2).wrapping_add(1) as usize;
            let (mut oL, mut oR): (i16, i16);
            if d.interpValue <= 0.0 {
                self.Delay_calcBufferPingPong(d, 0);
                oL = d.tmpBuffer[0];
                oR = d.tmpBuffer2[0];
                if d.timePending != 0 {
                    d.timePending = 0;
                    let t = d.pendingDelayTime;
                    Delay_setDelayTime(d, t);
                }
            } else {
                self.Delay_calcBufferPingPong(d, 0);
                self.Delay_calcBufferPingPong(d, 1);
                let iv = d.interpValue;
                oL = crossfade(d.tmpBuffer[0], d.tmpBuffer[1], iv);
                oR = crossfade(d.tmpBuffer2[0], d.tmpBuffer2[1], iv);
                d.interpValue = iv - d.interpStep;
                if d.interpValue <= 0.0 {
                    d.oldRead = d.newRead;
                    d.fracReadOld = d.fracReadNew;
                }
            }
            oL = (calcOnePole(&mut d.tone, oL as f64) as i32) as i16;
            oR = (calcOnePole(&mut d.tone2, oR as f64) as i32) as i16;

            let fx1 = fxBuffer[idx1];
            let fx2 = fxBuffer[idx2];
            let fbL = ((oL as f32 * d.feedback) as i32) as i16;
            let itgL = __QADD16(sx(fx1), sx(fbL)) as i16;
            let hL = to_line(itgL);
            let fbR = ((oR as f32 * d.feedback) as i32) as i16;
            let itgR = __QADD16(sx(fx2), sx(fbR)) as i16;
            let hR = to_line(itgR);
            let w = d.write;
            // out of range only for the first sample after a normal -> ping-pong switch with write > hs (C writes
            // past FX_BUFFER into the following DTCM globals; not modelled, the store is dropped: NOT bit-exact)
            if let Some(x) = self.FX_BUFFER.get_mut((hs + w) as usize) {
                *x = hL;
            }
            self.FX_BUFFER[w as usize] = hR;
            d.write = (w + 1) & hs;

            let x = mix_sat(fx1, fx2, oL, oR, dryWet);
            add_out(outL, k, x, channels);
            k += 2;

            i = i.wrapping_add(1);
            if i as u16 >= size {
                break;
            }
        }
    }

    /// C: `void Delay_process(Delay *d, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:187`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `Lxr::Delay_calcBuffer`, `dsp::fx::delay::Delay_setDelayTime`, `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`.
    ///
    /// `outL` is the interleaved output starting at the C `outL` pointer (`outR` is unused in C).
    pub fn Delay_process(&mut self, d: &mut Delay, fxBuffer: &[i16], outL: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        let mask = FX_BUFFER_SIZE as u32;
        let mut i: u8 = 0;
        let mut k: usize = 0;
        if size == 0 {
            return;
        }
        loop {
            let idx1 = i.wrapping_mul(2) as usize;
            let idx2 = i.wrapping_mul(2).wrapping_add(1) as usize;
            let mut out: i16;
            if d.interpValue <= 0.0 {
                self.Delay_calcBuffer(d, 0);
                out = d.tmpBuffer[0];
                if d.timePending != 0 {
                    d.timePending = 0;
                    let t = d.pendingDelayTime;
                    Delay_setDelayTime(d, t);
                }
            } else {
                self.Delay_calcBuffer(d, 0);
                self.Delay_calcBuffer(d, 1);
                let iv = d.interpValue;
                out = crossfade(d.tmpBuffer[0], d.tmpBuffer[1], iv);
                d.interpValue = iv - d.interpStep;
                if d.interpValue <= 0.0 {
                    d.oldRead = d.newRead;
                    d.fracReadOld = d.fracReadNew;
                }
            }
            out = (calcOnePole(&mut d.tone, out as f64) as i32) as i16;

            let fx1 = fxBuffer[idx1];
            let fx2 = fxBuffer[idx2];
            let sum = __QADD16(sx(fx1), sx(fx2));
            let fbk = ((out as f32 * d.feedback) as i32) as i16;
            let itg = __QADD16(sum, sx(fbk)) as i16;
            let w = d.write;
            self.FX_BUFFER[w as usize] = to_line(itg);
            d.write = (w + 1) & mask;

            let x = mix_sat(fx1, fx2, out, out, dryWet);
            add_out(outL, k, x, channels);
            k += 2;

            i = i.wrapping_add(1);
            if i as u16 >= size {
                break;
            }
        }
    }
}
