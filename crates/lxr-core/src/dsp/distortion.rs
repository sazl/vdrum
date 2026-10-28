//! Port of `Core/Inc/Soundengine/DSPAudio/distortion.h`, `Core/Src/Soundengine/DSPAudio/distortion.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct DistStruct` (Core/Inc/Soundengine/DSPAudio/distortion.h:22), 24 bytes.
/// C typedef name `Distortion`; the struct tag `DistStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distortion {
    /// `float shape;`
    pub shape: f32,
    /// `float inv_shape;`
    pub inv_shape: f32,
    /// `OnePoleFilter DriveParamFilter;`
    pub DriveParamFilter: OnePoleFilter,
}
/// C struct tag of `Distortion`.
pub type DistStruct = Distortion;
impl Distortion {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { shape: 0.0, inv_shape: 0.0, DriveParamFilter: OnePoleFilter::ZERO };
}
impl Default for Distortion {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `IN_ITCM_RAM __inline void setDistortionShape(Distortion *dist, float shape)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/distortion.c:18`. Reference build: out-of-line in ITCM. Group P1.
pub fn setDistortionShape(dist: &mut Distortion, shape: f32) {
    let a = shape - 0.0078125;
    let b = f32::from_bits(0x3f810000) - shape; // 1 - (shape - 0.0078125) folded to 1.0078125 - shape
    dist.shape = (a + a) / b;
}

/// C: `IN_ITCM_RAM void calcDistBlock(const Distortion *dist, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/distortion.c:24`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcDistBlock(dist: &Distortion, buf: &mut [i16], size: u8) {
    let shape = dist.shape;
    let shape1 = shape + 1.0;
    for i in 0..size as usize {
        let x = buf[i] as i32 as f32 * f32::from_bits(0x38000100); // 1/32767.f
        let num = shape1 * x;
        let den = vfma(1.0, shape, x.abs());
        let y = num / den;
        buf[i] = ((y * 32767.0) as i32) as i16;
    }
}

/// C: `void calcDistBlockFiltered(Distortion *dist, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/distortion.c:36`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::one_pole_lp::calcOnePole`.
pub fn calcDistBlockFiltered(dist: &mut Distortion, buf: &mut [i16], size: u8) {
    for i in 0..size as usize {
        let shape = crate::dsp::one_pole_lp::calcOnePole(&mut dist.DriveParamFilter, dist.shape as f64);
        let x = buf[i] as i32 as f32 * f32::from_bits(0x38000100); // 1/32767.f
        let num = (shape + 1.0) * x;
        let den = vfma(1.0, x.abs(), shape);
        let y = num / den;
        buf[i] = ((y * 32767.0) as i32) as i16;
    }
}

/// C: `float distortion_calcSampleFloat(const Distortion *dist, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/distortion.c:54`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn distortion_calcSampleFloat(dist: &Distortion, x: f32) -> f32 {
    // not linked in the firmware (untestable): ported from C
    (1.0 + dist.shape) * x / (1.0 + dist.shape * x.abs())
}
