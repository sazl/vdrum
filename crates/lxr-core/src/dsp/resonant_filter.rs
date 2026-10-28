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
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:23")
}

/// C: `void SVF_init(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:29`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::resonant_filter::SVF_directSetFilterValue`.
pub fn SVF_init(filter: &mut ResonantFilter) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:29")
}

/// C: `void SVF_reset(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:48`. Reference build: out-of-line in flash. Group P1.
pub fn SVF_reset(filter: &mut ResonantFilter) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:48")
}

/// C: `IN_ITCM_RAM static float fastTanh(float var)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:60`. Reference build: inlined into its callers only. Group P1.
pub fn fastTanh(var: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:60")
}

/// C: `IN_ITCM_RAM float fastTan(float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:67`. Reference build: out-of-line in ITCM. Group P1.
pub fn fastTan(x: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:67")
}

/// C: `IN_ITCM_RAM void SVF_recalcFreq(ResonantFilter* filter)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:81`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTan`.
pub fn SVF_recalcFreq(filter: &mut ResonantFilter) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:81")
}

/// C: `IN_ITCM_RAM void SVF_setDrive(ResonantFilter* filter,float drive)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:89`. Reference build: out-of-line in ITCM. Group P1.
pub fn SVF_setDrive(filter: &mut ResonantFilter, drive: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:89")
}

/// C: `IN_ITCM_RAM void SVF_directSetFilterValue(ResonantFilter* filter, float val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:100`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTan`.
pub fn SVF_directSetFilterValue(filter: &mut ResonantFilter, val: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:100")
}

/// C: `IN_ITCM_RAM float tanhXdX(float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:108`. Reference build: out-of-line in ITCM. Group P1.
pub fn tanhXdX(x: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:108")
}

/// C: `IN_ITCM_RAM float softClipTwo(float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:116`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::tanhXdX`.
pub fn softClipTwo(r#in: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:116")
}

/// C: `float removeMaliciousSample(float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:130`. Reference build: out-of-line in flash. Group P1.
pub fn removeMaliciousSample(r#in: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:130")
}

/// C: `IN_ITCM_RAM void SVF_calcBlockZDF(ResonantFilter* filter, const uint8_t type, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:157`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTanh`, `dsp::resonant_filter::removeMaliciousSample`, `dsp::resonant_filter::softClipTwo`.
pub fn SVF_calcBlockZDF(filter: &mut ResonantFilter, r#type: u8, buf: &mut [i16], size: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:157")
}

/// C: `IN_ITCM_RAM void SVF_calcBlockCheapZDF(ResonantFilter* filter, const uint8_t type, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/ResonantFilter.c:378`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::resonant_filter::fastTanh`, `dsp::resonant_filter::softClipTwo`.
pub fn SVF_calcBlockCheapZDF(filter: &mut ResonantFilter, r#type: u8, buf: &mut [i16], size: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/ResonantFilter.c:378")
}
