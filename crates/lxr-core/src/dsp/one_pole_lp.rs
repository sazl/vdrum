//! Port of `Core/Inc/Soundengine/DSPAudio/1PoleLp.h`, `Core/Src/Soundengine/DSPAudio/1PoleLp.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/Soundengine/DSPAudio/1PoleLp.h:20: #define CALC_FILTER_FREQ(f) (1-expf(-2.f*M_PI*f/(float)REAL_FS))

// ---- types
/// C: `struct OnePoleStruct` (Core/Inc/Soundengine/DSPAudio/1PoleLp.h:22), 16 bytes.
/// C typedef name `OnePoleFilter`; the struct tag `OnePoleStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnePoleFilter {
    /// `double r;`
    pub r: f64,
    /// `double b;`
    pub b: f64,
}
/// C struct tag of `OnePoleFilter`.
pub type OnePoleStruct = OnePoleFilter;
impl OnePoleFilter {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { r: 0.0, b: 0.0 };
}
impl Default for OnePoleFilter {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void initOnePole(OnePoleFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:21`. Reference build: out-of-line in flash. Group P1.
pub fn initOnePole(filter: &mut OnePoleFilter) {
    // literal pool: b = 0.0, r = (double)0.01f
    filter.b = 0.0;
    filter.r = f64::from_bits(0x3f847ae1_40000000);
}

/// C: `IN_ITCM_RAM float calcOnePole(OnePoleFilter* filter, double input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:27`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcOnePole(filter: &mut OnePoleFilter, input: f64) -> f32 {
    // vsub.f64 (input - b); r, b and the difference narrowed to f32; one vfma.f32 (fmaf); result widened back
    let d = input - filter.b;
    let r = filter.r as f32;
    let b = filter.b as f32;
    let res = vfma(b, r, d as f32);
    filter.b = res as f64;
    res
}

/// C: `float calcOnePoleAdaptiveVolume(OnePoleFilter* filter, double input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:35`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcOnePoleAdaptiveVolume(filter: &mut OnePoleFilter, input: f64) -> f32 {
    // not linked in the firmware (untestable): ported from C, double arithmetic as written
    let diff = (input - filter.b).abs();
    if diff > 0.5 || input < 0.01 {
        filter.b += 0.5 * (input - filter.b);
    } else if diff > 0.05f32 as f64 {
        filter.b += 0.01f32 as f64 * (input - filter.b);
    } else {
        filter.b += 0.001f32 as f64 * (input - filter.b);
    }
    filter.b as f32
}

/// C: `void calcOnePoleBlock(OnePoleFilter* filter, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:52`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcOnePoleBlock(filter: &mut OnePoleFilter, buf: &mut [i16], size: u8) {
    // not linked in the firmware (untestable): ported from C
    for i in 0..size as usize {
        filter.b += filter.r * (buf[i] as f64 - filter.b);
        buf[i] = filter.b as i32 as i16;
    }
}

/// C: `void calcOnePoleBlockFixedInput(OnePoleFilter* filter, float input,float* output, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:62`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcOnePoleBlockFixedInput(filter: &mut OnePoleFilter, input: f32, output: &mut [f32], size: u8) {
    // not linked in the firmware (untestable): ported from C
    for i in 0..size as usize {
        filter.b += filter.r * (input as f64 - filter.b);
        output[i] = filter.b as f32;
    }
}

/// C: `void setOnePoleCoef(OnePoleFilter* filter,double val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:72`. Reference build: out-of-line in flash. Group P1.
pub fn setOnePoleCoef(filter: &mut OnePoleFilter, val: f64) {
    filter.r = val;
}

/// C: `void setOnePoleValue(OnePoleFilter* filter,double val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:77`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn setOnePoleValue(filter: &mut OnePoleFilter, val: f64) {
    // not linked in the firmware (untestable): ported from C
    filter.b = val;
}

/// C: `void setOnePoleFreq(OnePoleFilter* filter,double f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/1PoleLp.c:82`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn setOnePoleFreq(filter: &mut OnePoleFilter, f: f64) {
    // not linked in the firmware (untestable): ported from C.
    // CALC_FILTER_FREQ(f) = 1-expf(-2.f*M_PI*f/(float)REAL_FS): the argument is computed in double (M_PI)
    let arg = -2.0f64 * core::f64::consts::PI * f / REAL_FS as f64;
    filter.r = 1.0 - libm::expf(arg as f32) as f64;
}
